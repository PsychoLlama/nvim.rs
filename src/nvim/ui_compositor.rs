use crate::src::nvim::global_cell::GlobalCell;
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
    fn abort() -> !;
    fn llabs(__x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static default_grid: GlobalCell<ScreenGrid>;
    fn schar_from_buf(buf: *const ::core::ffi::c_char, len: size_t) -> schar_T;
    fn schar_from_char(c: ::core::ffi::c_int) -> schar_T;
    static rdb_flags: GlobalCell<::core::ffi::c_uint>;
    static p_wd: GlobalCell<OptInt>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn hl_blend_attrs(
        back_attr: ::core::ffi::c_int,
        front_attr: ::core::ffi::c_int,
        through: *mut bool,
    ) -> ::core::ffi::c_int;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static msg_grid: GlobalCell<ScreenGrid>;
    fn os_sleep(ms: uint64_t);
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_flush();
    fn ui_composed_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_composed_call_grid_cursor_goto(grid: Integer, row: Integer, col: Integer);
    fn ui_composed_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn ui_composed_call_raw_line(
        grid: Integer,
        row: Integer,
        startcol: Integer,
        endcol: Integer,
        clearcol: Integer,
        clearattr: Integer,
        flags: LineFlags,
        chunk: *const schar_T,
        attrs: *const sattr_T,
    );
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
pub type Boolean = bool;
pub type Integer = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_12 = 76;
pub const HLF_PRE: C2Rust_Unnamed_12 = 75;
pub const HLF_OK: C2Rust_Unnamed_12 = 74;
pub const HLF_SO: C2Rust_Unnamed_12 = 73;
pub const HLF_SE: C2Rust_Unnamed_12 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_12 = 71;
pub const HLF_TS: C2Rust_Unnamed_12 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_12 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_12 = 68;
pub const HLF_CU: C2Rust_Unnamed_12 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_12 = 66;
pub const HLF_WBR: C2Rust_Unnamed_12 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_12 = 64;
pub const HLF_MSG: C2Rust_Unnamed_12 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_12 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_12 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_12 = 60;
pub const HLF_0: C2Rust_Unnamed_12 = 59;
pub const HLF_QFL: C2Rust_Unnamed_12 = 58;
pub const HLF_MC: C2Rust_Unnamed_12 = 57;
pub const HLF_CUL: C2Rust_Unnamed_12 = 56;
pub const HLF_CUC: C2Rust_Unnamed_12 = 55;
pub const HLF_TPF: C2Rust_Unnamed_12 = 54;
pub const HLF_TPS: C2Rust_Unnamed_12 = 53;
pub const HLF_TP: C2Rust_Unnamed_12 = 52;
pub const HLF_PBR: C2Rust_Unnamed_12 = 51;
pub const HLF_PST: C2Rust_Unnamed_12 = 50;
pub const HLF_PSB: C2Rust_Unnamed_12 = 49;
pub const HLF_PSX: C2Rust_Unnamed_12 = 48;
pub const HLF_PNX: C2Rust_Unnamed_12 = 47;
pub const HLF_PSK: C2Rust_Unnamed_12 = 46;
pub const HLF_PNK: C2Rust_Unnamed_12 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_12 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_12 = 43;
pub const HLF_PSI: C2Rust_Unnamed_12 = 42;
pub const HLF_PNI: C2Rust_Unnamed_12 = 41;
pub const HLF_SPL: C2Rust_Unnamed_12 = 40;
pub const HLF_SPR: C2Rust_Unnamed_12 = 39;
pub const HLF_SPC: C2Rust_Unnamed_12 = 38;
pub const HLF_SPB: C2Rust_Unnamed_12 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_12 = 36;
pub const HLF_SC: C2Rust_Unnamed_12 = 35;
pub const HLF_TXA: C2Rust_Unnamed_12 = 34;
pub const HLF_TXD: C2Rust_Unnamed_12 = 33;
pub const HLF_DED: C2Rust_Unnamed_12 = 32;
pub const HLF_CHD: C2Rust_Unnamed_12 = 31;
pub const HLF_ADD: C2Rust_Unnamed_12 = 30;
pub const HLF_FC: C2Rust_Unnamed_12 = 29;
pub const HLF_FL: C2Rust_Unnamed_12 = 28;
pub const HLF_WM: C2Rust_Unnamed_12 = 27;
pub const HLF_W: C2Rust_Unnamed_12 = 26;
pub const HLF_VNC: C2Rust_Unnamed_12 = 25;
pub const HLF_V: C2Rust_Unnamed_12 = 24;
pub const HLF_T: C2Rust_Unnamed_12 = 23;
pub const HLF_VSP: C2Rust_Unnamed_12 = 22;
pub const HLF_C: C2Rust_Unnamed_12 = 21;
pub const HLF_SNC: C2Rust_Unnamed_12 = 20;
pub const HLF_S: C2Rust_Unnamed_12 = 19;
pub const HLF_R: C2Rust_Unnamed_12 = 18;
pub const HLF_CLF: C2Rust_Unnamed_12 = 17;
pub const HLF_CLS: C2Rust_Unnamed_12 = 16;
pub const HLF_CLN: C2Rust_Unnamed_12 = 15;
pub const HLF_LNB: C2Rust_Unnamed_12 = 14;
pub const HLF_LNA: C2Rust_Unnamed_12 = 13;
pub const HLF_N: C2Rust_Unnamed_12 = 12;
pub const HLF_CM: C2Rust_Unnamed_12 = 11;
pub const HLF_M: C2Rust_Unnamed_12 = 10;
pub const HLF_LC: C2Rust_Unnamed_12 = 9;
pub const HLF_L: C2Rust_Unnamed_12 = 8;
pub const HLF_I: C2Rust_Unnamed_12 = 7;
pub const HLF_E: C2Rust_Unnamed_12 = 6;
pub const HLF_D: C2Rust_Unnamed_12 = 5;
pub const HLF_AT: C2Rust_Unnamed_12 = 4;
pub const HLF_TERM: C2Rust_Unnamed_12 = 3;
pub const HLF_EOB: C2Rust_Unnamed_12 = 2;
pub const HLF_8: C2Rust_Unnamed_12 = 1;
pub const HLF_NONE: C2Rust_Unnamed_12 = 0;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kOptRdbFlagFlush: C2Rust_Unnamed_13 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_13 = 16;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_13 = 8;
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_13 = 4;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_13 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_13 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct packer_buffer_t {
    pub startptr: *mut ::core::ffi::c_char,
    pub ptr: *mut ::core::ffi::c_char,
    pub endptr: *mut ::core::ffi::c_char,
    pub anydata: *mut ::core::ffi::c_void,
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer) -> ()>;
pub type PackerBuffer = packer_buffer_t;
pub type UIExtension = ::core::ffi::c_uint;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_14 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_14 = 1;
pub type LineFlags = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_background: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ScreenGrid,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_15 = C2Rust_Unnamed_15 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<*mut ScreenGrid>(),
};
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
static composed_uis: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub static layers: GlobalCell<C2Rust_Unnamed_15> = GlobalCell::new(KV_INITIAL_VALUE);
static bufsize: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
static linebuf: GlobalCell<*mut schar_T> = GlobalCell::new(::core::ptr::null_mut::<schar_T>());
static attrbuf: GlobalCell<*mut sattr_T> = GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
static chk_height: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static chk_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static curgrid: GlobalCell<*mut ScreenGrid> =
    GlobalCell::new(::core::ptr::null_mut::<ScreenGrid>());
static valid_screen: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static msg_current_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(INT_MAX);
static msg_was_scrolled: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_sep_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
static msg_sep_char: GlobalCell<schar_T> = GlobalCell::new(' ' as ::core::ffi::c_int as schar_T);
static dbghl_normal: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_clear: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_composed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static dbghl_recompose: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub unsafe extern "C" fn ui_comp_init() {
    if (*layers.ptr()).size == (*layers.ptr()).capacity {
        (*layers.ptr()).capacity = if (*layers.ptr()).capacity != 0 {
            (*layers.ptr()).capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*layers.ptr()).items = xrealloc(
            (*layers.ptr()).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut ScreenGrid>().wrapping_mul((*layers.ptr()).capacity),
        ) as *mut *mut ScreenGrid;
    } else {
    };
    let c2rust_fresh0 = (*layers.ptr()).size;
    (*layers.ptr()).size = (*layers.ptr()).size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *(*layers.ptr()).items.offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = default_grid.ptr();
    curgrid.set(default_grid.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_syn_init() {
    dbghl_normal.set(syn_check_group(
        b"RedrawDebugNormal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 18]>().wrapping_sub(1 as size_t),
    ));
    dbghl_clear.set(syn_check_group(
        b"RedrawDebugClear\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
    ));
    dbghl_composed.set(syn_check_group(
        b"RedrawDebugComposed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>().wrapping_sub(1 as size_t),
    ));
    dbghl_recompose.set(syn_check_group(
        b"RedrawDebugRecompose\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 21]>().wrapping_sub(1 as size_t),
    ));
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_attach(mut ui: *mut RemoteUI) {
    (*composed_uis.ptr()) += 1;
    (*ui).composed = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_detach(mut ui: *mut RemoteUI) {
    (*composed_uis.ptr()) -= 1;
    if composed_uis.get() == 0 as ::core::ffi::c_int {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            linebuf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            attrbuf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        *ptr__0;
        bufsize.set(0 as size_t);
    }
    (*ui).composed = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_should_draw() -> bool {
    return composed_uis.get() != 0 as ::core::ffi::c_int
        && valid_screen.get() as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_layers_adjust(mut layer_idx: size_t, mut raise: bool) {
    let mut size: size_t = (*layers.ptr()).size;
    let mut layer: *mut ScreenGrid = *(*layers.ptr()).items.offset(layer_idx as isize);
    if raise {
        while layer_idx < size.wrapping_sub(1 as size_t)
            && (*layer).zindex
                > (**(*layers.ptr())
                    .items
                    .offset(layer_idx.wrapping_add(1 as size_t) as isize))
                .zindex
        {
            *(*layers.ptr()).items.offset(layer_idx as isize) = *(*layers.ptr())
                .items
                .offset(layer_idx.wrapping_add(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(layer_idx as isize)).comp_index = layer_idx;
            (**(*layers.ptr()).items.offset(layer_idx as isize)).pending_comp_index_update =
                true_0 != 0;
            layer_idx = layer_idx.wrapping_add(1);
        }
    } else {
        while layer_idx > 0 as size_t
            && (*layer).zindex
                < (**(*layers.ptr())
                    .items
                    .offset(layer_idx.wrapping_sub(1 as size_t) as isize))
                .zindex
        {
            *(*layers.ptr()).items.offset(layer_idx as isize) = *(*layers.ptr())
                .items
                .offset(layer_idx.wrapping_sub(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(layer_idx as isize)).comp_index = layer_idx;
            (**(*layers.ptr()).items.offset(layer_idx as isize)).pending_comp_index_update =
                true_0 != 0;
            layer_idx = layer_idx.wrapping_sub(1);
        }
    }
    *(*layers.ptr()).items.offset(layer_idx as isize) = layer;
    (*layer).comp_index = layer_idx;
    (*layer).pending_comp_index_update = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_put_grid(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
    mut valid: bool,
    mut on_top: bool,
) -> bool {
    let mut moved: bool = false;
    (*grid).pending_comp_index_update = true_0 != 0;
    if (*grid).comp_index != 0 as size_t {
        moved = row != (*grid).comp_row || col != (*grid).comp_col;
        if ui_comp_should_draw() {
            (*grid).comp_disabled = true_0 != 0;
            compose_area(
                (*grid).comp_row as Integer,
                row as Integer,
                (*grid).comp_col as Integer,
                ((*grid).comp_col + (*grid).comp_width) as Integer,
            );
            if (*grid).comp_col < col {
                compose_area(
                    (if row > (*grid).comp_row {
                        row
                    } else {
                        (*grid).comp_row
                    }) as Integer,
                    (if row + height < (*grid).comp_row + (*grid).comp_height {
                        row + height
                    } else {
                        (*grid).comp_row + (*grid).comp_height
                    }) as Integer,
                    (*grid).comp_col as Integer,
                    col as Integer,
                );
            }
            if col + width < (*grid).comp_col + (*grid).comp_width {
                compose_area(
                    (if row > (*grid).comp_row {
                        row
                    } else {
                        (*grid).comp_row
                    }) as Integer,
                    (if row + height < (*grid).comp_row + (*grid).comp_height {
                        row + height
                    } else {
                        (*grid).comp_row + (*grid).comp_height
                    }) as Integer,
                    (col + width) as Integer,
                    ((*grid).comp_col + (*grid).comp_width) as Integer,
                );
            }
            compose_area(
                (row + height) as Integer,
                ((*grid).comp_row + (*grid).comp_height) as Integer,
                (*grid).comp_col as Integer,
                ((*grid).comp_col + (*grid).comp_width) as Integer,
            );
            (*grid).comp_disabled = false_0 != 0;
        }
        (*grid).comp_row = row;
        (*grid).comp_col = col;
    } else {
        moved = true_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < (*layers.ptr()).size {
            if *(*layers.ptr()).items.offset(i as isize) == grid {
                abort();
            }
            i = i.wrapping_add(1);
        }
        let mut insert_at: size_t = (*layers.ptr()).size;
        while insert_at > 0 as size_t
            && (**(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize))
            .zindex
                > (*grid).zindex
        {
            insert_at = insert_at.wrapping_sub(1);
        }
        if !(*curwin.ptr()).is_null()
            && *(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize)
                == &raw mut (*curwin.get()).w_grid_alloc
            && (**(*layers.ptr())
                .items
                .offset(insert_at.wrapping_sub(1 as size_t) as isize))
            .zindex
                == (*grid).zindex
            && !on_top
        {
            insert_at = insert_at.wrapping_sub(1);
        }
        if (*layers.ptr()).size == (*layers.ptr()).capacity {
            (*layers.ptr()).capacity = if (*layers.ptr()).capacity != 0 {
                (*layers.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*layers.ptr()).items = xrealloc(
                (*layers.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<*mut ScreenGrid>().wrapping_mul((*layers.ptr()).capacity),
            ) as *mut *mut ScreenGrid;
        } else {
        };
        (*layers.ptr()).size = (*layers.ptr()).size.wrapping_add(1);
        let mut i_0: size_t = (*layers.ptr()).size.wrapping_sub(1 as size_t);
        while i_0 > insert_at {
            *(*layers.ptr()).items.offset(i_0 as isize) = *(*layers.ptr())
                .items
                .offset(i_0.wrapping_sub(1 as size_t) as isize);
            (**(*layers.ptr()).items.offset(i_0 as isize)).comp_index = i_0;
            (**(*layers.ptr()).items.offset(i_0 as isize)).pending_comp_index_update = true_0 != 0;
            i_0 = i_0.wrapping_sub(1);
        }
        *(*layers.ptr()).items.offset(insert_at as isize) = grid;
        (*grid).comp_row = row;
        (*grid).comp_col = col;
        (*grid).comp_index = insert_at;
        (*grid).pending_comp_index_update = true_0 != 0;
    }
    (*grid).comp_height = height;
    (*grid).comp_width = width;
    if moved as ::core::ffi::c_int != 0
        && valid as ::core::ffi::c_int != 0
        && ui_comp_should_draw() as ::core::ffi::c_int != 0
    {
        compose_area(
            (*grid).comp_row as Integer,
            ((*grid).comp_row + (*grid).rows) as Integer,
            (*grid).comp_col as Integer,
            ((*grid).comp_col + (*grid).cols) as Integer,
        );
    }
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_remove_grid(mut grid: *mut ScreenGrid) {
    '_c2rust_label: {
        if grid != default_grid.ptr() {
        } else {
            __assert_fail(
                b"grid != &default_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                217 as ::core::ffi::c_uint,
                b"void ui_comp_remove_grid(ScreenGrid *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*grid).comp_index == 0 as size_t {
        return;
    }
    if curgrid.get() == grid {
        curgrid.set(default_grid.ptr());
    }
    let mut i: size_t = (*grid).comp_index;
    while i < (*layers.ptr()).size.wrapping_sub(1 as size_t) {
        *(*layers.ptr()).items.offset(i as isize) = *(*layers.ptr())
            .items
            .offset(i.wrapping_add(1 as size_t) as isize);
        (**(*layers.ptr()).items.offset(i as isize)).comp_index = i;
        (**(*layers.ptr()).items.offset(i as isize)).pending_comp_index_update = true_0 != 0;
        i = i.wrapping_add(1);
    }
    (*layers.ptr()).size = (*layers.ptr()).size.wrapping_sub(1);
    (*grid).comp_index = 0 as size_t;
    (*grid).pending_comp_index_update = true_0 != 0;
    ui_comp_compose_grid(grid);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_set_grid(mut handle: handle_T) -> bool {
    if (*curgrid.get()).handle == handle {
        return true_0 != 0;
    }
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut i: size_t = 0 as size_t;
    while i < (*layers.ptr()).size {
        if (**(*layers.ptr()).items.offset(i as isize)).handle == handle {
            grid = *(*layers.ptr()).items.offset(i as isize);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !grid.is_null() {
        curgrid.set(grid);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_raise_grid(mut grid: *mut ScreenGrid, mut new_index: size_t) {
    let mut old_index: size_t = (*grid).comp_index;
    let mut i: size_t = old_index;
    while i < new_index {
        *(*layers.ptr()).items.offset(i as isize) = *(*layers.ptr())
            .items
            .offset(i.wrapping_add(1 as size_t) as isize);
        (**(*layers.ptr()).items.offset(i as isize)).comp_index = i;
        (**(*layers.ptr()).items.offset(i as isize)).pending_comp_index_update = true_0 != 0;
        i = i.wrapping_add(1);
    }
    *(*layers.ptr()).items.offset(new_index as isize) = grid;
    (*grid).comp_index = new_index;
    (*grid).pending_comp_index_update = true_0 != 0;
    let mut i_0: size_t = old_index;
    while i_0 < new_index {
        let mut grid2: *mut ScreenGrid = *(*layers.ptr()).items.offset(i_0 as isize);
        let mut startcol: ::core::ffi::c_int = if (*grid).comp_col > (*grid2).comp_col {
            (*grid).comp_col
        } else {
            (*grid2).comp_col
        };
        let mut endcol: ::core::ffi::c_int =
            if (*grid).comp_col + (*grid).cols < (*grid2).comp_col + (*grid2).cols {
                (*grid).comp_col + (*grid).cols
            } else {
                (*grid2).comp_col + (*grid2).cols
            };
        compose_area(
            (if (*grid).comp_row > (*grid2).comp_row {
                (*grid).comp_row
            } else {
                (*grid2).comp_row
            }) as Integer,
            (if (*grid).comp_row + (*grid).rows < (*grid2).comp_row + (*grid2).rows {
                (*grid).comp_row + (*grid).rows
            } else {
                (*grid2).comp_row + (*grid2).rows
            }) as Integer,
            startcol as Integer,
            endcol as Integer,
        );
        i_0 = i_0.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_cursor_goto(
    mut grid_handle: Integer,
    mut r: Integer,
    mut c: Integer,
) {
    if !ui_comp_set_grid(grid_handle as handle_T) {
        return;
    }
    let mut cursor_row: ::core::ffi::c_int = (*curgrid.get()).comp_row + r as ::core::ffi::c_int;
    let mut cursor_col: ::core::ffi::c_int = (*curgrid.get()).comp_col + c as ::core::ffi::c_int;
    if curgrid.get() != default_grid.ptr() {
        let mut new_index: size_t = (*layers.ptr()).size.wrapping_sub(1 as size_t);
        while new_index > 1 as size_t
            && (**(*layers.ptr()).items.offset(new_index as isize)).zindex > (*curgrid.get()).zindex
        {
            new_index = new_index.wrapping_sub(1);
        }
        if (*curgrid.get()).comp_index < new_index {
            ui_comp_raise_grid(curgrid.get(), new_index);
        }
    }
    if cursor_col >= (*default_grid.ptr()).cols || cursor_row >= (*default_grid.ptr()).rows {
        return;
    }
    ui_composed_call_grid_cursor_goto(1 as Integer, cursor_row as Integer, cursor_col as Integer);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_mouse_focus(
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) -> *mut ScreenGrid {
    let mut i: ssize_t = (*layers.ptr()).size as ssize_t - 1 as ssize_t;
    while i > 0 as ssize_t {
        let mut grid: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
        if (*grid).mouse_enabled as ::core::ffi::c_int != 0
            && row >= (*grid).comp_row
            && row < (*grid).comp_row + (*grid).rows
            && col >= (*grid).comp_col
            && col < (*grid).comp_col + (*grid).cols
        {
            return grid;
        }
        i -= 1;
    }
    if ui_has(kUIMultigrid) {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            let mut grid_0: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
            if (*grid_0).mouse_enabled as ::core::ffi::c_int != 0
                && row >= (*wp).w_winrow
                && row < (*wp).w_winrow + (*grid_0).rows
                && col >= (*wp).w_wincol
                && col < (*wp).w_wincol + (*grid_0).cols
            {
                return grid_0;
            }
            wp = (*wp).w_next;
        }
    }
    return ::core::ptr::null_mut::<ScreenGrid>();
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_get_grid_at_coord(
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) -> *mut ScreenGrid {
    let mut i: ssize_t = (*layers.ptr()).size as ssize_t - 1 as ssize_t;
    while i > 0 as ssize_t {
        let mut grid: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
        if row >= (*grid).comp_row
            && row < (*grid).comp_row + (*grid).rows
            && col >= (*grid).comp_col
            && col < (*grid).comp_col + (*grid).cols
        {
            return grid;
        }
        i -= 1;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut grid_0: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
        if row >= (*grid_0).comp_row
            && row < (*grid_0).comp_row + (*grid_0).rows
            && col >= (*grid_0).comp_col
            && col < (*grid_0).comp_col + (*grid_0).cols
            && !(*wp).w_config.hide
        {
            return grid_0;
        }
        wp = (*wp).w_next;
    }
    return default_grid.ptr();
}
unsafe extern "C" fn compose_line(
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut flags: LineFlags,
) {
    startcol = if startcol > 0 as Integer {
        startcol
    } else {
        0 as Integer
    };
    let mut skipstart: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skipend: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if startcol > 0 as Integer
        && flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
    {
        startcol -= 1;
        skipstart = 1 as ::core::ffi::c_int;
    }
    if endcol < (*default_grid.ptr()).cols as Integer
        && flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
    {
        endcol += 1;
        skipend = 1 as ::core::ffi::c_int;
    }
    let mut col: ::core::ffi::c_int = startcol as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
    let mut bg_line: *mut schar_T = (*default_grid.ptr()).chars.offset(
        (*(*default_grid.ptr()).line_offset.offset(row as isize)).wrapping_add(startcol as size_t)
            as isize,
    );
    let mut bg_attrs: *mut sattr_T = (*default_grid.ptr()).attrs.offset(
        (*(*default_grid.ptr()).line_offset.offset(row as isize)).wrapping_add(startcol as size_t)
            as isize,
    );
    while (col as Integer) < endcol {
        let mut until: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < (*layers.ptr()).size {
            let mut g: *mut ScreenGrid = *(*layers.ptr()).items.offset(i as isize);
            let mut grid_width: ::core::ffi::c_int = if (*g).cols < (*g).comp_width {
                (*g).cols
            } else {
                (*g).comp_width
            };
            let mut grid_height: ::core::ffi::c_int = if (*g).rows < (*g).comp_height {
                (*g).rows
            } else {
                (*g).comp_height
            };
            if !((*g).comp_row as Integer > row
                || row >= ((*g).comp_row + grid_height) as Integer
                || (*g).comp_disabled as ::core::ffi::c_int != 0)
            {
                if (*g).comp_col <= col && col < (*g).comp_col + grid_width {
                    grid = g;
                    until = (*g).comp_col + grid_width;
                } else if (*g).comp_col > col {
                    until = if until < (*g).comp_col {
                        until
                    } else {
                        (*g).comp_col
                    };
                }
            }
            i = i.wrapping_add(1);
        }
        until = if until < endcol as ::core::ffi::c_int {
            until
        } else {
            endcol as ::core::ffi::c_int
        };
        '_c2rust_label: {
            if !grid.is_null() {
            } else {
                __assert_fail(
                    b"grid != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    408 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_0: {
            if until > col {
            } else {
                __assert_fail(
                    b"until > col\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    409 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_1: {
            if until <= (*default_grid.ptr()).cols {
            } else {
                __assert_fail(
                    b"until <= default_grid.cols\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    410 as ::core::ffi::c_uint,
                    b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut n: size_t = (until - col) as size_t;
        if row == msg_sep_row.get() as Integer && (*grid).comp_index <= (*msg_grid.ptr()).comp_index
        {
            grid = msg_grid.ptr();
            let mut msg_sep_attr: sattr_T = *(*hl_attr_active.ptr())
                .offset(HLF_MSGSEP as ::core::ffi::c_int as isize)
                as sattr_T;
            let mut i_0: ::core::ffi::c_int = col;
            while i_0 < until {
                *(*linebuf.ptr()).offset((i_0 as Integer - startcol) as isize) = msg_sep_char.get();
                *(*attrbuf.ptr()).offset((i_0 as Integer - startcol) as isize) = msg_sep_attr;
                i_0 += 1;
            }
        } else {
            let mut off: size_t = (*(*grid)
                .line_offset
                .offset((row - (*grid).comp_row as Integer) as isize))
            .wrapping_add((col - (*grid).comp_col) as size_t);
            memcpy(
                (*linebuf.ptr()).offset((col as Integer - startcol) as isize)
                    as *mut ::core::ffi::c_void,
                (*grid).chars.offset(off as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<schar_T>()),
            );
            memcpy(
                (*attrbuf.ptr()).offset((col as Integer - startcol) as isize)
                    as *mut ::core::ffi::c_void,
                (*grid).attrs.offset(off as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<sattr_T>()),
            );
            if (*grid).comp_col + (*grid).cols > until
                && *(*grid).chars.offset(off.wrapping_add(n) as isize) == NUL as schar_T
            {
                *(*linebuf.ptr())
                    .offset(((until - 1 as ::core::ffi::c_int) as Integer - startcol) as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
                if col as Integer == startcol && n == 1 as size_t {
                    skipstart = 0 as ::core::ffi::c_int;
                }
            }
        }
        if (*grid).blending {
            let mut width: ::core::ffi::c_int = 0;
            let mut i_1: ::core::ffi::c_int = col - startcol as ::core::ffi::c_int;
            while (i_1 as Integer) < until as Integer - startcol {
                width = 1 as ::core::ffi::c_int;
                let mut thru: bool = (*(*linebuf.ptr()).offset(i_1 as isize)
                    == ' ' as ::core::ffi::c_int as schar_T
                    || *(*linebuf.ptr()).offset(i_1 as isize)
                        == schar_from_char('⠀' as ::core::ffi::c_int))
                    && *bg_line.offset(i_1 as isize) != NUL as schar_T;
                if ((i_1 + 1 as ::core::ffi::c_int) as Integer) < endcol - startcol
                    && *bg_line.offset((i_1 + 1 as ::core::ffi::c_int) as isize) == NUL as schar_T
                {
                    width = 2 as ::core::ffi::c_int;
                    thru = thru as ::core::ffi::c_int
                        & (*(*linebuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                            == ' ' as ::core::ffi::c_int as schar_T
                            || *(*linebuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                == schar_from_char('⠀' as ::core::ffi::c_int))
                            as ::core::ffi::c_int
                        != 0;
                }
                *(*attrbuf.ptr()).offset(i_1 as isize) = hl_blend_attrs(
                    *bg_attrs.offset(i_1 as isize) as ::core::ffi::c_int,
                    *(*attrbuf.ptr()).offset(i_1 as isize) as ::core::ffi::c_int,
                    &raw mut thru,
                ) as sattr_T;
                if width == 2 as ::core::ffi::c_int {
                    *(*attrbuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize) =
                        hl_blend_attrs(
                            *bg_attrs.offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                            *(*attrbuf.ptr()).offset((i_1 + 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int,
                            &raw mut thru,
                        ) as sattr_T;
                }
                if thru {
                    memcpy(
                        (*linebuf.ptr()).offset(i_1 as isize) as *mut ::core::ffi::c_void,
                        bg_line.offset(i_1 as isize) as *const ::core::ffi::c_void,
                        (width as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
                    );
                }
                i_1 += width;
            }
        }
        if *(*linebuf.ptr()).offset((col as Integer - startcol) as isize) == NUL as schar_T {
            *(*linebuf.ptr()).offset((col as Integer - startcol) as isize) =
                ' ' as ::core::ffi::c_int as schar_T;
            if col as Integer == endcol - 1 as Integer {
                skipend = 0 as ::core::ffi::c_int;
            }
        } else if col as Integer == startcol
            && n > 1 as size_t
            && *(*linebuf.ptr()).offset(1 as ::core::ffi::c_int as isize) == NUL as schar_T
        {
            skipstart = 0 as ::core::ffi::c_int;
        }
        col = until;
    }
    if *(*linebuf.ptr()).offset((endcol - startcol - 1 as Integer) as isize) == NUL as schar_T {
        skipend = 0 as ::core::ffi::c_int;
    }
    '_c2rust_label_2: {
        if endcol <= chk_width.get() as Integer {
        } else {
            __assert_fail(
                b"endcol <= chk_width\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                477 as ::core::ffi::c_uint,
                b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_3: {
        if row < chk_height.get() as Integer {
        } else {
            __assert_fail(
                b"row < chk_height\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                478 as ::core::ffi::c_uint,
                b"void compose_line(Integer, Integer, Integer, LineFlags)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !(!grid.is_null()
        && (grid == default_grid.ptr()
            || (*grid).comp_col == 0 as ::core::ffi::c_int && (*grid).cols == Columns.get()))
    {
        flags = (flags as ::core::ffi::c_int & !(kLineFlagWrap as ::core::ffi::c_int)) as LineFlags;
    }
    let mut i_2: ::core::ffi::c_int = skipstart;
    while (i_2 as Integer) < endcol - skipend as Integer - startcol {
        if *(*attrbuf.ptr()).offset(i_2 as isize) < 0 as sattr_T {
            if rdb_flags.get() & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                abort();
            } else {
                *(*attrbuf.ptr()).offset(i_2 as isize) = 0 as ::core::ffi::c_int as sattr_T;
            }
        }
        i_2 += 1;
    }
    ui_composed_call_raw_line(
        1 as Integer,
        row,
        startcol + skipstart as Integer,
        endcol - skipend as Integer,
        endcol - skipend as Integer,
        0 as Integer,
        flags,
        (linebuf.get() as *const schar_T).offset(skipstart as isize),
        (attrbuf.get() as *const sattr_T).offset(skipstart as isize),
    );
}
unsafe extern "C" fn compose_debug(
    mut startrow: Integer,
    mut endrow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut syn_id: ::core::ffi::c_int,
    mut delay: bool,
) {
    if rdb_flags.get() & kOptRdbFlagCompositor as ::core::ffi::c_int as ::core::ffi::c_uint == 0
        || startcol >= endcol
    {
        return;
    }
    endrow = if endrow < (*default_grid.ptr()).rows as Integer {
        endrow
    } else {
        (*default_grid.ptr()).rows as Integer
    };
    endcol = if endcol < (*default_grid.ptr()).cols as Integer {
        endcol
    } else {
        (*default_grid.ptr()).cols as Integer
    };
    let mut attr: ::core::ffi::c_int = syn_id2attr(syn_id);
    if delay {
        debug_delay(endrow - startrow);
    }
    let mut row: ::core::ffi::c_int = startrow as ::core::ffi::c_int;
    while (row as Integer) < endrow {
        ui_composed_call_raw_line(
            1 as Integer,
            row as Integer,
            startcol,
            startcol,
            endcol,
            attr as Integer,
            false_0,
            linebuf.get() as *const schar_T,
            attrbuf.get() as *const sattr_T,
        );
        row += 1;
    }
    if delay {
        debug_delay(endrow - startrow);
    }
}
unsafe extern "C" fn debug_delay(mut lines: Integer) {
    ui_call_flush();
    let mut wd: uint64_t = llabs(p_wd.get() as ::core::ffi::c_longlong) as uint64_t;
    let mut factor: uint64_t = (if (if lines < 5 as Integer {
        lines
    } else {
        5 as Integer
    }) > 1 as Integer
    {
        if lines < 5 as Integer {
            lines
        } else {
            5 as Integer
        }
    } else {
        1 as Integer
    }) as uint64_t;
    os_sleep(factor.wrapping_mul(wd));
}
unsafe extern "C" fn compose_area(
    mut startrow: Integer,
    mut endrow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
) {
    compose_debug(
        startrow,
        endrow,
        startcol,
        endcol,
        dbghl_recompose.get(),
        true_0 != 0,
    );
    endrow = if endrow < (*default_grid.ptr()).rows as Integer {
        endrow
    } else {
        (*default_grid.ptr()).rows as Integer
    };
    endcol = if endcol < (*default_grid.ptr()).cols as Integer {
        endcol
    } else {
        (*default_grid.ptr()).cols as Integer
    };
    if endcol <= startcol {
        return;
    }
    let mut r: ::core::ffi::c_int = startrow as ::core::ffi::c_int;
    while (r as Integer) < endrow {
        compose_line(
            r as Integer,
            startcol,
            endcol,
            kLineFlagInvalid as ::core::ffi::c_int,
        );
        r += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_compose_grid(mut grid: *mut ScreenGrid) {
    if ui_comp_should_draw() {
        compose_area(
            (*grid).comp_row as Integer,
            ((*grid).comp_row + (*grid).rows) as Integer,
            (*grid).comp_col as Integer,
            ((*grid).comp_col + (*grid).cols) as Integer,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_raw_line(
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    row += (*curgrid.get()).comp_row as Integer;
    startcol += (*curgrid.get()).comp_col as Integer;
    endcol += (*curgrid.get()).comp_col as Integer;
    clearcol += (*curgrid.get()).comp_col as Integer;
    if curgrid.get() != default_grid.ptr() {
        flags = (flags as ::core::ffi::c_int & !(kLineFlagWrap as ::core::ffi::c_int)) as LineFlags;
    }
    '_c2rust_label: {
        if endcol <= clearcol {
        } else {
            __assert_fail(
                b"endcol <= clearcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_compositor.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                574 as ::core::ffi::c_uint,
                b"void ui_comp_raw_line(Integer, Integer, Integer, Integer, Integer, Integer, LineFlags, const schar_T *, const sattr_T *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if row >= (*default_grid.ptr()).rows as Integer {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_comp_raw_line\0".as_ptr() as *const ::core::ffi::c_char,
            580 as ::core::ffi::c_int,
            true_0 != 0,
            b"compositor: invalid row %ld on grid %ld\0".as_ptr() as *const ::core::ffi::c_char,
            row,
            grid,
        );
        return;
    }
    if clearcol > (*default_grid.ptr()).cols as Integer {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_comp_raw_line\0".as_ptr() as *const ::core::ffi::c_char,
            585 as ::core::ffi::c_int,
            true_0 != 0,
            b"compositor: invalid last column %ld on grid %ld\0".as_ptr()
                as *const ::core::ffi::c_char,
            clearcol,
            grid,
        );
        if startcol >= (*default_grid.ptr()).cols as Integer {
            return;
        }
        clearcol = (*default_grid.ptr()).cols as Integer;
        endcol = if endcol < clearcol { endcol } else { clearcol };
    }
    let mut covered: bool = curgrid_covered_above(row as ::core::ffi::c_int);
    if flags as ::core::ffi::c_int & kLineFlagInvalid as ::core::ffi::c_int != 0
        || covered as ::core::ffi::c_int != 0
        || (*curgrid.get()).blending as ::core::ffi::c_int != 0
    {
        compose_debug(
            row,
            row + 1 as Integer,
            startcol,
            clearcol,
            dbghl_composed.get(),
            true_0 != 0,
        );
        compose_line(row, startcol, clearcol, flags);
    } else {
        compose_debug(
            row,
            row + 1 as Integer,
            startcol,
            endcol,
            dbghl_normal.get(),
            endcol >= clearcol,
        );
        compose_debug(
            row,
            row + 1 as Integer,
            endcol,
            clearcol,
            dbghl_clear.get(),
            true_0 != 0,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as Integer) < endcol - startcol {
            '_c2rust_label_0: {
                if *attrs.offset(i as isize) >= 0 as sattr_T {
                } else {
                    __assert_fail(
                        b"attrs[i] >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ui_compositor.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        604 as ::core::ffi::c_uint,
                        b"void ui_comp_raw_line(Integer, Integer, Integer, Integer, Integer, Integer, LineFlags, const schar_T *, const sattr_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            i += 1;
        }
        ui_composed_call_raw_line(
            1 as Integer,
            row,
            startcol,
            endcol,
            clearcol,
            clearattr,
            flags,
            chunk,
            attrs,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_set_screen_valid(mut valid: bool) -> bool {
    let mut old_val: bool = valid_screen.get();
    valid_screen.set(valid);
    if !valid {
        msg_sep_row.set(-1 as ::core::ffi::c_int);
    }
    return old_val;
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_msg_set_pos(
    mut _grid: Integer,
    mut row: Integer,
    mut scrolled: Boolean,
    mut sep_char: String_0,
    mut _zindex: Integer,
    mut _compindex: Integer,
) {
    (*msg_grid.ptr()).pending_comp_index_update = true_0 != 0;
    (*msg_grid.ptr()).comp_row = row as ::core::ffi::c_int;
    if scrolled as ::core::ffi::c_int != 0 && row > 0 as Integer {
        msg_sep_row.set(row as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if !sep_char.data.is_null() {
            msg_sep_char.set(schar_from_buf(sep_char.data, sep_char.size));
        }
    } else {
        msg_sep_row.set(-1 as ::core::ffi::c_int);
    }
    if row > msg_current_row.get() as Integer && ui_comp_should_draw() as ::core::ffi::c_int != 0 {
        compose_area(
            (if msg_current_row.get() - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                msg_current_row.get() - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as Integer,
            row,
            0 as Integer,
            (*default_grid.ptr()).cols as Integer,
        );
    } else if row < msg_current_row.get() as Integer
        && ui_comp_should_draw() as ::core::ffi::c_int != 0
        && (msg_current_row.get() < Rows.get()
            || scrolled as ::core::ffi::c_int != 0 && !msg_was_scrolled.get())
    {
        let mut delta: ::core::ffi::c_int = msg_current_row.get() - row as ::core::ffi::c_int;
        if (*msg_grid.ptr()).blending {
            let mut first_row: ::core::ffi::c_int = if row as ::core::ffi::c_int
                - (if scrolled as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                > 0 as ::core::ffi::c_int
            {
                row as ::core::ffi::c_int
                    - (if scrolled as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
            } else {
                0 as ::core::ffi::c_int
            };
            compose_area(
                first_row as Integer,
                (Rows.get() - delta) as Integer,
                0 as Integer,
                Columns.get() as Integer,
            );
        } else {
            let mut first_row_0: ::core::ffi::c_int = if row as ::core::ffi::c_int
                - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                > 0 as ::core::ffi::c_int
            {
                row as ::core::ffi::c_int
                    - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
            } else {
                0 as ::core::ffi::c_int
            };
            ui_composed_call_grid_scroll(
                1 as Integer,
                first_row_0 as Integer,
                Rows.get() as Integer,
                0 as Integer,
                Columns.get() as Integer,
                delta as Integer,
                0 as Integer,
            );
            if scrolled as ::core::ffi::c_int != 0 && !msg_was_scrolled.get() && row > 0 as Integer
            {
                compose_area(
                    row - 1 as Integer,
                    row,
                    0 as Integer,
                    Columns.get() as Integer,
                );
            }
        }
    }
    msg_current_row.set(row as ::core::ffi::c_int);
    msg_was_scrolled.set(scrolled as bool);
}
unsafe extern "C" fn curgrid_covered_above(mut row: ::core::ffi::c_int) -> bool {
    let mut above_msg: bool = *(*layers.ptr())
        .items
        .offset((*layers.ptr()).size.wrapping_sub(1 as size_t) as isize)
        == msg_grid.ptr()
        && row
            < msg_current_row.get()
                - (if msg_was_scrolled.get() as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
    return (*layers.ptr()).size.wrapping_sub(
        (if above_msg as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as size_t,
    ) > (*curgrid.get()).comp_index.wrapping_add(1 as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_scroll(
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    top += (*curgrid.get()).comp_row as Integer;
    bot += (*curgrid.get()).comp_row as Integer;
    left += (*curgrid.get()).comp_col as Integer;
    right += (*curgrid.get()).comp_col as Integer;
    let mut covered: bool = curgrid_covered_above(
        (bot - (if rows > 0 as Integer {
            rows
        } else {
            0 as Integer
        })) as ::core::ffi::c_int,
    );
    if covered as ::core::ffi::c_int != 0 || (*curgrid.get()).blending as ::core::ffi::c_int != 0 {
        compose_debug(top, bot, left, right, dbghl_recompose.get(), true_0 != 0);
        let mut r: ::core::ffi::c_int = (top
            + (if -rows > 0 as Integer {
                -rows
            } else {
                0 as Integer
            })) as ::core::ffi::c_int;
        while (r as Integer)
            < bot
                - (if rows > 0 as Integer {
                    rows
                } else {
                    0 as Integer
                })
        {
            if *(*curgrid.get()).attrs.offset(
                (*(*curgrid.get())
                    .line_offset
                    .offset((r - (*curgrid.get()).comp_row) as isize))
                .wrapping_add(left as size_t)
                .wrapping_sub((*curgrid.get()).comp_col as size_t) as isize,
            ) >= 0 as sattr_T
            {
                compose_line(r as Integer, left, right, 0 as LineFlags);
            }
            r += 1;
        }
    } else {
        ui_composed_call_grid_scroll(1 as Integer, top, bot, left, right, rows, cols);
        if rdb_flags.get() & kOptRdbFlagCompositor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            debug_delay(2 as Integer);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_comp_grid_resize(
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    if grid == 1 as Integer {
        ui_composed_call_grid_resize(1 as Integer, width, height);
        chk_width.set(width as ::core::ffi::c_int);
        chk_height.set(height as ::core::ffi::c_int);
        let mut new_bufsize: size_t = width as size_t;
        if bufsize.get() != new_bufsize {
            xfree(linebuf.get() as *mut ::core::ffi::c_void);
            xfree(attrbuf.get() as *mut ::core::ffi::c_void);
            linebuf.set(
                xmalloc(new_bufsize.wrapping_mul(::core::mem::size_of::<schar_T>()))
                    as *mut schar_T,
            );
            attrbuf.set(
                xmalloc(new_bufsize.wrapping_mul(::core::mem::size_of::<sattr_T>()))
                    as *mut sattr_T,
            );
            bufsize.set(new_bufsize);
        }
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
