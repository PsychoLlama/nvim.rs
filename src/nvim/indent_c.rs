extern "C" {
    pub type MsgpackRpcRequestHandler;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn tolower(__c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut p_paste: ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getwhitecols_curline() -> intptr_t;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn tv_get_lnum(tv: *const typval_T) -> linenr_T;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut State: ::core::ffi::c_int;
    fn get_sw_value(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn get_indent() -> ::core::ffi::c_int;
    fn get_indent_lnum(lnum: linenr_T) -> ::core::ffi::c_int;
    fn get_expr_indent() -> ::core::ffi::c_int;
    fn fixthisline(get_the_indent: IndentGetter);
    fn get_special_key_code(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn trim_to_int(x: int64_t) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn mb_prevptr(
        line: *mut ::core::ffi::c_char,
        p: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_pos(pos: *const pos_T) -> *mut ::core::ffi::c_char;
    fn skip_to_option_part(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn findmatchlimit(
        oap: *mut oparg_T,
        initc: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        maxtravel: int64_t,
    ) -> *mut pos_T;
    fn check_linecomment(line: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn linewhite(lnum: linenr_T) -> bool;
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type uintmax_t = ::libc::uintmax_t;
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type time_t = __time_t;
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
pub const MAXLNUM: C2Rust_Unnamed = 2147483647;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_0 = 2147483647;
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
    pub b_wininfo: C2Rust_Unnamed_12,
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
    pub b_signcols: C2Rust_Unnamed_4,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_2,
    pub update_callbacks: C2Rust_Unnamed_1,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub data: C2Rust_Unnamed_3,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
pub struct C2Rust_Unnamed_4 {
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
    pub sst_union: C2Rust_Unnamed_5,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub data: C2Rust_Unnamed_6,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub fc_fixvar: [C2Rust_Unnamed_7; 12],
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
pub struct C2Rust_Unnamed_7 {
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
    pub uh_next: C2Rust_Unnamed_11,
    pub uh_prev: C2Rust_Unnamed_10,
    pub uh_alt_next: C2Rust_Unnamed_9,
    pub uh_alt_prev: C2Rust_Unnamed_8,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
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
pub struct C2Rust_Unnamed_12 {
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
    pub type_0: C2Rust_Unnamed_13,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const KEY_COMPLETE: C2Rust_Unnamed_14 = 259;
pub const KEY_OPEN_BACK: C2Rust_Unnamed_14 = 258;
pub const KEY_OPEN_FORW: C2Rust_Unnamed_14 = 257;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_15 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_15 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_15 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_15 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_15 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_15 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_15 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_15 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_15 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_15 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_15 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_15 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_15 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_15 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
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
pub type IndentGetter = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
pub const FM_BACKWARD: C2Rust_Unnamed_16 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cpp_baseclass_cache_T {
    pub found: ::core::ffi::c_int,
    pub lpos: lpos_T,
}
pub const FM_BLOCKSTOP: C2Rust_Unnamed_16 = 4;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_16 = 8;
pub const FM_FORWARD: C2Rust_Unnamed_16 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_LEFT: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const COM_RIGHT: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn lt(mut a: pos_T, mut b: pos_T) -> bool {
    if a.lnum != b.lnum {
        return a.lnum < b.lnum;
    } else if a.col != b.col {
        return a.col < b.col;
    } else {
        return a.coladd < b.coladd;
    };
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"_Bool in_cinkeys(int, int, _Bool)\0",
    )
};
unsafe extern "C" fn ind_find_start_comment() -> *mut pos_T {
    return find_start_comment((*curbuf).b_ind_maxcomment);
}
#[no_mangle]
pub unsafe extern "C" fn find_start_comment(mut ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut cur_maxcomment: int64_t = ind_maxcomment as int64_t;
    loop {
        pos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            '*' as ::core::ffi::c_int,
            FM_BACKWARD as ::core::ffi::c_int,
            cur_maxcomment,
        );
        if pos.is_null() {
            break;
        }
        if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
            break;
        }
        cur_maxcomment = ((*curwin).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as int64_t;
        if cur_maxcomment > 0 as int64_t {
            continue;
        }
        pos = ::core::ptr::null_mut::<pos_T>();
        break;
    }
    return pos;
}
unsafe extern "C" fn ind_find_start_CORS(mut is_raw: *mut linenr_T) -> *mut pos_T {
    static mut comment_pos_copy: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut comment_pos: *mut pos_T = find_start_comment((*curbuf).b_ind_maxcomment);
    if !comment_pos.is_null() {
        comment_pos_copy = *comment_pos;
        comment_pos = &raw mut comment_pos_copy;
    }
    let mut rs_pos: *mut pos_T = find_start_rawstring((*curbuf).b_ind_maxcomment);
    if comment_pos.is_null()
        || !rs_pos.is_null() && lt(*rs_pos, *comment_pos) as ::core::ffi::c_int != 0
    {
        if !is_raw.is_null() && !rs_pos.is_null() {
            *is_raw = (*rs_pos).lnum;
        }
        return rs_pos;
    }
    return comment_pos;
}
unsafe extern "C" fn find_start_rawstring(mut ind_maxcomment: ::core::ffi::c_int) -> *mut pos_T {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut cur_maxcomment: ::core::ffi::c_int = ind_maxcomment;
    loop {
        pos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            'R' as ::core::ffi::c_int,
            FM_BACKWARD as ::core::ffi::c_int,
            cur_maxcomment as int64_t,
        );
        if pos.is_null() {
            break;
        }
        if is_pos_in_string(ml_get((*pos).lnum), (*pos).col) == 0 {
            break;
        }
        cur_maxcomment =
            ((*curwin).w_cursor.lnum - (*pos).lnum - 1 as linenr_T) as ::core::ffi::c_int;
        if cur_maxcomment > 0 as ::core::ffi::c_int {
            continue;
        }
        pos = ::core::ptr::null_mut::<pos_T>();
        break;
    }
    return pos;
}
unsafe extern "C" fn skip_string(mut p: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0;
    loop {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\'' as ::core::ffi::c_int
        {
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            i = 2 as ::core::ffi::c_int;
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                i += 1;
                while ascii_isdigit(
                    *p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                ) {
                    i += 1;
                }
            }
            if !(*p.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != NUL
                && *p.offset(i as isize) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
            {
                break;
            }
            p = p.offset(i as isize);
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '"' as ::core::ffi::c_int
        {
            p = p.offset(1);
            while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                {
                    break;
                }
                p = p.offset(1);
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
            {
                break;
            }
        } else {
            if !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int)
            {
                break;
            }
            let mut delim: *const ::core::ffi::c_char = p.offset(2 as ::core::ffi::c_int as isize);
            let mut paren: *const ::core::ffi::c_char =
                vim_strchr(delim, '(' as ::core::ffi::c_int);
            if paren.is_null() {
                break;
            }
            let delim_len: ptrdiff_t = paren.offset_from(delim);
            p = p.offset(3 as ::core::ffi::c_int as isize);
            while *p != 0 {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ')' as ::core::ffi::c_int
                    && strncmp(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        delim,
                        delim_len as size_t,
                    ) == 0 as ::core::ffi::c_int
                    && *p.offset((delim_len + 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    p = p.offset((delim_len + 1 as ptrdiff_t) as isize);
                    break;
                } else {
                    p = p.offset(1);
                }
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
            {
                break;
            }
        }
        p = p.offset(1);
    }
    if *p == 0 {
        p = p.offset(-1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn is_pos_in_string(
    mut line: *const ::core::ffi::c_char,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = line;
    while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < col {
        p = skip_string(p);
        p = p.offset(1);
    }
    return !(p.offset_from(line) as colnr_T <= col) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cin_is_cinword(mut line: *const ::core::ffi::c_char) -> bool {
    let mut retval: bool = false_0 != 0;
    let mut cinw_len: size_t = strlen((*curbuf).b_p_cinw).wrapping_add(1 as size_t);
    let mut cinw_buf: *mut ::core::ffi::c_char = xmalloc(cinw_len) as *mut ::core::ffi::c_char;
    line = skipwhite(line);
    let mut cinw: *mut ::core::ffi::c_char = (*curbuf).b_p_cinw;
    while *cinw != 0 {
        let mut len: size_t = copy_option_part(
            &raw mut cinw,
            cinw_buf,
            cinw_len,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if !(strncmp(line, cinw_buf, len) == 0 as ::core::ffi::c_int
            && (!vim_iswordc(*line.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                || !vim_iswordc(
                    *line.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t
                        as ::core::ffi::c_int,
                )))
        {
            continue;
        }
        retval = true_0 != 0;
        break;
    }
    xfree(cinw_buf as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn cindent_on() -> bool {
    return p_paste == 0
        && ((*curbuf).b_p_cin != 0 || *(*curbuf).b_p_inde as ::core::ffi::c_int != NUL);
}
unsafe extern "C" fn cin_skipcomment(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    while *s != 0 {
        let mut prev_s: *const ::core::ffi::c_char = s;
        s = skipwhite(s);
        if (*curbuf).b_ind_hash_comment != 0 as ::core::ffi::c_int
            && s != prev_s
            && *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
        {
            s = s.offset(strlen(s) as isize);
            break;
        } else {
            if *s as ::core::ffi::c_int != '/' as ::core::ffi::c_int {
                break;
            }
            s = s.offset(1);
            if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                s = s.offset(strlen(s) as isize);
                break;
            } else {
                if *s as ::core::ffi::c_int != '*' as ::core::ffi::c_int {
                    break;
                }
                s = s.offset(1);
                while *s != 0 {
                    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '/' as ::core::ffi::c_int
                    {
                        s = s.offset(2 as ::core::ffi::c_int as isize);
                        break;
                    } else {
                        s = s.offset(1);
                    }
                }
            }
        }
    }
    return s;
}
unsafe extern "C" fn cin_nocode(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*cin_skipcomment(s) as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
}
unsafe extern "C" fn find_line_comment() -> *mut pos_T {
    static mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    pos = (*curwin).w_cursor;
    loop {
        pos.lnum -= 1;
        if pos.lnum <= 0 as linenr_T {
            break;
        }
        line = ml_get(pos.lnum);
        p = skipwhite(line);
        if cin_islinecomment(p) != 0 {
            pos.col = p.offset_from(line) as ::core::ffi::c_int as colnr_T;
            return &raw mut pos;
        }
        if *p as ::core::ffi::c_int != NUL {
            break;
        }
    }
    return ::core::ptr::null_mut::<pos_T>();
}
unsafe extern "C" fn cin_has_js_key(mut text: *const ::core::ffi::c_char) -> bool {
    let mut s: *const ::core::ffi::c_char = skipwhite(text);
    let mut quote: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int
    {
        quote = *s;
        s = s.offset(1);
    }
    if !vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    while vim_isIDc(*s as uint8_t as ::core::ffi::c_int) {
        s = s.offset(1);
    }
    if *s as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == quote as ::core::ffi::c_int {
        s = s.offset(1);
    }
    s = cin_skipcomment(s);
    return *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != ':' as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_islabel_skip(mut s: *mut *const ::core::ffi::c_char) -> bool {
    if !vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    while vim_isIDc(**s as uint8_t as ::core::ffi::c_int) {
        *s = (*s).offset(utfc_ptr2len(*s) as isize);
    }
    *s = cin_skipcomment(*s);
    return **s as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
        *s = (*s).offset(1);
        **s as ::core::ffi::c_int != ':' as ::core::ffi::c_int
    };
}
unsafe extern "C" fn cin_islabel() -> bool {
    let mut s: *const ::core::ffi::c_char = cin_skipcomment(get_cursor_line_ptr());
    if cin_isdefault(s) != 0 {
        return false_0 != 0;
    }
    if cin_isscopedecl(s) {
        return false_0 != 0;
    }
    if !cin_islabel_skip(&raw mut s) {
        return false_0 != 0;
    }
    if !ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>()).is_null() {
        return false_0 != 0;
    }
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    cursor_save = (*curwin).w_cursor;
    while (*curwin).w_cursor.lnum > 1 as linenr_T {
        (*curwin).w_cursor.lnum -= 1;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        trypos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
        if !trypos.is_null() {
            (*curwin).w_cursor = *trypos;
        }
        line = get_cursor_line_ptr();
        if cin_ispreproc(line) != 0 {
            continue;
        }
        line = cin_skipcomment(line);
        if *line as ::core::ffi::c_int == NUL {
            continue;
        }
        (*curwin).w_cursor = cursor_save;
        if cin_isterminated(line, true_0, false_0) as ::core::ffi::c_int != 0
            || cin_isscopedecl(line) as ::core::ffi::c_int != 0
            || cin_iscase(line, true_0 != 0) as ::core::ffi::c_int != 0
            || cin_islabel_skip(&raw mut line) as ::core::ffi::c_int != 0 && cin_nocode(line) != 0
        {
            return true_0 != 0;
        }
        return false_0 != 0;
    }
    (*curwin).w_cursor = cursor_save;
    return true_0 != 0;
}
unsafe extern "C" fn cin_skip_comment_and_string(
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = s;
    loop {
        r = p;
        p = cin_skipcomment(p);
        if *p != 0 {
            p = skip_string(p);
        }
        if p == r {
            break;
        }
    }
    return p;
}
unsafe extern "C" fn cin_is_compound_init(mut s: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = s;
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    while *p != 0 {
        if *p as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            r = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
            p = r;
        } else if strncmp(
            p,
            b"return\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0
            && !vim_isIDc(*p.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            && (p == s
                || p > s
                    && !vim_isIDc(
                        *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ))
        {
            r = cin_skipcomment(p.offset(6 as ::core::ffi::c_int as isize));
            p = r;
        } else {
            p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
        }
    }
    if r.is_null() {
        return false_0 != 0;
    }
    p = r;
    if cin_nocode(p) != 0 {
        return true_0 != 0;
    }
    if *p as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    if *p as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
        let mut open_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            p = cin_skip_comment_and_string(p.offset(1 as ::core::ffi::c_int as isize));
            if cin_nocode(p) != 0 {
                return true_0 != 0;
            }
            open_count += (*p as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
                as ::core::ffi::c_int
                - (*p as ::core::ffi::c_int == ')' as ::core::ffi::c_int) as ::core::ffi::c_int;
            if open_count == 0 {
                break;
            }
        }
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
        if cin_nocode(p) != 0 {
            return true_0 != 0;
        }
    }
    while *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    return cin_nocode(p) != 0;
}
unsafe extern "C" fn cin_isinit() -> bool {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut skip: [*mut ::core::ffi::c_char; 4] = [
        b"static\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"public\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"protected\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"private\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ];
    s = cin_skipcomment(get_cursor_line_ptr());
    if cin_starts_with(s, b"typedef\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
    }
    loop {
        let mut i: ::core::ffi::c_int = 0;
        let mut l: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
        {
            l = strlen(skip[i as usize]) as ::core::ffi::c_int;
            if cin_starts_with(s, skip[i as usize]) != 0 {
                s = cin_skipcomment(s.offset(l as isize));
                l = 0 as ::core::ffi::c_int;
                break;
            } else {
                i += 1;
            }
        }
        if l != 0 as ::core::ffi::c_int {
            break;
        }
    }
    if cin_starts_with(s, b"enum\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        return true_0 != 0;
    }
    return cin_is_compound_init(s);
}
unsafe extern "C" fn cin_iscase(mut s: *const ::core::ffi::c_char, mut strict: bool) -> bool {
    s = cin_skipcomment(s);
    if cin_starts_with(s, b"case\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        s = s.offset(4 as ::core::ffi::c_int as isize);
        while *s != 0 {
            s = cin_skipcomment(s);
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
            if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                {
                    s = s.offset(1);
                } else {
                    return true_0 != 0;
                }
            }
            if *s as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\'' as ::core::ffi::c_int
            {
                s = s.offset(2 as ::core::ffi::c_int as isize);
            } else if *s as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                && (*s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                    || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int)
            {
                return false_0 != 0;
            } else if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                if strict {
                    return false_0 != 0;
                }
                return true_0 != 0;
            }
            s = s.offset(1);
        }
        return false_0 != 0;
    }
    if cin_isdefault(s) != 0 {
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn cin_isdefault(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        s,
        b"default\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
        && {
            s = cin_skipcomment(s.offset(7 as ::core::ffi::c_int as isize));
            *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        }
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != ':' as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isscopedecl(mut p: *const ::core::ffi::c_char) -> bool {
    let mut s: *const ::core::ffi::c_char = cin_skipcomment(p);
    let cinsd_len: size_t = strlen((*curbuf).b_p_cinsd).wrapping_add(1 as size_t);
    let mut cinsd_buf: *mut ::core::ffi::c_char = xmalloc(cinsd_len) as *mut ::core::ffi::c_char;
    let mut found: bool = false_0 != 0;
    let mut cinsd: *mut ::core::ffi::c_char = (*curbuf).b_p_cinsd;
    while *cinsd != 0 {
        let len: size_t = copy_option_part(
            &raw mut cinsd,
            cinsd_buf,
            cinsd_len,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if strncmp(s, cinsd_buf, len) != 0 as ::core::ffi::c_int {
            continue;
        }
        let mut skip: *const ::core::ffi::c_char = cin_skipcomment(s.offset(len as isize));
        if !(*skip as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *skip.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ':' as ::core::ffi::c_int)
        {
            continue;
        }
        found = true_0 != 0;
        break;
    }
    xfree(cinsd_buf as *mut ::core::ffi::c_void);
    return found;
}
pub const FIND_NAMESPACE_LIM: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
unsafe extern "C" fn cin_is_cpp_namespace(mut s: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut has_name: bool = false_0 != 0;
    let mut has_name_start: bool = false_0 != 0;
    s = cin_skipcomment(s);
    while (strncmp(
        s,
        b"inline\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            s,
            b"export\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int)
        && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        s = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
    }
    if strncmp(
        s,
        b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
        9 as size_t,
    ) == 0 as ::core::ffi::c_int
        && (*s.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(9 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        p = cin_skipcomment(skipwhite(s.offset(9 as ::core::ffi::c_int as isize)));
        while *p as ::core::ffi::c_int != NUL {
            if ascii_iswhite(*p as ::core::ffi::c_int) {
                has_name = true_0 != 0;
                p = cin_skipcomment(skipwhite(p));
            } else {
                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                    break;
                }
                if vim_iswordc(*p as uint8_t as ::core::ffi::c_int) {
                    has_name_start = true_0 != 0;
                    if has_name {
                        return false_0 != 0;
                    }
                    p = p.offset(1);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    && vim_iswordc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0
                {
                    if !has_name_start || has_name as ::core::ffi::c_int != 0 {
                        return false_0 != 0;
                    }
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else {
                    return false_0 != 0;
                }
            }
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn after_label(mut l: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    while *l != 0 {
        if *l as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                l = l.offset(1);
            } else if !cin_iscase(l.offset(1 as ::core::ffi::c_int as isize), false_0 != 0) {
                break;
            }
        } else if *l as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
            && *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && *l.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\'' as ::core::ffi::c_int
        {
            l = l.offset(2 as ::core::ffi::c_int as isize);
        }
        l = l.offset(1);
    }
    if *l as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    l = cin_skipcomment(l.offset(1 as ::core::ffi::c_int as isize));
    if *l as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return l;
}
unsafe extern "C" fn get_indent_nolabel(mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut col: colnr_T = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    l = ml_get(lnum);
    p = after_label(l);
    if p.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    fp.col = p.offset_from(l) as colnr_T;
    fp.lnum = lnum;
    getvcol(
        curwin,
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn skip_label(
    mut lnum: linenr_T,
    mut pp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut amount: ::core::ffi::c_int = 0;
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    cursor_save = (*curwin).w_cursor;
    (*curwin).w_cursor.lnum = lnum;
    l = get_cursor_line_ptr();
    if cin_iscase(l, false_0 != 0) as ::core::ffi::c_int != 0
        || cin_isscopedecl(l) as ::core::ffi::c_int != 0
        || cin_islabel() as ::core::ffi::c_int != 0
    {
        amount = get_indent_nolabel(lnum);
        l = after_label(get_cursor_line_ptr());
        if l.is_null() {
            l = get_cursor_line_ptr();
        }
    } else {
        amount = get_indent();
        l = get_cursor_line_ptr();
    }
    *pp = l;
    (*curwin).w_cursor = cursor_save;
    return amount;
}
unsafe extern "C" fn cin_first_id_amount() -> ::core::ffi::c_int {
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut col: colnr_T = 0;
    line = get_cursor_line_ptr();
    p = skipwhite(line);
    len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
    if len == 6 as ::core::ffi::c_int
        && strncmp(
            p,
            b"static\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
        len = skiptowhite(p).offset_from(p) as ::core::ffi::c_int;
    }
    if len == 6 as ::core::ffi::c_int
        && strncmp(
            p,
            b"struct\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
    } else if len == 4 as ::core::ffi::c_int
        && strncmp(
            p,
            b"enum\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
    } else if len == 8 as ::core::ffi::c_int
        && strncmp(
            p,
            b"unsigned\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        || len == 6 as ::core::ffi::c_int
            && strncmp(
                p,
                b"signed\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
    {
        s = skipwhite(p.offset(len as isize));
        if strncmp(
            s,
            b"int\0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_iswhite(*s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
            || strncmp(
                s,
                b"long\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            || strncmp(
                s,
                b"short\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            || strncmp(
                s,
                b"char\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_iswhite(*s.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
        {
            p = s;
        }
    }
    len = 0 as ::core::ffi::c_int;
    while vim_isIDc(*p.offset(len as isize) as uint8_t as ::core::ffi::c_int) {
        len += 1;
    }
    if len == 0 as ::core::ffi::c_int
        || !ascii_iswhite(*p.offset(len as isize) as ::core::ffi::c_int)
        || cin_nocode(p) != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    p = skipwhite(p.offset(len as isize));
    fp.lnum = (*curwin).w_cursor.lnum;
    fp.col = p.offset_from(line) as colnr_T;
    getvcol(
        curwin,
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn cin_get_equal_amount(mut lnum: linenr_T) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut col: colnr_T = 0;
    let mut fp: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    if lnum > 1 as linenr_T {
        line = ml_get(lnum - 1 as linenr_T);
        if *line as ::core::ffi::c_int != NUL
            && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
    }
    s = ml_get(lnum);
    line = s;
    while *s as ::core::ffi::c_int != NUL
        && vim_strchr(
            b"=;{}\"'\0".as_ptr() as *const ::core::ffi::c_char,
            *s as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else {
            s = s.offset(1);
        }
    }
    if *s as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    s = skipwhite(s.offset(1 as ::core::ffi::c_int as isize));
    if cin_nocode(s) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if *s as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        s = s.offset(1);
    }
    fp.lnum = lnum;
    fp.col = s.offset_from(line) as colnr_T;
    getvcol(
        curwin,
        &raw mut fp,
        &raw mut col,
        ::core::ptr::null_mut::<colnr_T>(),
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return col;
}
unsafe extern "C" fn cin_ispreproc(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *skipwhite(s) as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        return true_0;
    }
    return false_0;
}
unsafe extern "C" fn cin_ispreproc_cont(
    mut pp: *mut *const ::core::ffi::c_char,
    mut lnump: *mut linenr_T,
    mut amount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = *pp;
    let mut lnum: linenr_T = *lnump;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut candidate_amount: ::core::ffi::c_int = *amount;
    if *line as ::core::ffi::c_int != NUL
        && *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
    {
        candidate_amount = get_indent_lnum(lnum);
    }
    loop {
        if cin_ispreproc(line) != 0 {
            retval = true_0;
            *lnump = lnum;
            break;
        } else {
            if lnum == 1 as linenr_T {
                break;
            }
            lnum -= 1;
            line = ml_get(lnum);
            if *line as ::core::ffi::c_int == NUL
                || *line.offset(strlen(line).wrapping_sub(1 as size_t) as isize)
                    as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                break;
            }
        }
    }
    if lnum != *lnump {
        *pp = ml_get(*lnump);
    }
    if retval != 0 {
        *amount = candidate_amount;
    }
    return retval;
}
unsafe extern "C" fn cin_iscomment(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int)) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_islinecomment(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isterminated(
    mut s: *const ::core::ffi::c_char,
    mut incl_open: ::core::ffi::c_int,
    mut incl_comma: ::core::ffi::c_int,
) -> ::core::ffi::c_char {
    let mut found_start: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut n_open: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut is_else: ::core::ffi::c_int = false_0;
    s = cin_skipcomment(s);
    if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int && cin_iselse(s) == 0
    {
        found_start = *s;
    }
    if found_start == 0 {
        is_else = cin_iselse(s);
    }
    while *s != 0 {
        s = skip_string(cin_skipcomment(s));
        if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
            && n_open > 0 as ::core::ffi::c_uint
        {
            n_open = n_open.wrapping_sub(1);
        }
        if (is_else == 0 || n_open == 0 as ::core::ffi::c_uint)
            && (*s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                || incl_comma != 0 && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            return *s;
        } else if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            if incl_open != 0 && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0 {
                return *s;
            } else {
                n_open = n_open.wrapping_add(1);
            }
        }
        if *s != 0 {
            s = s.offset(1);
        }
    }
    return found_start;
}
unsafe extern "C" fn cin_isfuncdecl(
    mut sp: *mut *const ::core::ffi::c_char,
    mut first_lnum: linenr_T,
    mut min_lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut lnum: linenr_T = first_lnum;
    let mut save_lnum: linenr_T = (*curwin).w_cursor.lnum;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut just_started: ::core::ffi::c_int = true_0;
    if sp.is_null() {
        s = ml_get(lnum);
    } else {
        s = *sp;
    }
    (*curwin).w_cursor.lnum = lnum;
    if find_last_paren(s, '(' as ::core::ffi::c_char, ')' as ::core::ffi::c_char) != 0 && {
        trypos = find_match_paren((*curbuf).b_ind_maxparen);
        !trypos.is_null()
    } {
        lnum = (*trypos).lnum;
        if lnum < min_lnum {
            (*curwin).w_cursor.lnum = save_lnum;
            return false_0;
        }
        s = ml_get(lnum);
    }
    (*curwin).w_cursor.lnum = save_lnum;
    if cin_ispreproc(s) != 0 {
        return false_0;
    }
    while *s as ::core::ffi::c_int != 0
        && *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
    {
        if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else if *s as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                s = s.offset(2 as ::core::ffi::c_int as isize);
            } else {
                return false_0;
            }
        } else {
            s = s.offset(1);
        }
    }
    if *s as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        return false_0;
    }
    while *s as ::core::ffi::c_int != 0
        && *s as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
        && *s as ::core::ffi::c_int != '"' as ::core::ffi::c_int
    {
        if *s as ::core::ffi::c_int == ')' as ::core::ffi::c_int
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            lnum = first_lnum - 1 as linenr_T;
            s = ml_get(lnum);
            if *s as ::core::ffi::c_int == NUL
                || *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                retval = true_0;
            }
            break;
        } else if *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || cin_nocode(s) != 0
        {
            let mut comma: ::core::ffi::c_int =
                (*s as ::core::ffi::c_int == ',' as ::core::ffi::c_int) as ::core::ffi::c_int;
            while lnum < (*curbuf).b_ml.ml_line_count {
                lnum += 1;
                s = ml_get(lnum);
                if cin_ispreproc(s) == 0 {
                    break;
                }
            }
            if lnum >= (*curbuf).b_ml.ml_line_count {
                break;
            }
            s = skipwhite(s);
            if just_started == 0
                && (comma == 0
                    && *s as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                    && *s as ::core::ffi::c_int != ')' as ::core::ffi::c_int)
            {
                break;
            }
            just_started = false_0;
        } else if cin_iscomment(s) != 0 {
            s = cin_skipcomment(s);
        } else {
            s = s.offset(1);
            just_started = false_0;
        }
    }
    if lnum != first_lnum && !sp.is_null() {
        *sp = ml_get(first_lnum);
    }
    return retval;
}
unsafe extern "C" fn cin_isif(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"if\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iselse(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    return (strncmp(
        p,
        b"else\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(4 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_isdo(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"do\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iswhileofdo(
    mut p: *const ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut retval: ::core::ffi::c_int = false_0;
    p = cin_skipcomment(p);
    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
        p = cin_skipcomment(p.offset(1 as ::core::ffi::c_int as isize));
    }
    if cin_starts_with(p, b"while\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
        cursor_save = (*curwin).w_cursor;
        (*curwin).w_cursor.lnum = lnum;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        p = get_cursor_line_ptr();
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != 'w' as ::core::ffi::c_int
        {
            p = p.offset(1);
            (*curwin).w_cursor.col += 1;
        }
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*curbuf).b_ind_maxparen as int64_t,
        );
        if !trypos.is_null()
            && *cin_skipcomment(ml_get_pos(trypos).offset(1 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int
                == ';' as ::core::ffi::c_int
        {
            retval = true_0;
        }
        (*curwin).w_cursor = cursor_save;
    }
    return retval;
}
unsafe extern "C" fn cin_is_if_for_while_before_offset(
    mut line: *const ::core::ffi::c_char,
    mut poffset: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut offset: ::core::ffi::c_int = *poffset;
    let c2rust_fresh3 = offset;
    offset = offset - 1;
    if c2rust_fresh3 < 2 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    while offset > 2 as ::core::ffi::c_int
        && ascii_iswhite(*line.offset(offset as isize) as ::core::ffi::c_int) as ::core::ffi::c_int
            != 0
    {
        offset -= 1;
    }
    offset -= 1 as ::core::ffi::c_int;
    '_probablyFound: {
        if strncmp(
            line.offset(offset as isize),
            b"if\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) != 0
        {
            if offset >= 1 as ::core::ffi::c_int {
                offset -= 1 as ::core::ffi::c_int;
                if strncmp(
                    line.offset(offset as isize),
                    b"for\0".as_ptr() as *const ::core::ffi::c_char,
                    3 as size_t,
                ) == 0
                {
                    break '_probablyFound;
                } else if offset >= 2 as ::core::ffi::c_int {
                    offset -= 2 as ::core::ffi::c_int;
                    if strncmp(
                        line.offset(offset as isize),
                        b"while\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0
                    {
                        break '_probablyFound;
                    }
                }
            }
            return 0 as ::core::ffi::c_int;
        }
    }
    if offset == 0
        || !vim_isIDc(
            *line.offset((offset - 1 as ::core::ffi::c_int) as isize) as uint8_t
                as ::core::ffi::c_int,
        )
    {
        *poffset = offset;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_iswhileofdo_end(mut terminated: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut i: ::core::ffi::c_int = 0;
    if terminated != ';' as ::core::ffi::c_int {
        return false_0;
    }
    line = get_cursor_line_ptr();
    p = line;
    while *p as ::core::ffi::c_int != NUL {
        p = cin_skipcomment(p);
        if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
            s = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            if *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                i = p.offset_from(line) as ::core::ffi::c_int;
                (*curwin).w_cursor.col = i as colnr_T;
                trypos = find_match_paren((*curbuf).b_ind_maxparen);
                if !trypos.is_null() {
                    s = cin_skipcomment(ml_get((*trypos).lnum));
                    if *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
                    }
                    if cin_starts_with(s, b"while\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
                        (*curwin).w_cursor.lnum = (*trypos).lnum;
                        return true_0;
                    }
                }
                line = get_cursor_line_ptr();
                p = line.offset(i as isize);
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
    }
    return false_0;
}
unsafe extern "C" fn cin_isbreak(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (strncmp(
        p,
        b"break\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*p.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_is_cpp_baseclass(
    mut cached: *mut cpp_baseclass_cache_T,
) -> ::core::ffi::c_int {
    let mut pos: *mut lpos_T = &raw mut (*cached).lpos;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut class_or_struct: ::core::ffi::c_int = 0;
    let mut lookfor_ctor_init: ::core::ffi::c_int = 0;
    let mut cpp_base_class: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = (*curwin).w_cursor.lnum;
    let mut line: *const ::core::ffi::c_char = get_cursor_line_ptr();
    if (*pos).lnum <= lnum {
        return (*cached).found;
    }
    (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
    s = skipwhite(line);
    if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
        return false_0;
    }
    s = cin_skipcomment(s);
    if *s as ::core::ffi::c_int == NUL {
        return false_0;
    }
    class_or_struct = false_0;
    lookfor_ctor_init = class_or_struct;
    cpp_base_class = lookfor_ctor_init;
    while lnum > 1 as linenr_T {
        line = ml_get(lnum - 1 as linenr_T);
        s = skipwhite(line);
        if *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int || *s as ::core::ffi::c_int == NUL
        {
            break;
        }
        while *s as ::core::ffi::c_int != NUL {
            s = cin_skipcomment(s);
            if *s as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                break;
            }
            if *s as ::core::ffi::c_int != NUL {
                s = s.offset(1);
            }
        }
        if *s as ::core::ffi::c_int != NUL {
            break;
        }
        lnum -= 1;
    }
    (*pos).lnum = lnum;
    line = ml_get(lnum);
    s = line;
    loop {
        if *s as ::core::ffi::c_int == NUL {
            if lnum == (*curwin).w_cursor.lnum {
                break;
            }
            lnum += 1;
            line = ml_get(lnum);
            s = line;
        }
        if s == line {
            if cin_iscase(s, false_0 != 0) {
                break;
            }
            s = cin_skipcomment(line);
            if *s as ::core::ffi::c_int == NUL {
                continue;
            }
        }
        if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '"' as ::core::ffi::c_int
            || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'R' as ::core::ffi::c_int
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
        {
            s = skip_string(s).offset(1 as ::core::ffi::c_int as isize);
        } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
        {
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                lookfor_ctor_init = false_0;
                s = cin_skipcomment(s.offset(2 as ::core::ffi::c_int as isize));
            } else if lookfor_ctor_init != 0 || class_or_struct != 0 {
                cpp_base_class = true_0;
                class_or_struct = false_0;
                lookfor_ctor_init = class_or_struct;
                (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
                s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
            } else {
                s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
            }
        } else if strncmp(
            s,
            b"class\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            && !vim_isIDc(
                *s.offset(5 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            )
            || strncmp(
                s,
                b"struct\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
                && !vim_isIDc(
                    *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                )
        {
            class_or_struct = true_0;
            lookfor_ctor_init = false_0;
            if *s as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                s = cin_skipcomment(s.offset(5 as ::core::ffi::c_int as isize));
            } else {
                s = cin_skipcomment(s.offset(6 as ::core::ffi::c_int as isize));
            }
        } else {
            if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '{' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '}' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ';' as ::core::ffi::c_int
            {
                class_or_struct = false_0;
                lookfor_ctor_init = class_or_struct;
                cpp_base_class = lookfor_ctor_init;
            } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ')' as ::core::ffi::c_int
            {
                class_or_struct = false_0;
                lookfor_ctor_init = true_0;
            } else if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '?' as ::core::ffi::c_int
            {
                return false_0;
            } else if !vim_isIDc(
                *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ) {
                class_or_struct = false_0;
                lookfor_ctor_init = false_0;
            } else if (*pos).col == 0 as ::core::ffi::c_int {
                lookfor_ctor_init = false_0;
                if cpp_base_class != 0 {
                    (*pos).col = s.offset_from(line) as colnr_T;
                }
            }
            if lnum == (*curwin).w_cursor.lnum
                && *s as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                && cin_nocode(s.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                (*pos).col = 0 as ::core::ffi::c_int as colnr_T;
            }
            s = cin_skipcomment(s.offset(1 as ::core::ffi::c_int as isize));
        }
    }
    (*cached).found = cpp_base_class;
    if cpp_base_class != 0 {
        (*pos).lnum = lnum;
    }
    return cpp_base_class;
}
unsafe extern "C" fn get_baseclass_amount(mut col: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut amount: ::core::ffi::c_int = 0;
    let mut vcol: colnr_T = 0;
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if col == 0 as ::core::ffi::c_int {
        amount = get_indent();
        if find_last_paren(
            get_cursor_line_ptr(),
            '(' as ::core::ffi::c_char,
            ')' as ::core::ffi::c_char,
        ) != 0
            && {
                trypos = find_match_paren((*curbuf).b_ind_maxparen);
                !trypos.is_null()
            }
        {
            amount = get_indent_lnum((*trypos).lnum);
        }
        if cin_ends_in(
            get_cursor_line_ptr(),
            b",\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0
        {
            amount += (*curbuf).b_ind_cpp_baseclass;
        }
    } else {
        (*curwin).w_cursor.col = col as colnr_T;
        getvcol(
            curwin,
            &raw mut (*curwin).w_cursor,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        amount = vcol;
    }
    if amount < (*curbuf).b_ind_cpp_baseclass {
        amount = (*curbuf).b_ind_cpp_baseclass;
    }
    return amount;
}
unsafe extern "C" fn cin_ends_in(
    mut s: *const ::core::ffi::c_char,
    mut find: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = s;
    let mut r: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = strlen(find) as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL {
        p = cin_skipcomment(p);
        if strncmp(p, find, len as size_t) == 0 as ::core::ffi::c_int {
            r = skipwhite(p.offset(len as isize));
            if cin_nocode(r) != 0 {
                return true_0;
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            p = p.offset(1);
        }
    }
    return false_0;
}
unsafe extern "C" fn cin_starts_with(
    mut s: *const ::core::ffi::c_char,
    mut word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut l: size_t = strlen(word);
    return (strncmp(s, word, l) == 0 as ::core::ffi::c_int
        && !vim_isIDc(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int))
        as ::core::ffi::c_int;
}
unsafe extern "C" fn cin_is_cpp_extern_c(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut has_string_literal: ::core::ffi::c_int = false_0;
    s = cin_skipcomment(s);
    if strncmp(
        s,
        b"extern\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        && (*s.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !vim_iswordc(
                *s.offset(6 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ))
    {
        p = cin_skipcomment(skipwhite(s.offset(6 as ::core::ffi::c_int as isize)));
        while *p as ::core::ffi::c_int != NUL {
            if ascii_iswhite(*p as ::core::ffi::c_int) {
                p = cin_skipcomment(skipwhite(p));
            } else {
                if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                    break;
                }
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'C' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    if has_string_literal != 0 {
                        return false_0;
                    }
                    has_string_literal = true_0;
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'C' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '+' as ::core::ffi::c_int
                    && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '+' as ::core::ffi::c_int
                    && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                {
                    if has_string_literal != 0 {
                        return false_0;
                    }
                    has_string_literal = true_0;
                    p = p.offset(5 as ::core::ffi::c_int as isize);
                } else {
                    return false_0;
                }
            }
        }
        return if has_string_literal != 0 {
            true_0
        } else {
            false_0
        };
    }
    return false_0;
}
unsafe extern "C" fn cin_skip2pos(mut trypos: *mut pos_T) -> ::core::ffi::c_int {
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new_p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    line = ml_get((*trypos).lnum);
    p = line;
    while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < (*trypos).col {
        if cin_iscomment(p) != 0 {
            p = cin_skipcomment(p);
        } else {
            new_p = skip_string(p);
            if new_p == p {
                p = p.offset(1);
            } else {
                p = new_p;
            }
        }
    }
    return p.offset_from(line) as ::core::ffi::c_int;
}
unsafe extern "C" fn find_start_brace() -> *mut pos_T {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    static mut pos_copy: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    cursor_save = (*curwin).w_cursor;
    loop {
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            '{' as ::core::ffi::c_int,
            FM_BLOCKSTOP as ::core::ffi::c_int,
            0 as int64_t,
        );
        if trypos.is_null() {
            break;
        }
        pos_copy = *trypos;
        trypos = &raw mut pos_copy;
        (*curwin).w_cursor = *trypos;
        pos = ::core::ptr::null_mut::<pos_T>();
        if cin_skip2pos(trypos) == (*trypos).col && {
            pos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
            pos.is_null()
        } {
            break;
        }
        if !pos.is_null() {
            (*curwin).w_cursor = *pos;
        }
    }
    (*curwin).w_cursor = cursor_save;
    return trypos;
}
unsafe extern "C" fn find_match_paren(mut ind_maxparen: ::core::ffi::c_int) -> *mut pos_T {
    return find_match_char('(' as ::core::ffi::c_char, ind_maxparen);
}
unsafe extern "C" fn find_match_char(
    mut c: ::core::ffi::c_char,
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    let mut cursor_save: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    static mut pos_copy: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut ind_maxp_wk: ::core::ffi::c_int = 0;
    cursor_save = (*curwin).w_cursor;
    ind_maxp_wk = ind_maxparen;
    loop {
        trypos = findmatchlimit(
            ::core::ptr::null_mut::<oparg_T>(),
            c as uint8_t as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            ind_maxp_wk as int64_t,
        );
        if trypos.is_null() {
            break;
        }
        if cin_skip2pos(trypos) > (*trypos).col {
            ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos).lnum))
                as ::core::ffi::c_int;
            if ind_maxp_wk > 0 as ::core::ffi::c_int {
                (*curwin).w_cursor = *trypos;
                (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                trypos = ::core::ptr::null_mut::<pos_T>();
                break;
            }
        } else {
            let mut trypos_wk: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
            pos_copy = *trypos;
            trypos = &raw mut pos_copy;
            (*curwin).w_cursor = *trypos;
            trypos_wk = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
            if trypos_wk.is_null() {
                break;
            }
            ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos_wk).lnum))
                as ::core::ffi::c_int;
            if ind_maxp_wk > 0 as ::core::ffi::c_int {
                (*curwin).w_cursor = *trypos_wk;
            } else {
                trypos = ::core::ptr::null_mut::<pos_T>();
                break;
            }
        }
    }
    (*curwin).w_cursor = cursor_save;
    return trypos;
}
unsafe extern "C" fn find_match_paren_after_brace(
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    let mut trypos: *mut pos_T = find_match_paren(ind_maxparen);
    if trypos.is_null() {
        return ::core::ptr::null_mut::<pos_T>();
    }
    let mut tryposBrace: *mut pos_T = find_start_brace();
    if !tryposBrace.is_null()
        && (if (*trypos).lnum != (*tryposBrace).lnum {
            ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
        } else {
            ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
        }) != 0
    {
        trypos = ::core::ptr::null_mut::<pos_T>();
    }
    return trypos;
}
unsafe extern "C" fn corr_ind_maxparen(mut startpos: *mut pos_T) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int =
        (*startpos).lnum as ::core::ffi::c_int - (*curwin).w_cursor.lnum as ::core::ffi::c_int;
    if n > 0 as ::core::ffi::c_int && n < (*curbuf).b_ind_maxparen / 2 as ::core::ffi::c_int {
        return (*curbuf).b_ind_maxparen - n;
    }
    return (*curbuf).b_ind_maxparen;
}
unsafe extern "C" fn find_last_paren(
    mut l: *const ::core::ffi::c_char,
    mut start: ::core::ffi::c_char,
    mut end: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = false_0;
    let mut open_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    i = 0 as ::core::ffi::c_int;
    while *l.offset(i as isize) as ::core::ffi::c_int != NUL {
        i = cin_skipcomment(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
        i = skip_string(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
        if *l.offset(i as isize) as ::core::ffi::c_int == start as ::core::ffi::c_int {
            open_count += 1;
        } else if *l.offset(i as isize) as ::core::ffi::c_int == end as ::core::ffi::c_int {
            if open_count > 0 as ::core::ffi::c_int {
                open_count -= 1;
            } else {
                (*curwin).w_cursor.col = i as colnr_T;
                retval = true_0;
            }
        }
        i += 1;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn parse_cino(mut buf: *mut buf_T) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut l: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut divider: ::core::ffi::c_int = 0;
    let mut fraction: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sw: ::core::ffi::c_int = get_sw_value(buf);
    (*buf).b_ind_level = sw;
    (*buf).b_ind_open_imag = 0 as ::core::ffi::c_int;
    (*buf).b_ind_no_brace = 0 as ::core::ffi::c_int;
    (*buf).b_ind_first_open = 0 as ::core::ffi::c_int;
    (*buf).b_ind_open_extra = 0 as ::core::ffi::c_int;
    (*buf).b_ind_close_extra = 0 as ::core::ffi::c_int;
    (*buf).b_ind_open_left_imag = 0 as ::core::ffi::c_int;
    (*buf).b_ind_jump_label = -1 as ::core::ffi::c_int;
    (*buf).b_ind_case = sw;
    (*buf).b_ind_case_code = sw;
    (*buf).b_ind_case_break = 0 as ::core::ffi::c_int;
    (*buf).b_ind_scopedecl = sw;
    (*buf).b_ind_scopedecl_code = sw;
    (*buf).b_ind_param = sw;
    (*buf).b_ind_func_type = sw;
    (*buf).b_ind_cpp_baseclass = sw;
    (*buf).b_ind_continuation = sw;
    (*buf).b_ind_unclosed = sw * 2 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed2 = sw;
    (*buf).b_ind_unclosed_noignore = 0 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed_wrapped = 0 as ::core::ffi::c_int;
    (*buf).b_ind_unclosed_whiteok = 0 as ::core::ffi::c_int;
    (*buf).b_ind_matching_paren = 0 as ::core::ffi::c_int;
    (*buf).b_ind_paren_prev = 0 as ::core::ffi::c_int;
    (*buf).b_ind_comment = 0 as ::core::ffi::c_int;
    (*buf).b_ind_in_comment = 3 as ::core::ffi::c_int;
    (*buf).b_ind_in_comment2 = 0 as ::core::ffi::c_int;
    (*buf).b_ind_maxparen = 20 as ::core::ffi::c_int;
    (*buf).b_ind_maxcomment = 70 as ::core::ffi::c_int;
    (*buf).b_ind_java = 0 as ::core::ffi::c_int;
    (*buf).b_ind_js = 0 as ::core::ffi::c_int;
    (*buf).b_ind_keep_case_label = 0 as ::core::ffi::c_int;
    (*buf).b_ind_cpp_namespace = 0 as ::core::ffi::c_int;
    (*buf).b_ind_if_for_while = 0 as ::core::ffi::c_int;
    (*buf).b_ind_hash_comment = 0 as ::core::ffi::c_int;
    (*buf).b_ind_cpp_extern_c = 0 as ::core::ffi::c_int;
    (*buf).b_ind_pragma = 0 as ::core::ffi::c_int;
    p = (*buf).b_p_cino;
    while *p != 0 {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        l = c2rust_fresh0;
        if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
        }
        let mut digits_start: *mut ::core::ffi::c_char = p;
        let mut n: int64_t =
            getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as int64_t;
        divider = 0 as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            p = p.offset(1);
            fraction = atoi(p);
            while ascii_isdigit(*p as ::core::ffi::c_int) {
                p = p.offset(1);
                if divider != 0 {
                    divider *= 10 as ::core::ffi::c_int;
                } else {
                    divider = 10 as ::core::ffi::c_int;
                }
            }
        }
        if *p as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
            if p == digits_start {
                n = sw as int64_t;
            } else {
                n *= sw as int64_t;
                if divider != 0 {
                    n += (sw as int64_t * fraction as int64_t
                        + (divider / 2 as ::core::ffi::c_int) as int64_t)
                        / divider as int64_t;
                }
            }
            p = p.offset(1);
        }
        if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
        {
            n = -n;
        }
        n = trim_to_int(n) as int64_t;
        match *l as ::core::ffi::c_int {
            62 => {
                (*buf).b_ind_level = n as ::core::ffi::c_int;
            }
            101 => {
                (*buf).b_ind_open_imag = n as ::core::ffi::c_int;
            }
            110 => {
                (*buf).b_ind_no_brace = n as ::core::ffi::c_int;
            }
            102 => {
                (*buf).b_ind_first_open = n as ::core::ffi::c_int;
            }
            123 => {
                (*buf).b_ind_open_extra = n as ::core::ffi::c_int;
            }
            125 => {
                (*buf).b_ind_close_extra = n as ::core::ffi::c_int;
            }
            94 => {
                (*buf).b_ind_open_left_imag = n as ::core::ffi::c_int;
            }
            76 => {
                (*buf).b_ind_jump_label = n as ::core::ffi::c_int;
            }
            58 => {
                (*buf).b_ind_case = n as ::core::ffi::c_int;
            }
            61 => {
                (*buf).b_ind_case_code = n as ::core::ffi::c_int;
            }
            98 => {
                (*buf).b_ind_case_break = n as ::core::ffi::c_int;
            }
            112 => {
                (*buf).b_ind_param = n as ::core::ffi::c_int;
            }
            116 => {
                (*buf).b_ind_func_type = n as ::core::ffi::c_int;
            }
            47 => {
                (*buf).b_ind_comment = n as ::core::ffi::c_int;
            }
            99 => {
                (*buf).b_ind_in_comment = n as ::core::ffi::c_int;
            }
            67 => {
                (*buf).b_ind_in_comment2 = n as ::core::ffi::c_int;
            }
            105 => {
                (*buf).b_ind_cpp_baseclass = n as ::core::ffi::c_int;
            }
            43 => {
                (*buf).b_ind_continuation = n as ::core::ffi::c_int;
            }
            40 => {
                (*buf).b_ind_unclosed = n as ::core::ffi::c_int;
            }
            117 => {
                (*buf).b_ind_unclosed2 = n as ::core::ffi::c_int;
            }
            85 => {
                (*buf).b_ind_unclosed_noignore = n as ::core::ffi::c_int;
            }
            87 => {
                (*buf).b_ind_unclosed_wrapped = n as ::core::ffi::c_int;
            }
            119 => {
                (*buf).b_ind_unclosed_whiteok = n as ::core::ffi::c_int;
            }
            109 => {
                (*buf).b_ind_matching_paren = n as ::core::ffi::c_int;
            }
            77 => {
                (*buf).b_ind_paren_prev = n as ::core::ffi::c_int;
            }
            41 => {
                (*buf).b_ind_maxparen = n as ::core::ffi::c_int;
            }
            42 => {
                (*buf).b_ind_maxcomment = n as ::core::ffi::c_int;
            }
            103 => {
                (*buf).b_ind_scopedecl = n as ::core::ffi::c_int;
            }
            104 => {
                (*buf).b_ind_scopedecl_code = n as ::core::ffi::c_int;
            }
            106 => {
                (*buf).b_ind_java = n as ::core::ffi::c_int;
            }
            74 => {
                (*buf).b_ind_js = n as ::core::ffi::c_int;
            }
            108 => {
                (*buf).b_ind_keep_case_label = n as ::core::ffi::c_int;
            }
            35 => {
                (*buf).b_ind_hash_comment = n as ::core::ffi::c_int;
            }
            78 => {
                (*buf).b_ind_cpp_namespace = n as ::core::ffi::c_int;
            }
            107 => {
                (*buf).b_ind_if_for_while = n as ::core::ffi::c_int;
            }
            69 => {
                (*buf).b_ind_cpp_extern_c = n as ::core::ffi::c_int;
            }
            80 => {
                (*buf).b_ind_pragma = n as ::core::ffi::c_int;
            }
            _ => {}
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_c_indent() -> ::core::ffi::c_int {
    let mut cur_curpos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut amount: ::core::ffi::c_int = 0;
    let mut scope_amount: ::core::ffi::c_int = 0;
    let mut cur_amount: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
    let mut col: colnr_T = 0;
    let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linecopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut comment_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut tryposBrace: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut tryposCopy: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut our_paren_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut start_brace: ::core::ffi::c_int = 0;
    let mut ourscope: linenr_T = 0;
    let mut l: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut terminated: ::core::ffi::c_char = 0;
    let mut lookfor: ::core::ffi::c_int = 0;
    let mut whilelevel: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut lookfor_break: ::core::ffi::c_int = 0;
    let mut lookfor_cpp_namespace: bool = false_0 != 0;
    let mut cont_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut original_line_islabel: ::core::ffi::c_int = 0;
    let mut added_to_amount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut raw_string_start: linenr_T = 0 as linenr_T;
    let mut cache_cpp_baseclass: cpp_baseclass_cache_T = cpp_baseclass_cache_T {
        found: false_0,
        lpos: lpos_T {
            lnum: MAXLNUM as ::core::ffi::c_int as linenr_T,
            col: 0 as colnr_T,
        },
    };
    let mut ind_continuation: ::core::ffi::c_int = (*curbuf).b_ind_continuation;
    cur_curpos = (*curwin).w_cursor;
    if cur_curpos.lnum == 1 as linenr_T {
        return 0 as ::core::ffi::c_int;
    }
    linecopy = xstrdup(ml_get(cur_curpos.lnum));
    if State & MODE_INSERT as ::core::ffi::c_int != 0
        && (*curwin).w_cursor.col < strlen(linecopy) as colnr_T
        && *linecopy.offset((*curwin).w_cursor.col as isize) as ::core::ffi::c_int
            == ')' as ::core::ffi::c_int
    {
        *linecopy.offset((*curwin).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
    }
    theline = skipwhite(linecopy);
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    original_line_islabel = cin_islabel() as ::core::ffi::c_int;
    comment_pos = ind_find_start_comment();
    if !comment_pos.is_null() {
        tryposCopy = *comment_pos;
        comment_pos = &raw mut tryposCopy;
    }
    trypos = find_start_rawstring((*curbuf).b_ind_maxcomment);
    if !trypos.is_null()
        && (comment_pos.is_null() || lt(*trypos, *comment_pos) as ::core::ffi::c_int != 0)
    {
        amount = -1 as ::core::ffi::c_int;
    } else {
        '_theend: {
            if *theline as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                && (*linecopy as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                    || in_cinkeys(
                        '#' as ::core::ffi::c_int,
                        ' ' as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0)
            {
                let directive: *const ::core::ffi::c_char =
                    skipwhite(theline.offset(1 as ::core::ffi::c_int as isize));
                if (*curbuf).b_ind_pragma == 0 as ::core::ffi::c_int
                    || strncmp(
                        directive,
                        b"pragma\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) != 0 as ::core::ffi::c_int
                {
                    amount = (*curbuf).b_ind_hash_comment;
                    break '_theend;
                }
            }
            if original_line_islabel != 0
                && (*curbuf).b_ind_js == 0
                && (*curbuf).b_ind_jump_label < 0 as ::core::ffi::c_int
            {
                amount = 0 as ::core::ffi::c_int;
            } else {
                if cin_islinecomment(theline) != 0 {
                    let mut linecomment_pos: pos_T = pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    };
                    trypos = find_line_comment();
                    if trypos.is_null() && (*curwin).w_cursor.lnum > 1 as linenr_T {
                        linecomment_pos.col =
                            check_linecomment(ml_get((*curwin).w_cursor.lnum - 1 as linenr_T))
                                as colnr_T;
                        if linecomment_pos.col != MAXCOL as ::core::ffi::c_int {
                            trypos = &raw mut linecomment_pos;
                            (*trypos).lnum = (*curwin).w_cursor.lnum - 1 as linenr_T;
                        }
                    }
                    if !trypos.is_null() {
                        getvcol(
                            curwin,
                            trypos,
                            &raw mut col,
                            ::core::ptr::null_mut::<colnr_T>(),
                            ::core::ptr::null_mut::<colnr_T>(),
                        );
                        amount = col as ::core::ffi::c_int;
                        break '_theend;
                    }
                }
                if cin_iscomment(theline) == 0 && !comment_pos.is_null() {
                    let mut lead_start_len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                    let mut lead_middle_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut lead_start: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_middle: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
                    let mut lead_end_len: ::core::ffi::c_int = 0;
                    let mut p: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut start_align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut start_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut done: ::core::ffi::c_int = false_0;
                    getvcol(
                        curwin,
                        comment_pos,
                        &raw mut col,
                        ::core::ptr::null_mut::<colnr_T>(),
                        ::core::ptr::null_mut::<colnr_T>(),
                    );
                    amount = col as ::core::ffi::c_int;
                    *(&raw mut lead_start as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
                    *(&raw mut lead_middle as *mut ::core::ffi::c_char) =
                        NUL as ::core::ffi::c_char;
                    p = (*curbuf).b_p_com;
                    while *p as ::core::ffi::c_int != NUL {
                        let mut align: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                        {
                            if *p as ::core::ffi::c_int == COM_START
                                || *p as ::core::ffi::c_int == COM_END
                                || *p as ::core::ffi::c_int == COM_MIDDLE
                            {
                                let c2rust_fresh1 = p;
                                p = p.offset(1);
                                what = *c2rust_fresh1 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                            } else if *p as ::core::ffi::c_int == COM_LEFT
                                || *p as ::core::ffi::c_int == COM_RIGHT
                            {
                                let c2rust_fresh2 = p;
                                p = p.offset(1);
                                align =
                                    *c2rust_fresh2 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                            } else if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                != 0
                                || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                            {
                                off =
                                    getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
                            } else {
                                p = p.offset(1);
                            }
                        }
                        if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                            p = p.offset(1);
                        }
                        lead_end_len = copy_option_part(
                            &raw mut p,
                            &raw mut lead_end as *mut ::core::ffi::c_char,
                            COM_MAX_LEN as size_t,
                            b",\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                        if what == COM_START {
                            strcpy(
                                &raw mut lead_start as *mut ::core::ffi::c_char,
                                &raw mut lead_end as *mut ::core::ffi::c_char,
                            );
                            lead_start_len = lead_end_len;
                            start_off = off;
                            start_align = align;
                        } else if what == COM_MIDDLE {
                            strcpy(
                                &raw mut lead_middle as *mut ::core::ffi::c_char,
                                &raw mut lead_end as *mut ::core::ffi::c_char,
                            );
                            lead_middle_len = lead_end_len;
                        } else {
                            if what != COM_END {
                                continue;
                            }
                            if strncmp(
                                theline,
                                &raw mut lead_middle as *mut ::core::ffi::c_char,
                                lead_middle_len as size_t,
                            ) == 0 as ::core::ffi::c_int
                                && strncmp(
                                    theline,
                                    &raw mut lead_end as *mut ::core::ffi::c_char,
                                    lead_end_len as size_t,
                                ) != 0 as ::core::ffi::c_int
                            {
                                done = true_0;
                                if (*curwin).w_cursor.lnum > 1 as linenr_T {
                                    look =
                                        skipwhite(ml_get((*curwin).w_cursor.lnum - 1 as linenr_T));
                                    if strncmp(
                                        look,
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                        lead_start_len as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        amount = get_indent_lnum(
                                            (*curwin).w_cursor.lnum - 1 as linenr_T,
                                        );
                                    } else if strncmp(
                                        look,
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                        lead_middle_len as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        amount = get_indent_lnum(
                                            (*curwin).w_cursor.lnum - 1 as linenr_T,
                                        );
                                        break;
                                    } else if strncmp(
                                        ml_get((*comment_pos).lnum)
                                            .offset((*comment_pos).col as isize),
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                        lead_start_len as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        continue;
                                    }
                                }
                                if start_off != 0 as ::core::ffi::c_int {
                                    amount += start_off;
                                } else if start_align == COM_RIGHT {
                                    amount += vim_strsize(
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                    ) - vim_strsize(
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    );
                                }
                                break;
                            } else {
                                if !(strncmp(
                                    theline,
                                    &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    lead_middle_len as size_t,
                                ) != 0 as ::core::ffi::c_int
                                    && strncmp(
                                        theline,
                                        &raw mut lead_end as *mut ::core::ffi::c_char,
                                        lead_end_len as size_t,
                                    ) == 0 as ::core::ffi::c_int)
                                {
                                    continue;
                                }
                                amount = get_indent_lnum((*curwin).w_cursor.lnum - 1 as linenr_T);
                                if off != 0 as ::core::ffi::c_int {
                                    amount += off;
                                } else if align == COM_RIGHT {
                                    amount += vim_strsize(
                                        &raw mut lead_start as *mut ::core::ffi::c_char,
                                    ) - vim_strsize(
                                        &raw mut lead_middle as *mut ::core::ffi::c_char,
                                    );
                                }
                                done = true_0;
                                break;
                            }
                        }
                    }
                    if done == 0 {
                        if *theline.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                        {
                            amount += 1 as ::core::ffi::c_int;
                        } else {
                            amount = -1 as ::core::ffi::c_int;
                            lnum = cur_curpos.lnum - 1 as linenr_T;
                            while lnum > (*comment_pos).lnum {
                                if linewhite(lnum) {
                                    lnum -= 1;
                                } else {
                                    amount = get_indent_lnum(lnum);
                                    break;
                                }
                            }
                            if amount == -1 as ::core::ffi::c_int {
                                if (*curbuf).b_ind_in_comment2 == 0 {
                                    start = ml_get((*comment_pos).lnum);
                                    look = start
                                        .offset((*comment_pos).col as isize)
                                        .offset(2 as ::core::ffi::c_int as isize);
                                    if *look as ::core::ffi::c_int != NUL {
                                        (*comment_pos).col =
                                            skipwhite(look).offset_from(start) as colnr_T;
                                    }
                                }
                                getvcol(
                                    curwin,
                                    comment_pos,
                                    &raw mut col,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                                amount = col as ::core::ffi::c_int;
                                if (*curbuf).b_ind_in_comment2 != 0
                                    || *look as ::core::ffi::c_int == NUL
                                {
                                    amount += (*curbuf).b_ind_in_comment;
                                }
                            }
                        }
                    }
                } else if *skipwhite(theline) as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                    && {
                        trypos =
                            find_match_char('[' as ::core::ffi::c_char, (*curbuf).b_ind_maxparen);
                        !trypos.is_null()
                    }
                {
                    amount = get_indent_lnum((*trypos).lnum);
                } else {
                    trypos = find_match_paren((*curbuf).b_ind_maxparen);
                    if !trypos.is_null() && (*curbuf).b_ind_java == 0 as ::core::ffi::c_int
                        || {
                            tryposBrace = find_start_brace();
                            !tryposBrace.is_null()
                        }
                        || !trypos.is_null()
                    {
                        if !trypos.is_null() && !tryposBrace.is_null() {
                            if if (*trypos).lnum != (*tryposBrace).lnum {
                                ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
                            } else {
                                ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
                            } != 0
                            {
                                trypos = ::core::ptr::null_mut::<pos_T>();
                            } else {
                                tryposBrace = ::core::ptr::null_mut::<pos_T>();
                            }
                        }
                        if !trypos.is_null() {
                            our_paren_pos = *trypos;
                            if *theline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ')' as ::core::ffi::c_int
                                && (*curbuf).b_ind_paren_prev != 0
                            {
                                amount = get_indent_lnum((*curwin).w_cursor.lnum - 1 as linenr_T);
                            } else {
                                amount = -1 as ::core::ffi::c_int;
                                lnum = cur_curpos.lnum - 1 as linenr_T;
                                while lnum > our_paren_pos.lnum {
                                    l = skipwhite(ml_get(lnum));
                                    if cin_nocode(l) == 0 {
                                        if cin_ispreproc_cont(
                                            &raw mut l,
                                            &raw mut lnum,
                                            &raw mut amount,
                                        ) == 0
                                        {
                                            (*curwin).w_cursor.lnum = lnum;
                                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<
                                                linenr_T,
                                            >(
                                            ));
                                            if !trypos.is_null() {
                                                lnum = (*trypos).lnum + 1 as linenr_T;
                                            } else {
                                                trypos = find_match_paren(corr_ind_maxparen(
                                                    &raw mut cur_curpos,
                                                ));
                                                if !trypos.is_null()
                                                    && (*trypos).lnum == our_paren_pos.lnum
                                                    && (*trypos).col == our_paren_pos.col
                                                {
                                                    amount = get_indent_lnum(lnum);
                                                    if *theline
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == ')' as ::core::ffi::c_int
                                                    {
                                                        if our_paren_pos.lnum != lnum
                                                            && cur_amount > amount
                                                        {
                                                            cur_amount = amount;
                                                        }
                                                        amount = -1 as ::core::ffi::c_int;
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    lnum -= 1;
                                }
                            }
                            if amount == -1 as ::core::ffi::c_int {
                                let mut ignore_paren_col: ::core::ffi::c_int =
                                    0 as ::core::ffi::c_int;
                                let mut is_if_for_while: ::core::ffi::c_int =
                                    0 as ::core::ffi::c_int;
                                if (*curbuf).b_ind_if_for_while != 0 {
                                    let mut cursor_save: pos_T = (*curwin).w_cursor;
                                    let mut outermost: pos_T = pos_T {
                                        lnum: 0,
                                        col: 0,
                                        coladd: 0,
                                    };
                                    let mut line: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    trypos = &raw mut our_paren_pos;
                                    loop {
                                        outermost = *trypos;
                                        (*curwin).w_cursor.lnum = outermost.lnum;
                                        (*curwin).w_cursor.col = outermost.col;
                                        trypos = find_match_paren((*curbuf).b_ind_maxparen);
                                        if !(!trypos.is_null() && (*trypos).lnum == outermost.lnum)
                                        {
                                            break;
                                        }
                                    }
                                    (*curwin).w_cursor = cursor_save;
                                    line = ml_get(outermost.lnum);
                                    is_if_for_while = cin_is_if_for_while_before_offset(
                                        line,
                                        &raw mut outermost.col,
                                    );
                                }
                                amount = skip_label(our_paren_pos.lnum, &raw mut look);
                                look = skipwhite(look);
                                if *look as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                                    let mut save_lnum: linenr_T = (*curwin).w_cursor.lnum;
                                    let mut line_0: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    let mut look_col: ::core::ffi::c_int = 0;
                                    (*curwin).w_cursor.lnum = our_paren_pos.lnum;
                                    line_0 = get_cursor_line_ptr();
                                    look_col = look.offset_from(line_0) as ::core::ffi::c_int;
                                    (*curwin).w_cursor.col =
                                        (look_col + 1 as ::core::ffi::c_int) as colnr_T;
                                    trypos = findmatchlimit(
                                        ::core::ptr::null_mut::<oparg_T>(),
                                        ')' as ::core::ffi::c_int,
                                        0 as ::core::ffi::c_int,
                                        (*curbuf).b_ind_maxparen as int64_t,
                                    );
                                    if !trypos.is_null()
                                        && (*trypos).lnum == our_paren_pos.lnum
                                        && (*trypos).col < our_paren_pos.col
                                    {
                                        ignore_paren_col = (*trypos).col as ::core::ffi::c_int
                                            + 1 as ::core::ffi::c_int;
                                    }
                                    (*curwin).w_cursor.lnum = save_lnum;
                                    look = ml_get(our_paren_pos.lnum).offset(look_col as isize);
                                }
                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ')' as ::core::ffi::c_int
                                    || (*curbuf).b_ind_unclosed == 0 as ::core::ffi::c_int
                                        && is_if_for_while == 0 as ::core::ffi::c_int
                                    || (*curbuf).b_ind_unclosed_noignore == 0
                                        && *look as ::core::ffi::c_int == '(' as ::core::ffi::c_int
                                        && ignore_paren_col == 0 as ::core::ffi::c_int
                                {
                                    if *theline.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        != ')' as ::core::ffi::c_int
                                    {
                                        cur_amount = MAXCOL as ::core::ffi::c_int;
                                        l = ml_get(our_paren_pos.lnum);
                                        if (*curbuf).b_ind_unclosed_wrapped != 0
                                            && cin_ends_in(
                                                l,
                                                b"(\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) != 0
                                        {
                                            n = 1 as ::core::ffi::c_int;
                                            col = 0 as ::core::ffi::c_int as colnr_T;
                                            while col < our_paren_pos.col {
                                                match *l.offset(col as isize) as ::core::ffi::c_int
                                                {
                                                    40 | 123 => {
                                                        n += 1;
                                                    }
                                                    41 | 125 => {
                                                        if n > 1 as ::core::ffi::c_int {
                                                            n -= 1;
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                col += 1;
                                            }
                                            our_paren_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                                            amount += n * (*curbuf).b_ind_unclosed_wrapped;
                                        } else if (*curbuf).b_ind_unclosed_whiteok != 0 {
                                            our_paren_pos.col += 1;
                                        } else {
                                            col = (our_paren_pos.col as ::core::ffi::c_int
                                                + 1 as ::core::ffi::c_int)
                                                as colnr_T;
                                            while ascii_iswhite(
                                                *l.offset(col as isize) as ::core::ffi::c_int
                                            ) {
                                                col += 1;
                                            }
                                            if *l.offset(col as isize) as ::core::ffi::c_int != NUL
                                            {
                                                our_paren_pos.col = col;
                                            } else {
                                                our_paren_pos.col += 1;
                                            }
                                        }
                                    }
                                    if our_paren_pos.col > 0 as ::core::ffi::c_int {
                                        getvcol(
                                            curwin,
                                            &raw mut our_paren_pos,
                                            &raw mut col,
                                            ::core::ptr::null_mut::<colnr_T>(),
                                            ::core::ptr::null_mut::<colnr_T>(),
                                        );
                                        if cur_amount > col {
                                            cur_amount = col as ::core::ffi::c_int;
                                        }
                                    }
                                }
                                if !(*theline.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ')' as ::core::ffi::c_int
                                    && (*curbuf).b_ind_matching_paren != 0)
                                {
                                    if (*curbuf).b_ind_unclosed == 0 as ::core::ffi::c_int
                                        && is_if_for_while == 0 as ::core::ffi::c_int
                                        || (*curbuf).b_ind_unclosed_noignore == 0
                                            && *look as ::core::ffi::c_int
                                                == '(' as ::core::ffi::c_int
                                            && ignore_paren_col == 0 as ::core::ffi::c_int
                                    {
                                        if cur_amount != MAXCOL as ::core::ffi::c_int {
                                            amount = cur_amount;
                                        }
                                    } else {
                                        col = our_paren_pos.col;
                                        while our_paren_pos.col > ignore_paren_col {
                                            our_paren_pos.col -= 1;
                                            match *ml_get_pos(&raw mut our_paren_pos)
                                                as ::core::ffi::c_int
                                            {
                                                40 => {
                                                    amount += (*curbuf).b_ind_unclosed2;
                                                    col = our_paren_pos.col;
                                                }
                                                41 => {
                                                    amount -= (*curbuf).b_ind_unclosed2;
                                                    col = MAXCOL as ::core::ffi::c_int as colnr_T;
                                                }
                                                _ => {}
                                            }
                                        }
                                        if col == MAXCOL as ::core::ffi::c_int {
                                            amount += (*curbuf).b_ind_unclosed;
                                        } else {
                                            (*curwin).w_cursor.lnum = our_paren_pos.lnum;
                                            (*curwin).w_cursor.col = col;
                                            if !find_match_paren_after_brace(
                                                (*curbuf).b_ind_maxparen,
                                            )
                                            .is_null()
                                            {
                                                amount += (*curbuf).b_ind_unclosed2;
                                            } else if is_if_for_while != 0 {
                                                amount += (*curbuf).b_ind_if_for_while;
                                            } else {
                                                amount += (*curbuf).b_ind_unclosed;
                                            }
                                        }
                                        if cur_amount < amount {
                                            amount = cur_amount;
                                        }
                                    }
                                }
                            }
                            if cin_iscomment(theline) != 0 {
                                amount += (*curbuf).b_ind_comment;
                            }
                        } else {
                            tryposCopy = *tryposBrace;
                            tryposBrace = &raw mut tryposCopy;
                            trypos = tryposBrace;
                            ourscope = (*trypos).lnum;
                            start = ml_get(ourscope);
                            look = skipwhite(start);
                            if *look as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                getvcol(
                                    curwin,
                                    trypos,
                                    &raw mut col,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                                amount = col as ::core::ffi::c_int;
                                if *start as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                                    start_brace = BRACE_IN_COL0;
                                } else {
                                    start_brace = BRACE_AT_START;
                                }
                            } else {
                                (*curwin).w_cursor.lnum = ourscope;
                                lnum = ourscope;
                                if find_last_paren(
                                    start,
                                    '(' as ::core::ffi::c_char,
                                    ')' as ::core::ffi::c_char,
                                ) != 0
                                    && {
                                        trypos = find_match_paren((*curbuf).b_ind_maxparen);
                                        !trypos.is_null()
                                    }
                                {
                                    lnum = (*trypos).lnum;
                                }
                                if ((*curbuf).b_ind_js != 0 || (*curbuf).b_ind_keep_case_label != 0)
                                    && cin_iscase(skipwhite(get_cursor_line_ptr()), false_0 != 0)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    amount = get_indent();
                                } else if (*curbuf).b_ind_js != 0 {
                                    amount = get_indent_lnum(lnum);
                                } else {
                                    amount = skip_label(lnum, &raw mut l);
                                }
                                start_brace = BRACE_AT_END;
                            }
                            let mut js_cur_has_key: bool = if (*curbuf).b_ind_js != 0 {
                                cin_has_js_key(theline) as ::core::ffi::c_int
                            } else {
                                false_0
                            } != 0;
                            if *theline.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '}' as ::core::ffi::c_int
                            {
                                amount += (*curbuf).b_ind_close_extra;
                            } else {
                                lookfor = LOOKFOR_INITIAL;
                                if cin_iselse(theline) != 0 {
                                    lookfor = LOOKFOR_IF;
                                } else if cin_iswhileofdo(theline, cur_curpos.lnum) != 0 {
                                    lookfor = LOOKFOR_DO;
                                }
                                if lookfor != LOOKFOR_INITIAL {
                                    (*curwin).w_cursor.lnum = cur_curpos.lnum;
                                    if find_match(lookfor, ourscope) == OK {
                                        amount = get_indent();
                                        break '_theend;
                                    }
                                }
                                if start_brace == BRACE_IN_COL0 {
                                    amount = (*curbuf).b_ind_open_left_imag;
                                    lookfor_cpp_namespace = true_0 != 0;
                                } else if start_brace == BRACE_AT_START
                                    && lookfor_cpp_namespace as ::core::ffi::c_int != 0
                                {
                                    lookfor_cpp_namespace = true_0 != 0;
                                } else if start_brace == BRACE_AT_END {
                                    amount += (*curbuf).b_ind_open_imag;
                                    l = skipwhite(get_cursor_line_ptr());
                                    if cin_is_cpp_namespace(l) {
                                        amount += (*curbuf).b_ind_cpp_namespace;
                                    } else if cin_is_cpp_extern_c(l) != 0 {
                                        amount += (*curbuf).b_ind_cpp_extern_c;
                                    }
                                } else {
                                    amount -= (*curbuf).b_ind_open_extra;
                                    if amount < 0 as ::core::ffi::c_int {
                                        amount = 0 as ::core::ffi::c_int;
                                    }
                                }
                                lookfor_break = false_0;
                                if cin_iscase(theline, false_0 != 0) {
                                    lookfor = LOOKFOR_CASE;
                                    amount += (*curbuf).b_ind_case;
                                } else if cin_isscopedecl(theline) {
                                    lookfor = LOOKFOR_SCOPEDECL;
                                    amount += (*curbuf).b_ind_scopedecl;
                                } else {
                                    if (*curbuf).b_ind_case_break != 0 && cin_isbreak(theline) != 0
                                    {
                                        lookfor_break = true_0;
                                    }
                                    lookfor = LOOKFOR_INITIAL;
                                    amount += (*curbuf).b_ind_level;
                                }
                                scope_amount = amount;
                                whilelevel = 0 as ::core::ffi::c_int;
                                (*curwin).w_cursor = cur_curpos;
                                's_2927: loop {
                                    (*curwin).w_cursor.lnum -= 1;
                                    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                    if (*curwin).w_cursor.lnum <= ourscope {
                                        if lookfor == LOOKFOR_ENUM_OR_INIT {
                                            if (*curwin).w_cursor.lnum == 0 as linenr_T
                                                || (*curwin).w_cursor.lnum
                                                    < ourscope
                                                        - (*curbuf).b_ind_maxparen as linenr_T
                                            {
                                                if cont_amount > 0 as ::core::ffi::c_int {
                                                    amount = cont_amount;
                                                } else if (*curbuf).b_ind_js == 0 {
                                                    amount += ind_continuation;
                                                }
                                                break;
                                            } else {
                                                trypos =
                                                    ind_find_start_CORS(::core::ptr::null_mut::<
                                                        linenr_T,
                                                    >(
                                                    ));
                                                if !trypos.is_null() {
                                                    (*curwin).w_cursor.lnum =
                                                        (*trypos).lnum + 1 as linenr_T;
                                                    (*curwin).w_cursor.col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                } else {
                                                    l = get_cursor_line_ptr();
                                                    if cin_ispreproc_cont(
                                                        &raw mut l,
                                                        &raw mut (*curwin).w_cursor.lnum,
                                                        &raw mut amount,
                                                    ) != 0
                                                    {
                                                        continue;
                                                    }
                                                    if cin_nocode(l) != 0 {
                                                        continue;
                                                    }
                                                    terminated =
                                                        cin_isterminated(l, false_0, true_0);
                                                    if start_brace != BRACE_IN_COL0
                                                        || cin_isfuncdecl(
                                                            &raw mut l,
                                                            (*curwin).w_cursor.lnum,
                                                            0 as linenr_T,
                                                        ) == 0
                                                    {
                                                        if terminated as ::core::ffi::c_int
                                                            == ',' as ::core::ffi::c_int
                                                        {
                                                            break;
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            != ';' as ::core::ffi::c_int
                                                            && cin_isinit() as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            break;
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            == 0 as ::core::ffi::c_int
                                                            || terminated as ::core::ffi::c_int
                                                                == '{' as ::core::ffi::c_int
                                                        {
                                                            continue;
                                                        }
                                                    }
                                                    if terminated as ::core::ffi::c_int
                                                        != ';' as ::core::ffi::c_int
                                                    {
                                                        trypos = ::core::ptr::null_mut::<pos_T>();
                                                        if find_last_paren(
                                                            l,
                                                            '(' as ::core::ffi::c_char,
                                                            ')' as ::core::ffi::c_char,
                                                        ) != 0
                                                        {
                                                            trypos = find_match_paren(
                                                                (*curbuf).b_ind_maxparen,
                                                            );
                                                        }
                                                        if trypos.is_null()
                                                            && find_last_paren(
                                                                l,
                                                                '{' as ::core::ffi::c_char,
                                                                '}' as ::core::ffi::c_char,
                                                            ) != 0
                                                        {
                                                            trypos = find_start_brace();
                                                        }
                                                        if !trypos.is_null() {
                                                            (*curwin).w_cursor.lnum =
                                                                (*trypos).lnum + 1 as linenr_T;
                                                            (*curwin).w_cursor.col =
                                                                0 as ::core::ffi::c_int as colnr_T;
                                                            continue;
                                                        }
                                                    }
                                                    if cont_amount > 0 as ::core::ffi::c_int {
                                                        amount = cont_amount;
                                                    } else {
                                                        amount += ind_continuation;
                                                    }
                                                    break;
                                                }
                                            }
                                        } else if lookfor == LOOKFOR_UNTERM {
                                            if cont_amount > 0 as ::core::ffi::c_int {
                                                amount = cont_amount;
                                            } else {
                                                amount += ind_continuation;
                                            }
                                            break;
                                        } else {
                                            if lookfor != LOOKFOR_TERM
                                                && lookfor != LOOKFOR_CPP_BASECLASS
                                                && lookfor != LOOKFOR_COMMA
                                            {
                                                amount = scope_amount;
                                                if *theline.offset(0 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == '{' as ::core::ffi::c_int
                                                {
                                                    amount += (*curbuf).b_ind_open_extra;
                                                    added_to_amount = (*curbuf).b_ind_open_extra;
                                                }
                                            }
                                            if !lookfor_cpp_namespace {
                                                break;
                                            }
                                            if (*curwin).w_cursor.lnum == ourscope {
                                                continue;
                                            }
                                            if (*curwin).w_cursor.lnum == 0 as linenr_T
                                                || (*curwin).w_cursor.lnum
                                                    < ourscope - FIND_NAMESPACE_LIM as linenr_T
                                            {
                                                break;
                                            }
                                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<
                                                linenr_T,
                                            >(
                                            ));
                                            if !trypos.is_null() {
                                                (*curwin).w_cursor.lnum =
                                                    (*trypos).lnum + 1 as linenr_T;
                                                (*curwin).w_cursor.col =
                                                    0 as ::core::ffi::c_int as colnr_T;
                                            } else {
                                                l = get_cursor_line_ptr();
                                                if cin_ispreproc_cont(
                                                    &raw mut l,
                                                    &raw mut (*curwin).w_cursor.lnum,
                                                    &raw mut amount,
                                                ) != 0
                                                {
                                                    continue;
                                                }
                                                if cin_is_cpp_namespace(l) {
                                                    amount += (*curbuf).b_ind_cpp_namespace
                                                        - added_to_amount;
                                                    break;
                                                } else if cin_is_cpp_extern_c(l) != 0 {
                                                    amount += (*curbuf).b_ind_cpp_extern_c
                                                        - added_to_amount;
                                                    break;
                                                } else if cin_nocode(l) == 0 {
                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        trypos = ind_find_start_CORS(&raw mut raw_string_start);
                                        if !trypos.is_null() {
                                            (*curwin).w_cursor.lnum =
                                                (*trypos).lnum + 1 as linenr_T;
                                            (*curwin).w_cursor.col =
                                                0 as ::core::ffi::c_int as colnr_T;
                                        } else {
                                            l = get_cursor_line_ptr();
                                            let mut iscase: bool = cin_iscase(l, false_0 != 0);
                                            if iscase as ::core::ffi::c_int != 0
                                                || cin_isscopedecl(l) as ::core::ffi::c_int != 0
                                            {
                                                if lookfor == LOOKFOR_CPP_BASECLASS {
                                                    break;
                                                }
                                                if whilelevel > 0 as ::core::ffi::c_int {
                                                    continue;
                                                }
                                                if lookfor == LOOKFOR_UNTERM
                                                    || lookfor == LOOKFOR_ENUM_OR_INIT
                                                {
                                                    if cont_amount > 0 as ::core::ffi::c_int {
                                                        amount = cont_amount;
                                                    } else {
                                                        amount += ind_continuation;
                                                    }
                                                    break;
                                                } else if iscase as ::core::ffi::c_int != 0
                                                    && lookfor == LOOKFOR_CASE
                                                    || iscase as ::core::ffi::c_int != 0
                                                        && lookfor_break != 0
                                                    || !iscase && lookfor == LOOKFOR_SCOPEDECL
                                                {
                                                    trypos = find_start_brace();
                                                    if !(trypos.is_null()
                                                        || (*trypos).lnum == ourscope)
                                                    {
                                                        continue;
                                                    }
                                                    amount = get_indent();
                                                    break;
                                                } else {
                                                    n = get_indent_nolabel((*curwin).w_cursor.lnum);
                                                    if lookfor == LOOKFOR_TERM {
                                                        if n != 0 {
                                                            amount = n;
                                                        }
                                                        if lookfor_break == 0 {
                                                            break;
                                                        }
                                                    }
                                                    if n != 0 {
                                                        amount = n;
                                                        l = after_label(get_cursor_line_ptr());
                                                        if !l.is_null()
                                                            && cin_is_cinword(l)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            if *theline.offset(
                                                                0 as ::core::ffi::c_int as isize,
                                                            )
                                                                as ::core::ffi::c_int
                                                                == '{' as ::core::ffi::c_int
                                                            {
                                                                amount +=
                                                                    (*curbuf).b_ind_open_extra;
                                                            } else {
                                                                amount += (*curbuf).b_ind_level
                                                                    + (*curbuf).b_ind_no_brace;
                                                            }
                                                        }
                                                        break;
                                                    } else {
                                                        scope_amount = get_indent()
                                                            + (if iscase as ::core::ffi::c_int != 0
                                                            {
                                                                (*curbuf).b_ind_case_code
                                                            } else {
                                                                (*curbuf).b_ind_scopedecl_code
                                                            });
                                                        lookfor = if (*curbuf).b_ind_case_break != 0
                                                        {
                                                            LOOKFOR_NOBREAK
                                                        } else {
                                                            LOOKFOR_ANY
                                                        };
                                                    }
                                                }
                                            } else if lookfor == LOOKFOR_CASE
                                                || lookfor == LOOKFOR_SCOPEDECL
                                            {
                                                if find_last_paren(
                                                    l,
                                                    '{' as ::core::ffi::c_char,
                                                    '}' as ::core::ffi::c_char,
                                                ) != 0
                                                    && {
                                                        trypos = find_start_brace();
                                                        !trypos.is_null()
                                                    }
                                                {
                                                    (*curwin).w_cursor.lnum =
                                                        (*trypos).lnum + 1 as linenr_T;
                                                    (*curwin).w_cursor.col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                }
                                            } else {
                                                if (*curbuf).b_ind_js == 0
                                                    && cin_islabel() as ::core::ffi::c_int != 0
                                                {
                                                    l = after_label(get_cursor_line_ptr());
                                                    if l.is_null() || cin_nocode(l) != 0 {
                                                        continue;
                                                    }
                                                }
                                                l = get_cursor_line_ptr();
                                                if cin_ispreproc_cont(
                                                    &raw mut l,
                                                    &raw mut (*curwin).w_cursor.lnum,
                                                    &raw mut amount,
                                                ) != 0
                                                    || cin_nocode(l) != 0
                                                {
                                                    continue;
                                                }
                                                n = 0 as ::core::ffi::c_int;
                                                if lookfor != LOOKFOR_TERM
                                                    && (*curbuf).b_ind_cpp_baseclass
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    n = cin_is_cpp_baseclass(
                                                        &raw mut cache_cpp_baseclass,
                                                    );
                                                    l = get_cursor_line_ptr();
                                                }
                                                if n != 0 {
                                                    if lookfor == LOOKFOR_UNTERM {
                                                        if cont_amount > 0 as ::core::ffi::c_int {
                                                            amount = cont_amount;
                                                        } else {
                                                            amount += ind_continuation;
                                                        }
                                                        break;
                                                    } else if *theline
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '{' as ::core::ffi::c_int
                                                    {
                                                        lookfor = LOOKFOR_UNTERM;
                                                        ind_continuation = 0 as ::core::ffi::c_int;
                                                    } else {
                                                        amount = get_baseclass_amount(
                                                            cache_cpp_baseclass.lpos.col
                                                                as ::core::ffi::c_int,
                                                        );
                                                        break;
                                                    }
                                                } else if lookfor == LOOKFOR_CPP_BASECLASS {
                                                    if cin_isterminated(l, true_0, false_0) != 0 {
                                                        break;
                                                    }
                                                } else {
                                                    terminated =
                                                        cin_isterminated(l, false_0, true_0);
                                                    if js_cur_has_key {
                                                        js_cur_has_key = false_0 != 0;
                                                        if (*curbuf).b_ind_js != 0
                                                            && terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                        {
                                                            lookfor = LOOKFOR_JS_KEY;
                                                        }
                                                    }
                                                    if lookfor == LOOKFOR_JS_KEY
                                                        && cin_has_js_key(l) as ::core::ffi::c_int
                                                            != 0
                                                    {
                                                        amount = get_indent();
                                                        break;
                                                    } else {
                                                        if lookfor == LOOKFOR_COMMA {
                                                            if !tryposBrace.is_null()
                                                                && (*tryposBrace).lnum
                                                                    >= (*curwin).w_cursor.lnum
                                                            {
                                                                break;
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                            {
                                                                break;
                                                            } else {
                                                                amount = get_indent();
                                                                if (*curwin).w_cursor.lnum
                                                                    - 1 as linenr_T
                                                                    == ourscope
                                                                {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        if terminated as ::core::ffi::c_int
                                                            == 0 as ::core::ffi::c_int
                                                            || lookfor != LOOKFOR_UNTERM
                                                                && terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                        {
                                                            if lookfor != LOOKFOR_ENUM_OR_INIT
                                                                && (*skipwhite(l)
                                                                    as ::core::ffi::c_int
                                                                    == '[' as ::core::ffi::c_int
                                                                    || *l.offset(
                                                                        strlen(l).wrapping_sub(
                                                                            1 as size_t,
                                                                        )
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '['
                                                                            as ::core::ffi::c_int)
                                                            {
                                                                amount += ind_continuation;
                                                            }
                                                            find_last_paren(
                                                                l,
                                                                '(' as ::core::ffi::c_char,
                                                                ')' as ::core::ffi::c_char,
                                                            );
                                                            trypos = find_match_paren(
                                                                corr_ind_maxparen(
                                                                    &raw mut cur_curpos,
                                                                ),
                                                            );
                                                            if !trypos.is_null()
                                                                && ((*trypos).lnum
                                                                    < (*tryposBrace).lnum
                                                                    || (*trypos).lnum
                                                                        == (*tryposBrace).lnum
                                                                        && (*trypos).col
                                                                            < (*tryposBrace).col)
                                                            {
                                                                trypos =
                                                                    ::core::ptr::null_mut::<pos_T>(
                                                                    );
                                                            }
                                                            l = get_cursor_line_ptr();
                                                            if trypos.is_null()
                                                                && terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                            {
                                                                if find_last_paren(
                                                                    l,
                                                                    '{' as ::core::ffi::c_char,
                                                                    '}' as ::core::ffi::c_char,
                                                                ) != 0
                                                                {
                                                                    trypos = find_start_brace();
                                                                }
                                                                l = get_cursor_line_ptr();
                                                            }
                                                            if !trypos.is_null() {
                                                                (*curwin).w_cursor = *trypos;
                                                                l = get_cursor_line_ptr();
                                                                if cin_iscase(l, false_0 != 0)
                                                                    as ::core::ffi::c_int
                                                                    != 0
                                                                    || cin_isscopedecl(l)
                                                                        as ::core::ffi::c_int
                                                                        != 0
                                                                {
                                                                    (*curwin).w_cursor.lnum += 1;
                                                                    (*curwin).w_cursor.col = 0
                                                                        as ::core::ffi::c_int
                                                                        as colnr_T;
                                                                    continue;
                                                                }
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                == ',' as ::core::ffi::c_int
                                                            {
                                                                while (*curwin).w_cursor.lnum
                                                                    > 1 as linenr_T
                                                                {
                                                                    l = ml_get(
                                                                        (*curwin).w_cursor.lnum
                                                                            - 1 as linenr_T,
                                                                    );
                                                                    if *l as ::core::ffi::c_int == NUL
                                                                        || *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                    (*curwin).w_cursor.lnum -= 1;
                                                                    (*curwin).w_cursor.col = 0
                                                                        as ::core::ffi::c_int
                                                                        as colnr_T;
                                                                }
                                                                l = get_cursor_line_ptr();
                                                            }
                                                            if (*curbuf).b_ind_js != 0 {
                                                                cur_amount = get_indent();
                                                            } else {
                                                                cur_amount = skip_label(
                                                                    (*curwin).w_cursor.lnum,
                                                                    &raw mut l,
                                                                );
                                                            }
                                                            if terminated as ::core::ffi::c_int
                                                                != ',' as ::core::ffi::c_int
                                                                && lookfor != LOOKFOR_TERM
                                                                && *theline.offset(
                                                                    0 as ::core::ffi::c_int
                                                                        as isize,
                                                                )
                                                                    as ::core::ffi::c_int
                                                                    == '{' as ::core::ffi::c_int
                                                            {
                                                                amount = cur_amount;
                                                                if *skipwhite(l)
                                                                    as ::core::ffi::c_int
                                                                    != '{' as ::core::ffi::c_int
                                                                {
                                                                    amount +=
                                                                        (*curbuf).b_ind_open_extra;
                                                                }
                                                                if !((*curbuf).b_ind_cpp_baseclass
                                                                    != 0
                                                                    && (*curbuf).b_ind_js == 0)
                                                                {
                                                                    break;
                                                                }
                                                                lookfor = LOOKFOR_CPP_BASECLASS;
                                                            } else if cin_is_cinword(l)
                                                                as ::core::ffi::c_int
                                                                != 0
                                                                || cin_iselse(skipwhite(l)) != 0
                                                            {
                                                                if lookfor == LOOKFOR_UNTERM
                                                                    || lookfor
                                                                        == LOOKFOR_ENUM_OR_INIT
                                                                {
                                                                    if cont_amount
                                                                        > 0 as ::core::ffi::c_int
                                                                    {
                                                                        amount = cont_amount;
                                                                    } else {
                                                                        amount += ind_continuation;
                                                                    }
                                                                    break;
                                                                } else {
                                                                    amount = cur_amount;
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf)
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    if lookfor != LOOKFOR_TERM {
                                                                        amount += (*curbuf)
                                                                            .b_ind_level
                                                                            + (*curbuf)
                                                                                .b_ind_no_brace;
                                                                        break;
                                                                    } else {
                                                                        l = skipwhite(
                                                                            get_cursor_line_ptr(),
                                                                        );
                                                                        if cin_isdo(l) != 0 {
                                                                            if whilelevel == 0 as ::core::ffi::c_int {
                                                                                break;
                                                                            }
                                                                            whilelevel -= 1;
                                                                        }
                                                                        if !(cin_iselse(l) != 0
                                                                            && whilelevel == 0 as ::core::ffi::c_int)
                                                                        {
                                                                            continue;
                                                                        }
                                                                        if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                                                                            (*curwin).w_cursor.col = (l
                                                                                .offset_from(get_cursor_line_ptr()) as ::core::ffi::c_int
                                                                                + 1 as ::core::ffi::c_int) as colnr_T;
                                                                        }
                                                                        trypos = find_start_brace();
                                                                        if trypos.is_null()
                                                                            || find_match(
                                                                                LOOKFOR_IF,
                                                                                (*trypos).lnum,
                                                                            ) == FAIL
                                                                        {
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            } else if lookfor == LOOKFOR_UNTERM {
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else if lookfor
                                                                == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if terminated as ::core::ffi::c_int
                                                                    == ',' as ::core::ffi::c_int
                                                                {
                                                                    if (*curbuf).b_ind_cpp_baseclass
                                                                        == 0 as ::core::ffi::c_int
                                                                    {
                                                                        break;
                                                                    }
                                                                    lookfor = LOOKFOR_CPP_BASECLASS;
                                                                } else if amount > cur_amount {
                                                                    amount = cur_amount;
                                                                }
                                                            } else {
                                                                l = get_cursor_line_ptr();
                                                                amount = cur_amount;
                                                                n = strlen(l) as ::core::ffi::c_int;
                                                                if (*curbuf).b_ind_js != 0
                                                                    && terminated as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                    && (*skipwhite(l) as ::core::ffi::c_int
                                                                        == ']' as ::core::ffi::c_int
                                                                        || n >= 2 as ::core::ffi::c_int
                                                                            && *l.offset((n - 2 as ::core::ffi::c_int) as isize)
                                                                                as ::core::ffi::c_int == ']' as ::core::ffi::c_int)
                                                                {
                                                                    break;
                                                                }
                                                                if lookfor == LOOKFOR_INITIAL
                                                                    && terminated
                                                                        as ::core::ffi::c_int
                                                                        == ',' as ::core::ffi::c_int
                                                                {
                                                                    if (*curbuf).b_ind_js != 0 {
                                                                        if cin_iscomment(skipwhite(
                                                                            l,
                                                                        )) != 0
                                                                        {
                                                                            break;
                                                                        }
                                                                        lookfor = LOOKFOR_COMMA;
                                                                        trypos = find_match_char(
                                                                            '[' as ::core::ffi::c_char,
                                                                            (*curbuf).b_ind_maxparen,
                                                                        );
                                                                        if trypos.is_null() {
                                                                            continue;
                                                                        }
                                                                        if (*trypos).lnum
                                                                            == (*curwin)
                                                                                .w_cursor
                                                                                .lnum
                                                                                - 1 as linenr_T
                                                                        {
                                                                            break;
                                                                        }
                                                                        ourscope = (*trypos).lnum;
                                                                    } else {
                                                                        lookfor =
                                                                            LOOKFOR_ENUM_OR_INIT;
                                                                        cont_amount =
                                                                            cin_first_id_amount();
                                                                    }
                                                                } else {
                                                                    if lookfor == LOOKFOR_INITIAL
                                                                        && *l as ::core::ffi::c_int != NUL
                                                                        && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                                                            as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                                                    {
                                                                        cont_amount = cin_get_equal_amount((*curwin).w_cursor.lnum);
                                                                    }
                                                                    if lookfor != LOOKFOR_TERM
                                                                        && lookfor != LOOKFOR_JS_KEY
                                                                        && lookfor != LOOKFOR_COMMA
                                                                        && raw_string_start
                                                                            != (*curwin)
                                                                                .w_cursor
                                                                                .lnum
                                                                    {
                                                                        lookfor = LOOKFOR_UNTERM;
                                                                    }
                                                                }
                                                            }
                                                        } else if cin_iswhileofdo_end(
                                                            terminated as uint8_t
                                                                as ::core::ffi::c_int,
                                                        ) != 0
                                                        {
                                                            if lookfor == LOOKFOR_UNTERM
                                                                || lookfor == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if cont_amount
                                                                    > 0 as ::core::ffi::c_int
                                                                {
                                                                    amount = cont_amount;
                                                                } else {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else {
                                                                if whilelevel
                                                                    == 0 as ::core::ffi::c_int
                                                                {
                                                                    lookfor = LOOKFOR_TERM;
                                                                    amount = get_indent();
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf)
                                                                            .b_ind_open_extra;
                                                                    }
                                                                }
                                                                whilelevel += 1;
                                                            }
                                                        } else if lookfor == LOOKFOR_NOBREAK
                                                            && cin_isbreak(skipwhite(
                                                                get_cursor_line_ptr(),
                                                            )) != 0
                                                        {
                                                            lookfor = LOOKFOR_ANY;
                                                        } else {
                                                            if whilelevel > 0 as ::core::ffi::c_int
                                                            {
                                                                l = cin_skipcomment(
                                                                    get_cursor_line_ptr(),
                                                                );
                                                                if cin_isdo(l) != 0 {
                                                                    amount = get_indent();
                                                                    whilelevel -= 1;
                                                                    continue;
                                                                }
                                                            }
                                                            if lookfor == LOOKFOR_UNTERM
                                                                || lookfor == LOOKFOR_ENUM_OR_INIT
                                                            {
                                                                if cont_amount
                                                                    > 0 as ::core::ffi::c_int
                                                                {
                                                                    amount = cont_amount;
                                                                } else {
                                                                    amount += ind_continuation;
                                                                }
                                                                break;
                                                            } else if lookfor == LOOKFOR_TERM {
                                                                if lookfor_break == 0
                                                                    && whilelevel
                                                                        == 0 as ::core::ffi::c_int
                                                                {
                                                                    break;
                                                                }
                                                            } else {
                                                                loop {
                                                                    l = get_cursor_line_ptr();
                                                                    if find_last_paren(
                                                                        l,
                                                                        '(' as ::core::ffi::c_char,
                                                                        ')' as ::core::ffi::c_char,
                                                                    ) != 0
                                                                        && {
                                                                            trypos = find_match_paren((*curbuf).b_ind_maxparen);
                                                                            !trypos.is_null()
                                                                        }
                                                                    {
                                                                        (*curwin).w_cursor =
                                                                            *trypos;
                                                                        l = get_cursor_line_ptr();
                                                                        if cin_iscase(l, false_0 != 0) as ::core::ffi::c_int != 0
                                                                            || cin_isscopedecl(l) as ::core::ffi::c_int != 0
                                                                        {
                                                                            (*curwin).w_cursor.lnum += 1;
                                                                            (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                            break;
                                                                        }
                                                                    }
                                                                    iscase = (*curbuf)
                                                                        .b_ind_keep_case_label
                                                                        != 0
                                                                        && cin_iscase(
                                                                            l,
                                                                            false_0 != 0,
                                                                        )
                                                                            as ::core::ffi::c_int
                                                                            != 0;
                                                                    amount = skip_label(
                                                                        (*curwin).w_cursor.lnum,
                                                                        &raw mut l,
                                                                    );
                                                                    if *theline.offset(
                                                                        0 as ::core::ffi::c_int
                                                                            as isize,
                                                                    )
                                                                        as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount += (*curbuf)
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    l = skipwhite(l);
                                                                    if *l as ::core::ffi::c_int
                                                                        == '{' as ::core::ffi::c_int
                                                                    {
                                                                        amount -= (*curbuf)
                                                                            .b_ind_open_extra;
                                                                    }
                                                                    lookfor = if iscase
                                                                        as ::core::ffi::c_int
                                                                        != 0
                                                                    {
                                                                        LOOKFOR_ANY
                                                                    } else {
                                                                        LOOKFOR_TERM
                                                                    };
                                                                    if lookfor == LOOKFOR_TERM
                                                                        && *l as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                                                                        && cin_iselse(l) != 0
                                                                        && whilelevel == 0 as ::core::ffi::c_int
                                                                    {
                                                                        trypos = find_start_brace();
                                                                        if trypos.is_null()
                                                                            || find_match(LOOKFOR_IF, (*trypos).lnum) == FAIL
                                                                        {
                                                                            break 's_2927;
                                                                        } else {
                                                                            break;
                                                                        }
                                                                    } else {
                                                                        l = get_cursor_line_ptr();
                                                                        if !(find_last_paren(
                                                                            l,
                                                                            '{' as ::core::ffi::c_char,
                                                                            '}' as ::core::ffi::c_char,
                                                                        ) != 0
                                                                            && {
                                                                                trypos = find_start_brace();
                                                                                !trypos.is_null()
                                                                            })
                                                                        {
                                                                            break;
                                                                        }
                                                                        (*curwin).w_cursor = *trypos;
                                                                        l = cin_skipcomment(get_cursor_line_ptr());
                                                                        if *l as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                                                                            || cin_iselse(l) == 0
                                                                        {
                                                                            continue;
                                                                        }
                                                                        (*curwin).w_cursor.lnum += 1;
                                                                        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if cin_iscomment(theline) != 0 {
                            amount += (*curbuf).b_ind_comment;
                        }
                        if (*curbuf).b_ind_jump_label > 0 as ::core::ffi::c_int
                            && original_line_islabel != 0
                        {
                            amount -= (*curbuf).b_ind_jump_label;
                        }
                    } else if *theline.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '{' as ::core::ffi::c_int
                    {
                        amount = (*curbuf).b_ind_first_open;
                    } else if cur_curpos.lnum < (*curbuf).b_ml.ml_line_count
                        && cin_nocode(theline) == 0
                        && vim_strchr(theline, '{' as ::core::ffi::c_int).is_null()
                        && vim_strchr(theline, '}' as ::core::ffi::c_int).is_null()
                        && cin_ends_in(theline, b":\0".as_ptr() as *const ::core::ffi::c_char) == 0
                        && cin_ends_in(theline, b",\0".as_ptr() as *const ::core::ffi::c_char) == 0
                        && cin_isfuncdecl(
                            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                            cur_curpos.lnum + 1 as linenr_T,
                            cur_curpos.lnum + 1 as linenr_T,
                        ) != 0
                        && cin_isterminated(theline, false_0, true_0) == 0
                    {
                        amount = (*curbuf).b_ind_func_type;
                    } else {
                        amount = 0 as ::core::ffi::c_int;
                        (*curwin).w_cursor = cur_curpos;
                        while (*curwin).w_cursor.lnum > 1 as linenr_T {
                            (*curwin).w_cursor.lnum -= 1;
                            (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            l = get_cursor_line_ptr();
                            trypos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
                            if !trypos.is_null() {
                                (*curwin).w_cursor.lnum = (*trypos).lnum + 1 as linenr_T;
                                (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                n = 0 as ::core::ffi::c_int;
                                if (*curbuf).b_ind_cpp_baseclass != 0 as ::core::ffi::c_int {
                                    n = cin_is_cpp_baseclass(&raw mut cache_cpp_baseclass);
                                    l = get_cursor_line_ptr();
                                }
                                if n != 0 {
                                    amount = get_baseclass_amount(
                                        cache_cpp_baseclass.lpos.col as ::core::ffi::c_int,
                                    );
                                    break;
                                } else {
                                    if cin_ispreproc_cont(
                                        &raw mut l,
                                        &raw mut (*curwin).w_cursor.lnum,
                                        &raw mut amount,
                                    ) != 0
                                    {
                                        continue;
                                    }
                                    if cin_nocode(l) != 0 {
                                        continue;
                                    }
                                    if cin_ends_in(l, b",\0".as_ptr() as *const ::core::ffi::c_char)
                                        != 0
                                        || *l as ::core::ffi::c_int != NUL && {
                                            n =
                                                *l.offset(
                                                    strlen(l).wrapping_sub(1 as size_t) as isize
                                                )
                                                    as uint8_t
                                                    as ::core::ffi::c_int;
                                            n == '\\' as ::core::ffi::c_int
                                        }
                                    {
                                        if find_last_paren(
                                            l,
                                            '(' as ::core::ffi::c_char,
                                            ')' as ::core::ffi::c_char,
                                        ) != 0
                                            && {
                                                trypos = find_match_paren((*curbuf).b_ind_maxparen);
                                                !trypos.is_null()
                                            }
                                        {
                                            (*curwin).w_cursor = *trypos;
                                        }
                                        while n == 0 as ::core::ffi::c_int
                                            && (*curwin).w_cursor.lnum > 1 as linenr_T
                                        {
                                            l = ml_get((*curwin).w_cursor.lnum - 1 as linenr_T);
                                            if *l as ::core::ffi::c_int == NUL
                                                || *l
                                                    .offset(strlen(l).wrapping_sub(1 as size_t)
                                                        as isize)
                                                    as ::core::ffi::c_int
                                                    != '\\' as ::core::ffi::c_int
                                            {
                                                break;
                                            }
                                            (*curwin).w_cursor.lnum -= 1;
                                            (*curwin).w_cursor.col =
                                                0 as ::core::ffi::c_int as colnr_T;
                                        }
                                        amount = get_indent();
                                        if amount == 0 as ::core::ffi::c_int {
                                            amount = cin_first_id_amount();
                                        }
                                        if amount == 0 as ::core::ffi::c_int {
                                            amount = ind_continuation;
                                        }
                                        break;
                                    } else {
                                        if cin_isfuncdecl(
                                            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                                            cur_curpos.lnum,
                                            0 as linenr_T,
                                        ) != 0
                                        {
                                            break;
                                        }
                                        l = get_cursor_line_ptr();
                                        if *skipwhite(l) as ::core::ffi::c_int
                                            == '}' as ::core::ffi::c_int
                                        {
                                            break;
                                        }
                                        if cin_ends_in(
                                            l,
                                            b"};\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            break;
                                        }
                                        if cin_ends_in(
                                            l,
                                            b"[\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            amount = get_indent() + ind_continuation;
                                            break;
                                        } else {
                                            look = skipwhite(l);
                                            if *look as ::core::ffi::c_int
                                                == ';' as ::core::ffi::c_int
                                                && cin_nocode(
                                                    look.offset(1 as ::core::ffi::c_int as isize),
                                                ) != 0
                                            {
                                                let mut curpos_save: pos_T = (*curwin).w_cursor;
                                                while (*curwin).w_cursor.lnum > 1 as linenr_T {
                                                    (*curwin).w_cursor.lnum -= 1;
                                                    look = ml_get((*curwin).w_cursor.lnum);
                                                    if !(cin_nocode(look) != 0
                                                        || cin_ispreproc_cont(
                                                            &raw mut look,
                                                            &raw mut (*curwin).w_cursor.lnum,
                                                            &raw mut amount,
                                                        ) != 0)
                                                    {
                                                        break;
                                                    }
                                                }
                                                if (*curwin).w_cursor.lnum > 0 as linenr_T
                                                    && cin_ends_in(
                                                        look,
                                                        b"}\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                {
                                                    break;
                                                }
                                                (*curwin).w_cursor = curpos_save;
                                            }
                                            if cin_isfuncdecl(
                                                &raw mut l,
                                                (*curwin).w_cursor.lnum,
                                                0 as linenr_T,
                                            ) != 0
                                            {
                                                amount = (*curbuf).b_ind_param;
                                                break;
                                            } else {
                                                if cin_ends_in(
                                                    l,
                                                    b";\0".as_ptr() as *const ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    l = ml_get(
                                                        (*curwin).w_cursor.lnum - 1 as linenr_T,
                                                    );
                                                    if cin_ends_in(
                                                        l,
                                                        b",\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                        || *l as ::core::ffi::c_int != NUL
                                                            && *l.offset(
                                                                strlen(l).wrapping_sub(1 as size_t)
                                                                    as isize,
                                                            )
                                                                as ::core::ffi::c_int
                                                                == '\\' as ::core::ffi::c_int
                                                    {
                                                        break;
                                                    }
                                                    l = get_cursor_line_ptr();
                                                }
                                                find_last_paren(
                                                    l,
                                                    '(' as ::core::ffi::c_char,
                                                    ')' as ::core::ffi::c_char,
                                                );
                                                trypos = find_match_paren((*curbuf).b_ind_maxparen);
                                                if !trypos.is_null() {
                                                    (*curwin).w_cursor = *trypos;
                                                }
                                                amount = get_indent();
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if cin_iscomment(theline) != 0 {
                            amount += (*curbuf).b_ind_comment;
                        }
                        if cur_curpos.lnum > 1 as linenr_T {
                            l = ml_get(cur_curpos.lnum - 1 as linenr_T);
                            if *l as ::core::ffi::c_int != NUL
                                && *l.offset(strlen(l).wrapping_sub(1 as size_t) as isize)
                                    as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                            {
                                cur_amount = cin_get_equal_amount(cur_curpos.lnum - 1 as linenr_T);
                                if cur_amount > 0 as ::core::ffi::c_int {
                                    amount = cur_amount;
                                } else if cur_amount == 0 as ::core::ffi::c_int {
                                    amount += ind_continuation;
                                }
                            }
                        }
                    }
                }
            }
        }
        if amount < 0 as ::core::ffi::c_int {
            amount = 0 as ::core::ffi::c_int;
        }
    }
    (*curwin).w_cursor = cur_curpos;
    xfree(linecopy as *mut ::core::ffi::c_void);
    return amount;
}
pub const BRACE_IN_COL0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BRACE_AT_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BRACE_AT_END: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_INITIAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LOOKFOR_IF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOOKFOR_DO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOOKFOR_CASE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOOKFOR_ANY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LOOKFOR_TERM: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LOOKFOR_UNTERM: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const LOOKFOR_SCOPEDECL: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const LOOKFOR_NOBREAK: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const LOOKFOR_CPP_BASECLASS: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const LOOKFOR_ENUM_OR_INIT: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const LOOKFOR_JS_KEY: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const LOOKFOR_COMMA: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
unsafe extern "C" fn find_match(
    mut lookfor: ::core::ffi::c_int,
    mut ourscope: linenr_T,
) -> ::core::ffi::c_int {
    let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut theirscope: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut mightbeif: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut elselevel: ::core::ffi::c_int = 0;
    let mut whilelevel: ::core::ffi::c_int = 0;
    if lookfor == LOOKFOR_IF {
        elselevel = 1 as ::core::ffi::c_int;
        whilelevel = 0 as ::core::ffi::c_int;
    } else {
        elselevel = 0 as ::core::ffi::c_int;
        whilelevel = 1 as ::core::ffi::c_int;
    }
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    while (*curwin).w_cursor.lnum > ourscope + 1 as linenr_T {
        (*curwin).w_cursor.lnum -= 1;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        look = cin_skipcomment(get_cursor_line_ptr());
        if cin_iselse(look) == 0
            && cin_isif(look) == 0
            && cin_isdo(look) == 0
            && cin_iswhileofdo(look, (*curwin).w_cursor.lnum) == 0
        {
            continue;
        }
        theirscope = find_start_brace();
        if theirscope.is_null() {
            break;
        }
        if (*theirscope).lnum < ourscope {
            break;
        }
        if (*theirscope).lnum > ourscope {
            continue;
        }
        look = cin_skipcomment(get_cursor_line_ptr());
        if !(lookfor == LOOKFOR_IF && whilelevel != 0) {
            if cin_iselse(look) != 0 {
                mightbeif = cin_skipcomment(look.offset(4 as ::core::ffi::c_int as isize));
                if cin_isif(mightbeif) == 0 {
                    elselevel += 1;
                }
                continue;
            } else if cin_isif(look) != 0 {
                elselevel -= 1;
                if elselevel == 0 as ::core::ffi::c_int && lookfor == LOOKFOR_IF {
                    whilelevel = 0 as ::core::ffi::c_int;
                }
            }
        }
        if cin_iswhileofdo(look, (*curwin).w_cursor.lnum) != 0 {
            whilelevel += 1;
        } else {
            if cin_isdo(look) != 0 {
                whilelevel -= 1;
            }
            if elselevel <= 0 as ::core::ffi::c_int && whilelevel <= 0 as ::core::ffi::c_int {
                return OK;
            }
        }
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn in_cinkeys(
    mut keytyped: ::core::ffi::c_int,
    mut when: ::core::ffi::c_int,
    mut line_is_empty: bool,
) -> bool {
    let mut look: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut try_match: bool = false;
    let mut try_match_word: bool = false;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut icase: bool = false;
    if keytyped == NUL {
        return false_0 != 0;
    }
    if *(*curbuf).b_p_inde as ::core::ffi::c_int != NUL {
        look = (*curbuf).b_p_indk;
    } else {
        look = (*curbuf).b_p_cink;
    }
    while *look != 0 {
        match when {
            42 => {
                try_match = *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int;
            }
            33 => {
                try_match = *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
            }
            _ => {
                try_match = *look as ::core::ffi::c_int != '*' as ::core::ffi::c_int;
            }
        }
        if *look as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *look as ::core::ffi::c_int == '!' as ::core::ffi::c_int
        {
            look = look.offset(1);
        }
        if *look as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
            try_match_word = try_match;
            if !line_is_empty {
                try_match = false_0 != 0;
            }
            look = look.offset(1);
        } else {
            try_match_word = false_0 != 0;
        }
        if *look as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                >= '?' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                <= '_' as ::core::ffi::c_int
        {
            if try_match as ::core::ffi::c_int != 0
                && keytyped
                    == (if (*look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        < 'a' as ::core::ffi::c_int
                        || *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            > 'z' as ::core::ffi::c_int
                    {
                        *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    } else {
                        *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                    }) ^ 0x40 as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(2 as ::core::ffi::c_int as isize);
        } else if *look as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == KEY_OPEN_FORW as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == 'O' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == KEY_OPEN_BACK as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0
                && keytyped == 'e' as ::core::ffi::c_int
                && (*curwin).w_cursor.col >= 4 as ::core::ffi::c_int
            {
                p = get_cursor_line_ptr();
                if skipwhite(p)
                    == p.offset((*curwin).w_cursor.col as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize))
                    && strncmp(
                        p.offset((*curwin).w_cursor.col as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize)),
                        b"else\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    return true_0 != 0;
                }
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if try_match as ::core::ffi::c_int != 0 && keytyped == ':' as ::core::ffi::c_int {
                p = get_cursor_line_ptr();
                if cin_iscase(p, false_0 != 0) as ::core::ffi::c_int != 0
                    || cin_isscopedecl(p) as ::core::ffi::c_int != 0
                    || cin_islabel() as ::core::ffi::c_int != 0
                {
                    return true_0 != 0;
                }
                p = get_cursor_line_ptr();
                if (*curwin).w_cursor.col > 2 as ::core::ffi::c_int
                    && *p.offset(
                        ((*curwin).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as isize,
                    ) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    && *p.offset(
                        ((*curwin).w_cursor.col as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
                            as isize,
                    ) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                {
                    *p.offset(
                        ((*curwin).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as isize,
                    ) = ' ' as ::core::ffi::c_char;
                    let i: bool = cin_iscase(p, false_0 != 0) as ::core::ffi::c_int != 0
                        || cin_isscopedecl(p) as ::core::ffi::c_int != 0
                        || cin_islabel() as ::core::ffi::c_int != 0;
                    p = get_cursor_line_ptr();
                    *p.offset(
                        ((*curwin).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as isize,
                    ) = ':' as ::core::ffi::c_char;
                    if i {
                        return true_0 != 0;
                    }
                }
            }
            look = look.offset(1);
        } else if *look as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            if try_match {
                if !vim_strchr(
                    b"<>!*oOe0:\0".as_ptr() as *const ::core::ffi::c_char,
                    *look.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                    && keytyped
                        == *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    return true_0 != 0;
                }
                if keytyped == get_special_key_code(look.offset(1 as ::core::ffi::c_int as isize)) {
                    return true_0 != 0;
                }
            }
            while *look as ::core::ffi::c_int != 0
                && *look as ::core::ffi::c_int != '>' as ::core::ffi::c_int
            {
                look = look.offset(1);
            }
            while *look as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                look = look.offset(1);
            }
        } else if *look as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ',' as ::core::ffi::c_int
            && *look.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            look = look.offset(1);
            if *look as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                icase = true_0 != 0;
                look = look.offset(1);
            } else {
                icase = false_0 != 0;
            }
            p = vim_strchr(look, ',' as ::core::ffi::c_int);
            if p.is_null() {
                p = look.offset(strlen(look) as isize);
            }
            if (try_match as ::core::ffi::c_int != 0 || try_match_word as ::core::ffi::c_int != 0)
                && (*curwin).w_cursor.col >= p.offset_from(look) as colnr_T
            {
                let mut match_0: bool = false_0 != 0;
                if keytyped == KEY_COMPLETE as ::core::ffi::c_int {
                    let mut n: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut s: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                    s = line.offset((*curwin).w_cursor.col as isize);
                    while s > line {
                        n = mb_prevptr(line, s);
                        if !vim_iswordp(n) {
                            break;
                        }
                        s = n;
                    }
                    '_c2rust_label: {
                        if p >= look
                            && p.offset_from(look) as uintmax_t <= 18446744073709551615 as uintmax_t
                        {
                        } else {
                            __assert_fail(
                                b"p >= look && (uintmax_t)(p - look) <= SIZE_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/indent_c.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                3933 as ::core::ffi::c_uint,
                                __ASSERT_FUNCTION.as_ptr(),
                            );
                        }
                    };
                    if s.offset(p.offset_from(look) as isize)
                        <= line.offset((*curwin).w_cursor.col as isize)
                        && (if icase as ::core::ffi::c_int != 0 {
                            mb_strnicmp(s, look, p.offset_from(look) as size_t)
                        } else {
                            strncmp(s, look, p.offset_from(look) as size_t)
                        }) == 0 as ::core::ffi::c_int
                    {
                        match_0 = true_0 != 0;
                    }
                } else if keytyped
                    == *p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    || icase as ::core::ffi::c_int != 0
                        && keytyped < 256 as ::core::ffi::c_int
                        && keytyped >= 0 as ::core::ffi::c_int
                        && tolower(keytyped)
                            == tolower(*p.offset(-1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int)
                {
                    let mut line_0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                    '_c2rust_label_0: {
                        if p >= look
                            && p.offset_from(look) as uintmax_t <= 18446744073709551615 as uintmax_t
                        {
                        } else {
                            __assert_fail(
                                b"p >= look && (uintmax_t)(p - look) <= SIZE_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/indent_c.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                3946 as ::core::ffi::c_uint,
                                __ASSERT_FUNCTION.as_ptr(),
                            );
                        }
                    };
                    if ((*curwin).w_cursor.col == p.offset_from(look) as colnr_T
                        || !vim_iswordc(
                            *line_0.offset((-p.offset_from(look) - 1 as isize) as isize) as uint8_t
                                as ::core::ffi::c_int,
                        ))
                        && (if icase as ::core::ffi::c_int != 0 {
                            mb_strnicmp(
                                line_0.offset(-(p.offset_from(look) as isize)),
                                look,
                                p.offset_from(look) as size_t,
                            )
                        } else {
                            strncmp(
                                line_0.offset(-(p.offset_from(look) as isize)),
                                look,
                                p.offset_from(look) as size_t,
                            )
                        }) == 0 as ::core::ffi::c_int
                    {
                        match_0 = true_0 != 0;
                    }
                }
                if match_0 as ::core::ffi::c_int != 0
                    && try_match_word as ::core::ffi::c_int != 0
                    && !try_match
                {
                    if getwhitecols_curline()
                        != ((*curwin).w_cursor.col as isize - p.offset_from(look))
                            as ::core::ffi::c_int as intptr_t
                    {
                        match_0 = false_0 != 0;
                    }
                }
                if match_0 {
                    return true_0 != 0;
                }
            }
            look = p;
        } else {
            if try_match as ::core::ffi::c_int != 0
                && *look as uint8_t as ::core::ffi::c_int == keytyped
            {
                return true_0 != 0;
            }
            if *look as ::core::ffi::c_int != NUL {
                look = look.offset(1);
            }
        }
        look = skip_to_option_part(look);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_c_expr_indent() {
    if *(*curbuf).b_p_inde as ::core::ffi::c_int != NUL {
        fixthisline(Some(
            get_expr_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
        ));
    } else {
        fixthisline(Some(
            get_c_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
        ));
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_cindent(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut pos: pos_T = (*curwin).w_cursor;
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum >= 1 as linenr_T && lnum <= (*curbuf).b_ml.ml_line_count {
        (*curwin).w_cursor.lnum = lnum;
        (*rettv).vval.v_number = get_c_indent() as varnumber_T;
        (*curwin).w_cursor = pos;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
