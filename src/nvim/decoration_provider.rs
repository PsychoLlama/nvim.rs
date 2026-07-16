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
    fn describe_ns(ns_id: NS, unknown: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn api_free_object(value: Object);
    fn api_free_array(value: Array);
    fn api_clear_error(value: *mut Error);
    fn api_object_to_bool(
        obj: Object,
        what: *const ::core::ffi::c_char,
        nil_value: bool,
        err: *mut Error,
    ) -> bool;
    static mut decor_state: DecorState;
    fn decor_check_to_be_deleted();
    static mut textlock: ::core::ffi::c_int;
    static mut display_tick: disptick_T;
    static mut ns_hl_active: NS;
    fn hl_check_ns() -> bool;
    fn api_free_luaref(ref_0: LuaRef);
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn msg_schedule_semsg_multiline(fmt: *const ::core::ffi::c_char, ...);
    fn validate_botline_win(wp: *mut win_T);
}
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type time_t = __time_t;
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
pub type NS = handle_T;
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
    pub data: C2Rust_Unnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
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
pub type DecorPriorityInternal = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorSignHighlight {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub text: [schar_T; 2],
    pub sign_name: *mut ::core::ffi::c_char,
    pub sign_add_id: ::core::ffi::c_int,
    pub number_hl_id: ::core::ffi::c_int,
    pub line_hl_id: ::core::ffi::c_int,
    pub cursorline_hl_id: ::core::ffi::c_int,
    pub next: uint32_t,
    pub url: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorProvider {
    pub ns_id: NS,
    pub state: C2Rust_Unnamed_13,
    pub win_skip_row: ::core::ffi::c_int,
    pub win_skip_col: ::core::ffi::c_int,
    pub redraw_start: LuaRef,
    pub redraw_buf: LuaRef,
    pub redraw_win: LuaRef,
    pub redraw_line: LuaRef,
    pub redraw_range: LuaRef,
    pub redraw_end: LuaRef,
    pub hl_def: LuaRef,
    pub spell_nav: LuaRef,
    pub conceal_line: LuaRef,
    pub hl_valid: ::core::ffi::c_int,
    pub hl_cached: bool,
    pub error_count: uint8_t,
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kDecorProviderDisabled: C2Rust_Unnamed_13 = 4;
pub const kDecorProviderRedrawDisabled: C2Rust_Unnamed_13 = 3;
pub const kDecorProviderWinDisabled: C2Rust_Unnamed_13 = 2;
pub const kDecorProviderActive: C2Rust_Unnamed_13 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_14; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
pub type DecorRangeKind = uint8_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorRange {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub end_row: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
    pub ordering: ::core::ffi::c_int,
    pub priority_internal: DecorPriorityInternal,
    pub owned: bool,
    pub kind: DecorRangeKind,
    pub data: C2Rust_Unnamed_15,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_15 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub ns_id: uint32_t,
    pub mark_id: uint32_t,
    pub pos: VirtTextPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorRangeSlot {
    pub range: DecorRange,
    pub next_free_i: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorState {
    pub itr: [MarkTreeIter; 1],
    pub slots: C2Rust_Unnamed_18,
    pub ranges_i: C2Rust_Unnamed_17,
    pub current_end: ::core::ffi::c_int,
    pub future_begin: ::core::ffi::c_int,
    pub free_slot_i: ::core::ffi::c_int,
    pub new_range_ordering: ::core::ffi::c_int,
    pub win: *mut win_T,
    pub top_row: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub col_last: ::core::ffi::c_int,
    pub current: ::core::ffi::c_int,
    pub eol_col: ::core::ffi::c_int,
    pub conceal: ::core::ffi::c_int,
    pub conceal_char: schar_T,
    pub conceal_attr: ::core::ffi::c_int,
    pub spell: TriState,
    pub running_decor_provider: bool,
    pub itr_valid: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_17 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_18 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorProvider,
}
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const CB_MAX_ERROR: C2Rust_Unnamed_20 = 3;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
static mut decor_providers: C2Rust_Unnamed_19 = C2Rust_Unnamed_19 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<DecorProvider>(),
};
unsafe extern "C" fn decor_provider_error(
    mut provider: *mut DecorProvider,
    mut name: *const ::core::ffi::c_char,
    mut msg: *const ::core::ffi::c_char,
) {
    let mut ns: *const ::core::ffi::c_char = describe_ns(
        (*provider).ns_id,
        b"(UNKNOWN PLUGIN)\0".as_ptr() as *const ::core::ffi::c_char,
    );
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"decor_provider_error\0".as_ptr() as *const ::core::ffi::c_char,
        29 as ::core::ffi::c_int,
        true_0 != 0,
        b"Error in decoration provider \"%s\" (ns=%s):\n%s\0".as_ptr()
            as *const ::core::ffi::c_char,
        name,
        ns,
        msg,
    );
    msg_schedule_semsg_multiline(
        b"Decoration provider \"%s\" (ns=%s):\n%s\0".as_ptr() as *const ::core::ffi::c_char,
        name,
        ns,
        msg,
    );
}
unsafe extern "C" fn decor_provider_invoke(
    mut provider_idx: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut ref_0: LuaRef,
    mut args: Array,
    mut default_true: bool,
    mut res: *mut Array,
) -> bool {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    textlock += 1;
    let mut ret: Object = nlua_call_ref(
        ref_0,
        name,
        args,
        (if !res.is_null() {
            kRetMulti as ::core::ffi::c_int
        } else {
            kRetNilBool as ::core::ffi::c_int
        }) as LuaRetMode,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    textlock -= 1;
    let mut provider: *mut DecorProvider = decor_providers.items.offset(provider_idx as isize);
    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        (*provider).error_count = 0 as uint8_t;
        if !res.is_null() {
            '_c2rust_label: {
                if ret.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"ret.type == kObjectTypeArray\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/decoration_provider.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        50 as ::core::ffi::c_uint,
                        b"_Bool decor_provider_invoke(int, const char *, LuaRef, Array, _Bool, Array *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            *res = ret.data.array;
            return true_0 != 0;
        } else if api_object_to_bool(
            ret,
            b"provider %s retval\0".as_ptr() as *const ::core::ffi::c_char,
            default_true,
            &raw mut err,
        ) {
            return true_0 != 0;
        }
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
        && ((*provider).error_count as ::core::ffi::c_int) < CB_MAX_ERROR as ::core::ffi::c_int
    {
        decor_provider_error(provider, name, err.msg);
        (*provider).error_count = (*provider).error_count.wrapping_add(1);
        if (*provider).error_count as ::core::ffi::c_int >= CB_MAX_ERROR as ::core::ffi::c_int {
            (*provider).state = kDecorProviderDisabled;
        }
    }
    api_clear_error(&raw mut err);
    api_free_object(ret);
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_spell(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
) {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).spell_nav != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 6] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 6];
            args.capacity = 6 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh0 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh1 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh1 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh2 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: start_row as Integer,
                },
            };
            let c2rust_fresh3 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: start_col as Integer,
                },
            };
            let c2rust_fresh4 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh4 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: end_row as Integer,
                },
            };
            let c2rust_fresh5 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: end_col as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"spell\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).spell_nav,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_conceal_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
) -> bool {
    let mut keys: size_t = (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys;
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).conceal_line != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 4] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 4];
            args.capacity = 4 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh6 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh7 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh7 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh8 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh8 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: row as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"conceal_line\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).conceal_line,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
    return (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys > keys;
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_start() {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_start != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh9 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh9 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick as ::core::ffi::c_int as Integer,
                },
            };
            let mut active: bool = decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"start\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_start,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
            (*decor_providers.items.offset(i as isize)).state =
                (if active as ::core::ffi::c_int != 0 {
                    kDecorProviderActive as ::core::ffi::c_int
                } else {
                    kDecorProviderRedrawDisabled as ::core::ffi::c_int
                }) as C2Rust_Unnamed_13;
        } else if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*decor_providers.items.offset(i as isize)).state = kDecorProviderActive;
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_win(mut wp: *mut win_T) {
    '_c2rust_label: {
        if decor_state.current_end == 0 as ::core::ffi::c_int
            && decor_state.future_begin == decor_state.ranges_i.size as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"decor_state.current_end == 0 && decor_state.future_begin == (int)kv_size(decor_state.ranges_i)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/decoration_provider.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                139 as ::core::ffi::c_uint,
                b"void decor_providers_invoke_win(win_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if decor_providers.size > 0 as size_t {
        validate_botline_win(wp);
    }
    let mut botline: linenr_T = if (*wp).w_botline < (*(*wp).w_buffer).b_ml.ml_line_count {
        (*wp).w_botline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderWinDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*p).state = kDecorProviderActive;
        }
        (*p).win_skip_row = 0 as ::core::ffi::c_int;
        (*p).win_skip_col = 0 as ::core::ffi::c_int;
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_win != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 4] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 4];
            args.capacity = 4 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh10 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh10 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh11 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh11 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh12 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh12 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: ((*wp).w_topline - 1 as linenr_T) as Integer,
                },
            };
            let c2rust_fresh13 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh13 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: (botline - 1 as linenr_T) as Integer,
                },
            };
            if !decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"win\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_win,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            ) {
                (*decor_providers.items.offset(i as isize)).state = kDecorProviderWinDisabled;
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
) {
    decor_state.running_decor_provider = true_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_line != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 3] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 3];
            args.capacity = 3 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh14 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh14 as isize) = object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed_12 {
                    integer: (*wp).handle as Integer,
                },
            };
            let c2rust_fresh15 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh15 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*(*wp).w_buffer).handle as Integer,
                },
            };
            let c2rust_fresh16 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh16 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: row as Integer,
                },
            };
            if !decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"line\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_line,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            ) {
                (*decor_providers.items.offset(i as isize)).state = kDecorProviderWinDisabled;
            }
            hl_check_ns();
        }
        i = i.wrapping_add(1);
    }
    decor_state.running_decor_provider = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_range(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
) {
    decor_state.running_decor_provider = true_0 != 0;
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_range != LUA_NOREF
        {
            if !((*p).win_skip_row > end_row
                || (*p).win_skip_row == end_row && (*p).win_skip_col >= end_col)
            {
                let mut args: Array = ARRAY_DICT_INIT;
                let mut args__items: [Object; 6] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_12 { boolean: false },
                }; 6];
                args.capacity = 6 as size_t;
                args.items = &raw mut args__items as *mut Object;
                let c2rust_fresh17 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh17 as isize) = object {
                    type_0: kObjectTypeWindow,
                    data: C2Rust_Unnamed_12 {
                        integer: (*wp).handle as Integer,
                    },
                };
                let c2rust_fresh18 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh18 as isize) = object {
                    type_0: kObjectTypeBuffer,
                    data: C2Rust_Unnamed_12 {
                        integer: (*(*wp).w_buffer).handle as Integer,
                    },
                };
                let c2rust_fresh19 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh19 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: start_row as Integer,
                    },
                };
                let c2rust_fresh20 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh20 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: start_col as Integer,
                    },
                };
                let c2rust_fresh21 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh21 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: end_row as Integer,
                    },
                };
                let c2rust_fresh22 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh22 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_12 {
                        integer: end_col as Integer,
                    },
                };
                let mut res: Array = ARRAY_DICT_INIT;
                let mut status: bool = decor_provider_invoke(
                    i as ::core::ffi::c_int,
                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                    (*p).redraw_range,
                    args,
                    true_0 != 0,
                    &raw mut res,
                );
                p = decor_providers.items.offset(i as isize);
                if !status {
                    (*p).state = kDecorProviderWinDisabled;
                } else if res.size >= 1 as size_t {
                    let mut first: Object = *res.items.offset(0 as ::core::ffi::c_int as isize);
                    if first.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if first.data.boolean as ::core::ffi::c_int == false_0 {
                            (*p).state = kDecorProviderWinDisabled;
                        }
                    } else if first.type_0 as ::core::ffi::c_uint
                        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut row: Integer = first.data.integer;
                        let mut col: Integer = 0 as Integer;
                        if res.size >= 2 as size_t {
                            let mut second: Object =
                                *res.items.offset(1 as ::core::ffi::c_int as isize);
                            if second.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                col = second.data.integer;
                            }
                        }
                        (*p).win_skip_row = row as ::core::ffi::c_int;
                        (*p).win_skip_col = col as ::core::ffi::c_int;
                    }
                }
                api_free_array(res);
                hl_check_ns();
            }
        }
        i = i.wrapping_add(1);
    }
    decor_state.running_decor_provider = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_buf(mut buf: *mut buf_T) {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            == kDecorProviderActive as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_buf != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh23 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh23 as isize) = object {
                type_0: kObjectTypeBuffer,
                data: C2Rust_Unnamed_12 {
                    integer: (*buf).handle as Integer,
                },
            };
            let c2rust_fresh24 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh24 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick as int64_t,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_buf,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_providers_invoke_end() {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).state as ::core::ffi::c_uint
            != kDecorProviderDisabled as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*p).redraw_end != LUA_NOREF
        {
            let mut args: Array = ARRAY_DICT_INIT;
            let mut args__items: [Object; 1] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_12 { boolean: false },
            }; 1];
            args.capacity = 1 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh25 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh25 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_12 {
                    integer: display_tick as ::core::ffi::c_int as Integer,
                },
            };
            decor_provider_invoke(
                i as ::core::ffi::c_int,
                b"end\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).redraw_end,
                args,
                true_0 != 0,
                ::core::ptr::null_mut::<Array>(),
            );
        }
        i = i.wrapping_add(1);
    }
    decor_check_to_be_deleted();
}
#[no_mangle]
pub unsafe extern "C" fn decor_provider_invalidate_hl() {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        (*decor_providers.items.offset(i as isize)).hl_cached = false_0 != 0;
        i = i.wrapping_add(1);
    }
    if ns_hl_active != 0 {
        ns_hl_active = -1 as ::core::ffi::c_int as NS;
        hl_check_ns();
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_decor_provider(mut ns_id: NS, mut force: bool) -> *mut DecorProvider {
    '_c2rust_label: {
        if ns_id > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"ns_id > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/decoration_provider.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                305 as ::core::ffi::c_uint,
                b"DecorProvider *get_decor_provider(NS, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: size_t = decor_providers.size;
    let mut i: size_t = 0 as size_t;
    while i < len {
        let mut p: *mut DecorProvider = decor_providers.items.offset(i as isize);
        if (*p).ns_id == ns_id {
            return p;
        }
        i = i.wrapping_add(1);
    }
    if !force {
        return ::core::ptr::null_mut::<DecorProvider>();
    }
    if decor_providers.capacity <= len {
        decor_providers.size = len.wrapping_add(1 as size_t);
        decor_providers.capacity = decor_providers.size;
        decor_providers.capacity = decor_providers.capacity.wrapping_sub(1);
        decor_providers.capacity |= decor_providers.capacity >> 1 as ::core::ffi::c_int;
        decor_providers.capacity |= decor_providers.capacity >> 2 as ::core::ffi::c_int;
        decor_providers.capacity |= decor_providers.capacity >> 4 as ::core::ffi::c_int;
        decor_providers.capacity |= decor_providers.capacity >> 8 as ::core::ffi::c_int;
        decor_providers.capacity |= decor_providers.capacity >> 16 as ::core::ffi::c_int;
        decor_providers.capacity = decor_providers.capacity.wrapping_add(1);
        decor_providers.capacity = decor_providers.capacity;
        decor_providers.items = xrealloc(
            decor_providers.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<DecorProvider>().wrapping_mul(decor_providers.capacity),
        ) as *mut DecorProvider;
    } else {
        if decor_providers.size <= len {
            decor_providers.size = len.wrapping_add(1 as size_t);
        } else {
        };
    };
    let mut item: *mut DecorProvider = decor_providers.items.offset(len as isize);
    *item = DecorProvider {
        ns_id: ns_id,
        state: kDecorProviderDisabled,
        win_skip_row: 0 as ::core::ffi::c_int,
        win_skip_col: 0 as ::core::ffi::c_int,
        redraw_start: LUA_NOREF,
        redraw_buf: LUA_NOREF,
        redraw_win: LUA_NOREF,
        redraw_line: LUA_NOREF,
        redraw_range: LUA_NOREF,
        redraw_end: LUA_NOREF,
        hl_def: LUA_NOREF,
        spell_nav: LUA_NOREF,
        conceal_line: -1 as LuaRef,
        hl_valid: false_0,
        hl_cached: false_0 != 0,
        error_count: 0 as uint8_t,
    };
    return item;
}
#[no_mangle]
pub unsafe extern "C" fn decor_provider_clear(mut p: *mut DecorProvider) {
    if p.is_null() {
        return;
    }
    if (*p).redraw_start != LUA_NOREF {
        api_free_luaref((*p).redraw_start);
        (*p).redraw_start = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_buf != LUA_NOREF {
        api_free_luaref((*p).redraw_buf);
        (*p).redraw_buf = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_win != LUA_NOREF {
        api_free_luaref((*p).redraw_win);
        (*p).redraw_win = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_line != LUA_NOREF {
        api_free_luaref((*p).redraw_line);
        (*p).redraw_line = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_range != LUA_NOREF {
        api_free_luaref((*p).redraw_range);
        (*p).redraw_range = LUA_NOREF as LuaRef;
    }
    if (*p).redraw_end != LUA_NOREF {
        api_free_luaref((*p).redraw_end);
        (*p).redraw_end = LUA_NOREF as LuaRef;
    }
    if (*p).spell_nav != LUA_NOREF {
        api_free_luaref((*p).spell_nav);
        (*p).spell_nav = LUA_NOREF as LuaRef;
    }
    if (*p).conceal_line != LUA_NOREF {
        api_free_luaref((*p).conceal_line);
        (*p).conceal_line = LUA_NOREF as LuaRef;
    }
    (*p).state = kDecorProviderDisabled;
}
#[no_mangle]
pub unsafe extern "C" fn decor_free_all_mem() {
    let mut i: size_t = 0 as size_t;
    while i < decor_providers.size {
        decor_provider_clear(decor_providers.items.offset(i as isize));
        i = i.wrapping_add(1);
    }
    xfree(decor_providers.items as *mut ::core::ffi::c_void);
    decor_providers.capacity = 0 as size_t;
    decor_providers.size = decor_providers.capacity;
    decor_providers.items = ::core::ptr::null_mut::<DecorProvider>();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
