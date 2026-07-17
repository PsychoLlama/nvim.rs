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
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn strtoimax(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> intmax_t;
    fn abort() -> !;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrchrnul(
        str: *const ::core::ffi::c_char,
        c: ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static mut dy_flags: ::core::ffi::c_uint;
    static mut p_isf: *mut ::core::ffi::c_char;
    static mut p_isi: *mut ::core::ffi::c_char;
    static mut p_isp: *mut ::core::ffi::c_char;
    static mut curbuf: *mut buf_T;
    fn utf_char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_ptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_printable(c: ::core::ffi::c_int) -> bool;
    fn utf_class_tab(c: ::core::ffi::c_int, chartab: *const uint64_t) -> ::core::ffi::c_int;
    fn mb_islower(a: ::core::ffi::c_int) -> bool;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_isupper(a: ::core::ffi::c_int) -> bool;
    static utf8len_tab: [uint8_t; 256];
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn skip_to_option_part(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_has_wildcard(p: *const ::core::ffi::c_char) -> bool;
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
pub type intmax_t = ::libc::intmax_t;
pub type size_t = usize;
pub type ssize_t = isize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type uvarnumber_T = uint64_t;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kOptDyFlagMsgsep: C2Rust_Unnamed_13 = 8;
pub const kOptDyFlagUhex: C2Rust_Unnamed_13 = 4;
pub const kOptDyFlagTruncate: C2Rust_Unnamed_13 = 2;
pub const kOptDyFlagLastline: C2Rust_Unnamed_13 = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_14 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_14 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_14 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_14 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_14 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_14 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_14 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_14 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_14 = 0;
pub const INT32_MIN: ::core::ffi::c_int =
    -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT32_MAX: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const INTMAX_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INTMAX_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isbdigit(mut c: ::core::ffi::c_int) -> bool {
    return c == '0' as ::core::ffi::c_int || c == '1' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isodigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '7' as ::core::ffi::c_int;
}
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const UVARNUMBER_MAX: ::core::ffi::c_ulong = UINT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut chartab_initialized: bool = false_0 != 0;
static mut g_chartab: [uint8_t; 256] = [0; 256];
pub const CT_CELL_MASK: ::core::ffi::c_int = 0x7 as ::core::ffi::c_int;
pub const CT_PRINT_CHAR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CT_ID_CHAR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CT_FNAME_CHAR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn init_chartab() -> ::core::ffi::c_int {
    return buf_init_chartab(curbuf, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn buf_init_chartab(
    mut buf: *mut buf_T,
    mut global: bool,
) -> ::core::ffi::c_int {
    if global {
        let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while c < ' ' as ::core::ffi::c_int {
            let c2rust_fresh0 = c;
            c = c + 1;
            g_chartab[c2rust_fresh0 as usize] =
                (if dy_flags & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
                    4 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                }) as uint8_t;
        }
        while c <= '~' as ::core::ffi::c_int {
            let c2rust_fresh1 = c;
            c = c + 1;
            g_chartab[c2rust_fresh1 as usize] =
                (1 as ::core::ffi::c_int + CT_PRINT_CHAR) as uint8_t;
        }
        while c < 256 as ::core::ffi::c_int {
            if c >= 0xa0 as ::core::ffi::c_int {
                let c2rust_fresh2 = c;
                c = c + 1;
                g_chartab[c2rust_fresh2 as usize] =
                    ((CT_PRINT_CHAR | CT_FNAME_CHAR) + 1 as ::core::ffi::c_int) as uint8_t;
            } else {
                let c2rust_fresh3 = c;
                c = c + 1;
                g_chartab[c2rust_fresh3 as usize] =
                    (if dy_flags & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                    {
                        4 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    }) as uint8_t;
            }
        }
    }
    memset(
        &raw mut (*buf).b_chartab as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[uint64_t; 4]>(),
    );
    if (*buf).b_p_lisp != 0 {
        (*buf).b_chartab[('-' as ::core::ffi::c_int as ::core::ffi::c_uint
            >> 6 as ::core::ffi::c_int) as usize] = ((*buf).b_chartab
            [('-' as ::core::ffi::c_int as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong)
                << ('-' as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int))
            as uint64_t;
    }
    let mut i: ::core::ffi::c_int = if global as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int
    };
    while i <= 3 as ::core::ffi::c_int {
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if i == 0 as ::core::ffi::c_int {
            p = p_isi;
        } else if i == 1 as ::core::ffi::c_int {
            p = p_isp;
        } else if i == 2 as ::core::ffi::c_int {
            p = p_isf;
        } else {
            p = (*buf).b_p_isk;
        }
        if parse_isopt(p, buf, false_0 != 0) == FAIL {
            return FAIL;
        }
        i += 1;
    }
    chartab_initialized = true_0 != 0;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_isopt(mut var: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return parse_isopt(var, ::core::ptr::null_mut::<buf_T>(), true_0 != 0);
}
unsafe extern "C" fn parse_isopt(
    mut var: *const ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut only_check: bool,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = var;
    while *p != 0 {
        let mut tilde: bool = false_0 != 0;
        let mut do_isalpha: bool = false_0 != 0;
        if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            tilde = true_0 != 0;
            p = p.offset(1);
        }
        let mut c: ::core::ffi::c_int = 0;
        if ascii_isdigit(*p as ::core::ffi::c_int) {
            c = getdigits_int(
                &raw mut p as *mut *mut ::core::ffi::c_char,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            );
        } else {
            c = mb_ptr2char_adv(&raw mut p);
        }
        let mut c2: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
            if ascii_isdigit(*p as ::core::ffi::c_int) {
                c2 = getdigits_int(
                    &raw mut p as *mut *mut ::core::ffi::c_char,
                    true_0 != 0,
                    0 as ::core::ffi::c_int,
                );
            } else {
                c2 = mb_ptr2char_adv(&raw mut p);
            }
        }
        if c <= 0 as ::core::ffi::c_int
            || c >= 256 as ::core::ffi::c_int
            || c2 < c && c2 != -1 as ::core::ffi::c_int
            || c2 >= 256 as ::core::ffi::c_int
            || !(*p as ::core::ffi::c_int == NUL
                || *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
        {
            return FAIL;
        }
        let mut trail_comma: bool = *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int;
        p = skip_to_option_part(p);
        if trail_comma as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int == NUL {
            return FAIL;
        }
        if only_check {
            continue;
        }
        if c2 == -1 as ::core::ffi::c_int {
            if c == '@' as ::core::ffi::c_int {
                do_isalpha = true_0 != 0;
                c = 1 as ::core::ffi::c_int;
                c2 = 255 as ::core::ffi::c_int;
            } else {
                c2 = c;
            }
        }
        while c <= c2 {
            if !do_isalpha
                || mb_islower(c) as ::core::ffi::c_int != 0
                || mb_isupper(c) as ::core::ffi::c_int != 0
            {
                if var == p_isi as *const ::core::ffi::c_char {
                    if tilde {
                        g_chartab[c as usize] = (g_chartab[c as usize] as ::core::ffi::c_int
                            & !CT_ID_CHAR as uint8_t as ::core::ffi::c_int)
                            as uint8_t;
                    } else {
                        g_chartab[c as usize] =
                            (g_chartab[c as usize] as ::core::ffi::c_int | CT_ID_CHAR) as uint8_t;
                    }
                } else if var == p_isp as *const ::core::ffi::c_char {
                    if c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int {
                        if tilde {
                            g_chartab[c as usize] = ((g_chartab[c as usize] as ::core::ffi::c_int
                                & !CT_CELL_MASK)
                                + (if dy_flags
                                    & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != 0
                                {
                                    4 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                })) as uint8_t;
                            g_chartab[c as usize] = (g_chartab[c as usize] as ::core::ffi::c_int
                                & !CT_PRINT_CHAR as uint8_t as ::core::ffi::c_int)
                                as uint8_t;
                        } else {
                            g_chartab[c as usize] = ((g_chartab[c as usize] as ::core::ffi::c_int
                                & !CT_CELL_MASK)
                                + 1 as ::core::ffi::c_int)
                                as uint8_t;
                            g_chartab[c as usize] = (g_chartab[c as usize] as ::core::ffi::c_int
                                | CT_PRINT_CHAR)
                                as uint8_t;
                        }
                    }
                } else if var == p_isf as *const ::core::ffi::c_char {
                    if tilde {
                        g_chartab[c as usize] = (g_chartab[c as usize] as ::core::ffi::c_int
                            & !CT_FNAME_CHAR as uint8_t as ::core::ffi::c_int)
                            as uint8_t;
                    } else {
                        g_chartab[c as usize] = (g_chartab[c as usize] as ::core::ffi::c_int
                            | CT_FNAME_CHAR)
                            as uint8_t;
                    }
                } else if tilde {
                    (*buf).b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize] = ((*buf)
                        .b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_ulonglong
                        & !((1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int)))
                        as uint64_t;
                } else {
                    (*buf).b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize] = ((*buf)
                        .b_chartab
                        [(c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_ulonglong
                        | (1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int))
                        as uint64_t;
                }
            }
            c += 1;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn trans_characters(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: ::core::ffi::c_int,
) {
    let mut trs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = strlen(buf) as ::core::ffi::c_int;
    let mut room: ::core::ffi::c_int = bufsize - len;
    while *buf as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut trs_len: ::core::ffi::c_int = 0;
        trs_len = utfc_ptr2len(buf);
        if trs_len > 1 as ::core::ffi::c_int {
            len -= trs_len;
        } else {
            trs = transchar_byte(*buf as uint8_t as ::core::ffi::c_int);
            trs_len = strlen(trs) as ::core::ffi::c_int;
            if trs_len > 1 as ::core::ffi::c_int {
                room -= trs_len - 1 as ::core::ffi::c_int;
                if room <= 0 as ::core::ffi::c_int {
                    return;
                }
                memmove(
                    buf.offset(trs_len as isize) as *mut ::core::ffi::c_void,
                    buf.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
            memmove(
                buf as *mut ::core::ffi::c_void,
                trs as *const ::core::ffi::c_void,
                trs_len as size_t,
            );
            len -= 1;
        }
        buf = buf.offset(trs_len as isize);
    }
}
#[no_mangle]
pub unsafe extern "C" fn transstr_len(s: *const ::core::ffi::c_char, mut untab: bool) -> size_t {
    let mut p: *const ::core::ffi::c_char = s;
    let mut len: size_t = 0 as size_t;
    while *p != 0 {
        let l: size_t = utfc_ptr2len(p) as size_t;
        if l > 1 as size_t {
            if vim_isprintc(utf_ptr2char(p)) {
                len = len.wrapping_add(l);
            } else {
                let mut off: size_t = 0 as size_t;
                while off < l {
                    let mut c: ::core::ffi::c_int = utf_ptr2char(p.offset(off as isize));
                    let mut hexbuf: [::core::ffi::c_char; 9] = [0; 9];
                    len = len.wrapping_add(transchar_hex(
                        &raw mut hexbuf as *mut ::core::ffi::c_char,
                        c,
                    ));
                    off = off.wrapping_add(utf_ptr2len(p.offset(off as isize)) as size_t);
                }
            }
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == TAB && !untab {
            len = len.wrapping_add(1 as size_t);
            p = p.offset(1);
        } else {
            let c2rust_fresh12 = p;
            p = p.offset(1);
            let b2c_l: ::core::ffi::c_int =
                byte2cells(*c2rust_fresh12 as uint8_t as ::core::ffi::c_int);
            len = len.wrapping_add(
                (if b2c_l > 0 as ::core::ffi::c_int {
                    b2c_l
                } else {
                    4 as ::core::ffi::c_int
                }) as size_t,
            );
        }
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn transstr_buf(
    s: *const ::core::ffi::c_char,
    slen: ssize_t,
    buf: *mut ::core::ffi::c_char,
    buflen: size_t,
    mut untab: bool,
) -> size_t {
    let mut p: *const ::core::ffi::c_char = s;
    let mut buf_p: *mut ::core::ffi::c_char = buf;
    let buf_e: *mut ::core::ffi::c_char = buf_p
        .offset(buflen as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    while (slen < 0 as ssize_t || p.offset_from(s) < slen as isize)
        && *p as ::core::ffi::c_int != NUL
        && buf_p < buf_e
    {
        let l: size_t = utfc_ptr2len(p) as size_t;
        if l > 1 as size_t {
            if buf_p.offset(l as isize) > buf_e {
                break;
            }
            if vim_isprintc(utf_ptr2char(p)) {
                memmove(
                    buf_p as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    l,
                );
                buf_p = buf_p.offset(l as isize);
            } else {
                let mut off: size_t = 0 as size_t;
                while off < l {
                    let mut c: ::core::ffi::c_int = utf_ptr2char(p.offset(off as isize));
                    let mut hexbuf: [::core::ffi::c_char; 9] = [0; 9];
                    let hexlen: size_t =
                        transchar_hex(&raw mut hexbuf as *mut ::core::ffi::c_char, c);
                    if buf_p.offset(hexlen as isize) > buf_e {
                        break;
                    }
                    memmove(
                        buf_p as *mut ::core::ffi::c_void,
                        &raw mut hexbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        hexlen,
                    );
                    buf_p = buf_p.offset(hexlen as isize);
                    off = off.wrapping_add(utf_ptr2len(p.offset(off as isize)) as size_t);
                }
            }
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == TAB && !untab {
            let c2rust_fresh13 = p;
            p = p.offset(1);
            let c2rust_fresh14 = buf_p;
            buf_p = buf_p.offset(1);
            *c2rust_fresh14 = *c2rust_fresh13;
        } else {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            let tb: *const ::core::ffi::c_char =
                transchar_byte(*c2rust_fresh15 as uint8_t as ::core::ffi::c_int);
            let tb_len: size_t = strlen(tb);
            if buf_p.offset(tb_len as isize) > buf_e {
                break;
            }
            memmove(
                buf_p as *mut ::core::ffi::c_void,
                tb as *const ::core::ffi::c_void,
                tb_len,
            );
            buf_p = buf_p.offset(tb_len as isize);
        }
    }
    *buf_p = NUL as ::core::ffi::c_char;
    '_c2rust_label: {
        if buf_p <= buf_e {
        } else {
            __assert_fail(
                b"buf_p <= buf_e\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/charset.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                395 as ::core::ffi::c_uint,
                b"size_t transstr_buf(const char *const, const ssize_t, char *const, const size_t, _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return buf_p.offset_from(buf) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn transstr(
    s: *const ::core::ffi::c_char,
    mut untab: bool,
) -> *mut ::core::ffi::c_char {
    let len: size_t = transstr_len(s, untab).wrapping_add(1 as size_t);
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    transstr_buf(s, -1 as ssize_t, buf, len, untab);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn kv_transstr(
    mut str: *mut StringBuilder,
    s: *const ::core::ffi::c_char,
    mut untab: bool,
) -> size_t {
    if s.is_null() {
        return 0 as size_t;
    }
    let len: size_t = transstr_len(s, untab);
    if (*str).capacity < (*str).size.wrapping_add(len).wrapping_add(1 as size_t) {
        (*str).capacity = (*str).size.wrapping_add(len).wrapping_add(1 as size_t);
        (*str).capacity = (*str).capacity.wrapping_sub(1);
        (*str).capacity |= (*str).capacity >> 1 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 2 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 4 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 8 as ::core::ffi::c_int;
        (*str).capacity |= (*str).capacity >> 16 as ::core::ffi::c_int;
        (*str).capacity = (*str).capacity.wrapping_add(1);
        (*str).capacity = (*str).capacity;
        (*str).items = xrealloc(
            (*str).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*str).capacity),
        ) as *mut ::core::ffi::c_char;
    }
    transstr_buf(
        s,
        -1 as ssize_t,
        (*str).items.offset((*str).size as isize),
        len.wrapping_add(1 as size_t),
        untab,
    );
    (*str).size = (*str).size.wrapping_add(len);
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn str_foldcase(
    mut str: *mut ::core::ffi::c_char,
    mut orglen: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut len: ::core::ffi::c_int = orglen;
    if buf.is_null() {
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        ga_grow(&raw mut ga, len + 1 as ::core::ffi::c_int);
        memmove(ga.ga_data, str as *const ::core::ffi::c_void, len as size_t);
        ga.ga_len = len;
    } else {
        if len >= buflen {
            len = buflen - 1 as ::core::ffi::c_int;
        }
        memmove(
            buf as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
    }
    if buf.is_null() {
        *(ga.ga_data as *mut ::core::ffi::c_char).offset(len as isize) = NUL as ::core::ffi::c_char;
    } else {
        *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (if buf.is_null() {
        *(ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize) as ::core::ffi::c_int
    } else {
        *buf.offset(i as isize) as ::core::ffi::c_int
    }) != NUL
    {
        let mut c: ::core::ffi::c_int = utf_ptr2char(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
        let mut olen: ::core::ffi::c_int = utf_ptr2len(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
        let mut lc: ::core::ffi::c_int = mb_tolower(c);
        if (c < 0x80 as ::core::ffi::c_int || olen > 1 as ::core::ffi::c_int) && c != lc {
            let mut nlen: ::core::ffi::c_int = utf_char2len(lc);
            if olen != nlen {
                if nlen > olen {
                    if buf.is_null() {
                        ga_grow(&raw mut ga, nlen - olen + 1 as ::core::ffi::c_int);
                    } else if len + nlen - olen >= buflen {
                        lc = c;
                        nlen = olen;
                    }
                }
                if olen != nlen {
                    if buf.is_null() {
                        memmove(
                            (ga.ga_data as *mut ::core::ffi::c_char)
                                .offset(i as isize)
                                .offset(nlen as isize)
                                as *mut ::core::ffi::c_void,
                            (ga.ga_data as *mut ::core::ffi::c_char)
                                .offset(i as isize)
                                .offset(olen as isize)
                                as *const ::core::ffi::c_void,
                            strlen(
                                (ga.ga_data as *mut ::core::ffi::c_char)
                                    .offset(i as isize)
                                    .offset(olen as isize),
                            )
                            .wrapping_add(1 as size_t),
                        );
                        ga.ga_len += nlen - olen;
                    } else {
                        memmove(
                            buf.offset(i as isize).offset(nlen as isize)
                                as *mut ::core::ffi::c_void,
                            buf.offset(i as isize).offset(olen as isize)
                                as *const ::core::ffi::c_void,
                            strlen(buf.offset(i as isize).offset(olen as isize))
                                .wrapping_add(1 as size_t),
                        );
                        len += nlen - olen;
                    }
                }
            }
            utf_char2bytes(
                lc,
                if buf.is_null() {
                    (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
                } else {
                    buf.offset(i as isize)
                },
            );
        }
        i += utfc_ptr2len(if buf.is_null() {
            (ga.ga_data as *mut ::core::ffi::c_char).offset(i as isize)
        } else {
            buf.offset(i as isize)
        });
    }
    if buf.is_null() {
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
    return buf;
}
static mut transchar_charbuf: [uint8_t; 11] = [0; 11];
#[no_mangle]
pub unsafe extern "C" fn transchar(mut c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    return transchar_buf(curbuf, c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_buf(
    mut buf: *const buf_T,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c < 0 as ::core::ffi::c_int {
        transchar_charbuf[0 as ::core::ffi::c_int as usize] = '~' as uint8_t;
        transchar_charbuf[1 as ::core::ffi::c_int as usize] = '@' as uint8_t;
        i = 2 as ::core::ffi::c_int;
        c = if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        };
    }
    if !chartab_initialized && (c >= ' ' as ::core::ffi::c_int && c <= '~' as ::core::ffi::c_int)
        || c <= 0xff as ::core::ffi::c_int && vim_isprintc(c) as ::core::ffi::c_int != 0
    {
        transchar_charbuf[i as usize] = c as uint8_t;
        transchar_charbuf[(i + 1 as ::core::ffi::c_int) as usize] = NUL as uint8_t;
    } else if c <= 0xff as ::core::ffi::c_int {
        transchar_nonprint(
            buf,
            (&raw mut transchar_charbuf as *mut uint8_t as *mut ::core::ffi::c_char)
                .offset(i as isize),
            c,
        );
    } else {
        transchar_hex(
            (&raw mut transchar_charbuf as *mut uint8_t as *mut ::core::ffi::c_char)
                .offset(i as isize),
            c,
        );
    }
    return &raw mut transchar_charbuf as *mut uint8_t as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn transchar_byte(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    return transchar_byte_buf(curbuf, c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_byte_buf(
    mut buf: *const buf_T,
    c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if c >= 0x80 as ::core::ffi::c_int {
        transchar_nonprint(
            buf,
            &raw mut transchar_charbuf as *mut uint8_t as *mut ::core::ffi::c_char,
            c,
        );
        return &raw mut transchar_charbuf as *mut uint8_t as *mut ::core::ffi::c_char;
    }
    return transchar_buf(buf, c);
}
#[no_mangle]
pub unsafe extern "C" fn transchar_nonprint(
    mut buf: *const buf_T,
    mut charbuf: *mut ::core::ffi::c_char,
    mut c: ::core::ffi::c_int,
) {
    if c == NL {
        c = NUL;
    } else if !buf.is_null() && c == CAR && get_fileformat(buf) == EOL_MAC {
        c = NL;
    }
    '_c2rust_label: {
        if c <= 0xff as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"c <= 0xff\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/charset.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                613 as ::core::ffi::c_uint,
                b"void transchar_nonprint(const buf_T *, char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if dy_flags & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || c > 0x7f as ::core::ffi::c_int
    {
        transchar_hex(charbuf, c);
    } else {
        *charbuf.offset(0 as ::core::ffi::c_int as isize) = '^' as ::core::ffi::c_char;
        *charbuf.offset(1 as ::core::ffi::c_int as isize) =
            (c ^ 0x40 as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_char;
        *charbuf.offset(2 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn transchar_hex(
    buf: *mut ::core::ffi::c_char,
    c: ::core::ffi::c_int,
) -> size_t {
    let mut i: size_t = 0 as size_t;
    let c2rust_fresh4 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh4 as isize) = '<' as ::core::ffi::c_char;
    if c > 0xff as ::core::ffi::c_int {
        if c > 0xffff as ::core::ffi::c_int {
            let c2rust_fresh5 = i;
            i = i.wrapping_add(1);
            *buf.offset(c2rust_fresh5 as isize) =
                nr2hex(c as ::core::ffi::c_uint >> 20 as ::core::ffi::c_int) as ::core::ffi::c_char;
            let c2rust_fresh6 = i;
            i = i.wrapping_add(1);
            *buf.offset(c2rust_fresh6 as isize) =
                nr2hex(c as ::core::ffi::c_uint >> 16 as ::core::ffi::c_int) as ::core::ffi::c_char;
        }
        let c2rust_fresh7 = i;
        i = i.wrapping_add(1);
        *buf.offset(c2rust_fresh7 as isize) =
            nr2hex(c as ::core::ffi::c_uint >> 12 as ::core::ffi::c_int) as ::core::ffi::c_char;
        let c2rust_fresh8 = i;
        i = i.wrapping_add(1);
        *buf.offset(c2rust_fresh8 as isize) =
            nr2hex(c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_char;
    }
    let c2rust_fresh9 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh9 as isize) =
        nr2hex(c as ::core::ffi::c_uint >> 4 as ::core::ffi::c_int) as ::core::ffi::c_char;
    let c2rust_fresh10 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh10 as isize) = nr2hex(c as ::core::ffi::c_uint) as ::core::ffi::c_char;
    let c2rust_fresh11 = i;
    i = i.wrapping_add(1);
    *buf.offset(c2rust_fresh11 as isize) = '>' as ::core::ffi::c_char;
    *buf.offset(i as isize) = NUL as ::core::ffi::c_char;
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn rl_mirror_ascii(
    mut str: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
) {
    let mut p1: *mut ::core::ffi::c_char = str;
    let mut p2: *mut ::core::ffi::c_char = (if !end.is_null() {
        end
    } else {
        str.offset(strlen(str) as isize)
    })
    .offset(-(1 as ::core::ffi::c_int as isize));
    while p1 < p2 {
        let mut t: ::core::ffi::c_char = *p1;
        *p1 = *p2;
        *p2 = t;
        p1 = p1.offset(1);
        p2 = p2.offset(-1);
    }
}
#[inline]
unsafe extern "C" fn nr2hex(mut n: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    if n & 0xf as ::core::ffi::c_uint <= 9 as ::core::ffi::c_uint {
        return (n & 0xf as ::core::ffi::c_uint).wrapping_add('0' as ::core::ffi::c_uint);
    }
    return (n & 0xf as ::core::ffi::c_uint)
        .wrapping_sub(10 as ::core::ffi::c_uint)
        .wrapping_add('a' as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn byte2cells(mut b: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if b >= 0x80 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return g_chartab[b as usize] as ::core::ffi::c_int & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn char2cells(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c < 0 as ::core::ffi::c_int {
        return char2cells(
            if c == K_SPECIAL {
                KS_SPECIAL
            } else {
                if c == NUL {
                    KS_ZERO
                } else {
                    -c & 0xff as ::core::ffi::c_int
                }
            },
        ) + 2 as ::core::ffi::c_int;
    }
    if c >= 0x80 as ::core::ffi::c_int {
        return utf_char2cells(c);
    }
    return g_chartab[(c & 0xff as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
        & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn ptr2cells(mut p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = p_in as *mut uint8_t;
    if *p as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
        return utf_ptr2cells(p_in);
    }
    return g_chartab[*p as usize] as ::core::ffi::c_int & CT_CELL_MASK;
}
#[no_mangle]
pub unsafe extern "C" fn vim_strsize(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return vim_strnsize(s, MAXCOL as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn vim_strnsize(
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !s.is_null() {
        } else {
            __assert_fail(
                b"s != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/charset.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                766 as ::core::ffi::c_uint,
                b"int vim_strnsize(const char *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *s as ::core::ffi::c_int != NUL && {
        len -= 1;
        len >= 0 as ::core::ffi::c_int
    } {
        let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
        size += ptr2cells(s);
        s = s.offset(l as isize);
        len -= l - 1 as ::core::ffi::c_int;
    }
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isIDc(mut c: ::core::ffi::c_int) -> bool {
    return c > 0 as ::core::ffi::c_int
        && c < 0x100 as ::core::ffi::c_int
        && g_chartab[c as usize] as ::core::ffi::c_int & CT_ID_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc(c: ::core::ffi::c_int) -> bool {
    return vim_iswordc_buf(c, curbuf);
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc_tab(c: ::core::ffi::c_int, chartab: *const uint64_t) -> bool {
    return if c >= 0x100 as ::core::ffi::c_int {
        (utf_class_tab(c, chartab) >= 2 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        (c > 0 as ::core::ffi::c_int
            && *chartab.offset((c as ::core::ffi::c_uint >> 6 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << (c & 0x3f as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_ulonglong) as ::core::ffi::c_int
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordc_buf(c: ::core::ffi::c_int, buf: *mut buf_T) -> bool {
    return vim_iswordc_tab(c, &raw mut (*buf).b_chartab as *mut uint64_t);
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool {
    return vim_iswordp_buf(p, curbuf);
}
#[no_mangle]
pub unsafe extern "C" fn vim_iswordp_buf(p: *const ::core::ffi::c_char, buf: *mut buf_T) -> bool {
    let mut c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
    if utf8len_tab[c as usize] as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
        c = utf_ptr2char(p);
    }
    return vim_iswordc_buf(c, buf);
}
#[no_mangle]
pub unsafe extern "C" fn vim_isfilec(mut c: ::core::ffi::c_int) -> bool {
    return c >= 0x100 as ::core::ffi::c_int
        || c > 0 as ::core::ffi::c_int
            && g_chartab[c as usize] as ::core::ffi::c_int & CT_FNAME_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_is_fname_char(mut c: ::core::ffi::c_int) -> bool {
    return vim_isfilec(c) as ::core::ffi::c_int != 0
        || c == ',' as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int
        || c == '@' as ::core::ffi::c_int
        || c == ':' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isfilec_or_wc(mut c: ::core::ffi::c_int) -> bool {
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return vim_isfilec(c) as ::core::ffi::c_int != 0
        || c == ']' as ::core::ffi::c_int
        || path_has_wildcard(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isprintc(mut c: ::core::ffi::c_int) -> bool {
    if c >= 0x100 as ::core::ffi::c_int {
        return utf_printable(c);
    }
    return c > 0 as ::core::ffi::c_int
        && g_chartab[c as usize] as ::core::ffi::c_int & CT_PRINT_CHAR != 0;
}
#[no_mangle]
pub unsafe extern "C" fn skipwhite(mut p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    while ascii_iswhite(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skipwhite_len(
    mut p: *const ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    while len > 0 as size_t && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        p = p.offset(1);
        len = len.wrapping_sub(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn getwhitecols_curline() -> intptr_t {
    return getwhitecols(get_cursor_line_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn getwhitecols(mut p: *const ::core::ffi::c_char) -> intptr_t {
    return skipwhite(p).offset_from(p) as intptr_t;
}
#[no_mangle]
pub unsafe extern "C" fn skipdigits(mut q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while ascii_isdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skipbin(mut q: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while ascii_isbdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiphex(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while ascii_isxdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptodigit(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptobin(
    mut q: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isbdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptohex(mut q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = q;
    while *p as ::core::ffi::c_int != NUL && !ascii_isxdigit(*p as ::core::ffi::c_int) {
        p = p.offset(1);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn skiptowhite(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != NUL
    {
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skiptowhite_esc(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != NUL
    {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn skip_to_newline(
    p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return xstrchrnul(p, NL as ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn try_getdigits(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut nr: *mut intmax_t,
) -> bool {
    *__errno_location() = 0 as ::core::ffi::c_int;
    *nr = strtoimax(*pp, pp, 10 as ::core::ffi::c_int);
    if *__errno_location() == ERANGE
        && (*nr == INTMAX_MIN as intmax_t || *nr == INTMAX_MAX as intmax_t)
    {
        return false_0 != 0;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: intmax_t,
) -> intmax_t {
    let mut number: intmax_t = 0;
    let mut ok: ::core::ffi::c_int = try_getdigits(pp, &raw mut number) as ::core::ffi::c_int;
    if strict as ::core::ffi::c_int != 0 && ok == 0 {
        abort();
    }
    return if ok != 0 { number } else { def };
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_int(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    if strict {
        '_c2rust_label: {
            if number >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as intmax_t
                && number <= 2147483647 as intmax_t
            {
            } else {
                __assert_fail(
                    b"number >= INT_MIN && number <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/charset.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1134 as ::core::ffi::c_uint,
                    b"int getdigits_int(char **, _Bool, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    } else if !(number >= INT_MIN as intmax_t && number <= INT_MAX as intmax_t) {
        return def;
    }
    return number as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_long(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: ::core::ffi::c_long,
) -> ::core::ffi::c_long {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    return number as ::core::ffi::c_long;
}
#[no_mangle]
pub unsafe extern "C" fn getdigits_int32(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut strict: bool,
    mut def: int32_t,
) -> int32_t {
    let mut number: intmax_t = getdigits(pp, strict, def as intmax_t);
    if strict {
        '_c2rust_label: {
            if number >= (-2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as intmax_t
                && number <= 2147483647 as intmax_t
            {
            } else {
                __assert_fail(
                    b"number >= INT32_MIN && number <= INT32_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/charset.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1166 as ::core::ffi::c_uint,
                    b"int32_t getdigits_int32(char **, _Bool, int32_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    } else if !(number >= INT32_MIN as intmax_t && number <= INT32_MAX as intmax_t) {
        return def;
    }
    return number as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn vim_isblankline(mut lbuf: *mut ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = skipwhite(lbuf);
    return *p as ::core::ffi::c_int == NUL
        || *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_str2nr(
    start: *const ::core::ffi::c_char,
    prep: *mut ::core::ffi::c_int,
    len: *mut ::core::ffi::c_int,
    what: ::core::ffi::c_int,
    nptr: *mut varnumber_T,
    unptr: *mut uvarnumber_T,
    maxlen: ::core::ffi::c_int,
    strict: bool,
    overflow: *mut bool,
) {
    let mut ptr: *const ::core::ffi::c_char = start;
    let mut pre: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let negative: bool = *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '-' as ::core::ffi::c_int;
    let mut un: uvarnumber_T = 0 as uvarnumber_T;
    if !len.is_null() {
        *len = 0 as ::core::ffi::c_int;
    }
    if negative {
        ptr = ptr.offset(1);
    }
    '_vim_str2nr_proceed: {
        '_vim_str2nr_oct: {
            '_vim_str2nr_bin: {
                '_vim_str2nr_hex: {
                    '_vim_str2nr_dec: {
                        if what & STR2NR_FORCE as ::core::ffi::c_int != 0 {
                            match what
                                & !(STR2NR_FORCE as ::core::ffi::c_int
                                    | STR2NR_QUOTE as ::core::ffi::c_int)
                            {
                                4 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'x' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'X' as ::core::ffi::c_int)
                                        && ascii_isxdigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_hex;
                                }
                                1 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'b' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'B' as ::core::ffi::c_int)
                                        && ascii_isbdigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_bin;
                                }
                                2 | 8 | 10 => {
                                    if (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr
                                            .offset(2 as ::core::ffi::c_int as isize)
                                            .offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && *ptr.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        && (*ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'o' as ::core::ffi::c_int
                                            || *ptr.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == 'O' as ::core::ffi::c_int)
                                        && ascii_isodigit(
                                            *ptr.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                    }
                                    break '_vim_str2nr_oct;
                                }
                                0 => {}
                                _ => {
                                    abort();
                                }
                            }
                        } else if what
                            & (STR2NR_HEX as ::core::ffi::c_int
                                | STR2NR_OCT as ::core::ffi::c_int
                                | STR2NR_OOCT as ::core::ffi::c_int
                                | STR2NR_BIN as ::core::ffi::c_int)
                            != 0
                            && (maxlen == 0 as ::core::ffi::c_int
                                || (ptr
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    .offset_from(start)
                                    as ::core::ffi::c_int)
                                    < maxlen)
                            && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '0' as ::core::ffi::c_int
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '8' as ::core::ffi::c_int
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != '9' as ::core::ffi::c_int
                        {
                            pre = *ptr.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int;
                            if what & STR2NR_HEX as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'X' as ::core::ffi::c_int
                                    || pre == 'x' as ::core::ffi::c_int)
                                && ascii_isxdigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_hex;
                            } else if what & STR2NR_BIN as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'B' as ::core::ffi::c_int
                                    || pre == 'b' as ::core::ffi::c_int)
                                && ascii_isbdigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_bin;
                            } else if what & STR2NR_OOCT as ::core::ffi::c_int != 0
                                && (maxlen == 0 as ::core::ffi::c_int
                                    || (ptr
                                        .offset(2 as ::core::ffi::c_int as isize)
                                        .offset_from(start)
                                        as ::core::ffi::c_int)
                                        < maxlen)
                                && (pre == 'O' as ::core::ffi::c_int
                                    || pre == 'o' as ::core::ffi::c_int)
                                && ascii_isodigit(*ptr.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
                                break '_vim_str2nr_oct;
                            } else {
                                pre = 0 as ::core::ffi::c_int;
                                if !(what & STR2NR_OCT as ::core::ffi::c_int == 0
                                    || !ascii_isodigit(
                                        *ptr.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ))
                                {
                                    let mut i: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
                                    while (maxlen == 0 as ::core::ffi::c_int
                                        || (ptr.offset(i as isize).offset_from(start)
                                            as ::core::ffi::c_int)
                                            < maxlen)
                                        && ascii_isdigit(
                                            *ptr.offset(i as isize) as ::core::ffi::c_int
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        if *ptr.offset(i as isize) as ::core::ffi::c_int
                                            > '7' as ::core::ffi::c_int
                                        {
                                            break '_vim_str2nr_dec;
                                        }
                                        i += 1;
                                    }
                                    pre = '0' as ::core::ffi::c_int;
                                    break '_vim_str2nr_oct;
                                }
                            }
                        }
                    }
                    let after_prefix_1: *const ::core::ffi::c_char = ptr;
                    while maxlen == 0 as ::core::ffi::c_int
                        || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
                    {
                        if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                            && ptr > after_prefix_1
                            && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                        {
                            ptr = ptr.offset(1);
                            if (maxlen == 0 as ::core::ffi::c_int
                                || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                                && ascii_isdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                continue;
                            }
                            ptr = ptr.offset(-1);
                        }
                        if !ascii_isdigit(*ptr as ::core::ffi::c_int) {
                            break;
                        }
                        let digit_1: uvarnumber_T = (*ptr as ::core::ffi::c_int
                            - '0' as ::core::ffi::c_int)
                            as uvarnumber_T;
                        if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(10 as uvarnumber_T)
                            || un
                                == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(10 as uvarnumber_T)
                                && (10 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                                    || digit_1
                                        <= (UVARNUMBER_MAX as uvarnumber_T)
                                            .wrapping_rem(10 as uvarnumber_T))
                        {
                            un = (10 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_1);
                        } else {
                            un = UVARNUMBER_MAX as uvarnumber_T;
                            if !overflow.is_null() {
                                *overflow = true_0 != 0;
                            }
                        }
                        ptr = ptr.offset(1);
                    }
                    break '_vim_str2nr_proceed;
                }
                let after_prefix_2: *const ::core::ffi::c_char = ptr;
                while maxlen == 0 as ::core::ffi::c_int
                    || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
                {
                    if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                        && ptr > after_prefix_2
                        && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                    {
                        ptr = ptr.offset(1);
                        if (maxlen == 0 as ::core::ffi::c_int
                            || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                            && ascii_isxdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            continue;
                        }
                        ptr = ptr.offset(-1);
                    }
                    if !ascii_isxdigit(*ptr as ::core::ffi::c_int) {
                        break;
                    }
                    let digit_2: uvarnumber_T = hex2nr(*ptr as ::core::ffi::c_int) as uvarnumber_T;
                    if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(16 as uvarnumber_T)
                        || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(16 as uvarnumber_T)
                            && (16 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                                || digit_2
                                    <= (UVARNUMBER_MAX as uvarnumber_T)
                                        .wrapping_rem(10 as uvarnumber_T))
                    {
                        un = (16 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_2);
                    } else {
                        un = UVARNUMBER_MAX as uvarnumber_T;
                        if !overflow.is_null() {
                            *overflow = true_0 != 0;
                        }
                    }
                    ptr = ptr.offset(1);
                }
                break '_vim_str2nr_proceed;
            }
            let after_prefix: *const ::core::ffi::c_char = ptr;
            while maxlen == 0 as ::core::ffi::c_int
                || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
            {
                if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                    && ptr > after_prefix
                    && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                {
                    ptr = ptr.offset(1);
                    if (maxlen == 0 as ::core::ffi::c_int
                        || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                        && (*ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                            || *ptr as ::core::ffi::c_int == '1' as ::core::ffi::c_int)
                    {
                        continue;
                    }
                    ptr = ptr.offset(-1);
                }
                if !(*ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                    || *ptr as ::core::ffi::c_int == '1' as ::core::ffi::c_int)
                {
                    break;
                }
                let digit: uvarnumber_T =
                    (*ptr as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as uvarnumber_T;
                if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(2 as uvarnumber_T)
                    || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(2 as uvarnumber_T)
                        && (2 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                            || digit
                                <= (UVARNUMBER_MAX as uvarnumber_T)
                                    .wrapping_rem(10 as uvarnumber_T))
                {
                    un = (2 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit);
                } else {
                    un = UVARNUMBER_MAX as uvarnumber_T;
                    if !overflow.is_null() {
                        *overflow = true_0 != 0;
                    }
                }
                ptr = ptr.offset(1);
            }
            break '_vim_str2nr_proceed;
        }
        let after_prefix_0: *const ::core::ffi::c_char = ptr;
        while maxlen == 0 as ::core::ffi::c_int
            || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen
        {
            if what & STR2NR_QUOTE as ::core::ffi::c_int != 0
                && ptr > after_prefix_0
                && *ptr as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
            {
                ptr = ptr.offset(1);
                if (maxlen == 0 as ::core::ffi::c_int
                    || (ptr.offset_from(start) as ::core::ffi::c_int) < maxlen)
                    && ascii_isodigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    continue;
                }
                ptr = ptr.offset(-1);
            }
            if !ascii_isodigit(*ptr as ::core::ffi::c_int) {
                break;
            }
            let digit_0: uvarnumber_T =
                (*ptr as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as uvarnumber_T;
            if un < (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(8 as uvarnumber_T)
                || un == (UVARNUMBER_MAX as uvarnumber_T).wrapping_div(8 as uvarnumber_T)
                    && (8 as ::core::ffi::c_int != 10 as ::core::ffi::c_int
                        || digit_0
                            <= (UVARNUMBER_MAX as uvarnumber_T).wrapping_rem(10 as uvarnumber_T))
            {
                un = (8 as uvarnumber_T).wrapping_mul(un).wrapping_add(digit_0);
            } else {
                un = UVARNUMBER_MAX as uvarnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            }
            ptr = ptr.offset(1);
        }
    }
    if strict as ::core::ffi::c_int != 0
        && ptr.offset_from(start) != maxlen as isize
        && (*ptr as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *ptr as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *ptr as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *ptr as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        return;
    }
    if !prep.is_null() {
        *prep = pre;
    }
    if !len.is_null() {
        *len = ptr.offset_from(start) as ::core::ffi::c_int;
    }
    if !nptr.is_null() {
        if negative {
            if un > VARNUMBER_MAX as uvarnumber_T {
                *nptr = VARNUMBER_MIN as varnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            } else {
                *nptr = -(un as varnumber_T);
            }
        } else {
            if un > VARNUMBER_MAX as uvarnumber_T {
                un = VARNUMBER_MAX as uvarnumber_T;
                if !overflow.is_null() {
                    *overflow = true_0 != 0;
                }
            }
            *nptr = un as varnumber_T;
        }
    }
    if !unptr.is_null() {
        *unptr = un;
    }
}
#[no_mangle]
pub unsafe extern "C" fn hex2nr(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int {
        return c - 'a' as ::core::ffi::c_int + 10 as ::core::ffi::c_int;
    }
    if c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int {
        return c - 'A' as ::core::ffi::c_int + 10 as ::core::ffi::c_int;
    }
    return c - '0' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn hexhex2nr(mut p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if !ascii_isxdigit(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        || !ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        return -1 as ::core::ffi::c_int;
    }
    return (hex2nr(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        << 4 as ::core::ffi::c_int)
        + hex2nr(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn rem_backslash(mut str: *const ::core::ffi::c_char) -> bool {
    return *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\\' as ::core::ffi::c_int
        && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
#[no_mangle]
pub unsafe extern "C" fn backslash_halve(mut p: *mut ::core::ffi::c_char) {
    while *p as ::core::ffi::c_int != 0 && !rem_backslash(p) {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int != NUL {
        let mut dst: *mut ::core::ffi::c_char = p;
        's_50: loop {
            let c2rust_fresh16 = dst;
            dst = dst.offset(1);
            *c2rust_fresh16 = *p.offset(1 as ::core::ffi::c_int as isize);
            p = p.offset(2 as ::core::ffi::c_int as isize);
            loop {
                if *p as ::core::ffi::c_int == NUL {
                    break 's_50;
                }
                if rem_backslash(p) {
                    break;
                }
                let c2rust_fresh17 = p;
                p = p.offset(1);
                let c2rust_fresh18 = dst;
                dst = dst.offset(1);
                *c2rust_fresh18 = *c2rust_fresh17;
            }
        }
        *dst = NUL as ::core::ffi::c_char;
    }
}
#[no_mangle]
pub unsafe extern "C" fn backslash_halve_save(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut res: *mut ::core::ffi::c_char =
        xmalloc(strlen(p).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut dst: *mut ::core::ffi::c_char = res;
    while *p as ::core::ffi::c_int != NUL {
        if rem_backslash(p) {
            let c2rust_fresh19 = dst;
            dst = dst.offset(1);
            *c2rust_fresh19 = *p.offset(1 as ::core::ffi::c_int as isize);
            p = p.offset(2 as ::core::ffi::c_int as isize);
        } else {
            let c2rust_fresh20 = p;
            p = p.offset(1);
            let c2rust_fresh21 = dst;
            dst = dst.offset(1);
            *c2rust_fresh21 = *c2rust_fresh20;
        }
    }
    *dst = NUL as ::core::ffi::c_char;
    return res;
}
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
