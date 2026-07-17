extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn lseek(__fd: ::core::ffi::c_int, __offset: __off_t, __whence: ::core::ffi::c_int) -> __off_t;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mh_get_int64_t(set: *mut Set_int64_t, key: int64_t) -> uint32_t;
    fn map_del_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut int64_t,
    ) -> int64_t;
    fn map_put_ref_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
        new_item: *mut bool,
    ) -> *mut int64_t;
    fn map_ref_int64_t_int64_t(
        map: *mut Map_int64_t_int64_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
    ) -> *mut int64_t;
    fn map_del_int64_t_ptr_t(
        map: *mut Map_int64_t_ptr_t,
        key: int64_t,
        key_alloc: *mut int64_t,
    ) -> ptr_t;
    fn map_put_ref_int64_t_ptr_t(
        map: *mut Map_int64_t_ptr_t,
        key: int64_t,
        key_alloc: *mut *mut int64_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_swapclose: [::core::ffi::c_char; 0];
    fn read_eintr(
        fd: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_void,
        bufsize: size_t,
    ) -> ssize_t;
    fn write_eintr(
        fd: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_void,
        bufsize: size_t,
    ) -> ssize_t;
    static mut firstbuf: *mut buf_T;
    static mut got_int: bool;
    static mut did_swapwrite_msg: bool;
    static mut main_loop: Loop;
    fn ml_open_file(buf: *mut buf_T);
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn os_open(
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_fsync(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_fileinfo_link(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_fd(file_descriptor: ::core::ffi::c_int, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_blocksize(file_info: *const FileInfo) -> uint64_t;
    fn os_char_avail() -> bool;
    fn os_breakcheck();
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
}
pub type __off_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type size_t = usize;
pub type off_t = __off_t;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type ssize_t = isize;
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
pub type off_T = off_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_20,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
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
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_19,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_18,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
pub type Proc = proc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut dict_T,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: *mut MultiQueue,
}
pub type MultiQueue = multiqueue;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_22,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub events: *mut MultiQueue,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
pub type uv_file = ::core::ffi::c_int;
pub type uv_stream_t = uv_stream_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_21,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
pub type uv_connection_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> ()>;
pub type uv_shutdown_t = uv_shutdown_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_shutdown_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub handle: *mut uv_stream_t,
    pub cb: uv_shutdown_cb,
}
pub type uv_shutdown_cb =
    Option<unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> ()>;
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
pub type uv_connect_t = uv_connect_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_connect_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_connect_cb,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
}
pub type uv_connect_cb = Option<unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()>;
pub type uv_read_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()>;
pub type uv_alloc_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_23,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_24,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_25,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
}
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MFS_ZERO: C2Rust_Unnamed_26 = 8;
pub const MFS_FLUSH: C2Rust_Unnamed_26 = 4;
pub const MFS_STOP: C2Rust_Unnamed_26 = 2;
pub const MFS_ALL: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAX_SWAP_PAGE_SIZE: C2Rust_Unnamed_27 = 50000;
pub const MIN_SWAP_PAGE_SIZE: C2Rust_Unnamed_27 = 1048;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const S_IRUSR: ::core::ffi::c_int = __S_IREAD;
pub const S_IWUSR: ::core::ffi::c_int = __S_IWRITE;
pub const S_IREAD: ::core::ffi::c_int = S_IRUSR;
pub const S_IWRITE: ::core::ffi::c_int = S_IWUSR;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut value_init_ptr_t: ptr_t = NULL;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_int64_t = Set_int64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<int64_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_int64_t_int64_t(
    mut map: *mut Map_int64_t_int64_t,
    mut key: int64_t,
    mut value: int64_t,
) {
    let mut val: *mut int64_t = map_put_ref_int64_t_int64_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut int64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_int64_t_ptr_t(
    mut map: *mut Map_int64_t_ptr_t,
    mut key: int64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut int64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const BH_DIRTY: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
pub const BH_LOCKED: ::core::ffi::c_uint = 2 as ::core::ffi::c_uint;
pub const MEMFILE_PAGE_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
static mut e_block_was_not_locked: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E293: Block was not locked\0")
};
#[no_mangle]
pub unsafe extern "C" fn mf_open(
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut memfile_T {
    let mut mfp: *mut memfile_T = xmalloc(::core::mem::size_of::<memfile_T>()) as *mut memfile_T;
    if fname.is_null() {
        (*mfp).mf_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*mfp).mf_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*mfp).mf_fd = -1 as ::core::ffi::c_int;
    } else if !mf_do_open(mfp, fname, flags) {
        xfree(mfp as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<memfile_T>();
    }
    (*mfp).mf_free_first = ::core::ptr::null_mut::<bhdr_T>();
    (*mfp).mf_dirty = MF_DIRTY_NO;
    (*mfp).mf_hash = Map_int64_t_ptr_t {
        set: SET_INIT,
        values: ::core::ptr::null_mut::<ptr_t>(),
    };
    (*mfp).mf_trans = Map_int64_t_int64_t {
        set: SET_INIT,
        values: ::core::ptr::null_mut::<int64_t>(),
    };
    (*mfp).mf_page_size = MEMFILE_PAGE_SIZE as ::core::ffi::c_uint;
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
    if (*mfp).mf_fd >= 0 as ::core::ffi::c_int
        && os_fileinfo_fd((*mfp).mf_fd, &raw mut file_info) as ::core::ffi::c_int != 0
    {
        let mut blocksize: uint64_t = os_fileinfo_blocksize(&raw mut file_info);
        if blocksize >= MIN_SWAP_PAGE_SIZE as ::core::ffi::c_int as uint64_t
            && blocksize <= MAX_SWAP_PAGE_SIZE as ::core::ffi::c_int as uint64_t
        {
            (*mfp).mf_page_size = blocksize as ::core::ffi::c_uint;
        }
    }
    let mut size: off_T = 0;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int || flags & (O_TRUNC | O_EXCL) != 0 || {
        size = lseek((*mfp).mf_fd, 0 as __off_t, SEEK_END) as off_T;
        size <= 0 as off_T
    } {
        (*mfp).mf_blocknr_max = 0 as blocknr_T;
    } else {
        '_c2rust_label: {
            if ::core::mem::size_of::<off_T>() <= ::core::mem::size_of::<blocknr_T>()
                && (*mfp).mf_page_size > 0 as ::core::ffi::c_uint
                && (*mfp).mf_page_size.wrapping_sub(1 as ::core::ffi::c_uint) as ::core::ffi::c_long
                    <= 9223372036854775807 as off_T - size
            {
            } else {
                __assert_fail(
                    b"sizeof(off_T) <= sizeof(blocknr_T) && mfp->mf_page_size > 0 && mfp->mf_page_size - 1 <= INT64_MAX - size\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memfile.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    133 as ::core::ffi::c_uint,
                    b"memfile_T *mf_open(char *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*mfp).mf_blocknr_max = (size as blocknr_T + (*mfp).mf_page_size as blocknr_T
            - 1 as blocknr_T)
            / (*mfp).mf_page_size as blocknr_T;
    }
    (*mfp).mf_blocknr_min = -1 as blocknr_T;
    (*mfp).mf_neg_count = 0 as blocknr_T;
    (*mfp).mf_infile_count = (*mfp).mf_blocknr_max;
    return mfp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_open_file(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if mf_do_open(mfp, fname, O_RDWR | O_CREAT | O_EXCL) {
        (*mfp).mf_dirty = MF_DIRTY_YES;
        return OK;
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn mf_close(mut mfp: *mut memfile_T, mut del_file: bool) {
    if mfp.is_null() {
        return;
    }
    if (*mfp).mf_fd >= 0 as ::core::ffi::c_int && close((*mfp).mf_fd) < 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_swapclose as *const ::core::ffi::c_char,
        ));
    }
    if del_file as ::core::ffi::c_int != 0 && !(*mfp).mf_fname.is_null() {
        os_remove((*mfp).mf_fname);
    }
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        mf_free_bhdr(hp);
        __i = __i.wrapping_add(1);
    }
    while !(*mfp).mf_free_first.is_null() {
        xfree(mf_rem_free(mfp) as *mut ::core::ffi::c_void);
    }
    xfree((*mfp).mf_hash.set.keys as *mut ::core::ffi::c_void);
    xfree((*mfp).mf_hash.set.h.hash as *mut ::core::ffi::c_void);
    (*mfp).mf_hash.set = SET_INIT;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_hash.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree((*mfp).mf_trans.set.keys as *mut ::core::ffi::c_void);
    xfree((*mfp).mf_trans.set.h.hash as *mut ::core::ffi::c_void);
    (*mfp).mf_trans.set = SET_INIT;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_trans.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
    mf_free_fnames(mfp);
    xfree(mfp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn mf_close_file(mut buf: *mut buf_T, mut getlines: bool) {
    let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
    if mfp.is_null() || (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        return;
    }
    if getlines {
        let mut lnum: linenr_T = 1 as linenr_T;
        while lnum <= (*buf).b_ml.ml_line_count {
            ml_get_buf(buf, lnum);
            lnum += 1;
        }
    }
    if close((*mfp).mf_fd) < 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_swapclose as *const ::core::ffi::c_char,
        ));
    }
    (*mfp).mf_fd = -1 as ::core::ffi::c_int;
    if !(*mfp).mf_fname.is_null() {
        os_remove((*mfp).mf_fname);
        mf_free_fnames(mfp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mf_new_page_size(
    mut mfp: *mut memfile_T,
    mut new_size: ::core::ffi::c_uint,
) {
    (*mfp).mf_page_size = new_size;
}
#[no_mangle]
pub unsafe extern "C" fn mf_new(
    mut mfp: *mut memfile_T,
    mut negative: bool,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut freep: *mut bhdr_T = (*mfp).mf_free_first;
    if !negative && !freep.is_null() && (*freep).bh_page_count >= page_count {
        if (*freep).bh_page_count > page_count {
            hp = mf_alloc_bhdr(mfp, page_count);
            (*hp).bh_bnum = (*freep).bh_bnum;
            (*freep).bh_bnum += page_count as blocknr_T;
            (*freep).bh_page_count = (*freep).bh_page_count.wrapping_sub(page_count);
        } else {
            let mut p: *mut ::core::ffi::c_void =
                xmalloc(((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t));
            hp = mf_rem_free(mfp);
            (*hp).bh_data = p;
        }
    } else {
        hp = mf_alloc_bhdr(mfp, page_count);
        if negative {
            let c2rust_fresh0 = (*mfp).mf_blocknr_min;
            (*mfp).mf_blocknr_min = (*mfp).mf_blocknr_min - 1;
            (*hp).bh_bnum = c2rust_fresh0;
            (*mfp).mf_neg_count += 1;
        } else {
            (*hp).bh_bnum = (*mfp).mf_blocknr_max;
            (*mfp).mf_blocknr_max += page_count as blocknr_T;
        }
    }
    (*hp).bh_flags = BH_LOCKED | BH_DIRTY;
    (*mfp).mf_dirty = MF_DIRTY_YES;
    (*hp).bh_page_count = page_count;
    map_put_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        hp as ptr_t,
    );
    memset(
        (*hp).bh_data,
        0 as ::core::ffi::c_int,
        ((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t),
    );
    return hp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_get(
    mut mfp: *mut memfile_T,
    mut nr: blocknr_T,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    if nr >= (*mfp).mf_blocknr_max || nr <= (*mfp).mf_blocknr_min {
        return ::core::ptr::null_mut::<bhdr_T>();
    }
    let mut hp: *mut bhdr_T =
        map_get_int64_t_ptr_t(&raw mut (*mfp).mf_hash, nr as int64_t) as *mut bhdr_T;
    if hp.is_null() {
        if nr < 0 as blocknr_T || nr >= (*mfp).mf_infile_count {
            return ::core::ptr::null_mut::<bhdr_T>();
        }
        if page_count > 0 as ::core::ffi::c_uint {
            hp = mf_alloc_bhdr(mfp, page_count);
        }
        if hp.is_null() {
            return ::core::ptr::null_mut::<bhdr_T>();
        }
        (*hp).bh_bnum = nr;
        (*hp).bh_flags = 0 as ::core::ffi::c_uint;
        (*hp).bh_page_count = page_count;
        if mf_read(mfp, hp) == FAIL {
            mf_free_bhdr(hp);
            return ::core::ptr::null_mut::<bhdr_T>();
        }
    } else {
        map_del_int64_t_ptr_t(
            &raw mut (*mfp).mf_hash,
            (*hp).bh_bnum as int64_t,
            ::core::ptr::null_mut::<int64_t>(),
        );
    }
    (*hp).bh_flags |= BH_LOCKED;
    map_put_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        hp as ptr_t,
    );
    return hp;
}
#[no_mangle]
pub unsafe extern "C" fn mf_put(
    mut mfp: *mut memfile_T,
    mut hp: *mut bhdr_T,
    mut dirty: bool,
    mut infile: bool,
) {
    let mut flags: ::core::ffi::c_uint = (*hp).bh_flags;
    if flags & BH_LOCKED == 0 as ::core::ffi::c_uint {
        iemsg(gettext(
            &raw const e_block_was_not_locked as *const ::core::ffi::c_char,
        ));
    }
    flags &= !BH_LOCKED;
    if dirty {
        flags |= BH_DIRTY;
        if (*mfp).mf_dirty as ::core::ffi::c_uint
            != MF_DIRTY_YES_NOSYNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*mfp).mf_dirty = MF_DIRTY_YES;
        }
    }
    (*hp).bh_flags = flags;
    if infile {
        mf_trans_add(mfp, hp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mf_free(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) {
    xfree((*hp).bh_data);
    map_del_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    if (*hp).bh_bnum < 0 as blocknr_T {
        xfree(hp as *mut ::core::ffi::c_void);
        (*mfp).mf_neg_count -= 1;
    } else {
        mf_ins_free(mfp, hp);
    };
}
#[no_mangle]
pub unsafe extern "C" fn mf_sync(
    mut mfp: *mut memfile_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut got_int_save: ::core::ffi::c_int = got_int as ::core::ffi::c_int;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        (*mfp).mf_dirty = MF_DIRTY_NO;
        return FAIL;
    }
    got_int = false_0 != 0;
    let mut status: ::core::ffi::c_int = OK;
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        if (flags & MFS_ALL as ::core::ffi::c_int != 0 || (*hp).bh_bnum >= 0 as blocknr_T)
            && (*hp).bh_flags & 1 as ::core::ffi::c_uint != 0
            && (status == 1 as ::core::ffi::c_int
                || (*hp).bh_bnum >= 0 as blocknr_T && (*hp).bh_bnum < (*mfp).mf_infile_count)
        {
            if !(flags & MFS_ZERO as ::core::ffi::c_int != 0 && (*hp).bh_bnum != 0 as blocknr_T) {
                if mf_write(mfp, hp) == 0 as ::core::ffi::c_int {
                    if status == 0 as ::core::ffi::c_int {
                        break;
                    }
                    status = 0 as ::core::ffi::c_int;
                }
                if flags & MFS_STOP as ::core::ffi::c_int != 0 {
                    if os_char_avail() {
                        break;
                    }
                } else if main_loop.recursive == 0 {
                    os_breakcheck();
                }
                if got_int {
                    break;
                }
            }
        }
        __i = __i.wrapping_add(1);
    }
    if hp.is_null() || status == FAIL {
        (*mfp).mf_dirty = MF_DIRTY_NO;
    }
    if flags & MFS_FLUSH as ::core::ffi::c_int != 0 {
        if os_fsync((*mfp).mf_fd) != 0 {
            status = FAIL;
        }
    }
    got_int = got_int as ::core::ffi::c_int | got_int_save != 0;
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn mf_set_dirty(mut mfp: *mut memfile_T) {
    let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*mfp).mf_hash.set.h.n_keys {
        hp = *(*mfp).mf_hash.values.offset(__i as isize) as *mut bhdr_T;
        if (*hp).bh_bnum > 0 as blocknr_T {
            (*hp).bh_flags |= 1 as ::core::ffi::c_uint;
        }
        __i = __i.wrapping_add(1);
    }
    (*mfp).mf_dirty = MF_DIRTY_YES;
}
#[no_mangle]
pub unsafe extern "C" fn mf_release_all() -> bool {
    let mut retval: bool = false_0 != 0;
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if !mfp.is_null() {
            if (*mfp).mf_fd < 0 as ::core::ffi::c_int
                && (*buf).b_may_swap as ::core::ffi::c_int != 0
            {
                ml_open_file(buf);
            }
            if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*mfp).mf_hash.set.h.size as ::core::ffi::c_int {
                    let mut hp: *mut bhdr_T =
                        *(*mfp).mf_hash.values.offset(i as isize) as *mut bhdr_T;
                    if (*hp).bh_flags & BH_LOCKED == 0
                        && ((*hp).bh_flags & BH_DIRTY == 0 || mf_write(mfp, hp) != FAIL)
                    {
                        map_del_int64_t_ptr_t(
                            &raw mut (*mfp).mf_hash,
                            (*hp).bh_bnum as int64_t,
                            ::core::ptr::null_mut::<int64_t>(),
                        );
                        mf_free_bhdr(hp);
                        retval = true_0 != 0;
                    } else {
                        i += 1;
                    }
                }
            }
        }
        buf = (*buf).b_next;
    }
    return retval;
}
unsafe extern "C" fn mf_alloc_bhdr(
    mut mfp: *mut memfile_T,
    mut page_count: ::core::ffi::c_uint,
) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = xmalloc(::core::mem::size_of::<bhdr_T>()) as *mut bhdr_T;
    (*hp).bh_data = xmalloc(((*mfp).mf_page_size as size_t).wrapping_mul(page_count as size_t));
    (*hp).bh_page_count = page_count;
    return hp;
}
unsafe extern "C" fn mf_free_bhdr(mut hp: *mut bhdr_T) {
    xfree((*hp).bh_data);
    xfree(hp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn mf_ins_free(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) {
    (*hp).bh_data = (*mfp).mf_free_first as *mut ::core::ffi::c_void;
    (*mfp).mf_free_first = hp;
}
unsafe extern "C" fn mf_rem_free(mut mfp: *mut memfile_T) -> *mut bhdr_T {
    let mut hp: *mut bhdr_T = (*mfp).mf_free_first;
    (*mfp).mf_free_first = (*hp).bh_data as *mut bhdr_T;
    return hp;
}
unsafe extern "C" fn mf_read(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) -> ::core::ffi::c_int {
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    let mut page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
    let mut offset: off_T = (page_size as blocknr_T * (*hp).bh_bnum) as off_T;
    if lseek((*mfp).mf_fd, offset as __off_t, SEEK_SET) != offset {
        semsg(
            b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(b"E294: Seek error in swap file read\0".as_ptr() as *const ::core::ffi::c_char),
            strerror(*__errno_location()),
        );
        return FAIL;
    }
    '_c2rust_label: {
        if (*hp).bh_page_count
            <= (2147483647 as ::core::ffi::c_int as ::core::ffi::c_uint)
                .wrapping_mul(2 as ::core::ffi::c_uint)
                .wrapping_add(1 as ::core::ffi::c_uint)
                .wrapping_div(page_size)
        {
        } else {
            __assert_fail(
                b"hp->bh_page_count <= UINT_MAX / page_size\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/memfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                545 as ::core::ffi::c_uint,
                b"int mf_read(memfile_T *, bhdr_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut size: ::core::ffi::c_uint = page_size.wrapping_mul((*hp).bh_page_count);
    if read_eintr((*mfp).mf_fd, (*hp).bh_data, size as size_t) as ::core::ffi::c_uint != size {
        semsg(
            b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            gettext(b"E295: Read error in swap file\0".as_ptr() as *const ::core::ffi::c_char),
            strerror(*__errno_location()),
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn mf_write(mut mfp: *mut memfile_T, mut hp: *mut bhdr_T) -> ::core::ffi::c_int {
    let mut hp2: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
    let mut page_count: ::core::ffi::c_uint = 0;
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int && !(*mfp).mf_reopen {
        return FAIL;
    }
    if (*hp).bh_bnum < 0 as blocknr_T {
        if mf_trans_add(mfp, hp) == FAIL {
            return FAIL;
        }
    }
    let mut page_size: ::core::ffi::c_uint = (*mfp).mf_page_size;
    loop {
        let mut nr: blocknr_T = (*hp).bh_bnum;
        if nr > (*mfp).mf_infile_count {
            nr = (*mfp).mf_infile_count;
            hp2 = map_get_int64_t_ptr_t(&raw mut (*mfp).mf_hash, nr as int64_t) as *mut bhdr_T;
        } else {
            hp2 = hp;
        }
        let mut offset: off_T = (page_size as blocknr_T * nr) as off_T;
        if hp2.is_null() {
            page_count = 1 as ::core::ffi::c_uint;
        } else {
            page_count = (*hp2).bh_page_count;
        }
        let mut size: ::core::ffi::c_uint = page_size.wrapping_mul(page_count);
        let mut attempt: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while attempt <= 2 as ::core::ffi::c_int {
            if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                if lseek((*mfp).mf_fd, offset as __off_t, SEEK_SET) != offset {
                    semsg(
                        b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"E296: Seek error in swap file write\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        strerror(*__errno_location()),
                    );
                    return FAIL;
                }
                let mut data: *mut ::core::ffi::c_void = if hp2.is_null() {
                    (*hp).bh_data
                } else {
                    (*hp2).bh_data
                };
                if write_eintr((*mfp).mf_fd, data, size as size_t) as ::core::ffi::c_uint == size {
                    break;
                }
            }
            if attempt == 1 as ::core::ffi::c_int {
                if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                    close((*mfp).mf_fd);
                }
                (*mfp).mf_fd = os_open((*mfp).mf_fname, (*mfp).mf_flags, S_IREAD | S_IWRITE);
                (*mfp).mf_reopen = (*mfp).mf_fd < 0 as ::core::ffi::c_int;
            }
            if attempt == 2 as ::core::ffi::c_int || (*mfp).mf_fd < 0 as ::core::ffi::c_int {
                if !did_swapwrite_msg {
                    emsg(gettext(
                        b"E297: Write error in swap file\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                did_swapwrite_msg = true_0 != 0;
                return FAIL;
            }
            attempt += 1;
        }
        did_swapwrite_msg = false_0 != 0;
        if !hp2.is_null() {
            (*hp2).bh_flags &= !BH_DIRTY;
        }
        if nr + page_count as blocknr_T > (*mfp).mf_infile_count {
            (*mfp).mf_infile_count = nr + page_count as blocknr_T;
        }
        if nr == (*hp).bh_bnum {
            break;
        }
    }
    return OK;
}
unsafe extern "C" fn mf_trans_add(
    mut mfp: *mut memfile_T,
    mut hp: *mut bhdr_T,
) -> ::core::ffi::c_int {
    if (*hp).bh_bnum >= 0 as blocknr_T {
        return OK;
    }
    let mut new_bnum: blocknr_T = 0;
    let mut freep: *mut bhdr_T = (*mfp).mf_free_first;
    let mut page_count: ::core::ffi::c_uint = (*hp).bh_page_count;
    if !freep.is_null() && (*freep).bh_page_count >= page_count {
        new_bnum = (*freep).bh_bnum;
        if (*freep).bh_page_count > page_count {
            (*freep).bh_bnum += page_count as blocknr_T;
            (*freep).bh_page_count = (*freep).bh_page_count.wrapping_sub(page_count);
        } else {
            freep = mf_rem_free(mfp);
            xfree(freep as *mut ::core::ffi::c_void);
        }
    } else {
        new_bnum = (*mfp).mf_blocknr_max;
        (*mfp).mf_blocknr_max += page_count as blocknr_T;
    }
    let mut old_bnum: blocknr_T = (*hp).bh_bnum;
    map_del_int64_t_ptr_t(
        &raw mut (*mfp).mf_hash,
        (*hp).bh_bnum as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    (*hp).bh_bnum = new_bnum;
    map_put_int64_t_ptr_t(&raw mut (*mfp).mf_hash, new_bnum as int64_t, hp as ptr_t);
    map_put_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_bnum as int64_t,
        new_bnum as int64_t,
    );
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn mf_trans_del(mut mfp: *mut memfile_T, mut old_nr: blocknr_T) -> blocknr_T {
    let mut num: *mut blocknr_T = map_ref_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_nr as int64_t,
        ::core::ptr::null_mut::<*mut int64_t>(),
    ) as *mut blocknr_T;
    if num.is_null() {
        return old_nr;
    }
    (*mfp).mf_neg_count -= 1;
    let mut new_bnum: blocknr_T = *num;
    map_del_int64_t_int64_t(
        &raw mut (*mfp).mf_trans,
        old_nr as int64_t,
        ::core::ptr::null_mut::<int64_t>(),
    );
    return new_bnum;
}
#[no_mangle]
pub unsafe extern "C" fn mf_free_fnames(mut mfp: *mut memfile_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_fname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*mfp).mf_ffname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
}
#[no_mangle]
pub unsafe extern "C" fn mf_set_fnames(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
) {
    (*mfp).mf_fname = fname;
    (*mfp).mf_ffname = FullName_save((*mfp).mf_fname, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn mf_fullname(mut mfp: *mut memfile_T) {
    if mfp.is_null() || (*mfp).mf_fname.is_null() || (*mfp).mf_ffname.is_null() {
        return;
    }
    xfree((*mfp).mf_fname as *mut ::core::ffi::c_void);
    (*mfp).mf_fname = (*mfp).mf_ffname;
    (*mfp).mf_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn mf_need_trans(mut mfp: *mut memfile_T) -> bool {
    return !(*mfp).mf_fname.is_null() && (*mfp).mf_neg_count > 0 as blocknr_T;
}
unsafe extern "C" fn mf_do_open(
    mut mfp: *mut memfile_T,
    mut fname: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    mf_set_fnames(mfp, fname);
    '_c2rust_label: {
        if !(*mfp).mf_fname.is_null() {
        } else {
            __assert_fail(
                b"mfp->mf_fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/memfile.rs\0".as_ptr() as *const ::core::ffi::c_char,
                763 as ::core::ffi::c_uint,
                b"_Bool mf_do_open(memfile_T *, char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
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
    if flags & O_CREAT != 0
        && os_fileinfo_link((*mfp).mf_fname, &raw mut file_info) as ::core::ffi::c_int != 0
    {
        (*mfp).mf_fd = -1 as ::core::ffi::c_int;
        emsg(gettext(
            b"E300: Swap file already exists (symlink attack?)\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    } else {
        flags |= O_NOFOLLOW;
        (*mfp).mf_flags = flags;
        (*mfp).mf_fd = os_open((*mfp).mf_fname, flags, S_IREAD | S_IWRITE);
    }
    if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
        mf_free_fnames(mfp);
        return false_0 != 0;
    }
    os_set_cloexec((*mfp).mf_fd);
    return true_0 != 0;
}
pub const __S_IREAD: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const __S_IWRITE: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
