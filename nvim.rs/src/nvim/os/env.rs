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
    fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_err_name(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_os_homedir(
        buffer: *mut ::core::ffi::c_char,
        size: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn uv_os_getenv(
        name: *const ::core::ffi::c_char,
        buffer: *mut ::core::ffi::c_char,
        size: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn uv_os_setenv(
        name: *const ::core::ffi::c_char,
        value: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn uv_os_unsetenv(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    static mut environ: *mut *mut ::core::ffi::c_char;
    fn getpid() -> __pid_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemdupz(
        data: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn xmemrchr(
        src: *const ::core::ffi::c_void,
        c: uint8_t,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn striequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn vim_isfilec(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn skip_expr(
        pp: *mut *mut ::core::ffi::c_char,
        evalarg: *mut evalarg_T,
    ) -> ::core::ffi::c_int;
    fn modify_fname(
        src: *mut ::core::ffi::c_char,
        tilde_file: bool,
        usedlen: *mut size_t,
        fnamep: *mut *mut ::core::ffi::c_char,
        bufp: *mut *mut ::core::ffi::c_char,
        fnamelen: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    static mut didset_vim: bool;
    static mut didset_vimruntime: bool;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut os_buf: [::core::ffi::c_char; 4096];
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_realpath(
        name: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    static mut nvim_testing: bool;
    static mut p_hf: *mut ::core::ffi::c_char;
    fn os_get_userdir(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn path_fnamencmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn concat_fnames(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        sep: bool,
    ) -> *mut ::core::ffi::c_char;
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn append_path(
        path: *mut ::core::ffi::c_char,
        to_append: *const ::core::ffi::c_char,
        max_len: size_t,
    ) -> ::core::ffi::c_int;
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn uname(__name: *mut utsname) -> ::core::ffi::c_int;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type time_t = __time_t;
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed = -8;
pub const UV_EUNATCH: C2Rust_Unnamed = -49;
pub const UV_ENODATA: C2Rust_Unnamed = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed = -94;
pub const UV_EILSEQ: C2Rust_Unnamed = -84;
pub const UV_EFTYPE: C2Rust_Unnamed = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed = -112;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_ENXIO: C2Rust_Unnamed = -6;
pub const UV_EOF: C2Rust_Unnamed = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_EXDEV: C2Rust_Unnamed = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed = -110;
pub const UV_ESRCH: C2Rust_Unnamed = -3;
pub const UV_ESPIPE: C2Rust_Unnamed = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed = -108;
pub const UV_EROFS: C2Rust_Unnamed = -30;
pub const UV_ERANGE: C2Rust_Unnamed = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed = -93;
pub const UV_EPROTO: C2Rust_Unnamed = -71;
pub const UV_EPIPE: C2Rust_Unnamed = -32;
pub const UV_EPERM: C2Rust_Unnamed = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed = -107;
pub const UV_ENOSYS: C2Rust_Unnamed = -38;
pub const UV_ENOSPC: C2Rust_Unnamed = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed = -92;
pub const UV_ENONET: C2Rust_Unnamed = -64;
pub const UV_ENOMEM: C2Rust_Unnamed = -12;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENODEV: C2Rust_Unnamed = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ENFILE: C2Rust_Unnamed = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed = -90;
pub const UV_EMFILE: C2Rust_Unnamed = -24;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EISCONN: C2Rust_Unnamed = -106;
pub const UV_EIO: C2Rust_Unnamed = -5;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EINTR: C2Rust_Unnamed = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed = -113;
pub const UV_EFBIG: C2Rust_Unnamed = -27;
pub const UV_EFAULT: C2Rust_Unnamed = -14;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed = -103;
pub const UV_ECHARSET: C2Rust_Unnamed = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed = -125;
pub const UV_EBUSY: C2Rust_Unnamed = -16;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const UV_EALREADY: C2Rust_Unnamed = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed = -98;
pub const UV_EACCES: C2Rust_Unnamed = -13;
pub const UV_E2BIG: C2Rust_Unnamed = -7;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const EXPAND_BUF_LEN: C2Rust_Unnamed_13 = 256;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_15 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_15 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_15 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_15 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_15 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_15 = 7;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub const WILD_PREV: C2Rust_Unnamed_15 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_15 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_15 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_15 = 2;
pub const WILD_FREE: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_16 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_16 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_16 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_16 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_16 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_16 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_16 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_16 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_16 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_16 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_16 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_16 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_16 = 1;
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
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
pub struct utsname {
    pub sysname: [::core::ffi::c_char; 65],
    pub nodename: [::core::ffi::c_char; 65],
    pub release: [::core::ffi::c_char; 65],
    pub version: [::core::ffi::c_char; 65],
    pub machine: [::core::ffi::c_char; 65],
    pub domainname: [::core::ffi::c_char; 65],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const RUNTIME_DIRNAME: [::core::ffi::c_char; 8] = unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"runtime\0")
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int
    + 1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut default_vim_dir: *mut ::core::ffi::c_char = b"/usr/local/share/nvim\0"
    .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut default_vimruntime_dir: *mut ::core::ffi::c_char = b"\0".as_ptr()
    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut default_lib_dir: *mut ::core::ffi::c_char = b"/usr/local/lib64/nvim\0"
    .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub unsafe extern "C" fn env_init() {
    nvim_testing = os_env_exists(
        b"NVIM_TEST\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn os_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: size_t = INIT_SIZE as size_t;
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    r = uv_os_getenv(name, &raw mut buf as *mut ::core::ffi::c_char, &raw mut size);
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        e = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r != 0 as ::core::ffi::c_int || size == 0 as size_t
            || *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut e
                as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        }
    } else if r != 0 as ::core::ffi::c_int || size == 0 as size_t
        || buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
    {
        e = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        e = xmemdupz(
            &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            size,
        ) as *mut ::core::ffi::c_char;
    }
    if r != 0 as ::core::ffi::c_int && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_UNKNOWN as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_getenv\0".as_ptr() as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return e;
}
pub const INIT_SIZE: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn os_getenv_buf(
    name: *const ::core::ffi::c_char,
    buf: *mut ::core::ffi::c_char,
    bufsize: size_t,
) -> *mut ::core::ffi::c_char {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut size: size_t = bufsize;
    let mut r: ::core::ffi::c_int = uv_os_getenv(name, buf, &raw mut size);
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        let mut e: *mut ::core::ffi::c_char = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r == 0 as ::core::ffi::c_int && size != 0 as size_t
            && *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            xmemcpyz(
                buf as *mut ::core::ffi::c_void,
                e as *const ::core::ffi::c_void,
                (if bufsize < size { bufsize } else { size }).wrapping_sub(1 as size_t),
            );
        }
        xfree(e as *mut ::core::ffi::c_void);
    }
    if r != 0 as ::core::ffi::c_int || size == 0 as size_t
        || *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if r != 0 as ::core::ffi::c_int && r != UV_ENOENT as ::core::ffi::c_int
            && r != UV_UNKNOWN as ::core::ffi::c_int
        {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"os_getenv_buf\0".as_ptr() as *const ::core::ffi::c_char,
                129 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_os_getenv(%s) failed: %d %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
                r,
                uv_err_name(r),
            );
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn os_getenv_noalloc(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return os_getenv_buf(
        name,
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn os_env_exists(
    mut name: *const ::core::ffi::c_char,
    mut nonempty: bool,
) -> bool {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    let mut size: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
    let mut r: ::core::ffi::c_int = uv_os_getenv(
        name,
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut size,
    );
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {} else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                165 as ::core::ffi::c_uint,
                b"_Bool os_env_exists(const char *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_ENOBUFS as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_env_exists\0".as_ptr() as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return r == 0 as ::core::ffi::c_int && (!nonempty || size > 0 as size_t)
        || r == UV_ENOBUFS as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_setenv(
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut overwrite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if overwrite == 0 && os_env_exists(name, false_0 != 0) as ::core::ffi::c_int != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0;
    r = uv_os_setenv(name, value);
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {} else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                204 as ::core::ffi::c_uint,
                b"int os_setenv(const char *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_setenv\0".as_ptr() as *const ::core::ffi::c_char,
            206 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_setenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn os_unsetenv(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = uv_os_unsetenv(name);
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_unsetenv\0".as_ptr() as *const ::core::ffi::c_char,
            220 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_unsetenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn os_get_fullenv_size() -> size_t {
    let mut len: size_t = 0 as size_t;
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    while !(*environ.offset(len as isize)).is_null() {
        len = len.wrapping_add(1);
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn os_free_fullenv(mut env: *mut *mut ::core::ffi::c_char) {
    if env.is_null() {
        return;
    }
    let mut it: *mut *mut ::core::ffi::c_char = env;
    while !(*it).is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void = it
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        it = it.offset(1);
    }
    xfree(env as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn os_copy_fullenv(
    mut env: *mut *mut ::core::ffi::c_char,
    mut env_size: size_t,
) {
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i < env_size && !(*environ.offset(i as isize)).is_null() {
        *env.offset(i as isize) = xstrdup(*environ.offset(i as isize));
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_getenvname_at_index(
    mut index: size_t,
) -> *mut ::core::ffi::c_char {
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i <= index {
        if (*environ.offset(i as isize)).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        i = i.wrapping_add(1);
    }
    let mut str: *mut ::core::ffi::c_char = *environ.offset(index as isize);
    '_c2rust_label: {
        if !str.is_null() {} else {
            __assert_fail(
                b"str != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                375 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let end: *const ::core::ffi::c_char = strchr(str, '=' as ::core::ffi::c_int);
    '_c2rust_label_0: {
        if !end.is_null() {} else {
            __assert_fail(
                b"end != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                377 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: ptrdiff_t = end.offset_from(str);
    '_c2rust_label_1: {
        if len > 0 as ptrdiff_t {} else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                379 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return xmemdupz(str as *const ::core::ffi::c_void, len as size_t)
        as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn os_get_pid() -> int64_t {
    return getpid() as int64_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_hint_priority() {}
#[no_mangle]
pub unsafe extern "C" fn os_get_hostname(
    mut hostname: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    let mut vutsname: utsname = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    if uname(&raw mut vutsname) < 0 as ::core::ffi::c_int {
        *hostname = NUL as ::core::ffi::c_char;
    } else {
        xstrlcpy(hostname, &raw mut vutsname.nodename as *mut ::core::ffi::c_char, size);
    };
}
static mut homedir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
    ::core::ffi::c_char,
>();
#[no_mangle]
pub unsafe extern "C" fn os_homedir() -> *const ::core::ffi::c_char {
    if homedir.is_null() {
        emsg(
            b"os_homedir failed: homedir not initialized\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return homedir;
}
#[no_mangle]
pub unsafe extern "C" fn init_homedir() {
    xfree(homedir as *mut ::core::ffi::c_void);
    homedir = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut var: *mut ::core::ffi::c_char = os_getenv(
        b"HOME\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut tofree: *mut ::core::ffi::c_char = var;
    if var.is_null() {
        var = os_uv_homedir();
    }
    if !var.is_null()
        && !os_realpath(
                var,
                &raw mut IObuff as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
            )
            .is_null()
    {
        var = &raw mut IObuff as *mut ::core::ffi::c_char;
    }
    if (var.is_null() || *var as ::core::ffi::c_int == NUL)
        && os_dirname(
            &raw mut os_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) == OK
    {
        var = &raw mut os_buf as *mut ::core::ffi::c_char;
    }
    if !var.is_null() {
        homedir = xstrdup(var);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
}
static mut homedir_buf: [::core::ffi::c_char; 4096] = [0; 4096];
unsafe extern "C" fn os_uv_homedir() -> *mut ::core::ffi::c_char {
    homedir_buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut homedir_size: size_t = MAXPATHL as size_t;
    let mut ret_value: ::core::ffi::c_int = uv_os_homedir(
        &raw mut homedir_buf as *mut ::core::ffi::c_char,
        &raw mut homedir_size,
    );
    if ret_value == 0 as ::core::ffi::c_int && homedir_size < MAXPATHL as size_t {
        return &raw mut homedir_buf as *mut ::core::ffi::c_char;
    }
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_uv_homedir\0".as_ptr() as *const ::core::ffi::c_char,
        570 as ::core::ffi::c_int,
        true_0 != 0,
        b"uv_os_homedir() failed %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
        ret_value,
        uv_strerror(ret_value),
    );
    homedir_buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_env_save(
    mut src: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return expand_env_save_opt(src, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn expand_env_save_opt(
    mut src: *mut ::core::ffi::c_char,
    mut one: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t)
        as *mut ::core::ffi::c_char;
    expand_env_esc(
        src,
        p,
        MAXPATHL,
        false_0 != 0,
        one,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn expand_env(
    mut src: *mut ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
) -> size_t {
    return expand_env_esc(
        src,
        dst,
        dstlen,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_env_esc(
    mut srcp: *const ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
    mut esc: bool,
    mut one: bool,
    mut prefix: *mut ::core::ffi::c_char,
) -> size_t {
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut var: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut copy_char: bool = false;
    let mut mustfree: bool = false;
    let mut at_start: bool = true_0 != 0;
    let dst_start: *mut ::core::ffi::c_char = dst;
    let mut prefix_len: ::core::ffi::c_int = if prefix.is_null() {
        0 as ::core::ffi::c_int
    } else {
        strlen(prefix) as ::core::ffi::c_int
    };
    let mut src: *mut ::core::ffi::c_char = skipwhite(srcp);
    dstlen -= 1;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as ::core::ffi::c_int {
        if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
            && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            var = src;
            src = src.offset(2 as ::core::ffi::c_int as isize);
            skip_expr(&raw mut src, ::core::ptr::null_mut::<evalarg_T>());
            if *src as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                src = src.offset(1);
            }
            let mut len: size_t = src.offset_from(var) as size_t;
            if len > dstlen as size_t {
                len = dstlen as size_t;
            }
            memcpy(
                dst as *mut ::core::ffi::c_void,
                var as *const ::core::ffi::c_void,
                len,
            );
            dst = dst.offset(len as isize);
            dstlen -= len as ::core::ffi::c_int;
        } else {
            copy_char = true_0 != 0;
            if *src as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                    && at_start as ::core::ffi::c_int != 0
            {
                mustfree = false_0 != 0;
                if *src as ::core::ffi::c_int != '~' as ::core::ffi::c_int {
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                    var = dst;
                    let mut c: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    if *tail as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        && !vim_isIDc('{' as ::core::ffi::c_int)
                    {
                        tail = tail.offset(1);
                        loop {
                            let c2rust_fresh0 = c;
                            c = c - 1;
                            if !(c2rust_fresh0 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int)
                            {
                                break;
                            }
                            let c2rust_fresh1 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh2 = var;
                            var = var.offset(1);
                            *c2rust_fresh2 = *c2rust_fresh1;
                        }
                    } else {
                        loop {
                            let c2rust_fresh3 = c;
                            c = c - 1;
                            if !(c2rust_fresh3 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && vim_isIDc(*tail as uint8_t as ::core::ffi::c_int)
                                    as ::core::ffi::c_int != 0)
                            {
                                break;
                            }
                            let c2rust_fresh4 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh5 = var;
                            var = var.offset(1);
                            *c2rust_fresh5 = *c2rust_fresh4;
                        }
                    }
                    if *src.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                    {
                        var = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        if *src.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        {
                            tail = tail.offset(1);
                        }
                        *var = NUL as ::core::ffi::c_char;
                        var = vim_getenv(dst);
                        mustfree = true_0 != 0;
                    }
                } else if *src.offset(1 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *src.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int != 0
                    || !vim_strchr(
                            b" ,\t\n\0".as_ptr() as *const ::core::ffi::c_char,
                            *src.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int,
                        )
                        .is_null()
                {
                    var = homedir;
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tail = src;
                    var = dst;
                    let mut c_0: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    loop {
                        let c2rust_fresh6 = c_0;
                        c_0 = c_0 - 1;
                        if !(c2rust_fresh6 > 0 as ::core::ffi::c_int
                            && *tail as ::core::ffi::c_int != 0
                            && vim_isfilec(*tail as uint8_t as ::core::ffi::c_int)
                                as ::core::ffi::c_int != 0
                            && !vim_ispathsep(*tail as ::core::ffi::c_int))
                        {
                            break;
                        }
                        let c2rust_fresh7 = tail;
                        tail = tail.offset(1);
                        let c2rust_fresh8 = var;
                        var = var.offset(1);
                        *c2rust_fresh8 = *c2rust_fresh7;
                    }
                    *var = NUL as ::core::ffi::c_char;
                    var = if *dst as ::core::ffi::c_int == NUL {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    } else {
                        os_get_userdir(dst.offset(1 as ::core::ffi::c_int as isize))
                    };
                    mustfree = true_0 != 0;
                    if var.is_null() {
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
                            xp_files: ::core::ptr::null_mut::<
                                *mut ::core::ffi::c_char,
                            >(),
                            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_buf: [0; 256],
                            xp_search_dir: kDirectionNotSet,
                            xp_pre_incsearch_pos: pos_T {
                                lnum: 0,
                                col: 0,
                                coladd: 0,
                            },
                        };
                        ExpandInit(&raw mut xpc);
                        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
                        var = ExpandOne(
                            &raw mut xpc,
                            dst,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            WILD_ADD_SLASH as ::core::ffi::c_int
                                | WILD_SILENT as ::core::ffi::c_int,
                            WILD_EXPAND_FREE as ::core::ffi::c_int,
                        );
                        mustfree = true_0 != 0;
                    }
                }
                if esc as ::core::ffi::c_int != 0 && !var.is_null()
                    && !strpbrk(var, b" \t\0".as_ptr() as *const ::core::ffi::c_char)
                        .is_null()
                {
                    let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped(
                        var,
                        b" \t\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    if mustfree {
                        xfree(var as *mut ::core::ffi::c_void);
                    }
                    var = p;
                    mustfree = true_0 != 0;
                }
                if !var.is_null() && *var as ::core::ffi::c_int != NUL {
                    let mut c_1: ::core::ffi::c_int = strlen(var) as ::core::ffi::c_int;
                    if (c_1 as size_t)
                        .wrapping_add(strlen(tail))
                        .wrapping_add(1 as size_t)
                        < dstlen as ::core::ffi::c_uint as size_t
                    {
                        strcpy(dst, var);
                        dstlen -= c_1;
                        if after_pathsep(dst, dst.offset(c_1 as isize)) != 0
                            && vim_ispathsep(*tail as ::core::ffi::c_int)
                                as ::core::ffi::c_int != 0
                        {
                            tail = tail.offset(1);
                        }
                        dst = dst.offset(c_1 as isize);
                        src = tail;
                        copy_char = false_0 != 0;
                    }
                }
                if mustfree {
                    xfree(var as *mut ::core::ffi::c_void);
                }
            }
            if copy_char {
                at_start = false_0 != 0;
                if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *src.offset(1 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int != NUL
                {
                    let c2rust_fresh9 = src;
                    src = src.offset(1);
                    let c2rust_fresh10 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh10 = *c2rust_fresh9;
                    dstlen -= 1;
                } else if (*src.offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                    || *src.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int == ',' as ::core::ffi::c_int) && !one
                {
                    at_start = true_0 != 0;
                }
                if dstlen > 0 as ::core::ffi::c_int {
                    let c2rust_fresh11 = src;
                    src = src.offset(1);
                    let c2rust_fresh12 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh12 = *c2rust_fresh11;
                    dstlen -= 1;
                    if !prefix.is_null()
                        && src.offset(-(prefix_len as isize))
                            >= srcp as *mut ::core::ffi::c_char
                        && strncmp(
                            src.offset(-(prefix_len as isize)),
                            prefix,
                            prefix_len as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        at_start = true_0 != 0;
                    }
                }
            }
        }
    }
    *dst = NUL as ::core::ffi::c_char;
    return dst.offset_from(dst_start) as size_t;
}
unsafe extern "C" fn vim_runtime_dir(
    mut vimdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if vimdir.is_null() || *vimdir as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut ::core::ffi::c_char = concat_fnames(
        vimdir,
        RUNTIME_DIRNAME.as_ptr(),
        true_0 != 0,
    );
    if os_isdir(p) {
        return p;
    }
    xfree(p as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn remove_tail(
    mut path: *mut ::core::ffi::c_char,
    mut pend: *mut ::core::ffi::c_char,
    mut dirname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(dirname);
    let mut new_tail: *mut ::core::ffi::c_char = pend
        .offset(-(len as isize))
        .offset(-(1 as ::core::ffi::c_int as isize));
    if new_tail >= path
        && path_fnamencmp(new_tail, dirname, len) == 0 as ::core::ffi::c_int
        && (new_tail == path || after_pathsep(path, new_tail) != 0)
    {
        return new_tail;
    }
    return pend;
}
#[no_mangle]
pub unsafe extern "C" fn vim_env_iter(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varval: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varval.is_null() {
        varval = val;
    }
    *dir = varval;
    let dirend: *const ::core::ffi::c_char = strchr(varval, delim as ::core::ffi::c_int);
    if dirend.is_null() {
        *len = strlen(varval);
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *len = dirend.offset_from(varval) as size_t;
    return dirend.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn vim_env_iter_rev(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varend: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varend.is_null() {
        varend = val
            .offset(strlen(val) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
    }
    let varlen: size_t = (varend.offset_from(val) as size_t).wrapping_add(1 as size_t);
    let colon: *const ::core::ffi::c_char = xmemrchr(
        val as *const ::core::ffi::c_void,
        delim as uint8_t,
        varlen,
    ) as *const ::core::ffi::c_char;
    if colon.is_null() {
        *len = varlen;
        *dir = val;
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *dir = colon.offset(1 as ::core::ffi::c_int as isize);
    *len = varend.offset_from(colon) as size_t;
    return colon.offset(-(1 as ::core::ffi::c_int as isize))
        as *const ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn vim_get_prefix_from_exepath(
    mut exe_name: *mut ::core::ffi::c_char,
) {
    xstrlcpy(
        exe_name,
        get_vim_var_str(VV_PROGPATH),
        (MAXPATHL as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
    );
    let mut path_end: *mut ::core::ffi::c_char = path_tail_with_sep(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
    path_end = path_tail(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn vim_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if *get_vim_var_str(VV_PROGPATH).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
        {} else {
            __assert_fail(
                b"get_vim_var_str(VV_PROGPATH)[0] != NUL\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                956 as ::core::ffi::c_uint,
                b"char *vim_getenv(const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut kos_env_path: *mut ::core::ffi::c_char = os_getenv(name);
    if !kos_env_path.is_null() {
        return kos_env_path;
    }
    let mut vimruntime: bool = strcmp(
        name,
        b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int;
    if !vimruntime
        && strcmp(name, b"VIM\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut vim_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if vimruntime as ::core::ffi::c_int != 0
        && *default_vimruntime_dir as ::core::ffi::c_int == NUL
    {
        kos_env_path = os_getenv(b"VIM\0".as_ptr() as *const ::core::ffi::c_char);
        if !kos_env_path.is_null() {
            vim_path = vim_runtime_dir(kos_env_path);
            if vim_path.is_null() {
                vim_path = kos_env_path;
            } else {
                xfree(kos_env_path as *mut ::core::ffi::c_void);
            }
        }
    }
    if vim_path.is_null() {
        if !p_hf.is_null() && vim_strchr(p_hf, '$' as ::core::ffi::c_int).is_null() {
            vim_path = p_hf;
        }
        let mut exe_name: [::core::ffi::c_char; 4096] = [0; 4096];
        if vim_path.is_null() {
            vim_get_prefix_from_exepath(&raw mut exe_name as *mut ::core::ffi::c_char);
            if append_path(
                &raw mut exe_name as *mut ::core::ffi::c_char,
                b"share/nvim/runtime/\0".as_ptr() as *const ::core::ffi::c_char,
                MAXPATHL as size_t,
            ) == OK
            {
                vim_path = &raw mut exe_name as *mut ::core::ffi::c_char;
            }
        }
        if !vim_path.is_null() {
            let mut vim_path_end: *mut ::core::ffi::c_char = path_tail(vim_path);
            if vim_path == p_hf {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    b"doc\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
            }
            if !vimruntime {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    RUNTIME_DIRNAME.as_ptr() as *mut ::core::ffi::c_char,
                );
            }
            if vim_path_end > vim_path && after_pathsep(vim_path, vim_path_end) != 0 {
                vim_path_end = vim_path_end.offset(-1);
            }
            '_c2rust_label_0: {
                if vim_path_end >= vim_path {} else {
                    __assert_fail(
                        b"vim_path_end >= vim_path\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1027 as ::core::ffi::c_uint,
                        b"char *vim_getenv(const char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            vim_path = xmemdupz(
                vim_path as *const ::core::ffi::c_void,
                vim_path_end.offset_from(vim_path) as size_t,
            ) as *mut ::core::ffi::c_char;
            if !os_isdir(vim_path) {
                xfree(vim_path as *mut ::core::ffi::c_void);
                vim_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        '_c2rust_label_1: {
            if vim_path != &raw mut exe_name as *mut ::core::ffi::c_char {} else {
                __assert_fail(
                    b"vim_path != exe_name\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1035 as ::core::ffi::c_uint,
                    b"char *vim_getenv(const char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    }
    if vim_path.is_null() {
        if vimruntime as ::core::ffi::c_int != 0
            && *default_vimruntime_dir as ::core::ffi::c_int != NUL
        {
            vim_path = xstrdup(default_vimruntime_dir);
        } else if *default_vim_dir as ::core::ffi::c_int != NUL {
            if vimruntime as ::core::ffi::c_int != 0
                && {
                    vim_path = vim_runtime_dir(default_vim_dir);
                    vim_path.is_null()
                }
            {
                vim_path = xstrdup(default_vim_dir);
            }
        }
    }
    if !vim_path.is_null() {
        if vimruntime {
            os_setenv(
                b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vimruntime = true_0 != 0;
        } else {
            os_setenv(
                b"VIM\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vim = true_0 != 0;
        }
    }
    return vim_path;
}
#[no_mangle]
pub unsafe extern "C" fn home_replace(
    buf: *const buf_T,
    mut src: *const ::core::ffi::c_char,
    dst: *mut ::core::ffi::c_char,
    mut dstlen: size_t,
    one: bool,
) -> size_t {
    let mut dirlen: size_t = 0 as size_t;
    let mut envlen: size_t = 0 as size_t;
    if src.is_null() {
        *dst = NUL as ::core::ffi::c_char;
        return 0 as size_t;
    }
    if !buf.is_null() && (*buf).b_help as ::core::ffi::c_int != 0 {
        let dlen: size_t = xstrlcpy(dst, path_tail(src), dstlen);
        return if dlen < dstlen.wrapping_sub(1 as size_t) {
            dlen
        } else {
            dstlen.wrapping_sub(1 as size_t)
        };
    }
    if !homedir.is_null() {
        dirlen = strlen(homedir);
    }
    let mut homedir_env: *mut ::core::ffi::c_char = os_getenv(
        b"HOME\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut homedir_env_mod: *mut ::core::ffi::c_char = homedir_env;
    let mut must_free: bool = false_0 != 0;
    if !homedir_env_mod.is_null()
        && *homedir_env_mod as ::core::ffi::c_int == '~' as ::core::ffi::c_int
    {
        must_free = true_0 != 0;
        let mut usedlen: size_t = 0 as size_t;
        let mut flen: size_t = strlen(homedir_env_mod);
        let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
            ::core::ffi::c_char,
        >();
        modify_fname(
            b":p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            &raw mut usedlen,
            &raw mut homedir_env_mod,
            &raw mut fbuf,
            &raw mut flen,
        );
        flen = strlen(homedir_env_mod);
        '_c2rust_label: {
            if homedir_env_mod != homedir_env {} else {
                __assert_fail(
                    b"homedir_env_mod != homedir_env\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1123 as ::core::ffi::c_uint,
                    b"size_t home_replace(const buf_T *const, const char *, char *const, size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if vim_ispathsep(
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int,
        ) {
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize) = NUL
                as ::core::ffi::c_char;
        }
    }
    if !homedir_env_mod.is_null() {
        envlen = strlen(homedir_env_mod);
    }
    if !one {
        src = skipwhite(src);
    }
    let mut dst_p: *mut ::core::ffi::c_char = dst;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as size_t {
        let mut p: *mut ::core::ffi::c_char = homedir;
        let mut len: size_t = dirlen;
        loop {
            if len != 0 && path_fnamencmp(src, p, len) == 0 as ::core::ffi::c_int
                && (vim_ispathsep(*src.offset(len as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int != 0
                    || !one
                        && (*src.offset(len as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int
                            || *src.offset(len as isize) as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int)
                    || *src.offset(len as isize) as ::core::ffi::c_int == NUL)
            {
                src = src.offset(len as isize);
                dstlen = dstlen.wrapping_sub(1);
                if dstlen > 0 as size_t {
                    let c2rust_fresh13 = dst_p;
                    dst_p = dst_p.offset(1);
                    *c2rust_fresh13 = '~' as ::core::ffi::c_char;
                }
                break;
            } else {
                if p == homedir_env_mod {
                    break;
                }
                p = homedir_env_mod;
                len = envlen;
            }
        }
        if dstlen == 0 as size_t {
            break;
        } else {
            while *src as ::core::ffi::c_int != 0
                && (one as ::core::ffi::c_int != 0
                    || *src as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        && *src as ::core::ffi::c_int != ' ' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh14 = src;
                src = src.offset(1);
                let c2rust_fresh15 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh15 = *c2rust_fresh14;
            }
            if dstlen == 0 as size_t {
                break;
            }
            while (*src as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh16 = src;
                src = src.offset(1);
                let c2rust_fresh17 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh17 = *c2rust_fresh16;
            }
        }
    }
    *dst_p = NUL as ::core::ffi::c_char;
    xfree(homedir_env as *mut ::core::ffi::c_void);
    if must_free {
        xfree(homedir_env_mod as *mut ::core::ffi::c_void);
    }
    return dst_p.offset_from(dst) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn home_replace_save(
    mut buf: *mut buf_T,
    mut src: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 3 as size_t;
    if !src.is_null() {
        len = len.wrapping_add(strlen(src));
    }
    let mut dst: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    home_replace(buf, src, dst, len, true_0 != 0);
    return dst;
}
#[no_mangle]
pub unsafe extern "C" fn get_env_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if idx >= 0 as ::core::ffi::c_int {} else {
            __assert_fail(
                b"idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1216 as ::core::ffi::c_uint,
                b"char *get_env_name(expand_T *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut envname: *mut ::core::ffi::c_char = os_getenvname_at_index(idx as size_t);
    if !envname.is_null() {
        xstrlcpy(
            &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
            envname,
            EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
        );
        xfree(envname as *mut ::core::ffi::c_void);
        return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn os_setenv_append_path(
    mut fname: *const ::core::ffi::c_char,
) -> bool {
    if !path_is_absolute(fname) {
        internal_error(
            b"os_setenv_append_path()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    let mut tail: *const ::core::ffi::c_char = path_tail_with_sep(
        fname as *mut ::core::ffi::c_char,
    );
    let mut dirlen: size_t = tail.offset_from(fname) as size_t;
    '_c2rust_label: {
        if tail >= fname
            && dirlen.wrapping_add(1 as size_t)
                < ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
        {} else {
            __assert_fail(
                b"tail >= fname && dirlen + 1 < sizeof(os_buf)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/env.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1247 as ::core::ffi::c_uint,
                b"_Bool os_setenv_append_path(const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xmemcpyz(
        &raw mut os_buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        fname as *const ::core::ffi::c_void,
        dirlen,
    );
    let mut path: *mut ::core::ffi::c_char = os_getenv(
        b"PATH\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let pathlen: size_t = if !path.is_null() { strlen(path) } else { 0 as size_t };
    let newlen: size_t = pathlen.wrapping_add(dirlen).wrapping_add(2 as size_t);
    let mut retval: bool = false_0 != 0;
    if newlen < MAX_ENVPATHLEN as size_t {
        let mut temp: *mut ::core::ffi::c_char = xmalloc(newlen)
            as *mut ::core::ffi::c_char;
        if pathlen == 0 as size_t {
            *temp.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        } else {
            xstrlcpy(temp, path, newlen);
            if ENV_SEPCHAR
                != *path.offset(pathlen.wrapping_sub(1 as size_t) as isize)
                    as ::core::ffi::c_int
            {
                xstrlcat(temp, ENV_SEPSTR.as_ptr(), newlen);
            }
        }
        xstrlcat(temp, &raw mut os_buf as *mut ::core::ffi::c_char, newlen);
        os_setenv(
            b"PATH\0".as_ptr() as *const ::core::ffi::c_char,
            temp,
            1 as ::core::ffi::c_int,
        );
        xfree(temp as *mut ::core::ffi::c_void);
        retval = true_0 != 0;
    }
    xfree(path as *mut ::core::ffi::c_void);
    return retval;
}
pub const MAX_ENVPATHLEN: ::core::ffi::c_int = INT_MAX;
#[no_mangle]
pub unsafe extern "C" fn os_shell_is_cmdexe(mut sh: *const ::core::ffi::c_char) -> bool {
    if *sh as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if striequal(sh, b"$COMSPEC\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut comspec: *mut ::core::ffi::c_char = os_getenv_noalloc(
            b"COMSPEC\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return striequal(
            b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char,
            path_tail(comspec),
        );
    }
    if striequal(sh, b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char)
        as ::core::ffi::c_int != 0
        || striequal(sh, b"cmd\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int != 0
    {
        return true_0 != 0;
    }
    return striequal(b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char, path_tail(sh));
}
#[no_mangle]
pub unsafe extern "C" fn vim_unsetenv_ext(mut var: *const ::core::ffi::c_char) {
    os_unsetenv(var);
    if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vim = false_0 != 0;
    } else if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn vim_setenv_ext(
    mut name: *const ::core::ffi::c_char,
    mut val: *const ::core::ffi::c_char,
) {
    os_setenv(name, val, 1 as ::core::ffi::c_int);
    if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"HOME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        init_homedir();
    } else if didset_vim as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vim = false_0 != 0;
    } else if didset_vimruntime as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime = false_0 != 0;
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const ENV_SEPSTR: [::core::ffi::c_char; 2] = unsafe {
    ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b":\0")
};
