use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn mh_get_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> uint32_t;
    static namespace_localscope: GlobalCell<Set_uint32_t>;
    static breakat_flags: GlobalCell<[::core::ffi::c_char; 256]>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor: bool) -> bool;
    fn decor_virt_lines(
        wp: *mut win_T,
        start_row: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        num_below: *mut ::core::ffi::c_int,
        lines: *mut VirtLines,
        apply_folds: bool,
    ) -> ::core::ffi::c_int;
    fn diff_check_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn diffopt_filler() -> bool;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn hasFoldingWin(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
        cache: bool,
        infop: *mut foldinfo_T,
    ) -> bool;
    fn lineFolded(win: *mut win_T, lnum: linenr_T) -> bool;
    static curwin: GlobalCell<*mut win_T>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    fn tabstop_padding(col: colnr_T, ts_arg: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int;
    fn get_breakindent_win(wp: *mut win_T, line: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn marktree_itr_get_filter(
        b: *mut MarkTree,
        row: int32_t,
        col: ::core::ffi::c_int,
        stop_row: ::core::ffi::c_int,
        stop_col: ::core::ffi::c_int,
        meta_filter: MetaFilter,
        itr: *mut MarkTreeIter,
    ) -> bool;
    fn marktree_itr_next_filter(
        b: *mut MarkTree,
        itr: *mut MarkTreeIter,
        stop_row: ::core::ffi::c_int,
        stop_col: ::core::ffi::c_int,
        meta_filter: MetaFilter,
    ) -> bool;
    fn marktree_itr_current(itr: *mut MarkTreeIter) -> MTKey;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    static utf8len_tab: [uint8_t; 256];
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_col_off2(wp: *mut win_T) -> ::core::ffi::c_int;
    fn get_showbreak_value(win: *mut win_T) -> *mut ::core::ffi::c_char;
    fn virtual_active(wp: *mut win_T) -> bool;
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type size_t = usize;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_13 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_13 = 4;
pub const kVTHide: C2Rust_Unnamed_13 = 2;
pub const kVTIsLines: C2Rust_Unnamed_13 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorInline {
    pub ext: bool,
    pub data: DecorInlineData,
}
pub type MetaIndex = ::core::ffi::c_uint;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub type MetaFilter = *const uint32_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kInvalidByteCells: C2Rust_Unnamed_16 = 4;
pub type CSType = bool;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kCharsizeFast: C2Rust_Unnamed_17 = 1;
pub const kCharsizeRegular: C2Rust_Unnamed_17 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharsizeArg {
    pub win: *mut win_T,
    pub line: *mut ::core::ffi::c_char,
    pub use_tabstop: bool,
    pub indent_width: ::core::ffi::c_int,
    pub virt_row: ::core::ffi::c_int,
    pub cur_text_width_left: ::core::ffi::c_int,
    pub cur_text_width_right: ::core::ffi::c_int,
    pub max_head_vcol: ::core::ffi::c_int,
    pub iter: [MarkTreeIter; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
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
#[inline]
unsafe extern "C" fn ns_in_win(mut ns_id: uint32_t, mut wp: *mut win_T) -> bool {
    if !set_has_uint32_t(namespace_localscope.ptr(), ns_id) {
        return true_0 != 0;
    }
    return set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
#[inline(always)]
unsafe extern "C" fn vim_isbreak(mut c: ::core::ffi::c_int) -> bool {
    return (*breakat_flags.ptr())[c as uint8_t as usize] != 0;
}
pub const kMTFilterSelect: uint32_t = -1 as ::core::ffi::c_int as uint32_t;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int = utf8len_tab[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
#[inline(always)]
unsafe extern "C" fn win_linetabsize(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
    mut len: colnr_T,
) -> ::core::ffi::c_int {
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return linesize_fast(&raw mut csarg, 0 as ::core::ffi::c_int, len);
    } else {
        return linesize_regular(&raw mut csarg, 0 as ::core::ffi::c_int, len);
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_chartabsize(
    mut wp: *mut win_T,
    mut p: *mut ::core::ffi::c_char,
    mut col: colnr_T,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if *p as ::core::ffi::c_int == TAB
        && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
    {
        return tabstop_padding(col, (*buf).b_p_ts, (*buf).b_p_vts_array);
    }
    return ptr2cells(p);
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize_col(
    mut startvcol: ::core::ffi::c_int,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, curwin.get(), 0 as linenr_T, s);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return linesize_fast(&raw mut csarg, startvcol, MAXCOL as ::core::ffi::c_int);
    } else {
        return linesize_regular(&raw mut csarg, startvcol, MAXCOL as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize(mut wp: *mut win_T, mut lnum: linenr_T) -> ::core::ffi::c_int {
    return win_linetabsize(
        wp,
        lnum,
        ml_get_buf((*wp).w_buffer, lnum),
        MAXCOL as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn linetabsize_eol(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    return linetabsize(wp, lnum)
        + (if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}
static inline_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([kMTFilterSelect, 0, 0, 0, 0]);
#[no_mangle]
pub unsafe extern "C" fn init_charsize_arg(
    mut csarg: *mut CharsizeArg,
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut line: *mut ::core::ffi::c_char,
) -> CSType {
    (*csarg).win = wp;
    (*csarg).line = line;
    (*csarg).max_head_vcol = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_left = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_right = 0 as ::core::ffi::c_int;
    (*csarg).virt_row = -1 as ::core::ffi::c_int;
    (*csarg).indent_width = INT_MIN;
    (*csarg).use_tabstop = (*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0;
    if lnum > 0 as linenr_T {
        if marktree_itr_get_filter(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            lnum as int32_t - 1 as int32_t,
            0 as ::core::ffi::c_int,
            lnum as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (inline_filter.ptr() as *const _) as MetaFilter,
            &raw mut (*csarg).iter as *mut MarkTreeIter,
        ) {
            (*csarg).virt_row = (lnum - 1 as linenr_T) as ::core::ffi::c_int;
        }
    }
    if (*csarg).virt_row >= 0 as ::core::ffi::c_int
        || (*wp).w_onebuf_opt.wo_wrap != 0
            && ((*wp).w_onebuf_opt.wo_lbr != 0
                || (*wp).w_onebuf_opt.wo_bri != 0
                || *get_showbreak_value(wp) as ::core::ffi::c_int != NUL)
    {
        return kCharsizeRegular as ::core::ffi::c_int != 0;
    } else {
        return kCharsizeFast as ::core::ffi::c_int != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn charsize_regular(
    mut csarg: *mut CharsizeArg,
    cur: *mut ::core::ffi::c_char,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    (*csarg).cur_text_width_left = 0 as ::core::ffi::c_int;
    (*csarg).cur_text_width_right = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = (*csarg).win;
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let mut line: *mut ::core::ffi::c_char = (*csarg).line;
    let use_tabstop: bool =
        cur_char == TAB as int32_t && (*csarg).use_tabstop as ::core::ffi::c_int != 0;
    let mut mb_added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_lcs_eol: bool =
        (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T;
    let mut size: ::core::ffi::c_int = 0;
    let mut is_doublewidth: ::core::ffi::c_int = false_0;
    if use_tabstop {
        size = tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array);
    } else if *cur as ::core::ffi::c_int == NUL {
        size = if has_lcs_eol as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    } else if cur_char < 0 as int32_t {
        size = kInvalidByteCells as ::core::ffi::c_int;
    } else {
        size = ptr2cells(cur);
        is_doublewidth =
            (size == 2 as ::core::ffi::c_int && cur_char >= 0x80 as int32_t) as ::core::ffi::c_int;
    }
    if (*csarg).virt_row >= 0 as ::core::ffi::c_int {
        let mut tab_size: ::core::ffi::c_int = size;
        let mut col: ::core::ffi::c_int = cur.offset_from(line) as ::core::ffi::c_int;
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut (*csarg).iter as *mut MarkTreeIter);
            if mark.pos.row != (*csarg).virt_row as int32_t || mark.pos.col > col as int32_t {
                break;
            }
            if mark.pos.col == col as int32_t {
                if !mt_invalid(mark) && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0 {
                    let mut decor: DecorInline = mt_decor(mark);
                    let mut vt: *mut DecorVirtText = if decor.ext as ::core::ffi::c_int != 0 {
                        decor.data.ext.vt
                    } else {
                        ::core::ptr::null_mut::<DecorVirtText>()
                    };
                    while !vt.is_null() {
                        if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int == 0
                            && (*vt).pos as ::core::ffi::c_uint
                                == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if mt_right(mark) {
                                (*csarg).cur_text_width_right += (*vt).width;
                            } else {
                                (*csarg).cur_text_width_left += (*vt).width;
                            }
                            size += (*vt).width;
                            if use_tabstop {
                                size -= tab_size;
                                tab_size = tabstop_padding(
                                    vcol + size as colnr_T,
                                    (*buf).b_p_ts,
                                    (*buf).b_p_vts_array,
                                );
                                size += tab_size;
                            }
                        }
                        vt = (*vt).next;
                    }
                }
            }
            marktree_itr_next_filter(
                &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
                &raw mut (*csarg).iter as *mut MarkTreeIter,
                (*csarg).virt_row + 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (inline_filter.ptr() as *const _) as MetaFilter,
            );
        }
    }
    if is_doublewidth != 0
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && in_win_border(wp, vcol + size as colnr_T - 2 as colnr_T) as ::core::ffi::c_int != 0
    {
        size += 1;
        mb_added = 1 as ::core::ffi::c_int;
    }
    let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
    let mut head: ::core::ffi::c_int = mb_added;
    if size > 0 as ::core::ffi::c_int
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && (*sbr as ::core::ffi::c_int != NUL || (*wp).w_onebuf_opt.wo_bri != 0)
    {
        let mut col_off_prev: ::core::ffi::c_int = win_col_off(wp);
        let mut width2: ::core::ffi::c_int = (*wp).w_view_width - col_off_prev + win_col_off2(wp);
        let mut wcol: colnr_T = vcol + col_off_prev as colnr_T;
        let mut max_head_vcol: colnr_T = (*csarg).max_head_vcol as colnr_T;
        let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut head_prev: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if wcol >= (*wp).w_view_width {
            wcol -= (*wp).w_view_width;
            col_off_prev = (*wp).w_view_width - width2;
            if wcol >= width2 && width2 > 0 as ::core::ffi::c_int {
                wcol %= width2;
            }
            head_prev = (*csarg).indent_width;
            if head_prev == INT_MIN {
                head_prev = 0 as ::core::ffi::c_int;
                if *sbr as ::core::ffi::c_int != NUL {
                    head_prev += vim_strsize(sbr);
                }
                if (*wp).w_onebuf_opt.wo_bri != 0 {
                    head_prev += get_breakindent_win(wp, line);
                }
                (*csarg).indent_width = head_prev;
            }
            if wcol < head_prev {
                head_prev -= wcol as ::core::ffi::c_int;
                wcol += head_prev;
                added += head_prev;
                if max_head_vcol <= 0 as ::core::ffi::c_int || vcol < max_head_vcol {
                    head += head_prev;
                }
            } else {
                head_prev = 0 as ::core::ffi::c_int;
            }
            wcol += col_off_prev;
        }
        if wcol as ::core::ffi::c_int + size > (*wp).w_view_width {
            let mut head_mid: ::core::ffi::c_int = (*csarg).indent_width;
            if head_mid == INT_MIN {
                head_mid = 0 as ::core::ffi::c_int;
                if *sbr as ::core::ffi::c_int != NUL {
                    head_mid += vim_strsize(sbr);
                }
                if (*wp).w_onebuf_opt.wo_bri != 0 {
                    head_mid += get_breakindent_win(wp, line);
                }
                (*csarg).indent_width = head_mid;
            }
            if head_mid > 0 as ::core::ffi::c_int {
                let mut prev_rem: ::core::ffi::c_int =
                    (*wp).w_view_width - wcol as ::core::ffi::c_int;
                let mut width: ::core::ffi::c_int = width2 - head_mid;
                if width <= 0 as ::core::ffi::c_int {
                    width = 1 as ::core::ffi::c_int;
                }
                let mut cnt: ::core::ffi::c_int =
                    (size - prev_rem + width - 1 as ::core::ffi::c_int) / width;
                added += cnt * head_mid;
                if max_head_vcol == 0 as ::core::ffi::c_int
                    || vcol as ::core::ffi::c_int + size + added < max_head_vcol
                {
                    head += cnt * head_mid;
                } else if width2 > 0 as ::core::ffi::c_int
                    && max_head_vcol > vcol as ::core::ffi::c_int + head_prev + prev_rem
                {
                    head += (max_head_vcol as ::core::ffi::c_int
                        - (vcol as ::core::ffi::c_int + head_prev + prev_rem)
                        + width2
                        - 1 as ::core::ffi::c_int)
                        / width2
                        * head_mid;
                } else if max_head_vcol < 0 as ::core::ffi::c_int {
                    let mut off: ::core::ffi::c_int =
                        mb_added + virt_text_cursor_off(csarg, *cur as ::core::ffi::c_int == NUL);
                    if off >= prev_rem {
                        if size > off {
                            head += (1 as ::core::ffi::c_int + (off - prev_rem) / width) * head_mid;
                        } else {
                            head += (off - prev_rem + width - 1 as ::core::ffi::c_int) / width
                                * head_mid;
                        }
                    }
                }
            }
        }
        size += added;
    }
    let mut need_lbr: bool = false_0 != 0;
    if (*wp).w_onebuf_opt.wo_lbr != 0
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && (*wp).w_view_width != 0 as ::core::ffi::c_int
        && vim_isbreak(
            *cur.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        ) as ::core::ffi::c_int
            != 0
        && !vim_isbreak(
            *cur.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        )
    {
        let mut t: *mut ::core::ffi::c_char = (*csarg).line;
        while vim_isbreak(
            *t.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        ) {
            t = t.offset(1);
        }
        need_lbr = cur >= t;
    }
    if need_lbr {
        let mut s: *mut ::core::ffi::c_char = cur;
        let mut numberextra: ::core::ffi::c_int = win_col_off(wp);
        let mut col_adj: colnr_T = size as colnr_T - 1 as colnr_T;
        let mut colmax: colnr_T = (*wp).w_view_width as colnr_T - numberextra as colnr_T - col_adj;
        if vcol >= colmax {
            colmax += col_adj;
            let mut n: ::core::ffi::c_int = colmax as ::core::ffi::c_int + win_col_off2(wp);
            if n > 0 as ::core::ffi::c_int {
                colmax += (((vcol - colmax) / n as colnr_T + 1 as colnr_T) * n as colnr_T - col_adj)
                    as ::core::ffi::c_int;
            }
        }
        let mut vcol2: colnr_T = vcol;
        loop {
            let mut ps: *mut ::core::ffi::c_char = s;
            s = s.offset(utfc_ptr2len(s) as isize);
            let mut c: ::core::ffi::c_int = *s as uint8_t as ::core::ffi::c_int;
            if !(c != NUL
                && (vim_isbreak(c) as ::core::ffi::c_int != 0
                    || vcol2 == vcol
                    || !vim_isbreak(*ps as uint8_t as ::core::ffi::c_int)))
            {
                break;
            }
            vcol2 += win_chartabsize(wp, s, vcol2);
            if vcol2 < colmax {
                continue;
            }
            size = (colmax - vcol + col_adj) as ::core::ffi::c_int;
            break;
        }
    }
    return CharSize {
        width: size,
        head: head,
    };
}
#[inline(always)]
unsafe extern "C" fn charsize_fast_impl(
    wp: *mut win_T,
    mut cur: *const ::core::ffi::c_char,
    mut use_tabstop: bool,
    vcol: colnr_T,
    cur_char: int32_t,
) -> CharSize {
    if cur_char == TAB as int32_t && use_tabstop as ::core::ffi::c_int != 0 {
        return CharSize {
            width: tabstop_padding(
                vcol,
                (*(*wp).w_buffer).b_p_ts,
                (*(*wp).w_buffer).b_p_vts_array,
            ),
            head: 0,
        };
    } else {
        let mut width: ::core::ffi::c_int = 0;
        if cur_char < 0 as int32_t {
            width = kInvalidByteCells as ::core::ffi::c_int;
        } else {
            width = ptr2cells(cur);
        }
        if width == 2 as ::core::ffi::c_int
            && cur_char >= 0x80 as int32_t
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && in_win_border(wp, vcol) as ::core::ffi::c_int != 0
        {
            return CharSize {
                width: 3 as ::core::ffi::c_int,
                head: 1 as ::core::ffi::c_int,
            };
        } else {
            return CharSize {
                width: width,
                head: 0,
            };
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn charsize_fast(
    mut csarg: *mut CharsizeArg,
    mut cur: *const ::core::ffi::c_char,
    mut vcol: colnr_T,
    mut cur_char: int32_t,
) -> CharSize {
    return charsize_fast_impl((*csarg).win, cur, (*csarg).use_tabstop, vcol, cur_char);
}
#[no_mangle]
pub unsafe extern "C" fn charsize_nowrap(
    mut buf: *mut buf_T,
    mut cur: *const ::core::ffi::c_char,
    mut use_tabstop: bool,
    mut vcol: colnr_T,
    mut cur_char: int32_t,
) -> ::core::ffi::c_int {
    if cur_char == TAB as int32_t && use_tabstop as ::core::ffi::c_int != 0 {
        return tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array);
    } else if cur_char < 0 as int32_t {
        return kInvalidByteCells as ::core::ffi::c_int;
    } else {
        return ptr2cells(cur);
    };
}
unsafe extern "C" fn in_win_border(mut wp: *mut win_T, mut vcol: colnr_T) -> bool {
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if vcol < width1 - 1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if vcol == width1 - 1 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    if width2 <= 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    return (vcol as ::core::ffi::c_int - width1) % width2 == width2 - 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn linesize_regular(
    csarg: *mut CharsizeArg,
    mut vcol_arg: ::core::ffi::c_int,
    len: colnr_T,
) -> ::core::ffi::c_int {
    let line: *mut ::core::ffi::c_char = (*csarg).line;
    let mut vcol: int64_t = vcol_arg as int64_t;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    while ci.ptr.offset_from(line) < len as isize && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += charsize_regular(csarg, ci.ptr, vcol_arg as colnr_T, ci.chr.value).width as int64_t;
        ci = utfc_next(ci);
        if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            vcol_arg = MAXCOL as ::core::ffi::c_int;
            break;
        } else {
            vcol_arg = vcol as ::core::ffi::c_int;
        }
    }
    if len == MAXCOL as ::core::ffi::c_int
        && (*csarg).virt_row >= 0 as ::core::ffi::c_int
        && *ci.ptr as ::core::ffi::c_int == NUL
    {
        let mut head: ::core::ffi::c_int =
            charsize_regular(csarg, ci.ptr, vcol_arg as colnr_T, ci.chr.value).head;
        vcol += ((*csarg).cur_text_width_left + (*csarg).cur_text_width_right + head) as int64_t;
        vcol_arg = if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            MAXCOL as ::core::ffi::c_int
        } else {
            vcol as ::core::ffi::c_int
        };
    }
    return vcol_arg;
}
#[no_mangle]
pub unsafe extern "C" fn linesize_fast(
    csarg: *const CharsizeArg,
    mut vcol_arg: ::core::ffi::c_int,
    len: colnr_T,
) -> ::core::ffi::c_int {
    let wp: *mut win_T = (*csarg).win;
    let use_tabstop: bool = (*csarg).use_tabstop;
    let line: *mut ::core::ffi::c_char = (*csarg).line;
    let mut vcol: int64_t = vcol_arg as int64_t;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    while ci.ptr.offset_from(line) < len as isize && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol_arg as colnr_T, ci.chr.value).width
            as int64_t;
        ci = utfc_next(ci);
        if vcol > MAXCOL as ::core::ffi::c_int as int64_t {
            vcol_arg = MAXCOL as ::core::ffi::c_int;
            break;
        } else {
            vcol_arg = vcol as ::core::ffi::c_int;
        }
    }
    return vcol_arg;
}
unsafe extern "C" fn virt_text_cursor_off(
    mut csarg: *const CharsizeArg,
    mut on_NUL: bool,
) -> ::core::ffi::c_int {
    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !on_NUL || State.get() & MODE_NORMAL as ::core::ffi::c_int == 0 {
        off += (*csarg).cur_text_width_left;
    }
    if !on_NUL && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        off += (*csarg).cur_text_width_right;
    }
    return off;
}
#[no_mangle]
pub unsafe extern "C" fn getvcol(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut start: *mut colnr_T,
    mut cursor: *mut colnr_T,
    mut end: *mut colnr_T,
) {
    let line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, (*pos).lnum);
    let end_col: colnr_T = (*pos).col;
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut on_NUL: bool = false_0 != 0;
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, (*pos).lnum, line);
    csarg.max_head_vcol = -1 as ::core::ffi::c_int;
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut char_size: CharSize = CharSize { width: 0, head: 0 };
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        let use_tabstop: bool = csarg.use_tabstop;
        loop {
            if *ci.ptr as ::core::ffi::c_int == NUL {
                char_size = CharSize {
                    width: 1 as ::core::ffi::c_int,
                    head: 0,
                };
                break;
            } else {
                char_size = charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol, ci.chr.value);
                let next: StrCharInfo = utfc_next(ci);
                if next.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next;
                vcol += char_size.width;
            }
        }
    } else {
        loop {
            char_size = charsize_regular(&raw mut csarg, ci.ptr, vcol, ci.chr.value);
            if *ci.ptr as ::core::ffi::c_int == NUL {
                char_size.width = 1 as ::core::ffi::c_int
                    + csarg.cur_text_width_left
                    + csarg.cur_text_width_right;
                on_NUL = true_0 != 0;
                break;
            } else {
                let next_0: StrCharInfo = utfc_next(ci);
                if next_0.ptr.offset_from(line) > end_col as isize {
                    break;
                }
                ci = next_0;
                vcol += char_size.width;
            }
        }
    }
    if *ci.ptr as ::core::ffi::c_int == NUL
        && end_col < MAXCOL as ::core::ffi::c_int
        && end_col as isize > ci.ptr.offset_from(line)
    {
        (*pos).col = ci.ptr.offset_from(line) as colnr_T;
    }
    let mut head: ::core::ffi::c_int = char_size.head;
    let mut incr: ::core::ffi::c_int = char_size.width;
    if !start.is_null() {
        *start = (vcol as ::core::ffi::c_int + head) as colnr_T;
    }
    if !end.is_null() {
        *end = (vcol as ::core::ffi::c_int + incr - 1 as ::core::ffi::c_int) as colnr_T;
    }
    if !cursor.is_null() {
        if ci.chr.value == TAB as int32_t
            && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0
            && (*wp).w_onebuf_opt.wo_list == 0
            && !virtual_active(wp)
            && !(VIsual_active.get() as ::core::ffi::c_int != 0
                && (*p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                    || ltoreq(*pos, VIsual.get()) as ::core::ffi::c_int != 0))
        {
            *cursor = (vcol as ::core::ffi::c_int + incr - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            vcol += virt_text_cursor_off(&raw mut csarg, on_NUL);
            *cursor = (vcol as ::core::ffi::c_int + head) as colnr_T;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn getvcol_nolist(mut posp: *mut pos_T) -> colnr_T {
    let mut list_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
    let mut vcol: colnr_T = 0;
    (*curwin.get()).w_onebuf_opt.wo_list = false_0;
    if (*posp).coladd != 0 {
        getvvcol(
            curwin.get(),
            posp,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
    } else {
        getvcol(
            curwin.get(),
            posp,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
        );
    }
    (*curwin.get()).w_onebuf_opt.wo_list = list_save;
    return vcol;
}
#[no_mangle]
pub unsafe extern "C" fn getvvcol(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut start: *mut colnr_T,
    mut cursor: *mut colnr_T,
    mut end: *mut colnr_T,
) {
    let mut col: colnr_T = 0;
    if virtual_active(wp) {
        getvcol(
            wp,
            pos,
            &raw mut col,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        let mut coladd: colnr_T = (*pos).coladd;
        let mut endadd: colnr_T = 0 as colnr_T;
        let mut ptr: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, (*pos).lnum);
        if (*pos).col < ml_get_buf_len((*wp).w_buffer, (*pos).lnum) {
            let mut c: ::core::ffi::c_int = utf_ptr2char(ptr.offset((*pos).col as isize));
            if c != TAB && vim_isprintc(c) as ::core::ffi::c_int != 0 {
                endadd = ptr2cells(ptr.offset((*pos).col as isize)) - 1 as ::core::ffi::c_int;
                if coladd > endadd {
                    endadd = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    coladd = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
        }
        col += coladd;
        if !start.is_null() {
            *start = col;
        }
        if !cursor.is_null() {
            *cursor = col;
        }
        if !end.is_null() {
            *end = col + endadd;
        }
    } else {
        getvcol(wp, pos, start, cursor, end);
    };
}
#[no_mangle]
pub unsafe extern "C" fn getvcols(
    mut wp: *mut win_T,
    mut pos1: *mut pos_T,
    mut pos2: *mut pos_T,
    mut left: *mut colnr_T,
    mut right: *mut colnr_T,
) {
    let mut from1: colnr_T = 0;
    let mut from2: colnr_T = 0;
    let mut to1: colnr_T = 0;
    let mut to2: colnr_T = 0;
    if lt(*pos1, *pos2) {
        getvvcol(
            wp,
            pos1,
            &raw mut from1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to1,
        );
        getvvcol(
            wp,
            pos2,
            &raw mut from2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to2,
        );
    } else {
        getvvcol(
            wp,
            pos2,
            &raw mut from1,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to1,
        );
        getvvcol(
            wp,
            pos1,
            &raw mut from2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut to2,
        );
    }
    if from2 < from1 {
        *left = from2;
    } else {
        *left = from1;
    }
    if to2 > to1 {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && from2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int >= to1
        {
            *right = (from2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
        } else {
            *right = to2;
        }
    } else {
        *right = to1;
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_may_fill(mut wp: *mut win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_diff != 0 && diffopt_filler() as ::core::ffi::c_int != 0
        || buf_meta_total((*wp).w_buffer, kMTMetaLines) != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_get_fill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut virt_lines: ::core::ffi::c_int = decor_virt_lines(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        lnum as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<VirtLines>(),
        true_0 != 0,
    );
    if diffopt_filler() {
        let mut n: ::core::ffi::c_int = diff_check_fill(wp, lnum);
        if n > 0 as ::core::ffi::c_int {
            return virt_lines + n;
        }
    }
    return virt_lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut limit_winheight: bool,
) -> ::core::ffi::c_int {
    return plines_win_nofill(wp, lnum, limit_winheight) + win_get_fill(wp, lnum);
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_nofill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut limit_winheight: bool,
) -> ::core::ffi::c_int {
    if decor_conceal_line(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).w_onebuf_opt.wo_wrap == 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if lineFolded(wp, lnum) {
        return 1 as ::core::ffi::c_int;
    }
    let lines: ::core::ffi::c_int = plines_win_nofold(wp, lnum);
    if limit_winheight as ::core::ffi::c_int != 0 && lines > (*wp).w_view_height {
        return (*wp).w_view_height;
    }
    return lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_nofold(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut s: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, s);
    if *s as ::core::ffi::c_int == NUL && csarg.virt_row < 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut col: int64_t = 0;
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        col = linesize_fast(
            &raw mut csarg,
            0 as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        ) as int64_t;
    } else {
        col = linesize_regular(
            &raw mut csarg,
            0 as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        ) as int64_t;
    }
    if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.eol != NUL as schar_T {
        col += 1 as int64_t;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if width <= 0 as ::core::ffi::c_int {
        return 32000 as ::core::ffi::c_int;
    }
    if col <= width as int64_t {
        return 1 as ::core::ffi::c_int;
    }
    col -= width as int64_t;
    width += win_col_off2(wp);
    let lines: int64_t =
        (col + (width - 1 as ::core::ffi::c_int) as int64_t) / width as int64_t + 1 as int64_t;
    return if lines > 0 as int64_t && lines <= INT_MAX as int64_t {
        lines as ::core::ffi::c_int
    } else {
        INT_MAX
    };
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut column: ::core::ffi::c_long,
) -> ::core::ffi::c_int {
    let mut lines: ::core::ffi::c_int = win_get_fill(wp, lnum);
    if (*wp).w_onebuf_opt.wo_wrap == 0 {
        return lines + 1 as ::core::ffi::c_int;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        return lines + 1 as ::core::ffi::c_int;
    }
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        let use_tabstop: bool = csarg.use_tabstop;
        while *ci.ptr as ::core::ffi::c_int != NUL && {
            column -= 1;
            column >= 0 as ::core::ffi::c_long
        } {
            vcol += charsize_fast_impl(wp, ci.ptr, use_tabstop, vcol, ci.chr.value).width;
            ci = utfc_next(ci);
        }
    } else {
        while *ci.ptr as ::core::ffi::c_int != NUL && {
            column -= 1;
            column >= 0 as ::core::ffi::c_long
        } {
            vcol += charsize_regular(&raw mut csarg, ci.ptr, vcol, ci.chr.value).width;
            ci = utfc_next(ci);
        }
    }
    let mut col: colnr_T = vcol;
    if ci.chr.value == TAB as int32_t
        && State.get() & MODE_NORMAL as ::core::ffi::c_int != 0
        && csarg.use_tabstop as ::core::ffi::c_int != 0
    {
        col += win_charsize(
            cstype,
            col as ::core::ffi::c_int,
            ci.ptr,
            ci.chr.value,
            &raw mut csarg,
        )
        .width
            - 1 as ::core::ffi::c_int;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    if width <= 0 as ::core::ffi::c_int {
        return 9999 as ::core::ffi::c_int;
    }
    lines += 1 as ::core::ffi::c_int;
    if col > width {
        lines += (col as ::core::ffi::c_int - width) / (width + win_col_off2(wp))
            + 1 as ::core::ffi::c_int;
    }
    return lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_win_full(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    nextp: *mut linenr_T,
    foldedp: *mut bool,
    cache: bool,
    limit_winheight: bool,
) -> ::core::ffi::c_int {
    let mut folded: bool = hasFoldingWin(
        wp,
        lnum,
        &raw mut lnum,
        nextp,
        cache,
        ::core::ptr::null_mut::<foldinfo_T>(),
    );
    if !foldedp.is_null() {
        *foldedp = folded;
    }
    let mut filler_lines: ::core::ffi::c_int = if lnum == (*wp).w_topline {
        (*wp).w_topfill
    } else {
        win_get_fill(wp, lnum)
    };
    if decor_conceal_line(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        return filler_lines;
    }
    return (if folded as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        plines_win_nofill(wp, lnum, limit_winheight)
    }) + filler_lines;
}
#[no_mangle]
pub unsafe extern "C" fn plines_m_win(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
    mut max: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while first <= last && count < max {
        let mut next: linenr_T = first;
        count += plines_win_full(
            wp,
            first,
            &raw mut next,
            ::core::ptr::null_mut::<bool>(),
            false_0 != 0,
            false_0 != 0,
        );
        first = next + 1 as linenr_T;
    }
    if first == (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        count += win_get_fill(wp, first);
    }
    return if max < count { max } else { count };
}
#[no_mangle]
pub unsafe extern "C" fn plines_m_win_fill(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = last as ::core::ffi::c_int - first as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + decor_virt_lines(
            wp,
            first as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            last as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<VirtLines>(),
            false_0 != 0,
        );
    if diffopt_filler() {
        let mut lnum: ::core::ffi::c_int = first as ::core::ffi::c_int;
        while lnum as linenr_T <= last {
            let mut n: ::core::ffi::c_int = diff_check_fill(wp, lnum as linenr_T);
            count += if n > 0 as ::core::ffi::c_int {
                n
            } else {
                0 as ::core::ffi::c_int
            };
            lnum += 1;
        }
    }
    return if count > 0 as ::core::ffi::c_int {
        count
    } else {
        0 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_text_height(
    wp: *mut win_T,
    start_lnum: linenr_T,
    start_vcol: int64_t,
    end_lnum: *mut linenr_T,
    end_vcol: *mut int64_t,
    fill: *mut int64_t,
    max: int64_t,
) -> int64_t {
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    width1 = if width1 > 0 as ::core::ffi::c_int {
        width1
    } else {
        0 as ::core::ffi::c_int
    };
    width2 = if width2 > 0 as ::core::ffi::c_int {
        width2
    } else {
        0 as ::core::ffi::c_int
    };
    let mut height_sum_fill: int64_t = 0 as int64_t;
    let mut height_cur_nofill: int64_t = 0 as int64_t;
    let mut height_sum_nofill: int64_t = 0 as int64_t;
    let mut lnum: linenr_T = start_lnum;
    let mut cur_lnum: linenr_T = lnum;
    let mut cur_folded: bool = false_0 != 0;
    if start_vcol >= 0 as int64_t {
        let mut lnum_next: linenr_T = lnum;
        cur_folded = hasFolding(wp, lnum, &raw mut lnum, &raw mut lnum_next);
        height_cur_nofill = plines_win_nofill(wp, lnum, false_0 != 0) as int64_t;
        height_sum_nofill += height_cur_nofill;
        let row_off: int64_t =
            if start_vcol < width1 as int64_t || width2 <= 0 as ::core::ffi::c_int {
                0 as int64_t
            } else {
                1 as int64_t + (start_vcol - width1 as int64_t) / width2 as int64_t
            };
        height_sum_nofill -= if row_off < height_cur_nofill {
            row_off
        } else {
            height_cur_nofill
        };
        lnum = lnum_next + 1 as linenr_T;
    }
    while lnum <= *end_lnum && height_sum_nofill + height_sum_fill < max {
        let mut lnum_next_0: linenr_T = lnum;
        cur_folded = hasFolding(wp, lnum, &raw mut lnum, &raw mut lnum_next_0);
        height_sum_fill += win_get_fill(wp, lnum) as int64_t;
        height_cur_nofill = plines_win_nofill(wp, lnum, false_0 != 0) as int64_t;
        height_sum_nofill += height_cur_nofill;
        cur_lnum = lnum;
        lnum = lnum_next_0 + 1 as linenr_T;
    }
    let mut vcol_end: int64_t = *end_vcol;
    let mut use_vcol: bool = vcol_end >= 0 as int64_t && lnum > *end_lnum;
    if use_vcol {
        height_sum_nofill -= height_cur_nofill;
        let row_off_0: int64_t = if vcol_end == 0 as int64_t {
            0 as int64_t
        } else if vcol_end <= width1 as int64_t || width2 <= 0 as ::core::ffi::c_int {
            1 as int64_t
        } else {
            1 as int64_t
                + (vcol_end - width1 as int64_t + width2 as int64_t - 1 as int64_t)
                    / width2 as int64_t
        };
        height_sum_nofill += if row_off_0 < height_cur_nofill {
            row_off_0
        } else {
            height_cur_nofill
        };
    }
    if cur_folded {
        vcol_end = 0 as int64_t;
    } else {
        let mut linesize: ::core::ffi::c_int = linetabsize_eol(wp, cur_lnum);
        vcol_end = if (if use_vcol as ::core::ffi::c_int != 0 {
            vcol_end
        } else {
            9223372036854775807 as int64_t
        }) < linesize as int64_t
        {
            if use_vcol as ::core::ffi::c_int != 0 {
                vcol_end
            } else {
                9223372036854775807 as int64_t
            }
        } else {
            linesize as int64_t
        };
    }
    let mut overflow: int64_t = height_sum_nofill + height_sum_fill - max;
    if overflow > 0 as int64_t && width2 > 0 as ::core::ffi::c_int && vcol_end > width2 as int64_t {
        vcol_end -= (vcol_end - width1 as int64_t) % width2 as int64_t
            + (overflow - 1 as int64_t) * width2 as int64_t;
    }
    *end_lnum = cur_lnum;
    *end_vcol = vcol_end;
    if !fill.is_null() {
        *fill = height_sum_fill;
    }
    return height_sum_fill + height_sum_nofill;
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
