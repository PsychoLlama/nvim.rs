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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strnlen(__string: *const ::core::ffi::c_char, __maxlen: size_t) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn arabic_shape(
        c: ::core::ffi::c_int,
        c1p: *mut ::core::ffi::c_int,
        prev_c: ::core::ffi::c_int,
        prev_c1: ::core::ffi::c_int,
        next_c: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn mh_clear(h: *mut MapHash);
    fn mh_put_glyph(set: *mut Set_glyph, key: String_0, new: *mut MHPutStatus) -> uint32_t;
    fn decor_check_invalid_glyphs();
    fn next_virt_text_chunk(
        vt: VirtText,
        pos: *mut size_t,
        attr: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static mut firstwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut full_screen: bool;
    static mut exmode_active: bool;
    static mut default_grid: ScreenGrid;
    static mut resizing_screen: bool;
    static mut linebuf_char: *mut schar_T;
    static mut linebuf_attr: *mut sattr_T;
    static mut linebuf_vcol: *mut colnr_T;
    static mut linebuf_scratch: *mut ::core::ffi::c_char;
    static mut p_arshape: ::core::ffi::c_int;
    static mut rdb_flags: ::core::ffi::c_uint;
    static mut p_tbidi: ::core::ffi::c_int;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn hl_apply_winblend(winbl: ::core::ffi::c_int, attr: ::core::ffi::c_int)
        -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2cells_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptrlen2schar(
        p: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        firstc: *mut ::core::ffi::c_int,
    ) -> schar_T;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_cp_bounds(
        base: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> CharBoundsOff;
    fn ui_line(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        invalid_row: bool,
        startcol: ::core::ffi::c_int,
        endcol: ::core::ffi::c_int,
        clearcol: ::core::ffi::c_int,
        clearattr: ::core::ffi::c_int,
        wrap: bool,
    );
    fn ui_grid_cursor_goto(
        grid_handle: handle_T,
        new_row: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn ui_check_cursor_grid(grid_handle: handle_T);
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn check_chars_options() -> *const ::core::ffi::c_char;
}
pub type __time_t = ::core::ffi::c_long;
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type time_t = __time_t;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type sscratch_T = int32_t;
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
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_glyph {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_char,
}
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
pub type BorderTextType = ::core::ffi::c_uint;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const SLF_INC_VCOL: C2Rust_Unnamed_13 = 4;
pub const SLF_WRAP: C2Rust_Unnamed_13 = 2;
pub const SLF_RIGHTLEFT: C2Rust_Unnamed_13 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharBoundsOff {
    pub begin_off: int8_t,
    pub end_off: int8_t,
}
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_14 = 4;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_14 = 8;
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
pub const kOptRdbFlagFlush: C2Rust_Unnamed_14 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_14 = 16;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_14 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_14 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_glyph = Set_glyph {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut linebuf_size: size_t = 0 as size_t;
static mut glyph_cache: Set_glyph = SET_INIT;
#[no_mangle]
pub unsafe extern "C" fn grid_adjust(
    mut grid: *mut GridView,
    mut row_off: *mut ::core::ffi::c_int,
    mut col_off: *mut ::core::ffi::c_int,
) -> *mut ScreenGrid {
    *row_off += (*grid).row_offset;
    *col_off += (*grid).col_offset;
    return (*grid).target;
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_str(mut str: *const ::core::ffi::c_char) -> schar_T {
    if str.is_null() {
        return 0 as schar_T;
    }
    return schar_from_buf(str, strlen(str));
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_buf(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) -> schar_T {
    '_c2rust_label: {
        if len < 32 as size_t {
        } else {
            __assert_fail(
                b"len < MAX_SCHAR_SIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                85 as ::core::ffi::c_uint,
                b"schar_T schar_from_buf(const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if len <= 4 as size_t {
        let mut sc: schar_T = 0 as schar_T;
        memcpy(
            &raw mut sc as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            len,
        );
        return sc;
    } else {
        let mut str: String_0 = String_0 {
            data: buf as *mut ::core::ffi::c_char,
            size: len,
        };
        let mut status: MHPutStatus = kMHExisting;
        let mut idx: uint32_t = mh_put_glyph(&raw mut glyph_cache, str, &raw mut status);
        '_c2rust_label_0: {
            if idx < 0xffffff as uint32_t {
            } else {
                __assert_fail(
                    b"idx < 0xFFFFFF\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    95 as ::core::ffi::c_uint,
                    b"schar_T schar_from_buf(const char *, size_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return (0xff as schar_T).wrapping_add((idx as schar_T) << 8 as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn schar_cache_clear_if_full() -> bool {
    if glyph_cache.h.n_keys > ((1 as ::core::ffi::c_int) << 21 as ::core::ffi::c_int) as uint32_t {
        schar_cache_clear();
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn schar_cache_clear() {
    decor_check_invalid_glyphs();
    mh_clear(&raw mut glyph_cache.h);
    if !check_chars_options().is_null() {
        abort();
    }
}
#[no_mangle]
pub unsafe extern "C" fn schar_high(mut sc: schar_T) -> bool {
    return sc & 0xff as schar_T == 0xff as schar_T;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get(
    mut buf_out: *mut ::core::ffi::c_char,
    mut sc: schar_T,
) -> size_t {
    let mut len: size_t = schar_get_adv(&raw mut buf_out, sc);
    *buf_out = NUL as ::core::ffi::c_char;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_adv(
    mut buf_out: *mut *mut ::core::ffi::c_char,
    mut sc: schar_T,
) -> size_t {
    let mut len: size_t = 0;
    if schar_high(sc) {
        let mut idx: uint32_t = sc as uint32_t >> 8 as ::core::ffi::c_int;
        '_c2rust_label: {
            if idx < glyph_cache.h.n_keys {
            } else {
                __assert_fail(
                    b"idx < glyph_cache.h.n_keys\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    163 as ::core::ffi::c_uint,
                    b"size_t schar_get_adv(char **, schar_T)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        len = strlen(glyph_cache.keys.offset(idx as isize));
        memcpy(
            *buf_out as *mut ::core::ffi::c_void,
            glyph_cache.keys.offset(idx as isize) as *const ::core::ffi::c_void,
            len,
        );
    } else {
        len = strnlen(&raw mut sc as *mut ::core::ffi::c_char, 4 as size_t);
        memcpy(
            *buf_out as *mut ::core::ffi::c_void,
            &raw mut sc as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            len,
        );
    }
    *buf_out = (*buf_out).offset(len as isize);
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn schar_len(mut sc: schar_T) -> size_t {
    if schar_high(sc) {
        let mut idx: uint32_t = sc as uint32_t >> 8 as ::core::ffi::c_int;
        '_c2rust_label: {
            if idx < glyph_cache.h.n_keys {
            } else {
                __assert_fail(
                    b"idx < glyph_cache.h.n_keys\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    178 as ::core::ffi::c_uint,
                    b"size_t schar_len(schar_T)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return strlen(glyph_cache.keys.offset(idx as isize));
    } else {
        return strnlen(&raw mut sc as *mut ::core::ffi::c_char, 4 as size_t);
    };
}
#[no_mangle]
pub unsafe extern "C" fn schar_cells(mut sc: schar_T) -> ::core::ffi::c_int {
    if sc < 0x80 as schar_T {
        return 1 as ::core::ffi::c_int;
    }
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    return utf_ptr2cells(&raw mut sc_buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn schar_get_first_byte(mut sc: schar_T) -> ::core::ffi::c_char {
    '_c2rust_label: {
        if !(schar_high(sc) as ::core::ffi::c_int != 0
            && sc >> 8 as ::core::ffi::c_int >= glyph_cache.h.n_keys)
        {
        } else {
            __assert_fail(
                b"!(schar_high(sc) && schar_idx(sc) >= glyph_cache.h.n_keys)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                206 as ::core::ffi::c_uint,
                b"char schar_get_first_byte(schar_T)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return (if schar_high(sc) as ::core::ffi::c_int != 0 {
        *glyph_cache
            .keys
            .offset((sc >> 8 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
    } else {
        *(&raw mut sc as *mut ::core::ffi::c_char) as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_first_codepoint(mut sc: schar_T) -> ::core::ffi::c_int {
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    return utf_ptr2char(&raw mut sc_buf as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn schar_get_ascii(mut sc: schar_T) -> ::core::ffi::c_char {
    return (if sc < 0x80 as schar_T {
        sc as ::core::ffi::c_char as ::core::ffi::c_int
    } else {
        NUL
    }) as ::core::ffi::c_char;
}
unsafe extern "C" fn schar_in_arabic_block(mut sc: schar_T) -> bool {
    let mut first_byte: ::core::ffi::c_char = schar_get_first_byte(sc);
    return first_byte as uint8_t as ::core::ffi::c_int & 0xfe as ::core::ffi::c_int
        == 0xd8 as ::core::ffi::c_int;
}
unsafe extern "C" fn schar_get_first_two_codepoints(
    mut sc: schar_T,
    mut c0: *mut ::core::ffi::c_int,
    mut c1: *mut ::core::ffi::c_int,
) {
    let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut sc_buf as *mut ::core::ffi::c_char, sc);
    *c0 = utf_ptr2char(&raw mut sc_buf as *mut ::core::ffi::c_char);
    let mut len: ::core::ffi::c_int = utf_ptr2len(&raw mut sc_buf as *mut ::core::ffi::c_char);
    if *c0 == NUL {
        *c1 = NUL;
    } else {
        *c1 = utf_ptr2char((&raw mut sc_buf as *mut ::core::ffi::c_char).offset(len as isize));
    };
}
#[no_mangle]
pub unsafe extern "C" fn line_do_arabic_shape(mut buf: *mut schar_T, mut cols: ::core::ffi::c_int) {
    let mut c1new: ::core::ffi::c_int = 0;
    let mut c0new: ::core::ffi::c_int = 0;
    let mut scbuf: [::core::ffi::c_char; 32] = [0; 32];
    let mut scbuf_new: [::core::ffi::c_char; 32] = [0; 32];
    let mut len: size_t = 0;
    let mut off: ::core::ffi::c_int = 0;
    let mut rest: size_t = 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < cols {
        if schar_in_arabic_block(*buf.offset(i as isize)) {
            break;
        }
        i += 1;
    }
    if i == cols {
        return;
    }
    let mut c0prev: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut c0: ::core::ffi::c_int = 0;
    let mut c1: ::core::ffi::c_int = 0;
    schar_get_first_two_codepoints(*buf.offset(i as isize), &raw mut c0, &raw mut c1);
    while i < cols {
        let mut c0next: ::core::ffi::c_int = 0;
        let mut c1next: ::core::ffi::c_int = 0;
        schar_get_first_two_codepoints(
            if (i + 1 as ::core::ffi::c_int) < cols {
                *buf.offset((i + 1 as ::core::ffi::c_int) as isize)
            } else {
                0 as schar_T
            },
            &raw mut c0next,
            &raw mut c1next,
        );
        if c0 & 0xff00 as ::core::ffi::c_int == 0x600 as ::core::ffi::c_int {
            c1new = c1;
            c0new = arabic_shape(c0, &raw mut c1new, c0next, c1next, c0prev);
            if !(c0new == c0 && c1new == c1) {
                scbuf = [0; 32];
                schar_get(
                    &raw mut scbuf as *mut ::core::ffi::c_char,
                    *buf.offset(i as isize),
                );
                scbuf_new = [0; 32];
                len =
                    utf_char2bytes(c0new, &raw mut scbuf_new as *mut ::core::ffi::c_char) as size_t;
                if c1new != 0 {
                    len = len.wrapping_add(utf_char2bytes(
                        c1new,
                        (&raw mut scbuf_new as *mut ::core::ffi::c_char).offset(len as isize),
                    ) as size_t);
                }
                off = utf_char2len(c0)
                    + (if c1 != 0 {
                        utf_char2len(c1)
                    } else {
                        0 as ::core::ffi::c_int
                    });
                rest = strlen((&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize));
                if rest.wrapping_add(len).wrapping_add(1 as size_t) > MAX_SCHAR_SIZE as size_t {
                    rest = rest.wrapping_sub(
                        (utf_cp_bounds(
                            (&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize),
                            (&raw mut scbuf as *mut ::core::ffi::c_char)
                                .offset(off as isize)
                                .offset(rest as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize)),
                        )
                        .begin_off as size_t)
                            .wrapping_add(1 as size_t),
                    );
                }
                memcpy(
                    (&raw mut scbuf_new as *mut ::core::ffi::c_char).offset(len as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut scbuf as *mut ::core::ffi::c_char).offset(off as isize)
                        as *const ::core::ffi::c_void,
                    rest,
                );
                *buf.offset(i as isize) = schar_from_buf(
                    &raw mut scbuf_new as *mut ::core::ffi::c_char,
                    len.wrapping_add(rest),
                );
            }
        }
        c0prev = c0;
        c0 = c0next;
        c1 = c1next;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear_line(
    mut grid: *mut ScreenGrid,
    mut off: size_t,
    mut width: ::core::ffi::c_int,
    mut valid: bool,
) {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < width {
        *(*grid)
            .chars
            .offset(off.wrapping_add(col as size_t) as isize) =
            ' ' as ::core::ffi::c_int as schar_T;
        col += 1;
    }
    let mut fill: ::core::ffi::c_int = if valid as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    memset(
        (*grid).attrs.offset(off as isize) as *mut ::core::ffi::c_void,
        fill,
        (width as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    memset(
        (*grid).vcols.offset(off as isize) as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        (width as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_invalidate(mut grid: *mut ScreenGrid) {
    memset(
        (*grid).attrs as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        ::core::mem::size_of::<sattr_T>()
            .wrapping_mul((*grid).rows as size_t)
            .wrapping_mul((*grid).cols as size_t),
    );
}
unsafe extern "C" fn grid_invalid_row(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
) -> bool {
    return *(*grid)
        .attrs
        .offset(*(*grid).line_offset.offset(row as isize) as isize)
        < 0 as sattr_T;
}
#[no_mangle]
pub unsafe extern "C" fn grid_getchar(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut attrp: *mut ::core::ffi::c_int,
) -> schar_T {
    if (*grid).chars.is_null() || row >= (*grid).rows || col >= (*grid).cols {
        return NUL as schar_T;
    }
    let mut off: size_t = (*(*grid).line_offset.offset(row as isize)).wrapping_add(col as size_t);
    if !attrp.is_null() {
        *attrp = *(*grid).attrs.offset(off as isize) as ::core::ffi::c_int;
    }
    return *(*grid).chars.offset(off as isize);
}
static mut grid_line_grid: *mut ScreenGrid = ::core::ptr::null_mut::<ScreenGrid>();
static mut grid_line_row: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
static mut grid_line_coloff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_maxcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_first: ::core::ffi::c_int = INT_MAX;
static mut grid_line_last: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_clear_to: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_bg_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_clear_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut grid_line_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn grid_line_start(mut view: *mut GridView, mut row: ::core::ffi::c_int) {
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid = grid_adjust(view, &raw mut row, &raw mut col);
    screengrid_line_start(grid, row, col);
}
#[no_mangle]
pub unsafe extern "C" fn screengrid_line_start(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    grid_line_maxcol = (*grid).cols;
    '_c2rust_label: {
        if grid_line_grid.is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                373 as ::core::ffi::c_uint,
                b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    grid_line_row = row;
    grid_line_grid = grid;
    grid_line_coloff = col;
    grid_line_first = linebuf_size as ::core::ffi::c_int;
    grid_line_maxcol = if grid_line_maxcol < (*grid).cols - grid_line_coloff {
        grid_line_maxcol
    } else {
        (*grid).cols - grid_line_coloff
    };
    grid_line_last = 0 as ::core::ffi::c_int;
    grid_line_clear_to = 0 as ::core::ffi::c_int;
    grid_line_bg_attr = 0 as ::core::ffi::c_int;
    grid_line_clear_attr = 0 as ::core::ffi::c_int;
    grid_line_flags = 0 as ::core::ffi::c_int;
    '_c2rust_label_0: {
        if grid_line_maxcol as size_t <= linebuf_size {
        } else {
            __assert_fail(
                b"(size_t)grid_line_maxcol <= linebuf_size\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                385 as ::core::ffi::c_uint,
                b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if full_screen as ::core::ffi::c_int != 0
        && rdb_flags & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        '_c2rust_label_1: {
            if !linebuf_char.is_null() {
            } else {
                __assert_fail(
                    b"linebuf_char\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    388 as ::core::ffi::c_uint,
                    b"void screengrid_line_start(ScreenGrid *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memset(
            linebuf_char as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<schar_T>().wrapping_mul(linebuf_size),
        );
        memset(
            linebuf_attr as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<sattr_T>().wrapping_mul(linebuf_size),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_getchar(
    mut col: ::core::ffi::c_int,
    mut attr: *mut ::core::ffi::c_int,
) -> schar_T {
    if col < grid_line_maxcol {
        col += grid_line_coloff;
        let mut off: size_t = (*(*grid_line_grid).line_offset.offset(grid_line_row as isize))
            .wrapping_add(col as size_t);
        if !attr.is_null() {
            *attr = *(*grid_line_grid).attrs.offset(off as isize) as ::core::ffi::c_int;
        }
        return *(*grid_line_grid).chars.offset(off as isize);
    } else {
        return ' ' as ::core::ffi::c_int as schar_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_put_schar(
    mut col: ::core::ffi::c_int,
    mut schar: schar_T,
    mut attr: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if !grid_line_grid.is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                418 as ::core::ffi::c_uint,
                b"void grid_line_put_schar(int, schar_T, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if col >= grid_line_maxcol {
        return;
    }
    *linebuf_char.offset(col as isize) = schar;
    *linebuf_attr.offset(col as isize) = attr as sattr_T;
    grid_line_first = if grid_line_first < col {
        grid_line_first
    } else {
        col
    };
    grid_line_last = if grid_line_last > col + 1 as ::core::ffi::c_int {
        grid_line_last
    } else {
        col + 1 as ::core::ffi::c_int
    };
    *linebuf_vcol.offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_puts(
    mut col: ::core::ffi::c_int,
    mut text: *const ::core::ffi::c_char,
    mut textlen: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ptr: *const ::core::ffi::c_char = text;
    let mut len: ::core::ffi::c_int = textlen;
    '_c2rust_label: {
        if !grid_line_grid.is_null() {
        } else {
            __assert_fail(
                b"grid_line_grid\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                444 as ::core::ffi::c_uint,
                b"int grid_line_puts(int, const char *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut start_col: ::core::ffi::c_int = col;
    let max_col: ::core::ffi::c_int = grid_line_maxcol;
    while col < max_col
        && (len < 0 as ::core::ffi::c_int || (ptr.offset_from(text) as ::core::ffi::c_int) < len)
        && *ptr as ::core::ffi::c_int != NUL
    {
        let mut mbyte_blen: ::core::ffi::c_int = 0;
        if len >= 0 as ::core::ffi::c_int {
            let mut maxlen: ::core::ffi::c_int =
                text.offset(len as isize).offset_from(ptr) as ::core::ffi::c_int;
            mbyte_blen = utfc_ptr2len_len(ptr, maxlen);
            if mbyte_blen > maxlen {
                mbyte_blen = 1 as ::core::ffi::c_int;
            }
        } else {
            mbyte_blen = utfc_ptr2len(ptr);
        }
        let mut firstc: ::core::ffi::c_int = 0;
        let mut schar: schar_T = utfc_ptrlen2schar(ptr, mbyte_blen, &raw mut firstc);
        let mut mbyte_cells: ::core::ffi::c_int = utf_ptr2cells_len(ptr, mbyte_blen);
        if mbyte_cells > 2 as ::core::ffi::c_int || schar == 0 as schar_T {
            mbyte_cells = 1 as ::core::ffi::c_int;
            schar = schar_from_char(0xfffd as ::core::ffi::c_int);
        }
        if col + mbyte_cells > max_col {
            schar = '>' as ::core::ffi::c_int as schar_T;
            mbyte_cells = 1 as ::core::ffi::c_int;
        }
        if ptr == text
            && col > grid_line_first
            && col < grid_line_last
            && *linebuf_char.offset(col as isize) == 0 as schar_T
        {
            *linebuf_char.offset((col - 1 as ::core::ffi::c_int) as isize) =
                '>' as ::core::ffi::c_int as schar_T;
        }
        *linebuf_char.offset(col as isize) = schar;
        *linebuf_attr.offset(col as isize) = attr as sattr_T;
        *linebuf_vcol.offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
        if mbyte_cells == 2 as ::core::ffi::c_int {
            *linebuf_char.offset((col + 1 as ::core::ffi::c_int) as isize) = 0 as schar_T;
            *linebuf_attr.offset((col + 1 as ::core::ffi::c_int) as isize) = attr as sattr_T;
            *linebuf_vcol.offset((col + 1 as ::core::ffi::c_int) as isize) =
                -1 as ::core::ffi::c_int as colnr_T;
        }
        col += mbyte_cells;
        ptr = ptr.offset(mbyte_blen as isize);
    }
    if col > start_col {
        grid_line_first = if grid_line_first < start_col {
            grid_line_first
        } else {
            start_col
        };
        grid_line_last = if grid_line_last > col {
            grid_line_last
        } else {
            col
        };
    }
    return col - start_col;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_fill(
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut sc: schar_T,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    end_col = if end_col < grid_line_maxcol {
        end_col
    } else {
        grid_line_maxcol
    };
    if start_col >= end_col {
        return end_col;
    }
    let mut col: ::core::ffi::c_int = start_col;
    while col < end_col {
        *linebuf_char.offset(col as isize) = sc;
        *linebuf_attr.offset(col as isize) = attr as sattr_T;
        *linebuf_vcol.offset(col as isize) = -1 as ::core::ffi::c_int as colnr_T;
        col += 1;
    }
    grid_line_first = if grid_line_first < start_col {
        grid_line_first
    } else {
        start_col
    };
    grid_line_last = if grid_line_last > end_col {
        grid_line_last
    } else {
        end_col
    };
    return end_col;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_clear_end(
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut bg_attr: ::core::ffi::c_int,
    mut clear_attr: ::core::ffi::c_int,
) {
    if grid_line_first > start_col {
        grid_line_first = start_col;
        grid_line_last = start_col;
    }
    grid_line_clear_to = end_col;
    grid_line_bg_attr = bg_attr;
    grid_line_clear_attr = clear_attr;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_cursor_goto(mut col: ::core::ffi::c_int) {
    ui_grid_cursor_goto((*grid_line_grid).handle, grid_line_row, col);
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_mirror(mut width: ::core::ffi::c_int) {
    grid_line_clear_to = if grid_line_last > grid_line_clear_to {
        grid_line_last
    } else {
        grid_line_clear_to
    };
    if grid_line_first >= grid_line_clear_to {
        return;
    }
    linebuf_mirror(
        &raw mut grid_line_first,
        &raw mut grid_line_last,
        &raw mut grid_line_clear_to,
        width,
    );
    grid_line_flags |= SLF_RIGHTLEFT as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn linebuf_mirror(
    mut firstp: *mut ::core::ffi::c_int,
    mut lastp: *mut ::core::ffi::c_int,
    mut clearp: *mut ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut first: ::core::ffi::c_int = *firstp;
    let mut last: ::core::ffi::c_int = *lastp;
    let mut n: size_t = (last - first) as size_t;
    let mut mirror: ::core::ffi::c_int = width - 1 as ::core::ffi::c_int;
    let mut scratch_char: *mut schar_T = linebuf_scratch as *mut schar_T;
    memcpy(
        scratch_char.offset(first as isize) as *mut ::core::ffi::c_void,
        linebuf_char.offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    let mut col: ::core::ffi::c_int = first;
    while col < last {
        let mut rev: ::core::ffi::c_int = mirror - col;
        if (col + 1 as ::core::ffi::c_int) < last
            && *scratch_char.offset((col + 1 as ::core::ffi::c_int) as isize) == 0 as schar_T
        {
            *linebuf_char.offset((rev - 1 as ::core::ffi::c_int) as isize) =
                *scratch_char.offset(col as isize);
            *linebuf_char.offset(rev as isize) = 0 as schar_T;
            col += 1;
        } else {
            *linebuf_char.offset(rev as isize) = *scratch_char.offset(col as isize);
        }
        col += 1;
    }
    let mut scratch_attr: *mut sattr_T = linebuf_scratch as *mut sattr_T;
    memcpy(
        scratch_attr.offset(first as isize) as *mut ::core::ffi::c_void,
        linebuf_attr.offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    let mut col_0: ::core::ffi::c_int = first;
    while col_0 < last {
        *linebuf_attr.offset((mirror - col_0) as isize) = *scratch_attr.offset(col_0 as isize);
        col_0 += 1;
    }
    let mut scratch_vcol: *mut colnr_T = linebuf_scratch as *mut colnr_T;
    memcpy(
        scratch_vcol.offset(first as isize) as *mut ::core::ffi::c_void,
        linebuf_vcol.offset(first as isize) as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
    let mut col_1: ::core::ffi::c_int = first;
    while col_1 < last {
        *linebuf_vcol.offset((mirror - col_1) as isize) = *scratch_vcol.offset(col_1 as isize);
        col_1 += 1;
    }
    *firstp = width - *clearp;
    *clearp = width - first;
    *lastp = width - last;
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_flush() {
    let mut grid: *mut ScreenGrid = grid_line_grid;
    grid_line_grid = ::core::ptr::null_mut::<ScreenGrid>();
    grid_line_clear_to = if grid_line_last > grid_line_clear_to {
        grid_line_last
    } else {
        grid_line_clear_to
    };
    '_c2rust_label: {
        if grid_line_clear_to <= grid_line_maxcol {
        } else {
            __assert_fail(
                b"grid_line_clear_to <= grid_line_maxcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                595 as ::core::ffi::c_uint,
                b"void grid_line_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if grid_line_first >= grid_line_clear_to {
        return;
    }
    grid_put_linebuf(
        grid,
        grid_line_row,
        grid_line_coloff,
        grid_line_first,
        grid_line_last,
        grid_line_clear_to,
        grid_line_bg_attr,
        grid_line_clear_attr,
        -1 as colnr_T,
        grid_line_flags,
    );
}
#[no_mangle]
pub unsafe extern "C" fn grid_line_flush_if_valid_row() {
    if grid_line_row < 0 as ::core::ffi::c_int || grid_line_row >= (*grid_line_grid).rows {
        if rdb_flags & kOptRdbFlagInvalid as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            abort();
        } else {
            grid_line_grid = ::core::ptr::null_mut::<ScreenGrid>();
            return;
        }
    }
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn grid_clear(
    mut grid: *mut GridView,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) {
    let mut row: ::core::ffi::c_int = start_row;
    while row < end_row {
        grid_line_start(grid, row);
        end_col = if end_col < grid_line_maxcol {
            end_col
        } else {
            grid_line_maxcol
        };
        if grid_line_row >= (*grid_line_grid).rows || start_col >= end_col {
            grid_line_grid = ::core::ptr::null_mut::<ScreenGrid>();
            return;
        }
        grid_line_clear_end(start_col, end_col, attr, 0 as ::core::ffi::c_int);
        grid_line_flush();
        row += 1;
    }
}
unsafe extern "C" fn grid_char_needs_redraw(
    mut grid: *mut ScreenGrid,
    mut col: ::core::ffi::c_int,
    mut off_to: size_t,
    mut cols: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return (cols > 0 as ::core::ffi::c_int
        && (*linebuf_char.offset(col as isize) != *(*grid).chars.offset(off_to as isize)
            || *linebuf_attr.offset(col as isize) != *(*grid).attrs.offset(off_to as isize)
            || cols > 1 as ::core::ffi::c_int
                && *linebuf_char.offset((col + 1 as ::core::ffi::c_int) as isize) == 0 as schar_T
                && *linebuf_char.offset((col + 1 as ::core::ffi::c_int) as isize)
                    != *(*grid)
                        .chars
                        .offset(off_to.wrapping_add(1 as size_t) as isize)
            || exmode_active as ::core::ffi::c_int != 0
            || rdb_flags & kOptRdbFlagNodelta as ::core::ffi::c_int as ::core::ffi::c_uint != 0))
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn grid_put_linebuf(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut coloff: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut endcol: ::core::ffi::c_int,
    mut clear_width: ::core::ffi::c_int,
    mut bg_attr: ::core::ffi::c_int,
    mut clear_attr: ::core::ffi::c_int,
    mut last_vcol: colnr_T,
    mut flags: ::core::ffi::c_int,
) {
    let mut redraw_next: bool = false;
    let mut clear_next: bool = false_0 != 0;
    '_c2rust_label: {
        if 0 as ::core::ffi::c_int <= row && row < (*grid).rows {
        } else {
            __assert_fail(
                b"0 <= row && row < grid->rows\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                672 as ::core::ffi::c_uint,
                b"void grid_put_linebuf(ScreenGrid *, int, int, int, int, int, int, int, colnr_T, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if endcol > (*grid).cols {
        endcol = (*grid).cols;
    }
    if (*grid).chars.is_null() || row >= (*grid).rows || coloff >= (*grid).cols {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"grid_put_linebuf\0".as_ptr() as *const ::core::ffi::c_char,
            681 as ::core::ffi::c_int,
            true_0 != 0,
            b"invalid state, skipped\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut invalid_row: bool = grid != &raw mut default_grid
        && grid_invalid_row(grid, row) as ::core::ffi::c_int != 0
        && col == 0 as ::core::ffi::c_int;
    let mut off_to: size_t =
        (*(*grid).line_offset.offset(row as isize)).wrapping_add(coloff as size_t);
    let max_off_to: size_t =
        (*(*grid).line_offset.offset(row as isize)).wrapping_add((*grid).cols as size_t);
    if col > 0 as ::core::ffi::c_int
        && *(*grid)
            .chars
            .offset(off_to.wrapping_add(col as size_t) as isize)
            == 0 as schar_T
    {
        *linebuf_char.offset((col - 1 as ::core::ffi::c_int) as isize) =
            '>' as ::core::ffi::c_int as schar_T;
        *linebuf_attr.offset((col - 1 as ::core::ffi::c_int) as isize) = *(*grid)
            .attrs
            .offset(off_to.wrapping_add(col as size_t).wrapping_sub(1 as size_t) as isize);
        col -= 1;
    }
    let mut clear_start: ::core::ffi::c_int = endcol;
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0 {
        clear_start = col;
        col = endcol;
        endcol = clear_width;
        clear_width = col;
    }
    if p_arshape != 0 && p_tbidi == 0 && endcol > col {
        line_do_arabic_shape(linebuf_char.offset(col as isize), endcol - col);
    }
    if bg_attr != 0 {
        let mut c: ::core::ffi::c_int = col;
        while c < endcol {
            *linebuf_attr.offset(c as isize) = hl_combine_attr(
                bg_attr,
                *linebuf_attr.offset(c as isize) as ::core::ffi::c_int,
            ) as sattr_T;
            c += 1;
        }
    }
    redraw_next =
        grid_char_needs_redraw(grid, col, off_to.wrapping_add(col as size_t), endcol - col) != 0;
    let mut start_dirty: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut end_dirty: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while col < endcol {
        let mut char_cells: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if (col + 1 as ::core::ffi::c_int) < endcol
            && *linebuf_char.offset((col + 1 as ::core::ffi::c_int) as isize) == 0 as schar_T
        {
            char_cells = 2 as ::core::ffi::c_int;
        }
        let mut redraw_this: bool = redraw_next;
        let mut off: size_t = off_to.wrapping_add(col as size_t);
        redraw_next = grid_char_needs_redraw(
            grid,
            col + char_cells,
            off.wrapping_add(char_cells as size_t),
            endcol - col - char_cells,
        ) != 0;
        if redraw_this {
            if start_dirty == -1 as ::core::ffi::c_int {
                start_dirty = col;
            }
            end_dirty = col + char_cells;
            if col + char_cells == endcol
                && off.wrapping_add(char_cells as size_t) < max_off_to
                && *(*grid)
                    .chars
                    .offset(off.wrapping_add(char_cells as size_t) as isize)
                    == NUL as schar_T
            {
                clear_next = true_0 != 0;
            }
            *(*grid).chars.offset(off as isize) = *linebuf_char.offset(col as isize);
            if char_cells == 2 as ::core::ffi::c_int {
                *(*grid).chars.offset(off.wrapping_add(1 as size_t) as isize) =
                    *linebuf_char.offset((col + 1 as ::core::ffi::c_int) as isize);
            }
            *(*grid).attrs.offset(off as isize) = *linebuf_attr.offset(col as isize);
            if char_cells == 2 as ::core::ffi::c_int {
                *(*grid).attrs.offset(off.wrapping_add(1 as size_t) as isize) =
                    *linebuf_attr.offset(col as isize);
            }
        }
        *(*grid).vcols.offset(off as isize) = *linebuf_vcol.offset(col as isize);
        if char_cells == 2 as ::core::ffi::c_int {
            *(*grid).vcols.offset(off.wrapping_add(1 as size_t) as isize) =
                *linebuf_vcol.offset((col + 1 as ::core::ffi::c_int) as isize);
        }
        col += char_cells;
    }
    if clear_next {
        *(*grid)
            .chars
            .offset(off_to.wrapping_add(col as size_t) as isize) =
            ' ' as ::core::ffi::c_int as schar_T;
        end_dirty += 1;
    }
    if off_to.wrapping_add(clear_width as size_t) < max_off_to
        && *(*grid)
            .chars
            .offset(off_to.wrapping_add(clear_width as size_t) as isize)
            == 0 as schar_T
    {
        clear_width += 1;
    }
    let mut clear_dirty_start: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut clear_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0 {
        col = clear_width - 1 as ::core::ffi::c_int;
        while col >= clear_start {
            let mut off_0: size_t = off_to.wrapping_add(col as size_t);
            *(*grid).vcols.offset(off_0 as isize) =
                if flags & SLF_INC_VCOL as ::core::ffi::c_int != 0 {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
            col -= 1;
        }
    }
    clear_attr = hl_combine_attr(bg_attr, clear_attr);
    col = clear_start;
    while col < clear_width {
        let mut off_1: size_t = off_to.wrapping_add(col as size_t);
        if *(*grid).chars.offset(off_1 as isize) != ' ' as ::core::ffi::c_int as schar_T
            || *(*grid).attrs.offset(off_1 as isize) != clear_attr as sattr_T
            || rdb_flags & kOptRdbFlagNodelta as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            *(*grid).chars.offset(off_1 as isize) = ' ' as ::core::ffi::c_int as schar_T;
            *(*grid).attrs.offset(off_1 as isize) = clear_attr as sattr_T;
            if clear_dirty_start == -1 as ::core::ffi::c_int {
                clear_dirty_start = col;
            }
            clear_end = col + 1 as ::core::ffi::c_int;
        }
        if flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0 {
            *(*grid).vcols.offset(off_1 as isize) =
                if flags & SLF_INC_VCOL as ::core::ffi::c_int != 0 {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
        }
        col += 1;
    }
    if flags & SLF_RIGHTLEFT as ::core::ffi::c_int != 0
        && start_dirty != -1 as ::core::ffi::c_int
        && clear_dirty_start != -1 as ::core::ffi::c_int
    {
        if (*grid).throttled as ::core::ffi::c_int != 0
            || clear_dirty_start >= start_dirty - 5 as ::core::ffi::c_int
        {
            start_dirty = clear_dirty_start;
        } else {
            ui_line(
                grid,
                row,
                invalid_row,
                coloff + clear_dirty_start,
                coloff + clear_dirty_start,
                coloff + clear_end,
                clear_attr,
                flags & SLF_WRAP as ::core::ffi::c_int != 0,
            );
        }
        clear_end = end_dirty;
    } else if start_dirty == -1 as ::core::ffi::c_int {
        start_dirty = clear_dirty_start;
        end_dirty = clear_dirty_start;
    } else if clear_end < end_dirty {
        clear_end = end_dirty;
    } else {
        end_dirty = endcol;
    }
    if clear_end > start_dirty {
        if !(*grid).throttled {
            ui_line(
                grid,
                row,
                invalid_row,
                coloff + start_dirty,
                coloff + end_dirty,
                coloff + clear_end,
                clear_attr,
                flags & SLF_WRAP as ::core::ffi::c_int != 0,
            );
        } else if !(*grid).dirty_col.is_null() {
            if clear_end > *(*grid).dirty_col.offset(row as isize) {
                *(*grid).dirty_col.offset(row as isize) = clear_end;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_alloc(
    mut grid: *mut ScreenGrid,
    mut rows: ::core::ffi::c_int,
    mut columns: ::core::ffi::c_int,
    mut copy: bool,
    mut valid: bool,
) {
    let mut new_row: ::core::ffi::c_int = 0;
    let mut ngrid: ScreenGrid = *grid;
    '_c2rust_label: {
        if rows >= 0 as ::core::ffi::c_int && columns >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"rows >= 0 && columns >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/grid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                846 as ::core::ffi::c_uint,
                b"void grid_alloc(ScreenGrid *, int, int, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut ncells: size_t = (rows as size_t).wrapping_mul(columns as size_t);
    ngrid.chars = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<schar_T>())) as *mut schar_T;
    ngrid.attrs = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<sattr_T>())) as *mut sattr_T;
    ngrid.vcols = xmalloc(ncells.wrapping_mul(::core::mem::size_of::<colnr_T>())) as *mut colnr_T;
    memset(
        ngrid.vcols as *mut ::core::ffi::c_void,
        -1 as ::core::ffi::c_int,
        ncells.wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
    ngrid.line_offset =
        xmalloc((rows as size_t).wrapping_mul(::core::mem::size_of::<size_t>())) as *mut size_t;
    ngrid.rows = rows;
    ngrid.cols = columns;
    new_row = 0 as ::core::ffi::c_int;
    while new_row < ngrid.rows {
        *ngrid.line_offset.offset(new_row as isize) =
            (new_row as size_t).wrapping_mul(ngrid.cols as size_t);
        grid_clear_line(
            &raw mut ngrid,
            *ngrid.line_offset.offset(new_row as isize),
            columns,
            valid,
        );
        if copy {
            if new_row < (*grid).rows && !(*grid).chars.is_null() {
                let mut len: ::core::ffi::c_int = if (*grid).cols < ngrid.cols {
                    (*grid).cols
                } else {
                    ngrid.cols
                };
                memmove(
                    ngrid
                        .chars
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .chars
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
                );
                memmove(
                    ngrid
                        .attrs
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .attrs
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
                );
                memmove(
                    ngrid
                        .vcols
                        .offset(*ngrid.line_offset.offset(new_row as isize) as isize)
                        as *mut ::core::ffi::c_void,
                    (*grid)
                        .vcols
                        .offset(*(*grid).line_offset.offset(new_row as isize) as isize)
                        as *const ::core::ffi::c_void,
                    (len as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
                );
            }
        }
        new_row += 1;
    }
    grid_free(grid);
    *grid = ngrid;
    if linebuf_size < columns as size_t {
        xfree(linebuf_char as *mut ::core::ffi::c_void);
        xfree(linebuf_attr as *mut ::core::ffi::c_void);
        xfree(linebuf_vcol as *mut ::core::ffi::c_void);
        xfree(linebuf_scratch as *mut ::core::ffi::c_void);
        linebuf_char = xmalloc((columns as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()))
            as *mut schar_T;
        linebuf_attr = xmalloc((columns as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()))
            as *mut sattr_T;
        linebuf_vcol = xmalloc((columns as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()))
            as *mut colnr_T;
        linebuf_scratch =
            xmalloc((columns as size_t).wrapping_mul(::core::mem::size_of::<sscratch_T>()))
                as *mut ::core::ffi::c_char;
        linebuf_size = columns as size_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_free(mut grid: *mut ScreenGrid) {
    xfree((*grid).chars as *mut ::core::ffi::c_void);
    xfree((*grid).attrs as *mut ::core::ffi::c_void);
    xfree((*grid).vcols as *mut ::core::ffi::c_void);
    xfree((*grid).line_offset as *mut ::core::ffi::c_void);
    (*grid).chars = ::core::ptr::null_mut::<schar_T>();
    (*grid).attrs = ::core::ptr::null_mut::<sattr_T>();
    (*grid).vcols = ::core::ptr::null_mut::<colnr_T>();
    (*grid).line_offset = ::core::ptr::null_mut::<size_t>();
}
#[no_mangle]
pub unsafe extern "C" fn win_grid_alloc(mut wp: *mut win_T) {
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let mut grid_allocated: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;
    let mut total_rows: ::core::ffi::c_int = (*wp).w_height_outer;
    let mut total_cols: ::core::ffi::c_int = (*wp).w_width_outer;
    let mut want_allocation: bool = ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        || (*wp).w_floating as ::core::ffi::c_int != 0;
    let mut has_allocation: bool = !(*grid_allocated).chars.is_null();
    if (*wp).w_view_height > (*wp).w_lines_size {
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        xfree((*wp).w_lines as *mut ::core::ffi::c_void);
        (*wp).w_lines = xcalloc(
            ((*wp).w_view_height as size_t).wrapping_add(1 as size_t),
            ::core::mem::size_of::<wline_T>(),
        ) as *mut wline_T;
        (*wp).w_lines_size = (*wp).w_view_height;
    }
    let mut was_resized: bool = false_0 != 0;
    if want_allocation as ::core::ffi::c_int != 0
        && (!has_allocation
            || (*grid_allocated).rows != total_rows
            || (*grid_allocated).cols != total_cols)
    {
        grid_alloc(
            grid_allocated,
            total_rows,
            total_cols,
            (*wp).w_grid_alloc.valid,
            false_0 != 0,
        );
        (*grid_allocated).valid = true_0 != 0;
        if (*wp).w_floating as ::core::ffi::c_int != 0
            && (*wp).w_config.border as ::core::ffi::c_int != 0
        {
            (*wp).w_redr_border = true_0 != 0;
        }
        was_resized = true_0 != 0;
    } else if !want_allocation && has_allocation as ::core::ffi::c_int != 0 {
        grid_free(grid_allocated);
        (*grid_allocated).valid = false_0 != 0;
        was_resized = true_0 != 0;
    } else if want_allocation as ::core::ffi::c_int != 0
        && has_allocation as ::core::ffi::c_int != 0
        && !(*wp).w_grid_alloc.valid
    {
        grid_invalidate(grid_allocated);
        (*grid_allocated).valid = true_0 != 0;
    }
    if want_allocation {
        (*grid).target = grid_allocated;
        (*grid).row_offset = (*wp).w_winrow_off;
        (*grid).col_offset = (*wp).w_wincol_off;
    } else {
        (*grid).target = &raw mut default_grid;
        (*grid).row_offset = (*wp).w_winrow + (*wp).w_winrow_off;
        (*grid).col_offset = (*wp).w_wincol + (*wp).w_wincol_off;
    }
    if (resizing_screen as ::core::ffi::c_int != 0 || was_resized as ::core::ffi::c_int != 0)
        && want_allocation as ::core::ffi::c_int != 0
    {
        ui_call_grid_resize(
            (*grid_allocated).handle as Integer,
            (*grid_allocated).cols as Integer,
            (*grid_allocated).rows as Integer,
        );
        ui_check_cursor_grid((*grid_allocated).handle);
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_assign_handle(mut grid: *mut ScreenGrid) {
    static mut last_grid_handle: ::core::ffi::c_int = DEFAULT_GRID_HANDLE;
    if (*grid).handle == 0 as ::core::ffi::c_int {
        last_grid_handle += 1;
        (*grid).handle = last_grid_handle as handle_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_ins_lines(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
    mut end: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut j: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_uint = 0;
    if line_count <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < line_count {
        if width != (*grid).cols {
            j = end - 1 as ::core::ffi::c_int - i;
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                linecopy(grid, j + line_count, j, col, width);
            }
            j += line_count;
            grid_clear_line(
                grid,
                (*(*grid).line_offset.offset(j as isize)).wrapping_add(col as size_t),
                width,
                false_0 != 0,
            );
        } else {
            j = end - 1 as ::core::ffi::c_int - i;
            temp = *(*grid).line_offset.offset(j as isize) as ::core::ffi::c_uint;
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                *(*grid).line_offset.offset((j + line_count) as isize) =
                    *(*grid).line_offset.offset(j as isize);
            }
            *(*grid).line_offset.offset((j + line_count) as isize) = temp as size_t;
            grid_clear_line(grid, temp as size_t, (*grid).cols, false_0 != 0);
        }
        i += 1;
    }
    if !(*grid).throttled {
        ui_call_grid_scroll(
            (*grid).handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            -line_count as Integer,
            0 as Integer,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn grid_del_lines(
    mut grid: *mut ScreenGrid,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
    mut end: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut j: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_uint = 0;
    if line_count <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < line_count {
        if width != (*grid).cols {
            j = row + i;
            loop {
                j += line_count;
                if j > end - 1 as ::core::ffi::c_int {
                    break;
                }
                linecopy(grid, j - line_count, j, col, width);
            }
            j -= line_count;
            grid_clear_line(
                grid,
                (*(*grid).line_offset.offset(j as isize)).wrapping_add(col as size_t),
                width,
                false_0 != 0,
            );
        } else {
            j = row + i;
            temp = *(*grid).line_offset.offset(j as isize) as ::core::ffi::c_uint;
            loop {
                j += line_count;
                if j > end - 1 as ::core::ffi::c_int {
                    break;
                }
                *(*grid).line_offset.offset((j - line_count) as isize) =
                    *(*grid).line_offset.offset(j as isize);
            }
            *(*grid).line_offset.offset((j - line_count) as isize) = temp as size_t;
            grid_clear_line(grid, temp as size_t, (*grid).cols, false_0 != 0);
        }
        i += 1;
    }
    if !(*grid).throttled {
        ui_call_grid_scroll(
            (*grid).handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            line_count as Integer,
            0 as Integer,
        );
    }
}
unsafe extern "C" fn grid_draw_bordertext(
    mut vt: VirtText,
    mut col: ::core::ffi::c_int,
    mut winbl: ::core::ffi::c_int,
    mut hl_attr: *const ::core::ffi::c_int,
    mut bt: BorderTextType,
    mut overflow: ::core::ffi::c_int,
) {
    let mut default_attr: ::core::ffi::c_int = *hl_attr.offset(
        (if bt as ::core::ffi::c_uint
            == kBorderTextTitle as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            HLF_BTITLE as ::core::ffi::c_int
        } else {
            HLF_BFOOTER as ::core::ffi::c_int
        }) as isize,
    );
    if overflow > 0 as ::core::ffi::c_int {
        grid_line_puts(
            1 as ::core::ffi::c_int,
            b"<\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            hl_apply_winblend(winbl, default_attr),
        );
        col += 1 as ::core::ffi::c_int;
        overflow += 1 as ::core::ffi::c_int;
    }
    let mut i: size_t = 0 as size_t;
    while i < vt.size {
        let mut attr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut text: *mut ::core::ffi::c_char =
            next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
        if text.is_null() {
            break;
        }
        if attr == -1 as ::core::ffi::c_int {
            attr = default_attr;
        }
        if overflow > 0 as ::core::ffi::c_int {
            let mut cells: ::core::ffi::c_int = mb_string2cells(text) as ::core::ffi::c_int;
            if overflow >= cells {
                overflow -= cells;
                continue;
            } else {
                let mut p: *mut ::core::ffi::c_char = text;
                while *p as ::core::ffi::c_int != 0 && overflow > 0 as ::core::ffi::c_int {
                    overflow -= utf_ptr2cells(p);
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                text = p;
            }
        }
        attr = hl_apply_winblend(winbl, attr);
        col += grid_line_puts(col, text, -1 as ::core::ffi::c_int, attr);
    }
}
unsafe extern "C" fn get_bordertext_col(
    mut total_col: ::core::ffi::c_int,
    mut text_width: ::core::ffi::c_int,
    mut align: AlignTextPos,
) -> ::core::ffi::c_int {
    match align as ::core::ffi::c_uint {
        0 => return 1 as ::core::ffi::c_int,
        1 => {
            return if (total_col - text_width) / 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                > 1 as ::core::ffi::c_int
            {
                (total_col - text_width) / 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        2 => {
            return if total_col - text_width + 1 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                total_col - text_width + 1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        _ => {}
    }
    unreachable!();
}
#[no_mangle]
pub unsafe extern "C" fn grid_draw_border(
    mut grid: *mut ScreenGrid,
    mut config: *mut WinConfig,
    mut adj: *mut ::core::ffi::c_int,
    mut winbl: ::core::ffi::c_int,
    mut hl_attr: *mut ::core::ffi::c_int,
) {
    let mut attrs: *mut ::core::ffi::c_int =
        &raw mut (*config).border_attr as *mut ::core::ffi::c_int;
    let mut default_adj: [::core::ffi::c_int; 4] = [
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ];
    if adj.is_null() {
        adj = &raw mut default_adj as *mut ::core::ffi::c_int;
    }
    let mut chars: [schar_T; 8] = [0; 8];
    if hl_attr.is_null() {
        hl_attr = hl_attr_active;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        chars[i as usize] = schar_from_str(
            &raw mut *(&raw mut (*config).border_chars as *mut [::core::ffi::c_char; 32])
                .offset(i as isize) as *mut ::core::ffi::c_char,
        );
        i += 1;
    }
    let mut irow: ::core::ffi::c_int = (*grid).rows
        - *adj.offset(0 as ::core::ffi::c_int as isize)
        - *adj.offset(2 as ::core::ffi::c_int as isize);
    let mut icol: ::core::ffi::c_int = (*grid).cols
        - *adj.offset(1 as ::core::ffi::c_int as isize)
        - *adj.offset(3 as ::core::ffi::c_int as isize);
    if *adj.offset(0 as ::core::ffi::c_int as isize) != 0 {
        screengrid_line_start(grid, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[0 as ::core::ffi::c_int as usize],
                *attrs.offset(0 as ::core::ffi::c_int as isize),
            );
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < icol {
            grid_line_put_schar(
                i_0 + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[1 as ::core::ffi::c_int as usize],
                *attrs.offset(1 as ::core::ffi::c_int as isize),
            );
            i_0 += 1;
        }
        if (*config).title {
            let mut title_col: ::core::ffi::c_int =
                get_bordertext_col(icol, (*config).title_width, (*config).title_pos);
            grid_draw_bordertext(
                (*config).title_chunks,
                title_col,
                winbl,
                hl_attr,
                kBorderTextTitle,
                (*config).title_width - icol,
            );
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[2 as ::core::ffi::c_int as usize],
                *attrs.offset(2 as ::core::ffi::c_int as isize),
            );
        }
        grid_line_flush();
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < irow {
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            screengrid_line_start(
                grid,
                i_1 + *adj.offset(0 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[7 as ::core::ffi::c_int as usize],
                *attrs.offset(7 as ::core::ffi::c_int as isize),
            );
            grid_line_flush();
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            let mut ic: ::core::ffi::c_int = if i_1 == 0 as ::core::ffi::c_int
                && *adj.offset(0 as ::core::ffi::c_int as isize) == 0
                && chars[2 as ::core::ffi::c_int as usize] != 0
            {
                2 as ::core::ffi::c_int
            } else {
                3 as ::core::ffi::c_int
            };
            screengrid_line_start(
                grid,
                i_1 + *adj.offset(0 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[ic as usize],
                *attrs.offset(ic as isize),
            );
            grid_line_flush();
        }
        i_1 += 1;
    }
    if *adj.offset(2 as ::core::ffi::c_int as isize) != 0 {
        screengrid_line_start(
            grid,
            irow + *adj.offset(0 as ::core::ffi::c_int as isize),
            0 as ::core::ffi::c_int,
        );
        if *adj.offset(3 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                0 as ::core::ffi::c_int,
                chars[6 as ::core::ffi::c_int as usize],
                *attrs.offset(6 as ::core::ffi::c_int as isize),
            );
        }
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < icol {
            let mut ic_0: ::core::ffi::c_int = if i_2 == 0 as ::core::ffi::c_int
                && *adj.offset(3 as ::core::ffi::c_int as isize) == 0
                && chars[6 as ::core::ffi::c_int as usize] != 0
            {
                6 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            };
            grid_line_put_schar(
                i_2 + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[ic_0 as usize],
                *attrs.offset(ic_0 as isize),
            );
            i_2 += 1;
        }
        if (*config).footer {
            let mut footer_col: ::core::ffi::c_int =
                get_bordertext_col(icol, (*config).footer_width, (*config).footer_pos);
            grid_draw_bordertext(
                (*config).footer_chunks,
                footer_col,
                winbl,
                hl_attr,
                kBorderTextFooter,
                (*config).footer_width - icol,
            );
        }
        if *adj.offset(1 as ::core::ffi::c_int as isize) != 0 {
            grid_line_put_schar(
                icol + *adj.offset(3 as ::core::ffi::c_int as isize),
                chars[4 as ::core::ffi::c_int as usize],
                *attrs.offset(4 as ::core::ffi::c_int as isize),
            );
        }
        grid_line_flush();
    }
}
unsafe extern "C" fn linecopy(
    mut grid: *mut ScreenGrid,
    mut to: ::core::ffi::c_int,
    mut from: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
) {
    let mut off_to: ::core::ffi::c_uint = (*(*grid).line_offset.offset(to as isize))
        .wrapping_add(col as size_t)
        as ::core::ffi::c_uint;
    let mut off_from: ::core::ffi::c_uint = (*(*grid).line_offset.offset(from as isize))
        .wrapping_add(col as size_t)
        as ::core::ffi::c_uint;
    memmove(
        (*grid).chars.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).chars.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    memmove(
        (*grid).attrs.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).attrs.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<sattr_T>()),
    );
    memmove(
        (*grid).vcols.offset(off_to as isize) as *mut ::core::ffi::c_void,
        (*grid).vcols.offset(off_from as isize) as *const ::core::ffi::c_void,
        (width as size_t).wrapping_mul(::core::mem::size_of::<colnr_T>()),
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_win_by_grid_handle(mut handle: handle_T) -> *mut win_T {
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_grid_alloc.handle == handle {
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn schar_from_char(mut c: ::core::ffi::c_int) -> schar_T {
    let mut sc: schar_T = 0 as schar_T;
    if c >= 0x200000 as ::core::ffi::c_int {
        c = 0xfffd as ::core::ffi::c_int;
    }
    utf_char2bytes(c, &raw mut sc as *mut ::core::ffi::c_char);
    return sc;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
