extern "C" {
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
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
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn map_put_ref_uint32_t_uint32_t(
        map: *mut Map_uint32_t_uint32_t,
        key: uint32_t,
        key_alloc: *mut *mut uint32_t,
        new_item: *mut bool,
    ) -> *mut uint32_t;
    fn map_ref_uint32_t_uint32_t(
        map: *mut Map_uint32_t_uint32_t,
        key: uint32_t,
        key_alloc: *mut *mut uint32_t,
    ) -> *mut uint32_t;
    fn map_del_uint32_t_uint32_t(
        map: *mut Map_uint32_t_uint32_t,
        key: uint32_t,
        key_alloc: *mut uint32_t,
    ) -> uint32_t;
    fn buf_updates_send_splice(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        start_byte: bcount_t,
        old_row: ::core::ffi::c_int,
        old_col: colnr_T,
        old_byte: bcount_t,
        new_row: ::core::ffi::c_int,
        new_col: colnr_T,
        new_byte: bcount_t,
    );
    fn decor_redraw(
        buf: *mut buf_T,
        row1: ::core::ffi::c_int,
        row2: ::core::ffi::c_int,
        col1: ::core::ffi::c_int,
        decor: DecorInline,
    );
    fn buf_put_decor(
        buf: *mut buf_T,
        decor: DecorInline,
        row: ::core::ffi::c_int,
        row2: ::core::ffi::c_int,
    );
    fn buf_decor_remove(
        buf: *mut buf_T,
        row1: ::core::ffi::c_int,
        row2: ::core::ffi::c_int,
        col1: ::core::ffi::c_int,
        decor: DecorInline,
        free: bool,
    );
    fn decor_free(decor: DecorInline);
    fn decor_state_invalidate(buf: *mut buf_T);
    fn buf_signcols_count_range(
        buf: *mut buf_T,
        row1: ::core::ffi::c_int,
        row2: ::core::ffi::c_int,
        add: ::core::ffi::c_int,
        clear: TriState,
    );
    fn decor_type_flags(decor: DecorInline) -> uint16_t;
    static mut curbuf: *mut buf_T;
    static mut curbuf_splice_pending: ::core::ffi::c_int;
    fn marktree_put(
        b: *mut MarkTree,
        key: MTKey,
        end_row: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        end_right: bool,
    );
    fn marktree_del_itr(b: *mut MarkTree, itr: *mut MarkTreeIter, rev: bool) -> uint64_t;
    fn marktree_revise_meta(b: *mut MarkTree, itr: *mut MarkTreeIter, old_key: MTKey);
    fn marktree_clear(b: *mut MarkTree);
    fn marktree_move(
        b: *mut MarkTree,
        itr: *mut MarkTreeIter,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
    );
    fn marktree_itr_get(
        b: *mut MarkTree,
        row: int32_t,
        col: ::core::ffi::c_int,
        itr: *mut MarkTreeIter,
    ) -> bool;
    fn marktree_itr_get_ext(
        b: *mut MarkTree,
        p: MTPos,
        itr: *mut MarkTreeIter,
        last: bool,
        gravity: bool,
        oldbase: *mut MTPos,
        meta_filter: MetaFilter,
    ) -> bool;
    fn marktree_itr_next(b: *mut MarkTree, itr: *mut MarkTreeIter) -> bool;
    fn marktree_itr_current(itr: *mut MarkTreeIter) -> MTKey;
    fn marktree_itr_get_overlap(
        b: *mut MarkTree,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
        itr: *mut MarkTreeIter,
    ) -> bool;
    fn marktree_itr_step_overlap(
        b: *mut MarkTree,
        itr: *mut MarkTreeIter,
        pair: *mut MTPair,
    ) -> bool;
    fn marktree_splice(
        b: *mut MarkTree,
        start_line: int32_t,
        start_col: ::core::ffi::c_int,
        old_extent_line: ::core::ffi::c_int,
        old_extent_col: ::core::ffi::c_int,
        new_extent_line: ::core::ffi::c_int,
        new_extent_col: ::core::ffi::c_int,
    ) -> bool;
    fn marktree_move_region(
        b: *mut MarkTree,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        extent_row: ::core::ffi::c_int,
        extent_col: colnr_T,
        new_row: ::core::ffi::c_int,
        new_col: colnr_T,
    );
    fn marktree_lookup_ns(
        b: *mut MarkTree,
        ns: uint32_t,
        id: uint32_t,
        end: bool,
        itr: *mut MarkTreeIter,
    ) -> MTKey;
    fn marktree_lookup(b: *mut MarkTree, id: uint64_t, itr: *mut MarkTreeIter) -> MTKey;
    fn marktree_get_altpos(
        b: *mut MarkTree,
        mark: MTKey,
        itr: *mut MarkTreeIter,
    ) -> MTPos;
    fn marktree_get_alt(b: *mut MarkTree, mark: MTKey, itr: *mut MarkTreeIter) -> MTKey;
    fn ml_find_line_or_offset(
        buf: *mut buf_T,
        lnum: linenr_T,
        offp: *mut ::core::ffi::c_int,
        no_ff: bool,
    ) -> ::core::ffi::c_int;
    fn u_force_get_undo_header(buf: *mut buf_T) -> *mut u_header_T;
}
pub type ptrdiff_t = isize;
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
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct undo_object {
    pub type_0: UndoObjectType,
    pub data: C2Rust_Unnamed_6,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub splice: ExtmarkSplice,
    pub move_0: ExtmarkMove,
    pub savepos: ExtmarkSavePos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSavePos {
    pub mark: uint64_t,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub invalidated: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkMove {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub extent_row: ::core::ffi::c_int,
    pub extent_col: ::core::ffi::c_int,
    pub new_row: ::core::ffi::c_int,
    pub new_col: ::core::ffi::c_int,
    pub start_byte: bcount_t,
    pub extent_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type bcount_t = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSplice {
    pub start_row: ::core::ffi::c_int,
    pub start_col: colnr_T,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub new_row: ::core::ffi::c_int,
    pub new_col: colnr_T,
    pub start_byte: bcount_t,
    pub old_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type UndoObjectType = ::core::ffi::c_uint;
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorInline {
    pub ext: bool,
    pub data: DecorInlineData,
}
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
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
}
pub type ExtmarkOp = ::core::ffi::c_uint;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkInfoArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTPair,
}
pub type ExtmarkType = ::core::ffi::c_uint;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSignHL: ExtmarkType = 4;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const KV_INITIAL_VALUE: ExtmarkInfoArray = ExtmarkInfoArray {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<MTPair>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint32_t_uint32_t = Map_uint32_t_uint32_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<uint32_t>(),
};
#[no_mangle]
pub unsafe extern "C" fn extmark_set(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut idp: *mut uint32_t,
    mut row: ::core::ffi::c_int,
    mut col: colnr_T,
    mut end_row: ::core::ffi::c_int,
    mut end_col: colnr_T,
    mut decor: DecorInline,
    mut decor_flags: uint16_t,
    mut right_gravity: bool,
    mut end_right_gravity: bool,
    mut no_undo: bool,
    mut invalidate: bool,
    mut err: *mut Error,
) {
    let mut mark: MTKey = MTKey {
        pos: MTPos { row: 0, col: 0 },
        ns: 0,
        id: 0,
        flags: 0,
        decor_data: DecorInlineData {
            hl: DecorHighlightInline {
                flags: 0,
                priority: 0,
                hl_id: 0,
                conceal_char: 0,
            },
        },
    };
    let mut ns: *mut uint32_t = map_put_ref_uint32_t_uint32_t(
        &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
        ns_id,
        ::core::ptr::null_mut::<*mut uint32_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    let mut id: uint32_t = if !idp.is_null() { *idp } else { 0 as uint32_t };
    let mut flags: uint16_t = (mt_flags(right_gravity, no_undo, invalidate, decor.ext)
        as ::core::ffi::c_int | decor_flags as ::core::ffi::c_int) as uint16_t;
    '_revised: {
        if id == 0 as uint32_t {
            *ns = (*ns).wrapping_add(1);
            id = *ns;
        } else {
            let mut itr: [MarkTreeIter; 1] = [
                MarkTreeIter {
                    pos: MTPos { row: 0 as int32_t, col: 0 },
                    lvl: 0,
                    x: ::core::ptr::null_mut::<MTNode>(),
                    i: 0,
                    s: [C2Rust_Unnamed_14 {
                        oldcol: 0,
                        i: 0,
                    }; 20],
                    intersect_idx: 0,
                    intersect_pos: MTPos { row: 0, col: 0 },
                    intersect_pos_x: MTPos { row: 0, col: 0 },
                },
            ];
            let mut old_mark: MTKey = marktree_lookup_ns(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                ns_id,
                id,
                false_0 != 0,
                &raw mut itr as *mut MarkTreeIter,
            );
            if old_mark.id != 0 {
                if mt_paired(old_mark) as ::core::ffi::c_int != 0
                    || end_row > -1 as ::core::ffi::c_int
                {
                    extmark_del_id(buf, ns_id, id);
                } else {
                    '_c2rust_label: {
                        if !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {} else {
                            __assert_fail(
                                b"marktree_itr_valid(itr)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/extmark.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                70 as ::core::ffi::c_uint,
                                b"void extmark_set(buf_T *, uint32_t, uint32_t *, int, colnr_T, int, colnr_T, DecorInline, uint16_t, _Bool, _Bool, _Bool, _Bool, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if old_mark.pos.row == row as int32_t
                        && old_mark.pos.col == col as int32_t
                    {
                        if !mt_invalid(old_mark)
                            && mt_decor_any(old_mark) as ::core::ffi::c_int != 0
                        {
                            (*(*(&raw mut itr as *mut MarkTreeIter)).x)
                                .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x)
                                .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                                .flags as ::core::ffi::c_int
                                & !MT_FLAG_EXTERNAL_MASK as uint16_t as ::core::ffi::c_int)
                                as uint16_t;
                            buf_decor_remove(
                                buf,
                                row,
                                row,
                                col as ::core::ffi::c_int,
                                mt_decor(old_mark),
                                true_0 != 0,
                            );
                        }
                        (*(*(&raw mut itr as *mut MarkTreeIter)).x)
                            .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                            .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x)
                            .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                            .flags as ::core::ffi::c_int | flags as ::core::ffi::c_int)
                            as uint16_t;
                        (*(*(&raw mut itr as *mut MarkTreeIter)).x)
                            .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                            .decor_data = decor.data;
                        marktree_revise_meta(
                            &raw mut (*buf).b_marktree as *mut MarkTree,
                            &raw mut itr as *mut MarkTreeIter,
                            old_mark,
                        );
                        break '_revised;
                    } else {
                        marktree_del_itr(
                            &raw mut (*buf).b_marktree as *mut MarkTree,
                            &raw mut itr as *mut MarkTreeIter,
                            false_0 != 0,
                        );
                        if !mt_invalid(old_mark) {
                            buf_decor_remove(
                                buf,
                                old_mark.pos.row as ::core::ffi::c_int,
                                old_mark.pos.row as ::core::ffi::c_int,
                                old_mark.pos.col as ::core::ffi::c_int,
                                mt_decor(old_mark),
                                true_0 != 0,
                            );
                        }
                    }
                }
            } else {
                *ns = if *ns > id { *ns } else { id };
            }
        }
        mark = MTKey {
            pos: MTPos {
                row: row as int32_t,
                col: col as int32_t,
            },
            ns: ns_id,
            id: id,
            flags: flags,
            decor_data: decor.data,
        };
        marktree_put(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            mark,
            end_row,
            end_col as ::core::ffi::c_int,
            end_right_gravity,
        );
        decor_state_invalidate(buf);
    }
    if decor_flags as ::core::ffi::c_int != 0 || decor.ext as ::core::ffi::c_int != 0 {
        buf_put_decor(
            buf,
            decor,
            row,
            if end_row > -1 as ::core::ffi::c_int { end_row } else { row },
        );
        decor_redraw(
            buf,
            row,
            if end_row > -1 as ::core::ffi::c_int { end_row } else { row },
            col as ::core::ffi::c_int,
            decor,
        );
    }
    if !idp.is_null() {
        *idp = id;
    }
}
unsafe extern "C" fn extmark_setraw(
    mut buf: *mut buf_T,
    mut mark: uint64_t,
    mut row: ::core::ffi::c_int,
    mut col: colnr_T,
    mut invalid: bool,
) {
    let mut itr: [MarkTreeIter; 1] = [
        MarkTreeIter {
            pos: MTPos { row: 0 as int32_t, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        },
    ];
    let mut key: MTKey = marktree_lookup(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        mark,
        &raw mut itr as *mut MarkTreeIter,
    );
    let mut move_0: bool = key.pos.row != row as int32_t
        || key.pos.col != col as int32_t;
    if key.pos.row < 0 as int32_t || !move_0 && !invalid {
        return;
    }
    if !invalid && mt_decor_any(key) as ::core::ffi::c_int != 0
        && key.pos.row != row as int32_t
    {
        decor_redraw(
            buf,
            key.pos.row as ::core::ffi::c_int,
            key.pos.row as ::core::ffi::c_int,
            key.pos.col as ::core::ffi::c_int,
            mt_decor(key),
        );
    }
    let mut row1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut altitr: [MarkTreeIter; 1] = [*(&raw mut itr as *mut MarkTreeIter)];
    let mut alt: MTKey = marktree_get_alt(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        key,
        &raw mut altitr as *mut MarkTreeIter,
    );
    if invalid {
        (*(*(&raw mut itr as *mut MarkTreeIter)).x)
            .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
            .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x)
            .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
            .flags as ::core::ffi::c_int
            & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int) as uint16_t;
        (*(*(&raw mut altitr as *mut MarkTreeIter)).x)
            .key[(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
            .flags = ((*(*(&raw mut altitr as *mut MarkTreeIter)).x)
            .key[(*(&raw mut altitr as *mut MarkTreeIter)).i as usize]
            .flags as ::core::ffi::c_int
            & !MT_FLAG_INVALID as uint16_t as ::core::ffi::c_int) as uint16_t;
        marktree_revise_meta(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            if mt_end(key) as ::core::ffi::c_int != 0 {
                &raw mut altitr as *mut MarkTreeIter
            } else {
                &raw mut itr as *mut MarkTreeIter
            },
            if mt_end(key) as ::core::ffi::c_int != 0 { alt } else { key },
        );
    } else if !mt_invalid(key)
        && key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
        && (*buf).b_signcols.autom as ::core::ffi::c_int != 0
    {
        row1 = (if alt.pos.row
            < (if key.pos.row < row as int32_t { key.pos.row } else { row as int32_t })
        {
            alt.pos.row
        } else if key.pos.row < row as int32_t {
            key.pos.row
        } else {
            row as int32_t
        }) as ::core::ffi::c_int;
        row2 = (if alt.pos.row
            > (if key.pos.row > row as int32_t { key.pos.row } else { row as int32_t })
        {
            alt.pos.row
        } else if key.pos.row > row as int32_t {
            key.pos.row
        } else {
            row as int32_t
        }) as ::core::ffi::c_int;
        buf_signcols_count_range(
            buf,
            row1,
            if ((*curbuf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
                (*curbuf).b_ml.ml_line_count as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            } else {
                row2
            },
            0 as ::core::ffi::c_int,
            kTrue,
        );
    }
    if move_0 {
        marktree_move(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row,
            col as ::core::ffi::c_int,
        );
    }
    if invalid {
        buf_put_decor(
            buf,
            mt_decor(key),
            if (row as int32_t) < alt.pos.row {
                row
            } else {
                alt.pos.row as ::core::ffi::c_int
            },
            if row as int32_t > alt.pos.row {
                row
            } else {
                alt.pos.row as ::core::ffi::c_int
            },
        );
    } else if !mt_invalid(key)
        && key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
        && (*buf).b_signcols.autom as ::core::ffi::c_int != 0
    {
        buf_signcols_count_range(
            buf,
            row1,
            if ((*curbuf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
                (*curbuf).b_ml.ml_line_count as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            } else {
                row2
            },
            0 as ::core::ffi::c_int,
            kNone,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_del_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> bool {
    let mut itr: [MarkTreeIter; 1] = [
        MarkTreeIter {
            pos: MTPos { row: 0 as int32_t, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        },
    ];
    let mut key: MTKey = marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns_id,
        id,
        false_0 != 0,
        &raw mut itr as *mut MarkTreeIter,
    );
    if key.id != 0 {
        extmark_del(buf, &raw mut itr as *mut MarkTreeIter, key, false_0 != 0);
    }
    return key.id > 0 as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn extmark_del(
    mut buf: *mut buf_T,
    mut itr: *mut MarkTreeIter,
    mut key: MTKey,
    mut restore: bool,
) {
    '_c2rust_label: {
        if key.pos.row >= 0 as int32_t {} else {
            __assert_fail(
                b"key.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/extmark.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                167 as ::core::ffi::c_uint,
                b"void extmark_del(buf_T *, MarkTreeIter *, MTKey, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut key2: MTKey = key;
    let mut other: uint64_t = marktree_del_itr(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        itr,
        false_0 != 0,
    );
    if other != 0 {
        key2 = marktree_lookup(&raw mut (*buf).b_marktree as *mut MarkTree, other, itr);
        '_c2rust_label_0: {
            if key2.pos.row >= 0 as int32_t {} else {
                __assert_fail(
                    b"key2.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/extmark.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    173 as ::core::ffi::c_uint,
                    b"void extmark_del(buf_T *, MarkTreeIter *, MTKey, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        marktree_del_itr(&raw mut (*buf).b_marktree as *mut MarkTree, itr, false_0 != 0);
        if restore {
            marktree_itr_get(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                key.pos.row,
                key.pos.col as ::core::ffi::c_int,
                itr,
            );
        }
    }
    if mt_decor_any(key) {
        if mt_invalid(key) {
            decor_free(mt_decor(key));
        } else {
            if mt_end(key) {
                let mut k: MTKey = key;
                key = key2;
                key2 = k;
            }
            buf_decor_remove(
                buf,
                key.pos.row as ::core::ffi::c_int,
                key2.pos.row as ::core::ffi::c_int,
                key.pos.col as ::core::ffi::c_int,
                mt_decor(key),
                true_0 != 0,
            );
        }
    }
    decor_state_invalidate(buf);
}
#[no_mangle]
pub unsafe extern "C" fn extmark_clear(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
) -> bool {
    if (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set.h.size == 0 {
        return false_0 != 0;
    }
    let mut all_ns: bool = ns_id == 0 as uint32_t;
    let mut ns: *mut uint32_t = ::core::ptr::null_mut::<uint32_t>();
    if !all_ns {
        ns = map_ref_uint32_t_uint32_t(
            &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
            ns_id,
            ::core::ptr::null_mut::<*mut uint32_t>(),
        );
        if ns.is_null() {
            return false_0 != 0;
        }
    }
    let mut marks_cleared_any: bool = false_0 != 0;
    let mut marks_cleared_all: bool = l_row == 0 as ::core::ffi::c_int
        && l_col == 0 as ::core::ffi::c_int;
    let mut itr: [MarkTreeIter; 1] = [
        MarkTreeIter {
            pos: MTPos { row: 0 as int32_t, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        },
    ];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        l_row as int32_t,
        l_col as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > u_row as int32_t
            || mark.pos.row == u_row as int32_t && mark.pos.col > u_col as int32_t
        {
            if mark.pos.row >= 0 as int32_t {
                marks_cleared_all = false_0 != 0;
            }
            break;
        } else if mark.ns == ns_id || all_ns as ::core::ffi::c_int != 0 {
            marks_cleared_any = true_0 != 0;
            extmark_del(buf, &raw mut itr as *mut MarkTreeIter, mark, true_0 != 0);
        } else {
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
    }
    if marks_cleared_all {
        if all_ns {
            xfree(
                (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set.keys
                    as *mut ::core::ffi::c_void,
            );
            xfree(
                (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t))
                    .set
                    .h
                    .hash as *mut ::core::ffi::c_void,
            );
            (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set = SET_INIT;
            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*buf)
                .b_extmark_ns as *mut Map_uint32_t_uint32_t))
                .values as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
            *(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t) = MAP_INIT;
        } else {
            map_del_uint32_t_uint32_t(
                &raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t,
                ns_id,
                ::core::ptr::null_mut::<uint32_t>(),
            );
        }
    }
    if marks_cleared_any {
        decor_state_invalidate(buf);
    }
    return marks_cleared_any;
}
#[no_mangle]
pub unsafe extern "C" fn extmark_get(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
    mut amount: int64_t,
    mut type_filter: ExtmarkType,
    mut overlap: bool,
) -> ExtmarkInfoArray {
    let mut array: ExtmarkInfoArray = KV_INITIAL_VALUE;
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_14 {
            oldcol: 0,
            i: 0,
        }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    if overlap {
        if !marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            l_row,
            l_col as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        ) {
            return array;
        }
        while (array.size as int64_t) < amount {
            let mut pair: MTPair = MTPair {
                start: MTKey {
                    pos: MTPos { row: 0, col: 0 },
                    ns: 0,
                    id: 0,
                    flags: 0,
                    decor_data: DecorInlineData {
                        hl: DecorHighlightInline {
                            flags: 0,
                            priority: 0,
                            hl_id: 0,
                            conceal_char: 0,
                        },
                    },
                },
                end_pos: MTPos { row: 0, col: 0 },
                end_right_gravity: false,
            };
            if !marktree_itr_step_overlap(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                &raw mut pair,
            ) {
                break;
            }
            push_mark(&raw mut array, ns_id, type_filter, pair);
        }
    } else {
        marktree_itr_get_ext(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            MTPos {
                row: l_row as int32_t,
                col: l_col as int32_t,
            },
            &raw mut itr as *mut MarkTreeIter,
            false_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
    while (array.size as int64_t) < amount {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > u_row as int32_t
            || mark.pos.row == u_row as int32_t && mark.pos.col > u_col as int32_t
        {
            break;
        }
        if !mt_end(mark) {
            let mut end: MTKey = marktree_get_alt(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                mark,
                ::core::ptr::null_mut::<MarkTreeIter>(),
            );
            push_mark(&raw mut array, ns_id, type_filter, mtpair_from(mark, end));
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    return array;
}
unsafe extern "C" fn push_mark(
    mut array: *mut ExtmarkInfoArray,
    mut ns_id: uint32_t,
    mut type_filter: ExtmarkType,
    mut mark: MTPair,
) {
    if !(ns_id == UINT32_MAX as uint32_t || mark.start.ns == ns_id) {
        return;
    }
    if type_filter as ::core::ffi::c_uint
        != kExtmarkNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !mt_decor_any(mark.start) {
            return;
        }
        let mut type_flags: uint16_t = decor_type_flags(mt_decor(mark.start));
        if type_flags as ::core::ffi::c_uint & type_filter as ::core::ffi::c_uint == 0 {
            return;
        }
    }
    if (*array).size == (*array).capacity {
        (*array).capacity = (if (*array).capacity != 0 {
            (*array).capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        });
        (*array).items = xrealloc(
            (*array).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<MTPair>().wrapping_mul((*array).capacity),
        ) as *mut MTPair;
    } else {};
    let c2rust_fresh0 = (*array).size;
    (*array).size = (*array).size.wrapping_add(1);
    *(*array).items.offset(c2rust_fresh0 as isize) = mark;
}
#[no_mangle]
pub unsafe extern "C" fn extmark_from_id(
    mut buf: *mut buf_T,
    mut ns_id: uint32_t,
    mut id: uint32_t,
) -> MTPair {
    let mut mark: MTKey = marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns_id,
        id,
        false_0 != 0,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    );
    if mark.id == 0 {
        return mtpair_from(mark, mark);
    }
    '_c2rust_label: {
        if mark.pos.row >= 0 as int32_t {} else {
            __assert_fail(
                b"mark.pos.row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/extmark.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                328 as ::core::ffi::c_uint,
                b"MTPair extmark_from_id(buf_T *, uint32_t, uint32_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut end: MTKey = marktree_get_alt(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        mark,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    );
    return mtpair_from(mark, end);
}
#[no_mangle]
pub unsafe extern "C" fn extmark_free_all(mut buf: *mut buf_T) {
    let mut itr: [MarkTreeIter; 1] = [
        MarkTreeIter {
            pos: MTPos { row: 0 as int32_t, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        },
    ];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        0 as int32_t,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t {
            break;
        }
        if !(mt_paired(mark) as ::core::ffi::c_int != 0
            && mt_end(mark) as ::core::ffi::c_int != 0)
        {
            decor_free(mt_decor(mark));
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    marktree_clear(&raw mut (*buf).b_marktree as *mut MarkTree);
    (*buf).b_signcols.max = 0 as ::core::ffi::c_int;
    memset(
        &raw mut (*buf).b_signcols.count as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_int; 9]>(),
    );
    xfree(
        (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set.keys
            as *mut ::core::ffi::c_void,
    );
    xfree(
        (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set.h.hash
            as *mut ::core::ffi::c_void,
    );
    (*(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t)).set = SET_INIT;
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*buf)
        .b_extmark_ns as *mut Map_uint32_t_uint32_t))
        .values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    *(&raw mut (*buf).b_extmark_ns as *mut Map_uint32_t_uint32_t) = MAP_INIT;
}
#[no_mangle]
pub unsafe extern "C" fn extmark_splice_delete(
    mut buf: *mut buf_T,
    mut l_row: ::core::ffi::c_int,
    mut l_col: colnr_T,
    mut u_row: ::core::ffi::c_int,
    mut u_col: colnr_T,
    mut uvp: *mut extmark_undo_vec_t,
    mut only_copy: bool,
    mut op: ExtmarkOp,
) {
    let mut itr: [MarkTreeIter; 1] = [
        MarkTreeIter {
            pos: MTPos { row: 0 as int32_t, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_14 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        },
    ];
    let mut undo: ExtmarkUndoObject = ExtmarkUndoObject {
        type_0: kExtmarkSplice,
        data: C2Rust_Unnamed_6 {
            splice: ExtmarkSplice {
                start_row: 0,
                start_col: 0,
                old_row: 0,
                old_col: 0,
                new_row: 0,
                new_col: 0,
                start_byte: 0,
                old_byte: 0,
                new_byte: 0,
            },
        },
    };
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        l_row as int32_t,
        l_col as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > u_row as int32_t {
            break;
        }
        let mut copy: bool = true_0 != 0;
        if mark.pos.row == l_row as int32_t
            && (mark.pos.col - !mt_right(mark) as ::core::ffi::c_int) < l_col as int32_t
        {
            copy = false_0 != 0;
        } else if mark.pos.row == u_row as int32_t {
            if mark.pos.col > u_col as int32_t + 1 as int32_t {
                break;
            }
            if mark.pos.col + mt_right(mark) as int32_t > u_col as int32_t {
                copy = false_0 != 0;
            }
        }
        let mut invalidated: bool = false_0 != 0;
        if !only_copy && !mt_invalid(mark)
            && mt_invalidate(mark) as ::core::ffi::c_int != 0 && !mt_end(mark)
        {
            let mut enditr: [MarkTreeIter; 1] = [*(&raw mut itr as *mut MarkTreeIter)];
            let mut endpos: MTPos = marktree_get_altpos(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                mark,
                &raw mut enditr as *mut MarkTreeIter,
            );
            if !mt_paired(mark) && mark.pos.row < u_row as int32_t
                || mt_paired(mark) as ::core::ffi::c_int != 0
                    && (mark.pos.row > l_row as int32_t
                        || mark.pos.row == l_row as int32_t
                            && mark.pos.col >= l_col as int32_t)
                    && (endpos.row < u_row as int32_t
                        || endpos.row == u_row as int32_t
                            && endpos.col <= u_col as int32_t)
            {
                if mt_no_undo(mark) {
                    extmark_del(
                        buf,
                        &raw mut itr as *mut MarkTreeIter,
                        mark,
                        true_0 != 0,
                    );
                    continue;
                } else {
                    copy = true_0 != 0;
                    invalidated = true_0 != 0;
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x)
                        .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x)
                        .key[(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .flags as ::core::ffi::c_int | MT_FLAG_INVALID) as uint16_t;
                    (*(*(&raw mut enditr as *mut MarkTreeIter)).x)
                        .key[(*(&raw mut enditr as *mut MarkTreeIter)).i as usize]
                        .flags = ((*(*(&raw mut enditr as *mut MarkTreeIter)).x)
                        .key[(*(&raw mut enditr as *mut MarkTreeIter)).i as usize]
                        .flags as ::core::ffi::c_int | MT_FLAG_INVALID) as uint16_t;
                    marktree_revise_meta(
                        &raw mut (*buf).b_marktree as *mut MarkTree,
                        &raw mut itr as *mut MarkTreeIter,
                        mark,
                    );
                    buf_decor_remove(
                        buf,
                        mark.pos.row as ::core::ffi::c_int,
                        endpos.row as ::core::ffi::c_int,
                        mark.pos.col as ::core::ffi::c_int,
                        mt_decor(mark),
                        false_0 != 0,
                    );
                }
            }
        }
        if copy as ::core::ffi::c_int != 0
            && (only_copy as ::core::ffi::c_int != 0
                || !uvp.is_null()
                    && op as ::core::ffi::c_uint
                        == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !mt_no_undo(mark))
        {
            let mut pos: ExtmarkSavePos = ExtmarkSavePos {
                mark: mt_lookup_key(mark),
                old_row: mark.pos.row as ::core::ffi::c_int,
                old_col: mark.pos.col as colnr_T,
                invalidated: invalidated,
            };
            undo.data.savepos = pos;
            undo.type_0 = kExtmarkSavePos;
            if (*uvp).size == (*uvp).capacity {
                (*uvp).capacity = (if (*uvp).capacity != 0 {
                    (*uvp).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                });
                (*uvp).items = xrealloc(
                    (*uvp).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<ExtmarkUndoObject>()
                        .wrapping_mul((*uvp).capacity),
                ) as *mut ExtmarkUndoObject;
            } else {};
            let c2rust_fresh1 = (*uvp).size;
            (*uvp).size = (*uvp).size.wrapping_add(1);
            *(*uvp).items.offset(c2rust_fresh1 as isize) = undo;
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn extmark_apply_undo(
    mut undo_info: ExtmarkUndoObject,
    mut undo: bool,
) {
    if undo_info.type_0 as ::core::ffi::c_uint
        == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut splice: ExtmarkSplice = undo_info.data.splice;
        if undo {
            extmark_splice_impl(
                curbuf,
                splice.start_row,
                splice.start_col,
                splice.start_byte,
                splice.new_row,
                splice.new_col,
                splice.new_byte,
                splice.old_row,
                splice.old_col,
                splice.old_byte,
                kExtmarkNoUndo,
            );
        } else {
            extmark_splice_impl(
                curbuf,
                splice.start_row,
                splice.start_col,
                splice.start_byte,
                splice.old_row,
                splice.old_col,
                splice.old_byte,
                splice.new_row,
                splice.new_col,
                splice.new_byte,
                kExtmarkNoUndo,
            );
        }
    } else if undo_info.type_0 as ::core::ffi::c_uint
        == kExtmarkSavePos as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut pos: ExtmarkSavePos = undo_info.data.savepos;
        if undo as ::core::ffi::c_int != 0 && pos.old_row >= 0 as ::core::ffi::c_int {
            extmark_setraw(curbuf, pos.mark, pos.old_row, pos.old_col, pos.invalidated);
        }
    } else if undo_info.type_0 as ::core::ffi::c_uint
        == kExtmarkMove as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut move_0: ExtmarkMove = undo_info.data.move_0;
        if undo {
            extmark_move_region(
                curbuf,
                move_0.new_row,
                move_0.new_col as colnr_T,
                move_0.new_byte,
                move_0.extent_row,
                move_0.extent_col as colnr_T,
                move_0.extent_byte,
                move_0.start_row,
                move_0.start_col as colnr_T,
                move_0.start_byte,
                kExtmarkNoUndo,
            );
        } else {
            extmark_move_region(
                curbuf,
                move_0.start_row,
                move_0.start_col as colnr_T,
                move_0.start_byte,
                move_0.extent_row,
                move_0.extent_col as colnr_T,
                move_0.extent_byte,
                move_0.new_row,
                move_0.new_col as colnr_T,
                move_0.new_byte,
                kExtmarkNoUndo,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut undo: ExtmarkOp,
) {
    if curbuf_splice_pending != 0 {
        return;
    }
    let mut start_byte: bcount_t = ml_find_line_or_offset(
        buf,
        line1,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut old_byte: bcount_t = 0 as bcount_t;
    let mut new_byte: bcount_t = 0 as bcount_t;
    let mut old_row: ::core::ffi::c_int = 0;
    let mut new_row: ::core::ffi::c_int = 0;
    if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
        old_row = (line2 - line1 + 1 as linenr_T) as ::core::ffi::c_int;
        old_byte = (*buf).deleted_bytes2 as bcount_t;
        new_row = (amount_after + old_row as linenr_T) as ::core::ffi::c_int;
    } else {
        '_c2rust_label: {
            if line2 == MAXLNUM as ::core::ffi::c_int as linenr_T {} else {
                __assert_fail(
                    b"line2 == MAXLNUM\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/extmark.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    500 as ::core::ffi::c_uint,
                    b"void extmark_adjust(buf_T *, linenr_T, linenr_T, linenr_T, linenr_T, ExtmarkOp)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        old_row = 0 as ::core::ffi::c_int;
        new_row = amount as ::core::ffi::c_int;
    }
    if new_row > 0 as ::core::ffi::c_int {
        new_byte = ml_find_line_or_offset(
            buf,
            line1 + new_row as linenr_T,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            true_0 != 0,
        ) as bcount_t - start_byte;
    }
    extmark_splice_impl(
        buf,
        line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        start_byte,
        old_row,
        0 as colnr_T,
        old_byte,
        new_row,
        0 as colnr_T,
        new_byte,
        undo,
    );
}
#[no_mangle]
pub unsafe extern "C" fn extmark_splice(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut old_row: ::core::ffi::c_int,
    mut old_col: colnr_T,
    mut old_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    let mut offset: ::core::ffi::c_int = ml_find_line_or_offset(
        buf,
        start_row as linenr_T + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    );
    if offset < 0 as ::core::ffi::c_int && (*buf).b_ml.ml_chunksize.is_null() {
        offset = 0 as ::core::ffi::c_int;
    }
    extmark_splice_impl(
        buf,
        start_row,
        start_col,
        (offset as colnr_T + start_col) as bcount_t,
        old_row,
        old_col,
        old_byte,
        new_row,
        new_col,
        new_byte,
        undo,
    );
}
#[no_mangle]
pub unsafe extern "C" fn extmark_splice_impl(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut start_byte: bcount_t,
    mut old_row: ::core::ffi::c_int,
    mut old_col: colnr_T,
    mut old_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    (*buf).deleted_bytes2 = 0 as size_t;
    buf_updates_send_splice(
        buf,
        start_row,
        start_col,
        start_byte,
        old_row,
        old_col,
        old_byte,
        new_row,
        new_col,
        new_byte,
    );
    if old_row > 0 as ::core::ffi::c_int || old_col > 0 as ::core::ffi::c_int {
        let mut end_row: ::core::ffi::c_int = start_row + old_row;
        let mut end_col: ::core::ffi::c_int = (if old_row != 0 {
            0 as ::core::ffi::c_int
        } else {
            start_col as ::core::ffi::c_int
        }) + old_col as ::core::ffi::c_int;
        let mut uhp: *mut u_header_T = u_force_get_undo_header(buf);
        let mut uvp: *mut extmark_undo_vec_t = if !uhp.is_null() {
            &raw mut (*uhp).uh_extmark
        } else {
            ::core::ptr::null_mut::<extmark_undo_vec_t>()
        };
        extmark_splice_delete(
            buf,
            start_row,
            start_col,
            end_row,
            end_col as colnr_T,
            uvp,
            false_0 != 0,
            undo,
        );
    }
    if old_row > 0 as ::core::ffi::c_int || new_row > 0 as ::core::ffi::c_int {
        let mut count: ::core::ffi::c_int = if (*buf).b_prev_line_count
            > 0 as ::core::ffi::c_int
        {
            (*buf).b_prev_line_count
        } else {
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int
        };
        buf_signcols_count_range(
            buf,
            start_row,
            if (count - 1 as ::core::ffi::c_int) < start_row + old_row {
                count - 1 as ::core::ffi::c_int
            } else {
                start_row + old_row
            },
            0 as ::core::ffi::c_int,
            kTrue,
        );
        (*buf).b_prev_line_count = 0 as ::core::ffi::c_int;
    }
    marktree_splice(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        start_row as int32_t,
        start_col as ::core::ffi::c_int,
        old_row,
        old_col as ::core::ffi::c_int,
        new_row,
        new_col as ::core::ffi::c_int,
    );
    if old_row > 0 as ::core::ffi::c_int || new_row > 0 as ::core::ffi::c_int {
        let mut row2: ::core::ffi::c_int = if ((*buf).b_ml.ml_line_count - 1 as linenr_T)
            < start_row as linenr_T + new_row as linenr_T
        {
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            start_row + new_row
        };
        buf_signcols_count_range(buf, start_row, row2, 0 as ::core::ffi::c_int, kNone);
    }
    if undo as ::core::ffi::c_uint
        == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut uhp_0: *mut u_header_T = u_force_get_undo_header(buf);
        if uhp_0.is_null() {
            return;
        }
        let mut merged: bool = false_0 != 0;
        if old_row == 0 as ::core::ffi::c_int && new_row == 0 as ::core::ffi::c_int
            && (*uhp_0).uh_extmark.size != 0
        {
            let mut item: *mut ExtmarkUndoObject = (*uhp_0)
                .uh_extmark
                .items
                .offset((*uhp_0).uh_extmark.size.wrapping_sub(1 as size_t) as isize);
            if (*item).type_0 as ::core::ffi::c_uint
                == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut splice: *mut ExtmarkSplice = &raw mut (*item).data.splice;
                if (*splice).start_row == start_row
                    && (*splice).old_row == 0 as ::core::ffi::c_int
                    && (*splice).new_row == 0 as ::core::ffi::c_int
                {
                    if old_col == 0 as ::core::ffi::c_int
                        && start_col >= (*splice).start_col
                        && start_col <= (*splice).start_col + (*splice).new_col
                    {
                        (*splice).new_col += new_col;
                        (*splice).new_byte += new_byte;
                        merged = true_0 != 0;
                    } else if new_col == 0 as ::core::ffi::c_int
                        && start_col == (*splice).start_col + (*splice).new_col
                    {
                        (*splice).old_col += old_col;
                        (*splice).old_byte += old_byte;
                        merged = true_0 != 0;
                    } else if new_col == 0 as ::core::ffi::c_int
                        && start_col + old_col == (*splice).start_col
                    {
                        (*splice).start_col = start_col;
                        (*splice).start_byte = start_byte;
                        (*splice).old_col += old_col;
                        (*splice).old_byte += old_byte;
                        merged = true_0 != 0;
                    }
                }
            }
        }
        if !merged {
            let mut splice_0: ExtmarkSplice = ExtmarkSplice {
                start_row: 0,
                start_col: 0,
                old_row: 0,
                old_col: 0,
                new_row: 0,
                new_col: 0,
                start_byte: 0,
                old_byte: 0,
                new_byte: 0,
            };
            splice_0.start_row = start_row;
            splice_0.start_col = start_col;
            splice_0.start_byte = start_byte;
            splice_0.old_row = old_row;
            splice_0.old_col = old_col;
            splice_0.old_byte = old_byte;
            splice_0.new_row = new_row;
            splice_0.new_col = new_col;
            splice_0.new_byte = new_byte;
            if (*uhp_0).uh_extmark.size == (*uhp_0).uh_extmark.capacity {
                (*uhp_0).uh_extmark.capacity = (if (*uhp_0).uh_extmark.capacity != 0 {
                    (*uhp_0).uh_extmark.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                });
                (*uhp_0).uh_extmark.items = xrealloc(
                    (*uhp_0).uh_extmark.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<ExtmarkUndoObject>()
                        .wrapping_mul((*uhp_0).uh_extmark.capacity),
                ) as *mut ExtmarkUndoObject;
            } else {};
            let c2rust_fresh3 = (*uhp_0).uh_extmark.size;
            (*uhp_0).uh_extmark.size = (*uhp_0).uh_extmark.size.wrapping_add(1);
            *(*uhp_0).uh_extmark.items.offset(c2rust_fresh3 as isize) = undo_object {
                type_0: kExtmarkSplice,
                data: C2Rust_Unnamed_6 {
                    splice: splice_0,
                },
            };
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn extmark_splice_cols(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut old_col: colnr_T,
    mut new_col: colnr_T,
    mut undo: ExtmarkOp,
) {
    extmark_splice(
        buf,
        start_row,
        start_col,
        0 as ::core::ffi::c_int,
        old_col,
        old_col as bcount_t,
        0 as ::core::ffi::c_int,
        new_col,
        new_col as bcount_t,
        undo,
    );
}
#[no_mangle]
pub unsafe extern "C" fn extmark_move_region(
    mut buf: *mut buf_T,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut start_byte: bcount_t,
    mut extent_row: ::core::ffi::c_int,
    mut extent_col: colnr_T,
    mut extent_byte: bcount_t,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
    mut new_byte: bcount_t,
    mut undo: ExtmarkOp,
) {
    (*buf).deleted_bytes2 = 0 as size_t;
    buf_updates_send_splice(
        buf,
        start_row,
        start_col,
        start_byte,
        extent_row,
        extent_col,
        extent_byte,
        0 as ::core::ffi::c_int,
        0 as colnr_T,
        0 as bcount_t,
    );
    let mut row1: ::core::ffi::c_int = if start_row < new_row {
        start_row
    } else {
        new_row
    };
    let mut row2: ::core::ffi::c_int = (if start_row > new_row {
        start_row
    } else {
        new_row
    }) + extent_row;
    buf_signcols_count_range(buf, row1, row2, 0 as ::core::ffi::c_int, kTrue);
    marktree_move_region(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        start_row,
        start_col,
        extent_row,
        extent_col,
        new_row,
        new_col,
    );
    buf_signcols_count_range(buf, row1, row2, 0 as ::core::ffi::c_int, kNone);
    buf_updates_send_splice(
        buf,
        new_row,
        new_col,
        new_byte,
        0 as ::core::ffi::c_int,
        0 as colnr_T,
        0 as bcount_t,
        extent_row,
        extent_col,
        extent_byte,
    );
    if undo as ::core::ffi::c_uint
        == kExtmarkUndo as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut uhp: *mut u_header_T = u_force_get_undo_header(buf);
        if uhp.is_null() {
            return;
        }
        let mut move_0: ExtmarkMove = ExtmarkMove {
            start_row: 0,
            start_col: 0,
            extent_row: 0,
            extent_col: 0,
            new_row: 0,
            new_col: 0,
            start_byte: 0,
            extent_byte: 0,
            new_byte: 0,
        };
        move_0.start_row = start_row;
        move_0.start_col = start_col as ::core::ffi::c_int;
        move_0.start_byte = start_byte;
        move_0.extent_row = extent_row;
        move_0.extent_col = extent_col as ::core::ffi::c_int;
        move_0.extent_byte = extent_byte;
        move_0.new_row = new_row;
        move_0.new_col = new_col as ::core::ffi::c_int;
        move_0.new_byte = new_byte;
        if (*uhp).uh_extmark.size == (*uhp).uh_extmark.capacity {
            (*uhp).uh_extmark.capacity = (if (*uhp).uh_extmark.capacity != 0 {
                (*uhp).uh_extmark.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            });
            (*uhp).uh_extmark.items = xrealloc(
                (*uhp).uh_extmark.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ExtmarkUndoObject>()
                    .wrapping_mul((*uhp).uh_extmark.capacity),
            ) as *mut ExtmarkUndoObject;
        } else {};
        let c2rust_fresh2 = (*uhp).uh_extmark.size;
        (*uhp).uh_extmark.size = (*uhp).uh_extmark.size.wrapping_add(1);
        *(*uhp).uh_extmark.items.offset(c2rust_fresh2 as isize) = undo_object {
            type_0: kExtmarkMove,
            data: C2Rust_Unnamed_6 { move_0: move_0 },
        };
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MT_FLAG_END: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const MT_FLAG_PAIRED: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const MT_FLAG_NO_UNDO: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const MT_FLAG_INVALIDATE: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
pub const MT_FLAG_INVALID: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int = (1 as ::core::ffi::c_int
    as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int = (1 as ::core::ffi::c_int as uint16_t
    as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int = (1 as ::core::ffi::c_int
    as uint16_t as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int = (1 as ::core::ffi::c_int
    as uint16_t as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int = (1 as ::core::ffi::c_int
    as uint16_t as ::core::ffi::c_int) << 13 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int = (1 as ::core::ffi::c_int
    as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_EXT | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;
pub const MT_FLAG_EXTERNAL_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_MASK
    | MT_FLAG_NO_UNDO | MT_FLAG_INVALIDATE | MT_FLAG_INVALID
    | MT_FLAG_DECOR_CONCEAL_LINES;
pub const MARKTREE_END_FLAG: uint64_t = 1 as ::core::ffi::c_int as uint64_t;
#[inline]
unsafe extern "C" fn mt_lookup_id(
    mut ns: uint32_t,
    mut id: uint32_t,
    mut enda: bool,
) -> uint64_t {
    return (ns as uint64_t) << 33 as ::core::ffi::c_int
        | (id << 1 as ::core::ffi::c_int) as uint64_t
        | (if enda as ::core::ffi::c_int != 0 {
            MARKTREE_END_FLAG
        } else {
            0 as uint64_t
        });
}
#[inline]
unsafe extern "C" fn mt_lookup_key(mut key: MTKey) -> uint64_t {
    return mt_lookup_id(
        key.ns,
        key.id,
        key.flags as ::core::ffi::c_int & MT_FLAG_END != 0,
    );
}
#[inline]
unsafe extern "C" fn mt_paired(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_PAIRED != 0;
}
#[inline]
unsafe extern "C" fn mt_end(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_END != 0;
}
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_no_undo(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_NO_UNDO != 0;
}
#[inline]
unsafe extern "C" fn mt_invalidate(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALIDATE != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor_any(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_MASK != 0;
}
#[inline]
unsafe extern "C" fn mt_flags(
    mut right_gravity: bool,
    mut no_undo: bool,
    mut invalidate: bool,
    mut decor_ext: bool,
) -> uint16_t {
    return ((if right_gravity as ::core::ffi::c_int != 0 {
        MT_FLAG_RIGHT_GRAVITY
    } else {
        0 as ::core::ffi::c_int
    })
        | (if no_undo as ::core::ffi::c_int != 0 {
            MT_FLAG_NO_UNDO
        } else {
            0 as ::core::ffi::c_int
        })
        | (if invalidate as ::core::ffi::c_int != 0 {
            MT_FLAG_INVALIDATE
        } else {
            0 as ::core::ffi::c_int
        })
        | (if decor_ext as ::core::ffi::c_int != 0 {
            MT_FLAG_DECOR_EXT
        } else {
            0 as ::core::ffi::c_int
        })) as uint16_t;
}
#[inline]
unsafe extern "C" fn mtpair_from(mut start: MTKey, mut end: MTKey) -> MTPair {
    return MTPair {
        start: start,
        end_pos: end.pos,
        end_right_gravity: mt_right(end),
    };
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
