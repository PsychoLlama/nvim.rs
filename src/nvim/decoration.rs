use crate::src::nvim::global_cell::GlobalCell;
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
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn mh_get_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> uint32_t;
    static mut namespace_localscope: Set_uint32_t;
    fn virt_text_to_array(vt: VirtText, hl_name: bool, arena: *mut Arena) -> Array;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn changed_lines_invalidate_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
    );
    static virt_text_pos_str: [*const ::core::ffi::c_char; 0];
    static hl_mode_str: [*const ::core::ffi::c_char; 0];
    static mut decor_state: DecorState;
    static mut decor_items: C2Rust_Unnamed_26;
    fn decor_providers_invoke_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int) -> bool;
    fn extmark_set(
        buf: *mut buf_T,
        ns_id: uint32_t,
        idp: *mut uint32_t,
        row: ::core::ffi::c_int,
        col: colnr_T,
        end_row: ::core::ffi::c_int,
        end_col: colnr_T,
        decor: DecorInline,
        decor_flags: uint16_t,
        right_gravity: bool,
        end_right_gravity: bool,
        no_undo: bool,
        invalidate: bool,
        err: *mut Error,
    );
    fn redraw_buf_line_later(buf: *mut buf_T, line: linenr_T, force: bool);
    fn redraw_buf_range_later(buf: *mut buf_T, first: linenr_T, last: linenr_T);
    fn conceal_cursor_line(wp: *const win_T) -> bool;
    fn hasAnyFolding(win: *mut win_T) -> ::core::ffi::c_int;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    fn schar_high(sc: schar_T) -> bool;
    fn schar_get(buf_out: *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn schar_get_first_codepoint(sc: schar_T) -> ::core::ffi::c_int;
    fn schar_from_char(c: ::core::ffi::c_int) -> schar_T;
    fn hl_add_url(attr: ::core::ffi::c_int, url: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn syn_id2name(id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn marktree_itr_get(
        b: *mut MarkTree,
        row: int32_t,
        col: ::core::ffi::c_int,
        itr: *mut MarkTreeIter,
    ) -> bool;
    fn marktree_itr_next(b: *mut MarkTree, itr: *mut MarkTreeIter) -> bool;
    fn marktree_itr_get_filter(
        b: *mut MarkTree,
        row: int32_t,
        col: ::core::ffi::c_int,
        stop_row: ::core::ffi::c_int,
        stop_col: ::core::ffi::c_int,
        meta_filter: MetaFilter,
        itr: *mut MarkTreeIter,
    ) -> bool;
    fn marktree_itr_step_out_filter(
        b: *mut MarkTree,
        itr: *mut MarkTreeIter,
        meta_filter: MetaFilter,
    ) -> bool;
    fn marktree_itr_next_filter(
        b: *mut MarkTree,
        itr: *mut MarkTreeIter,
        stop_row: ::core::ffi::c_int,
        stop_col: ::core::ffi::c_int,
        meta_filter: MetaFilter,
    ) -> bool;
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
    fn marktree_get_altpos(b: *mut MarkTree, mark: MTKey, itr: *mut MarkTreeIter) -> MTPos;
    fn changed_window_setting(wp: *mut win_T);
    fn buf_has_signs(buf: *const buf_T) -> bool;
    fn describe_sign_text(buf: *mut ::core::ffi::c_char, sign_text: *mut schar_T) -> size_t;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type OptInt = int64_t;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct undo_object {
    pub type_0: UndoObjectType,
    pub data: C2Rust_Unnamed_7,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
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
    pub data: C2Rust_Unnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_16 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_16 = 1;
pub type DecorPriorityInternal = uint32_t;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_17 = 128;
pub const kSHConceal: C2Rust_Unnamed_17 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_17 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_17 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_17 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_17 = 4;
pub const kSHHlEol: C2Rust_Unnamed_17 = 2;
pub const kSHIsSign: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_18 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_18 = 4;
pub const kVTHide: C2Rust_Unnamed_18 = 2;
pub const kVTIsLines: C2Rust_Unnamed_18 = 1;
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
    pub s: [C2Rust_Unnamed_19; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignItem {
    pub sh: *mut DecorSignHighlight,
    pub id: uint32_t,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_20 = 9;
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
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kDecorKindUIWatched: C2Rust_Unnamed_21 = 4;
pub const kDecorKindVirtLines: C2Rust_Unnamed_21 = 3;
pub const kDecorKindVirtText: C2Rust_Unnamed_21 = 2;
pub const kDecorKindSign: C2Rust_Unnamed_21 = 1;
pub const kDecorKindHighlight: C2Rust_Unnamed_21 = 0;
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
    pub data: C2Rust_Unnamed_22,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_23,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
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
    pub slots: C2Rust_Unnamed_25,
    pub ranges_i: C2Rust_Unnamed_24,
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
pub struct C2Rust_Unnamed_24 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_25 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_27 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SignItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub name: *mut ::core::ffi::c_char,
    pub val: ::core::ffi::c_int,
}
pub const kExtmarkHighlight: C2Rust_Unnamed_29 = 32;
pub const kExtmarkSign: C2Rust_Unnamed_29 = 2;
pub const kExtmarkNone: C2Rust_Unnamed_29 = 1;
pub const kExtmarkVirtText: C2Rust_Unnamed_29 = 8;
pub const kExtmarkVirtLines: C2Rust_Unnamed_29 = 16;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const kExtmarkSignHL: C2Rust_Unnamed_29 = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
pub const kMTFilterSelect: uint32_t = -1 as ::core::ffi::c_int as uint32_t;
#[inline]
unsafe extern "C" fn ns_in_win(mut ns_id: uint32_t, mut wp: *mut win_T) -> bool {
    if !set_has_uint32_t(&raw mut namespace_localscope, ns_id) {
        return true_0 != 0;
    }
    return set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id);
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
#[no_mangle]
pub static decor_freelist: GlobalCell<uint32_t> = GlobalCell::new(UINT32_MAX as uint32_t);
#[no_mangle]
pub static to_free_virt: GlobalCell<*mut DecorVirtText> =
    GlobalCell::new(::core::ptr::null_mut::<DecorVirtText>());
#[no_mangle]
pub static to_free_sh: GlobalCell<uint32_t> = GlobalCell::new(UINT32_MAX as uint32_t);
#[no_mangle]
pub unsafe extern "C" fn bufhl_add_hl_pos_offset(
    mut buf: *mut buf_T,
    mut src_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut pos_start: lpos_T,
    mut pos_end: lpos_T,
    mut offset: colnr_T,
) {
    let mut hl_start: colnr_T = 0 as colnr_T;
    let mut hl_end: colnr_T = 0 as colnr_T;
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    let mut lnum: linenr_T = pos_start.lnum;
    while lnum <= pos_end.lnum {
        let mut end_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pos_start.lnum < lnum && lnum < pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if lnum == pos_start.lnum && lnum < pos_end.lnum {
            hl_start = pos_start.col + offset;
            end_off = 1 as ::core::ffi::c_int;
            hl_end = 0 as ::core::ffi::c_int as colnr_T;
        } else if pos_start.lnum < lnum && lnum == pos_end.lnum {
            hl_start = (if offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
            {
                offset as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            hl_end = pos_end.col + offset;
        } else if pos_start.lnum == lnum && pos_end.lnum == lnum {
            hl_start = pos_start.col + offset;
            hl_end = pos_end.col + offset;
        }
        extmark_set(
            buf,
            src_id as uint32_t,
            ::core::ptr::null_mut::<uint32_t>(),
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            hl_start,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int + end_off,
            hl_end,
            decor,
            MT_FLAG_DECOR_HL as uint16_t,
            true_0 != 0,
            false_0 != 0,
            true_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<Error>(),
        );
        lnum += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
) {
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            let mut below: bool =
                (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                    && (*vt).flags as ::core::ffi::c_int & kVTLinesAbove as ::core::ffi::c_int == 0;
            let mut vt_lnum: linenr_T = row1 as linenr_T + 1 as linenr_T + below as linenr_T;
            redraw_buf_line_later(buf, vt_lnum, true_0 != 0);
            if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                || (*vt).pos as ::core::ffi::c_uint
                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut vt_col: colnr_T =
                    if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                        0 as colnr_T
                    } else {
                        col1 as colnr_T
                    };
                changed_lines_invalidate_buf(
                    buf,
                    vt_lnum,
                    vt_col,
                    vt_lnum + 1 as linenr_T,
                    0 as linenr_T,
                );
            }
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            decor_redraw_sh(buf, row1, row2, *sh);
            idx = (*sh).next;
        }
    } else {
        decor_redraw_sh(buf, row1, row2, decor_sh_from_inline(decor.data.hl));
    };
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: DecorSignHighlight,
) {
    if sh.hl_id != 0
        || !sh.url.is_null()
        || sh.flags as ::core::ffi::c_int
            & (kSHIsSign as ::core::ffi::c_int
                | kSHSpellOn as ::core::ffi::c_int
                | kSHSpellOff as ::core::ffi::c_int
                | kSHConceal as ::core::ffi::c_int)
            != 0
    {
        if row2 >= row1 {
            redraw_buf_range_later(
                buf,
                row1 as linenr_T + 1 as linenr_T,
                row2 as linenr_T + 1 as linenr_T,
            );
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                changed_window_setting(wp);
            }
            wp = (*wp).w_next;
        }
    }
    if sh.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        redraw_buf_line_later(buf, row1 as linenr_T + 1 as linenr_T, false_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_put_sh(mut item: DecorSignHighlight) -> uint32_t {
    if decor_freelist.get() != UINT32_MAX as uint32_t {
        let mut pos: uint32_t = decor_freelist.get();
        decor_freelist.set((*decor_items.items.offset(decor_freelist.get() as isize)).next);
        *decor_items.items.offset(pos as isize) = item;
        return pos;
    } else {
        let mut pos_0: uint32_t = decor_items.size as uint32_t;
        if decor_items.size == decor_items.capacity {
            decor_items.capacity = if decor_items.capacity != 0 {
                decor_items.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            decor_items.items = xrealloc(
                decor_items.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<DecorSignHighlight>().wrapping_mul(decor_items.capacity),
            ) as *mut DecorSignHighlight;
        } else {
        };
        let c2rust_fresh0 = decor_items.size;
        decor_items.size = decor_items.size.wrapping_add(1);
        *decor_items.items.offset(c2rust_fresh0 as isize) = item;
        return pos_0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn decor_put_vt(
    mut vt: DecorVirtText,
    mut next: *mut DecorVirtText,
) -> *mut DecorVirtText {
    let mut decor_alloc: *mut DecorVirtText =
        xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
    *decor_alloc = vt;
    (*decor_alloc).next = next;
    return decor_alloc;
}
#[no_mangle]
pub unsafe extern "C" fn decor_sh_from_inline(
    mut item: DecorHighlightInline,
) -> DecorSignHighlight {
    '_c2rust_label: {
        if item.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"!(item.flags & kSHIsSign)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                166 as ::core::ffi::c_uint,
                b"DecorSignHighlight decor_sh_from_inline(DecorHighlightInline)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut conv: DecorSignHighlight = DecorSignHighlight {
        flags: item.flags,
        priority: item.priority,
        hl_id: item.hl_id,
        text: [item.conceal_char, 0],
        sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sign_add_id: 0,
        number_hl_id: 0 as ::core::ffi::c_int,
        line_hl_id: 0 as ::core::ffi::c_int,
        cursorline_hl_id: 0 as ::core::ffi::c_int,
        next: DECOR_ID_INVALID as uint32_t,
        url: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    return conv;
}
#[no_mangle]
pub unsafe extern "C" fn buf_put_decor(
    mut buf: *mut buf_T,
    mut decor: DecorInline,
    mut row: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    if decor.ext as ::core::ffi::c_int != 0 && (row as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            buf_put_decor_sh(buf, sh, row, row2);
            idx = (*sh).next;
        }
    }
}
unsafe extern "C" fn may_force_numberwidth_recompute(mut buf: *mut buf_T, mut unplace: bool) {
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf
                && (*wp).w_minscwidth == SCL_NUM
                && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                && (unplace as ::core::ffi::c_int != 0
                    || (*wp).w_nrwidth_width < 2 as ::core::ffi::c_int)
            {
                (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
static sign_add_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn buf_put_decor_sh(
    mut buf: *mut buf_T,
    mut sh: *mut DecorSignHighlight,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        let c2rust_fresh1 = sign_add_id.get();
        sign_add_id.set(sign_add_id.get() + 1);
        (*sh).sign_add_id = c2rust_fresh1;
        if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
            buf_signcols_count_range(buf, row1, row2, 1 as ::core::ffi::c_int, kFalse);
            may_force_numberwidth_recompute(buf, false_0 != 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn buf_decor_remove(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut col1: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut free: bool,
) {
    decor_redraw(buf, row1, row2, col1, decor);
    if decor.ext as ::core::ffi::c_int != 0 && (row1 as linenr_T) < (*buf).b_ml.ml_line_count {
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        row2 = (if ((*buf).b_ml.ml_line_count - 1 as linenr_T) < row2 as linenr_T {
            (*buf).b_ml.ml_line_count - 1 as linenr_T
        } else {
            row2 as linenr_T
        }) as ::core::ffi::c_int;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            buf_remove_decor_sh(buf, row1, row2, sh);
            idx = (*sh).next;
        }
    }
    if free {
        decor_free(decor);
    }
}
#[no_mangle]
pub unsafe extern "C" fn buf_remove_decor_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
            if buf_meta_total(buf, kMTMetaSignText) != 0 {
                buf_signcols_count_range(buf, row1, row2, -1 as ::core::ffi::c_int, kFalse);
            } else {
                may_force_numberwidth_recompute(buf, true_0 != 0);
                (*buf).b_signcols.count[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
                (*buf).b_signcols.max = 0 as ::core::ffi::c_int;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_free(mut decor: DecorInline) {
    if !decor.ext {
        return;
    }
    let mut vt: *mut DecorVirtText = decor.data.ext.vt;
    let mut idx: uint32_t = decor.data.ext.sh_idx;
    if decor_state.running_decor_provider {
        while !vt.is_null() {
            if (*vt).next.is_null() {
                (*vt).next = to_free_virt.get();
                to_free_virt.set(decor.data.ext.vt);
                break;
            } else {
                vt = (*vt).next;
            }
        }
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            if (*sh).next == DECOR_ID_INVALID as uint32_t {
                (*sh).next = to_free_sh.get();
                to_free_sh.set(decor.data.ext.sh_idx);
                break;
            } else {
                idx = (*sh).next;
            }
        }
    } else {
        decor_free_inner(vt, idx);
    };
}
unsafe extern "C" fn decor_free_inner(mut vt: *mut DecorVirtText, mut first_idx: uint32_t) {
    while !vt.is_null() {
        if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
            clear_virtlines(&raw mut (*vt).data.virt_lines);
        } else {
            clear_virttext(&raw mut (*vt).data.virt_text);
        }
        let mut tofree: *mut DecorVirtText = vt;
        vt = (*vt).next;
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    let mut idx: uint32_t = first_idx;
    while idx != DECOR_ID_INVALID as uint32_t {
        let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).sign_name as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        }
        (*sh).flags = 0 as uint16_t;
        if !(*sh).url.is_null() {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*sh).url as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL;
            *ptr__0;
        }
        if (*sh).next == DECOR_ID_INVALID as uint32_t {
            (*sh).next = decor_freelist.get();
            decor_freelist.set(first_idx);
            break;
        } else {
            idx = (*sh).next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_state_invalidate(mut buf: *mut buf_T) {
    if !decor_state.win.is_null() && (*decor_state.win).w_buffer == buf {
        decor_state.itr_valid = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_check_to_be_deleted() {
    '_c2rust_label: {
        if !decor_state.running_decor_provider {
        } else {
            __assert_fail(
                b"!decor_state.running_decor_provider\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                330 as ::core::ffi::c_uint,
                b"void decor_check_to_be_deleted(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    decor_free_inner(to_free_virt.get(), to_free_sh.get());
    to_free_virt.set(::core::ptr::null_mut::<DecorVirtText>());
    to_free_sh.set(DECOR_ID_INVALID as uint32_t);
    decor_state.win = ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn decor_state_free(mut state: *mut DecorState) {
    xfree((*state).slots.items as *mut ::core::ffi::c_void);
    (*state).slots.capacity = 0 as size_t;
    (*state).slots.size = (*state).slots.capacity;
    (*state).slots.items = ::core::ptr::null_mut::<DecorRangeSlot>();
    xfree((*state).ranges_i.items as *mut ::core::ffi::c_void);
    (*state).ranges_i.capacity = 0 as size_t;
    (*state).ranges_i.size = (*state).ranges_i.capacity;
    (*state).ranges_i.items = ::core::ptr::null_mut::<::core::ffi::c_int>();
}
#[no_mangle]
pub unsafe extern "C" fn clear_virttext(mut text: *mut VirtText) {
    let mut i: size_t = 0 as size_t;
    while i < (*text).size {
        xfree((*(*text).items.offset(i as isize)).text as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree((*text).items as *mut ::core::ffi::c_void);
    (*text).capacity = 0 as size_t;
    (*text).size = (*text).capacity;
    (*text).items = ::core::ptr::null_mut::<VirtTextChunk>();
    *text = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
}
#[no_mangle]
pub unsafe extern "C" fn clear_virtlines(mut lines: *mut VirtLines) {
    let mut i: size_t = 0 as size_t;
    while i < (*lines).size {
        clear_virttext(&raw mut (*(*lines).items.offset(i as isize)).line);
        i = i.wrapping_add(1);
    }
    xfree((*lines).items as *mut ::core::ffi::c_void);
    (*lines).capacity = 0 as size_t;
    (*lines).size = (*lines).capacity;
    (*lines).items = ::core::ptr::null_mut::<virt_line>();
    *lines = VirtLines {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<virt_line>(),
    };
}
#[no_mangle]
pub unsafe extern "C" fn decor_check_invalid_glyphs() {
    let mut i: size_t = 0 as size_t;
    while i < decor_items.size {
        let mut it: *mut DecorSignHighlight = decor_items.items.offset(i as isize);
        let mut width: ::core::ffi::c_int =
            if (*it).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                SIGN_WIDTH as ::core::ffi::c_int
            } else if (*it).flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < width {
            if schar_high((*it).text[j as usize]) {
                (*it).text[j as usize] =
                    schar_from_char(schar_get_first_codepoint((*it).text[j as usize]));
            }
            j += 1;
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn next_virt_text_chunk(
    mut vt: VirtText,
    mut pos: *mut size_t,
    mut attr: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while text.is_null() && *pos < vt.size {
        text = (*vt.items.offset(*pos as isize)).text;
        let mut hl_id: ::core::ffi::c_int = (*vt.items.offset(*pos as isize)).hl_id;
        if hl_id >= 0 as ::core::ffi::c_int {
            *attr = if *attr > 0 as ::core::ffi::c_int {
                *attr
            } else {
                0 as ::core::ffi::c_int
            };
            if hl_id > 0 as ::core::ffi::c_int {
                *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
            }
        }
        *pos = (*pos).wrapping_add(1);
    }
    return text;
}
#[no_mangle]
pub unsafe extern "C" fn decor_find_virttext(
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut ns_id: uint64_t,
) -> *mut DecorVirtText {
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row as int32_t,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    let mut decor: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
            break;
        }
        if !mt_invalid(mark) {
            decor = mt_decor_virt(mark);
            while !decor.is_null()
                && (*decor).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
            {
                decor = (*decor).next;
            }
            if (ns_id == 0 as uint64_t || ns_id == mark.ns as uint64_t) && !decor.is_null() {
                return decor;
            }
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    return ::core::ptr::null_mut::<DecorVirtText>();
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_reset(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
) -> bool {
    (*state).row = -1 as ::core::ffi::c_int;
    (*state).win = wp;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let beg_pos: [::core::ffi::c_int; 2] = [0 as ::core::ffi::c_int, (*state).future_begin];
    let end_pos: [::core::ffi::c_int; 2] = [
        (*state).current_end,
        (*state).ranges_i.size as ::core::ffi::c_int,
    ];
    let mut pos_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while pos_i < 2 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = beg_pos[pos_i as usize];
        while i < end_pos[pos_i as usize] {
            let r: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
            if (*r).owned as ::core::ffi::c_int != 0
                && (*r).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
            {
                clear_virttext(&raw mut (*(*r).data.vt).data.virt_text);
                xfree((*r).data.vt as *mut ::core::ffi::c_void);
            }
            i += 1;
        }
        pos_i += 1;
    }
    (*state).slots.size = 0 as size_t;
    (*state).ranges_i.size = 0 as size_t;
    (*state).free_slot_i = -1 as ::core::ffi::c_int;
    (*state).current_end = 0 as ::core::ffi::c_int;
    (*state).future_begin = 0 as ::core::ffi::c_int;
    (*state).new_range_ordering = 0 as ::core::ffi::c_int;
    return (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys != 0;
}
#[no_mangle]
pub unsafe extern "C" fn decor_virt_pos(mut decor: *const DecorRange) -> bool {
    return (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
        || (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn decor_virt_pos_kind(mut decor: *const DecorRange) -> VirtTextPos {
    if (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
        return (*(*decor).data.vt).pos;
    }
    if (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int {
        return (*decor).data.ui.pos;
    }
    return kVPosEndOfLine;
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_start(
    mut wp: *mut win_T,
    mut top_row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) -> bool {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    (*state).top_row = top_row;
    (*state).itr_valid = true_0 != 0;
    if !marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        top_row,
        0 as ::core::ffi::c_int,
        &raw mut (*state).itr as *mut MarkTreeIter,
    ) {
        return false_0 != 0;
    }
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
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut (*state).itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        let mut m: MTKey = pair.start;
        if mt_invalid(m) as ::core::ffi::c_int != 0 || !mt_decor_any(m) {
            continue;
        }
        decor_range_add_from_inline(
            state,
            pair.start.pos.row as ::core::ffi::c_int,
            pair.start.pos.col as ::core::ffi::c_int,
            pair.end_pos.row as ::core::ffi::c_int,
            pair.end_pos.col as ::core::ffi::c_int,
            mt_decor(m),
            false_0 != 0,
            m.ns,
            m.id,
        );
    }
    return true_0 != 0;
}
unsafe extern "C" fn decor_state_pack(mut state: *mut DecorState) {
    let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let cur_end: ::core::ffi::c_int = (*state).current_end;
    let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
    if fut_beg == count {
        count = cur_end;
        fut_beg = count;
    } else if fut_beg != cur_end {
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        memmove(
            indices.offset(cur_end as isize) as *mut ::core::ffi::c_void,
            indices.offset(fut_beg as isize) as *const ::core::ffi::c_void,
            ((count - fut_beg) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        count = cur_end + (count - fut_beg);
        fut_beg = cur_end;
    }
    (*state).ranges_i.size = count as size_t;
    (*state).future_begin = fut_beg;
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) {
    decor_state_pack(state);
    if (*state).row == -1 as ::core::ffi::c_int {
        decor_redraw_start(wp, row, state);
    } else if !(*state).itr_valid {
        marktree_itr_get(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            row as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut (*state).itr as *mut MarkTreeIter,
        );
        (*state).itr_valid = true_0 != 0;
    }
    (*state).row = row;
    (*state).col_last = -1 as ::core::ffi::c_int;
    (*state).eol_col = -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn decor_has_more_decorations(
    mut state: *mut DecorState,
    mut row: ::core::ffi::c_int,
) -> bool {
    if (*state).current_end != 0 as ::core::ffi::c_int
        || (*state).future_begin != (*state).ranges_i.size as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    let mut k: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
    return k.pos.row >= 0 as int32_t && k.pos.row <= row as int32_t;
}
unsafe extern "C" fn decor_range_add_from_inline(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
) {
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            decor_range_add_virt(state, start_row, start_col, end_row, end_col, vt, owned);
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            decor_range_add_sh(
                state,
                start_row,
                start_col,
                end_row,
                end_col,
                sh,
                owned,
                ns,
                mark_id,
                0 as DecorPriority,
            );
            idx = (*sh).next;
        }
    } else {
        let mut sh_0: DecorSignHighlight = decor_sh_from_inline(decor.data.hl);
        decor_range_add_sh(
            state,
            start_row,
            start_col,
            end_row,
            end_col,
            &raw mut sh_0,
            owned,
            ns,
            mark_id,
            0 as DecorPriority,
        );
    };
}
unsafe extern "C" fn decor_range_insert(mut state: *mut DecorState, mut range: *mut DecorRange) {
    let c2rust_fresh2 = (*state).new_range_ordering;
    (*state).new_range_ordering = (*state).new_range_ordering + 1;
    (*range).ordering = c2rust_fresh2;
    let mut index: ::core::ffi::c_int = 0;
    if (*state).free_slot_i >= 0 as ::core::ffi::c_int {
        index = (*state).free_slot_i;
        let mut slot: *mut DecorRangeSlot = (*state).slots.items.offset(index as isize);
        (*state).free_slot_i = (*slot).next_free_i;
        (*slot).range = *range;
    } else {
        index = (*state).slots.size as ::core::ffi::c_int;
        if (*state).slots.size == (*state).slots.capacity {
            (*state).slots.capacity = if (*state).slots.capacity != 0 {
                (*state).slots.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*state).slots.items = xrealloc(
                (*state).slots.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<DecorRangeSlot>().wrapping_mul((*state).slots.capacity),
            ) as *mut DecorRangeSlot;
        } else {
        };
        let c2rust_fresh3 = (*state).slots.size;
        (*state).slots.size = (*state).slots.size.wrapping_add(1);
        (*(*state).slots.items.offset(c2rust_fresh3 as isize)).range = *range;
    }
    let row: ::core::ffi::c_int = (*range).start_row;
    let col: ::core::ffi::c_int = (*range).start_col;
    let count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut begin: ::core::ffi::c_int = (*state).future_begin;
    let mut end: ::core::ffi::c_int = count;
    while begin < end {
        let mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
        let mr: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(mid as isize) as isize)).range;
        let mrow: ::core::ffi::c_int = (*mr).start_row;
        let mcol: ::core::ffi::c_int = (*mr).start_col;
        if mrow < row || mrow == row && mcol <= col {
            begin = mid + 1 as ::core::ffi::c_int;
            if mrow == row && mcol == col {
                break;
            }
        } else {
            end = mid;
        }
    }
    if (*state).ranges_i.size == (*state).ranges_i.capacity {
        (*state).ranges_i.capacity = if (*state).ranges_i.capacity != 0 {
            (*state).ranges_i.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*state).ranges_i.items = xrealloc(
            (*state).ranges_i.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul((*state).ranges_i.capacity),
        ) as *mut ::core::ffi::c_int;
    } else {
    };
    (*state).ranges_i.size = (*state).ranges_i.size.wrapping_add(1);
    let item: *mut ::core::ffi::c_int = (*state).ranges_i.items.offset(begin as isize);
    memmove(
        item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        item as *const ::core::ffi::c_void,
        ((count - begin) as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
    );
    *item = index;
}
#[no_mangle]
pub unsafe extern "C" fn decor_range_add_virt(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut vt: *mut DecorVirtText,
    mut owned: bool,
) {
    let mut is_lines: bool =
        (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0;
    let mut range: DecorRange = DecorRange {
        start_row: start_row,
        start_col: start_col,
        end_row: end_row,
        end_col: end_col,
        ordering: 0,
        priority_internal: ((*vt).priority as DecorPriorityInternal) << 16 as ::core::ffi::c_int,
        owned: owned,
        kind: (if is_lines as ::core::ffi::c_int != 0 {
            kDecorKindVirtLines as ::core::ffi::c_int
        } else {
            kDecorKindVirtText as ::core::ffi::c_int
        }) as DecorRangeKind,
        data: C2Rust_Unnamed_22 { vt: vt },
        attr_id: 0 as ::core::ffi::c_int,
        draw_col: -10 as ::core::ffi::c_int,
    };
    decor_range_insert(state, &raw mut range);
}
#[no_mangle]
pub unsafe extern "C" fn decor_range_add_sh(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
    mut subpriority: DecorPriority,
) {
    if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        return;
    }
    let mut range: DecorRange = DecorRange {
        start_row: start_row,
        start_col: start_col,
        end_row: end_row,
        end_col: end_col,
        ordering: 0,
        priority_internal: (((*sh).priority as DecorPriorityInternal) << 16 as ::core::ffi::c_int)
            .wrapping_add(subpriority as DecorPriorityInternal),
        owned: owned,
        kind: kDecorKindHighlight as ::core::ffi::c_int as DecorRangeKind,
        data: C2Rust_Unnamed_22 { sh: *sh },
        attr_id: 0 as ::core::ffi::c_int,
        draw_col: -10 as ::core::ffi::c_int,
    };
    if (*sh).hl_id != 0
        || !(*sh).url.is_null()
        || (*sh).flags as ::core::ffi::c_int
            & (kSHConceal as ::core::ffi::c_int
                | kSHSpellOn as ::core::ffi::c_int
                | kSHSpellOff as ::core::ffi::c_int)
            != 0
    {
        if (*sh).hl_id != 0 {
            range.attr_id = syn_id2attr((*sh).hl_id);
        }
        decor_range_insert(state, &raw mut range);
    }
    if (*sh).flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        range.kind = kDecorKindUIWatched as ::core::ffi::c_int as DecorRangeKind;
        range.data.ui.ns_id = ns;
        range.data.ui.mark_id = mark_id;
        range.data.ui.pos = (if (*sh).flags as ::core::ffi::c_int
            & kSHUIWatchedOverlay as ::core::ffi::c_int
            != 0
        {
            kVPosOverlay as ::core::ffi::c_int
        } else {
            kVPosEndOfLine as ::core::ffi::c_int
        }) as VirtTextPos;
        decor_range_insert(state, &raw mut range);
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_init_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut item: *mut DecorRange,
) {
    let mut vt: *mut DecorVirtText =
        if (*item).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
            (*item).data.vt
        } else {
            ::core::ptr::null_mut::<DecorVirtText>()
        };
    let mut pos: VirtTextPos = decor_virt_pos_kind(item);
    if win_col < 0 as ::core::ffi::c_int
        && pos as ::core::ffi::c_uint != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*item).draw_col = win_col;
    } else if pos as ::core::ffi::c_uint
        == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*item).draw_col = if !vt.is_null()
            && (*vt).flags as ::core::ffi::c_int & kVTHide as ::core::ffi::c_int != 0
            && hidden as ::core::ffi::c_int != 0
        {
            INT_MIN
        } else {
            win_col
        };
    } else {
        (*item).draw_col = -1 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn decor_recheck_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
) {
    let end: ::core::ffi::c_int = (*state).current_end;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < end {
        let r: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
        if (*r).draw_col == -3 as ::core::ffi::c_int {
            decor_init_draw_col(win_col, hidden, r);
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_col_impl(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let buf: *mut buf_T = (*wp).w_buffer;
    let row: ::core::ffi::c_int = (*state).row;
    let mut col_last: ::core::ffi::c_int = max_col_last;
    let mut endpos: MTPos = MTPos { row: 0, col: 0 };
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
            break;
        }
        if mark.pos.row == row as int32_t && mark.pos.col > col as int32_t {
            col_last = (if (col_last as int32_t) < mark.pos.col - 1 as int32_t {
                col_last as int32_t
            } else {
                mark.pos.col - 1 as int32_t
            }) as ::core::ffi::c_int;
            break;
        } else {
            if !(mt_invalid(mark) as ::core::ffi::c_int != 0
                || mt_end(mark) as ::core::ffi::c_int != 0
                || !mt_decor_any(mark)
                || !ns_in_win(mark.ns, wp))
            {
                endpos = marktree_get_altpos(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    mark,
                    ::core::ptr::null_mut::<MarkTreeIter>(),
                );
                decor_range_add_from_inline(
                    state,
                    mark.pos.row as ::core::ffi::c_int,
                    mark.pos.col as ::core::ffi::c_int,
                    endpos.row as ::core::ffi::c_int,
                    endpos.col as ::core::ffi::c_int,
                    mt_decor(mark),
                    false_0 != 0,
                    mark.ns,
                    mark.id,
                );
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut (*state).itr as *mut MarkTreeIter,
            );
        }
    }
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
    let mut cur_end: ::core::ffi::c_int = (*state).current_end;
    let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
    while fut_beg < count {
        let index: ::core::ffi::c_int = *indices.offset(fut_beg as isize);
        let r: *mut DecorRange = &raw mut (*slots.offset(index as isize)).range;
        if (*r).start_row > row || (*r).start_row == row && (*r).start_col > col {
            break;
        }
        let ordering: ::core::ffi::c_int = (*r).ordering;
        let priority: DecorPriorityInternal = (*r).priority_internal;
        let mut begin: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut end: ::core::ffi::c_int = cur_end;
        while begin < end {
            let mut mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
            let mut mi: ::core::ffi::c_int = *indices.offset(mid as isize);
            let mut mr: *mut DecorRange = &raw mut (*slots.offset(mi as isize)).range;
            if (*mr).priority_internal < priority
                || (*mr).priority_internal == priority && (*mr).ordering < ordering
            {
                begin = mid + 1 as ::core::ffi::c_int;
            } else {
                end = mid;
            }
        }
        let item: *mut ::core::ffi::c_int = indices.offset(begin as isize);
        memmove(
            item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            item as *const ::core::ffi::c_void,
            ((cur_end - begin) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        *item = index;
        cur_end += 1;
        fut_beg += 1;
    }
    if fut_beg < count {
        let mut r_0: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(fut_beg as isize) as isize)).range;
        if (*r_0).start_row == row {
            col_last = if col_last < (*r_0).start_col - 1 as ::core::ffi::c_int {
                col_last
            } else {
                (*r_0).start_col - 1 as ::core::ffi::c_int
            };
        }
    }
    let mut new_cur_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal_char: schar_T = 0 as schar_T;
    let mut conceal_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut spell: TriState = kNone;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cur_end {
        let index_0: ::core::ffi::c_int = *indices.offset(i as isize);
        let slot: *mut DecorRangeSlot = slots.offset(index_0 as isize);
        let r_1: *mut DecorRange = &raw mut (*slot).range;
        let mut keep: bool = false;
        if (*r_1).end_row < row || (*r_1).end_row == row && (*r_1).end_col <= col {
            keep = (*r_1).start_row >= row && decor_virt_pos(r_1) as ::core::ffi::c_int != 0;
        } else {
            keep = true_0 != 0;
            if (*r_1).end_row == row && (*r_1).end_col > col {
                col_last = if col_last < (*r_1).end_col - 1 as ::core::ffi::c_int {
                    col_last
                } else {
                    (*r_1).end_col - 1 as ::core::ffi::c_int
                };
            }
            if (*r_1).attr_id > 0 as ::core::ffi::c_int {
                attr = hl_combine_attr(attr, (*r_1).attr_id);
            }
            if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
                && (*r_1).data.sh.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int
                    != 0
            {
                conceal = 1 as ::core::ffi::c_int;
                if (*r_1).start_row == row && (*r_1).start_col == col {
                    let mut sh: *mut DecorSignHighlight = &raw mut (*r_1).data.sh;
                    conceal = 2 as ::core::ffi::c_int;
                    conceal_char = (*sh).text[0 as ::core::ffi::c_int as usize];
                    col_last = if col_last < (*r_1).start_col {
                        col_last
                    } else {
                        (*r_1).start_col
                    };
                    conceal_attr = (*r_1).attr_id;
                }
            }
            if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int {
                if (*r_1).data.sh.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int
                    != 0
                {
                    spell = kTrue;
                } else if (*r_1).data.sh.flags as ::core::ffi::c_int
                    & kSHSpellOff as ::core::ffi::c_int
                    != 0
                {
                    spell = kFalse;
                }
                if !(*r_1).data.sh.url.is_null() {
                    attr = hl_add_url(attr, (*r_1).data.sh.url);
                }
            }
        }
        if (*r_1).start_row == row
            && (*r_1).start_col <= col
            && decor_virt_pos(r_1) as ::core::ffi::c_int != 0
            && (*r_1).draw_col == -10 as ::core::ffi::c_int
        {
            decor_init_draw_col(win_col, hidden, r_1);
        }
        if keep {
            let c2rust_fresh4 = new_cur_end;
            new_cur_end = new_cur_end + 1;
            *indices.offset(c2rust_fresh4 as isize) = index_0;
        } else {
            if (*r_1).owned {
                if (*r_1).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
                    clear_virttext(&raw mut (*(*r_1).data.vt).data.virt_text);
                    xfree((*r_1).data.vt as *mut ::core::ffi::c_void);
                } else if (*r_1).kind as ::core::ffi::c_int
                    == kDecorKindHighlight as ::core::ffi::c_int
                {
                    xfree((*r_1).data.sh.url as *mut ::core::ffi::c_void);
                }
            }
            let mut fi: *mut ::core::ffi::c_int = &raw mut (*state).free_slot_i;
            (*slot).next_free_i = *fi;
            *fi = index_0;
        }
        i += 1;
    }
    cur_end = new_cur_end;
    if fut_beg == count {
        count = cur_end;
        fut_beg = count;
    }
    (*state).ranges_i.size = count as size_t;
    (*state).future_begin = fut_beg;
    (*state).current_end = cur_end;
    (*state).col_last = col_last;
    (*state).current = attr;
    (*state).conceal = conceal;
    (*state).conceal_char = conceal_char;
    (*state).conceal_attr = conceal_attr;
    (*state).spell = spell;
    return attr;
}
static conceal_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, 0, kMTFilterSelect]);
#[no_mangle]
pub unsafe extern "C" fn decor_conceal_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut check_cursor: bool,
) -> bool {
    if row < 0 as ::core::ffi::c_int
        || (*wp).w_onebuf_opt.wo_cole < 2 as OptInt
        || !check_cursor
            && wp == curwin
            && row as linenr_T + 1 as linenr_T == (*wp).w_cursor.lnum
            && !conceal_cursor_line(wp)
    {
        return false_0 != 0;
    }
    if buf_meta_total((*wp).w_buffer, kMTMetaConcealLines) == 0 {
        return decor_providers_invoke_conceal_line(wp, row);
    }
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
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_itr_get_overlap(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        row,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if mt_conceal_lines(pair.start) as ::core::ffi::c_int != 0
            && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (conceal_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row > row as int32_t {
            break;
        }
        if mt_conceal_lines(mark) as ::core::ffi::c_int != 0
            && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
        {
            return true_0 != 0;
        }
        marktree_itr_next_filter(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (conceal_filter.ptr() as *const _) as MetaFilter,
        );
    }
    return decor_providers_invoke_conceal_line(wp, row);
}
#[no_mangle]
pub unsafe extern "C" fn win_lines_concealed(mut wp: *mut win_T) -> bool {
    return hasAnyFolding(wp) != 0 || (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt;
}
#[no_mangle]
pub unsafe extern "C" fn sign_item_cmp(
    mut p1: *const ::core::ffi::c_void,
    mut p2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const SignItem = p1 as *mut SignItem;
    let mut s2: *const SignItem = p2 as *mut SignItem;
    if (*(*s1).sh).priority as ::core::ffi::c_int != (*(*s2).sh).priority as ::core::ffi::c_int {
        return if ((*(*s1).sh).priority as ::core::ffi::c_int)
            < (*(*s2).sh).priority as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    if (*s1).id != (*s2).id {
        return if (*s1).id < (*s2).id {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    if (*(*s1).sh).sign_add_id != (*(*s2).sh).sign_add_id {
        return if (*(*s1).sh).sign_add_id < (*(*s2).sh).sign_add_id {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return 0 as ::core::ffi::c_int;
}
static sign_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, kMTFilterSelect, kMTFilterSelect, 0]);
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_signs(
    mut wp: *mut win_T,
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut sattrs: *mut SignTextAttrs,
    mut line_id: *mut ::core::ffi::c_int,
    mut cul_id: *mut ::core::ffi::c_int,
    mut num_id: *mut ::core::ffi::c_int,
) {
    if !buf_has_signs(buf) {
        return;
    }
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
    let mut num_text: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut signs: C2Rust_Unnamed_27 = C2Rust_Unnamed_27 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<SignItem>(),
    };
    marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if !mt_invalid(pair.start)
            && mt_decor_sign(pair.start) as ::core::ffi::c_int != 0
            && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
        {
            let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(pair.start));
            num_text += ((*sh).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                as ::core::ffi::c_int;
            if signs.size == signs.capacity {
                signs.capacity = if signs.capacity != 0 {
                    signs.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                signs.items = xrealloc(
                    signs.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                ) as *mut SignItem;
            } else {
            };
            let c2rust_fresh5 = signs.size;
            signs.size = signs.size.wrapping_add(1);
            *signs.items.offset(c2rust_fresh5 as isize) = SignItem {
                sh: sh,
                id: pair.start.id,
            };
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (sign_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row != row as int32_t {
            break;
        }
        if !mt_invalid(mark)
            && !mt_end(mark)
            && mt_decor_sign(mark) as ::core::ffi::c_int != 0
            && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
        {
            let mut sh_0: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
            num_text += ((*sh_0).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                as ::core::ffi::c_int;
            if signs.size == signs.capacity {
                signs.capacity = if signs.capacity != 0 {
                    signs.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                signs.items = xrealloc(
                    signs.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                ) as *mut SignItem;
            } else {
            };
            let c2rust_fresh6 = signs.size;
            signs.size = signs.size.wrapping_add(1);
            *signs.items.offset(c2rust_fresh6 as isize) = SignItem {
                sh: sh_0,
                id: mark.id,
            };
        }
        marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (sign_filter.ptr() as *const _) as MetaFilter,
        );
    }
    if signs.size != 0 {
        let mut width: ::core::ffi::c_int = if (*wp).w_minscwidth == SCL_NUM {
            1 as ::core::ffi::c_int
        } else {
            (*wp).w_scwidth
        };
        let mut len: ::core::ffi::c_int = if width < num_text { width } else { num_text };
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<SignItem>(),
            Some(
                sign_item_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut i: size_t = 0 as size_t;
        while i < signs.size {
            let mut sh_1: *mut DecorSignHighlight = (*signs.items.offset(i as isize)).sh;
            if !sattrs.is_null() && idx < len && (*sh_1).text[0 as ::core::ffi::c_int as usize] != 0
            {
                memcpy(
                    &raw mut (*sattrs.offset(idx as isize)).text as *mut schar_T
                        as *mut ::core::ffi::c_void,
                    &raw mut (*sh_1).text as *mut schar_T as *const ::core::ffi::c_void,
                    (SIGN_WIDTH as ::core::ffi::c_int as size_t)
                        .wrapping_mul(::core::mem::size_of::<sattr_T>()),
                );
                let c2rust_fresh7 = idx;
                idx = idx + 1;
                (*sattrs.offset(c2rust_fresh7 as isize)).hl_id = (*sh_1).hl_id;
            }
            if !num_id.is_null() && *num_id <= 0 as ::core::ffi::c_int {
                *num_id = (*sh_1).number_hl_id;
            }
            if !line_id.is_null() && *line_id <= 0 as ::core::ffi::c_int {
                *line_id = (*sh_1).line_hl_id;
            }
            if !cul_id.is_null() && *cul_id <= 0 as ::core::ffi::c_int {
                *cul_id = (*sh_1).cursorline_hl_id;
            }
            i = i.wrapping_add(1);
        }
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<SignItem>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_find_sign(mut decor: DecorInline) -> *mut DecorSignHighlight {
    if !decor.ext {
        return ::core::ptr::null_mut::<DecorSignHighlight>();
    }
    let mut decor_id: uint32_t = decor.data.ext.sh_idx;
    loop {
        if decor_id == DECOR_ID_INVALID as uint32_t {
            return ::core::ptr::null_mut::<DecorSignHighlight>();
        }
        let mut sh: *mut DecorSignHighlight = decor_items.items.offset(decor_id as isize);
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            return sh;
        }
        decor_id = (*sh).next;
    }
}
static signtext_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, kMTFilterSelect, 0]);
#[no_mangle]
pub unsafe extern "C" fn buf_signcols_count_range(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut add: ::core::ffi::c_int,
    mut clear: TriState,
) {
    if !(*buf).b_signcols.autom || row2 < row1 || buf_meta_total(buf, kMTMetaSignText) == 0 {
        return;
    }
    let mut count: *mut ::core::ffi::c_int = xcalloc(
        (row2 + 1 as ::core::ffi::c_int - row1) as size_t,
        ::core::mem::size_of::<::core::ffi::c_int>(),
    ) as *mut ::core::ffi::c_int;
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut pair: MTPair = MTPair {
        start: MTKey {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
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
    marktree_itr_get_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        row1,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while marktree_itr_step_overlap(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        &raw mut pair,
    ) {
        if pair.start.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && !mt_invalid(pair.start)
        {
            let mut i: ::core::ffi::c_int = row1;
            while i as int32_t
                <= (if (row2 as int32_t) < pair.end_pos.row {
                    row2 as int32_t
                } else {
                    pair.end_pos.row
                })
            {
                *count.offset((i - row1) as isize) += 1;
                i += 1;
            }
        }
    }
    marktree_itr_step_out_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        &raw mut itr as *mut MarkTreeIter,
        (signtext_filter.ptr() as *const _) as MetaFilter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row > row2 as int32_t {
            break;
        }
        if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
            && !mt_invalid(mark)
            && !mt_end(mark)
        {
            let mut end: MTPos = marktree_get_altpos(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                mark,
                ::core::ptr::null_mut::<MarkTreeIter>(),
            );
            let mut i_0: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
            while i_0 as int32_t
                <= (if (row2 as int32_t) < end.row {
                    row2 as int32_t
                } else {
                    end.row
                })
            {
                *count.offset((i_0 - row1) as isize) += 1;
                i_0 += 1;
            }
        }
        marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            row2 + 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (signtext_filter.ptr() as *const _) as MetaFilter,
        );
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < row2 + 1 as ::core::ffi::c_int - row1 {
        let mut prevwidth: ::core::ffi::c_int =
            if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) - add {
                SIGN_SHOW_MAX as ::core::ffi::c_int
            } else {
                *count.offset(i_1 as isize) - add
            };
        if clear as ::core::ffi::c_int != kNone as ::core::ffi::c_int
            && prevwidth > 0 as ::core::ffi::c_int
        {
            (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize] -= 1;
            '_c2rust_label: {
                if (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize]
                    >= 0 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"buf->b_signcols.count[prevwidth - 1] >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1078 as ::core::ffi::c_uint,
                        b"void buf_signcols_count_range(buf_T *, int, int, int, TriState)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        let mut width: ::core::ffi::c_int =
            if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) {
                SIGN_SHOW_MAX as ::core::ffi::c_int
            } else {
                *count.offset(i_1 as isize)
            };
        if clear as ::core::ffi::c_int != kTrue as ::core::ffi::c_int
            && width > 0 as ::core::ffi::c_int
        {
            (*buf).b_signcols.count[(width - 1 as ::core::ffi::c_int) as usize] += 1;
            if width > (*buf).b_signcols.max {
                (*buf).b_signcols.max = width;
            }
        }
        i_1 += 1;
    }
    xfree(count as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_end(mut state: *mut DecorState) {
    (*state).win = ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn decor_redraw_eol(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
    mut eol_attr: *mut ::core::ffi::c_int,
    mut eol_col: ::core::ffi::c_int,
) -> bool {
    decor_redraw_col(
        wp,
        MAXCOL as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
        false_0 != 0,
        state,
        MAXCOL as ::core::ffi::c_int,
    );
    (*state).eol_col = eol_col;
    let count: ::core::ffi::c_int = (*state).current_end;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut has_virt_pos: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        let mut r: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
        has_virt_pos = has_virt_pos as ::core::ffi::c_int
            | ((*r).start_row == (*state).row && decor_virt_pos(r) as ::core::ffi::c_int != 0)
                as ::core::ffi::c_int
            != 0;
        if (*r).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
            && (*r).data.sh.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int != 0
        {
            *eol_attr = hl_combine_attr(*eol_attr, (*r).attr_id);
        }
        i += 1;
    }
    return has_virt_pos;
}
static lines_filter: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, kMTFilterSelect, 0, 0, 0]);
#[no_mangle]
pub unsafe extern "C" fn decor_virt_lines(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut num_below: *mut ::core::ffi::c_int,
    mut lines: *mut VirtLines,
    mut apply_folds: bool,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if buf_meta_total(buf, kMTMetaLines) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }];
    if !marktree_itr_get_filter(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        if start_row - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            start_row as int32_t - 1 as int32_t
        } else {
            0 as int32_t
        },
        0 as ::core::ffi::c_int,
        end_row,
        0 as ::core::ffi::c_int,
        (lines_filter.ptr() as *const _) as MetaFilter,
        &raw mut itr as *mut MarkTreeIter,
    ) {
        return 0 as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if start_row >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"start_row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1138 as ::core::ffi::c_uint,
                b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        let mut vt: *mut DecorVirtText = mt_decor_virt(mark);
        if !mt_invalid(mark) && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0 {
            while !vt.is_null() {
                if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                    let mut above: bool = (*vt).flags as ::core::ffi::c_int
                        & kVTLinesAbove as ::core::ffi::c_int
                        != 0;
                    let mut mrow: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
                    let mut draw_row: ::core::ffi::c_int = mrow
                        + (if above as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        });
                    if draw_row >= start_row
                        && draw_row < end_row
                        && (!apply_folds
                            || !(hasFolding(
                                wp,
                                mrow as linenr_T + 1 as linenr_T,
                                ::core::ptr::null_mut::<linenr_T>(),
                                ::core::ptr::null_mut::<linenr_T>(),
                            ) as ::core::ffi::c_int
                                != 0
                                || decor_conceal_line(wp, mrow, false_0 != 0)
                                    as ::core::ffi::c_int
                                    != 0))
                    {
                        virt_lines += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                        if !lines.is_null() {
                            if (*vt).data.virt_lines.size > 0 as size_t {
                                if (*lines).capacity
                                    < (*lines).size.wrapping_add((*vt).data.virt_lines.size)
                                {
                                    (*lines).capacity =
                                        (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                                    (*lines).capacity = (*lines).capacity.wrapping_sub(1);
                                    (*lines).capacity |=
                                        (*lines).capacity >> 1 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 2 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 4 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 8 as ::core::ffi::c_int;
                                    (*lines).capacity |=
                                        (*lines).capacity >> 16 as ::core::ffi::c_int;
                                    (*lines).capacity = (*lines).capacity.wrapping_add(1);
                                    (*lines).capacity = (*lines).capacity;
                                    (*lines).items = xrealloc(
                                        (*lines).items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<virt_line>()
                                            .wrapping_mul((*lines).capacity),
                                    )
                                        as *mut virt_line;
                                }
                                '_c2rust_label_0: {
                                    if !(*lines).items.is_null() {
                                    } else {
                                        __assert_fail(
                                            b"(*lines).items\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/decoration.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1155 as ::core::ffi::c_uint,
                                            b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                memcpy(
                                    (*lines).items.offset((*lines).size as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*vt).data.virt_lines.items as *const ::core::ffi::c_void,
                                    ::core::mem::size_of::<virt_line>()
                                        .wrapping_mul((*vt).data.virt_lines.size),
                                );
                                (*lines).size =
                                    (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                            }
                        }
                        if !num_below.is_null() && !above {
                            *num_below += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                        }
                    }
                }
                vt = (*vt).next;
            }
        }
        if !marktree_itr_next_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            end_row,
            0 as ::core::ffi::c_int,
            (lines_filter.ptr() as *const _) as MetaFilter,
        ) {
            break;
        }
    }
    return virt_lines;
}
#[no_mangle]
pub unsafe extern "C" fn decor_to_dict_legacy(
    mut dict: *mut Dict,
    mut decor: DecorInline,
    mut hl_name: bool,
    mut arena: *mut Arena,
) {
    let mut sh_hl: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut sh_sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    let mut virt_lines: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
    let mut priority: int32_t = -1 as int32_t;
    if decor.ext {
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                virt_lines = vt;
            } else {
                virt_text = vt;
            }
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                sh_sign = *sh;
            } else {
                sh_hl = *sh;
            }
            idx = (*sh).next;
        }
    } else {
        sh_hl = decor_sh_from_inline(decor.data.hl);
    }
    if sh_hl.hl_id != 0 {
        let c2rust_fresh8 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"hl_group\0".as_ptr() as *const ::core::ffi::c_char),
            value: hl_group_name(sh_hl.hl_id, hl_name),
        };
        let c2rust_fresh9 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"hl_eol\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: sh_hl.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        priority = sh_hl.priority as int32_t;
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
        let mut buf: [::core::ffi::c_char; 32] = [0; 32];
        schar_get(
            &raw mut buf as *mut ::core::ffi::c_char,
            sh_hl.text[0 as ::core::ffi::c_int as usize],
        );
        let c2rust_fresh10 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"conceal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: arena_string(
                        arena,
                        cstr_as_string(&raw mut buf as *mut ::core::ffi::c_char),
                    ),
                },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
        let c2rust_fresh11 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"conceal_lines\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(b"\0".as_ptr() as *const ::core::ffi::c_char),
                },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int != 0 {
        let c2rust_fresh12 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: true },
            },
        };
    } else if sh_hl.flags as ::core::ffi::c_int & kSHSpellOff as ::core::ffi::c_int != 0 {
        let c2rust_fresh13 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: false },
            },
        };
    }
    if sh_hl.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
        let c2rust_fresh14 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"ui_watched\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 { boolean: true },
            },
        };
    }
    if !sh_hl.url.is_null() {
        let c2rust_fresh15 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"url\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(sh_hl.url),
                },
            },
        };
    }
    if !virt_text.is_null() {
        if (*virt_text).hl_mode != 0 {
            let c2rust_fresh16 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                key: cstr_as_string(b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(
                            *(&raw const hl_mode_str as *const *const ::core::ffi::c_char)
                                .offset((*virt_text).hl_mode as isize),
                        ),
                    },
                },
            };
        }
        let mut chunks: Array = virt_text_to_array((*virt_text).data.virt_text, hl_name, arena);
        let c2rust_fresh17 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: chunks },
            },
        };
        let c2rust_fresh18 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text_hide\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_text).flags as ::core::ffi::c_int
                        & kVTHide as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh19 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(
                b"virt_text_repeat_linebreak\0".as_ptr() as *const ::core::ffi::c_char
            ),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_text).flags as ::core::ffi::c_int
                        & kVTRepeatLinebreak as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        if (*virt_text).pos as ::core::ffi::c_uint
            == kVPosWinCol as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let c2rust_fresh20 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_text_win_col\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_14 {
                        integer: (*virt_text).col as Integer,
                    },
                },
            };
        }
        let c2rust_fresh21 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(
                        *(&raw const virt_text_pos_str as *const *const ::core::ffi::c_char)
                            .offset((*virt_text).pos as isize),
                    ),
                },
            },
        };
        priority = (*virt_text).priority as int32_t;
    }
    if !virt_lines.is_null() {
        let mut all_chunks: Array = arena_array(arena, (*virt_lines).data.virt_lines.size);
        let mut virt_lines_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < (*virt_lines).data.virt_lines.size {
            virt_lines_flags = (*(*virt_lines).data.virt_lines.items.offset(i as isize)).flags;
            let mut chunks_0: Array = virt_text_to_array(
                (*(*virt_lines).data.virt_lines.items.offset(i as isize)).line,
                hl_name,
                arena,
            );
            if all_chunks.size == all_chunks.capacity {
                all_chunks.capacity = if all_chunks.capacity != 0 {
                    all_chunks.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                all_chunks.items = xrealloc(
                    all_chunks.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(all_chunks.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh22 = all_chunks.size;
            all_chunks.size = all_chunks.size.wrapping_add(1);
            *all_chunks.items.offset(c2rust_fresh22 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: chunks_0 },
            };
            i = i.wrapping_add(1);
        }
        let c2rust_fresh23 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_14 { array: all_chunks },
            },
        };
        let c2rust_fresh24 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_above\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: (*virt_lines).flags as ::core::ffi::c_int
                        & kVTLinesAbove as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh25 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_leftcol\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_14 {
                    boolean: virt_lines_flags & kVLLeftcol as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh26 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(
                        if virt_lines_flags & kVLScroll as ::core::ffi::c_int != 0 {
                            b"scroll\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"trunc\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    ),
                },
            },
        };
        priority = (*virt_lines).priority as int32_t;
    }
    if sh_sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
        if sh_sign.text[0 as ::core::ffi::c_int as usize] != 0 {
            let mut buf_0: [::core::ffi::c_char; 64] = [0; 64];
            describe_sign_text(
                &raw mut buf_0 as *mut ::core::ffi::c_char,
                &raw mut sh_sign.text as *mut schar_T,
            );
            let c2rust_fresh27 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                key: cstr_as_string(b"sign_text\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: arena_string(
                            arena,
                            cstr_as_string(&raw mut buf_0 as *mut ::core::ffi::c_char),
                        ),
                    },
                },
            };
        }
        if !sh_sign.sign_name.is_null() {
            let c2rust_fresh28 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                key: cstr_as_string(b"sign_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(sh_sign.sign_name),
                    },
                },
            };
        }
        let mut hls: [C2Rust_Unnamed_28; 5] = [
            C2Rust_Unnamed_28 {
                name: b"sign_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"number_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.number_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"line_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.line_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: b"cursorline_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                val: sh_sign.cursorline_hl_id,
            },
            C2Rust_Unnamed_28 {
                name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                val: 0 as ::core::ffi::c_int,
            },
        ];
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !hls[j as usize].name.is_null() {
            if hls[j as usize].val != 0 {
                let c2rust_fresh29 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                    key: cstr_as_string(hls[j as usize].name),
                    value: hl_group_name(hls[j as usize].val, hl_name),
                };
            }
            j += 1;
        }
        priority = sh_sign.priority as int32_t;
    }
    if priority != -1 as int32_t {
        let c2rust_fresh30 = (*dict).size;
        (*dict).size = (*dict).size.wrapping_add(1);
        *(*dict).items.offset(c2rust_fresh30 as isize) = key_value_pair {
            key: cstr_as_string(b"priority\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_14 {
                    integer: priority as Integer,
                },
            },
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn decor_type_flags(mut decor: DecorInline) -> uint16_t {
    if decor.ext {
        let mut type_flags: uint16_t = kExtmarkNone as ::core::ffi::c_int as uint16_t;
        let mut vt: *mut DecorVirtText = decor.data.ext.vt;
        while !vt.is_null() {
            type_flags = (type_flags as ::core::ffi::c_int
                | if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                    kExtmarkVirtLines as ::core::ffi::c_int
                } else {
                    kExtmarkVirtText as ::core::ffi::c_int
                }) as uint16_t;
            vt = (*vt).next;
        }
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID as uint32_t {
            let mut sh: *mut DecorSignHighlight = decor_items.items.offset(idx as isize);
            type_flags = (type_flags as ::core::ffi::c_int
                | if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                    kExtmarkSign as ::core::ffi::c_int
                } else {
                    kExtmarkHighlight as ::core::ffi::c_int
                }) as uint16_t;
            idx = (*sh).next;
        }
        return type_flags;
    } else {
        return (if decor.data.hl.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0
        {
            kExtmarkSign as ::core::ffi::c_int
        } else {
            kExtmarkHighlight as ::core::ffi::c_int
        }) as uint16_t;
    };
}
#[no_mangle]
pub unsafe extern "C" fn hl_group_name(mut hl_id: ::core::ffi::c_int, mut hl_name: bool) -> Object {
    if hl_name {
        return object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: cstr_as_string(syn_id2name(hl_id)),
            },
        };
    } else {
        return object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_14 {
                integer: hl_id as Integer,
            },
        };
    };
}
#[inline(always)]
unsafe extern "C" fn decor_redraw_col(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if col <= (*state).col_last {
        return (*state).current;
    }
    return decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last);
}
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const MT_FLAG_END: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 13 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_EXT
    | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT
    | MT_FLAG_DECOR_SIGNHL
    | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;
#[inline]
unsafe extern "C" fn mt_end(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_END != 0;
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
unsafe extern "C" fn mt_decor_sign(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL) != 0;
}
#[inline]
unsafe extern "C" fn mt_conceal_lines(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_CONCEAL_LINES != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
#[inline]
unsafe extern "C" fn mt_decor_virt(mut mark: MTKey) -> *mut DecorVirtText {
    return if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0 {
        mark.decor_data.ext.vt
    } else {
        ::core::ptr::null_mut::<DecorVirtText>()
    };
}
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
