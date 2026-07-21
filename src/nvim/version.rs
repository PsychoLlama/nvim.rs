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
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn api_free_object(value: Object);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    static mut p_ls: OptInt;
    static mut p_shm: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn screenclear();
    fn plain_vgetc() -> ::core::ffi::c_int;
    static mut default_gridview: GridView;
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_flush();
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut topframe: *mut frame_T;
    static mut curbuf: *mut buf_T;
    static mut starting: ::core::ffi::c_int;
    static mut got_int: bool;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn syn_name2id(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_puts(s: *const ::core::ffi::c_char);
    static default_vim_dir: GlobalCell<*mut ::core::ffi::c_char>;
    static default_vimruntime_dir: GlobalCell<*mut ::core::ffi::c_char>;
    fn ui_has(ext: UIExtension) -> bool;
    fn one_window(win: *mut win_T, tp: *mut tabpage_T) -> bool;
}
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
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
    pub cs_pend: C2Rust_Unnamed_14,
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
pub union C2Rust_Unnamed_14 {
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_15 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_15 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_15 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_15 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_15 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_15 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_15 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_15 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_15 = 79;
pub const SHM_OVER: C2Rust_Unnamed_15 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_15 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_15 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_15 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_15 = 97;
pub const SHM_WRI: C2Rust_Unnamed_15 = 119;
pub const SHM_LINES: C2Rust_Unnamed_15 = 108;
pub const SHM_MOD: C2Rust_Unnamed_15 = 109;
pub const SHM_RO: C2Rust_Unnamed_15 = 114;
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
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const LOWEST_WIN_ID: C2Rust_Unnamed_16 = 1000;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const SYS_VIMRC_FILE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"$VIM/sysinit.vim\0")
};
pub const NVIM_VERSION_LONG: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"NVIM v0.12.4\0") };
#[no_mangle]
pub static Versions: GlobalCell<[*mut ::core::ffi::c_char; 5]> = GlobalCell::new([
    b"8.1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"8.2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"9.0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"9.1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"9.2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
#[no_mangle]
pub static longVersion: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(NVIM_VERSION_LONG.as_ptr() as *mut ::core::ffi::c_char);
#[no_mangle]
pub static version_buildtype: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"Build type: Debug\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
#[no_mangle]
pub static version_cflags: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(b"Compilation: /nix/store/vr15iyyykg9zai6fpgvhcgyw7gckl78w-gcc-wrapper-14.3.0/bin/gcc -g  -Wall -Wextra -pedantic -Wno-unused-parameter -Wstrict-prototypes -std=gnu99 -Wshadow -Wconversion -Wvla -Wdouble-promotion -Wmissing-noreturn -Wmissing-format-attribute -Wmissing-prototypes -Wno-unused-function -fsigned-char -fstack-protector-strong -Wno-conversion -fno-common -Wimplicit-fallthrough -fdiagnostics-color=always -Wno-free-nonheap-object -DHAVE_UNIBILIUM -DNVIM_LOG_DEBUG -DUNIT_TESTING -D_GNU_SOURCE -DUTF8PROC_STATIC -I.deps/usr/include/luajit-2.1 -I.deps/usr/include -Ibuild/src/nvim/auto -Ibuild/include -Ibuild/cmake.config -Isrc -I/nix/store/l1fi677mcxsa175gf0zvpk68kf1calbn-glibc-iconv-2.40/include -I/nix/store/gi4cz4ir3zlwhf1azqfgxqdnczfrwsr7-glibc-2.40-66-dev/include \0"
    .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
static vim_versions: GlobalCell<[::core::ffi::c_int; 5]> = GlobalCell::new([
    801 as ::core::ffi::c_int,
    802 as ::core::ffi::c_int,
    900 as ::core::ffi::c_int,
    901 as ::core::ffi::c_int,
    902 as ::core::ffi::c_int,
]);
static num_patches: GlobalCell<[::core::ffi::c_int; 5]> = GlobalCell::new([
    2331 as ::core::ffi::c_int,
    3803 as ::core::ffi::c_int,
    1574 as ::core::ffi::c_int,
    1612 as ::core::ffi::c_int,
    121 as ::core::ffi::c_int,
]);
static c2rust_lvalue: GlobalCell<[::core::ffi::c_int; 2331]> = GlobalCell::new([
    2424, 2423, 2422, 2421, 2420, 2419, 2417, 2416, 2414, 2413, 2412, 2411, 2410, 2409, 2408, 2407,
    2406, 2405, 2404, 2403, 2402, 2401, 2400, 2398, 2397, 2396, 2395, 2394, 2393, 2392, 2391, 2390,
    2389, 2388, 2387, 2385, 2384, 2383, 2382, 2380, 2379, 2378, 2377, 2376, 2375, 2374, 2373, 2372,
    2371, 2370, 2369, 2368, 2367, 2366, 2365, 2364, 2363, 2361, 2360, 2359, 2358, 2357, 2356, 2355,
    2354, 2353, 2352, 2350, 2349, 2348, 2347, 2346, 2345, 2344, 2343, 2342, 2341, 2340, 2339, 2338,
    2337, 2336, 2335, 2334, 2333, 2332, 2331, 2330, 2329, 2328, 2327, 2326, 2325, 2323, 2322, 2321,
    2320, 2319, 2318, 2317, 2316, 2315, 2314, 2313, 2312, 2311, 2310, 2309, 2308, 2307, 2306, 2305,
    2304, 2303, 2302, 2301, 2300, 2299, 2298, 2297, 2296, 2295, 2293, 2291, 2290, 2289, 2288, 2287,
    2286, 2285, 2284, 2283, 2282, 2281, 2280, 2278, 2277, 2276, 2275, 2274, 2272, 2271, 2270, 2269,
    2268, 2267, 2266, 2265, 2264, 2263, 2262, 2261, 2260, 2259, 2258, 2257, 2256, 2255, 2254, 2253,
    2252, 2249, 2248, 2247, 2246, 2245, 2244, 2243, 2242, 2240, 2239, 2238, 2237, 2236, 2235, 2234,
    2233, 2231, 2229, 2228, 2227, 2226, 2225, 2224, 2223, 2222, 2221, 2220, 2218, 2217, 2216, 2215,
    2214, 2213, 2212, 2211, 2210, 2209, 2207, 2206, 2205, 2204, 2203, 2202, 2201, 2200, 2199, 2198,
    2197, 2196, 2194, 2191, 2190, 2188, 2187, 2186, 2185, 2184, 2183, 2182, 2180, 2179, 2178, 2177,
    2175, 2174, 2173, 2172, 2171, 2170, 2169, 2168, 2167, 2166, 2165, 2164, 2163, 2162, 2161, 2160,
    2159, 2157, 2154, 2152, 2151, 2150, 2149, 2148, 2147, 2146, 2145, 2144, 2143, 2141, 2140, 2138,
    2137, 2136, 2135, 2134, 2133, 2132, 2131, 2130, 2129, 2128, 2127, 2126, 2125, 2124, 2123, 2122,
    2120, 2119, 2118, 2117, 2116, 2115, 2114, 2113, 2112, 2111, 2110, 2109, 2108, 2107, 2106, 2105,
    2104, 2103, 2102, 2101, 2100, 2099, 2098, 2097, 2096, 2095, 2094, 2091, 2090, 2089, 2088, 2087,
    2086, 2085, 2084, 2083, 2082, 2081, 2080, 2079, 2078, 2077, 2075, 2074, 2073, 2072, 2071, 2070,
    2069, 2068, 2067, 2066, 2065, 2064, 2063, 2062, 2061, 2060, 2059, 2058, 2057, 2056, 2055, 2054,
    2053, 2052, 2051, 2050, 2049, 2048, 2047, 2046, 2045, 2044, 2043, 2042, 2041, 2038, 2037, 2036,
    2035, 2034, 2033, 2032, 2031, 2030, 2029, 2028, 2027, 2026, 2025, 2024, 2023, 2021, 2020, 2019,
    2018, 2017, 2016, 2015, 2014, 2013, 2012, 2011, 2010, 2009, 2008, 2007, 2006, 2005, 2004, 2003,
    2002, 2001, 2000, 1998, 1997, 1996, 1995, 1994, 1993, 1992, 1991, 1990, 1989, 1988, 1987, 1986,
    1984, 1983, 1982, 1981, 1980, 1979, 1978, 1977, 1976, 1975, 1974, 1973, 1972, 1971, 1970, 1968,
    1967, 1966, 1965, 1964, 1963, 1962, 1961, 1960, 1959, 1958, 1957, 1956, 1955, 1954, 1953, 1952,
    1951, 1950, 1949, 1948, 1947, 1946, 1945, 1943, 1942, 1941, 1940, 1939, 1938, 1937, 1936, 1935,
    1934, 1933, 1932, 1931, 1930, 1929, 1927, 1926, 1925, 1924, 1923, 1922, 1921, 1920, 1919, 1918,
    1917, 1916, 1915, 1914, 1913, 1912, 1911, 1910, 1909, 1907, 1903, 1902, 1901, 1900, 1899, 1898,
    1897, 1896, 1895, 1894, 1893, 1891, 1890, 1889, 1888, 1887, 1886, 1885, 1883, 1881, 1879, 1878,
    1877, 1876, 1875, 1874, 1873, 1872, 1871, 1870, 1869, 1868, 1867, 1866, 1865, 1864, 1863, 1862,
    1861, 1860, 1859, 1858, 1857, 1856, 1855, 1854, 1853, 1852, 1850, 1849, 1848, 1847, 1846, 1845,
    1844, 1843, 1842, 1841, 1840, 1839, 1838, 1837, 1836, 1835, 1834, 1833, 1832, 1831, 1830, 1829,
    1828, 1827, 1826, 1825, 1824, 1823, 1822, 1821, 1820, 1818, 1817, 1816, 1815, 1814, 1812, 1811,
    1810, 1809, 1808, 1807, 1806, 1805, 1804, 1803, 1802, 1801, 1800, 1798, 1797, 1796, 1795, 1794,
    1793, 1792, 1791, 1790, 1789, 1788, 1786, 1785, 1783, 1782, 1781, 1780, 1779, 1778, 1777, 1776,
    1775, 1774, 1773, 1772, 1771, 1769, 1768, 1767, 1766, 1765, 1764, 1763, 1762, 1761, 1760, 1759,
    1758, 1757, 1756, 1755, 1754, 1753, 1752, 1751, 1750, 1749, 1748, 1747, 1746, 1745, 1744, 1743,
    1742, 1741, 1740, 1739, 1738, 1737, 1736, 1735, 1734, 1733, 1732, 1731, 1730, 1729, 1728, 1727,
    1726, 1725, 1724, 1723, 1722, 1721, 1720, 1719, 1717, 1716, 1715, 1712, 1711, 1710, 1709, 1708,
    1707, 1706, 1705, 1704, 1703, 1702, 1701, 1700, 1699, 1698, 1697, 1696, 1695, 1694, 1693, 1692,
    1691, 1689, 1688, 1687, 1686, 1685, 1684, 1683, 1682, 1681, 1680, 1679, 1678, 1677, 1676, 1675,
    1674, 1672, 1671, 1670, 1669, 1668, 1667, 1666, 1665, 1664, 1663, 1662, 1661, 1660, 1658, 1657,
    1656, 1655, 1654, 1653, 1652, 1651, 1650, 1649, 1648, 1647, 1646, 1645, 1644, 1643, 1642, 1641,
    1640, 1639, 1638, 1637, 1636, 1635, 1634, 1633, 1632, 1631, 1630, 1629, 1627, 1625, 1624, 1623,
    1622, 1621, 1620, 1619, 1618, 1617, 1616, 1615, 1614, 1613, 1611, 1610, 1608, 1606, 1605, 1604,
    1603, 1602, 1601, 1600, 1599, 1598, 1596, 1595, 1594, 1593, 1592, 1591, 1590, 1588, 1587, 1586,
    1585, 1584, 1583, 1582, 1581, 1579, 1578, 1576, 1575, 1573, 1572, 1571, 1570, 1569, 1568, 1567,
    1566, 1565, 1564, 1563, 1562, 1560, 1557, 1556, 1555, 1554, 1552, 1551, 1550, 1549, 1547, 1546,
    1545, 1544, 1543, 1542, 1541, 1540, 1539, 1536, 1535, 1534, 1533, 1532, 1531, 1530, 1529, 1528,
    1527, 1526, 1524, 1522, 1521, 1520, 1519, 1518, 1517, 1516, 1515, 1514, 1513, 1512, 1511, 1510,
    1509, 1508, 1507, 1506, 1505, 1504, 1503, 1502, 1501, 1500, 1499, 1498, 1497, 1496, 1495, 1494,
    1493, 1492, 1491, 1490, 1489, 1488, 1487, 1486, 1485, 1484, 1483, 1482, 1481, 1480, 1479, 1478,
    1477, 1476, 1475, 1474, 1472, 1471, 1470, 1469, 1468, 1467, 1466, 1465, 1464, 1463, 1462, 1461,
    1460, 1459, 1458, 1457, 1456, 1455, 1454, 1453, 1452, 1451, 1450, 1449, 1448, 1447, 1446, 1445,
    1444, 1443, 1442, 1441, 1440, 1439, 1438, 1437, 1436, 1435, 1434, 1433, 1432, 1431, 1430, 1429,
    1428, 1427, 1426, 1425, 1424, 1423, 1422, 1421, 1420, 1419, 1418, 1416, 1415, 1414, 1413, 1412,
    1411, 1410, 1409, 1408, 1407, 1406, 1405, 1404, 1403, 1402, 1401, 1400, 1399, 1398, 1397, 1396,
    1395, 1394, 1393, 1392, 1391, 1390, 1389, 1388, 1387, 1386, 1385, 1384, 1383, 1382, 1381, 1380,
    1379, 1378, 1377, 1376, 1375, 1374, 1373, 1372, 1371, 1370, 1369, 1368, 1367, 1366, 1365, 1364,
    1363, 1362, 1361, 1360, 1359, 1358, 1357, 1356, 1355, 1354, 1353, 1352, 1351, 1350, 1349, 1348,
    1347, 1346, 1345, 1344, 1343, 1342, 1341, 1340, 1339, 1338, 1337, 1336, 1335, 1334, 1333, 1332,
    1331, 1330, 1329, 1328, 1327, 1326, 1325, 1324, 1323, 1322, 1321, 1320, 1319, 1318, 1317, 1316,
    1315, 1314, 1313, 1312, 1311, 1310, 1309, 1308, 1307, 1306, 1305, 1304, 1303, 1302, 1301, 1300,
    1299, 1298, 1297, 1296, 1295, 1294, 1293, 1292, 1291, 1290, 1289, 1288, 1287, 1286, 1285, 1284,
    1283, 1282, 1281, 1280, 1279, 1278, 1277, 1276, 1275, 1274, 1273, 1272, 1271, 1270, 1269, 1268,
    1267, 1266, 1265, 1264, 1263, 1262, 1261, 1260, 1259, 1258, 1257, 1256, 1255, 1254, 1253, 1252,
    1251, 1250, 1249, 1248, 1247, 1246, 1245, 1244, 1243, 1242, 1241, 1240, 1239, 1238, 1237, 1236,
    1235, 1234, 1233, 1232, 1231, 1230, 1229, 1228, 1227, 1226, 1225, 1223, 1222, 1221, 1220, 1219,
    1217, 1216, 1215, 1214, 1213, 1212, 1211, 1210, 1209, 1208, 1207, 1206, 1205, 1204, 1203, 1202,
    1201, 1200, 1199, 1198, 1197, 1196, 1195, 1194, 1193, 1192, 1191, 1190, 1189, 1188, 1187, 1186,
    1185, 1184, 1183, 1182, 1181, 1180, 1179, 1178, 1177, 1176, 1175, 1174, 1173, 1172, 1171, 1170,
    1169, 1168, 1167, 1166, 1165, 1164, 1163, 1162, 1161, 1160, 1159, 1158, 1157, 1156, 1155, 1154,
    1153, 1152, 1151, 1150, 1149, 1148, 1147, 1146, 1145, 1144, 1143, 1142, 1141, 1140, 1139, 1138,
    1137, 1136, 1135, 1134, 1133, 1132, 1131, 1130, 1129, 1128, 1127, 1126, 1125, 1124, 1123, 1122,
    1121, 1120, 1119, 1118, 1117, 1116, 1115, 1114, 1113, 1112, 1111, 1110, 1109, 1108, 1107, 1106,
    1105, 1104, 1103, 1102, 1101, 1100, 1099, 1098, 1097, 1096, 1095, 1094, 1093, 1092, 1091, 1090,
    1089, 1088, 1087, 1086, 1085, 1084, 1083, 1082, 1081, 1080, 1079, 1078, 1077, 1076, 1075, 1074,
    1073, 1072, 1071, 1070, 1069, 1068, 1067, 1066, 1065, 1064, 1063, 1062, 1061, 1060, 1059, 1058,
    1057, 1056, 1055, 1054, 1053, 1052, 1051, 1050, 1049, 1048, 1047, 1046, 1045, 1043, 1042, 1041,
    1040, 1039, 1038, 1037, 1036, 1035, 1034, 1033, 1032, 1031, 1030, 1029, 1028, 1027, 1026, 1025,
    1024, 1023, 1022, 1021, 1020, 1019, 1018, 1017, 1016, 1015, 1014, 1013, 1012, 1011, 1010, 1009,
    1008, 1007, 1006, 1005, 1004, 1003, 1002, 1001, 1000, 999, 998, 997, 996, 995, 994, 993, 992,
    991, 990, 989, 988, 987, 986, 985, 984, 983, 982, 981, 980, 979, 978, 977, 976, 975, 974, 973,
    972, 971, 970, 969, 968, 967, 966, 965, 964, 963, 962, 961, 960, 959, 958, 957, 956, 955, 954,
    952, 951, 950, 949, 948, 947, 946, 945, 944, 943, 942, 941, 940, 939, 938, 937, 936, 935, 934,
    933, 932, 931, 930, 929, 928, 927, 926, 925, 924, 923, 922, 921, 920, 919, 918, 917, 916, 915,
    914, 913, 912, 911, 910, 909, 908, 907, 906, 905, 904, 903, 902, 901, 900, 899, 898, 897, 896,
    895, 893, 892, 891, 890, 889, 888, 887, 886, 885, 884, 883, 882, 881, 880, 879, 878, 877, 875,
    874, 873, 872, 871, 870, 869, 868, 867, 866, 865, 864, 862, 861, 860, 859, 858, 857, 856, 855,
    854, 853, 852, 851, 850, 849, 848, 847, 846, 845, 844, 843, 842, 841, 840, 839, 838, 837, 836,
    835, 834, 833, 832, 831, 830, 829, 828, 827, 826, 825, 824, 823, 822, 821, 820, 819, 818, 817,
    816, 815, 814, 813, 812, 811, 810, 809, 808, 807, 806, 805, 804, 803, 802, 801, 800, 799, 798,
    797, 796, 795, 794, 793, 792, 791, 790, 789, 788, 787, 786, 785, 784, 783, 782, 781, 780, 779,
    778, 777, 776, 775, 774, 773, 772, 771, 770, 769, 767, 766, 765, 764, 763, 762, 761, 760, 759,
    758, 757, 756, 755, 754, 753, 752, 751, 750, 749, 748, 747, 746, 745, 744, 743, 742, 741, 740,
    739, 738, 737, 736, 735, 734, 733, 732, 731, 730, 729, 728, 727, 726, 725, 724, 723, 722, 721,
    720, 719, 718, 717, 716, 715, 714, 713, 712, 711, 710, 709, 708, 707, 706, 705, 704, 703, 702,
    701, 700, 699, 698, 697, 696, 695, 694, 693, 692, 691, 690, 689, 688, 687, 686, 685, 684, 683,
    682, 681, 680, 679, 678, 677, 676, 675, 674, 673, 672, 671, 670, 669, 668, 667, 666, 665, 664,
    663, 662, 661, 660, 659, 658, 657, 656, 655, 654, 653, 652, 651, 650, 649, 648, 647, 646, 645,
    644, 643, 642, 641, 640, 639, 638, 637, 636, 635, 634, 633, 632, 631, 630, 629, 628, 627, 626,
    625, 624, 623, 622, 621, 620, 619, 618, 617, 616, 615, 614, 613, 612, 611, 610, 609, 608, 607,
    606, 605, 604, 603, 602, 601, 600, 599, 598, 597, 596, 595, 594, 593, 592, 591, 590, 589, 588,
    587, 586, 585, 584, 583, 582, 581, 580, 579, 578, 577, 576, 575, 574, 573, 572, 571, 570, 569,
    568, 567, 566, 565, 564, 563, 562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 551, 550,
    549, 548, 547, 546, 545, 544, 543, 542, 541, 540, 539, 538, 537, 536, 535, 534, 533, 532, 531,
    530, 529, 528, 527, 526, 525, 524, 523, 522, 521, 520, 519, 518, 517, 516, 515, 514, 513, 512,
    511, 510, 509, 508, 507, 506, 505, 504, 503, 502, 501, 500, 499, 498, 497, 496, 495, 494, 493,
    492, 491, 490, 489, 488, 487, 486, 485, 484, 483, 482, 481, 480, 479, 478, 477, 476, 475, 474,
    473, 472, 471, 470, 469, 468, 467, 466, 465, 464, 463, 462, 461, 460, 459, 458, 457, 456, 455,
    454, 453, 452, 451, 450, 449, 448, 447, 446, 445, 444, 443, 442, 441, 440, 439, 438, 437, 436,
    435, 434, 433, 432, 431, 430, 429, 428, 427, 426, 425, 424, 423, 422, 421, 420, 419, 418, 417,
    416, 415, 414, 413, 412, 411, 410, 409, 408, 407, 406, 405, 404, 403, 402, 401, 400, 399, 398,
    397, 396, 395, 394, 393, 392, 391, 390, 389, 388, 387, 386, 385, 384, 383, 382, 381, 380, 379,
    378, 377, 376, 375, 374, 373, 372, 371, 370, 369, 368, 367, 366, 365, 364, 363, 362, 361, 360,
    359, 358, 357, 356, 355, 354, 353, 352, 351, 350, 349, 348, 347, 346, 345, 344, 343, 342, 341,
    340, 339, 338, 337, 336, 335, 334, 333, 332, 331, 330, 329, 328, 327, 326, 325, 324, 323, 322,
    321, 320, 319, 318, 317, 316, 315, 314, 313, 312, 311, 310, 309, 308, 307, 306, 305, 304, 303,
    302, 301, 300, 299, 298, 297, 296, 295, 294, 293, 292, 291, 290, 289, 288, 287, 286, 285, 284,
    283, 282, 281, 280, 279, 278, 277, 276, 275, 274, 273, 272, 271, 270, 269, 268, 267, 266, 265,
    264, 263, 262, 261, 260, 259, 258, 257, 256, 255, 254, 253, 252, 251, 250, 249, 248, 247, 246,
    245, 244, 243, 242, 241, 240, 239, 238, 237, 236, 235, 234, 233, 232, 231, 230, 229, 228, 227,
    226, 225, 224, 223, 222, 221, 220, 219, 218, 217, 216, 215, 214, 213, 212, 211, 210, 209, 208,
    207, 206, 205, 204, 203, 202, 201, 200, 199, 198, 197, 196, 195, 194, 193, 192, 191, 190, 189,
    188, 187, 186, 185, 184, 183, 182, 181, 180, 179, 178, 177, 176, 175, 174, 173, 172, 171, 170,
    169, 168, 167, 166, 165, 164, 163, 162, 161, 160, 159, 158, 157, 156, 155, 154, 153, 152, 151,
    150, 149, 148, 147, 146, 145, 144, 143, 142, 141, 140, 139, 138, 137, 136, 135, 134, 133, 132,
    131, 130, 129, 128, 127, 126, 125, 124, 123, 122, 121, 120, 119, 118, 117, 116, 115, 114, 113,
    112, 111, 110, 109, 108, 107, 106, 105, 104, 103, 102, 101, 100, 99, 98, 97, 96, 95, 94, 93,
    92, 91, 90, 89, 88, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 69, 68,
    67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44,
    43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20,
    19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
]);
static c2rust_lvalue_0: GlobalCell<[::core::ffi::c_int; 3803]> = GlobalCell::new([
    5171, 5170, 5168, 5167, 5166, 5165, 5164, 5163, 5162, 5161, 5159, 5158, 5157, 5155, 5154, 5153,
    5152, 5151, 5150, 5149, 5148, 5146, 5145, 5142, 5138, 5137, 5135, 5133, 5132, 5130, 5126, 5125,
    5123, 5122, 5121, 5120, 5119, 5117, 5116, 5110, 5109, 5108, 5107, 5106, 5105, 5104, 5103, 5102,
    5101, 5099, 5098, 5097, 5096, 5095, 5092, 5091, 5090, 5088, 5087, 5086, 5085, 5083, 5082, 5081,
    5080, 5079, 5078, 5077, 5076, 5075, 5074, 5073, 5072, 5071, 5070, 5069, 5068, 5067, 5066, 5064,
    5063, 5060, 5059, 5058, 5055, 5054, 5052, 5050, 5047, 5046, 5045, 5044, 5043, 5042, 5041, 5040,
    5038, 5037, 5035, 5034, 5031, 5029, 5027, 5025, 5024, 5023, 5022, 5021, 5019, 5018, 5017, 5016,
    5015, 5014, 5013, 5012, 5010, 5009, 5008, 5007, 5005, 5002, 5001, 4999, 4998, 4996, 4995, 4993,
    4991, 4990, 4989, 4987, 4986, 4985, 4984, 4980, 4979, 4978, 4977, 4976, 4975, 4974, 4973, 4972,
    4969, 4968, 4963, 4961, 4960, 4959, 4957, 4956, 4954, 4953, 4951, 4950, 4949, 4948, 4947, 4946,
    4944, 4943, 4941, 4940, 4939, 4938, 4936, 4935, 4934, 4932, 4931, 4930, 4929, 4928, 4926, 4925,
    4924, 4923, 4922, 4921, 4920, 4919, 4918, 4917, 4916, 4914, 4911, 4910, 4908, 4907, 4905, 4904,
    4903, 4901, 4900, 4899, 4898, 4895, 4893, 4892, 4890, 4889, 4885, 4884, 4883, 4882, 4881, 4878,
    4874, 4871, 4868, 4867, 4866, 4865, 4864, 4863, 4862, 4861, 4860, 4859, 4858, 4857, 4856, 4855,
    4853, 4851, 4850, 4849, 4848, 4846, 4845, 4844, 4843, 4842, 4841, 4840, 4839, 4838, 4837, 4836,
    4835, 4834, 4833, 4831, 4830, 4829, 4828, 4827, 4826, 4825, 4824, 4823, 4820, 4819, 4818, 4817,
    4816, 4815, 4813, 4812, 4811, 4810, 4809, 4808, 4807, 4806, 4805, 4802, 4801, 4799, 4797, 4796,
    4795, 4794, 4793, 4792, 4791, 4790, 4788, 4787, 4786, 4785, 4783, 4781, 4779, 4778, 4776, 4773,
    4772, 4771, 4770, 4769, 4768, 4767, 4766, 4765, 4764, 4763, 4762, 4760, 4759, 4757, 4756, 4754,
    4753, 4752, 4750, 4749, 4747, 4746, 4745, 4744, 4743, 4741, 4740, 4739, 4738, 4737, 4735, 4734,
    4733, 4732, 4731, 4730, 4729, 4728, 4726, 4725, 4724, 4723, 4722, 4721, 4720, 4719, 4718, 4716,
    4715, 4714, 4713, 4711, 4710, 4708, 4707, 4706, 4705, 4704, 4703, 4702, 4701, 4700, 4699, 4697,
    4696, 4695, 4693, 4692, 4691, 4690, 4689, 4688, 4687, 4686, 4685, 4681, 4680, 4679, 4678, 4677,
    4676, 4675, 4674, 4672, 4671, 4670, 4668, 4667, 4666, 4665, 4664, 4661, 4660, 4658, 4655, 4654,
    4652, 4651, 4649, 4647, 4646, 4645, 4644, 4643, 4641, 4640, 4639, 4638, 4637, 4636, 4633, 4632,
    4631, 4630, 4629, 4628, 4627, 4626, 4625, 4624, 4623, 4621, 4620, 4618, 4617, 4614, 4613, 4612,
    4611, 4609, 4608, 4607, 4603, 4602, 4601, 4600, 4599, 4598, 4596, 4595, 4594, 4593, 4592, 4591,
    4590, 4587, 4585, 4583, 4581, 4580, 4579, 4577, 4574, 4572, 4571, 4570, 4569, 4568, 4566, 4565,
    4564, 4563, 4562, 4561, 4560, 4559, 4558, 4557, 4556, 4555, 4554, 4553, 4552, 4550, 4549, 4548,
    4547, 4546, 4545, 4544, 4543, 4542, 4541, 4538, 4537, 4533, 4532, 4531, 4529, 4528, 4527, 4524,
    4523, 4521, 4520, 4518, 4517, 4516, 4513, 4512, 4507, 4504, 4502, 4501, 4498, 4497, 4496, 4495,
    4494, 4492, 4491, 4488, 4486, 4485, 4484, 4483, 4482, 4481, 4479, 4478, 4477, 4476, 4475, 4474,
    4473, 4471, 4470, 4469, 4468, 4467, 4466, 4465, 4464, 4463, 4462, 4461, 4456, 4455, 4454, 4453,
    4452, 4451, 4450, 4447, 4446, 4445, 4444, 4443, 4442, 4440, 4438, 4437, 4436, 4434, 4433, 4432,
    4428, 4427, 4424, 4423, 4422, 4421, 4420, 4419, 4418, 4417, 4416, 4414, 4413, 4412, 4411, 4410,
    4409, 4407, 4406, 4405, 4404, 4403, 4402, 4401, 4400, 4399, 4398, 4397, 4396, 4395, 4394, 4393,
    4392, 4391, 4390, 4389, 4388, 4387, 4386, 4385, 4383, 4382, 4379, 4378, 4377, 4376, 4374, 4373,
    4370, 4369, 4368, 4366, 4364, 4363, 4362, 4361, 4360, 4359, 4358, 4356, 4355, 4353, 4352, 4351,
    4349, 4346, 4345, 4343, 4342, 4341, 4340, 4339, 4338, 4337, 4336, 4335, 4334, 4331, 4329, 4328,
    4327, 4326, 4325, 4324, 4323, 4320, 4319, 4318, 4316, 4315, 4314, 4313, 4312, 4311, 4310, 4309,
    4307, 4306, 4305, 4303, 4300, 4299, 4298, 4297, 4296, 4294, 4292, 4290, 4289, 4288, 4287, 4285,
    4284, 4283, 4281, 4280, 4278, 4275, 4274, 4273, 4272, 4271, 4268, 4267, 4262, 4261, 4258, 4254,
    4253, 4251, 4249, 4248, 4247, 4245, 4244, 4242, 4241, 4239, 4238, 4237, 4236, 4235, 4234, 4233,
    4230, 4228, 4227, 4222, 4221, 4220, 4219, 4218, 4217, 4216, 4215, 4214, 4213, 4212, 4211, 4210,
    4208, 4207, 4206, 4204, 4203, 4200, 4198, 4197, 4196, 4194, 4193, 4191, 4190, 4189, 4188, 4187,
    4186, 4185, 4184, 4183, 4182, 4181, 4180, 4179, 4178, 4177, 4176, 4173, 4172, 4169, 4168, 4166,
    4165, 4163, 4161, 4160, 4159, 4158, 4156, 4155, 4154, 4152, 4151, 4150, 4146, 4144, 4143, 4142,
    4141, 4140, 4139, 4138, 4137, 4134, 4133, 4132, 4131, 4130, 4129, 4127, 4126, 4124, 4121, 4120,
    4119, 4116, 4115, 4112, 4110, 4109, 4103, 4102, 4100, 4098, 4097, 4096, 4095, 4094, 4093, 4092,
    4091, 4090, 4089, 4088, 4085, 4081, 4080, 4079, 4078, 4077, 4076, 4075, 4074, 4073, 4072, 4071,
    4070, 4066, 4065, 4064, 4062, 4061, 4060, 4058, 4057, 4055, 4054, 4052, 4049, 4048, 4043, 4042,
    4038, 4037, 4036, 4034, 4033, 4032, 4029, 4028, 4027, 4026, 4023, 4021, 4020, 4018, 4017, 4016,
    4015, 4014, 4013, 4007, 4002, 4001, 3999, 3998, 3995, 3993, 3992, 3990, 3989, 3988, 3984, 3982,
    3981, 3980, 3978, 3974, 3973, 3969, 3968, 3966, 3964, 3963, 3962, 3958, 3956, 3954, 3953, 3952,
    3951, 3950, 3949, 3948, 3947, 3946, 3944, 3943, 3942, 3940, 3939, 3938, 3937, 3936, 3935, 3934,
    3933, 3932, 3931, 3926, 3925, 3922, 3921, 3920, 3919, 3917, 3915, 3914, 3912, 3909, 3908, 3906,
    3905, 3903, 3900, 3898, 3896, 3893, 3891, 3889, 3888, 3887, 3886, 3885, 3884, 3883, 3882, 3880,
    3879, 3878, 3876, 3875, 3874, 3873, 3870, 3869, 3868, 3867, 3865, 3863, 3862, 3861, 3860, 3858,
    3856, 3855, 3853, 3852, 3850, 3848, 3846, 3845, 3844, 3843, 3841, 3839, 3838, 3834, 3833, 3831,
    3829, 3827, 3826, 3825, 3824, 3823, 3822, 3821, 3820, 3819, 3818, 3814, 3813, 3811, 3808, 3806,
    3805, 3804, 3803, 3802, 3800, 3797, 3796, 3795, 3793, 3792, 3791, 3790, 3788, 3787, 3786, 3785,
    3784, 3783, 3782, 3781, 3780, 3779, 3778, 3777, 3776, 3775, 3774, 3773, 3772, 3770, 3769, 3768,
    3767, 3766, 3765, 3762, 3759, 3758, 3757, 3756, 3755, 3754, 3752, 3751, 3748, 3747, 3746, 3745,
    3744, 3743, 3742, 3741, 3740, 3739, 3737, 3736, 3735, 3734, 3730, 3729, 3726, 3725, 3724, 3719,
    3718, 3717, 3716, 3715, 3714, 3713, 3712, 3711, 3710, 3709, 3708, 3707, 3705, 3703, 3702, 3701,
    3699, 3698, 3695, 3693, 3691, 3689, 3688, 3687, 3686, 3685, 3684, 3681, 3680, 3679, 3678, 3677,
    3676, 3675, 3674, 3673, 3672, 3671, 3670, 3669, 3667, 3666, 3665, 3664, 3663, 3662, 3661, 3660,
    3659, 3658, 3657, 3655, 3654, 3653, 3652, 3648, 3647, 3644, 3643, 3642, 3641, 3640, 3639, 3638,
    3636, 3635, 3632, 3630, 3629, 3627, 3626, 3625, 3623, 3622, 3621, 3619, 3618, 3617, 3616, 3615,
    3613, 3612, 3611, 3610, 3609, 3607, 3606, 3604, 3603, 3602, 3601, 3600, 3599, 3598, 3596, 3595,
    3594, 3593, 3591, 3588, 3587, 3586, 3585, 3584, 3583, 3582, 3581, 3580, 3579, 3577, 3576, 3575,
    3574, 3573, 3572, 3571, 3570, 3568, 3567, 3566, 3565, 3564, 3563, 3561, 3558, 3556, 3555, 3554,
    3553, 3552, 3551, 3550, 3549, 3547, 3546, 3545, 3543, 3542, 3541, 3540, 3539, 3537, 3534, 3533,
    3532, 3531, 3530, 3529, 3528, 3527, 3526, 3525, 3523, 3522, 3521, 3520, 3519, 3518, 3517, 3515,
    3514, 3513, 3512, 3510, 3509, 3508, 3507, 3506, 3505, 3504, 3501, 3500, 3499, 3498, 3497, 3496,
    3495, 3494, 3493, 3492, 3491, 3490, 3489, 3488, 3487, 3486, 3484, 3483, 3482, 3480, 3478, 3477,
    3476, 3475, 3473, 3472, 3471, 3470, 3469, 3468, 3467, 3466, 3465, 3464, 3463, 3462, 3461, 3460,
    3459, 3458, 3457, 3455, 3454, 3453, 3452, 3451, 3450, 3449, 3448, 3446, 3443, 3442, 3441, 3440,
    3439, 3438, 3437, 3435, 3434, 3433, 3432, 3431, 3430, 3428, 3426, 3425, 3424, 3421, 3420, 3419,
    3417, 3416, 3415, 3414, 3412, 3410, 3409, 3408, 3407, 3406, 3403, 3402, 3400, 3399, 3398, 3397,
    3395, 3394, 3393, 3392, 3391, 3390, 3389, 3388, 3387, 3386, 3385, 3384, 3383, 3382, 3381, 3379,
    3378, 3377, 3375, 3374, 3373, 3372, 3369, 3368, 3366, 3364, 3363, 3362, 3361, 3360, 3358, 3357,
    3355, 3354, 3353, 3350, 3349, 3348, 3345, 3338, 3337, 3336, 3334, 3333, 3332, 3331, 3330, 3329,
    3328, 3327, 3326, 3325, 3322, 3321, 3319, 3314, 3313, 3312, 3311, 3310, 3309, 3308, 3307, 3306,
    3304, 3302, 3298, 3296, 3295, 3294, 3293, 3292, 3291, 3290, 3289, 3286, 3285, 3284, 3283, 3282,
    3281, 3280, 3278, 3277, 3272, 3270, 3267, 3266, 3265, 3264, 3263, 3262, 3260, 3259, 3257, 3256,
    3255, 3254, 3253, 3252, 3250, 3248, 3247, 3246, 3245, 3243, 3242, 3236, 3233, 3231, 3230, 3227,
    3226, 3225, 3224, 3222, 3221, 3220, 3219, 3218, 3216, 3214, 3213, 3212, 3210, 3209, 3208, 3207,
    3205, 3204, 3202, 3199, 3198, 3196, 3195, 3193, 3192, 3191, 3187, 3184, 3181, 3178, 3174, 3172,
    3168, 3167, 3166, 3165, 3164, 3163, 3160, 3159, 3158, 3157, 3156, 3155, 3153, 3151, 3147, 3145,
    3143, 3141, 3140, 3139, 3138, 3136, 3135, 3134, 3132, 3131, 3130, 3125, 3122, 3121, 3120, 3119,
    3118, 3116, 3115, 3114, 3110, 3109, 3108, 3106, 3103, 3102, 3101, 3098, 3097, 3096, 3095, 3094,
    3093, 3092, 3089, 3088, 3087, 3086, 3085, 3083, 3082, 3081, 3080, 3078, 3077, 3076, 3075, 3073,
    3072, 3071, 3070, 3066, 3065, 3064, 3063, 3062, 3061, 3057, 3055, 3053, 3052, 3051, 3050, 3049,
    3046, 3045, 3044, 3043, 3042, 3041, 3040, 3039, 3038, 3037, 3035, 3034, 3033, 3032, 3030, 3027,
    3026, 3025, 3020, 3019, 3018, 3017, 3016, 3014, 3013, 3012, 3011, 3009, 3008, 3007, 3006, 3003,
    3002, 3001, 3000, 2999, 2997, 2996, 2995, 2994, 2993, 2992, 2990, 2989, 2987, 2986, 2985, 2984,
    2981, 2980, 2979, 2978, 2977, 2976, 2974, 2973, 2971, 2970, 2969, 2966, 2961, 2960, 2957, 2954,
    2953, 2952, 2949, 2948, 2947, 2946, 2945, 2943, 2941, 2940, 2939, 2938, 2937, 2936, 2935, 2934,
    2933, 2929, 2927, 2924, 2923, 2922, 2921, 2920, 2919, 2918, 2917, 2916, 2914, 2913, 2912, 2911,
    2910, 2909, 2908, 2907, 2906, 2905, 2904, 2903, 2902, 2901, 2900, 2899, 2898, 2896, 2891, 2890,
    2889, 2887, 2886, 2885, 2884, 2883, 2880, 2879, 2878, 2877, 2876, 2875, 2873, 2871, 2870, 2869,
    2868, 2865, 2863, 2862, 2861, 2859, 2858, 2857, 2856, 2854, 2852, 2850, 2849, 2848, 2845, 2843,
    2841, 2840, 2839, 2838, 2837, 2836, 2835, 2834, 2833, 2832, 2831, 2830, 2829, 2828, 2827, 2826,
    2825, 2823, 2820, 2819, 2818, 2814, 2813, 2812, 2810, 2809, 2808, 2807, 2804, 2801, 2798, 2797,
    2796, 2795, 2794, 2792, 2791, 2790, 2789, 2788, 2786, 2784, 2783, 2782, 2781, 2780, 2779, 2778,
    2777, 2776, 2774, 2773, 2772, 2770, 2769, 2768, 2767, 2766, 2765, 2764, 2762, 2761, 2757, 2756,
    2755, 2750, 2749, 2748, 2747, 2741, 2738, 2737, 2736, 2735, 2734, 2732, 2730, 2729, 2728, 2727,
    2726, 2724, 2723, 2722, 2721, 2720, 2719, 2718, 2717, 2716, 2715, 2714, 2713, 2712, 2711, 2709,
    2707, 2706, 2705, 2704, 2703, 2702, 2701, 2700, 2697, 2695, 2694, 2693, 2692, 2691, 2690, 2689,
    2688, 2687, 2686, 2684, 2683, 2682, 2678, 2674, 2673, 2671, 2670, 2667, 2664, 2662, 2661, 2659,
    2658, 2656, 2655, 2654, 2653, 2651, 2650, 2648, 2647, 2646, 2644, 2643, 2641, 2640, 2639, 2637,
    2634, 2632, 2631, 2627, 2626, 2625, 2624, 2623, 2622, 2619, 2618, 2617, 2615, 2613, 2612, 2611,
    2610, 2609, 2608, 2607, 2606, 2605, 2604, 2602, 2601, 2600, 2599, 2596, 2595, 2594, 2593, 2592,
    2591, 2590, 2589, 2588, 2587, 2586, 2585, 2584, 2582, 2580, 2577, 2575, 2574, 2572, 2571, 2570,
    2564, 2561, 2560, 2559, 2557, 2556, 2555, 2554, 2551, 2550, 2549, 2548, 2547, 2545, 2543, 2542,
    2541, 2539, 2536, 2535, 2534, 2533, 2532, 2524, 2523, 2522, 2520, 2518, 2517, 2516, 2515, 2514,
    2513, 2510, 2509, 2508, 2507, 2506, 2505, 2504, 2503, 2502, 2500, 2499, 2496, 2495, 2492, 2491,
    2490, 2489, 2488, 2487, 2483, 2479, 2477, 2476, 2475, 2474, 2473, 2472, 2471, 2470, 2469, 2468,
    2467, 2466, 2465, 2464, 2463, 2462, 2461, 2460, 2459, 2458, 2457, 2456, 2454, 2452, 2449, 2448,
    2447, 2446, 2444, 2442, 2439, 2438, 2437, 2436, 2435, 2433, 2432, 2431, 2430, 2429, 2427, 2426,
    2425, 2424, 2423, 2422, 2421, 2420, 2419, 2418, 2414, 2413, 2412, 2411, 2410, 2409, 2408, 2407,
    2406, 2404, 2403, 2402, 2401, 2399, 2392, 2390, 2389, 2388, 2387, 2386, 2385, 2384, 2383, 2382,
    2379, 2377, 2376, 2375, 2374, 2373, 2370, 2368, 2366, 2363, 2361, 2360, 2359, 2358, 2356, 2355,
    2354, 2353, 2352, 2351, 2350, 2349, 2348, 2347, 2346, 2345, 2344, 2343, 2342, 2341, 2340, 2337,
    2336, 2335, 2334, 2333, 2332, 2330, 2329, 2328, 2327, 2326, 2324, 2323, 2319, 2318, 2316, 2314,
    2313, 2312, 2309, 2307, 2304, 2303, 2301, 2300, 2299, 2295, 2294, 2293, 2291, 2289, 2288, 2287,
    2286, 2285, 2284, 2283, 2281, 2280, 2279, 2278, 2277, 2276, 2274, 2273, 2270, 2269, 2263, 2261,
    2260, 2259, 2258, 2255, 2254, 2252, 2249, 2248, 2247, 2246, 2244, 2243, 2241, 2240, 2237, 2236,
    2235, 2234, 2233, 2232, 2231, 2229, 2227, 2221, 2219, 2218, 2217, 2215, 2211, 2210, 2207, 2206,
    2203, 2202, 2201, 2200, 2199, 2198, 2197, 2196, 2192, 2191, 2190, 2189, 2186, 2185, 2184, 2182,
    2181, 2180, 2177, 2176, 2175, 2174, 2172, 2171, 2168, 2167, 2166, 2163, 2161, 2159, 2158, 2156,
    2155, 2154, 2153, 2152, 2151, 2150, 2149, 2147, 2145, 2144, 2143, 2142, 2141, 2140, 2139, 2138,
    2136, 2134, 2132, 2130, 2127, 2126, 2124, 2123, 2121, 2119, 2118, 2116, 2115, 2114, 2113, 2112,
    2111, 2110, 2109, 2108, 2107, 2106, 2104, 2103, 2102, 2100, 2099, 2098, 2096, 2095, 2094, 2093,
    2091, 2089, 2088, 2087, 2086, 2085, 2084, 2083, 2081, 2080, 2079, 2078, 2077, 2076, 2075, 2073,
    2072, 2070, 2069, 2068, 2067, 2064, 2062, 2060, 2059, 2058, 2056, 2054, 2051, 2050, 2049, 2048,
    2047, 2046, 2045, 2044, 2043, 2042, 2041, 2038, 2037, 2036, 2033, 2032, 2031, 2030, 2029, 2028,
    2027, 2026, 2025, 2024, 2023, 2020, 2019, 2018, 2016, 2014, 2013, 2011, 2010, 2009, 2008, 2007,
    2006, 2005, 2004, 2003, 2002, 2000, 1999, 1998, 1995, 1994, 1993, 1992, 1991, 1987, 1986, 1985,
    1984, 1983, 1982, 1981, 1980, 1979, 1978, 1976, 1975, 1974, 1973, 1972, 1971, 1970, 1969, 1967,
    1966, 1965, 1964, 1963, 1962, 1961, 1960, 1959, 1958, 1957, 1955, 1953, 1952, 1950, 1949, 1947,
    1946, 1945, 1944, 1942, 1941, 1940, 1939, 1938, 1936, 1935, 1933, 1932, 1929, 1928, 1927, 1926,
    1925, 1922, 1921, 1920, 1919, 1916, 1915, 1913, 1912, 1911, 1910, 1909, 1908, 1907, 1906, 1905,
    1904, 1903, 1902, 1901, 1900, 1899, 1898, 1897, 1896, 1893, 1892, 1890, 1887, 1886, 1885, 1883,
    1882, 1881, 1880, 1878, 1877, 1875, 1874, 1873, 1872, 1871, 1869, 1868, 1867, 1866, 1865, 1864,
    1863, 1862, 1860, 1859, 1857, 1856, 1852, 1850, 1848, 1847, 1844, 1843, 1842, 1841, 1839, 1837,
    1835, 1834, 1833, 1831, 1830, 1829, 1828, 1827, 1823, 1822, 1820, 1818, 1817, 1816, 1815, 1812,
    1810, 1808, 1806, 1805, 1804, 1803, 1802, 1801, 1800, 1799, 1798, 1797, 1794, 1793, 1792, 1791,
    1790, 1788, 1787, 1786, 1785, 1784, 1783, 1781, 1780, 1779, 1778, 1777, 1776, 1775, 1774, 1773,
    1772, 1770, 1768, 1767, 1766, 1765, 1764, 1763, 1762, 1761, 1760, 1759, 1758, 1757, 1756, 1754,
    1751, 1750, 1749, 1748, 1747, 1746, 1745, 1743, 1742, 1741, 1740, 1739, 1738, 1737, 1736, 1735,
    1734, 1733, 1731, 1730, 1728, 1727, 1726, 1725, 1724, 1722, 1721, 1720, 1717, 1716, 1715, 1714,
    1713, 1710, 1709, 1708, 1707, 1706, 1705, 1704, 1703, 1702, 1700, 1698, 1697, 1696, 1695, 1693,
    1689, 1687, 1686, 1684, 1681, 1680, 1679, 1678, 1677, 1676, 1675, 1674, 1673, 1672, 1670, 1668,
    1667, 1666, 1665, 1663, 1661, 1660, 1659, 1658, 1656, 1655, 1654, 1653, 1652, 1651, 1649, 1648,
    1647, 1646, 1645, 1643, 1642, 1640, 1639, 1638, 1635, 1634, 1633, 1632, 1631, 1630, 1629, 1626,
    1625, 1624, 1623, 1622, 1621, 1618, 1613, 1612, 1609, 1608, 1607, 1605, 1600, 1599, 1598, 1596,
    1595, 1594, 1591, 1589, 1588, 1587, 1586, 1585, 1584, 1583, 1580, 1579, 1578, 1570, 1568, 1567,
    1566, 1565, 1564, 1561, 1560, 1559, 1557, 1555, 1554, 1553, 1552, 1549, 1548, 1547, 1546, 1545,
    1544, 1543, 1542, 1540, 1537, 1536, 1535, 1532, 1531, 1530, 1529, 1528, 1526, 1524, 1523, 1522,
    1521, 1520, 1517, 1516, 1515, 1513, 1512, 1511, 1510, 1508, 1507, 1506, 1505, 1503, 1501, 1500,
    1497, 1495, 1493, 1490, 1489, 1488, 1487, 1484, 1483, 1481, 1479, 1476, 1474, 1473, 1472, 1471,
    1470, 1469, 1467, 1466, 1465, 1464, 1463, 1462, 1461, 1458, 1457, 1456, 1455, 1454, 1453, 1452,
    1451, 1450, 1449, 1448, 1446, 1445, 1443, 1442, 1441, 1440, 1438, 1436, 1434, 1433, 1429, 1428,
    1427, 1425, 1424, 1422, 1421, 1420, 1419, 1417, 1415, 1414, 1413, 1411, 1410, 1409, 1408, 1407,
    1406, 1405, 1404, 1402, 1401, 1400, 1398, 1397, 1396, 1395, 1394, 1393, 1391, 1390, 1389, 1388,
    1386, 1385, 1384, 1383, 1381, 1379, 1378, 1377, 1375, 1374, 1370, 1369, 1366, 1364, 1361, 1360,
    1358, 1357, 1356, 1354, 1353, 1352, 1351, 1350, 1348, 1347, 1346, 1345, 1344, 1340, 1339, 1338,
    1337, 1336, 1335, 1334, 1331, 1330, 1327, 1321, 1320, 1319, 1318, 1317, 1316, 1315, 1312, 1311,
    1310, 1309, 1307, 1306, 1305, 1304, 1303, 1302, 1301, 1299, 1298, 1297, 1296, 1295, 1294, 1293,
    1292, 1291, 1289, 1287, 1285, 1284, 1283, 1282, 1281, 1280, 1279, 1278, 1277, 1275, 1274, 1273,
    1271, 1270, 1269, 1267, 1266, 1265, 1264, 1262, 1261, 1260, 1259, 1256, 1255, 1254, 1252, 1251,
    1250, 1249, 1248, 1247, 1246, 1245, 1244, 1242, 1241, 1240, 1238, 1236, 1234, 1233, 1232, 1231,
    1230, 1228, 1226, 1225, 1224, 1223, 1222, 1221, 1219, 1217, 1216, 1215, 1214, 1212, 1211, 1210,
    1209, 1208, 1207, 1206, 1203, 1202, 1201, 1200, 1199, 1198, 1197, 1196, 1195, 1194, 1193, 1190,
    1189, 1188, 1187, 1186, 1184, 1181, 1180, 1179, 1177, 1175, 1174, 1173, 1172, 1171, 1170, 1169,
    1168, 1166, 1165, 1164, 1163, 1162, 1161, 1159, 1156, 1153, 1152, 1149, 1148, 1147, 1145, 1139,
    1135, 1134, 1131, 1130, 1129, 1128, 1127, 1125, 1124, 1123, 1121, 1119, 1118, 1117, 1115, 1114,
    1113, 1111, 1110, 1109, 1108, 1107, 1106, 1104, 1103, 1102, 1101, 1100, 1099, 1098, 1095, 1094,
    1093, 1092, 1089, 1088, 1087, 1086, 1085, 1084, 1083, 1082, 1080, 1079, 1078, 1076, 1075, 1074,
    1073, 1072, 1071, 1070, 1069, 1068, 1067, 1065, 1064, 1063, 1062, 1061, 1060, 1059, 1058, 1057,
    1056, 1055, 1054, 1053, 1052, 1051, 1050, 1049, 1048, 1047, 1046, 1045, 1044, 1043, 1041, 1040,
    1038, 1037, 1036, 1035, 1034, 1033, 1031, 1030, 1029, 1027, 1026, 1025, 1022, 1021, 1020, 1018,
    1017, 1016, 1014, 1013, 1010, 1009, 1008, 1007, 1006, 1005, 1004, 1002, 1001, 1000, 999, 998,
    997, 996, 994, 993, 991, 989, 987, 986, 985, 984, 983, 982, 981, 980, 979, 978, 976, 975, 974,
    971, 969, 968, 967, 966, 965, 964, 963, 962, 959, 958, 957, 956, 955, 954, 953, 952, 951, 950,
    949, 948, 946, 945, 944, 943, 942, 941, 940, 939, 938, 937, 936, 935, 934, 933, 932, 931, 930,
    929, 928, 927, 926, 925, 924, 923, 922, 921, 920, 919, 918, 917, 916, 915, 914, 913, 912, 911,
    910, 909, 908, 907, 905, 903, 902, 901, 899, 896, 895, 894, 893, 892, 891, 890, 889, 887, 886,
    885, 884, 883, 882, 881, 880, 879, 877, 876, 874, 873, 872, 871, 870, 869, 868, 867, 866, 865,
    864, 862, 861, 860, 859, 857, 856, 855, 854, 853, 852, 851, 849, 846, 845, 844, 843, 842, 841,
    840, 839, 838, 837, 836, 835, 833, 832, 831, 830, 829, 828, 827, 826, 825, 824, 823, 822, 821,
    819, 818, 817, 816, 815, 814, 813, 812, 811, 810, 809, 808, 807, 806, 805, 804, 803, 802, 801,
    800, 799, 798, 797, 796, 795, 794, 793, 792, 791, 790, 789, 788, 787, 786, 785, 784, 783, 782,
    781, 780, 779, 778, 777, 776, 774, 773, 772, 770, 769, 768, 767, 766, 765, 764, 763, 762, 761,
    760, 759, 758, 757, 756, 754, 753, 752, 750, 749, 746, 744, 742, 741, 740, 739, 736, 735, 734,
    733, 732, 731, 728, 727, 726, 725, 724, 723, 722, 721, 720, 719, 717, 716, 715, 714, 713, 712,
    711, 710, 709, 708, 707, 706, 705, 704, 703, 702, 701, 700, 698, 697, 696, 695, 693, 692, 691,
    690, 689, 688, 687, 686, 685, 682, 681, 680, 678, 676, 674, 673, 672, 671, 670, 669, 668, 667,
    666, 665, 664, 663, 660, 659, 658, 657, 655, 654, 651, 649, 648, 646, 645, 644, 643, 642, 640,
    639, 638, 637, 636, 635, 634, 633, 632, 631, 630, 629, 628, 625, 623, 622, 621, 620, 619, 618,
    617, 616, 615, 614, 612, 610, 609, 608, 607, 606, 605, 603, 602, 600, 599, 598, 597, 594, 593,
    592, 591, 590, 589, 587, 584, 583, 581, 580, 579, 578, 577, 576, 575, 573, 572, 569, 568, 566,
    565, 564, 561, 560, 559, 558, 556, 555, 554, 553, 552, 551, 550, 549, 548, 547, 546, 545, 544,
    542, 541, 540, 539, 538, 535, 534, 533, 532, 531, 527, 525, 524, 522, 521, 520, 519, 515, 513,
    512, 511, 510, 509, 508, 507, 506, 505, 504, 503, 502, 501, 499, 498, 497, 496, 495, 494, 492,
    491, 490, 489, 488, 487, 485, 484, 483, 482, 481, 480, 479, 478, 477, 476, 475, 474, 473, 472,
    471, 470, 469, 468, 466, 465, 464, 463, 462, 461, 460, 458, 457, 456, 455, 454, 452, 451, 450,
    449, 448, 447, 446, 445, 444, 443, 442, 441, 440, 439, 438, 437, 436, 435, 434, 433, 431, 430,
    429, 428, 426, 424, 423, 422, 420, 419, 418, 417, 416, 415, 414, 413, 412, 411, 410, 409, 408,
    407, 406, 405, 404, 403, 402, 401, 400, 398, 397, 396, 395, 394, 393, 392, 391, 390, 389, 388,
    387, 385, 384, 383, 382, 381, 380, 379, 378, 377, 376, 375, 374, 373, 372, 371, 370, 369, 368,
    366, 365, 363, 362, 361, 360, 359, 358, 357, 355, 354, 353, 352, 351, 350, 349, 348, 347, 345,
    343, 342, 340, 338, 337, 336, 335, 333, 332, 330, 329, 327, 326, 325, 323, 321, 317, 316, 315,
    314, 313, 312, 310, 309, 308, 307, 305, 304, 303, 302, 301, 300, 297, 296, 295, 294, 293, 292,
    289, 288, 287, 285, 284, 283, 282, 281, 280, 278, 277, 276, 275, 274, 273, 272, 271, 270, 269,
    268, 267, 266, 265, 264, 263, 262, 261, 260, 259, 257, 255, 254, 252, 250, 249, 248, 247, 246,
    245, 244, 243, 242, 241, 240, 239, 237, 236, 235, 234, 233, 231, 230, 229, 228, 227, 226, 225,
    224, 223, 221, 220, 219, 218, 217, 216, 215, 214, 213, 212, 211, 210, 208, 206, 205, 203, 200,
    198, 197, 195, 193, 192, 190, 189, 188, 187, 186, 184, 183, 181, 180, 179, 178, 177, 176, 175,
    174, 172, 171, 170, 169, 168, 167, 166, 165, 164, 163, 162, 161, 158, 157, 156, 155, 154, 153,
    152, 151, 148, 147, 146, 145, 144, 143, 141, 140, 139, 138, 137, 136, 135, 134, 133, 132, 131,
    129, 128, 126, 125, 124, 123, 122, 121, 120, 119, 118, 117, 116, 115, 114, 113, 112, 111, 109,
    108, 107, 106, 105, 104, 103, 102, 101, 100, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88,
    87, 86, 85, 84, 83, 82, 81, 80, 79, 78, 77, 76, 75, 74, 72, 71, 70, 69, 68, 67, 66, 65, 64, 63,
    62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 47, 46, 45, 44, 42, 41, 40, 39, 38, 37,
    36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 19, 18, 17, 16, 15, 14, 13, 12, 10,
    9, 8, 7, 6, 5, 4, 3, 2, 1,
]);
static c2rust_lvalue_1: GlobalCell<[::core::ffi::c_int; 1574]> = GlobalCell::new([
    2190, 2189, 2188, 2187, 2185, 2183, 2182, 2180, 2179, 2178, 2177, 2175, 2173, 2168, 2166, 2163,
    2161, 2159, 2158, 2155, 2154, 2151, 2150, 2149, 2148, 2146, 2145, 2143, 2142, 2141, 2140, 2139,
    2138, 2137, 2136, 2135, 2134, 2133, 2131, 2129, 2128, 2126, 2125, 2124, 2122, 2121, 2120, 2119,
    2118, 2117, 2116, 2115, 2114, 2113, 2112, 2111, 2110, 2109, 2108, 2107, 2106, 2105, 2104, 2103,
    2102, 2101, 2100, 2098, 2097, 2095, 2094, 2092, 2091, 2090, 2088, 2087, 2083, 2081, 2080, 2079,
    2077, 2075, 2074, 2073, 2072, 2071, 2070, 2068, 2067, 2066, 2065, 2064, 2063, 2062, 2061, 2060,
    2059, 2058, 2057, 2056, 2053, 2052, 2051, 2050, 2049, 2048, 2047, 2046, 2045, 2044, 2043, 2042,
    2041, 2040, 2039, 2038, 2037, 2036, 2035, 2033, 2032, 2031, 2030, 2028, 2025, 2024, 2023, 2022,
    2021, 2018, 2017, 2016, 2014, 2011, 2010, 2009, 2008, 2007, 2006, 2004, 2003, 2002, 2000, 1998,
    1997, 1996, 1995, 1993, 1992, 1990, 1989, 1988, 1987, 1985, 1984, 1983, 1981, 1979, 1978, 1976,
    1975, 1974, 1973, 1972, 1971, 1970, 1969, 1968, 1967, 1966, 1965, 1964, 1963, 1962, 1961, 1960,
    1958, 1956, 1954, 1952, 1950, 1949, 1947, 1946, 1944, 1943, 1942, 1940, 1939, 1938, 1937, 1936,
    1934, 1932, 1931, 1930, 1929, 1927, 1925, 1923, 1922, 1921, 1919, 1918, 1917, 1915, 1913, 1912,
    1911, 1908, 1907, 1905, 1904, 1903, 1902, 1900, 1899, 1897, 1896, 1895, 1894, 1893, 1892, 1891,
    1889, 1888, 1887, 1884, 1883, 1882, 1881, 1880, 1879, 1878, 1877, 1875, 1874, 1873, 1872, 1871,
    1870, 1868, 1866, 1864, 1863, 1861, 1860, 1859, 1858, 1857, 1856, 1854, 1853, 1852, 1850, 1849,
    1848, 1847, 1846, 1845, 1844, 1843, 1841, 1840, 1839, 1838, 1837, 1835, 1834, 1833, 1832, 1831,
    1830, 1828, 1826, 1825, 1824, 1823, 1820, 1819, 1817, 1816, 1815, 1813, 1812, 1811, 1810, 1809,
    1808, 1807, 1805, 1803, 1802, 1800, 1799, 1798, 1797, 1795, 1794, 1793, 1792, 1791, 1790, 1787,
    1785, 1783, 1781, 1780, 1779, 1778, 1777, 1776, 1775, 1774, 1773, 1772, 1771, 1768, 1767, 1766,
    1765, 1764, 1763, 1762, 1761, 1759, 1758, 1757, 1755, 1754, 1753, 1752, 1751, 1750, 1748, 1747,
    1746, 1745, 1743, 1742, 1739, 1738, 1737, 1736, 1735, 1734, 1733, 1732, 1731, 1730, 1729, 1728,
    1726, 1725, 1724, 1723, 1722, 1721, 1720, 1718, 1717, 1716, 1715, 1714, 1713, 1712, 1711, 1710,
    1709, 1708, 1707, 1706, 1705, 1704, 1703, 1702, 1701, 1700, 1698, 1697, 1696, 1694, 1693, 1692,
    1691, 1690, 1689, 1688, 1687, 1686, 1685, 1684, 1683, 1682, 1681, 1680, 1679, 1678, 1677, 1676,
    1675, 1673, 1672, 1671, 1670, 1669, 1668, 1667, 1666, 1665, 1664, 1663, 1662, 1661, 1659, 1658,
    1657, 1656, 1655, 1654, 1653, 1652, 1651, 1649, 1648, 1647, 1646, 1645, 1644, 1643, 1642, 1640,
    1639, 1638, 1637, 1636, 1635, 1634, 1633, 1632, 1631, 1630, 1629, 1628, 1627, 1626, 1622, 1621,
    1620, 1618, 1617, 1616, 1615, 1614, 1613, 1612, 1610, 1609, 1608, 1607, 1606, 1603, 1602, 1601,
    1600, 1599, 1598, 1597, 1596, 1595, 1591, 1588, 1587, 1586, 1585, 1584, 1583, 1582, 1580, 1578,
    1577, 1575, 1573, 1568, 1567, 1565, 1564, 1563, 1562, 1561, 1556, 1555, 1554, 1553, 1552, 1551,
    1549, 1548, 1547, 1546, 1545, 1543, 1542, 1541, 1540, 1539, 1538, 1536, 1535, 1534, 1533, 1532,
    1530, 1527, 1526, 1525, 1524, 1523, 1522, 1521, 1520, 1518, 1517, 1516, 1515, 1514, 1513, 1512,
    1511, 1509, 1508, 1507, 1506, 1505, 1503, 1502, 1501, 1500, 1499, 1498, 1497, 1495, 1494, 1493,
    1492, 1491, 1490, 1489, 1488, 1487, 1486, 1485, 1484, 1483, 1480, 1479, 1478, 1477, 1476, 1475,
    1474, 1473, 1472, 1470, 1469, 1468, 1467, 1466, 1465, 1464, 1462, 1461, 1460, 1458, 1456, 1455,
    1452, 1451, 1449, 1448, 1447, 1446, 1445, 1444, 1443, 1442, 1441, 1439, 1438, 1437, 1436, 1435,
    1434, 1433, 1431, 1430, 1429, 1428, 1427, 1426, 1425, 1423, 1422, 1421, 1420, 1419, 1417, 1416,
    1415, 1412, 1409, 1408, 1407, 1406, 1405, 1402, 1401, 1400, 1397, 1395, 1394, 1393, 1392, 1391,
    1389, 1388, 1386, 1385, 1384, 1383, 1382, 1380, 1378, 1376, 1374, 1373, 1372, 1371, 1370, 1369,
    1368, 1367, 1366, 1365, 1364, 1363, 1362, 1361, 1360, 1359, 1358, 1357, 1356, 1355, 1354, 1353,
    1352, 1351, 1350, 1349, 1348, 1347, 1346, 1345, 1342, 1341, 1340, 1339, 1337, 1336, 1335, 1332,
    1331, 1330, 1329, 1324, 1323, 1322, 1319, 1318, 1316, 1315, 1314, 1312, 1311, 1310, 1309, 1308,
    1307, 1306, 1303, 1300, 1299, 1298, 1297, 1296, 1295, 1294, 1293, 1292, 1291, 1290, 1289, 1288,
    1282, 1281, 1280, 1279, 1278, 1277, 1274, 1273, 1272, 1271, 1270, 1269, 1268, 1265, 1263, 1261,
    1260, 1259, 1258, 1257, 1256, 1255, 1253, 1250, 1247, 1246, 1245, 1244, 1243, 1242, 1240, 1238,
    1236, 1233, 1231, 1230, 1229, 1228, 1227, 1225, 1223, 1222, 1221, 1219, 1217, 1216, 1215, 1214,
    1213, 1212, 1211, 1208, 1207, 1206, 1205, 1204, 1202, 1201, 1200, 1197, 1195, 1194, 1192, 1191,
    1190, 1189, 1187, 1183, 1182, 1181, 1180, 1177, 1176, 1175, 1174, 1173, 1172, 1171, 1170, 1167,
    1165, 1164, 1163, 1162, 1158, 1156, 1155, 1154, 1153, 1147, 1145, 1144, 1143, 1142, 1141, 1139,
    1138, 1137, 1136, 1135, 1132, 1131, 1129, 1127, 1125, 1122, 1121, 1120, 1119, 1116, 1115, 1114,
    1113, 1110, 1109, 1107, 1106, 1105, 1104, 1103, 1102, 1101, 1100, 1099, 1098, 1097, 1096, 1094,
    1093, 1092, 1091, 1090, 1089, 1088, 1085, 1084, 1083, 1082, 1079, 1078, 1076, 1075, 1072, 1071,
    1067, 1066, 1064, 1062, 1061, 1059, 1057, 1056, 1055, 1052, 1051, 1050, 1049, 1047, 1046, 1043,
    1042, 1038, 1037, 1036, 1034, 1033, 1032, 1030, 1029, 1028, 1027, 1026, 1025, 1024, 1021, 1020,
    1017, 1016, 1015, 1014, 1012, 1011, 1010, 1009, 1008, 1007, 1005, 1003, 1000, 998, 997, 994,
    993, 992, 991, 990, 989, 987, 986, 984, 981, 972, 970, 969, 967, 966, 965, 964, 963, 958, 953,
    952, 951, 950, 947, 946, 944, 942, 941, 937, 936, 935, 932, 931, 929, 928, 927, 926, 925, 922,
    921, 920, 919, 916, 915, 914, 913, 911, 910, 909, 908, 906, 904, 902, 901, 900, 898, 897, 895,
    893, 892, 888, 886, 884, 883, 882, 879, 878, 876, 875, 874, 872, 870, 869, 867, 866, 865, 864,
    863, 862, 861, 860, 859, 858, 857, 855, 854, 853, 852, 851, 849, 847, 846, 845, 844, 843, 841,
    840, 839, 838, 837, 836, 835, 834, 832, 831, 830, 829, 826, 825, 824, 823, 822, 821, 820, 819,
    818, 817, 816, 815, 814, 811, 810, 808, 807, 806, 805, 804, 803, 798, 795, 792, 790, 789, 788,
    787, 786, 785, 783, 782, 780, 779, 778, 776, 774, 773, 772, 771, 770, 766, 765, 764, 762, 761,
    760, 759, 758, 757, 756, 755, 754, 752, 751, 750, 749, 747, 746, 745, 743, 742, 741, 739, 738,
    736, 735, 734, 733, 731, 729, 728, 727, 726, 724, 717, 716, 715, 713, 712, 711, 707, 706, 704,
    702, 701, 700, 699, 697, 696, 695, 692, 691, 690, 689, 688, 686, 685, 684, 681, 680, 679, 678,
    677, 676, 675, 673, 672, 671, 670, 668, 667, 666, 665, 664, 662, 661, 658, 657, 656, 654, 652,
    650, 649, 647, 646, 645, 644, 643, 642, 641, 640, 635, 634, 633, 632, 628, 624, 622, 621, 620,
    616, 614, 612, 610, 609, 608, 607, 605, 604, 603, 602, 601, 600, 599, 598, 597, 596, 595, 594,
    593, 590, 589, 588, 586, 585, 584, 583, 582, 581, 580, 579, 577, 576, 573, 572, 570, 568, 567,
    566, 565, 563, 562, 561, 560, 559, 557, 555, 551, 550, 549, 548, 546, 545, 544, 543, 542, 541,
    540, 539, 537, 536, 533, 532, 530, 529, 528, 527, 526, 525, 524, 523, 522, 521, 519, 517, 514,
    512, 511, 510, 509, 508, 507, 505, 504, 503, 501, 499, 497, 495, 494, 493, 492, 490, 486, 484,
    483, 479, 478, 477, 476, 475, 471, 470, 469, 467, 466, 465, 464, 462, 461, 458, 457, 456, 455,
    454, 452, 449, 448, 446, 445, 443, 441, 440, 434, 433, 430, 428, 427, 426, 425, 424, 423, 422,
    421, 420, 419, 417, 415, 414, 413, 412, 411, 410, 409, 408, 407, 406, 404, 403, 402, 401, 398,
    397, 395, 394, 393, 391, 390, 389, 388, 386, 385, 382, 380, 379, 378, 372, 370, 369, 368, 367,
    365, 362, 361, 360, 359, 358, 355, 353, 349, 346, 343, 342, 341, 339, 336, 335, 334, 333, 332,
    331, 330, 322, 320, 319, 318, 316, 314, 313, 305, 304, 303, 302, 300, 299, 290, 286, 285, 284,
    283, 281, 279, 278, 277, 276, 275, 274, 272, 271, 270, 269, 268, 267, 266, 265, 264, 263, 262,
    261, 260, 259, 258, 257, 256, 255, 254, 252, 251, 250, 249, 248, 246, 245, 244, 243, 242, 241,
    240, 239, 238, 237, 236, 235, 234, 233, 232, 231, 230, 228, 227, 226, 224, 223, 222, 221, 220,
    219, 218, 217, 216, 213, 212, 211, 210, 209, 207, 206, 205, 204, 203, 200, 198, 197, 196, 195,
    194, 193, 192, 191, 190, 187, 183, 182, 181, 180, 179, 178, 177, 176, 175, 174, 173, 172, 169,
    168, 167, 166, 161, 159, 154, 152, 149, 147, 145, 143, 141, 138, 137, 136, 135, 133, 132, 131,
    130, 129, 127, 126, 124, 123, 120, 118, 117, 116, 115, 114, 113, 112, 111, 110, 109, 108, 106,
    105, 104, 103, 102, 101, 99, 98, 97, 96, 95, 94, 93, 92, 90, 89, 88, 87, 86, 85, 84, 83, 82,
    81, 79, 77, 76, 74, 73, 71, 70, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 57, 55, 54, 53, 52, 51,
    50, 49, 48, 47, 46, 45, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 31, 30, 29, 26, 25, 24, 23,
    22, 21, 20, 19, 18, 17, 16, 15, 12, 11, 10, 9, 8, 6, 5, 4, 3, 2, 1,
]);
static c2rust_lvalue_2: GlobalCell<[::core::ffi::c_int; 1612]> = GlobalCell::new([
    2147, 2146, 2138, 2136, 2135, 2133, 2132, 2130, 2128, 2127, 2125, 2123, 2122, 2119, 2118, 2117,
    2116, 2112, 2111, 2110, 2109, 2108, 2107, 2105, 2103, 2102, 2101, 2100, 2099, 2098, 2097, 2096,
    2095, 2094, 2093, 2091, 2090, 2088, 2087, 2086, 2085, 2079, 2078, 2075, 2074, 2073, 2071, 2070,
    2069, 2068, 2067, 2066, 2065, 2064, 2062, 2061, 2060, 2059, 2058, 2056, 2055, 2053, 2052, 2051,
    2050, 2048, 2047, 2043, 2042, 2041, 2040, 2039, 2038, 2037, 2036, 2035, 2034, 2032, 2031, 2030,
    2028, 2027, 2026, 2025, 2024, 2023, 2021, 2020, 2019, 2018, 2017, 2016, 2011, 2010, 2009, 2008,
    2007, 2002, 2001, 1998, 1997, 1991, 1990, 1989, 1988, 1987, 1984, 1983, 1981, 1980, 1979, 1978,
    1977, 1974, 1971, 1970, 1969, 1967, 1965, 1964, 1963, 1962, 1961, 1960, 1958, 1957, 1954, 1953,
    1952, 1951, 1950, 1949, 1948, 1947, 1945, 1944, 1943, 1942, 1941, 1939, 1938, 1937, 1936, 1935,
    1934, 1933, 1931, 1930, 1929, 1928, 1925, 1924, 1923, 1922, 1920, 1919, 1918, 1917, 1916, 1915,
    1914, 1913, 1912, 1910, 1909, 1908, 1906, 1905, 1903, 1901, 1893, 1892, 1891, 1890, 1889, 1888,
    1886, 1885, 1884, 1883, 1881, 1880, 1879, 1878, 1877, 1876, 1875, 1872, 1871, 1870, 1869, 1868,
    1867, 1861, 1859, 1858, 1855, 1854, 1853, 1852, 1851, 1850, 1849, 1846, 1845, 1844, 1843, 1842,
    1841, 1840, 1839, 1838, 1836, 1834, 1833, 1832, 1831, 1829, 1827, 1825, 1824, 1823, 1822, 1821,
    1820, 1819, 1818, 1817, 1816, 1815, 1814, 1812, 1811, 1810, 1809, 1808, 1807, 1806, 1805, 1804,
    1802, 1801, 1800, 1799, 1798, 1797, 1794, 1793, 1790, 1789, 1787, 1786, 1783, 1781, 1780, 1779,
    1778, 1777, 1776, 1775, 1774, 1773, 1772, 1771, 1769, 1768, 1767, 1766, 1765, 1764, 1763, 1762,
    1761, 1760, 1759, 1758, 1755, 1754, 1753, 1752, 1751, 1750, 1749, 1747, 1746, 1744, 1742, 1740,
    1739, 1738, 1737, 1735, 1734, 1732, 1731, 1730, 1729, 1724, 1723, 1722, 1718, 1717, 1716, 1714,
    1713, 1709, 1707, 1705, 1702, 1701, 1700, 1698, 1697, 1695, 1694, 1693, 1691, 1690, 1689, 1688,
    1687, 1686, 1685, 1683, 1681, 1680, 1679, 1678, 1677, 1676, 1673, 1672, 1671, 1670, 1667, 1665,
    1664, 1663, 1662, 1661, 1660, 1659, 1658, 1657, 1654, 1650, 1649, 1648, 1647, 1646, 1645, 1644,
    1642, 1640, 1638, 1633, 1632, 1631, 1630, 1629, 1628, 1627, 1626, 1625, 1624, 1623, 1622, 1621,
    1620, 1619, 1618, 1616, 1615, 1614, 1613, 1612, 1611, 1610, 1609, 1608, 1607, 1606, 1605, 1604,
    1603, 1602, 1601, 1600, 1599, 1598, 1597, 1596, 1594, 1593, 1592, 1590, 1585, 1583, 1582, 1581,
    1580, 1578, 1576, 1573, 1572, 1571, 1570, 1569, 1567, 1565, 1563, 1561, 1560, 1559, 1557, 1556,
    1555, 1554, 1553, 1552, 1551, 1550, 1549, 1548, 1547, 1546, 1544, 1541, 1539, 1538, 1536, 1535,
    1532, 1528, 1527, 1526, 1521, 1520, 1519, 1518, 1517, 1516, 1515, 1512, 1511, 1510, 1509, 1508,
    1507, 1506, 1505, 1504, 1503, 1502, 1501, 1498, 1497, 1493, 1491, 1490, 1488, 1484, 1483, 1482,
    1481, 1479, 1478, 1477, 1476, 1475, 1474, 1473, 1472, 1471, 1470, 1468, 1467, 1466, 1464, 1463,
    1459, 1456, 1455, 1454, 1453, 1452, 1450, 1449, 1447, 1446, 1445, 1444, 1443, 1442, 1441, 1440,
    1439, 1437, 1435, 1434, 1433, 1432, 1431, 1428, 1426, 1424, 1422, 1421, 1420, 1419, 1417, 1416,
    1413, 1410, 1409, 1408, 1407, 1405, 1404, 1402, 1401, 1399, 1398, 1396, 1395, 1393, 1389, 1388,
    1387, 1386, 1385, 1384, 1383, 1382, 1381, 1380, 1379, 1378, 1377, 1376, 1375, 1374, 1373, 1370,
    1369, 1367, 1365, 1362, 1361, 1360, 1359, 1358, 1357, 1356, 1354, 1353, 1351, 1350, 1349, 1348,
    1347, 1346, 1345, 1344, 1343, 1342, 1341, 1340, 1339, 1338, 1337, 1336, 1332, 1331, 1329, 1328,
    1327, 1326, 1325, 1323, 1322, 1321, 1320, 1319, 1318, 1317, 1316, 1315, 1314, 1313, 1312, 1311,
    1310, 1309, 1308, 1307, 1306, 1305, 1304, 1303, 1302, 1301, 1300, 1299, 1298, 1297, 1296, 1293,
    1292, 1291, 1290, 1289, 1288, 1287, 1286, 1284, 1283, 1279, 1278, 1277, 1276, 1275, 1273, 1272,
    1271, 1270, 1269, 1268, 1266, 1265, 1264, 1263, 1262, 1261, 1260, 1259, 1258, 1257, 1256, 1255,
    1254, 1253, 1252, 1251, 1250, 1249, 1248, 1247, 1246, 1245, 1243, 1240, 1238, 1236, 1235, 1231,
    1229, 1228, 1226, 1224, 1222, 1221, 1220, 1219, 1218, 1217, 1216, 1215, 1214, 1213, 1212, 1211,
    1210, 1206, 1205, 1204, 1203, 1202, 1201, 1200, 1199, 1198, 1197, 1196, 1195, 1194, 1193, 1191,
    1190, 1189, 1188, 1187, 1186, 1185, 1184, 1182, 1181, 1178, 1177, 1176, 1175, 1174, 1173, 1172,
    1171, 1170, 1169, 1168, 1167, 1166, 1165, 1164, 1161, 1160, 1158, 1157, 1156, 1155, 1153, 1152,
    1151, 1149, 1148, 1147, 1145, 1143, 1141, 1140, 1139, 1137, 1136, 1135, 1134, 1133, 1132, 1131,
    1130, 1129, 1128, 1127, 1126, 1125, 1124, 1122, 1121, 1117, 1116, 1112, 1111, 1109, 1108, 1107,
    1104, 1103, 1102, 1101, 1098, 1096, 1095, 1093, 1092, 1091, 1090, 1087, 1086, 1085, 1083, 1081,
    1080, 1079, 1077, 1076, 1075, 1074, 1073, 1072, 1071, 1070, 1069, 1068, 1067, 1066, 1065, 1063,
    1061, 1059, 1057, 1056, 1055, 1053, 1052, 1049, 1048, 1047, 1046, 1045, 1043, 1042, 1041, 1038,
    1036, 1034, 1033, 1032, 1030, 1028, 1027, 1026, 1023, 1022, 1021, 1020, 1019, 1018, 1017, 1015,
    1013, 1012, 1011, 1010, 1009, 1008, 1007, 1006, 1005, 1003, 1002, 1001, 1000, 999, 998, 997,
    996, 995, 993, 992, 991, 990, 989, 987, 986, 984, 983, 982, 981, 978, 977, 976, 974, 972, 971,
    970, 968, 967, 965, 964, 963, 962, 961, 960, 959, 958, 957, 956, 955, 954, 953, 951, 950, 948,
    947, 946, 945, 943, 942, 941, 940, 939, 938, 936, 935, 934, 933, 930, 929, 928, 927, 926, 925,
    924, 923, 922, 921, 920, 919, 918, 917, 915, 914, 913, 912, 911, 910, 908, 907, 906, 905, 904,
    903, 902, 901, 900, 899, 898, 897, 896, 895, 894, 893, 892, 891, 890, 889, 888, 886, 884, 883,
    881, 880, 877, 876, 875, 874, 873, 870, 869, 867, 866, 865, 864, 862, 861, 860, 858, 857, 855,
    853, 850, 849, 847, 846, 845, 842, 841, 840, 839, 838, 837, 835, 833, 832, 831, 830, 829, 827,
    826, 825, 824, 823, 822, 821, 820, 819, 816, 814, 812, 811, 810, 809, 806, 805, 804, 803, 802,
    799, 798, 797, 796, 795, 794, 793, 792, 791, 790, 789, 787, 786, 785, 783, 782, 781, 779, 778,
    777, 776, 775, 774, 773, 772, 771, 770, 769, 767, 766, 765, 764, 763, 762, 761, 760, 759, 758,
    756, 755, 754, 753, 752, 749, 748, 746, 745, 744, 743, 742, 741, 740, 739, 738, 735, 734, 733,
    732, 731, 730, 729, 728, 727, 726, 725, 724, 723, 722, 721, 720, 719, 717, 716, 713, 711, 710,
    708, 707, 706, 705, 704, 703, 702, 701, 700, 699, 698, 697, 696, 695, 694, 693, 692, 691, 690,
    689, 688, 687, 686, 684, 683, 679, 678, 677, 676, 675, 674, 673, 672, 671, 669, 668, 667, 666,
    663, 662, 661, 659, 658, 657, 656, 655, 654, 653, 652, 651, 650, 649, 648, 647, 645, 644, 643,
    642, 641, 640, 639, 638, 637, 636, 635, 634, 633, 632, 631, 630, 629, 628, 627, 626, 623, 622,
    621, 619, 618, 617, 616, 615, 614, 612, 610, 609, 606, 605, 604, 603, 602, 601, 599, 598, 596,
    594, 593, 592, 591, 588, 586, 585, 583, 582, 580, 579, 577, 575, 574, 573, 572, 569, 568, 567,
    566, 565, 562, 561, 558, 557, 556, 555, 554, 553, 552, 551, 550, 549, 547, 546, 544, 543, 541,
    540, 539, 538, 537, 536, 535, 534, 533, 532, 531, 530, 529, 528, 526, 525, 524, 521, 520, 519,
    518, 517, 516, 515, 512, 511, 507, 506, 505, 504, 503, 499, 498, 496, 495, 494, 493, 492, 491,
    490, 489, 488, 487, 486, 485, 484, 483, 480, 479, 478, 477, 476, 475, 474, 473, 472, 470, 469,
    468, 467, 466, 465, 464, 463, 460, 458, 456, 454, 453, 452, 451, 449, 447, 446, 444, 443, 442,
    441, 440, 439, 438, 437, 436, 435, 434, 433, 432, 430, 428, 427, 426, 425, 424, 423, 421, 419,
    418, 416, 415, 414, 413, 412, 411, 410, 409, 408, 407, 406, 405, 404, 402, 401, 400, 399, 397,
    396, 395, 394, 392, 390, 389, 388, 386, 384, 383, 382, 381, 380, 378, 375, 374, 372, 371, 367,
    366, 365, 364, 363, 362, 361, 359, 357, 355, 354, 353, 351, 350, 348, 346, 344, 343, 341, 340,
    335, 334, 332, 331, 329, 328, 326, 325, 324, 323, 322, 321, 320, 319, 318, 317, 316, 315, 313,
    312, 311, 310, 308, 307, 306, 305, 304, 303, 302, 301, 300, 299, 298, 297, 296, 295, 294, 293,
    292, 291, 290, 289, 287, 286, 285, 283, 281, 280, 279, 278, 277, 276, 275, 274, 273, 272, 269,
    268, 267, 266, 265, 262, 260, 259, 258, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244,
    243, 242, 241, 240, 239, 238, 237, 236, 235, 234, 232, 231, 230, 228, 227, 226, 225, 224, 220,
    218, 217, 216, 215, 214, 213, 212, 211, 210, 209, 208, 207, 205, 204, 203, 199, 198, 197, 196,
    195, 194, 192, 191, 190, 189, 188, 187, 186, 184, 183, 182, 181, 180, 178, 177, 176, 175, 174,
    172, 171, 170, 169, 168, 167, 166, 164, 163, 162, 161, 159, 158, 157, 156, 155, 154, 153, 152,
    151, 150, 149, 147, 146, 145, 144, 143, 142, 141, 140, 138, 137, 135, 134, 132, 131, 130, 129,
    128, 127, 126, 124, 121, 120, 119, 118, 117, 116, 115, 114, 113, 112, 111, 110, 109, 108, 106,
    105, 104, 103, 102, 101, 100, 98, 97, 94, 93, 92, 89, 88, 87, 86, 85, 84, 83, 82, 81, 79, 78,
    77, 76, 75, 74, 73, 72, 68, 67, 66, 65, 63, 61, 60, 59, 58, 56, 54, 52, 49, 48, 47, 46, 43, 42,
    41, 40, 39, 38, 37, 36, 35, 33, 32, 31, 25, 23, 22, 21, 20, 18, 16, 15, 14, 13, 12, 11, 10, 9,
    8, 7, 5, 4, 3, 2, 1,
]);
static c2rust_lvalue_3: GlobalCell<[::core::ffi::c_int; 121]> = GlobalCell::new([
    239, 237, 235, 233, 231, 230, 226, 225, 224, 223, 222, 220, 219, 217, 210, 209, 204, 202, 201,
    191, 188, 187, 183, 182, 180, 177, 176, 174, 173, 170, 166, 165, 162, 159, 157, 156, 155, 154,
    152, 148, 147, 146, 143, 142, 141, 140, 137, 136, 133, 132, 131, 130, 128, 125, 124, 123, 122,
    121, 120, 114, 109, 108, 106, 105, 102, 97, 91, 90, 88, 87, 86, 84, 82, 81, 78, 77, 75, 74, 71,
    70, 67, 66, 65, 64, 63, 62, 61, 59, 58, 57, 56, 55, 54, 52, 47, 46, 44, 41, 40, 38, 36, 33, 32,
    31, 30, 28, 27, 25, 24, 23, 22, 20, 18, 13, 12, 10, 9, 8, 7, 4, 1,
]);
static included_patchsets: GlobalCell<[*const ::core::ffi::c_int; 5]> = GlobalCell::new(unsafe {
    [
        (c2rust_lvalue.as_raw() as *const _) as *const ::core::ffi::c_int,
        (c2rust_lvalue_0.as_raw() as *const _) as *const ::core::ffi::c_int,
        (c2rust_lvalue_1.as_raw() as *const _) as *const ::core::ffi::c_int,
        (c2rust_lvalue_2.as_raw() as *const _) as *const ::core::ffi::c_int,
        (c2rust_lvalue_3.as_raw() as *const _) as *const ::core::ffi::c_int,
    ]
});
#[no_mangle]
pub unsafe extern "C" fn has_nvim_version(version_str: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = version_str;
    let mut minor: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut patch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !ascii_isdigit(*p as ::core::ffi::c_int) {
        return false_0 != 0;
    }
    let mut major: ::core::ffi::c_int = atoi(p);
    p = strchr(p, '.' as ::core::ffi::c_int);
    if !p.is_null() {
        p = p.offset(1);
        if !ascii_isdigit(*p as ::core::ffi::c_int) {
            return false_0 != 0;
        }
        minor = atoi(p);
        p = strchr(p, '.' as ::core::ffi::c_int);
        if !p.is_null() {
            p = p.offset(1);
            if !ascii_isdigit(*p as ::core::ffi::c_int) {
                return false_0 != 0;
            }
            patch = atoi(p);
        }
    }
    return major < NVIM_VERSION_MAJOR
        || major == NVIM_VERSION_MAJOR
            && (minor < NVIM_VERSION_MINOR
                || minor == NVIM_VERSION_MINOR && patch <= NVIM_VERSION_PATCH);
}
#[no_mangle]
pub unsafe extern "C" fn min_vim_version() -> ::core::ffi::c_int {
    return (*vim_versions.ptr())[0 as ::core::ffi::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn highest_patch() -> ::core::ffi::c_int {
    return *(*included_patchsets.ptr())[0 as ::core::ffi::c_int as usize]
        .offset(0 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn has_vim_patch(
    mut n: ::core::ffi::c_int,
    mut major_minor_version: ::core::ffi::c_int,
) -> bool {
    let mut v_i: ::core::ffi::c_int = 0;
    if major_minor_version > 0 as ::core::ffi::c_int {
        if major_minor_version < (*vim_versions.ptr())[0 as ::core::ffi::c_int as usize] {
            return true_0 != 0;
        }
        let size: size_t = ::core::mem::size_of::<[::core::ffi::c_int; 5]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_int; 5]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                    == 0) as ::core::ffi::c_int as size_t,
            );
        v_i = -1 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < size {
            if (*vim_versions.ptr())[i as usize] == major_minor_version {
                v_i = i as ::core::ffi::c_int;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if v_i == -1 as ::core::ffi::c_int {
            return false_0 != 0;
        }
    } else {
        v_i = 0 as ::core::ffi::c_int;
    }
    let mut l: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut h: ::core::ffi::c_int = (*num_patches.ptr())[v_i as usize] - 1 as ::core::ffi::c_int;
    loop {
        let m: ::core::ffi::c_int = (l + h) / 2 as ::core::ffi::c_int;
        if *(*included_patchsets.ptr())[v_i as usize].offset(m as isize) == n {
            return true_0 != 0;
        }
        if l == h {
            break;
        }
        if *(*included_patchsets.ptr())[v_i as usize].offset(m as isize) < n {
            h = m;
        } else {
            l = m + 1 as ::core::ffi::c_int;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ex_version(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        list_version();
    }
}
unsafe extern "C" fn version_msg_wrap(mut s: *mut ::core::ffi::c_char, mut wrap: bool) {
    let mut len: ::core::ffi::c_int = vim_strsize(s)
        + (if wrap as ::core::ffi::c_int != 0 {
            2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    if !got_int
        && len < Columns
        && msg_col + len >= Columns
        && *s as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    if !got_int {
        if wrap {
            msg_puts(b"[\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(s);
        if wrap {
            msg_puts(b"]\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
}
unsafe extern "C" fn version_msg(mut s: *mut ::core::ffi::c_char) {
    version_msg_wrap(s, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn list_in_columns(
    mut items: *mut *mut ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
) {
    let mut item_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while if size < 0 as ::core::ffi::c_int {
        !(*items.offset(i as isize)).is_null() as ::core::ffi::c_int
    } else {
        (i < size) as ::core::ffi::c_int
    } != 0
    {
        let mut l: ::core::ffi::c_int = vim_strsize(*items.offset(i as isize))
            + (if i == current {
                2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        if l > width {
            width = l;
        }
        item_count += 1;
        i += 1;
    }
    width += 1 as ::core::ffi::c_int;
    if Columns < width {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < item_count {
            version_msg_wrap(*items.offset(i_0 as isize), i_0 == current);
            if msg_col > 0 as ::core::ffi::c_int && i_0 < item_count - 1 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            i_0 += 1;
        }
        return;
    }
    let mut ncol: ::core::ffi::c_int = (Columns + 1 as ::core::ffi::c_int) / width;
    let mut nrow: ::core::ffi::c_int = item_count / ncol
        + (if item_count % ncol != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    let mut cur_row: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !got_int && i_1 < nrow * ncol {
        let mut idx: ::core::ffi::c_int = i_1 / ncol + i_1 % ncol * nrow;
        if idx < item_count {
            let mut last_col: bool =
                (i_1 + 1 as ::core::ffi::c_int) % ncol == 0 as ::core::ffi::c_int;
            if idx == current {
                msg_putchar('[' as ::core::ffi::c_int);
            }
            msg_puts(*items.offset(idx as isize));
            if idx == current {
                msg_putchar(']' as ::core::ffi::c_int);
            }
            if last_col {
                if msg_col > 0 as ::core::ffi::c_int && cur_row < nrow {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                cur_row += 1;
            } else {
                while msg_col % width != 0 {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
            }
        } else if msg_col > 0 as ::core::ffi::c_int {
            if cur_row < nrow {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            cur_row += 1;
        }
        i_1 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn list_lua_version() {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Object = nlua_exec(
        String_0 {
            data: b"return ((jit and jit.version) and jit.version or _VERSION)\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 59]>().wrapping_sub(1 as size_t),
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
    '_c2rust_label: {
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        } else {
            __assert_fail(
                b"!ERROR_SET(&err)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/version.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4159 as ::core::ffi::c_uint,
                b"void list_lua_version(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"ret.type == kObjectTypeString\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/version.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4160 as ::core::ffi::c_uint,
                b"void list_lua_version(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    msg_puts(ret.data.string.data);
    api_free_object(ret);
}
#[no_mangle]
pub unsafe extern "C" fn list_version() {
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_puts(longVersion.get());
    msg_putchar('\n' as ::core::ffi::c_int);
    msg_puts(version_buildtype.get());
    msg_putchar('\n' as ::core::ffi::c_int);
    list_lua_version();
    if p_verbose > 0 as OptInt {
        msg_putchar('\n' as ::core::ffi::c_int);
        msg_puts(b"Vim versions: \0".as_ptr() as *const ::core::ffi::c_char);
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[::core::ffi::c_int; 5]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_int; 5]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if i != 0 {
                msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
            }
            msg_puts((*Versions.ptr())[i as usize]);
            i = i.wrapping_add(1);
        }
        msg_putchar('\n' as ::core::ffi::c_int);
        msg_puts(version_cflags.get());
        version_msg(b"\n\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
        version_msg(gettext(
            b"   system vimrc file: \"\0".as_ptr() as *const ::core::ffi::c_char
        ));
        version_msg(SYS_VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char);
        version_msg(b"\"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
        if *default_vim_dir.get() as ::core::ffi::c_int != NUL {
            version_msg(gettext(
                b"  fall-back for $VIM: \"\0".as_ptr() as *const ::core::ffi::c_char
            ));
            version_msg(default_vim_dir.get());
            version_msg(
                b"\"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char
            );
        }
        if *default_vimruntime_dir.get() as ::core::ffi::c_int != NUL {
            version_msg(gettext(
                b" f-b for $VIMRUNTIME: \"\0".as_ptr() as *const ::core::ffi::c_char
            ));
            version_msg(default_vimruntime_dir.get());
            version_msg(
                b"\"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char
            );
        }
    }
    version_msg(
        (if p_verbose > 0 as OptInt {
            b"\nRun :checkhealth for more info\0".as_ptr() as *const ::core::ffi::c_char
        } else if starting != 0 {
            b"\nRun \"nvim -V1 -v\" for more info\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\nRun \":verbose version\" for more info\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn may_show_intro() -> bool {
    return buf_is_empty(curbuf) as ::core::ffi::c_int != 0
        && (*curbuf).b_fname.is_null()
        && (*curbuf).handle == 1 as ::core::ffi::c_int
        && (*curwin).handle == LOWEST_WIN_ID as ::core::ffi::c_int
        && one_window(curwin, ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int != 0
        && vim_strchr(p_shm, SHM_INTRO as ::core::ffi::c_int).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn intro_message(mut colon: bool) {
    static lines: GlobalCell<[*mut ::core::ffi::c_char; 18]> = GlobalCell::new([
        b"\xE2\x94\x82 \xE2\x95\xB2 \xE2\x94\x82\xE2\x94\x82\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"\xE2\x94\x82\xE2\x94\x82\xE2\x95\xB2\xE2\x95\xB2\xE2\x94\x82\xE2\x94\x82\0"
            .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"\xE2\x94\x82\xE2\x94\x82 \xE2\x95\xB2 \xE2\x94\x82\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"NVIM v0.12.4\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        b"\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\0"
            .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"Nvim is open source and freely distributable\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"https://neovim.io/#chat\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        b"\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\0"
            .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"type  :help nvim<Enter>     if you are new! \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"type  :checkhealth<Enter>   to optimize Nvim\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"type  :q<Enter>             to exit         \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"type  :help<Enter>          for help        \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\0"
            .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"type  :help news<Enter>     for v%s.%s notes \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\xE2\x94\x80\0"
            .as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"Help poor children in Uganda!\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        b"type  :help Kuwasha<Enter>  for information \0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut lines_size: size_t = ::core::mem::size_of::<[*mut ::core::ffi::c_char; 18]>()
        .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut ::core::ffi::c_char; 18]>()
                .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as size_t,
        );
    '_c2rust_label: {
        if lines_size <= 9223372036854775807 as ::core::ffi::c_long as size_t {
        } else {
            __assert_fail(
                b"lines_size <= LONG_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/version.rs\0".as_ptr() as *const ::core::ffi::c_char,
                4258 as ::core::ffi::c_uint,
                b"void intro_message(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut blanklines: ::core::ffi::c_int =
        Rows - (lines_size as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    if p_ls > 1 as OptInt {
        blanklines -= Rows - (*topframe).fr_height;
    }
    if blanklines < 0 as ::core::ffi::c_int {
        blanklines = 0 as ::core::ffi::c_int;
    }
    let mut row: ::core::ffi::c_int = blanklines / 2 as ::core::ffi::c_int;
    if row >= 2 as ::core::ffi::c_int && Columns >= 50 as ::core::ffi::c_int
        || colon as ::core::ffi::c_int != 0
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 18]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 18]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
        {
            let mut p: *mut ::core::ffi::c_char =
                (*lines.ptr())[i as usize] as *mut ::core::ffi::c_char;
            let mut mesg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut mesg_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if !strstr(p, b"news\0".as_ptr() as *const ::core::ffi::c_char).is_null() {
                p = gettext(p);
                mesg_size = snprintf(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as size_t,
                    p,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"12\0".as_ptr() as *const ::core::ffi::c_char,
                );
                '_c2rust_label_0: {
                    if mesg_size > 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"mesg_size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/version.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            4284 as ::core::ffi::c_uint,
                            b"void intro_message(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                mesg = xmallocz(mesg_size as size_t) as *mut ::core::ffi::c_char;
                snprintf(
                    mesg,
                    (mesg_size as size_t).wrapping_add(1 as size_t),
                    p,
                    b"0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"12\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if mesg.is_null() {
                if *p as ::core::ffi::c_int != NUL {
                    mesg = gettext(p);
                } else {
                    mesg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
            }
            if *mesg as ::core::ffi::c_int != NUL && row < Rows - 1 as ::core::ffi::c_int {
                do_intro_line(row, mesg, colon, i < 3 as ::core::ffi::c_int);
            }
            row += 1;
            if mesg_size > 0 as ::core::ffi::c_int {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut mesg as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
            }
            i += 1;
        }
    }
}
unsafe extern "C" fn do_intro_line(
    mut row: ::core::ffi::c_int,
    mut mesg: *mut ::core::ffi::c_char,
    mut colon: bool,
    mut is_logo: bool,
) {
    let mut l: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_int = vim_strsize(mesg);
    col = (Columns - col) / 2 as ::core::ffi::c_int;
    if col < 0 as ::core::ffi::c_int {
        col = 0 as ::core::ffi::c_int;
    }
    grid_line_start(
        if !colon && ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 {
            &raw mut (*firstwin).w_grid
        } else {
            &raw mut default_gridview
        },
        row,
    );
    let mut id_attr: ::core::ffi::c_int = syn_id2attr(syn_name2id(
        b"Identifier\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut nontext_attr: ::core::ffi::c_int = syn_id2attr(syn_name2id(
        b"NonText\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut special_attr: ::core::ffi::c_int = syn_id2attr(syn_name2id(
        b"Special\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut string_attr: ::core::ffi::c_int = syn_id2attr(syn_name2id(
        b"String\0".as_ptr() as *const ::core::ffi::c_char
    ));
    if is_logo {
        let mut seen_diagonal: bool = false_0 != 0;
        let mut p: *mut ::core::ffi::c_char = mesg;
        while *p as ::core::ffi::c_int != NUL {
            let mut clen: ::core::ffi::c_int = utfc_ptr2len(p);
            let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if *p as uint8_t as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int {
                seen_diagonal = seen_diagonal as ::core::ffi::c_int != 0
                    || clen == 3 as ::core::ffi::c_int
                        && utf_ptr2char(p) == 0x2572 as ::core::ffi::c_int;
                attr = if seen_diagonal as ::core::ffi::c_int != 0 {
                    string_attr
                } else {
                    special_attr
                };
            }
            col += grid_line_puts(col, p, clen, attr);
            p = p.offset(clen as isize);
        }
        grid_line_flush();
        return;
    }
    let mut is_version: bool = *mesg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'N' as ::core::ffi::c_int
        && *mesg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'V' as ::core::ffi::c_int
        && *mesg.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'I' as ::core::ffi::c_int
        && *mesg.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'M' as ::core::ffi::c_int;
    let mut is_sep: bool = utfc_ptr2len(mesg) == 3 as ::core::ffi::c_int
        && utf_ptr2char(mesg) == 0x2500 as ::core::ffi::c_int;
    if is_version as ::core::ffi::c_int != 0 || is_sep as ::core::ffi::c_int != 0 {
        let mut clen_0: ::core::ffi::c_int = if is_sep as ::core::ffi::c_int != 0 {
            3 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        let mut attr_0: ::core::ffi::c_int = if is_sep as ::core::ffi::c_int != 0 {
            nontext_attr
        } else {
            string_attr
        };
        let mut p_0: *mut ::core::ffi::c_char = mesg;
        while *p_0 as ::core::ffi::c_int != NUL {
            col += grid_line_puts(col, p_0, clen_0, attr_0);
            p_0 = p_0.offset(clen_0 as isize);
        }
        grid_line_flush();
        return;
    }
    let mut p_1: *mut ::core::ffi::c_char = mesg;
    while *p_1 as ::core::ffi::c_int != NUL {
        l = 0 as ::core::ffi::c_int;
        while *p_1.offset(l as isize) as ::core::ffi::c_int != NUL
            && (l == 0 as ::core::ffi::c_int
                || *p_1.offset(l as isize) as ::core::ffi::c_int != '<' as ::core::ffi::c_int
                    && *p_1.offset((l - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                        != '>' as ::core::ffi::c_int)
        {
            l += utfc_ptr2len(p_1.offset(l as isize)) - 1 as ::core::ffi::c_int;
            l += 1;
        }
        '_c2rust_label: {
            if row <= 2147483647 as ::core::ffi::c_int && col <= 2147483647 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"row <= INT_MAX && col <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/version.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4374 as ::core::ffi::c_uint,
                    b"void do_intro_line(int, char *, _Bool, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if *p_1 as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            col += grid_line_puts(
                col,
                p_1,
                l,
                *hl_attr_active.offset(HLF_8 as ::core::ffi::c_int as isize),
            );
        } else {
            let mut colon_pos: *mut ::core::ffi::c_char = memchr(
                p_1 as *const ::core::ffi::c_void,
                ':' as ::core::ffi::c_int,
                l as size_t,
            ) as *mut ::core::ffi::c_char;
            if !colon_pos.is_null()
                && *p_1.offset(l as isize) as ::core::ffi::c_int == '<' as ::core::ffi::c_int
            {
                let mut prefix_len: ::core::ffi::c_int =
                    colon_pos.offset_from(p_1) as ::core::ffi::c_int;
                col += grid_line_puts(col, p_1, prefix_len, 0 as ::core::ffi::c_int);
                col += grid_line_puts(
                    col,
                    colon_pos,
                    1 as ::core::ffi::c_int,
                    *hl_attr_active.offset(HLF_8 as ::core::ffi::c_int as isize),
                );
                let mut cmd_len: ::core::ffi::c_int = l - prefix_len - 1 as ::core::ffi::c_int;
                col += grid_line_puts(
                    col,
                    colon_pos.offset(1 as ::core::ffi::c_int as isize),
                    cmd_len,
                    id_attr,
                );
            } else {
                col += grid_line_puts(col, p_1, l, 0 as ::core::ffi::c_int);
            }
        }
        p_1 = p_1.offset(l as isize);
    }
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn ex_intro(mut _eap: *mut exarg_T) {
    screenclear();
    intro_message(true_0 != 0);
    plain_vgetc();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NVIM_VERSION_MAJOR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NVIM_VERSION_MINOR: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const NVIM_VERSION_PATCH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
