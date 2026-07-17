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
    fn memmove(
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
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut p_fic: ::core::ffi::c_int;
    static mut p_path: *mut ::core::ffi::c_char;
    static mut p_cdpath: *mut ::core::ffi::c_char;
    static mut p_su: *mut ::core::ffi::c_char;
    static mut p_wig: *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn rem_backslash(str: *const ::core::ffi::c_char) -> bool;
    fn backslash_halve(p: *mut ::core::ffi::c_char);
    fn backslash_halve_save(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn globpath(
        path: *mut ::core::ffi::c_char,
        file: *mut ::core::ffi::c_char,
        ga: *mut garray_T,
        expand_options: ::core::ffi::c_int,
        dirs: bool,
    );
    fn eval_to_string(
        arg: *mut ::core::ffi::c_char,
        join_list: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    fn eval_vars(
        src: *mut ::core::ffi::c_char,
        srcstart: *const ::core::ffi::c_char,
        usedlen: *mut size_t,
        lnump: *mut linenr_T,
        errormsg: *mut *const ::core::ffi::c_char,
        escaped: *mut ::core::ffi::c_int,
        empty_is_error: bool,
    ) -> *mut ::core::ffi::c_char;
    fn match_file_list(
        list: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        ffname: *mut ::core::ffi::c_char,
    ) -> bool;
    fn file_pat_to_reg_pat(
        pat: *const ::core::ffi::c_char,
        pat_end: *const ::core::ffi::c_char,
        allow_dirs: *mut ::core::ffi::c_char,
        no_bslash: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_remove_duplicate_strings(gap: *mut garray_T);
    fn ga_concat_strings(
        gap: *const garray_T,
        sep: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_toupper(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_isalpha(a: ::core::ffi::c_int) -> bool;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_strcmp_ic(
        ic: bool,
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static mut emsg_off: ::core::ffi::c_int;
    static mut curbuf: *mut buf_T;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut got_int: bool;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_can_exe(
        name: *const ::core::ffi::c_char,
        abspath: *mut *mut ::core::ffi::c_char,
        use_path: bool,
    ) -> bool;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_file_is_readable(name: *const ::core::ffi::c_char) -> bool;
    fn os_scandir(dir: *mut Directory, path: *const ::core::ffi::c_char) -> bool;
    fn os_scandir_next(dir: *mut Directory) -> *const ::core::ffi::c_char;
    fn os_closedir(dir: *mut Directory);
    fn os_fileinfo(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_link(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_id_equal(file_info_1: *const FileInfo, file_info_2: *const FileInfo) -> bool;
    fn os_fileid(path: *const ::core::ffi::c_char, file_id: *mut FileID) -> bool;
    fn os_fileid_equal(file_id_1: *const FileID, file_id_2: *const FileID) -> bool;
    fn os_realpath(
        name: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn os_breakcheck();
    fn os_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn expand_env_save_opt(src: *mut ::core::ffi::c_char, one: bool) -> *mut ::core::ffi::c_char;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn vim_env_iter(
        delim: ::core::ffi::c_char,
        val: *const ::core::ffi::c_char,
        iter: *const ::core::ffi::c_void,
        dir: *mut *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_void;
    fn os_expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn get_cmd_output(
        cmd: *mut ::core::ffi::c_char,
        infile: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        ret_len: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type size_t = usize;
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __mode_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type gid_t = __gid_t;
pub type mode_t = __mode_t;
pub type uid_t = __uid_t;
pub type off_t = __off_t;
pub type ssize_t = isize;
pub type time_t = __time_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
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
pub type uv_mutex_t = pthread_mutex_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_17,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_12,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_close_cb = Option<unsafe extern "C" fn(*mut uv_handle_t) -> ()>;
pub type uv_handle_type = ::core::ffi::c_uint;
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub type uv_loop_t = uv_loop_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_16,
    pub internal_fields: *mut ::core::ffi::c_void,
    pub stop_flag: ::core::ffi::c_uint,
    pub flags: ::core::ffi::c_ulong,
    pub backend_fd: ::core::ffi::c_int,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub watchers: *mut *mut uv__io_t,
    pub nwatchers: ::core::ffi::c_uint,
    pub nfds: ::core::ffi::c_uint,
    pub wq: uv__queue,
    pub wq_mutex: uv_mutex_t,
    pub wq_async: uv_async_t,
    pub cloexec_lock: uv_rwlock_t,
    pub closing_handles: *mut uv_handle_t,
    pub process_handles: uv__queue,
    pub prepare_handles: uv__queue,
    pub check_handles: uv__queue,
    pub idle_handles: uv__queue,
    pub async_handles: uv__queue,
    pub async_unused: Option<unsafe extern "C" fn() -> ()>,
    pub async_io_watcher: uv__io_t,
    pub async_wfd: ::core::ffi::c_int,
    pub timer_heap: C2Rust_Unnamed_15,
    pub timer_counter: uint64_t,
    pub time: uint64_t,
    pub signal_pipefd: [::core::ffi::c_int; 2],
    pub signal_io_watcher: uv__io_t,
    pub child_watcher: uv_signal_t,
    pub emfile_fd: ::core::ffi::c_int,
    pub inotify_read_watcher: uv__io_t,
    pub inotify_watchers: *mut ::core::ffi::c_void,
    pub inotify_fd: ::core::ffi::c_int,
}
pub type uv__io_t = uv__io_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__io_s {
    pub cb: uv__io_cb,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub pevents: ::core::ffi::c_uint,
    pub events: ::core::ffi::c_uint,
    pub fd: ::core::ffi::c_int,
}
pub type uv__io_cb =
    Option<unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> ()>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_14,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_13,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_16 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type uv_file = ::core::ffi::c_int;
pub type uv_req_type = ::core::ffi::c_uint;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
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
pub struct uv__work {
    pub work: Option<unsafe extern "C" fn(*mut uv__work) -> ()>,
    pub done: Option<unsafe extern "C" fn(*mut uv__work, ::core::ffi::c_int) -> ()>,
    pub loop_0: *mut uv_loop_s,
    pub wq: uv__queue,
}
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
pub type uv_dirent_t = uv_dirent_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_dirent_s {
    pub name: *const ::core::ffi::c_char,
    pub type_0: uv_dirent_type_t,
}
pub type uv_dirent_type_t = ::core::ffi::c_uint;
pub const UV_DIRENT_BLOCK: uv_dirent_type_t = 7;
pub const UV_DIRENT_CHAR: uv_dirent_type_t = 6;
pub const UV_DIRENT_SOCKET: uv_dirent_type_t = 5;
pub const UV_DIRENT_FIFO: uv_dirent_type_t = 4;
pub const UV_DIRENT_LINK: uv_dirent_type_t = 3;
pub const UV_DIRENT_DIR: uv_dirent_type_t = 2;
pub const UV_DIRENT_FILE: uv_dirent_type_t = 1;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_fs_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub fs_type: uv_fs_type,
    pub loop_0: *mut uv_loop_t,
    pub cb: uv_fs_cb,
    pub result: ssize_t,
    pub ptr: *mut ::core::ffi::c_void,
    pub path: *const ::core::ffi::c_char,
    pub statbuf: uv_stat_t,
    pub new_path: *const ::core::ffi::c_char,
    pub file: uv_file,
    pub flags: ::core::ffi::c_int,
    pub mode: mode_t,
    pub nbufs: ::core::ffi::c_uint,
    pub bufs: *mut uv_buf_t,
    pub off: off_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
    pub atime: ::core::ffi::c_double,
    pub mtime: ::core::ffi::c_double,
    pub work_req: uv__work,
    pub bufsml: [uv_buf_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec_t {
    pub tv_sec: ::core::ffi::c_long,
    pub tv_nsec: ::core::ffi::c_long,
}
pub type uv_fs_cb = Option<unsafe extern "C" fn(*mut uv_fs_t) -> ()>;
pub type uv_fs_t = uv_fs_s;
pub type uv_fs_type = ::core::ffi::c_int;
pub const UV_FS_LUTIME: uv_fs_type = 36;
pub const UV_FS_MKSTEMP: uv_fs_type = 35;
pub const UV_FS_STATFS: uv_fs_type = 34;
pub const UV_FS_CLOSEDIR: uv_fs_type = 33;
pub const UV_FS_READDIR: uv_fs_type = 32;
pub const UV_FS_OPENDIR: uv_fs_type = 31;
pub const UV_FS_LCHOWN: uv_fs_type = 30;
pub const UV_FS_COPYFILE: uv_fs_type = 29;
pub const UV_FS_REALPATH: uv_fs_type = 28;
pub const UV_FS_FCHOWN: uv_fs_type = 27;
pub const UV_FS_CHOWN: uv_fs_type = 26;
pub const UV_FS_READLINK: uv_fs_type = 25;
pub const UV_FS_SYMLINK: uv_fs_type = 24;
pub const UV_FS_LINK: uv_fs_type = 23;
pub const UV_FS_SCANDIR: uv_fs_type = 22;
pub const UV_FS_RENAME: uv_fs_type = 21;
pub const UV_FS_MKDTEMP: uv_fs_type = 20;
pub const UV_FS_MKDIR: uv_fs_type = 19;
pub const UV_FS_RMDIR: uv_fs_type = 18;
pub const UV_FS_UNLINK: uv_fs_type = 17;
pub const UV_FS_FDATASYNC: uv_fs_type = 16;
pub const UV_FS_FSYNC: uv_fs_type = 15;
pub const UV_FS_FCHMOD: uv_fs_type = 14;
pub const UV_FS_CHMOD: uv_fs_type = 13;
pub const UV_FS_ACCESS: uv_fs_type = 12;
pub const UV_FS_FUTIME: uv_fs_type = 11;
pub const UV_FS_UTIME: uv_fs_type = 10;
pub const UV_FS_FTRUNCATE: uv_fs_type = 9;
pub const UV_FS_FSTAT: uv_fs_type = 8;
pub const UV_FS_LSTAT: uv_fs_type = 7;
pub const UV_FS_STAT: uv_fs_type = 6;
pub const UV_FS_SENDFILE: uv_fs_type = 5;
pub const UV_FS_WRITE: uv_fs_type = 4;
pub const UV_FS_READ: uv_fs_type = 3;
pub const UV_FS_CLOSE: uv_fs_type = 2;
pub const UV_FS_OPEN: uv_fs_type = 1;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const UV_FS_UNKNOWN: uv_fs_type = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_18 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_18 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_18 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_18 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_18 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_18 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_18 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_18 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_18 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_18 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_18 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_18 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_18 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_18 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_18 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_18 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_19 = 64;
pub const kShellOptWrite: C2Rust_Unnamed_19 = 32;
pub const kShellOptRead: C2Rust_Unnamed_19 = 16;
pub const kShellOptSilent: C2Rust_Unnamed_19 = 8;
pub const kShellOptDoOut: C2Rust_Unnamed_19 = 4;
pub const kShellOptExpand: C2Rust_Unnamed_19 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_20 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_20 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_20 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_20 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_20 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_20 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_20 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_20 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_20 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_20 = 512;
pub const EW_ICASE: C2Rust_Unnamed_20 = 256;
pub const EW_PATH: C2Rust_Unnamed_20 = 128;
pub const EW_EXEC: C2Rust_Unnamed_20 = 64;
pub const EW_SILENT: C2Rust_Unnamed_20 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_20 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_20 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_20 = 4;
pub const EW_FILE: C2Rust_Unnamed_20 = 2;
pub const EW_DIR: C2Rust_Unnamed_20 = 1;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub const URL_BACKSLASH: C2Rust_Unnamed_21 = 2;
pub const URL_SLASH: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn path_full_compare(
    s1: *mut ::core::ffi::c_char,
    s2: *mut ::core::ffi::c_char,
    checkname: bool,
    expandenv: bool,
) -> FileComparison {
    let mut expanded1: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut full1: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut full2: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut file_id_1: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    let mut file_id_2: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if expandenv {
        expand_env(s1, &raw mut expanded1 as *mut ::core::ffi::c_char, MAXPATHL);
    } else {
        xstrlcpy(
            &raw mut expanded1 as *mut ::core::ffi::c_char,
            s1,
            MAXPATHL as size_t,
        );
    }
    let mut id_ok_1: bool = os_fileid(
        &raw mut expanded1 as *mut ::core::ffi::c_char,
        &raw mut file_id_1,
    );
    let mut id_ok_2: bool = os_fileid(s2, &raw mut file_id_2);
    if !id_ok_1 && !id_ok_2 {
        if checkname {
            vim_FullName(
                &raw mut expanded1 as *mut ::core::ffi::c_char,
                &raw mut full1 as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
            vim_FullName(
                s2,
                &raw mut full2 as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                false_0 != 0,
            );
            if path_fnamecmp(
                &raw mut full1 as *mut ::core::ffi::c_char,
                &raw mut full2 as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                return kEqualFileNames;
            }
        }
        return kBothFilesMissing;
    }
    if !id_ok_1 || !id_ok_2 {
        return kOneFileMissing;
    }
    if os_fileid_equal(&raw mut file_id_1, &raw mut file_id_2) {
        return kEqualFiles;
    }
    return kDifferentFiles;
}
#[no_mangle]
pub unsafe extern "C" fn path_tail(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if fname.is_null() {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut tail: *const ::core::ffi::c_char = get_past_head(fname);
    let mut p: *const ::core::ffi::c_char = tail;
    while *p as ::core::ffi::c_int != NUL {
        if vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
            tail = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return tail as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn path_tail_with_sep(
    mut fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut past_head: *mut ::core::ffi::c_char = get_past_head(fname);
    let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
    while tail > past_head && after_pathsep(fname, tail) != 0 {
        tail = tail.offset(-1);
    }
    return tail;
}
#[no_mangle]
pub unsafe extern "C" fn invocation_path_tail(
    mut invocation: *const ::core::ffi::c_char,
    mut len: *mut size_t,
) -> *const ::core::ffi::c_char {
    let mut tail: *const ::core::ffi::c_char = get_past_head(invocation);
    let mut p: *const ::core::ffi::c_char = tail;
    while *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
        let mut was_sep: bool = vim_ispathsep_nocolon(*p as ::core::ffi::c_int);
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        if was_sep {
            tail = p;
        }
    }
    if !len.is_null() {
        *len = p.offset_from(tail) as size_t;
    }
    return tail;
}
#[no_mangle]
pub unsafe extern "C" fn path_next_component(
    mut fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *fname as ::core::ffi::c_int != NUL && !vim_ispathsep(*fname as ::core::ffi::c_int) {
        fname = fname.offset(utfc_ptr2len(fname as *mut ::core::ffi::c_char) as isize);
    }
    if *fname as ::core::ffi::c_int != NUL {
        fname = fname.offset(1);
    }
    return fname;
}
#[no_mangle]
pub unsafe extern "C" fn path_head_length() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn is_path_head(mut path: *const ::core::ffi::c_char) -> bool {
    return vim_ispathsep(*path as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn get_past_head(
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut retval: *const ::core::ffi::c_char = path;
    while vim_ispathsep(*retval as ::core::ffi::c_int) {
        retval = retval.offset(1);
    }
    return retval as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn vim_ispathsep(mut c: ::core::ffi::c_int) -> bool {
    return c == '/' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_ispathsep_nocolon(mut c: ::core::ffi::c_int) -> bool {
    return vim_ispathsep(c);
}
#[no_mangle]
pub unsafe extern "C" fn vim_ispathlistsep(mut c: ::core::ffi::c_int) -> bool {
    return c == ':' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn shorten_dir_len(
    mut str: *mut ::core::ffi::c_char,
    mut trim_len: ::core::ffi::c_int,
) {
    let mut tail: *mut ::core::ffi::c_char = path_tail(str);
    let mut d: *mut ::core::ffi::c_char = str;
    let mut skip: bool = false_0 != 0;
    let mut dirchunk_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = str;
    loop {
        if s >= tail {
            let c2rust_fresh0 = d;
            d = d.offset(1);
            *c2rust_fresh0 = *s;
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
        } else if vim_ispathsep(*s as ::core::ffi::c_int) {
            let c2rust_fresh1 = d;
            d = d.offset(1);
            *c2rust_fresh1 = *s;
            skip = false_0 != 0;
            dirchunk_len = 0 as ::core::ffi::c_int;
        } else if !skip {
            let c2rust_fresh2 = d;
            d = d.offset(1);
            *c2rust_fresh2 = *s;
            if *s as ::core::ffi::c_int != '~' as ::core::ffi::c_int
                && *s as ::core::ffi::c_int != '.' as ::core::ffi::c_int
            {
                dirchunk_len += 1;
                if dirchunk_len >= trim_len {
                    skip = true_0 != 0;
                }
            }
            let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
            loop {
                l -= 1;
                if l <= 0 as ::core::ffi::c_int {
                    break;
                }
                s = s.offset(1);
                let c2rust_fresh3 = d;
                d = d.offset(1);
                *c2rust_fresh3 = *s;
            }
        }
        s = s.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn shorten_dir(mut str: *mut ::core::ffi::c_char) {
    shorten_dir_len(str, 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn dir_of_file_exists(mut fname: *mut ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
    if p == fname {
        return true_0 != 0;
    }
    let mut c: ::core::ffi::c_char = *p;
    *p = NUL as ::core::ffi::c_char;
    let mut retval: bool = os_isdir(fname);
    *p = c;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn path_fnamecmp(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return mb_strcmp_ic(p_fic != 0, fname1, fname2);
}
#[no_mangle]
pub unsafe extern "C" fn path_fnamencmp(
    fname1: *const ::core::ffi::c_char,
    fname2: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if p_fic != 0 {
        return mb_strnicmp(fname1, fname2, len);
    }
    return strncmp(fname1, fname2, len);
}
#[inline]
unsafe extern "C" fn do_concat_fnames(
    mut fname1: *mut ::core::ffi::c_char,
    len1: size_t,
    mut fname2: *const ::core::ffi::c_char,
    len2: size_t,
    sep: bool,
) -> *mut ::core::ffi::c_char {
    if sep as ::core::ffi::c_int != 0
        && *fname1 as ::core::ffi::c_int != 0
        && after_pathsep(fname1, fname1.offset(len1 as isize)) == 0
    {
        *fname1.offset(len1 as isize) = PATHSEP as ::core::ffi::c_char;
        memmove(
            fname1
                .offset(len1 as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            fname2 as *const ::core::ffi::c_void,
            len2.wrapping_add(1 as size_t),
        );
    } else {
        memmove(
            fname1.offset(len1 as isize) as *mut ::core::ffi::c_void,
            fname2 as *const ::core::ffi::c_void,
            len2.wrapping_add(1 as size_t),
        );
    }
    return fname1;
}
#[no_mangle]
pub unsafe extern "C" fn concat_fnames(
    mut fname1: *const ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    let len1: size_t = strlen(fname1);
    let len2: size_t = strlen(fname2);
    let mut dest: *mut ::core::ffi::c_char =
        xmalloc(len1.wrapping_add(len2).wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
    memmove(
        dest as *mut ::core::ffi::c_void,
        fname1 as *const ::core::ffi::c_void,
        len1.wrapping_add(1 as size_t),
    );
    return do_concat_fnames(dest, len1, fname2, len2, sep);
}
#[no_mangle]
pub unsafe extern "C" fn concat_fnames_realloc(
    mut fname1: *mut ::core::ffi::c_char,
    mut fname2: *const ::core::ffi::c_char,
    mut sep: bool,
) -> *mut ::core::ffi::c_char {
    let len1: size_t = strlen(fname1);
    let len2: size_t = strlen(fname2);
    return do_concat_fnames(
        xrealloc(
            fname1 as *mut ::core::ffi::c_void,
            len1.wrapping_add(len2).wrapping_add(3 as size_t),
        ) as *mut ::core::ffi::c_char,
        len1,
        fname2,
        len2,
        sep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn add_pathsep(mut p: *mut ::core::ffi::c_char) -> bool {
    let len: size_t = strlen(p);
    if *p as ::core::ffi::c_int != NUL && after_pathsep(p, p.offset(len as isize)) == 0 {
        let pathsep_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
        if len > (MAXPATHL as size_t).wrapping_sub(pathsep_len) {
            return false_0 != 0;
        }
        memcpy(
            p.offset(len as isize) as *mut ::core::ffi::c_void,
            PATHSEPSTR.as_ptr() as *const ::core::ffi::c_void,
            pathsep_len,
        );
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn FullName_save(
    mut fname: *const ::core::ffi::c_char,
    mut force: bool,
) -> *mut ::core::ffi::c_char {
    if fname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if vim_FullName(fname, buf, MAXPATHL as size_t, force) == FAIL {
        xfree(buf as *mut ::core::ffi::c_void);
        return xstrdup(fname);
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn save_abs_path(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if !path_is_absolute(name) {
        return FullName_save(name, true_0 != 0);
    }
    return xstrdup(name);
}
#[no_mangle]
pub unsafe extern "C" fn path_has_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    while *p != 0 {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else {
            let mut wildcards: *const ::core::ffi::c_char =
                b"*?[{`'$\0".as_ptr() as *const ::core::ffi::c_char;
            if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null()
                || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '~' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                return true_0 != 0;
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn pstrcmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return pathcmp(
        *(a as *mut *mut ::core::ffi::c_char),
        *(b as *mut *mut ::core::ffi::c_char),
        -1 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn path_has_exp_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    while *p as ::core::ffi::c_int != NUL {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else {
            let mut wildcards: *const ::core::ffi::c_char =
                b"*?[{\0".as_ptr() as *const ::core::ffi::c_char;
            if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null() {
                return true_0 != 0;
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> size_t {
    return do_path_expand(gap, path, 0 as size_t, flags, false_0 != 0);
}
unsafe extern "C" fn scandir_next_with_dots(mut dir: *mut Directory) -> *const ::core::ffi::c_char {
    static mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if dir.is_null() {
        count = 0 as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    count += 1 as ::core::ffi::c_int;
    if count == 1 as ::core::ffi::c_int || count == 2 as ::core::ffi::c_int {
        return if count == 1 as ::core::ffi::c_int {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"..\0".as_ptr() as *const ::core::ffi::c_char
        };
    }
    return os_scandir_next(dir);
}
unsafe extern "C" fn do_path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut wildoff: size_t,
    mut flags: ::core::ffi::c_int,
    mut didstar: bool,
) -> size_t {
    let mut start_len: ::core::ffi::c_int = (*gap).ga_len;
    let mut starstar: bool = false_0 != 0;
    static mut stardepth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if stardepth > 0 as ::core::ffi::c_int && flags & EW_NOBREAK as ::core::ffi::c_int == 0 {
        os_breakcheck();
        if got_int {
            return 0 as size_t;
        }
    }
    let buflen: size_t = strlen(path).wrapping_add(MAXPATHL as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = buf;
    let mut s: *mut ::core::ffi::c_char = buf;
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_end: *const ::core::ffi::c_char = path;
    while *path_end as ::core::ffi::c_int != NUL {
        if path_end >= path.offset(wildoff as isize)
            && rem_backslash(path_end) as ::core::ffi::c_int != 0
        {
            let c2rust_fresh5 = path_end;
            path_end = path_end.offset(1);
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = *c2rust_fresh5;
        } else if vim_ispathsep_nocolon(*path_end as ::core::ffi::c_int) {
            if !e.is_null() {
                break;
            }
            s = p.offset(1 as ::core::ffi::c_int as isize);
        } else if path_end >= path.offset(wildoff as isize)
            && (!vim_strchr(
                b"*?[{~$\0".as_ptr() as *const ::core::ffi::c_char,
                *path_end as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
                || p_fic == 0
                    && flags & EW_ICASE as ::core::ffi::c_int != 0
                    && mb_isalpha(utf_ptr2char(path_end)) as ::core::ffi::c_int != 0)
        {
            e = p;
        }
        let mut charlen: ::core::ffi::c_int = utfc_ptr2len(path_end);
        memcpy(
            p as *mut ::core::ffi::c_void,
            path_end as *const ::core::ffi::c_void,
            charlen as size_t,
        );
        p = p.offset(charlen as isize);
        path_end = path_end.offset(charlen as isize);
    }
    e = p;
    *e = NUL as ::core::ffi::c_char;
    p = buf.offset(wildoff as isize);
    while p < s {
        if rem_backslash(p) {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            e = e.offset(-1);
            s = s.offset(-1);
        }
        p = p.offset(1);
    }
    p = s;
    while p < e {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
        {
            starstar = true_0 != 0;
        }
        p = p.offset(1);
    }
    let mut starts_with_dot: ::core::ffi::c_int =
        (*s as ::core::ffi::c_int == '.' as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
        s,
        e,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0,
    );
    if pat.is_null() {
        xfree(buf as *mut ::core::ffi::c_void);
        return 0 as size_t;
    }
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = flags & EW_ICASE as ::core::ffi::c_int != 0 || p_fic != 0;
    if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
        emsg_silent += 1;
    }
    let mut nobreak: bool = flags & EW_NOBREAK as ::core::ffi::c_int != 0;
    regmatch.regprog = vim_regcomp(
        pat,
        RE_MAGIC
            | (if nobreak as ::core::ffi::c_int != 0 {
                RE_NOBREAK
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
        emsg_silent -= 1;
    }
    xfree(pat as *mut ::core::ffi::c_void);
    if regmatch.regprog.is_null()
        && flags & EW_NOTWILD as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        xfree(buf as *mut ::core::ffi::c_void);
        return 0 as size_t;
    }
    let mut len: size_t = s.offset_from(buf) as size_t;
    if !didstar
        && stardepth < 100 as ::core::ffi::c_int
        && starstar as ::core::ffi::c_int != 0
        && e.offset_from(s) == 2 as isize
        && *path_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
    {
        vim_snprintf(
            s,
            buflen.wrapping_sub(len),
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            path_end.offset(1 as ::core::ffi::c_int as isize),
        );
        stardepth += 1;
        do_path_expand(gap, buf, len, flags, true_0 != 0);
        stardepth -= 1;
    }
    *s = NUL as ::core::ffi::c_char;
    let mut dir: Directory = Directory {
        request: uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        },
        ent: uv_dirent_t {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            type_0: UV_DIRENT_UNKNOWN,
        },
    };
    let mut dirpath: *mut ::core::ffi::c_char = (if *buf as ::core::ffi::c_int == NUL {
        b".\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        buf as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    if os_file_is_readable(dirpath) as ::core::ffi::c_int != 0
        && os_scandir(&raw mut dir, dirpath) as ::core::ffi::c_int != 0
    {
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        scandir_next_with_dots(::core::ptr::null_mut::<Directory>());
        while !got_int && {
            name = scandir_next_with_dots(&raw mut dir);
            !name.is_null()
        } {
            len = s.offset_from(buf) as size_t;
            if !((*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '.' as ::core::ffi::c_int
                || starts_with_dot != 0
                || flags & EW_DODOT as ::core::ffi::c_int != 0
                    && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && (*name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '.' as ::core::ffi::c_int
                        || *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL))
                && (!regmatch.regprog.is_null()
                    && vim_regexec(&raw mut regmatch, name, 0 as colnr_T) as ::core::ffi::c_int
                        != 0
                    || flags & EW_NOTWILD as ::core::ffi::c_int != 0
                        && path_fnamencmp(
                            path.offset(len as isize),
                            name,
                            e.offset_from(s) as size_t,
                        ) == 0 as ::core::ffi::c_int))
            {
                continue;
            }
            len = len.wrapping_add(vim_snprintf(
                s,
                buflen.wrapping_sub(len),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            ) as size_t);
            if len.wrapping_add(1 as size_t) >= buflen {
                continue;
            }
            if starstar as ::core::ffi::c_int != 0 && stardepth < 100 as ::core::ffi::c_int {
                vim_snprintf(
                    buf.offset(len as isize),
                    buflen.wrapping_sub(len),
                    b"/**%s\0".as_ptr() as *const ::core::ffi::c_char,
                    path_end,
                );
                stardepth += 1;
                do_path_expand(gap, buf, len.wrapping_add(1 as size_t), flags, true_0 != 0);
                stardepth -= 1;
            }
            vim_snprintf(
                buf.offset(len as isize),
                buflen.wrapping_sub(len),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                path_end,
            );
            if path_has_exp_wildcard(path_end) {
                if stardepth < 100 as ::core::ffi::c_int {
                    stardepth += 1;
                    do_path_expand(gap, buf, len.wrapping_add(1 as size_t), flags, false_0 != 0);
                    stardepth -= 1;
                }
            } else {
                let mut file_info: FileInfo = FileInfo {
                    stat: uv_stat_t {
                        st_dev: 0,
                        st_mode: 0,
                        st_nlink: 0,
                        st_uid: 0,
                        st_gid: 0,
                        st_rdev: 0,
                        st_ino: 0,
                        st_size: 0,
                        st_blksize: 0,
                        st_blocks: 0,
                        st_flags: 0,
                        st_gen: 0,
                        st_atim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_mtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_ctim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_birthtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                    },
                };
                if *path_end as ::core::ffi::c_int != NUL {
                    backslash_halve(
                        buf.offset(len as isize)
                            .offset(1 as ::core::ffi::c_int as isize),
                    );
                }
                if if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
                    os_fileinfo_link(buf, &raw mut file_info) as ::core::ffi::c_int
                } else {
                    os_path_exists(buf) as ::core::ffi::c_int
                } != 0
                {
                    addfile(gap, buf, flags);
                }
            }
        }
        os_closedir(&raw mut dir);
    }
    xfree(buf as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    let mut matches: size_t = ((*gap).ga_len - start_len) as size_t;
    if matches > 0 as size_t && !got_int {
        qsort(
            ((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(start_len as isize)
                as *mut ::core::ffi::c_void,
            matches,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
            Some(
                pstrcmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    return matches;
}
unsafe extern "C" fn find_previous_pathsep(
    mut path: *mut ::core::ffi::c_char,
    mut psep: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *psep > path && vim_ispathsep(**psep as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        *psep = (*psep).offset(-1);
    }
    while *psep > path {
        if vim_ispathsep(**psep as ::core::ffi::c_int) {
            return OK;
        }
        *psep = (*psep).offset(
            -((utf_head_off(path, (*psep).offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return FAIL;
}
unsafe extern "C" fn is_unique(
    mut maybe_unique: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut i: ::core::ffi::c_int,
) -> bool {
    let mut candidate_len: size_t = strlen(maybe_unique);
    let mut other_paths: *mut *mut ::core::ffi::c_char =
        (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < (*gap).ga_len {
        if j != i {
            let mut other_path_len: size_t = strlen(*other_paths.offset(j as isize));
            if other_path_len >= candidate_len {
                let mut rival: *mut ::core::ffi::c_char = (*other_paths.offset(j as isize))
                    .offset(other_path_len as isize)
                    .offset(-(candidate_len as isize));
                if path_fnamecmp(maybe_unique, rival) == 0 as ::core::ffi::c_int
                    && (rival == *other_paths.offset(j as isize)
                        || vim_ispathsep(*rival.offset(-(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_int) as ::core::ffi::c_int
                            != 0)
                {
                    return false_0 != 0;
                }
            }
        }
        j += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn expand_path_option(
    mut curdir: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) {
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut curdirlen: size_t = 0 as size_t;
    while *path_option as ::core::ffi::c_int != NUL {
        let mut buflen: size_t = copy_option_part(
            &raw mut path_option,
            buf,
            MAXPATHL as size_t,
            b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if !vim_strchr(buf, '`' as ::core::ffi::c_int).is_null() {
            continue;
        }
        if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || vim_ispathsep(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            if (*curbuf).b_ffname.is_null() {
                continue;
            }
            let mut p: *mut ::core::ffi::c_char = path_tail((*curbuf).b_ffname);
            let mut plen: size_t = p.offset_from((*curbuf).b_ffname) as size_t;
            if plen.wrapping_add(strlen(buf)) >= MAXPATHL as size_t {
                continue;
            }
            if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                *buf.offset(plen as isize) = NUL as ::core::ffi::c_char;
            } else {
                memmove(
                    buf.offset(plen as isize) as *mut ::core::ffi::c_void,
                    buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    buflen.wrapping_sub(2 as size_t).wrapping_add(1 as size_t),
                );
            }
            memmove(
                buf as *mut ::core::ffi::c_void,
                (*curbuf).b_ffname as *const ::core::ffi::c_void,
                plen,
            );
            buflen = simplify_filename(buf);
        } else if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            strcpy(buf, curdir);
            if curdirlen == 0 as size_t {
                curdirlen = strlen(curdir);
            }
            buflen = curdirlen;
        } else {
            if path_with_url(buf) != 0 {
                continue;
            }
            if !path_is_absolute(buf) {
                if curdirlen == 0 as size_t {
                    curdirlen = strlen(curdir);
                }
                if curdirlen.wrapping_add(buflen).wrapping_add(3 as size_t) > MAXPATHL as size_t {
                    continue;
                }
                memmove(
                    buf.offset(curdirlen as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    buflen.wrapping_add(1 as size_t),
                );
                strcpy(buf, curdir);
                *buf.offset(curdirlen as isize) = PATHSEP as ::core::ffi::c_char;
                buflen = simplify_filename(buf);
            }
        }
        ga_grow(gap, 1 as ::core::ffi::c_int);
        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) =
            xmemdupz(buf as *const ::core::ffi::c_void, buflen) as *mut ::core::ffi::c_char;
        (*gap).ga_len += 1;
    }
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_path_cutoff(
    mut fname: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) -> *mut ::core::ffi::c_char {
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut path_part: *mut *mut ::core::ffi::c_char =
        (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut cutoff: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *fname.offset(j as isize) as ::core::ffi::c_int
            == *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
            && *fname.offset(j as isize) as ::core::ffi::c_int != NUL
            && *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL
        {
            j += 1;
        }
        if j > maxlen {
            maxlen = j;
            cutoff = fname.offset(j as isize);
        }
        i += 1;
    }
    if !cutoff.is_null() {
        while vim_ispathsep(*cutoff as ::core::ffi::c_int) {
            cutoff = cutoff.offset(utfc_ptr2len(cutoff) as isize);
        }
    }
    return cutoff;
}
unsafe extern "C" fn uniquefy_paths(
    mut gap: *mut garray_T,
    mut pattern: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
) {
    let mut fnames: *mut *mut ::core::ffi::c_char = (*gap).ga_data as *mut *mut ::core::ffi::c_char;
    let mut sort_again: bool = false_0 != 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut path_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut in_curdir: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut short_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ga_remove_duplicate_strings(gap);
    ga_init(
        &raw mut path_ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    let mut len: size_t = strlen(pattern);
    let mut file_pattern: *mut ::core::ffi::c_char =
        xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
    *file_pattern.offset(0 as ::core::ffi::c_int as isize) = '*' as ::core::ffi::c_char;
    *file_pattern.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    strcpy(
        file_pattern.offset(1 as ::core::ffi::c_int as isize),
        pattern,
    );
    let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
        file_pattern,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0,
    );
    xfree(file_pattern as *mut ::core::ffi::c_void);
    if pat.is_null() {
        return;
    }
    regmatch.rm_ic = true_0 != 0;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    xfree(pat as *mut ::core::ffi::c_void);
    if regmatch.regprog.is_null() {
        return;
    }
    let mut curdir: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(curdir, MAXPATHL as size_t);
    expand_path_option(curdir, path_option, &raw mut path_ga);
    in_curdir = xcalloc(
        (*gap).ga_len as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len && !got_int {
        let mut path: *mut ::core::ffi::c_char = *fnames.offset(i as isize);
        let mut dir_end: *const ::core::ffi::c_char = gettail_dir(path);
        len = strlen(path);
        let mut is_in_curdir: bool =
            path_fnamencmp(curdir, path, dir_end.offset_from(path) as size_t)
                == 0 as ::core::ffi::c_int
                && *curdir.offset(dir_end.offset_from(path) as isize) as ::core::ffi::c_int == NUL;
        if is_in_curdir {
            *in_curdir.offset(i as isize) =
                xmemdupz(path as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        }
        let mut path_cutoff: *mut ::core::ffi::c_char = get_path_cutoff(path, &raw mut path_ga);
        if *pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            && *pattern.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
            && vim_ispathsep_nocolon(
                *pattern.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
            && !path_cutoff.is_null()
            && vim_regexec(&raw mut regmatch, path_cutoff, 0 as colnr_T) as ::core::ffi::c_int != 0
            && is_unique(path_cutoff, gap, i) as ::core::ffi::c_int != 0
        {
            sort_again = true_0 != 0;
            memmove(
                path as *mut ::core::ffi::c_void,
                path_cutoff as *const ::core::ffi::c_void,
                strlen(path_cutoff).wrapping_add(1 as size_t),
            );
        } else {
            let mut pathsep_p: *mut ::core::ffi::c_char = path
                .offset(len as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            while find_previous_pathsep(path, &raw mut pathsep_p) != 0 {
                if !(vim_regexec(
                    &raw mut regmatch,
                    pathsep_p.offset(1 as ::core::ffi::c_int as isize),
                    0 as colnr_T,
                ) as ::core::ffi::c_int
                    != 0
                    && is_unique(pathsep_p.offset(1 as ::core::ffi::c_int as isize), gap, i)
                        as ::core::ffi::c_int
                        != 0
                    && !path_cutoff.is_null()
                    && pathsep_p.offset(1 as ::core::ffi::c_int as isize) >= path_cutoff)
                {
                    continue;
                }
                sort_again = true_0 != 0;
                memmove(
                    path as *mut ::core::ffi::c_void,
                    pathsep_p.offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (path
                        .offset(len as isize)
                        .offset_from(pathsep_p.offset(1 as ::core::ffi::c_int as isize))
                        as size_t)
                        .wrapping_add(1 as size_t),
                );
                break;
            }
        }
        if path_is_absolute(path) {
            short_name = path_shorten_fname(path, curdir);
            if !short_name.is_null() && short_name > path.offset(1 as ::core::ffi::c_int as isize) {
                vim_snprintf(
                    path,
                    MAXPATHL as size_t,
                    b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    short_name,
                );
            }
        }
        os_breakcheck();
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*gap).ga_len && !got_int {
        let mut path_0: *mut ::core::ffi::c_char = *in_curdir.offset(i_0 as isize);
        if !path_0.is_null() {
            short_name = path_shorten_fname(path_0, curdir);
            if short_name.is_null() {
                short_name = path_0;
            }
            if is_unique(short_name, gap, i_0) {
                strcpy(*fnames.offset(i_0 as isize), short_name);
            } else {
                let mut rel_pathsize: size_t = (1 as size_t)
                    .wrapping_add(
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    )
                    .wrapping_add(strlen(short_name))
                    .wrapping_add(1 as size_t);
                let mut rel_path: *mut ::core::ffi::c_char =
                    xmalloc(rel_pathsize) as *mut ::core::ffi::c_char;
                vim_snprintf(
                    rel_path,
                    rel_pathsize,
                    b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    short_name,
                );
                xfree(*fnames.offset(i_0 as isize) as *mut ::core::ffi::c_void);
                *fnames.offset(i_0 as isize) = rel_path;
                sort_again = true_0 != 0;
                os_breakcheck();
            }
        }
        i_0 += 1;
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < (*gap).ga_len {
        xfree(*in_curdir.offset(i_1 as isize) as *mut ::core::ffi::c_void);
        i_1 += 1;
    }
    xfree(in_curdir as *mut ::core::ffi::c_void);
    ga_clear_strings(&raw mut path_ga);
    vim_regfree(regmatch.regprog);
    if sort_again {
        ga_remove_duplicate_strings(gap);
    }
}
#[no_mangle]
pub unsafe extern "C" fn gettail_dir(
    fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut dir_end: *const ::core::ffi::c_char = fname;
    let mut next_dir_end: *const ::core::ffi::c_char = fname;
    let mut look_for_sep: bool = true_0 != 0;
    let mut p: *const ::core::ffi::c_char = fname;
    while *p as ::core::ffi::c_int != NUL {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            if look_for_sep {
                next_dir_end = p;
                look_for_sep = false_0 != 0;
            }
        } else {
            if !look_for_sep {
                dir_end = next_dir_end;
            }
            look_for_sep = true_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return dir_end;
}
unsafe extern "C" fn expand_in_path(
    gap: *mut garray_T,
    pattern: *mut ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut path_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut path_option: *mut ::core::ffi::c_char =
        if *(*curbuf).b_p_path as ::core::ffi::c_int == NUL {
            p_path
        } else {
            (*curbuf).b_p_path
        };
    let curdir: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    os_dirname(curdir, MAXPATHL as size_t);
    ga_init(
        &raw mut path_ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if flags & EW_CDPATH as ::core::ffi::c_int != 0 {
        expand_path_option(curdir, p_cdpath, &raw mut path_ga);
    } else {
        expand_path_option(curdir, path_option, &raw mut path_ga);
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    if path_ga.ga_len <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let paths: *mut ::core::ffi::c_char = ga_concat_strings(
        &raw mut path_ga,
        b",\0".as_ptr() as *const ::core::ffi::c_char,
    );
    ga_clear_strings(&raw mut path_ga);
    let mut glob_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & EW_ICASE as ::core::ffi::c_int != 0 {
        glob_flags |= WILD_ICASE as ::core::ffi::c_int;
    }
    if flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
        glob_flags |= WILD_ADD_SLASH as ::core::ffi::c_int;
    }
    globpath(
        paths,
        pattern,
        gap,
        glob_flags,
        flags & EW_CDPATH as ::core::ffi::c_int != 0,
    );
    xfree(paths as *mut ::core::ffi::c_void);
    return (*gap).ga_len;
}
unsafe extern "C" fn has_env_var(mut p: *mut ::core::ffi::c_char) -> bool {
    while *p != 0 {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else if !vim_strchr(
            b"$\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return true_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return false_0 != 0;
}
unsafe extern "C" fn has_special_wildchar(
    mut p: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    while *p != 0 {
        if *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            break;
        }
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\r' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\n' as ::core::ffi::c_int
        {
            p = p.offset(1);
        } else if !vim_strchr(
            SPECIAL_WILDCHAR.as_ptr(),
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                && flags & EW_NOTFOUND as ::core::ffi::c_int == 0)
            {
                if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    && vim_strchr(p, '}' as ::core::ffi::c_int).is_null())
                {
                    if !((*p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
                        && vim_strchr(p, *p as uint8_t as ::core::ffi::c_int).is_null())
                    {
                        return true_0 != 0;
                    }
                }
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn gen_expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut recursive: bool = false_0 != 0;
    let mut add_pat: ::core::ffi::c_int = 0;
    let mut did_expand_in_path: bool = false_0 != 0;
    let mut path_option: *mut ::core::ffi::c_char =
        if *(*curbuf).b_p_path as ::core::ffi::c_int == NUL {
            p_path
        } else {
            (*curbuf).b_p_path
        };
    if recursive {
        return os_expand_wildcards(num_pat, pat, num_file, file, flags);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_pat {
        if has_special_wildchar(*pat.offset(i as isize), flags) as ::core::ffi::c_int != 0
            && !(vim_backtick(*pat.offset(i as isize)) as ::core::ffi::c_int != 0
                && *(*pat.offset(i as isize)).offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int)
        {
            return os_expand_wildcards(num_pat, pat, num_file, file, flags);
        }
        i += 1;
    }
    recursive = true_0 != 0;
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
    );
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < num_pat && !got_int {
        add_pat = -1 as ::core::ffi::c_int;
        p = *pat.offset(i_0 as isize);
        if vim_backtick(p) {
            add_pat = expand_backtick(&raw mut ga, p, flags);
            if add_pat == -1 as ::core::ffi::c_int {
                recursive = false_0 != 0;
                ga_clear_strings(&raw mut ga);
                *num_file = 0 as ::core::ffi::c_int;
                *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                return FAIL;
            }
        } else {
            if has_env_var(p) as ::core::ffi::c_int != 0
                && flags & EW_NOTENV as ::core::ffi::c_int == 0
                || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
            {
                p = expand_env_save_opt(p, true_0 != 0);
                if p.is_null() {
                    p = *pat.offset(i_0 as isize);
                } else if has_env_var(p) as ::core::ffi::c_int != 0
                    || *p as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                {
                    xfree(p as *mut ::core::ffi::c_void);
                    ga_clear_strings(&raw mut ga);
                    i_0 = os_expand_wildcards(
                        num_pat,
                        pat,
                        num_file,
                        file,
                        flags | EW_KEEPDOLLAR as ::core::ffi::c_int,
                    );
                    recursive = false_0 != 0;
                    return i_0;
                }
            }
            if path_has_exp_wildcard(p) as ::core::ffi::c_int != 0
                || flags & EW_ICASE as ::core::ffi::c_int != 0
            {
                if flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int) != 0
                    && !path_is_absolute(p)
                    && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && (vim_ispathsep(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                                && vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0))
                {
                    recursive = false_0 != 0;
                    add_pat = expand_in_path(&raw mut ga, p, flags);
                    recursive = true_0 != 0;
                    did_expand_in_path = true_0 != 0;
                } else {
                    recursive = false_0 != 0;
                    let mut tmp_add_pat: size_t = path_expand(&raw mut ga, p, flags);
                    recursive = true_0 != 0;
                    '_c2rust_label: {
                        if tmp_add_pat <= 2147483647 as ::core::ffi::c_int as size_t {
                        } else {
                            __assert_fail(
                                b"tmp_add_pat <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1375 as ::core::ffi::c_uint,
                                b"int gen_expand_wildcards(int, char **, int *, char ***, int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    add_pat = tmp_add_pat as ::core::ffi::c_int;
                }
            }
        }
        if add_pat == -1 as ::core::ffi::c_int
            || add_pat == 0 as ::core::ffi::c_int && flags & EW_NOTFOUND as ::core::ffi::c_int != 0
        {
            let mut t: *mut ::core::ffi::c_char = backslash_halve_save(p);
            if flags & EW_NOTFOUND as ::core::ffi::c_int != 0 {
                addfile(
                    &raw mut ga,
                    t,
                    flags | EW_DIR as ::core::ffi::c_int | EW_FILE as ::core::ffi::c_int,
                );
            } else {
                addfile(&raw mut ga, t, flags);
            }
            if t != p {
                xfree(t as *mut ::core::ffi::c_void);
            }
        }
        if did_expand_in_path as ::core::ffi::c_int != 0
            && !(ga.ga_len <= 0 as ::core::ffi::c_int)
            && flags & (EW_PATH as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int) != 0
        {
            recursive = false_0 != 0;
            uniquefy_paths(&raw mut ga, p, path_option);
            recursive = true_0 != 0;
        }
        if p != *pat.offset(i_0 as isize) {
            xfree(p as *mut ::core::ffi::c_void);
        }
        i_0 += 1;
    }
    *num_file = ga.ga_len;
    *file = (if !ga.ga_data.is_null() {
        ga.ga_data
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    recursive = false_0 != 0;
    return if flags & EW_EMPTYOK as ::core::ffi::c_int != 0 || !ga.ga_data.is_null() {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn FreeWild(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    if count <= 0 as ::core::ffi::c_int || files.is_null() {
        return;
    }
    loop {
        let c2rust_fresh7 = count;
        count = count - 1;
        if c2rust_fresh7 == 0 {
            break;
        }
        xfree(*files.offset(count as isize) as *mut ::core::ffi::c_void);
    }
    xfree(files as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn vim_backtick(mut p: *mut ::core::ffi::c_char) -> bool {
    return *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *p
            .offset(strlen(p) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int;
}
unsafe extern "C" fn expand_backtick(
    mut gap: *mut garray_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmd: *mut ::core::ffi::c_char = xmemdupz(
        pat.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        strlen(pat).wrapping_sub(2 as size_t),
    ) as *mut ::core::ffi::c_char;
    if *cmd as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
        buffer = eval_to_string(
            cmd.offset(1 as ::core::ffi::c_int as isize),
            true_0 != 0,
            false_0 != 0,
        );
    } else {
        buffer = get_cmd_output(
            cmd,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            if flags & EW_SILENT as ::core::ffi::c_int != 0 {
                kShellOptSilent as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            ::core::ptr::null_mut::<size_t>(),
        );
    }
    xfree(cmd as *mut ::core::ffi::c_void);
    if buffer.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    cmd = buffer;
    while *cmd as ::core::ffi::c_int != NUL {
        cmd = skipwhite(cmd);
        p = cmd;
        while *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if p > cmd {
            let mut i: ::core::ffi::c_char = *p;
            *p = NUL as ::core::ffi::c_char;
            addfile(gap, cmd, flags);
            *p = i;
            cnt += 1;
        }
        cmd = p;
        while *cmd as ::core::ffi::c_int != NUL
            && (*cmd as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int)
        {
            cmd = cmd.offset(1);
        }
    }
    xfree(buffer as *mut ::core::ffi::c_void);
    return cnt;
}
#[no_mangle]
pub unsafe extern "C" fn addfile(
    mut gap: *mut garray_T,
    mut f: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut isdir: bool = false;
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    if flags & EW_NOTFOUND as ::core::ffi::c_int == 0
        && (if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
            !os_fileinfo_link(f, &raw mut file_info) as ::core::ffi::c_int
        } else {
            !os_path_exists(f) as ::core::ffi::c_int
        }) != 0
    {
        return;
    }
    isdir = os_isdir(f);
    if isdir as ::core::ffi::c_int != 0 && flags & EW_DIR as ::core::ffi::c_int == 0
        || !isdir && flags & EW_FILE as ::core::ffi::c_int == 0
    {
        return;
    }
    if !isdir
        && flags & EW_EXEC as ::core::ffi::c_int != 0
        && !os_can_exe(
            f,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            flags & EW_SHELLCMD as ::core::ffi::c_int == 0,
        )
    {
        return;
    }
    let mut p: *mut ::core::ffi::c_char = xmalloc(
        strlen(f)
            .wrapping_add(1 as size_t)
            .wrapping_add(isdir as size_t),
    ) as *mut ::core::ffi::c_char;
    strcpy(p, f);
    if isdir as ::core::ffi::c_int != 0 && flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
        add_pathsep(p);
    }
    ga_grow(gap, 1 as ::core::ffi::c_int);
    *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) = p;
    (*gap).ga_len += 1;
}
#[no_mangle]
pub unsafe extern "C" fn simplify_filename(mut filename: *mut ::core::ffi::c_char) -> size_t {
    let mut components: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut stripping_disabled: bool = false_0 != 0;
    let mut relative: bool = true_0 != 0;
    let mut p: *mut ::core::ffi::c_char = filename;
    if vim_ispathsep(*p as ::core::ffi::c_int) {
        relative = false_0 != 0;
        loop {
            p = p.offset(1);
            if !vim_ispathsep(*p as ::core::ffi::c_int) {
                break;
            }
        }
    }
    let mut start: *mut ::core::ffi::c_char = p;
    let mut p_end: *mut ::core::ffi::c_char = p.offset(strlen(p) as isize);
    if start > filename.offset(2 as ::core::ffi::c_int as isize) {
        memmove(
            filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
        );
        p_end = p_end.offset(
            -(p.offset_from(filename.offset(1 as ::core::ffi::c_int as isize)) as size_t as isize),
        );
        p = filename.offset(1 as ::core::ffi::c_int as isize);
        start = p;
    }
    loop {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                (p_end.offset_from(p.offset(1 as ::core::ffi::c_int as isize)) as size_t)
                    .wrapping_add(1 as size_t),
            );
            p_end = p_end.offset(-1);
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            if p == start && relative as ::core::ffi::c_int != 0 {
                p = p.offset(
                    (1 as ::core::ffi::c_int
                        + (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL)
                            as ::core::ffi::c_int) as isize,
                );
            } else {
                let mut tail: *mut ::core::ffi::c_char = p.offset(1 as ::core::ffi::c_int as isize);
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    while vim_ispathsep(*tail as ::core::ffi::c_int) {
                        tail = tail.offset(utfc_ptr2len(tail) as isize);
                    }
                } else if p > start {
                    p = p.offset(-1);
                }
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail.offset_from(p) as size_t as isize));
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            let mut tail_0: *mut ::core::ffi::c_char = p.offset(2 as ::core::ffi::c_int as isize);
            while vim_ispathsep(*tail_0 as ::core::ffi::c_int) {
                tail_0 = tail_0.offset(utfc_ptr2len(tail_0) as isize);
            }
            if components > 0 as ::core::ffi::c_int {
                let mut do_strip: bool = false_0 != 0;
                if !stripping_disabled {
                    let mut saved_char: ::core::ffi::c_char =
                        *p.offset(-1 as ::core::ffi::c_int as isize);
                    *p.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                    let mut file_info: FileInfo = FileInfo {
                        stat: uv_stat_t {
                            st_dev: 0,
                            st_mode: 0,
                            st_nlink: 0,
                            st_uid: 0,
                            st_gid: 0,
                            st_rdev: 0,
                            st_ino: 0,
                            st_size: 0,
                            st_blksize: 0,
                            st_blocks: 0,
                            st_flags: 0,
                            st_gen: 0,
                            st_atim: uv_timespec_t {
                                tv_sec: 0,
                                tv_nsec: 0,
                            },
                            st_mtim: uv_timespec_t {
                                tv_sec: 0,
                                tv_nsec: 0,
                            },
                            st_ctim: uv_timespec_t {
                                tv_sec: 0,
                                tv_nsec: 0,
                            },
                            st_birthtim: uv_timespec_t {
                                tv_sec: 0,
                                tv_nsec: 0,
                            },
                        },
                    };
                    if !os_fileinfo_link(filename, &raw mut file_info) {
                        do_strip = true_0 != 0;
                    }
                    *p.offset(-1 as ::core::ffi::c_int as isize) = saved_char;
                    p = p.offset(-1);
                    while p > start && after_pathsep(start, p) == 0 {
                        p = p.offset(
                            -((utf_head_off(start, p.offset(-(1 as ::core::ffi::c_int as isize)))
                                + 1 as ::core::ffi::c_int) as isize),
                        );
                    }
                    if !do_strip {
                        saved_char = *tail_0;
                        *tail_0 = NUL as ::core::ffi::c_char;
                        if os_fileinfo(filename, &raw mut file_info) {
                            do_strip = true_0 != 0;
                        } else {
                            stripping_disabled = true_0 != 0;
                        }
                        *tail_0 = saved_char;
                        if do_strip {
                            let mut new_file_info: FileInfo = FileInfo {
                                stat: uv_stat_t {
                                    st_dev: 0,
                                    st_mode: 0,
                                    st_nlink: 0,
                                    st_uid: 0,
                                    st_gid: 0,
                                    st_rdev: 0,
                                    st_ino: 0,
                                    st_size: 0,
                                    st_blksize: 0,
                                    st_blocks: 0,
                                    st_flags: 0,
                                    st_gen: 0,
                                    st_atim: uv_timespec_t {
                                        tv_sec: 0,
                                        tv_nsec: 0,
                                    },
                                    st_mtim: uv_timespec_t {
                                        tv_sec: 0,
                                        tv_nsec: 0,
                                    },
                                    st_ctim: uv_timespec_t {
                                        tv_sec: 0,
                                        tv_nsec: 0,
                                    },
                                    st_birthtim: uv_timespec_t {
                                        tv_sec: 0,
                                        tv_nsec: 0,
                                    },
                                },
                            };
                            if p == start && relative as ::core::ffi::c_int != 0 {
                                os_fileinfo(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char,
                                    &raw mut new_file_info,
                                );
                            } else {
                                saved_char = *p;
                                *p = NUL as ::core::ffi::c_char;
                                os_fileinfo(filename, &raw mut new_file_info);
                                *p = saved_char;
                            }
                            if !os_fileinfo_id_equal(&raw mut file_info, &raw mut new_file_info) {
                                do_strip = false_0 != 0;
                            }
                        }
                    }
                }
                if !do_strip {
                    p = tail_0;
                    components = 0 as ::core::ffi::c_int;
                } else {
                    if p == start
                        && relative as ::core::ffi::c_int != 0
                        && *tail_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                    {
                        let c2rust_fresh4 = p;
                        p = p.offset(1);
                        *c2rust_fresh4 = '.' as ::core::ffi::c_char;
                        *p = NUL as ::core::ffi::c_char;
                    } else {
                        if p > start
                            && *tail_0.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                        {
                            p = p.offset(-1);
                        }
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            tail_0 as *const ::core::ffi::c_void,
                            (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                        );
                        p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
                    }
                    components -= 1;
                }
            } else if p == start && !relative {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail_0 as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
            } else {
                if p == start.offset(2 as ::core::ffi::c_int as isize)
                    && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                {
                    memmove(
                        p.offset(-(2 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
                    );
                    p_end = p_end.offset(-(2 as ::core::ffi::c_int as isize));
                    tail_0 = tail_0.offset(-(2 as ::core::ffi::c_int as isize));
                }
                p = tail_0;
            }
        } else {
            components += 1;
            p = path_next_component(p) as *mut ::core::ffi::c_char;
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
    }
    return p_end.offset_from(filename) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn path_has_drive_letter(
    mut p: *const ::core::ffi::c_char,
    mut path_len: size_t,
) -> bool {
    return path_len >= 2 as size_t
        && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int)
        && (path_len == 2 as size_t
            || (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int) as ::core::ffi::c_int
                | (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0);
}
#[no_mangle]
pub unsafe extern "C" fn path_is_url(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if strncmp(
        p,
        b":/\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return URL_SLASH as ::core::ffi::c_int;
    } else if strncmp(
        p,
        b":\\\\\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return URL_BACKSLASH as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_with_url(
    mut fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*fname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *fname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *fname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *fname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
    {
        return 0 as ::core::ffi::c_int;
    }
    if path_has_drive_letter(fname, strlen(fname)) {
        return 0 as ::core::ffi::c_int;
    }
    p = fname.offset(1 as ::core::ffi::c_int as isize);
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '+' as ::core::ffi::c_int
        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
        || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return path_is_url(p);
}
#[no_mangle]
pub unsafe extern "C" fn path_with_extension(
    mut path: *const ::core::ffi::c_char,
    mut extension: *const ::core::ffi::c_char,
) -> bool {
    let mut last_dot: *const ::core::ffi::c_char = strrchr(path, '.' as ::core::ffi::c_int);
    if last_dot.is_null() {
        return false_0 != 0;
    }
    return mb_strcmp_ic(
        p_fic != 0,
        last_dot.offset(1 as ::core::ffi::c_int as isize),
        extension,
    ) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isAbsName(mut name: *const ::core::ffi::c_char) -> bool {
    return path_with_url(name) != 0 as ::core::ffi::c_int
        || path_is_absolute(name) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_FullName(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: bool,
) -> ::core::ffi::c_int {
    *buf = NUL as ::core::ffi::c_char;
    if fname.is_null() {
        return FAIL;
    }
    if strlen(fname) > len.wrapping_sub(1 as size_t) {
        xstrlcpy(buf, fname, len);
        return FAIL;
    }
    if path_with_url(fname) != 0 {
        xstrlcpy(buf, fname, len);
        return OK;
    }
    let mut rv: ::core::ffi::c_int = path_to_absolute(fname, buf, len, force as ::core::ffi::c_int);
    if rv == FAIL {
        xstrlcpy(buf, fname, len);
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn fix_fname(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return FullName_save(fname, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn path_fix_case(mut name: *mut ::core::ffi::c_char) {
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    if !os_fileinfo_link(name, &raw mut file_info) {
        return;
    }
    let mut slash: *mut ::core::ffi::c_char = strrchr(name, '/' as ::core::ffi::c_int);
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir: Directory = Directory {
        request: uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        },
        ent: uv_dirent_t {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            type_0: UV_DIRENT_UNKNOWN,
        },
    };
    let mut ok: bool = false;
    if slash.is_null() {
        ok = os_scandir(&raw mut dir, b".\0".as_ptr() as *const ::core::ffi::c_char);
        tail = name;
    } else {
        *slash = NUL as ::core::ffi::c_char;
        ok = os_scandir(&raw mut dir, name);
        *slash = '/' as ::core::ffi::c_char;
        tail = slash.offset(1 as ::core::ffi::c_int as isize);
    }
    if !ok {
        return;
    }
    let mut taillen: size_t = strlen(tail);
    let mut entry: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        entry = os_scandir_next(&raw mut dir);
        if entry.is_null() {
            break;
        }
        if !(strcasecmp(tail, entry as *mut ::core::ffi::c_char) == 0 as ::core::ffi::c_int
            && taillen == strlen(entry))
        {
            continue;
        }
        let mut newname: [::core::ffi::c_char; 4097] = [0; 4097];
        xstrlcpy(
            &raw mut newname as *mut ::core::ffi::c_char,
            name,
            (MAXPATHL + 1 as ::core::ffi::c_int) as size_t,
        );
        xstrlcpy(
            (&raw mut newname as *mut ::core::ffi::c_char).offset(tail.offset_from(name) as isize),
            entry,
            (MAXPATHL as isize - tail.offset_from(name) + 1 as isize) as size_t,
        );
        let mut file_info_new: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if !(os_fileinfo_link(
            &raw mut newname as *mut ::core::ffi::c_char,
            &raw mut file_info_new,
        ) as ::core::ffi::c_int
            != 0
            && os_fileinfo_id_equal(&raw mut file_info, &raw mut file_info_new)
                as ::core::ffi::c_int
                != 0)
        {
            continue;
        }
        strcpy(tail, entry as *mut ::core::ffi::c_char);
        break;
    }
    os_closedir(&raw mut dir);
}
#[no_mangle]
pub unsafe extern "C" fn after_pathsep(
    mut b: *const ::core::ffi::c_char,
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return (p > b
        && vim_ispathsep(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
        && utf_head_off(b, p.offset(-(1 as ::core::ffi::c_int as isize)))
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn same_directory(
    mut f1: *mut ::core::ffi::c_char,
    mut f2: *mut ::core::ffi::c_char,
) -> bool {
    let mut ffname: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut t1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if f1.is_null() || f2.is_null() {
        return false_0 != 0;
    }
    vim_FullName(
        f1,
        &raw mut ffname as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        false_0 != 0,
    );
    t1 = path_tail_with_sep(&raw mut ffname as *mut ::core::ffi::c_char);
    t2 = path_tail_with_sep(f2);
    return t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) == t2.offset_from(f2)
        && pathcmp(
            &raw mut ffname as *mut ::core::ffi::c_char,
            f2,
            t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn pathcmp(
    mut p: *const ::core::ffi::c_char,
    mut q: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    i = 0 as ::core::ffi::c_int;
    j = 0 as ::core::ffi::c_int;
    while maxlen < 0 as ::core::ffi::c_int || i < maxlen && j < maxlen {
        let mut c1: ::core::ffi::c_int = utf_ptr2char(p.offset(i as isize));
        let mut c2: ::core::ffi::c_int = utf_ptr2char(q.offset(j as isize));
        if c1 == NUL {
            if c2 == NUL {
                return 0 as ::core::ffi::c_int;
            }
            s = q;
            i = j;
            break;
        } else if c2 == NUL {
            s = p;
            break;
        } else {
            if if p_fic != 0 {
                (mb_toupper(c1) != mb_toupper(c2)) as ::core::ffi::c_int
            } else {
                (c1 != c2) as ::core::ffi::c_int
            } != 0
            {
                if vim_ispathsep(c1) {
                    return -1 as ::core::ffi::c_int;
                }
                if vim_ispathsep(c2) {
                    return 1 as ::core::ffi::c_int;
                }
                return if p_fic != 0 {
                    mb_toupper(c1) - mb_toupper(c2)
                } else {
                    c1 - c2
                };
            }
            i += utfc_ptr2len(p.offset(i as isize));
            j += utfc_ptr2len(q.offset(j as isize));
        }
    }
    if s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut c1_0: ::core::ffi::c_int = utf_ptr2char(s.offset(i as isize));
    let mut c2_0: ::core::ffi::c_int = utf_ptr2char(
        s.offset(i as isize)
            .offset(utfc_ptr2len(s.offset(i as isize)) as isize),
    );
    if c2_0 == NUL
        && i > 0 as ::core::ffi::c_int
        && after_pathsep(s, s.offset(i as isize)) == 0
        && c1_0 == '/' as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if s == q {
        return -1 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_try_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut dirname: *mut ::core::ffi::c_char =
        xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = full_path;
    if os_dirname(dirname, MAXPATHL as size_t) == OK {
        p = path_shorten_fname(full_path, dirname);
        if p.is_null() || *p as ::core::ffi::c_int == NUL {
            p = full_path;
        }
    }
    xfree(dirname as *mut ::core::ffi::c_void);
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn path_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
    mut dir_name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if full_path.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    '_c2rust_label: {
        if !dir_name.is_null() {
        } else {
            __assert_fail(
                b"dir_name != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2108 as ::core::ffi::c_uint,
                b"char *path_shorten_fname(char *, char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: size_t = strlen(dir_name);
    if path_fnamencmp(dir_name, full_path, len) != 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if len == path_head_length() as size_t && is_path_head(dir_name) as ::core::ffi::c_int != 0 {
        return full_path.offset(len as isize);
    }
    let mut p: *mut ::core::ffi::c_char = full_path.offset(len as isize);
    if !vim_ispathsep(*p as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    loop {
        p = p.offset(1);
        if !vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
            break;
        }
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn expand_wildcards_eval(
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut eval_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut exp_pat: *mut ::core::ffi::c_char = *pat;
    let mut ignored_msg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut usedlen: size_t = 0;
    let is_cur_alt_file: bool = *exp_pat as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *exp_pat as ::core::ffi::c_int == '#' as ::core::ffi::c_int;
    let mut star_follows: bool = false_0 != 0;
    if is_cur_alt_file as ::core::ffi::c_int != 0
        || *exp_pat as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        emsg_off += 1;
        eval_pat = eval_vars(
            exp_pat,
            exp_pat,
            &raw mut usedlen,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut ignored_msg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        );
        emsg_off -= 1;
        if !eval_pat.is_null() {
            star_follows = strcmp(
                exp_pat.offset(usedlen as isize),
                b"*\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int;
            exp_pat = concat_str(eval_pat, exp_pat.offset(usedlen as isize));
        }
    }
    if !exp_pat.is_null() {
        ret = expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut exp_pat,
            num_file,
            file,
            flags,
        );
    }
    if !eval_pat.is_null() {
        if *num_file == 0 as ::core::ffi::c_int
            && is_cur_alt_file as ::core::ffi::c_int != 0
            && star_follows as ::core::ffi::c_int != 0
        {
            *file = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                as *mut *mut ::core::ffi::c_char;
            **file = eval_pat;
            eval_pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
            *num_file = 1 as ::core::ffi::c_int;
            ret = OK;
        }
        xfree(exp_pat as *mut ::core::ffi::c_void);
        xfree(eval_pat as *mut ::core::ffi::c_void);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn expand_wildcards(
    mut num_pat: ::core::ffi::c_int,
    mut pat: *mut *mut ::core::ffi::c_char,
    mut num_files: *mut ::core::ffi::c_int,
    mut files: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int =
        gen_expand_wildcards(num_pat, pat, num_files, files, flags);
    if flags & EW_KEEPALL as ::core::ffi::c_int != 0 || retval == FAIL {
        return retval;
    }
    if *p_wig != 0 {
        '_c2rust_label: {
            if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
            } else {
                __assert_fail(
                    b"*num_files == 0 || *files != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2221 as ::core::ffi::c_uint,
                    b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < *num_files {
            let mut ffname: *mut ::core::ffi::c_char =
                FullName_save(*(*files).offset(i as isize), false_0 != 0);
            '_c2rust_label_0: {
                if !(*(*files).offset(i as isize)).is_null() {
                } else {
                    __assert_fail(
                        b"(*files)[i] != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2224 as ::core::ffi::c_uint,
                        b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            '_c2rust_label_1: {
                if !ffname.is_null() {
                } else {
                    __assert_fail(
                        b"ffname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2225 as ::core::ffi::c_uint,
                        b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if match_file_list(p_wig, *(*files).offset(i as isize), ffname) {
                xfree(*(*files).offset(i as isize) as *mut ::core::ffi::c_void);
                let mut j: ::core::ffi::c_int = i;
                while (j + 1 as ::core::ffi::c_int) < *num_files {
                    *(*files).offset(j as isize) =
                        *(*files).offset((j + 1 as ::core::ffi::c_int) as isize);
                    j += 1;
                }
                *num_files -= 1;
                i -= 1;
            }
            xfree(ffname as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    '_c2rust_label_2: {
        if *num_files == 0 as ::core::ffi::c_int || !(*files).is_null() {
        } else {
            __assert_fail(
                b"*num_files == 0 || *files != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2241 as ::core::ffi::c_uint,
                b"int expand_wildcards(int, char **, int *, char ***, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if *num_files > 1 as ::core::ffi::c_int && !got_int {
        let mut non_suf_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < *num_files {
            if !match_suffix(*(*files).offset(i_0 as isize)) {
                let mut p: *mut ::core::ffi::c_char = *(*files).offset(i_0 as isize);
                let mut j_0: ::core::ffi::c_int = i_0;
                while j_0 > non_suf_match {
                    *(*files).offset(j_0 as isize) =
                        *(*files).offset((j_0 - 1 as ::core::ffi::c_int) as isize);
                    j_0 -= 1;
                }
                let c2rust_fresh8 = non_suf_match;
                non_suf_match = non_suf_match + 1;
                let c2rust_lvalue_ptr = &raw mut *(*files).offset(c2rust_fresh8 as isize);
                *c2rust_lvalue_ptr = p;
            }
            i_0 += 1;
        }
    }
    if *num_files == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void = files as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        return FAIL;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn match_suffix(mut fname: *mut ::core::ffi::c_char) -> bool {
    let mut suf_buf: [::core::ffi::c_char; 30] = [0; 30];
    let mut fnamelen: size_t = strlen(fname);
    let mut setsuflen: size_t = 0 as size_t;
    let mut setsuf: *mut ::core::ffi::c_char = p_su;
    while *setsuf != 0 {
        setsuflen = copy_option_part(
            &raw mut setsuf,
            &raw mut suf_buf as *mut ::core::ffi::c_char,
            MAXSUFLEN as size_t,
            b".,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if setsuflen == 0 as size_t {
            let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
            if !vim_strchr(tail, '.' as ::core::ffi::c_int).is_null() {
                continue;
            }
            setsuflen = 1 as size_t;
            break;
        } else {
            if fnamelen >= setsuflen
                && path_fnamencmp(
                    &raw mut suf_buf as *mut ::core::ffi::c_char,
                    fname
                        .offset(fnamelen as isize)
                        .offset(-(setsuflen as isize)),
                    setsuflen,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            setsuflen = 0 as size_t;
        }
    }
    return setsuflen != 0 as size_t;
}
pub const MAXSUFLEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn path_full_dir_name(
    mut directory: *mut ::core::ffi::c_char,
    mut buffer: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if strlen(directory) == 0 as size_t {
        return os_dirname(buffer, len);
    }
    if !os_realpath(directory, buffer, len).is_null() {
        return OK;
    }
    if path_is_absolute(directory) {
        return FAIL;
    }
    let mut old_dir: [::core::ffi::c_char; 4096] = [0; 4096];
    if os_dirname(
        &raw mut old_dir as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == FAIL
    {
        return FAIL;
    }
    xstrlcpy(buffer, &raw mut old_dir as *mut ::core::ffi::c_char, len);
    if append_path(buffer, directory, len) == FAIL {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn append_path(
    mut path: *mut ::core::ffi::c_char,
    mut to_append: *const ::core::ffi::c_char,
    mut max_len: size_t,
) -> ::core::ffi::c_int {
    let mut current_length: size_t = strlen(path);
    let mut to_append_length: size_t = strlen(to_append);
    if to_append_length == 0 as size_t
        || strcmp(to_append, b".\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    if current_length > 0 as size_t
        && !vim_ispathsep_nocolon(
            *path.offset(current_length.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        )
    {
        if current_length
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            )
            .wrapping_add(1 as size_t)
            > max_len
        {
            return FAIL;
        }
        xstrlcpy(
            path.offset(current_length as isize),
            PATHSEPSTR.as_ptr(),
            max_len.wrapping_sub(current_length),
        );
        current_length = (current_length as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    if current_length
        .wrapping_add(to_append_length)
        .wrapping_add(1 as size_t)
        > max_len
    {
        return FAIL;
    }
    xstrlcpy(
        path.offset(current_length as isize),
        to_append,
        max_len.wrapping_sub(current_length),
    );
    return OK;
}
unsafe extern "C" fn path_to_absolute(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    *buf = NUL as ::core::ffi::c_char;
    let mut relative_directory: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    let mut end_of_path: *const ::core::ffi::c_char = fname;
    if force != 0 || !path_is_absolute(fname) {
        p = strrchr(fname, '/' as ::core::ffi::c_int);
        if p.is_null()
            && strcmp(fname, b"..\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            p = fname.offset(2 as ::core::ffi::c_int as isize);
        }
        if !p.is_null() {
            if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && strcmp(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    b"..\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(3 as ::core::ffi::c_int as isize);
            }
            '_c2rust_label: {
                if p >= fname {
                } else {
                    __assert_fail(
                        b"p >= fname\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2403 as ::core::ffi::c_uint,
                        b"int path_to_absolute(const char *, char *, size_t, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                relative_directory as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                (p.offset_from(fname) + 1 as isize) as size_t,
            );
            *relative_directory.offset((p.offset_from(fname) + 1 as isize) as isize) =
                NUL as ::core::ffi::c_char;
            end_of_path = if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                p.offset(1 as ::core::ffi::c_int as isize)
            } else {
                p
            };
        } else {
            *relative_directory.offset(0 as ::core::ffi::c_int as isize) =
                NUL as ::core::ffi::c_char;
        }
        if FAIL == path_full_dir_name(relative_directory, buf, len) {
            xfree(relative_directory as *mut ::core::ffi::c_void);
            return FAIL;
        }
    }
    xfree(relative_directory as *mut ::core::ffi::c_void);
    return append_path(buf, end_of_path, len);
}
#[no_mangle]
pub unsafe extern "C" fn path_is_absolute(mut fname: *const ::core::ffi::c_char) -> bool {
    return *fname as ::core::ffi::c_int == '/' as ::core::ffi::c_int
        || *fname as ::core::ffi::c_int == '~' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn path_guess_exepath(
    mut argv0: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
) {
    let mut path: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    if path.is_null() || path_is_absolute(argv0) as ::core::ffi::c_int != 0 {
        xstrlcpy(buf, argv0, bufsize);
    } else if *argv0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        || !strchr(argv0, PATHSEP).is_null()
    {
        if os_dirname(buf, MAXPATHL as size_t) != OK {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        xstrlcat(buf, PATHSEPSTR.as_ptr(), bufsize);
        xstrlcat(buf, argv0, bufsize);
    } else {
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ENV_SEPCHAR as ::core::ffi::c_char,
                path,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            if dir_len.wrapping_add(1 as size_t)
                <= ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
            {
                xmemcpyz(
                    &raw mut NameBuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    dir as *const ::core::ffi::c_void,
                    dir_len,
                );
                xstrlcat(
                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                xstrlcat(
                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                    argv0,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                if os_can_exe(
                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                ) {
                    xstrlcpy(buf, &raw mut NameBuff as *mut ::core::ffi::c_char, bufsize);
                    return;
                }
            }
            if iter.is_null() {
                break;
            }
        }
        xstrlcpy(buf, argv0, bufsize);
    }
    xfree(path as *mut ::core::ffi::c_void);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPECIAL_WILDCHAR: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"`'{\0") };
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_NOBREAK: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
