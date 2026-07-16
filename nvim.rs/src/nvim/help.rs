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
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn bt_help(buf: *const buf_T) -> bool;
    fn set_buflisted(on: ::core::ffi::c_int);
    fn wipe_buffer(buf: *mut buf_T, aucmd: bool);
    static mut p_hf: *mut ::core::ffi::c_char;
    static mut p_hh: OptInt;
    static mut p_hlg: *mut ::core::ffi::c_char;
    static mut p_rtp: *mut ::core::ffi::c_char;
    static mut p_sb: ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn sort_strings(files: *mut *mut ::core::ffi::c_char, count: ::core::ffi::c_int);
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn buf_init_chartab(buf: *mut buf_T, global: bool) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    static e_noident: [::core::ffi::c_char; 0];
    static e_fnametoolong: [::core::ffi::c_char; 0];
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_fgets(buf: *mut ::core::ffi::c_char, size: ::core::ffi::c_int, fp: *mut FILE) -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut Columns: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut restart_edit: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut KeyTyped: bool;
    static mut got_int: bool;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_free_object(value: Object);
    fn api_clear_error(value: *mut Error);
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg_multiline(
        s: *const ::core::ffi::c_char,
        kind: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        multiline: bool,
    ) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn check_buf_options(buf: *mut buf_T);
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn line_breakcheck();
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn gen_expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn do_in_path(
        path: *const ::core::ffi::c_char,
        prefix: *const ::core::ffi::c_char,
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        callback: DoInRuntimepathCB,
        cookie: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn do_tag(
        tag: *mut ::core::ffi::c_char,
        type_0: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
        verbose: bool,
    );
    fn find_tags(
        pat: *mut ::core::ffi::c_char,
        num_matches: *mut ::core::ffi::c_int,
        matchesp: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mincount: ::core::ffi::c_int,
        buf_ffname: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_setheight(height: ::core::ffi::c_int);
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
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type Boolean = bool;
pub type Integer = int64_t;
pub type Float = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
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
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_14 = 76;
pub const HLF_PRE: C2Rust_Unnamed_14 = 75;
pub const HLF_OK: C2Rust_Unnamed_14 = 74;
pub const HLF_SO: C2Rust_Unnamed_14 = 73;
pub const HLF_SE: C2Rust_Unnamed_14 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_14 = 71;
pub const HLF_TS: C2Rust_Unnamed_14 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_14 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_14 = 68;
pub const HLF_CU: C2Rust_Unnamed_14 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_14 = 66;
pub const HLF_WBR: C2Rust_Unnamed_14 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_14 = 64;
pub const HLF_MSG: C2Rust_Unnamed_14 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_14 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_14 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_14 = 60;
pub const HLF_0: C2Rust_Unnamed_14 = 59;
pub const HLF_QFL: C2Rust_Unnamed_14 = 58;
pub const HLF_MC: C2Rust_Unnamed_14 = 57;
pub const HLF_CUL: C2Rust_Unnamed_14 = 56;
pub const HLF_CUC: C2Rust_Unnamed_14 = 55;
pub const HLF_TPF: C2Rust_Unnamed_14 = 54;
pub const HLF_TPS: C2Rust_Unnamed_14 = 53;
pub const HLF_TP: C2Rust_Unnamed_14 = 52;
pub const HLF_PBR: C2Rust_Unnamed_14 = 51;
pub const HLF_PST: C2Rust_Unnamed_14 = 50;
pub const HLF_PSB: C2Rust_Unnamed_14 = 49;
pub const HLF_PSX: C2Rust_Unnamed_14 = 48;
pub const HLF_PNX: C2Rust_Unnamed_14 = 47;
pub const HLF_PSK: C2Rust_Unnamed_14 = 46;
pub const HLF_PNK: C2Rust_Unnamed_14 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_14 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_14 = 43;
pub const HLF_PSI: C2Rust_Unnamed_14 = 42;
pub const HLF_PNI: C2Rust_Unnamed_14 = 41;
pub const HLF_SPL: C2Rust_Unnamed_14 = 40;
pub const HLF_SPR: C2Rust_Unnamed_14 = 39;
pub const HLF_SPC: C2Rust_Unnamed_14 = 38;
pub const HLF_SPB: C2Rust_Unnamed_14 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_14 = 36;
pub const HLF_SC: C2Rust_Unnamed_14 = 35;
pub const HLF_TXA: C2Rust_Unnamed_14 = 34;
pub const HLF_TXD: C2Rust_Unnamed_14 = 33;
pub const HLF_DED: C2Rust_Unnamed_14 = 32;
pub const HLF_CHD: C2Rust_Unnamed_14 = 31;
pub const HLF_ADD: C2Rust_Unnamed_14 = 30;
pub const HLF_FC: C2Rust_Unnamed_14 = 29;
pub const HLF_FL: C2Rust_Unnamed_14 = 28;
pub const HLF_WM: C2Rust_Unnamed_14 = 27;
pub const HLF_W: C2Rust_Unnamed_14 = 26;
pub const HLF_VNC: C2Rust_Unnamed_14 = 25;
pub const HLF_V: C2Rust_Unnamed_14 = 24;
pub const HLF_T: C2Rust_Unnamed_14 = 23;
pub const HLF_VSP: C2Rust_Unnamed_14 = 22;
pub const HLF_C: C2Rust_Unnamed_14 = 21;
pub const HLF_SNC: C2Rust_Unnamed_14 = 20;
pub const HLF_S: C2Rust_Unnamed_14 = 19;
pub const HLF_R: C2Rust_Unnamed_14 = 18;
pub const HLF_CLF: C2Rust_Unnamed_14 = 17;
pub const HLF_CLS: C2Rust_Unnamed_14 = 16;
pub const HLF_CLN: C2Rust_Unnamed_14 = 15;
pub const HLF_LNB: C2Rust_Unnamed_14 = 14;
pub const HLF_LNA: C2Rust_Unnamed_14 = 13;
pub const HLF_N: C2Rust_Unnamed_14 = 12;
pub const HLF_CM: C2Rust_Unnamed_14 = 11;
pub const HLF_M: C2Rust_Unnamed_14 = 10;
pub const HLF_LC: C2Rust_Unnamed_14 = 9;
pub const HLF_L: C2Rust_Unnamed_14 = 8;
pub const HLF_I: C2Rust_Unnamed_14 = 7;
pub const HLF_E: C2Rust_Unnamed_14 = 6;
pub const HLF_D: C2Rust_Unnamed_14 = 5;
pub const HLF_AT: C2Rust_Unnamed_14 = 4;
pub const HLF_TERM: C2Rust_Unnamed_14 = 3;
pub const HLF_EOB: C2Rust_Unnamed_14 = 2;
pub const HLF_8: C2Rust_Unnamed_14 = 1;
pub const HLF_NONE: C2Rust_Unnamed_14 = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_15 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_15 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_15 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_15 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_15 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_15 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_15 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_15 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_15 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_15 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_15 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_15 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_15 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_15 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_15 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_15 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_15 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_15 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_15 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_15 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_15 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_15 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_15 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_15 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_15 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_15 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_15 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_15 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_15 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_15 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_15 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_15 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_15 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_15 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_15 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_15 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_15 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_15 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_15 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_15 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_15 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_15 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_15 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_15 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_15 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_15 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_15 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_15 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_15 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_15 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_15 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_15 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_15 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_15 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_15 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_15 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_15 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_15 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_15 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_15 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_15 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_15 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_15 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_15 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_15 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_15 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_15 = -2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
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
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
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
    pub cs_pend: C2Rust_Unnamed_16,
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
pub union C2Rust_Unnamed_16 {
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_18 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_18 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_18 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_18 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_18 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_18 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_18 = 7;
pub const WILD_ALL: C2Rust_Unnamed_18 = 6;
pub const WILD_PREV: C2Rust_Unnamed_18 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_18 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_18 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_18 = 2;
pub const WILD_FREE: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_19 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_19 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_19 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_19 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_19 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_19 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_19 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_19 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_19 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_19 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_19 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_19 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_19 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_19 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_19 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_19 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_20 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_20 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_20 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_20 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_20 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_20 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_21 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_21 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_21 = 0;
pub type DoInRuntimepathCB = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut *mut ::core::ffi::c_char,
        bool,
        *mut ::core::ffi::c_void,
    ) -> bool,
>;
pub const DT_HELP: C2Rust_Unnamed_25 = 8;
pub const TAG_MANY: C2Rust_Unnamed_26 = 300;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_26 = 256;
pub const TAG_VERBOSE: C2Rust_Unnamed_26 = 32;
pub const TAG_NAMES: C2Rust_Unnamed_26 = 2;
pub const TAG_REGEXP: C2Rust_Unnamed_26 = 4;
pub const TAG_HELP: C2Rust_Unnamed_26 = 1;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_26 = 128;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const WSP_TOP: C2Rust_Unnamed_27 = 8;
pub const WSP_BOT: C2Rust_Unnamed_27 = 16;
pub const WSP_HELP: C2Rust_Unnamed_27 = 32;
pub const OPT_LOCAL: C2Rust_Unnamed_22 = 2;
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const EW_SILENT: C2Rust_Unnamed_23 = 32;
pub const EW_FILE: C2Rust_Unnamed_23 = 2;
pub const DIP_DIR: C2Rust_Unnamed_24 = 2;
pub const DIP_ALL: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_22 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_22 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_22 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_22 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_22 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_22 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_23 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_23 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_23 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_23 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_23 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_23 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_23 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_23 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_23 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_23 = 512;
pub const EW_ICASE: C2Rust_Unnamed_23 = 256;
pub const EW_PATH: C2Rust_Unnamed_23 = 128;
pub const EW_EXEC: C2Rust_Unnamed_23 = 64;
pub const EW_KEEPALL: C2Rust_Unnamed_23 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_23 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_23 = 4;
pub const EW_DIR: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_24 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_24 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_24 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_24 = 32;
pub const DIP_OPT: C2Rust_Unnamed_24 = 16;
pub const DIP_START: C2Rust_Unnamed_24 = 8;
pub const DIP_ERR: C2Rust_Unnamed_24 = 4;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const DT_FREE: C2Rust_Unnamed_25 = 99;
pub const DT_LTAG: C2Rust_Unnamed_25 = 11;
pub const DT_JUMP: C2Rust_Unnamed_25 = 9;
pub const DT_SELECT: C2Rust_Unnamed_25 = 7;
pub const DT_LAST: C2Rust_Unnamed_25 = 6;
pub const DT_FIRST: C2Rust_Unnamed_25 = 5;
pub const DT_PREV: C2Rust_Unnamed_25 = 4;
pub const DT_NEXT: C2Rust_Unnamed_25 = 3;
pub const DT_POP: C2Rust_Unnamed_25 = 2;
pub const DT_TAG: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const TAG_INS_COMP: C2Rust_Unnamed_26 = 64;
pub const TAG_NOIC: C2Rust_Unnamed_26 = 8;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_27 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_27 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_27 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_27 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_27 = 64;
pub const WSP_HOR: C2Rust_Unnamed_27 = 4;
pub const WSP_VERT: C2Rust_Unnamed_27 = 2;
pub const WSP_ROOM: C2Rust_Unnamed_27 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 57] = unsafe {
    ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
        *b"int find_help_tags(const char *, int *, char ***, _Bool)\0",
    )
};
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn ex_help(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut helpfd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut num_matches: ::core::ffi::c_int = 0;
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut empty_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut alt_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let old_KeyTyped: bool = KeyTyped;
    if !eap.is_null() {
        arg = (*eap).arg;
        while *arg != 0 {
            if *arg as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '|' as ::core::ffi::c_int
            {
                let c2rust_fresh0 = arg;
                arg = arg.offset(1);
                *c2rust_fresh0 = NUL as ::core::ffi::c_char;
                (*eap).nextcmd = arg;
                break;
            } else {
                arg = arg.offset(1);
            }
        }
        arg = (*eap).arg;
        if (*eap).skip != 0 {
            return;
        }
    } else {
        arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut p: *mut ::core::ffi::c_char = arg
        .offset(strlen(arg) as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    while p > arg
        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
    {
        let c2rust_fresh1 = p;
        p = p.offset(-1);
        *c2rust_fresh1 = NUL as ::core::ffi::c_char;
    }
    let mut lang: *mut ::core::ffi::c_char = check_help_lang(arg);
    let mut helpbang: bool =
        !eap.is_null() && (*eap).forceit != 0 && *arg as ::core::ffi::c_int == NUL;
    if *arg as ::core::ffi::c_int == NUL && !helpbang {
        arg = b"help.txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut allocated_arg: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if helpbang {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut res: Object = nlua_exec(
            String_0 {
                data: b"return require'vim._core.help'.resolve_tag()\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 45]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            && res.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            && res.data.string.size > 0 as size_t
        {
            allocated_arg = xstrdup(res.data.string.data);
            arg = allocated_arg;
        }
        api_free_object(res);
        api_clear_error(&raw mut err);
        if allocated_arg.is_null() {
            emsg(gettext(&raw const e_noident as *const ::core::ffi::c_char));
            return;
        }
    }
    let mut n: ::core::ffi::c_int = find_help_tags(
        arg,
        &raw mut num_matches,
        &raw mut matches,
        !eap.is_null() && (*eap).forceit != 0,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if n != FAIL && !lang.is_null() {
        i = 0 as ::core::ffi::c_int;
        while i < num_matches {
            let mut len: ::core::ffi::c_int =
                strlen(*matches.offset(i as isize)) as ::core::ffi::c_int;
            if len > 3 as ::core::ffi::c_int
                && *(*matches.offset(i as isize)).offset((len - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '@' as ::core::ffi::c_int
                && strcasecmp(
                    (*matches.offset(i as isize))
                        .offset(len as isize)
                        .offset(-(2 as ::core::ffi::c_int as isize)),
                    lang,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
    }
    if i >= num_matches || n == FAIL {
        if !lang.is_null() {
            semsg(
                gettext(b"E661: No '%s' help for %s\0".as_ptr() as *const ::core::ffi::c_char),
                lang,
                arg,
            );
        } else {
            semsg(
                gettext(b"E149: No help for %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
        }
        if n != FAIL {
            FreeWild(num_matches, matches);
        }
        xfree(allocated_arg as *mut ::core::ffi::c_void);
        return;
    }
    let mut tag: *mut ::core::ffi::c_char = xstrdup(*matches.offset(i as isize));
    FreeWild(num_matches, matches);
    '_erret: {
        if !bt_help((*curwin).w_buffer) || cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
            if cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
                wp = ::core::ptr::null_mut::<win_T>();
            } else {
                wp = ::core::ptr::null_mut::<win_T>();
                let mut wp2: *mut win_T = if curtab == curtab {
                    firstwin
                } else {
                    (*curtab).tp_firstwin
                };
                while !wp2.is_null() {
                    if bt_help((*wp2).w_buffer) as ::core::ffi::c_int != 0
                        && !(*wp2).w_config.hide
                        && (*wp2).w_config.focusable as ::core::ffi::c_int != 0
                    {
                        wp = wp2;
                        break;
                    } else {
                        wp2 = (*wp2).w_next;
                    }
                }
            }
            if !wp.is_null() && (*(*wp).w_buffer).b_nwindows > 0 as ::core::ffi::c_int {
                win_enter(wp, true_0 != 0);
            } else {
                helpfd = os_fopen(p_hf, READBIN.as_ptr());
                if helpfd.is_null() {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"Help file \"%s\" not found\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        p_hf,
                    );
                    break '_erret;
                } else {
                    fclose(helpfd);
                    n = WSP_HELP as ::core::ffi::c_int;
                    if cmdmod.cmod_split == 0 as ::core::ffi::c_int
                        && (*curwin).w_width != Columns
                        && (*curwin).w_width < 80 as ::core::ffi::c_int
                    {
                        n |= if p_sb != 0 {
                            WSP_BOT as ::core::ffi::c_int
                        } else {
                            WSP_TOP as ::core::ffi::c_int
                        };
                    }
                    if win_split(0 as ::core::ffi::c_int, n) == FAIL {
                        break '_erret;
                    } else {
                        if ((*curwin).w_height as OptInt) < p_hh {
                            win_setheight(p_hh as ::core::ffi::c_int);
                        }
                        alt_fnum = (*curbuf).handle as ::core::ffi::c_int;
                        do_ecmd(
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<exarg_T>(),
                            ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                            ECMD_HIDE as ::core::ffi::c_int + ECMD_SET_HELP as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<win_T>(),
                        );
                        if cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            (*curwin).w_alt_fnum = alt_fnum;
                        }
                        empty_fnum = (*curbuf).handle as ::core::ffi::c_int;
                    }
                }
            }
        }
        restart_edit = 0 as ::core::ffi::c_int;
        KeyTyped = old_KeyTyped;
        do_tag(
            tag,
            DT_HELP as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            false_0,
            true_0 != 0,
        );
        if empty_fnum != 0 as ::core::ffi::c_int && (*curbuf).handle != empty_fnum {
            let mut buf: *mut buf_T = buflist_findnr(empty_fnum);
            if !buf.is_null() && (*buf).b_nwindows == 0 as ::core::ffi::c_int {
                wipe_buffer(buf, true_0 != 0);
            }
        }
        if alt_fnum != 0 as ::core::ffi::c_int
            && (*curwin).w_alt_fnum == empty_fnum
            && cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            (*curwin).w_alt_fnum = alt_fnum;
        }
    }
    xfree(tag as *mut ::core::ffi::c_void);
    xfree(allocated_arg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ex_helpclose(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !win.is_null() {
        if bt_help((*win).w_buffer) {
            win_close(win, false_0 != 0, (*eap).forceit != 0);
            return;
        }
        win = (*win).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_help_lang(
    mut arg: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = strlen(arg) as ::core::ffi::c_int;
    if len >= 3 as ::core::ffi::c_int
        && *arg.offset((len - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '@' as ::core::ffi::c_int
        && (*arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset((len - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
        && (*arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        *arg.offset((len - 3 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
        return arg
            .offset(len as isize)
            .offset(-(2 as ::core::ffi::c_int as isize));
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn help_heuristic(
    mut matched_string: *mut ::core::ffi::c_char,
    mut offset: ::core::ffi::c_int,
    mut wrong_case: bool,
) -> ::core::ffi::c_int {
    let mut num_letters: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = matched_string;
    while *p != 0 {
        if *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            num_letters += 1;
        }
        p = p.offset(1);
    }
    if offset > 0 as ::core::ffi::c_int
        && (*matched_string.offset(offset as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *matched_string.offset(offset as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*matched_string.offset(offset as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0)
        && (*matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(
                *matched_string.offset((offset - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0)
    {
        offset += 10000 as ::core::ffi::c_int;
    } else if offset > 2 as ::core::ffi::c_int {
        offset *= 200 as ::core::ffi::c_int;
    }
    if wrong_case {
        offset += 5000 as ::core::ffi::c_int;
    }
    if *matched_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '+' as ::core::ffi::c_int
        && *matched_string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        offset += 100 as ::core::ffi::c_int;
    }
    return 100 as ::core::ffi::c_int * num_letters
        + strlen(matched_string) as ::core::ffi::c_int
        + offset;
}
unsafe extern "C" fn help_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut ::core::ffi::c_char = (*(s1 as *mut *mut ::core::ffi::c_char))
        .offset(strlen(*(s1 as *mut *mut ::core::ffi::c_char)) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    let mut p2: *mut ::core::ffi::c_char = (*(s2 as *mut *mut ::core::ffi::c_char))
        .offset(strlen(*(s2 as *mut *mut ::core::ffi::c_char)) as isize)
        .offset(1 as ::core::ffi::c_int as isize);
    let mut cmp: ::core::ffi::c_int = strcmp(p1, p2);
    if cmp != 0 as ::core::ffi::c_int {
        return cmp;
    }
    return strcmp(
        *(s1 as *mut *mut ::core::ffi::c_char),
        *(s2 as *mut *mut ::core::ffi::c_char),
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_help_tags(
    mut arg: *const ::core::ffi::c_char,
    mut num_matches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut keep_lang: bool,
) -> ::core::ffi::c_int {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh2 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(arg),
        },
    };
    let mut res: Object = nlua_exec(
        String_0 {
            data: b"return require'vim._core.help'.escape_subject(...)\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E as ::core::ffi::c_int,
            true_0 != 0,
        );
        api_clear_error(&raw mut err);
        return FAIL;
    }
    api_clear_error(&raw mut err);
    '_c2rust_label: {
        if res.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"res.type == kObjectTypeString\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/help.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                353 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    xstrlcpy(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        res.data.string.data,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
    );
    api_free_object(res);
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *num_matches = 0 as ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = TAG_HELP as ::core::ffi::c_int
        | TAG_REGEXP as ::core::ffi::c_int
        | TAG_NAMES as ::core::ffi::c_int
        | TAG_VERBOSE as ::core::ffi::c_int
        | TAG_NO_TAGFUNC as ::core::ffi::c_int;
    if keep_lang {
        flags |= TAG_KEEP_LANG as ::core::ffi::c_int;
    }
    if find_tags(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        num_matches,
        matches,
        flags,
        MAXCOL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) == OK
        && *num_matches > 0 as ::core::ffi::c_int
    {
        qsort(
            *matches as *mut ::core::ffi::c_void,
            *num_matches as size_t,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
            Some(
                help_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        while *num_matches > TAG_MANY as ::core::ffi::c_int {
            *num_matches -= 1;
            xfree(*(*matches).offset(*num_matches as isize) as *mut ::core::ffi::c_void);
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cleanup_help_tags(
    mut num_file: ::core::ffi::c_int,
    mut file: *mut *mut ::core::ffi::c_char,
) {
    let mut buf: [::core::ffi::c_char; 4] = [0; 4];
    let mut p: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    if *p_hlg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && (*p_hlg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 'e' as ::core::ffi::c_int
            || *p_hlg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 'n' as ::core::ffi::c_int)
    {
        let c2rust_fresh3 = p;
        p = p.offset(1);
        *c2rust_fresh3 = '@' as ::core::ffi::c_char;
        let c2rust_fresh4 = p;
        p = p.offset(1);
        *c2rust_fresh4 = *p_hlg.offset(0 as ::core::ffi::c_int as isize);
        let c2rust_fresh5 = p;
        p = p.offset(1);
        *c2rust_fresh5 = *p_hlg.offset(1 as ::core::ffi::c_int as isize);
    }
    *p = NUL as ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_file {
        let mut len: ::core::ffi::c_int =
            strlen(*file.offset(i as isize)) as ::core::ffi::c_int - 3 as ::core::ffi::c_int;
        if len > 0 as ::core::ffi::c_int {
            if strcmp(
                (*file.offset(i as isize)).offset(len as isize),
                b"@en\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                let mut j: ::core::ffi::c_int = 0;
                j = 0 as ::core::ffi::c_int;
                while j < num_file {
                    if j != i
                        && strlen(*file.offset(j as isize)) as ::core::ffi::c_int
                            == len + 3 as ::core::ffi::c_int
                        && strncmp(
                            *file.offset(i as isize),
                            *file.offset(j as isize),
                            (len as size_t).wrapping_add(1 as size_t),
                        ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    j += 1;
                }
                if j == num_file {
                    *(*file.offset(i as isize)).offset(len as isize) = NUL as ::core::ffi::c_char;
                }
            }
        }
        i += 1;
    }
    if *(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < num_file {
            let mut len_0: ::core::ffi::c_int =
                strlen(*file.offset(i_0 as isize)) as ::core::ffi::c_int - 3 as ::core::ffi::c_int;
            if len_0 > 0 as ::core::ffi::c_int {
                if strcmp(
                    (*file.offset(i_0 as isize)).offset(len_0 as isize),
                    &raw mut buf as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    *(*file.offset(i_0 as isize)).offset(len_0 as isize) =
                        NUL as ::core::ffi::c_char;
                }
            }
            i_0 += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn prepare_help_buffer() {
    (*curbuf).b_help = true_0 != 0;
    set_option_direct(
        kOptBuftype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"help\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
    );
    let mut p: *mut ::core::ffi::c_char = b"!-~,^*,^|,^\",192-255\0".as_ptr()
        as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char;
    if strcmp((*curbuf).b_p_isk, p) != 0 as ::core::ffi::c_int {
        set_option_direct(
            kOptIskeyword,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p),
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
            0 as scid_T,
        );
        check_buf_options(curbuf);
        buf_init_chartab(curbuf, false_0 != 0);
    }
    set_option_direct(
        kOptFoldmethod,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"manual\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
    );
    (*curbuf).b_p_ts = 8 as OptInt;
    (*curwin).w_onebuf_opt.wo_list = false_0;
    (*curbuf).b_p_ma = false_0;
    (*curbuf).b_p_bin = false_0;
    (*curwin).w_onebuf_opt.wo_nu = 0 as ::core::ffi::c_int;
    (*curwin).w_onebuf_opt.wo_rnu = 0 as ::core::ffi::c_int;
    (*curwin).w_onebuf_opt.wo_scb = false_0;
    (*curwin).w_onebuf_opt.wo_crb = false_0;
    (*curwin).w_onebuf_opt.wo_arab = false_0;
    (*curwin).w_onebuf_opt.wo_rl = false_0;
    (*curwin).w_onebuf_opt.wo_fen = false_0;
    (*curwin).w_onebuf_opt.wo_diff = false_0;
    (*curwin).w_onebuf_opt.wo_spell = false_0;
    set_buflisted(false_0);
}
#[no_mangle]
pub unsafe extern "C" fn get_local_additions() {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut res: Object = nlua_exec(
        String_0 {
            data: b"return require'vim._core.help'.local_additions()\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 49]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        },
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E as ::core::ffi::c_int,
            true_0 != 0,
        );
    }
    api_free_object(res);
    api_clear_error(&raw mut err);
}
#[no_mangle]
pub unsafe extern "C" fn ex_exusage(mut eap: *mut exarg_T) {
    do_cmdline_cmd(b"help ex-cmd-index\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn ex_viusage(mut eap: *mut exarg_T) {
    do_cmdline_cmd(b"help normal-index\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn helptags_one(
    mut dir: *mut ::core::ffi::c_char,
    mut ext: *const ::core::ffi::c_char,
    mut tagfname: *const ::core::ffi::c_char,
    mut add_help_tags: bool,
    mut ignore_writeerr: bool,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut filecount: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dirlen: size_t = xstrlcpy(
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        dir,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
    if dirlen >= MAXPATHL as size_t
        || xstrlcat(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            b"/**/*\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
        || xstrlcat(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            ext,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut buff_list: [*mut ::core::ffi::c_char; 1] =
        [&raw mut NameBuff as *mut ::core::ffi::c_char];
    let res: ::core::ffi::c_int = gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut buff_list as *mut *mut ::core::ffi::c_char,
        &raw mut filecount,
        &raw mut files,
        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
    );
    if res == FAIL || filecount == 0 as ::core::ffi::c_int {
        if !got_int {
            semsg(
                gettext(b"E151: No match: %s\0".as_ptr() as *const ::core::ffi::c_char),
                &raw mut NameBuff as *mut ::core::ffi::c_char,
            );
        }
        if res != FAIL {
            FreeWild(filecount, files);
        }
        return;
    }
    memcpy(
        &raw mut NameBuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        dir as *const ::core::ffi::c_void,
        dirlen.wrapping_add(1 as size_t),
    );
    if !add_pathsep(&raw mut NameBuff as *mut ::core::ffi::c_char)
        || xstrlcat(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            tagfname,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let fd_tags: *mut FILE = os_fopen(
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        b"w\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if fd_tags.is_null() {
        if !ignore_writeerr {
            semsg(
                gettext(
                    b"E152: Cannot open %s for writing\0".as_ptr() as *const ::core::ffi::c_char
                ),
                &raw mut NameBuff as *mut ::core::ffi::c_char,
            );
        }
        FreeWild(filecount, files);
        return;
    }
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    if add_help_tags as ::core::ffi::c_int != 0
        || path_full_compare(
            b"$VIMRUNTIME/doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            dir,
            false_0 != 0,
            true_0 != 0,
        ) as ::core::ffi::c_uint
            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut s_len: size_t = (18 as size_t).wrapping_add(strlen(tagfname));
        s = xmalloc(s_len) as *mut ::core::ffi::c_char;
        snprintf(
            s,
            s_len,
            b"help-tags\t%s\t1\n\0".as_ptr() as *const ::core::ffi::c_char,
            tagfname,
        );
        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = s;
        ga.ga_len += 1;
    }
    let mut fi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while fi < filecount && !got_int {
        let fd: *mut FILE = os_fopen(
            *files.offset(fi as isize),
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            semsg(
                gettext(
                    b"E153: Unable to open %s for reading\0".as_ptr() as *const ::core::ffi::c_char
                ),
                *files.offset(fi as isize),
            );
        } else {
            let fname: *const ::core::ffi::c_char = (*files.offset(fi as isize))
                .offset(dirlen as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            let mut in_example: bool = false_0 != 0;
            while !vim_fgets(&raw mut IObuff as *mut ::core::ffi::c_char, IOSIZE, fd) && !got_int {
                if in_example {
                    if !vim_strchr(
                        b" \t\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                        IObuff[0 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int,
                    )
                    .is_null()
                    {
                        continue;
                    }
                    in_example = false_0 != 0;
                }
                let mut p1: *mut ::core::ffi::c_char = vim_strchr(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    '*' as ::core::ffi::c_int,
                );
                while !p1.is_null() {
                    let mut p2: *mut ::core::ffi::c_char = strchr(
                        p1.offset(1 as ::core::ffi::c_int as isize),
                        '*' as ::core::ffi::c_int,
                    );
                    if !p2.is_null() && p2 > p1.offset(1 as ::core::ffi::c_int as isize) {
                        s = p1.offset(1 as ::core::ffi::c_int as isize);
                        while s < p2 {
                            if *s as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                || *s as ::core::ffi::c_int == '\t' as ::core::ffi::c_int
                                || *s as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                            {
                                break;
                            }
                            s = s.offset(1);
                        }
                        if s == p2
                            && (p1 == &raw mut IObuff as *mut ::core::ffi::c_char
                                || *p1.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ' ' as ::core::ffi::c_int
                                || *p1.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '\t' as ::core::ffi::c_int)
                            && (!vim_strchr(
                                b" \t\n\r\0".as_ptr() as *const ::core::ffi::c_char,
                                *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int,
                            )
                            .is_null()
                                || *s.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL)
                        {
                            *p2 = NUL as ::core::ffi::c_char;
                            p1 = p1.offset(1);
                            let mut s_len_0: size_t = (p2.offset_from(p1) as size_t)
                                .wrapping_add(strlen(fname))
                                .wrapping_add(2 as size_t);
                            s = xmalloc(s_len_0) as *mut ::core::ffi::c_char;
                            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                            *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                                .offset(ga.ga_len as isize) = s;
                            ga.ga_len += 1;
                            snprintf(
                                s,
                                s_len_0,
                                b"%s\t%s\0".as_ptr() as *const ::core::ffi::c_char,
                                p1,
                                fname,
                            );
                            p2 = vim_strchr(
                                p2.offset(1 as ::core::ffi::c_int as isize),
                                '*' as ::core::ffi::c_int,
                            );
                        }
                    }
                    p1 = p2;
                }
                let mut off: size_t = strlen(&raw mut IObuff as *mut ::core::ffi::c_char);
                if off >= 2 as size_t
                    && IObuff[off.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    off = off.wrapping_sub(2 as size_t);
                    while off > 0 as size_t
                        && (IObuff[off as usize] as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && IObuff[off as usize] as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint
                            || ascii_isdigit(IObuff[off as usize] as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0)
                    {
                        off = off.wrapping_sub(1);
                    }
                    if IObuff[off as usize] as ::core::ffi::c_int == '>' as ::core::ffi::c_int
                        && (off == 0 as size_t
                            || IObuff[off.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int)
                    {
                        in_example = true_0 != 0;
                    }
                }
                line_breakcheck();
            }
            fclose(fd);
        }
        fi += 1;
    }
    FreeWild(filecount, files);
    if !got_int && !ga.ga_data.is_null() {
        sort_strings(ga.ga_data as *mut *mut ::core::ffi::c_char, ga.ga_len);
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut p1_0: *mut ::core::ffi::c_char = *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                .offset((i - 1 as ::core::ffi::c_int) as isize);
            let mut p2_0: *mut ::core::ffi::c_char =
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            while *p1_0 as ::core::ffi::c_int == *p2_0 as ::core::ffi::c_int {
                if *p2_0 as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
                    *p2_0 = NUL as ::core::ffi::c_char;
                    vim_snprintf(
                        &raw mut NameBuff as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        gettext(b"E154: Duplicate tag \"%s\" in file %s/%s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                        dir,
                        p2_0.offset(1 as ::core::ffi::c_int as isize),
                    );
                    emsg(&raw mut NameBuff as *mut ::core::ffi::c_char);
                    *p2_0 = '\t' as ::core::ffi::c_char;
                    break;
                } else {
                    p1_0 = p1_0.offset(1);
                    p2_0 = p2_0.offset(1);
                }
            }
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < ga.ga_len {
            s = *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i_0 as isize);
            if strncmp(
                s,
                b"help-tags\t\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                fputs(s, fd_tags);
            } else {
                fprintf(
                    fd_tags,
                    b"%s\t/*\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
                let mut p1_1: *mut ::core::ffi::c_char = s;
                while *p1_1 as ::core::ffi::c_int != '\t' as ::core::ffi::c_int {
                    if *p1_1 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        || *p1_1 as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    {
                        putc('\\' as ::core::ffi::c_int, fd_tags);
                    }
                    putc(*p1_1 as ::core::ffi::c_int, fd_tags);
                    p1_1 = p1_1.offset(1);
                }
                fprintf(fd_tags, b"*\n\0".as_ptr() as *const ::core::ffi::c_char);
            }
            i_0 += 1;
        }
    }
    let mut _gap: *mut garray_T = &raw mut ga;
    if !(*_gap).ga_data.is_null() {
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < (*_gap).ga_len {
            let mut _item: *mut *mut ::core::ffi::c_void =
                ((*_gap).ga_data as *mut *mut ::core::ffi::c_void).offset(i_1 as isize);
            xfree(*_item);
            i_1 += 1;
        }
    }
    ga_clear(_gap);
    fclose(fd_tags);
}
unsafe extern "C" fn do_helptags(
    mut dirname: *mut ::core::ffi::c_char,
    mut add_help_tags: bool,
    mut ignore_writeerr: bool,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut lang: [::core::ffi::c_char; 2] = [0; 2];
    let mut ext: [::core::ffi::c_char; 5] = [0; 5];
    let mut fname: [::core::ffi::c_char; 8] = [0; 8];
    let mut filecount: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    xstrlcpy(
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        dirname,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
    if !add_pathsep(&raw mut NameBuff as *mut ::core::ffi::c_char)
        || xstrlcat(
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            b"**\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) >= MAXPATHL as size_t
    {
        emsg(gettext(
            &raw const e_fnametoolong as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut buff_list: [*mut ::core::ffi::c_char; 1] =
        [&raw mut NameBuff as *mut ::core::ffi::c_char];
    if gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut buff_list as *mut *mut ::core::ffi::c_char,
        &raw mut filecount,
        &raw mut files,
        EW_FILE as ::core::ffi::c_int | EW_SILENT as ::core::ffi::c_int,
    ) == FAIL
        || filecount == 0 as ::core::ffi::c_int
    {
        semsg(
            gettext(b"E151: No match: %s\0".as_ptr() as *const ::core::ffi::c_char),
            &raw mut NameBuff as *mut ::core::ffi::c_char,
        );
        return;
    }
    let mut j: ::core::ffi::c_int = 0;
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < filecount {
        let mut len: ::core::ffi::c_int = strlen(*files.offset(i as isize)) as ::core::ffi::c_int;
        's_52: {
            if len > 4 as ::core::ffi::c_int {
                if strcasecmp(
                    (*files.offset(i as isize))
                        .offset(len as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize)),
                    b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    lang[0 as ::core::ffi::c_int as usize] = 'e' as ::core::ffi::c_char;
                    lang[1 as ::core::ffi::c_int as usize] = 'n' as ::core::ffi::c_char;
                } else if *(*files.offset(i as isize))
                    .offset((len - 4 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && (*(*files.offset(i as isize))
                        .offset((len - 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *(*files.offset(i as isize))
                                .offset((len - 3 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                    && (*(*files.offset(i as isize))
                        .offset((len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint
                        || *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *(*files.offset(i as isize))
                                .offset((len - 2 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                    && (if (*(*files.offset(i as isize))
                        .offset((len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    }) == 'x' as ::core::ffi::c_int
                {
                    lang[0 as ::core::ffi::c_int as usize] = (if (*(*files.offset(i as isize))
                        .offset((len - 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 3 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    })
                        as ::core::ffi::c_char;
                    lang[1 as ::core::ffi::c_int as usize] = (if (*(*files.offset(i as isize))
                        .offset((len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int)
                        < 'A' as ::core::ffi::c_int
                        || *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            > 'Z' as ::core::ffi::c_int
                    {
                        *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                    } else {
                        *(*files.offset(i as isize))
                            .offset((len - 2 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    })
                        as ::core::ffi::c_char;
                } else {
                    break 's_52;
                }
                j = 0 as ::core::ffi::c_int;
                while j < ga.ga_len {
                    if strncmp(
                        &raw mut lang as *mut ::core::ffi::c_char,
                        (ga.ga_data as *mut ::core::ffi::c_char).offset(j as isize),
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    j += 2 as ::core::ffi::c_int;
                }
                if j == ga.ga_len {
                    ga_grow(&raw mut ga, 2 as ::core::ffi::c_int);
                    let c2rust_fresh6 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    *(ga.ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh6 as isize) =
                        lang[0 as ::core::ffi::c_int as usize];
                    let c2rust_fresh7 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    *(ga.ga_data as *mut ::core::ffi::c_char).offset(c2rust_fresh7 as isize) =
                        lang[1 as ::core::ffi::c_int as usize];
                }
            }
        }
        i += 1;
    }
    j = 0 as ::core::ffi::c_int;
    while j < ga.ga_len {
        strcpy(
            &raw mut fname as *mut ::core::ffi::c_char,
            b"tags-xx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        fname[5 as ::core::ffi::c_int as usize] =
            *(ga.ga_data as *mut ::core::ffi::c_char).offset(j as isize);
        fname[6 as ::core::ffi::c_int as usize] = *(ga.ga_data as *mut ::core::ffi::c_char)
            .offset((j + 1 as ::core::ffi::c_int) as isize);
        if fname[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == 'e' as ::core::ffi::c_int
            && fname[6 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
        {
            fname[4 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            strcpy(
                &raw mut ext as *mut ::core::ffi::c_char,
                b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            strcpy(
                &raw mut ext as *mut ::core::ffi::c_char,
                b".xxx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            ext[1 as ::core::ffi::c_int as usize] = fname[5 as ::core::ffi::c_int as usize];
            ext[2 as ::core::ffi::c_int as usize] = fname[6 as ::core::ffi::c_int as usize];
        }
        helptags_one(
            dirname,
            &raw mut ext as *mut ::core::ffi::c_char,
            &raw mut fname as *mut ::core::ffi::c_char,
            add_help_tags,
            ignore_writeerr,
        );
        j += 2 as ::core::ffi::c_int;
    }
    ga_clear(&raw mut ga);
    FreeWild(filecount, files);
}
unsafe extern "C" fn helptags_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        do_helptags(
            *fnames.offset(i as isize),
            *(cookie as *mut bool),
            true_0 != 0,
        );
        if !all {
            return true_0 != 0;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ex_helptags(mut eap: *mut exarg_T) {
    let mut xpc: expand_T = expand_T {
        xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_context: 0,
        xp_pattern_len: 0,
        xp_prefix: XP_PREFIX_NONE,
        xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_luaref: 0,
        xp_script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
        xp_backslash: 0,
        xp_shell: false,
        xp_numfiles: 0,
        xp_col: 0,
        xp_selected: 0,
        xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        xp_buf: [0; 256],
        xp_search_dir: kDirectionNotSet,
        xp_pre_incsearch_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    let mut add_help_tags: bool = false_0 != 0;
    if strncmp(
        (*eap).arg,
        b"++t\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*(*eap).arg.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        add_help_tags = true_0 != 0;
        (*eap).arg = skipwhite((*eap).arg.offset(3 as ::core::ffi::c_int as isize));
    }
    if strcmp((*eap).arg, b"ALL\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        do_in_path(
            p_rtp,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            b"doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                helptags_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut add_help_tags as *mut ::core::ffi::c_void,
        );
    } else {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_DIRECTORIES as ::core::ffi::c_int;
        let mut dirname: *mut ::core::ffi::c_char = ExpandOne(
            &raw mut xpc,
            (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            WILD_LIST_NOTFOUND as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int,
            WILD_EXPAND_FREE as ::core::ffi::c_int,
        );
        if dirname.is_null() || !os_isdir(dirname) {
            semsg(
                gettext(b"E150: Not a directory: %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        } else {
            do_helptags(dirname, add_help_tags, false_0 != 0);
        }
        xfree(dirname as *mut ::core::ffi::c_void);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
