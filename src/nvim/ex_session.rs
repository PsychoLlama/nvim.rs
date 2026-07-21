use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn alist_name(aep: *mut aentry_T) -> *mut ::core::ffi::c_char;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn bt_help(buf: *const buf_T) -> bool;
    fn bt_normal(buf: *const buf_T) -> bool;
    fn bt_terminal(buf: *const buf_T) -> bool;
    fn bt_nofilename(buf: *const buf_T) -> bool;
    static e_noname: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    static e_prev_dir: [::core::ffi::c_char; 0];
    static e_write: [::core::ffi::c_char; 0];
    fn var_flavour(varname: *mut ::core::ffi::c_char) -> var_flavour_T;
    static hash_removed: ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn get_globvar_dict() -> *mut dict_T;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn vim_mkdir_emsg(
        name: *const ::core::ffi::c_char,
        prot: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn open_exfile(
        fname: *mut ::core::ffi::c_char,
        forceit: ::core::ffi::c_int,
        mode: *mut ::core::ffi::c_char,
    ) -> *mut FILE;
    fn vim_strsave_fnameescape(
        fname: *const ::core::ffi::c_char,
        what: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_chdirfile(fname: *mut ::core::ffi::c_char, cause: CdCause) -> ::core::ffi::c_int;
    fn shorten_fnames(force: ::core::ffi::c_int);
    fn put_folds(fd: *mut FILE, wp: *mut win_T) -> ::core::ffi::c_int;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static topframe: GlobalCell<*mut frame_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static firstbuf: GlobalCell<*mut buf_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static global_alist: GlobalCell<alist_T>;
    static globaldir: GlobalCell<*mut ::core::ffi::c_char>;
    static no_hlsearch: GlobalCell<bool>;
    fn makemap(fd: *mut FILE, buf: *mut buf_T) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn makeset(
        fd: *mut FILE,
        opt_flags: ::core::ffi::c_int,
        local_only: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn makefoldset(fd: *mut FILE) -> ::core::ffi::c_int;
    static p_acd: GlobalCell<::core::ffi::c_int>;
    static p_hls: GlobalCell<::core::ffi::c_int>;
    static p_stal: GlobalCell<OptInt>;
    static ssop_flags: GlobalCell<::core::ffi::c_uint>;
    static p_shm: GlobalCell<*mut ::core::ffi::c_char>;
    static p_vdir: GlobalCell<*mut ::core::ffi::c_char>;
    static vop_flags: GlobalCell<::core::ffi::c_uint>;
    static p_wh: GlobalCell<OptInt>;
    static p_wiw: GlobalCell<OptInt>;
    fn os_chdir(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn home_replace_save(
        buf: *mut buf_T,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn vim_FullName(
        fname: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
        force: bool,
    ) -> ::core::ffi::c_int;
    fn do_source(
        fname: *mut ::core::ffi::c_char,
        check_other: bool,
        is_vimrc: ::core::ffi::c_int,
        ret_sid: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type CdCause = ::core::ffi::c_int;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub const kCdCauseOther: CdCause = -1;
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
    pub cs_pend: C2Rust_Unnamed_13,
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
pub union C2Rust_Unnamed_13 {
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
pub type var_flavour_T = ::core::ffi::c_uint;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const VSE_BUFFER: C2Rust_Unnamed_14 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_14 = 1;
pub const VSE_NONE: C2Rust_Unnamed_14 = 0;
pub const DOSO_NONE: C2Rust_Unnamed_17 = 0;
pub const kOptSsopFlagCurdir: C2Rust_Unnamed_16 = 4096;
pub const kOptSsopFlagCursor: C2Rust_Unnamed_16 = 16384;
pub const kOptSsopFlagFolds: C2Rust_Unnamed_16 = 8192;
pub const kOptSsopFlagOptions: C2Rust_Unnamed_16 = 32;
pub const OPT_LOCAL: C2Rust_Unnamed_15 = 2;
pub const kOptSsopFlagLocaloptions: C2Rust_Unnamed_16 = 16;
pub const kOptSsopFlagSesdir: C2Rust_Unnamed_16 = 2048;
pub const kOptSsopFlagTerminal: C2Rust_Unnamed_16 = 65536;
pub const kOptSsopFlagTabpages: C2Rust_Unnamed_16 = 32768;
pub const kOptSsopFlagHelp: C2Rust_Unnamed_16 = 64;
pub const kOptSsopFlagBlank: C2Rust_Unnamed_16 = 128;
pub const kOptSsopFlagWinsize: C2Rust_Unnamed_16 = 8;
pub const kOptSsopFlagResize: C2Rust_Unnamed_16 = 4;
pub const kOptSsopFlagGlobals: C2Rust_Unnamed_16 = 256;
pub const kOptSsopFlagBuffers: C2Rust_Unnamed_16 = 1;
pub const OPT_GLOBAL: C2Rust_Unnamed_15 = 1;
pub const OPT_SKIPRTP: C2Rust_Unnamed_15 = 128;
pub const kOptSsopFlagSkiprtp: C2Rust_Unnamed_16 = 131072;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_15 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_15 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_15 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_15 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_15 = 4;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptSsopFlagUnix: C2Rust_Unnamed_16 = 1024;
pub const kOptSsopFlagSlash: C2Rust_Unnamed_16 = 512;
pub const kOptSsopFlagWinpos: C2Rust_Unnamed_16 = 2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DOSO_VIMRC: C2Rust_Unnamed_17 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const SESSION_FILE: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Session.vim\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static did_lcd: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
unsafe extern "C" fn put_view_curpos(
    mut fd: *mut FILE,
    mut wp: *const win_T,
    mut spaces: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    if (*wp).w_curswant == MAXCOL as ::core::ffi::c_int {
        r = fprintf(
            fd,
            b"%snormal! $\n\0".as_ptr() as *const ::core::ffi::c_char,
            spaces,
        );
    } else {
        r = fprintf(
            fd,
            b"%snormal! 0%d|\n\0".as_ptr() as *const ::core::ffi::c_char,
            spaces,
            (*wp).w_virtcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
        );
    }
    return (r >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn ses_winsizes(
    mut fd: *mut FILE,
    mut restore_size: bool,
    mut tab_firstwin: *mut win_T,
) -> ::core::ffi::c_int {
    if restore_size as ::core::ffi::c_int != 0
        && ssop_flags.get() & kOptSsopFlagWinsize as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = tab_firstwin;
        while !wp.is_null() {
            if ses_do_win(wp) != 0 {
                n += 1;
                if (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height
                    < (*topframe.get()).fr_height
                    && fprintf(
                        fd,
                        b"exe '%dresize ' . ((&lines * %ld + %ld) / %ld)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        n,
                        (*wp).w_height as int64_t,
                        Rows.get() as int64_t / 2 as int64_t,
                        Rows.get() as int64_t,
                    ) < 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                if (*wp).w_width < Columns.get()
                    && fprintf(
                        fd,
                        b"exe 'vert %dresize ' . ((&columns * %ld + %ld) / %ld)\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        n,
                        (*wp).w_width as int64_t,
                        Columns.get() as int64_t / 2 as int64_t,
                        Columns.get() as int64_t,
                    ) < 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
            }
            wp = (*wp).w_next;
        }
    } else if FAIL
        == put_line(
            fd,
            b"wincmd =\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ses_win_rec(mut fd: *mut FILE, mut fr: *mut frame_T) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        return OK;
    }
    let mut frc: *mut frame_T = ses_skipframe((*fr).fr_child);
    if !frc.is_null() {
        loop {
            frc = ses_skipframe((*frc).fr_next);
            if frc.is_null() {
                break;
            }
            if fprintf(
                fd,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"wincmd _ | wincmd |\n\0".as_ptr() as *const ::core::ffi::c_char,
                if (*fr).fr_layout as ::core::ffi::c_int == FR_COL {
                    b"split\n\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"vsplit\n\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
            count += 1;
        }
    }
    if count > 0 as ::core::ffi::c_int
        && fprintf(
            fd,
            if (*fr).fr_layout as ::core::ffi::c_int == FR_COL {
                b"%dwincmd k\n\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"%dwincmd h\n\0".as_ptr() as *const ::core::ffi::c_char
            },
            count,
        ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    frc = ses_skipframe((*fr).fr_child);
    while !frc.is_null() {
        ses_win_rec(fd, frc);
        frc = ses_skipframe((*frc).fr_next);
        if !frc.is_null()
            && put_line(
                fd,
                b"wincmd w\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
    }
    return OK;
}
unsafe extern "C" fn ses_skipframe(mut fr: *mut frame_T) -> *mut frame_T {
    let mut frc: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frc = fr;
    while !frc.is_null() {
        if ses_do_frame(frc) {
            break;
        }
        frc = (*frc).fr_next;
    }
    return frc;
}
unsafe extern "C" fn ses_do_frame(mut fr: *const frame_T) -> bool {
    let mut frc: *const frame_T = ::core::ptr::null::<frame_T>();
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        return ses_do_win((*fr).fr_win) != 0;
    }
    frc = (*fr).fr_child;
    while !frc.is_null() {
        if ses_do_frame(frc) {
            return true_0 != 0;
        }
        frc = (*frc).fr_next;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ses_do_win(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if (*wp).w_floating {
        return false_0;
    }
    if (*(*wp).w_buffer).b_fname.is_null()
        || (*(*wp).w_buffer).terminal.is_null()
            && bt_nofilename((*wp).w_buffer) as ::core::ffi::c_int != 0
    {
        return (ssop_flags.get() & kOptSsopFlagBlank as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    if bt_help((*wp).w_buffer) {
        return (ssop_flags.get() & kOptSsopFlagHelp as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    if bt_terminal((*wp).w_buffer) {
        return (ssop_flags.get()
            & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
    }
    return true_0;
}
unsafe extern "C" fn ses_arglist(
    mut fd: *mut FILE,
    mut cmd: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut fullname: bool,
    mut flagp: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if fprintf(
        fd,
        b"%s\n%s\n\0".as_ptr() as *const ::core::ffi::c_char,
        cmd,
        b"%argdel\0".as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut s: *mut ::core::ffi::c_char =
            alist_name(((*gap).ga_data as *mut aentry_T).offset(i as isize));
        if !s.is_null() {
            if fullname {
                buf = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                vim_FullName(s, buf, MAXPATHL as size_t, false_0 != 0);
                s = buf;
            }
            let mut fname_esc: *mut ::core::ffi::c_char = ses_escape_fname(s, flagp);
            if fprintf(
                fd,
                b"$argadd %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                fname_esc,
            ) < 0 as ::core::ffi::c_int
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                xfree(buf as *mut ::core::ffi::c_void);
                return FAIL;
            }
            xfree(fname_esc as *mut ::core::ffi::c_void);
            xfree(buf as *mut ::core::ffi::c_void);
        }
        i += 1;
    }
    return OK;
}
unsafe extern "C" fn ses_get_fname(
    mut buf: *mut buf_T,
    mut flagp: *const ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_char {
    if !(*buf).b_sfname.is_null()
        && flagp == ssop_flags.ptr() as *const ::core::ffi::c_uint
        && ssop_flags.get()
            & (kOptSsopFlagCurdir as ::core::ffi::c_int | kOptSsopFlagSesdir as ::core::ffi::c_int)
                as ::core::ffi::c_uint
            != 0
        && p_acd.get() == 0
        && did_lcd.get() == 0
    {
        return (*buf).b_sfname;
    }
    return (*buf).b_ffname;
}
unsafe extern "C" fn ses_fname(
    mut fd: *mut FILE,
    mut buf: *mut buf_T,
    mut flagp: *mut ::core::ffi::c_uint,
    mut add_eol: bool,
) -> ::core::ffi::c_int {
    let mut name: *mut ::core::ffi::c_char = ses_get_fname(buf, flagp);
    if ses_put_fname(fd, name, flagp) == FAIL
        || add_eol as ::core::ffi::c_int != 0
            && fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ses_escape_fname(
    mut name: *mut ::core::ffi::c_char,
    mut _flagp: *mut ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sname: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
    p = sname;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            *p = '/' as ::core::ffi::c_char;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    p = vim_strsave_fnameescape(sname, VSE_NONE as ::core::ffi::c_int);
    xfree(sname as *mut ::core::ffi::c_void);
    return p;
}
unsafe extern "C" fn ses_put_fname(
    mut fd: *mut FILE,
    mut name: *mut ::core::ffi::c_char,
    mut flagp: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ses_escape_fname(name, flagp);
    let mut retval: bool = if fputs(p, fd) < 0 as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    } != 0;
    xfree(p as *mut ::core::ffi::c_void);
    return retval as ::core::ffi::c_int;
}
unsafe extern "C" fn put_view(
    mut fd: *mut FILE,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
    mut add_edit: bool,
    mut flagp: *mut ::core::ffi::c_uint,
    mut current_arg_idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut f: ::core::ffi::c_int = 0;
    let mut did_next: bool = false_0 != 0;
    let mut do_cursor: bool = flagp == ssop_flags.ptr()
        || *flagp & kOptSsopFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
    if (*wp).w_alist == global_alist.ptr() {
        if FAIL
            == put_line(
                fd,
                b"argglobal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    } else if ses_arglist(
        fd,
        b"arglocal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        &raw mut (*(*wp).w_alist).al_ga,
        flagp == vop_flags.ptr()
            || *flagp & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint == 0
            || !(*tp).tp_localdir.is_null()
            || !(*wp).w_localdir.is_null(),
        flagp,
    ) == FAIL
    {
        return FAIL;
    }
    if (*wp).w_arg_idx != current_arg_idx
        && (*wp).w_arg_idx < (*(*wp).w_alist).al_ga.ga_len
        && flagp == ssop_flags.ptr()
    {
        if fprintf(
            fd,
            b"%ldargu\n\0".as_ptr() as *const ::core::ffi::c_char,
            (*wp).w_arg_idx as int64_t + 1 as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        did_next = true_0 != 0;
    }
    if add_edit as ::core::ffi::c_int != 0 && (!did_next || (*wp).w_arg_idx_invalid != 0) {
        let mut fname_esc: *mut ::core::ffi::c_char =
            ses_escape_fname(ses_get_fname((*wp).w_buffer, flagp), flagp);
        if bt_help((*wp).w_buffer) {
            let mut curtag: *mut ::core::ffi::c_char =
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            if (0 as ::core::ffi::c_int) < (*wp).w_tagstackidx
                && (*wp).w_tagstackidx <= (*wp).w_tagstacklen
            {
                curtag = (*wp).w_tagstack[((*wp).w_tagstackidx - 1 as ::core::ffi::c_int) as usize]
                    .tagname;
            }
            if put_line(
                fd,
                b"enew | setl bt=help\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            ) == FAIL
                || fprintf(
                    fd,
                    b"help %s\0".as_ptr() as *const ::core::ffi::c_char,
                    curtag,
                ) < 0 as ::core::ffi::c_int
                || put_eol(fd) == FAIL
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                return FAIL;
            }
        } else if !(*(*wp).w_buffer).b_ffname.is_null()
            && (!bt_nofilename((*wp).w_buffer) || !(*(*wp).w_buffer).terminal.is_null())
        {
            if fprintf(
                fd,
                b"if bufexists(fnamemodify(\"%s\", \":p\")) | buffer %s | else | edit %s | endif\nif &buftype ==# 'terminal'\n  silent file %s\nendif\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                fname_esc,
                fname_esc,
                fname_esc,
                fname_esc,
            ) < 0 as ::core::ffi::c_int
            {
                xfree(fname_esc as *mut ::core::ffi::c_void);
                return FAIL;
            }
        } else {
            if FAIL
                == put_line(
                    fd,
                    b"enew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if !(*(*wp).w_buffer).b_ffname.is_null() {
                if fprintf(
                    fd,
                    b"file %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                    fname_esc,
                ) < 0 as ::core::ffi::c_int
                {
                    xfree(fname_esc as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            do_cursor = false_0 != 0;
        }
        xfree(fname_esc as *mut ::core::ffi::c_void);
    }
    if (*wp).w_alt_fnum != 0 {
        let alt: *mut buf_T = buflist_findnr((*wp).w_alt_fnum);
        if flagp == ssop_flags.ptr()
            && !alt.is_null()
            && !(*alt).b_fname.is_null()
            && *(*alt).b_fname as ::core::ffi::c_int != NUL
            && (*alt).b_p_bl != 0
            && !(bt_terminal(alt) as ::core::ffi::c_int != 0
                && ssop_flags.get()
                    & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && (fputs(b"balt \0".as_ptr() as *const ::core::ffi::c_char, fd)
                < 0 as ::core::ffi::c_int
                || ses_fname(fd, alt, flagp, true_0 != 0) == FAIL)
        {
            return FAIL;
        }
    }
    if *flagp
        & (kOptSsopFlagOptions as ::core::ffi::c_int
            | kOptSsopFlagLocaloptions as ::core::ffi::c_int) as ::core::ffi::c_uint
        != 0
        && makemap(fd, (*wp).w_buffer) == FAIL
    {
        return FAIL;
    }
    let mut save_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    curbuf.set((*curwin.get()).w_buffer);
    if *flagp
        & (kOptSsopFlagOptions as ::core::ffi::c_int
            | kOptSsopFlagLocaloptions as ::core::ffi::c_int) as ::core::ffi::c_uint
        != 0
    {
        f = makeset(
            fd,
            OPT_LOCAL as ::core::ffi::c_int,
            (flagp == vop_flags.ptr()
                || *flagp & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint == 0)
                as ::core::ffi::c_int,
        );
    } else if *flagp & kOptSsopFlagFolds as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        f = makefoldset(fd);
    } else {
        f = OK;
    }
    curwin.set(save_curwin);
    curbuf.set((*curwin.get()).w_buffer);
    if f == FAIL {
        return FAIL;
    }
    if *flagp & kOptSsopFlagFolds as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && !(*(*wp).w_buffer).b_ffname.is_null()
        && (bt_normal((*wp).w_buffer) as ::core::ffi::c_int != 0
            || bt_help((*wp).w_buffer) as ::core::ffi::c_int != 0)
    {
        if put_folds(fd, wp) == FAIL {
            return FAIL;
        }
    }
    if do_cursor {
        if (*wp).w_view_height <= 0 as ::core::ffi::c_int {
            if fprintf(
                fd,
                b"let s:l = %d\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*wp).w_cursor.lnum,
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
        } else if fprintf(
            fd,
            b"let s:l = %d - ((%d * winheight(0) + %d) / %d)\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            (*wp).w_cursor.lnum,
            (*wp).w_cursor.lnum - (*wp).w_topline,
            (*wp).w_view_height / 2 as ::core::ffi::c_int,
            (*wp).w_view_height,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if fprintf(
            fd,
            b"if s:l < 1 | let s:l = 1 | endif\nkeepjumps exe s:l\nnormal! zt\nkeepjumps %d\n\0"
                .as_ptr() as *const ::core::ffi::c_char,
            (*wp).w_cursor.lnum,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if (*wp).w_cursor.col == 0 as ::core::ffi::c_int {
            if FAIL
                == put_line(
                    fd,
                    b"normal! 0\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
        } else if (*wp).w_onebuf_opt.wo_wrap == 0
            && (*wp).w_leftcol > 0 as ::core::ffi::c_int
            && (*wp).w_width > 0 as ::core::ffi::c_int
        {
            if fprintf(
                fd,
                b"let s:c = %ld - ((%ld * winwidth(0) + %ld) / %ld)\nif s:c > 0\n  exe 'normal! ' . s:c . '|zs' . %ld . '|'\nelse\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                (*wp).w_virtcol as int64_t + 1 as int64_t,
                ((*wp).w_virtcol - (*wp).w_leftcol) as int64_t,
                ((*wp).w_width / 2 as ::core::ffi::c_int) as int64_t,
                (*wp).w_width as int64_t,
                (*wp).w_virtcol as int64_t + 1 as int64_t,
            ) < 0 as ::core::ffi::c_int
                || put_view_curpos(
                    fd,
                    wp,
                    b"  \0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
                || put_line(
                    fd,
                    b"endif\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
            {
                return FAIL;
            }
        } else if put_view_curpos(
            fd,
            wp,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
        {
            return FAIL;
        }
    }
    if !(*wp).w_localdir.is_null()
        && (flagp != vop_flags.ptr()
            || *flagp & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0)
    {
        if fputs(b"lcd \0".as_ptr() as *const ::core::ffi::c_char, fd) < 0 as ::core::ffi::c_int
            || ses_put_fname(fd, (*wp).w_localdir, flagp) == FAIL
            || fprintf(fd, b"\n\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        did_lcd.set(true_0);
    }
    return OK;
}
unsafe extern "C" fn store_session_globals(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let this_varhi_ht_: *mut hashtab_T =
        &raw mut (*(get_globvar_dict as unsafe extern "C" fn() -> *mut dict_T)()).dv_hashtab;
    let mut this_varhi_todo_: size_t = (*this_varhi_ht_).ht_used;
    let mut this_varhi_: *mut hashitem_T = (*this_varhi_ht_).ht_array;
    while this_varhi_todo_ != 0 {
        if !((*this_varhi_).hi_key.is_null()
            || (*this_varhi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            this_varhi_todo_ = this_varhi_todo_.wrapping_sub(1);
            let this_var: *mut dictitem_T = (*this_varhi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            if ((*this_var).di_tv.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*this_var).di_tv.v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
                && var_flavour(&raw mut (*this_var).di_key as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_uint
                    == VAR_FLAVOUR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let p: *mut ::core::ffi::c_char = vim_strsave_escaped(
                    tv_get_string(&raw mut (*this_var).di_tv),
                    b"\\\"\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                );
                let mut t: *mut ::core::ffi::c_char = p;
                while *t as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                    if *t as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                        *t = 'n' as ::core::ffi::c_char;
                    } else if *t as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                        *t = 'r' as ::core::ffi::c_char;
                    }
                    t = t.offset(1);
                }
                if fprintf(
                    fd,
                    b"let %s = %c%s%c\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*this_var).di_key as *mut ::core::ffi::c_char,
                    if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        '"' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                    p,
                    if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        '"' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                ) < 0 as ::core::ffi::c_int
                    || put_eol(fd) == 0 as ::core::ffi::c_int
                {
                    xfree(p as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
                xfree(p as *mut ::core::ffi::c_void);
            } else if (*this_var).di_tv.v_type as ::core::ffi::c_uint
                == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                && var_flavour(&raw mut (*this_var).di_key as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_uint
                    == VAR_FLAVOUR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut f: float_T = (*this_var).di_tv.vval.v_float;
                let mut sign: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
                if f < 0 as ::core::ffi::c_int as float_T {
                    f = -f;
                    sign = '-' as ::core::ffi::c_int;
                }
                if fprintf(
                    fd,
                    b"let %s = %c%f\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*this_var).di_key as *mut ::core::ffi::c_char,
                    sign,
                    f,
                ) < 0 as ::core::ffi::c_int
                    || put_eol(fd) == 0 as ::core::ffi::c_int
                {
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        this_varhi_ = this_varhi_.offset(1);
    }
    return OK;
}
unsafe extern "C" fn makeopens(
    mut fd: *mut FILE,
    mut dirnow: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut only_save_windows: bool = true_0 != 0;
    let mut restore_size: bool = true_0 != 0;
    let mut edited_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tab_firstwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tab_topframe: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut cur_arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ssop_flags.get() & kOptSsopFlagBuffers as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        only_save_windows = false_0 != 0;
    }
    if FAIL
        == put_line(
            fd,
            b"let v:this_session=expand(\"<sfile>:p\")\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if FAIL
        == put_line(
            fd,
            b"doautoall SessionLoadPre\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagGlobals as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if store_session_globals(fd) == FAIL {
            return FAIL;
        }
    }
    if FAIL
        == put_line(
            fd,
            b"silent only\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && put_line(
            fd,
            b"silent tabonly\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if FAIL
            == put_line(
                fd,
                b"exe \"cd \" . escape(expand(\"<sfile>:p:h\"), ' ')\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    } else if ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        let mut sname: *mut ::core::ffi::c_char = home_replace_save(
            ::core::ptr::null_mut::<buf_T>(),
            if !(*globaldir.ptr()).is_null() {
                globaldir.get()
            } else {
                dirnow
            },
        );
        let mut fname_esc: *mut ::core::ffi::c_char = ses_escape_fname(sname, ssop_flags.ptr());
        if fprintf(
            fd,
            b"cd %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            fname_esc,
        ) < 0 as ::core::ffi::c_int
        {
            xfree(fname_esc as *mut ::core::ffi::c_void);
            xfree(sname as *mut ::core::ffi::c_void);
            return FAIL;
        }
        xfree(fname_esc as *mut ::core::ffi::c_void);
        xfree(sname as *mut ::core::ffi::c_void);
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''\n  let s:wipebuf = bufnr('%')\nendif\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint
        == 0 as ::core::ffi::c_uint
    {
        if FAIL
            == put_line(
                fd,
                b"let s:shortmess_save = &shortmess\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    }
    if FAIL
        == put_line(
            fd,
            b"set shortmess+=aoO\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(only_save_windows as ::core::ffi::c_int != 0
            && (*buf).b_nwindows == 0 as ::core::ffi::c_int)
            && !((*buf).b_help as ::core::ffi::c_int != 0
                && ssop_flags.get() & kOptSsopFlagHelp as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && !(bt_terminal(buf) as ::core::ffi::c_int != 0
                && ssop_flags.get()
                    & kOptSsopFlagTerminal as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0)
            && !(*buf).b_fname.is_null()
            && (*buf).b_p_bl != 0
        {
            if fprintf(
                fd,
                b"badd +%ld \0".as_ptr() as *const ::core::ffi::c_char,
                if (*buf).b_wininfo.size == 0 as size_t {
                    1 as int64_t
                } else {
                    (**(*buf)
                        .b_wininfo
                        .items
                        .offset(0 as ::core::ffi::c_int as isize))
                    .wi_mark
                    .mark
                    .lnum as int64_t
                },
            ) < 0 as ::core::ffi::c_int
                || ses_fname(fd, buf, ssop_flags.ptr(), true_0 != 0) == FAIL
            {
                return FAIL;
            }
        }
        buf = (*buf).b_next;
    }
    if ses_arglist(
        fd,
        b"argglobal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        &raw mut (*global_alist.ptr()).al_ga,
        ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint == 0,
        ssop_flags.ptr(),
    ) == FAIL
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagResize as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"set lines=%ld columns=%ld\n\0".as_ptr() as *const ::core::ffi::c_char,
            Rows.get() as int64_t,
            Columns.get() as int64_t,
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    let mut restore_stal: bool = false_0 != 0;
    if p_stal.get() == 1 as OptInt && !(*first_tabpage.get()).tp_next.is_null() {
        if FAIL
            == put_line(
                fd,
                b"set stal=2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
        restore_stal = true_0 != 0;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if !(*tp).tp_next.is_null()
                && put_line(
                    fd,
                    b"tabnew +setlocal\\ bufhidden=wipe\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                ) == FAIL
            {
                return FAIL;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if !(*first_tabpage.get()).tp_next.is_null()
            && put_line(
                fd,
                b"tabrewind\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
    }
    let mut restore_height_width: bool = false_0 != 0;
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut need_tabnext: bool = false_0 != 0;
        let mut cnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            if tp_0 == curtab.get() {
                tab_firstwin = firstwin.get();
                tab_topframe = topframe.get();
            } else {
                tab_firstwin = (*tp_0).tp_firstwin;
                tab_topframe = (*tp_0).tp_topframe;
            }
            if tp_0 != first_tabpage.get() {
                need_tabnext = true_0 != 0;
            }
        } else {
            tp_0 = curtab.get() as *mut tabpage_T;
            tab_firstwin = firstwin.get();
            tab_topframe = topframe.get();
        }
        let mut wp: *mut win_T = tab_firstwin;
        while !wp.is_null() {
            if ses_do_win(wp) != 0
                && !(*(*wp).w_buffer).b_ffname.is_null()
                && !bt_help((*wp).w_buffer)
                && !bt_nofilename((*wp).w_buffer)
            {
                if need_tabnext as ::core::ffi::c_int != 0
                    && put_line(
                        fd,
                        b"tabnext\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == FAIL
                {
                    return FAIL;
                }
                need_tabnext = false_0 != 0;
                if fputs(b"edit \0".as_ptr() as *const ::core::ffi::c_char, fd)
                    < 0 as ::core::ffi::c_int
                    || ses_fname(fd, (*wp).w_buffer, ssop_flags.ptr(), true_0 != 0) == FAIL
                {
                    return FAIL;
                }
                if (*wp).w_arg_idx_invalid == 0 {
                    edited_win = wp;
                }
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if need_tabnext as ::core::ffi::c_int != 0
            && put_line(
                fd,
                b"tabnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
        {
            return FAIL;
        }
        if (*tab_topframe).fr_layout as ::core::ffi::c_int != FR_LEAF {
            if FAIL
                == put_line(
                    fd,
                    b"let s:save_splitbelow = &splitbelow\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let s:save_splitright = &splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"set splitbelow splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if ses_win_rec(fd, tab_topframe) == FAIL {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let &splitbelow = s:save_splitbelow\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if FAIL
                == put_line(
                    fd,
                    b"let &splitright = s:save_splitright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
        }
        let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp_0: *mut win_T = tab_firstwin;
        while !wp_0.is_null() {
            if ses_do_win(wp_0) != 0 {
                nr += 1;
            } else if !(*wp_0).w_floating {
                restore_size = false_0 != 0;
            }
            if curwin.get() == wp_0 {
                cnr = nr;
            }
            wp_0 = (*wp_0).w_next;
        }
        if !tab_firstwin.is_null() && !(*tab_firstwin).w_next.is_null() {
            if FAIL
                == put_line(
                    fd,
                    b"wincmd t\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                )
            {
                return FAIL;
            }
            if !restore_height_width {
                if FAIL
                    == put_line(
                        fd,
                        b"let s:save_winminheight = &winminheight\0".as_ptr()
                            as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    )
                {
                    return FAIL;
                }
                if FAIL
                    == put_line(
                        fd,
                        b"let s:save_winminwidth = &winminwidth\0".as_ptr()
                            as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    )
                {
                    return FAIL;
                }
            }
            if fprintf(
                fd,
                b"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1\n\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
            restore_height_width = true_0 != 0;
        }
        if nr > 1 as ::core::ffi::c_int && ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }
        if ssop_flags.get() & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && !(*tp_0).tp_localdir.is_null()
        {
            if fputs(b"tcd \0".as_ptr() as *const ::core::ffi::c_char, fd) < 0 as ::core::ffi::c_int
                || ses_put_fname(fd, (*tp_0).tp_localdir, ssop_flags.ptr()) == FAIL
                || put_eol(fd) == FAIL
            {
                return FAIL;
            }
            did_lcd.set(true_0);
        }
        let mut wp_1: *mut win_T = tab_firstwin;
        while !wp_1.is_null() {
            if ses_do_win(wp_1) != 0 {
                if put_view(
                    fd,
                    wp_1,
                    tp_0 as *mut tabpage_T,
                    wp_1 != edited_win,
                    ssop_flags.ptr(),
                    cur_arg_idx,
                ) == FAIL
                {
                    return FAIL;
                }
                if nr > 1 as ::core::ffi::c_int
                    && put_line(
                        fd,
                        b"wincmd w\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == FAIL
                {
                    return FAIL;
                }
                next_arg_idx = (*wp_1).w_arg_idx;
            }
            wp_1 = (*wp_1).w_next;
        }
        cur_arg_idx = next_arg_idx;
        if cnr > 1 as ::core::ffi::c_int
            && fprintf(
                fd,
                b"%dwincmd w\n\0".as_ptr() as *const ::core::ffi::c_char,
                cnr,
            ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        if nr > 1 as ::core::ffi::c_int && ses_winsizes(fd, restore_size, tab_firstwin) == FAIL {
            return FAIL;
        }
        if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint == 0
        {
            break;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    if ssop_flags.get() & kOptSsopFlagTabpages as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"tabnext %d\n\0".as_ptr() as *const ::core::ffi::c_char,
            tabpage_index(curtab.get()),
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    if restore_stal as ::core::ffi::c_int != 0
        && put_line(
            fd,
            b"set stal=1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == FAIL
    {
        return FAIL;
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'\n  silent exe 'bwipe ' . s:wipebuf\nendif\nunlet! s:wipebuf\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if fprintf(
        fd,
        b"set winheight=%ld winwidth=%ld\n\0".as_ptr() as *const ::core::ffi::c_char,
        p_wh.get(),
        p_wiw.get(),
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    if ssop_flags.get() & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if fprintf(
            fd,
            b"set shortmess=%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            p_shm.get(),
        ) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    } else if FAIL
        == put_line(
            fd,
            b"let &shortmess = s:shortmess_save\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        )
    {
        return FAIL;
    }
    if restore_height_width {
        if FAIL
            == put_line(
                fd,
                b"let &winminheight = s:save_winminheight\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
        if FAIL
            == put_line(
                fd,
                b"let &winminwidth = s:save_winminwidth\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            )
        {
            return FAIL;
        }
    }
    if fprintf(
        fd,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        b"let s:sx = expand(\"<sfile>:p:r\").\"x.vim\"\nif filereadable(s:sx)\n  exe \"source \" . fnameescape(s:sx)\nendif\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_loadview(mut eap: *mut exarg_T) {
    let mut fname: *mut ::core::ffi::c_char = get_view_file(*(*eap).arg);
    if fname.is_null() {
        return;
    }
    if do_source(
        fname,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) == FAIL
    {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
    }
    xfree(fname as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ex_mkrc(mut eap: *mut exarg_T) {
    let mut view_session: bool = false_0 != 0;
    let mut using_vdir: ::core::ffi::c_int = false_0;
    let mut viewFile: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int
    {
        view_session = true_0 != 0;
    }
    did_lcd.set(false_0);
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int
        && (*(*eap).arg as ::core::ffi::c_int == NUL
            || ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && *(*eap).arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL)
    {
        (*eap).forceit = true_0;
        fname = get_view_file(*(*eap).arg);
        if fname.is_null() {
            return;
        }
        viewFile = fname;
        using_vdir = true_0;
    } else if *(*eap).arg as ::core::ffi::c_int != NUL {
        fname = (*eap).arg;
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkvimrc as ::core::ffi::c_int {
        fname = VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char;
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
        fname = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
    } else {
        fname = EXRC_FILE.as_ptr() as *mut ::core::ffi::c_char;
    }
    if using_vdir != 0 && !os_isdir(p_vdir.get()) {
        vim_mkdir_emsg(p_vdir.get(), 0o755 as ::core::ffi::c_int);
    }
    let mut fd: *mut FILE = open_exfile(
        fname,
        (*eap).forceit,
        WRITEBIN.as_ptr() as *mut ::core::ffi::c_char,
    );
    if !fd.is_null() {
        let mut failed: bool = false_0 != 0;
        let mut flagp: *mut ::core::ffi::c_uint = ::core::ptr::null_mut::<::core::ffi::c_uint>();
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkview as ::core::ffi::c_int {
            flagp = vop_flags.ptr();
        } else {
            flagp = ssop_flags.ptr();
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mkvimrc as ::core::ffi::c_int {
            put_line(
                fd,
                b"version 6.0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
            if put_line(
                fd,
                b"let SessionLoad = 1\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            ) == FAIL
            {
                failed = true_0 != 0;
            }
        }
        if !view_session
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
                && *flagp & kOptSsopFlagOptions as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            let mut flags: ::core::ffi::c_int = OPT_GLOBAL as ::core::ffi::c_int;
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int
                && *flagp & kOptSsopFlagSkiprtp as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            {
                flags |= OPT_SKIPRTP as ::core::ffi::c_int;
            }
            failed = failed as ::core::ffi::c_int
                | (makemap(fd, ::core::ptr::null_mut::<buf_T>()) == FAIL
                    || makeset(fd, flags, false_0) == FAIL) as ::core::ffi::c_int
                != 0;
        }
        if !failed && view_session as ::core::ffi::c_int != 0 {
            if put_line(
                fd,
                b"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1\0"
                    .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == FAIL
            {
                failed = true_0 != 0;
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
                let mut dirnow: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                dirnow = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                if os_dirname(dirnow, MAXPATHL as size_t) == FAIL
                    || os_chdir(dirnow) != 0 as ::core::ffi::c_int
                {
                    *dirnow = NUL as ::core::ffi::c_char;
                }
                if *dirnow as ::core::ffi::c_int != NUL
                    && ssop_flags.get()
                        & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                {
                    if vim_chdirfile(fname, kCdCauseOther) == OK {
                        shorten_fnames(true_0);
                    }
                } else if *dirnow as ::core::ffi::c_int != NUL
                    && ssop_flags.get()
                        & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    && !(*globaldir.ptr()).is_null()
                {
                    if os_chdir(globaldir.get()) == 0 as ::core::ffi::c_int {
                        shorten_fnames(true_0);
                    }
                }
                failed = failed as ::core::ffi::c_int
                    | (makeopens(fd, dirnow) == FAIL) as ::core::ffi::c_int
                    != 0;
                if *dirnow as ::core::ffi::c_int != NUL
                    && (ssop_flags.get()
                        & kOptSsopFlagSesdir as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        || ssop_flags.get()
                            & kOptSsopFlagCurdir as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                            && !(*globaldir.ptr()).is_null())
                {
                    if os_chdir(dirnow) != 0 as ::core::ffi::c_int {
                        emsg(gettext(&raw const e_prev_dir as *const ::core::ffi::c_char));
                    }
                    shorten_fnames(true_0);
                }
                xfree(dirnow as *mut ::core::ffi::c_void);
            } else {
                failed = failed as ::core::ffi::c_int
                    | (put_view(
                        fd,
                        curwin.get(),
                        curtab.get(),
                        using_vdir == 0,
                        flagp,
                        -1 as ::core::ffi::c_int,
                    ) == FAIL) as ::core::ffi::c_int
                    != 0;
            }
            if fprintf(
                fd,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"let &g:so = s:so_save | let &g:siso = s:siso_save\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if p_hls.get() != 0
                && fprintf(
                    fd,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"set hlsearch\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if no_hlsearch.get() as ::core::ffi::c_int != 0
                && fprintf(
                    fd,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"nohlsearch\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if fprintf(
                fd,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"doautoall SessionLoadPost\n\0".as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            {
                failed = true_0 != 0;
            }
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
                if fprintf(
                    fd,
                    b"unlet SessionLoad\n\0".as_ptr() as *const ::core::ffi::c_char,
                ) < 0 as ::core::ffi::c_int
                {
                    failed = true_0 != 0;
                }
            }
        }
        if put_line(
            fd,
            b"\" vim: set ft=vim :\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == FAIL
        {
            failed = true_0 != 0;
        }
        failed = failed as ::core::ffi::c_int | fclose(fd) != 0;
        if failed {
            emsg(gettext(&raw const e_write as *const ::core::ffi::c_char));
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_mksession as ::core::ffi::c_int {
            let tbuf: *mut ::core::ffi::c_char =
                xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
            if vim_FullName(fname, tbuf, MAXPATHL as size_t, false_0 != 0) == OK {
                set_vim_var_string(VV_THIS_SESSION, tbuf, -1 as ptrdiff_t);
            }
            xfree(tbuf as *mut ::core::ffi::c_void);
        }
    }
    xfree(viewFile as *mut ::core::ffi::c_void);
    apply_autocmds(
        EVENT_SESSIONWRITEPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
}
unsafe extern "C" fn get_view_file(mut c: ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if (*curbuf.get()).b_ffname.is_null() {
        emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut sname: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), (*curbuf.get()).b_ffname);
    let mut len: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = sname;
    while *p != 0 {
        if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            || vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            len = len.wrapping_add(1);
        }
        p = p.offset(1);
    }
    let mut retval: *mut ::core::ffi::c_char = xmalloc(
        strlen(sname)
            .wrapping_add(len)
            .wrapping_add(strlen(p_vdir.get()))
            .wrapping_add(9 as size_t),
    ) as *mut ::core::ffi::c_char;
    strcpy(retval, p_vdir.get());
    add_pathsep(retval);
    let mut s: *mut ::core::ffi::c_char = retval.offset(strlen(retval) as isize);
    let mut p_0: *mut ::core::ffi::c_char = sname;
    while *p_0 != 0 {
        if *p_0 as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            let c2rust_fresh0 = s;
            s = s.offset(1);
            *c2rust_fresh0 = '=' as ::core::ffi::c_char;
            let c2rust_fresh1 = s;
            s = s.offset(1);
            *c2rust_fresh1 = '=' as ::core::ffi::c_char;
        } else if vim_ispathsep(*p_0 as ::core::ffi::c_int) {
            let c2rust_fresh2 = s;
            s = s.offset(1);
            *c2rust_fresh2 = '=' as ::core::ffi::c_char;
            let c2rust_fresh3 = s;
            s = s.offset(1);
            *c2rust_fresh3 = '+' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh4 = s;
            s = s.offset(1);
            *c2rust_fresh4 = *p_0;
        }
        p_0 = p_0.offset(1);
    }
    let c2rust_fresh5 = s;
    s = s.offset(1);
    *c2rust_fresh5 = '=' as ::core::ffi::c_char;
    let c2rust_fresh6 = s;
    s = s.offset(1);
    *c2rust_fresh6 = c;
    xmemcpyz(
        s as *mut ::core::ffi::c_void,
        b".vim\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
    );
    xfree(sname as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn put_eol(mut fd: *mut FILE) -> ::core::ffi::c_int {
    if putc('\n' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn put_line(
    mut fd: *mut FILE,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if fprintf(fd, b"%s\n\0".as_ptr() as *const ::core::ffi::c_char, s) < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    return OK;
}
pub const EXRC_FILE: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b".exrc\0") };
pub const VIMRC_FILE: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".nvimrc\0") };
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WRITEBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wb\0") };
