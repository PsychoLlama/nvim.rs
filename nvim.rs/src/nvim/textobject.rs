extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn inc_cursor() -> ::core::ffi::c_int;
    fn dec_cursor() -> ::core::ffi::c_int;
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn showmode() -> ::core::ffi::c_int;
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn oneleft() -> ::core::ffi::c_int;
    fn do_searchpair(
        spat: *const ::core::ffi::c_char,
        mpat: *const ::core::ffi::c_char,
        epat: *const ::core::ffi::c_char,
        dir: ::core::ffi::c_int,
        skip: *const typval_T,
        flags: ::core::ffi::c_int,
        match_pos: *mut pos_T,
        lnum_stop: linenr_T,
        time_limit: int64_t,
    ) -> ::core::ffi::c_int;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    static mut redraw_cmdline: bool;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut VIsual_select_exclu_adj: bool;
    static mut VIsual_mode: ::core::ffi::c_int;
    fn inindent(extra: ::core::ffi::c_int) -> bool;
    fn setpcmark();
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_class(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_pos(pos: *const pos_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int;
    fn inc(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn incl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn dec(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn decl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn adjust_skipcol();
    fn unadjust_for_sel() -> bool;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_para: *mut ::core::ffi::c_char;
    static mut p_sections: *mut ::core::ffi::c_char;
    static mut p_sel: *mut ::core::ffi::c_char;
    static mut p_ws: ::core::ffi::c_int;
    fn findmatch(oap: *mut oparg_T, initc: ::core::ffi::c_int) -> *mut pos_T;
    fn findmatchlimit(
        oap: *mut oparg_T,
        initc: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        maxtravel: int64_t,
    ) -> *mut pos_T;
    fn linewhite(lnum: linenr_T) -> bool;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_13 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_13 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_13 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_13 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_13 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_13 = 20;
pub const UPD_VALID: C2Rust_Unnamed_13 = 10;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FM_SKIPCOMM: C2Rust_Unnamed_14 = 8;
pub const FM_BLOCKSTOP: C2Rust_Unnamed_14 = 4;
pub const FM_FORWARD: C2Rust_Unnamed_14 = 2;
pub const FM_BACKWARD: C2Rust_Unnamed_14 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
#[inline(always)]
unsafe extern "C" fn ltoreq(mut a: pos_T, mut b: pos_T) -> bool {
    return lt(a, b) as ::core::ffi::c_int != 0 || equalpos(a, b) as ::core::ffi::c_int != 0;
}
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const CPO_ENDOFSENT: ::core::ffi::c_int = 'J' as ::core::ffi::c_int;
pub const CPO_MATCHBSL: ::core::ffi::c_int = 'M' as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn findsent(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut found_dot: bool = false;
    let mut startlnum: ::core::ffi::c_int = 0;
    let mut cpo_J: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut func: Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int> = None;
    let mut noskip: bool = false_0 != 0;
    let mut pos: pos_T = (*curwin).w_cursor;
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        func = Some(incl as unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int)
            as Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int>;
    } else {
        func = Some(decl as unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int)
            as Option<unsafe extern "C" fn(*mut pos_T) -> ::core::ffi::c_int>;
    }
    loop {
        let c2rust_fresh0 = count;
        count = count - 1;
        if c2rust_fresh0 == 0 {
            break;
        }
        let prev_pos: pos_T = pos;
        '_found: {
            if gchar_pos(&raw mut pos) == NUL {
                while Some(func.expect("non-null function pointer"))
                    .expect("non-null function pointer")(&raw mut pos)
                    != -1 as ::core::ffi::c_int
                {
                    if gchar_pos(&raw mut pos) != NUL {
                        break;
                    }
                }
                if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                    break '_found;
                }
            } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                && pos.col == 0 as ::core::ffi::c_int
                && startPS(pos.lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0
            {
                if pos.lnum == (*curbuf).b_ml.ml_line_count {
                    return FAIL;
                }
                pos.lnum += 1;
                break '_found;
            } else if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                decl(&raw mut pos);
            }
            found_dot = false_0 != 0;
            loop {
                c = gchar_pos(&raw mut pos);
                if !(ascii_iswhite(c) as ::core::ffi::c_int != 0
                    || !vim_strchr(b".!?)]\"'\0".as_ptr() as *const ::core::ffi::c_char, c)
                        .is_null())
                {
                    break;
                }
                let mut tpos: pos_T = pos;
                if decl(&raw mut tpos) == -1 as ::core::ffi::c_int
                    || *ml_get(tpos.lnum) as ::core::ffi::c_int == NUL
                        && dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                {
                    break;
                }
                if found_dot {
                    break;
                }
                if !vim_strchr(b".!?\0".as_ptr() as *const ::core::ffi::c_char, c).is_null() {
                    found_dot = true_0 != 0;
                }
                if !vim_strchr(b")]\"'\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                    && vim_strchr(
                        b".!?)]\"'\0".as_ptr() as *const ::core::ffi::c_char,
                        gchar_pos(&raw mut tpos),
                    )
                    .is_null()
                {
                    break;
                }
                decl(&raw mut pos);
            }
            startlnum = pos.lnum as ::core::ffi::c_int;
            cpo_J = !vim_strchr(p_cpo, CPO_ENDOFSENT).is_null();
            loop {
                c = gchar_pos(&raw mut pos);
                if c == NUL
                    || pos.col == 0 as ::core::ffi::c_int
                        && startPS(pos.lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0
                {
                    if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                        && pos.lnum != startlnum as linenr_T
                    {
                        pos.lnum += 1;
                    }
                    break;
                } else {
                    if c == '.' as ::core::ffi::c_int
                        || c == '!' as ::core::ffi::c_int
                        || c == '?' as ::core::ffi::c_int
                    {
                        let mut tpos_0: pos_T = pos;
                        loop {
                            c = inc(&raw mut tpos_0);
                            if c == -1 as ::core::ffi::c_int {
                                break;
                            }
                            c = gchar_pos(&raw mut tpos_0);
                            if vim_strchr(b")]\"'\0".as_ptr() as *const ::core::ffi::c_char, c)
                                .is_null()
                            {
                                break;
                            }
                        }
                        if c == -1 as ::core::ffi::c_int
                            || !cpo_J
                                && (c == ' ' as ::core::ffi::c_int
                                    || c == '\t' as ::core::ffi::c_int)
                            || c == NUL
                            || cpo_J as ::core::ffi::c_int != 0
                                && (c == ' ' as ::core::ffi::c_int
                                    && inc(&raw mut tpos_0) >= 0 as ::core::ffi::c_int
                                    && gchar_pos(&raw mut tpos_0) == ' ' as ::core::ffi::c_int)
                        {
                            pos = tpos_0;
                            if gchar_pos(&raw mut pos) == NUL {
                                inc(&raw mut pos);
                            }
                            break;
                        }
                    }
                    if Some(func.expect("non-null function pointer"))
                        .expect("non-null function pointer")(&raw mut pos)
                        != -1 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if count != 0 {
                        return FAIL;
                    }
                    noskip = true_0 != 0;
                    break;
                }
            }
        }
        while !noskip && {
            c = gchar_pos(&raw mut pos);
            c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int
        } {
            if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                break;
            }
        }
        if !equalpos(prev_pos, pos) {
            continue;
        }
        if Some(func.expect("non-null function pointer")).expect("non-null function pointer")(
            &raw mut pos,
        ) == -1 as ::core::ffi::c_int
        {
            if count != 0 {
                return FAIL;
            }
            break;
        } else {
            count += 1;
        }
    }
    setpcmark();
    (*curwin).w_cursor = pos;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn findpar(
    mut pincl: *mut bool,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut what: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut first: bool = false;
    let mut fold_first: linenr_T = 0;
    let mut fold_last: linenr_T = 0;
    let mut fold_skipped: bool = false;
    let mut curr: linenr_T = (*curwin).w_cursor.lnum;
    loop {
        let c2rust_fresh1 = count;
        count = count - 1;
        if c2rust_fresh1 == 0 {
            break;
        }
        let mut did_skip: bool = false_0 != 0;
        first = true_0 != 0;
        loop {
            if *ml_get(curr) as ::core::ffi::c_int != NUL {
                did_skip = true_0 != 0;
            }
            fold_skipped = false_0 != 0;
            if first as ::core::ffi::c_int != 0
                && hasFolding(curwin, curr, &raw mut fold_first, &raw mut fold_last)
                    as ::core::ffi::c_int
                    != 0
            {
                curr = (if dir > 0 as ::core::ffi::c_int {
                    fold_last
                } else {
                    fold_first
                }) + dir as linenr_T;
                fold_skipped = true_0 != 0;
            }
            if !first
                && did_skip as ::core::ffi::c_int != 0
                && startPS(curr, what, both) as ::core::ffi::c_int != 0
            {
                break;
            }
            if fold_skipped {
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
            }
            curr = (curr as ::core::ffi::c_int + dir) as linenr_T;
            if curr < 1 as linenr_T || curr > (*curbuf).b_ml.ml_line_count {
                if count != 0 {
                    return false_0 != 0;
                }
                curr = (curr as ::core::ffi::c_int - dir) as linenr_T;
                break;
            } else {
                first = false_0 != 0;
            }
        }
    }
    setpcmark();
    if both as ::core::ffi::c_int != 0
        && *ml_get(curr) as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        curr += 1;
    }
    (*curwin).w_cursor.lnum = curr;
    if curr == (*curbuf).b_ml.ml_line_count
        && what != '}' as ::core::ffi::c_int
        && dir == FORWARD as ::core::ffi::c_int
    {
        let mut line: *mut ::core::ffi::c_char = ml_get(curr);
        (*curwin).w_cursor.col = ml_get_len(curr);
        if (*curwin).w_cursor.col != 0 as ::core::ffi::c_int {
            (*curwin).w_cursor.col -= 1;
            (*curwin).w_cursor.col -=
                utf_head_off(line, line.offset((*curwin).w_cursor.col as isize));
            *pincl = true_0 != 0;
        }
    } else {
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return true_0 != 0;
}
unsafe extern "C" fn inmacro(
    mut opt: *mut ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
) -> bool {
    let mut macro_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    macro_0 = opt;
    while *macro_0.offset(0 as ::core::ffi::c_int as isize) != 0 {
        if (*macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            || *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
                && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int))
            && (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                || (*macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *macro_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int)
                    && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int))
        {
            break;
        }
        macro_0 = macro_0.offset(1);
        if *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        macro_0 = macro_0.offset(1);
    }
    return *macro_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
#[no_mangle]
pub unsafe extern "C" fn startPS(
    mut lnum: linenr_T,
    mut para: ::core::ffi::c_int,
    mut both: bool,
) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as uint8_t as ::core::ffi::c_int == para
        || *s as ::core::ffi::c_int == '\u{c}' as ::core::ffi::c_int
        || both as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == '}' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if *s as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        && (inmacro(p_sections, s.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            != 0
            || para == 0
                && inmacro(p_para, s.offset(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    != 0)
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
static mut cls_bigword: bool = false;
unsafe extern "C" fn cls() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = gchar_cursor();
    if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int || c == NUL {
        return 0 as ::core::ffi::c_int;
    }
    c = utf_class(c);
    if c != 0 as ::core::ffi::c_int && cls_bigword as ::core::ffi::c_int != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn fwd_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword = bigword;
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin,
            (*curwin).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin).w_cursor.lnum,
        ) {
            coladvance(curwin, MAXCOL as ::core::ffi::c_int);
        }
        let mut sclass: ::core::ffi::c_int = cls();
        let mut last_line: ::core::ffi::c_int =
            ((*curwin).w_cursor.lnum == (*curbuf).b_ml.ml_line_count) as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = inc_cursor();
        if i == -1 as ::core::ffi::c_int || i >= 1 as ::core::ffi::c_int && last_line != 0 {
            return FAIL;
        }
        if i >= 1 as ::core::ffi::c_int
            && eol as ::core::ffi::c_int != 0
            && count == 0 as ::core::ffi::c_int
        {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = inc_cursor();
                if i == -1 as ::core::ffi::c_int
                    || i >= 1 as ::core::ffi::c_int
                        && eol as ::core::ffi::c_int != 0
                        && count == 0 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                && *get_cursor_line_ptr() as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = inc_cursor();
            if i == -1 as ::core::ffi::c_int
                || i >= 1 as ::core::ffi::c_int
                    && eol as ::core::ffi::c_int != 0
                    && count == 0 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn bck_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword = bigword;
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin,
            (*curwin).w_cursor.lnum,
            &raw mut (*curwin).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        sclass = cls();
        if dec_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if !stop || sclass == cls() || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if dec_cursor() == -1 as ::core::ffi::c_int {
                        return OK;
                    }
                }
                if skip_chars(cls(), BACKWARD as ::core::ffi::c_int) {
                    return OK;
                }
            }
            inc_cursor();
        }
        stop = false_0 != 0;
    }
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn end_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
    mut empty: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword = bigword;
    if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && VIsual_active as ::core::ffi::c_int != 0
        && VIsual_mode == 'v' as ::core::ffi::c_int
        && VIsual_select_exclu_adj as ::core::ffi::c_int != 0
    {
        unadjust_for_sel();
    }
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin,
            (*curwin).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin).w_cursor.lnum,
        ) {
            coladvance(curwin, MAXCOL as ::core::ffi::c_int);
        }
        sclass = cls();
        if inc_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if cls() == sclass && sclass != 0 as ::core::ffi::c_int {
                if skip_chars(sclass, FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            } else if !stop || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if empty as ::core::ffi::c_int != 0
                        && (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if inc_cursor() == -1 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
                if skip_chars(cls(), FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            }
            dec_cursor();
        }
        stop = false_0 != 0;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn bckend_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword = bigword;
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        let mut i: ::core::ffi::c_int = 0;
        let mut sclass: ::core::ffi::c_int = cls();
        i = dec_cursor();
        if i == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        if eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = dec_cursor();
                if i == -1 as ::core::ffi::c_int
                    || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                && *ml_get((*curwin).w_cursor.lnum) as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = dec_cursor();
            if i == -1 as ::core::ffi::c_int
                || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    adjust_skipcol();
    return OK;
}
unsafe extern "C" fn skip_chars(
    mut cclass: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> bool {
    while cls() == cclass {
        if (if dir == FORWARD as ::core::ffi::c_int {
            inc_cursor()
        } else {
            dec_cursor()
        }) == -1 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn back_in_line() {
    let mut sclass: ::core::ffi::c_int = cls();
    while (*curwin).w_cursor.col != 0 as ::core::ffi::c_int {
        dec_cursor();
        if cls() == sclass {
            continue;
        }
        inc_cursor();
        break;
    }
}
unsafe extern "C" fn find_first_blank(mut posp: *mut pos_T) {
    while decl(posp) != -1 as ::core::ffi::c_int {
        let mut c: ::core::ffi::c_int = gchar_pos(posp);
        if ascii_iswhite(c) {
            continue;
        }
        incl(posp);
        break;
    }
}
unsafe extern "C" fn findsent_forward(mut count: ::core::ffi::c_int, mut at_start_sent: bool) {
    loop {
        let c2rust_fresh3 = count;
        count = count - 1;
        if c2rust_fresh3 == 0 {
            break;
        }
        findsent(FORWARD, 1 as ::core::ffi::c_int);
        if at_start_sent {
            find_first_blank(&raw mut (*curwin).w_cursor);
        }
        if count == 0 as ::core::ffi::c_int || at_start_sent as ::core::ffi::c_int != 0 {
            decl(&raw mut (*curwin).w_cursor);
        }
        at_start_sent = !at_start_sent;
    }
}
#[no_mangle]
pub unsafe extern "C" fn current_word(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut bigword: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true_0 != 0;
    let mut include_white: bool = false_0 != 0;
    cls_bigword = bigword;
    clearpos(&raw mut start_pos);
    if VIsual_active as ::core::ffi::c_int != 0
        && *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && lt(VIsual, (*curwin).w_cursor) as ::core::ffi::c_int != 0
    {
        dec_cursor();
    }
    if !VIsual_active || equalpos((*curwin).w_cursor, VIsual) as ::core::ffi::c_int != 0 {
        back_in_line();
        start_pos = (*curwin).w_cursor;
        if (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int == include as ::core::ffi::c_int
        {
            if end_word(1 as ::core::ffi::c_int, bigword, true_0 != 0, true_0 != 0) == FAIL {
                return FAIL;
            }
        } else {
            fwd_word(1 as ::core::ffi::c_int, bigword, true_0 != 0);
            if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
                decl(&raw mut (*curwin).w_cursor);
            } else {
                oneleft();
            }
            if include {
                include_white = true_0 != 0;
            }
        }
        if VIsual_active {
            VIsual = start_pos;
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
        }
        count -= 1;
    }
    while count > 0 as ::core::ffi::c_int {
        inclusive = true_0 != 0;
        if VIsual_active as ::core::ffi::c_int != 0
            && lt((*curwin).w_cursor, VIsual) as ::core::ffi::c_int != 0
        {
            if decl(&raw mut (*curwin).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if bck_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL {
                    return FAIL;
                }
            } else {
                if bckend_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL {
                    return FAIL;
                }
                incl(&raw mut (*curwin).w_cursor);
            }
        } else {
            if incl(&raw mut (*curwin).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if fwd_word(1 as ::core::ffi::c_int, bigword, true_0 != 0) == FAIL
                    && count > 1 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                if oneleft() == FAIL {
                    inclusive = false_0 != 0;
                }
            } else if end_word(1 as ::core::ffi::c_int, bigword, true_0 != 0, true_0 != 0) == FAIL {
                return FAIL;
            }
        }
        count -= 1;
    }
    if include_white as ::core::ffi::c_int != 0
        && (cls() != 0 as ::core::ffi::c_int
            || (*curwin).w_cursor.col == 0 as ::core::ffi::c_int && !inclusive)
    {
        let mut pos: pos_T = (*curwin).w_cursor;
        (*curwin).w_cursor = start_pos;
        if oneleft() == OK {
            back_in_line();
            if cls() == 0 as ::core::ffi::c_int && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
            {
                if VIsual_active {
                    VIsual = (*curwin).w_cursor;
                } else {
                    (*oap).start = (*curwin).w_cursor;
                }
            }
        }
        (*curwin).w_cursor = pos;
    }
    if VIsual_active {
        if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && inclusive as ::core::ffi::c_int != 0
            && ltoreq(VIsual, (*curwin).w_cursor) as ::core::ffi::c_int != 0
        {
            inc_cursor();
        }
        if VIsual_mode == 'V' as ::core::ffi::c_int {
            VIsual_mode = 'v' as ::core::ffi::c_int;
            redraw_cmdline = true_0 != 0;
        }
    } else {
        (*oap).inclusive = inclusive;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn current_sent(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_blank: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut at_start_sent: bool = false;
    let mut ncount: ::core::ffi::c_int = 0;
    let mut start_pos: pos_T = (*curwin).w_cursor;
    let mut pos: pos_T = start_pos;
    findsent(FORWARD, 1 as ::core::ffi::c_int);
    '_extend: {
        if !(VIsual_active as ::core::ffi::c_int != 0 && !equalpos(start_pos, VIsual)) {
            loop {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    break;
                }
                incl(&raw mut pos);
            }
            if equalpos(pos, (*curwin).w_cursor) {
                start_blank = true_0 != 0;
                find_first_blank(&raw mut start_pos);
            } else {
                start_blank = false_0 != 0;
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
                start_pos = (*curwin).w_cursor;
            }
            if include {
                ncount = count * 2 as ::core::ffi::c_int;
            } else {
                ncount = count;
                if start_blank {
                    ncount -= 1;
                }
            }
            if ncount > 0 as ::core::ffi::c_int {
                findsent_forward(ncount, true_0 != 0);
            } else {
                decl(&raw mut (*curwin).w_cursor);
            }
            if include {
                if start_blank {
                    find_first_blank(&raw mut (*curwin).w_cursor);
                    c = gchar_pos(&raw mut (*curwin).w_cursor);
                    if ascii_iswhite(c) {
                        decl(&raw mut (*curwin).w_cursor);
                    }
                } else {
                    c = gchar_cursor();
                    if !ascii_iswhite(c) as ::core::ffi::c_int != 0 {
                        find_first_blank(&raw mut start_pos);
                    }
                }
            }
            if VIsual_active {
                if equalpos(start_pos, (*curwin).w_cursor) {
                    break '_extend;
                } else {
                    if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                        (*curwin).w_cursor.col += 1;
                    }
                    VIsual = start_pos;
                    VIsual_mode = 'v' as ::core::ffi::c_int;
                    redraw_cmdline = true_0 != 0;
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                }
            } else {
                if incl(&raw mut (*curwin).w_cursor) == -1 as ::core::ffi::c_int {
                    (*oap).inclusive = true_0 != 0;
                } else {
                    (*oap).inclusive = false_0 != 0;
                }
                (*oap).start = start_pos;
                (*oap).motion_type = kMTCharWise;
            }
            return OK;
        }
    }
    if lt(start_pos, VIsual) {
        at_start_sent = true_0 != 0;
        decl(&raw mut pos);
        while lt(pos, (*curwin).w_cursor) {
            c = gchar_pos(&raw mut pos);
            if !ascii_iswhite(c) {
                at_start_sent = false_0 != 0;
                break;
            } else {
                incl(&raw mut pos);
            }
        }
        if !at_start_sent {
            findsent(BACKWARD, 1 as ::core::ffi::c_int);
            if equalpos((*curwin).w_cursor, start_pos) {
                at_start_sent = true_0 != 0;
            } else {
                findsent(FORWARD, 1 as ::core::ffi::c_int);
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        loop {
            let c2rust_fresh2 = count;
            count = count - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            if at_start_sent {
                find_first_blank(&raw mut (*curwin).w_cursor);
            }
            c = gchar_cursor();
            if !at_start_sent || !include && !ascii_iswhite(c) {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        incl(&raw mut pos);
        at_start_sent = true_0 != 0;
        if !equalpos(pos, (*curwin).w_cursor) {
            at_start_sent = false_0 != 0;
            while lt(pos, (*curwin).w_cursor) {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    at_start_sent = true_0 != 0;
                    break;
                } else {
                    incl(&raw mut pos);
                }
            }
            if at_start_sent {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            } else {
                (*curwin).w_cursor = start_pos;
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        findsent_forward(count, at_start_sent);
        if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            (*curwin).w_cursor.col += 1;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn current_block(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut what: ::core::ffi::c_int,
    mut other: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut end_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut sol: bool = false_0 != 0;
    let mut old_pos: pos_T = (*curwin).w_cursor;
    let mut old_end: pos_T = (*curwin).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active || equalpos(VIsual, (*curwin).w_cursor) as ::core::ffi::c_int != 0 {
        setpcmark();
        if what == '{' as ::core::ffi::c_int {
            while inindent(1 as ::core::ffi::c_int) {
                if inc_cursor() != 0 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        if gchar_cursor() == what {
            (*curwin).w_cursor.col += 1;
        }
    } else if lt(VIsual, (*curwin).w_cursor) {
        old_start = VIsual;
        (*curwin).w_cursor = VIsual;
    } else {
        old_end = VIsual;
    }
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo;
    p_cpo = (if !vim_strchr(p_cpo, CPO_MATCHBSL).is_null() {
        b"%M\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"%\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
    if !pos.is_null() {
        loop {
            let c2rust_fresh4 = count;
            count = count - 1;
            if c2rust_fresh4 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
            if pos.is_null() {
                break;
            }
            (*curwin).w_cursor = *pos;
            start_pos = *pos;
        }
    } else {
        loop {
            let c2rust_fresh5 = count;
            count = count - 1;
            if c2rust_fresh5 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                what,
                FM_FORWARD as ::core::ffi::c_int,
                0 as int64_t,
            );
            if pos.is_null() {
                break;
            }
            (*curwin).w_cursor = *pos;
            start_pos = *pos;
        }
    }
    p_cpo = save_cpo;
    if pos.is_null() || {
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        end_pos.is_null()
    } {
        (*curwin).w_cursor = old_pos;
        return FAIL;
    }
    (*curwin).w_cursor = *end_pos;
    while !include {
        incl(&raw mut start_pos);
        sol = (*curwin).w_cursor.col == 0 as ::core::ffi::c_int;
        decl(&raw mut (*curwin).w_cursor);
        while inindent(1 as ::core::ffi::c_int) {
            sol = true_0 != 0;
            if decl(&raw mut (*curwin).w_cursor) != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if equalpos(start_pos, *end_pos) as ::core::ffi::c_int != 0
            && VIsual_active as ::core::ffi::c_int != 0
        {
            (*curwin).w_cursor = old_pos;
            return FAIL;
        }
        if !(!lt(start_pos, old_start)
            && !lt(old_end, (*curwin).w_cursor)
            && !equalpos(start_pos, (*curwin).w_cursor)
            && VIsual_active as ::core::ffi::c_int != 0)
        {
            break;
        }
        (*curwin).w_cursor = old_start;
        decl(&raw mut (*curwin).w_cursor);
        pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
        if pos.is_null() {
            (*curwin).w_cursor = old_pos;
            return FAIL;
        }
        start_pos = *pos;
        (*curwin).w_cursor = *pos;
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        if end_pos.is_null() {
            (*curwin).w_cursor = old_pos;
            return FAIL;
        }
        (*curwin).w_cursor = *end_pos;
    }
    if VIsual_active {
        if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            inc(&raw mut (*curwin).w_cursor);
        }
        if sol as ::core::ffi::c_int != 0 && gchar_cursor() != NUL {
            inc(&raw mut (*curwin).w_cursor);
        }
        VIsual = start_pos;
        VIsual_mode = 'v' as ::core::ffi::c_int;
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        showmode();
    } else {
        (*oap).start = start_pos;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = false_0 != 0;
        if sol {
            incl(&raw mut (*curwin).w_cursor);
        } else if ltoreq(start_pos, (*curwin).w_cursor) {
            (*oap).inclusive = true_0 != 0;
        } else {
            (*curwin).w_cursor = start_pos;
        }
    }
    return OK;
}
unsafe extern "C" fn in_html_tag(mut end_tag: bool) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lc: ::core::ffi::c_int = NUL;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    p = line.offset((*curwin).w_cursor.col as isize);
    while p > line {
        if *p as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            break;
        }
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            break;
        }
    }
    if *p as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    pos.lnum = (*curwin).w_cursor.lnum;
    pos.col = p.offset_from(line) as colnr_T;
    p = p.offset(utfc_ptr2len(p) as isize);
    if end_tag {
        return *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    loop {
        if inc(&raw mut pos) < 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut c: ::core::ffi::c_int = *ml_get_pos(&raw mut pos) as uint8_t as ::core::ffi::c_int;
        if c == '>' as ::core::ffi::c_int {
            break;
        }
        lc = c;
    }
    return lc != '/' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn current_tagblock(
    mut oap: *mut oparg_T,
    mut count_arg: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut spat_len: size_t = 0;
    let mut spat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut epat_len: size_t = 0;
    let mut epat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut r: ::core::ffi::c_int = 0;
    let mut end_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut count: ::core::ffi::c_int = count_arg;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut do_include: bool = include;
    let mut save_p_ws: bool = p_ws != 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut is_inclusive: bool = true_0 != 0;
    p_ws = false_0;
    let mut old_pos: pos_T = (*curwin).w_cursor;
    let mut old_end: pos_T = (*curwin).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active || *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
        decl(&raw mut old_end);
    }
    if !VIsual_active || equalpos(VIsual, (*curwin).w_cursor) as ::core::ffi::c_int != 0 {
        setpcmark();
        while inindent(1 as ::core::ffi::c_int) {
            if inc_cursor() != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if in_html_tag(false_0 != 0) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
                if inc_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
        } else if in_html_tag(true_0 != 0) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
                if dec_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
            dec_cursor();
            old_end = (*curwin).w_cursor;
        }
    } else if lt(VIsual, (*curwin).w_cursor) {
        old_start = VIsual;
        (*curwin).w_cursor = VIsual;
    } else {
        old_end = VIsual;
    }
    '_theend: {
        loop {
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while n < count {
                if do_searchpair(
                    b"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    b"</[^>]*>\0".as_ptr() as *const ::core::ffi::c_char,
                    BACKWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                ) <= 0 as ::core::ffi::c_int
                {
                    (*curwin).w_cursor = old_pos;
                    break '_theend;
                } else {
                    n += 1;
                }
            }
            start_pos = (*curwin).w_cursor;
            inc_cursor();
            p = get_cursor_pos_ptr();
            cp = p;
            while *cp as ::core::ffi::c_int != NUL
                && *cp as ::core::ffi::c_int != '>' as ::core::ffi::c_int
                && !ascii_iswhite(*cp as ::core::ffi::c_int)
            {
                cp = cp.offset(utfc_ptr2len(cp) as isize);
            }
            len = cp.offset_from(p) as ::core::ffi::c_int;
            if len == 0 as ::core::ffi::c_int {
                (*curwin).w_cursor = old_pos;
                break '_theend;
            } else {
                spat_len = (len as size_t).wrapping_add(39 as size_t);
                spat = xmalloc(spat_len) as *mut ::core::ffi::c_char;
                epat_len = (len as size_t).wrapping_add(9 as size_t);
                epat = xmalloc(epat_len) as *mut ::core::ffi::c_char;
                snprintf(
                    spat,
                    spat_len,
                    b"<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    len,
                    p,
                );
                snprintf(
                    epat,
                    epat_len,
                    b"</%.*s>\\c\0".as_ptr() as *const ::core::ffi::c_char,
                    len,
                    p,
                );
                r = do_searchpair(
                    spat,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    epat,
                    FORWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                );
                xfree(spat as *mut ::core::ffi::c_void);
                xfree(epat as *mut ::core::ffi::c_void);
                if r < 1 as ::core::ffi::c_int
                    || lt((*curwin).w_cursor, old_end) as ::core::ffi::c_int != 0
                {
                    count = 1 as ::core::ffi::c_int;
                    (*curwin).w_cursor = start_pos;
                } else {
                    if do_include {
                        while *get_cursor_pos_ptr() as ::core::ffi::c_int
                            != '>' as ::core::ffi::c_int
                        {
                            if inc_cursor() < 0 as ::core::ffi::c_int {
                                break;
                            }
                        }
                    } else {
                        let mut c: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int
                            && !VIsual_active
                            && (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                        {
                            is_inclusive = false_0 != 0;
                        } else if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                            dec_cursor();
                        }
                    }
                    end_pos = (*curwin).w_cursor;
                    if do_include {
                        break;
                    }
                    let mut in_quotes: bool = false_0 != 0;
                    (*curwin).w_cursor = start_pos;
                    while inc_cursor() >= 0 as ::core::ffi::c_int {
                        p = get_cursor_pos_ptr();
                        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int && !in_quotes {
                            inc_cursor();
                            start_pos = (*curwin).w_cursor;
                            break;
                        } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                        {
                            in_quotes = !in_quotes;
                        }
                    }
                    (*curwin).w_cursor = end_pos;
                    if !(VIsual_active as ::core::ffi::c_int != 0
                        && equalpos(start_pos, old_start) as ::core::ffi::c_int != 0
                        && equalpos(end_pos, old_end) as ::core::ffi::c_int != 0)
                    {
                        break;
                    }
                    do_include = true_0 != 0;
                    (*curwin).w_cursor = old_start;
                    count = count_arg;
                }
            }
        }
        if VIsual_active {
            if lt(end_pos, start_pos) {
                (*curwin).w_cursor = start_pos;
            } else if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                inc_cursor();
            }
            VIsual = start_pos;
            VIsual_mode = 'v' as ::core::ffi::c_int;
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            showmode();
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
            if lt(end_pos, start_pos) {
                (*curwin).w_cursor = start_pos;
                (*oap).inclusive = false_0 != 0;
            } else {
                (*oap).inclusive = is_inclusive;
            }
        }
        retval = OK;
    }
    p_ws = save_p_ws as ::core::ffi::c_int;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn current_par(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut type_0: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut dir: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = OK;
    let mut do_white: ::core::ffi::c_int = false_0;
    if type_0 == 'S' as ::core::ffi::c_int {
        return FAIL;
    }
    let mut start_lnum: linenr_T = (*curwin).w_cursor.lnum;
    '_extend: {
        if !(VIsual_active as ::core::ffi::c_int != 0 && start_lnum != VIsual.lnum) {
            let mut white_in_front: bool = linewhite(start_lnum);
            while start_lnum > 1 as linenr_T {
                if white_in_front {
                    if !linewhite(start_lnum - 1 as linenr_T) {
                        break;
                    }
                } else if linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                    || startPS(start_lnum, 0 as ::core::ffi::c_int, false) as ::core::ffi::c_int
                        != 0
                {
                    break;
                }
                start_lnum -= 1;
            }
            let mut end_lnum: linenr_T = start_lnum;
            while end_lnum <= (*curbuf).b_ml.ml_line_count
                && linewhite(end_lnum) as ::core::ffi::c_int != 0
            {
                end_lnum += 1;
            }
            end_lnum -= 1;
            let mut i_0: ::core::ffi::c_int = count;
            if !include && white_in_front as ::core::ffi::c_int != 0 {
                i_0 -= 1;
            }
            loop {
                let c2rust_fresh6 = i_0;
                i_0 = i_0 - 1;
                if c2rust_fresh6 == 0 {
                    break;
                }
                if end_lnum == (*curbuf).b_ml.ml_line_count {
                    return FAIL;
                }
                if !include {
                    do_white = linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int;
                }
                if include as ::core::ffi::c_int != 0 || do_white == 0 {
                    end_lnum += 1;
                    while end_lnum < (*curbuf).b_ml.ml_line_count
                        && !linewhite(end_lnum + 1 as linenr_T)
                        && !startPS(end_lnum + 1 as linenr_T, 0 as ::core::ffi::c_int, false)
                    {
                        end_lnum += 1;
                    }
                }
                if i_0 == 0 as ::core::ffi::c_int
                    && white_in_front as ::core::ffi::c_int != 0
                    && include as ::core::ffi::c_int != 0
                {
                    break;
                }
                if include as ::core::ffi::c_int != 0 || do_white != 0 {
                    while end_lnum < (*curbuf).b_ml.ml_line_count
                        && linewhite(end_lnum + 1 as linenr_T) as ::core::ffi::c_int != 0
                    {
                        end_lnum += 1;
                    }
                }
            }
            if !white_in_front && !linewhite(end_lnum) && include as ::core::ffi::c_int != 0 {
                while start_lnum > 1 as linenr_T
                    && linewhite(start_lnum - 1 as linenr_T) as ::core::ffi::c_int != 0
                {
                    start_lnum -= 1;
                }
            }
            if VIsual_active {
                if VIsual_mode == 'V' as ::core::ffi::c_int && start_lnum == (*curwin).w_cursor.lnum
                {
                    break '_extend;
                } else {
                    if VIsual.lnum != start_lnum {
                        VIsual.lnum = start_lnum;
                        VIsual.col = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    VIsual_mode = 'V' as ::core::ffi::c_int;
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                    showmode();
                }
            } else {
                (*oap).start.lnum = start_lnum;
                (*oap).start.col = 0 as ::core::ffi::c_int as colnr_T;
                (*oap).motion_type = kMTLineWise;
            }
            (*curwin).w_cursor.lnum = end_lnum;
            (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            return OK;
        }
    }
    dir = if start_lnum < VIsual.lnum {
        BACKWARD as ::core::ffi::c_int
    } else {
        FORWARD as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = count;
    loop {
        i -= 1;
        if i < 0 as ::core::ffi::c_int {
            break;
        }
        if start_lnum
            == (if dir == BACKWARD as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                (*curbuf).b_ml.ml_line_count
            })
        {
            retval = FAIL;
            break;
        } else {
            let mut prev_start_is_white: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut t: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while t < 2 as ::core::ffi::c_int {
                start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                let mut start_is_white: ::core::ffi::c_int =
                    linewhite(start_lnum) as ::core::ffi::c_int;
                if prev_start_is_white == start_is_white {
                    start_lnum = (start_lnum as ::core::ffi::c_int - dir) as linenr_T;
                    break;
                } else {
                    while start_lnum
                        != (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf).b_ml.ml_line_count
                        })
                    {
                        if start_is_white
                            != linewhite(start_lnum + dir as linenr_T) as ::core::ffi::c_int
                            || start_is_white == 0
                                && startPS(
                                    start_lnum
                                        + (if dir > 0 as ::core::ffi::c_int {
                                            1 as linenr_T
                                        } else {
                                            0 as linenr_T
                                        }),
                                    0 as ::core::ffi::c_int,
                                    false,
                                ) as ::core::ffi::c_int
                                    != 0
                        {
                            break;
                        }
                        start_lnum = (start_lnum as ::core::ffi::c_int + dir) as linenr_T;
                    }
                    if !include {
                        break;
                    }
                    if start_lnum
                        == (if dir == BACKWARD as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            (*curbuf).b_ml.ml_line_count
                        })
                    {
                        break;
                    }
                    prev_start_is_white = start_is_white;
                    t += 1;
                }
            }
        }
    }
    (*curwin).w_cursor.lnum = start_lnum;
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    return retval;
}
unsafe extern "C" fn find_next_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    loop {
        let mut c: ::core::ffi::c_int = *line.offset(col as isize) as uint8_t as ::core::ffi::c_int;
        if c == NUL {
            return -1 as ::core::ffi::c_int;
        } else {
            if !escape.is_null() && !vim_strchr(escape, c).is_null() {
                col += 1;
                if *line.offset(col as isize) as ::core::ffi::c_int == NUL {
                    return -1 as ::core::ffi::c_int;
                }
            } else if c == quotechar {
                break;
            }
            col += utfc_ptr2len(line.offset(col as isize));
        }
    }
    return col;
}
unsafe extern "C" fn find_prev_quote(
    mut line: *mut ::core::ffi::c_char,
    mut col_start: ::core::ffi::c_int,
    mut quotechar: ::core::ffi::c_int,
    mut escape: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    while col_start > 0 as ::core::ffi::c_int {
        col_start -= 1;
        col_start -= utf_head_off(line, line.offset(col_start as isize));
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !escape.is_null() {
            while col_start - n > 0 as ::core::ffi::c_int
                && !vim_strchr(
                    escape,
                    *line.offset((col_start - n - 1 as ::core::ffi::c_int) as isize) as uint8_t
                        as ::core::ffi::c_int,
                )
                .is_null()
            {
                n += 1;
            }
        }
        if n & 1 as ::core::ffi::c_int != 0 {
            col_start -= n;
        } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar {
            break;
        }
    }
    return col_start;
}
#[no_mangle]
pub unsafe extern "C" fn current_quote(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut quotechar: ::core::ffi::c_int,
) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut col_end: ::core::ffi::c_int = 0;
    let mut col_start: ::core::ffi::c_int = (*curwin).w_cursor.col as ::core::ffi::c_int;
    let mut inclusive: bool = false_0 != 0;
    let mut vis_empty: bool = true_0 != 0;
    let mut vis_bef_curs: bool = false_0 != 0;
    let mut did_exclusive_adj: bool = false_0 != 0;
    let mut inside_quotes: bool = false_0 != 0;
    let mut selected_quote: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut restore_vis_bef: bool = false_0 != 0;
    if VIsual_active {
        if VIsual.lnum != (*curwin).w_cursor.lnum {
            return false_0 != 0;
        }
        vis_bef_curs = lt(VIsual, (*curwin).w_cursor);
        vis_empty = equalpos(VIsual, (*curwin).w_cursor);
        if *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            if vis_bef_curs {
                dec_cursor();
                did_exclusive_adj = true_0 != 0;
            } else if !vis_empty {
                dec(&raw mut VIsual);
                did_exclusive_adj = true_0 != 0;
            }
            vis_empty = equalpos(VIsual, (*curwin).w_cursor);
            if !vis_bef_curs && !vis_empty {
                let mut t: pos_T = (*curwin).w_cursor;
                (*curwin).w_cursor = VIsual;
                VIsual = t;
                vis_bef_curs = true_0 != 0;
                restore_vis_bef = true_0 != 0;
            }
        }
    }
    if !vis_empty {
        if vis_bef_curs {
            inside_quotes = VIsual.col > 0 as ::core::ffi::c_int
                && *line
                    .offset((VIsual.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                    as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset((*curwin).w_cursor.col as isize) as ::core::ffi::c_int != NUL
                && *line.offset(
                    ((*curwin).w_cursor.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = VIsual.col as ::core::ffi::c_int;
            col_end = (*curwin).w_cursor.col as ::core::ffi::c_int;
        } else {
            inside_quotes = (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
                && *line.offset(
                    ((*curwin).w_cursor.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as isize,
                ) as uint8_t as ::core::ffi::c_int
                    == quotechar
                && *line.offset(VIsual.col as isize) as ::core::ffi::c_int != NUL
                && *line
                    .offset((VIsual.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    as uint8_t as ::core::ffi::c_int
                    == quotechar;
            i = (*curwin).w_cursor.col as ::core::ffi::c_int;
            col_end = VIsual.col as ::core::ffi::c_int;
        }
        while i <= col_end {
            if *line.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            let c2rust_fresh7 = i;
            i = i + 1;
            if *line.offset(c2rust_fresh7 as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                continue;
            }
            selected_quote = true_0 != 0;
            break;
        }
    }
    '_abort_search: {
        's_368: {
            if !vis_empty
                && *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
            {
                if vis_bef_curs {
                    col_start = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    } else {
                        col_end = find_next_quote(
                            line,
                            col_start + 1 as ::core::ffi::c_int,
                            quotechar,
                            (*curbuf).b_p_qe,
                        );
                        if col_end < 0 as ::core::ffi::c_int {
                            col_end = col_start;
                            col_start = (*curwin).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                } else {
                    col_end = find_prev_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if *line.offset(col_end as isize) as uint8_t as ::core::ffi::c_int != quotechar
                    {
                        break '_abort_search;
                    } else {
                        col_start = find_prev_quote(line, col_end, quotechar, (*curbuf).b_p_qe);
                        if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int
                            != quotechar
                        {
                            col_start = col_end;
                            col_end = (*curwin).w_cursor.col as ::core::ffi::c_int;
                        }
                    }
                }
            } else if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int == quotechar
                || !vis_empty
            {
                let mut first_col: ::core::ffi::c_int = col_start;
                if !vis_empty {
                    if vis_bef_curs {
                        first_col = find_next_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    } else {
                        first_col = find_prev_quote(
                            line,
                            col_start,
                            quotechar,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    }
                }
                col_start = 0 as ::core::ffi::c_int;
                loop {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int || col_start > first_col {
                        break '_abort_search;
                    }
                    col_end = find_next_quote(
                        line,
                        col_start + 1 as ::core::ffi::c_int,
                        quotechar,
                        (*curbuf).b_p_qe,
                    );
                    if col_end < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                    if col_start <= first_col && first_col <= col_end {
                        break 's_368;
                    }
                    col_start = col_end + 1 as ::core::ffi::c_int;
                }
            } else {
                col_start = find_prev_quote(line, col_start, quotechar, (*curbuf).b_p_qe);
                if *line.offset(col_start as isize) as uint8_t as ::core::ffi::c_int != quotechar {
                    col_start = find_next_quote(
                        line,
                        col_start,
                        quotechar,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    if col_start < 0 as ::core::ffi::c_int {
                        break '_abort_search;
                    }
                }
                col_end = find_next_quote(
                    line,
                    col_start + 1 as ::core::ffi::c_int,
                    quotechar,
                    (*curbuf).b_p_qe,
                );
                if col_end < 0 as ::core::ffi::c_int {
                    break '_abort_search;
                }
            }
        }
        if include {
            if ascii_iswhite(
                *line.offset((col_end + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            ) {
                while ascii_iswhite(*line.offset((col_end + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int)
                {
                    col_end += 1;
                }
            } else {
                while col_start > 0 as ::core::ffi::c_int
                    && ascii_iswhite(*line.offset((col_start - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0
                {
                    col_start -= 1;
                }
            }
        }
        if !include
            && count < 2 as ::core::ffi::c_int
            && (vis_empty as ::core::ffi::c_int != 0 || !inside_quotes)
        {
            col_start += 1;
        }
        (*curwin).w_cursor.col = col_start as colnr_T;
        if VIsual_active {
            if vis_empty as ::core::ffi::c_int != 0
                || vis_bef_curs as ::core::ffi::c_int != 0
                    && !selected_quote
                    && (inside_quotes as ::core::ffi::c_int != 0
                        || *line.offset(VIsual.col as isize) as uint8_t as ::core::ffi::c_int
                            != quotechar
                            && (VIsual.col == 0 as ::core::ffi::c_int
                                || *line.offset(
                                    (VIsual.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                        as isize,
                                ) as uint8_t
                                    as ::core::ffi::c_int
                                    != quotechar))
            {
                VIsual = (*curwin).w_cursor;
                redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            }
        } else {
            (*oap).start = (*curwin).w_cursor;
            (*oap).motion_type = kMTCharWise;
        }
        (*curwin).w_cursor.col = col_end as colnr_T;
        if (include as ::core::ffi::c_int != 0
            || count > 1 as ::core::ffi::c_int
            || !vis_empty && inside_quotes as ::core::ffi::c_int != 0)
            && inc_cursor() == 2 as ::core::ffi::c_int
        {
            inclusive = true_0 != 0;
        }
        if VIsual_active {
            if vis_empty as ::core::ffi::c_int != 0 || vis_bef_curs as ::core::ffi::c_int != 0 {
                if *p_sel as ::core::ffi::c_int != 'e' as ::core::ffi::c_int {
                    dec_cursor();
                }
            } else {
                if inside_quotes as ::core::ffi::c_int != 0
                    || !selected_quote
                        && *line.offset(VIsual.col as isize) as uint8_t as ::core::ffi::c_int
                            != quotechar
                        && (*line.offset(VIsual.col as isize) as ::core::ffi::c_int == NUL
                            || *line.offset(
                                (VIsual.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as uint8_t as ::core::ffi::c_int
                                != quotechar)
                {
                    dec_cursor();
                    VIsual = (*curwin).w_cursor;
                }
                (*curwin).w_cursor.col = col_start as colnr_T;
            }
            if VIsual_mode == 'V' as ::core::ffi::c_int {
                VIsual_mode = 'v' as ::core::ffi::c_int;
                redraw_cmdline = true_0 != 0;
            }
        } else {
            (*oap).inclusive = inclusive;
        }
        return true_0 != 0;
    }
    if VIsual_active as ::core::ffi::c_int != 0
        && *p_sel as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
    {
        if did_exclusive_adj {
            inc_cursor();
        }
        if restore_vis_bef {
            let mut t_0: pos_T = (*curwin).w_cursor;
            (*curwin).w_cursor = VIsual;
            VIsual = t_0;
        }
    }
    return false_0 != 0;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
