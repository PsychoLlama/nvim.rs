extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ins_bytes(p: *mut ::core::ffi::c_char);
    fn ins_str(s: *mut ::core::ffi::c_char, slen: size_t);
    fn del_char(fixpos: bool) -> ::core::ffi::c_int;
    fn del_bytes(count: colnr_T, fixpos_arg: bool, use_delcombine: bool) -> ::core::ffi::c_int;
    fn open_line(
        dir: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        second_line_indent: ::core::ffi::c_int,
        did_do_comment: *mut bool,
    ) -> bool;
    fn get_leader_len(
        line: *mut ::core::ffi::c_char,
        flags: *mut *mut ::core::ffi::c_char,
        backward: bool,
        include_space: bool,
    ) -> ::core::ffi::c_int;
    static mut p_paste: ::core::ffi::c_int;
    static mut p_smd: ::core::ffi::c_int;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getwhitecols_curline() -> intptr_t;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn inc_cursor() -> ::core::ffi::c_int;
    fn dec_cursor() -> ::core::ffi::c_int;
    fn check_cursor_col(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn pchar_cursor(c: ::core::ffi::c_char);
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn get_cursor_pos_len() -> colnr_T;
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn undisplay_dollar();
    fn backspace_until_column(col: ::core::ffi::c_int);
    fn insertchar(
        c: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        second_indent: ::core::ffi::c_int,
    );
    fn beginline(flags: ::core::ffi::c_int);
    fn get_nolist_virtcol() -> colnr_T;
    fn set_can_cindent(val: bool);
    fn eval_to_number(expr: *mut ::core::ffi::c_char, use_simple_function: bool) -> varnumber_T;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_vim_var_char(c: ::core::ffi::c_int);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn beep_flush();
    static mut current_sctx: sctx_T;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut sandbox: ::core::ffi::c_int;
    static mut did_ai: bool;
    static mut did_si: bool;
    static mut can_si: bool;
    static mut can_si_back: bool;
    static mut old_indent: ::core::ffi::c_int;
    static mut saved_cursor: pos_T;
    static mut Insstart: pos_T;
    static mut State: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut got_int: bool;
    static mut replace_offset: ::core::ffi::c_int;
    static mut cmdwin_buf: *mut buf_T;
    fn get_indent() -> ::core::ffi::c_int;
    fn get_indent_lnum(lnum: linenr_T) -> ::core::ffi::c_int;
    fn set_indent(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> bool;
    fn get_number_indent(lnum: linenr_T) -> ::core::ffi::c_int;
    fn change_indent(
        type_0: ::core::ffi::c_int,
        amount: ::core::ffi::c_int,
        round: ::core::ffi::c_int,
        call_changed_bytes: bool,
    );
    fn get_expr_indent() -> ::core::ffi::c_int;
    fn get_lisp_indent() -> ::core::ffi::c_int;
    fn cindent_on() -> bool;
    fn get_c_indent() -> ::core::ffi::c_int;
    fn mark_col_adjust(
        lnum: linenr_T,
        mincol: colnr_T,
        lnum_amount: linenr_T,
        col_amount: colnr_T,
        spaces_removed: ::core::ffi::c_int,
    );
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_iscomposing_first(c: ::core::ffi::c_int) -> bool;
    fn utf_allow_break_before(cc: ::core::ffi::c_int) -> bool;
    fn utf_allow_break(cc: ::core::ffi::c_int, ncc: ::core::ffi::c_int) -> bool;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_replace(lnum: linenr_T, line: *mut ::core::ffi::c_char, copy: bool)
        -> ::core::ffi::c_int;
    fn msgmore(n: ::core::ffi::c_int);
    fn update_topline(wp: *mut win_T);
    fn do_join(
        count: size_t,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark: bool,
    ) -> ::core::ffi::c_int;
    fn was_set_insecurely(
        wp: *mut win_T,
        opt_idx: OptIndex,
        opt_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn line_breakcheck();
    fn check_linecomment(line: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn startPS(lnum: linenr_T, para: ::core::ffi::c_int, both: bool) -> bool;
    fn ui_cursor_shape();
    fn u_save_cursor() -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn win_fdccol_count(wp: *mut win_T) -> ::core::ffi::c_int;
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
pub type size_t = usize;
pub type time_t = __time_t;
pub type ptrdiff_t = isize;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_13 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_13 = 3;
pub const BACKWARD: C2Rust_Unnamed_13 = -1;
pub const FORWARD: C2Rust_Unnamed_13 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_13 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
pub type OptIndex = ::core::ffi::c_int;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const kBufOptWrapmargin: C2Rust_Unnamed_14 = 91;
pub const kBufOptVartabstop: C2Rust_Unnamed_14 = 90;
pub const kBufOptVarsofttabstop: C2Rust_Unnamed_14 = 89;
pub const kBufOptUndolevels: C2Rust_Unnamed_14 = 88;
pub const kBufOptUndofile: C2Rust_Unnamed_14 = 87;
pub const kBufOptThesaurusfunc: C2Rust_Unnamed_14 = 86;
pub const kBufOptThesaurus: C2Rust_Unnamed_14 = 85;
pub const kBufOptTextwidth: C2Rust_Unnamed_14 = 84;
pub const kBufOptTags: C2Rust_Unnamed_14 = 83;
pub const kBufOptTagfunc: C2Rust_Unnamed_14 = 82;
pub const kBufOptTagcase: C2Rust_Unnamed_14 = 81;
pub const kBufOptTabstop: C2Rust_Unnamed_14 = 80;
pub const kBufOptSyntax: C2Rust_Unnamed_14 = 79;
pub const kBufOptSynmaxcol: C2Rust_Unnamed_14 = 78;
pub const kBufOptSwapfile: C2Rust_Unnamed_14 = 77;
pub const kBufOptSuffixesadd: C2Rust_Unnamed_14 = 76;
pub const kBufOptSpelloptions: C2Rust_Unnamed_14 = 75;
pub const kBufOptSpelllang: C2Rust_Unnamed_14 = 74;
pub const kBufOptSpellfile: C2Rust_Unnamed_14 = 73;
pub const kBufOptSpellcapcheck: C2Rust_Unnamed_14 = 72;
pub const kBufOptSofttabstop: C2Rust_Unnamed_14 = 71;
pub const kBufOptSmartindent: C2Rust_Unnamed_14 = 70;
pub const kBufOptShiftwidth: C2Rust_Unnamed_14 = 69;
pub const kBufOptScrollback: C2Rust_Unnamed_14 = 68;
pub const kBufOptReadonly: C2Rust_Unnamed_14 = 67;
pub const kBufOptQuoteescape: C2Rust_Unnamed_14 = 66;
pub const kBufOptPreserveindent: C2Rust_Unnamed_14 = 65;
pub const kBufOptPath: C2Rust_Unnamed_14 = 64;
pub const kBufOptOmnifunc: C2Rust_Unnamed_14 = 63;
pub const kBufOptNrformats: C2Rust_Unnamed_14 = 62;
pub const kBufOptModified: C2Rust_Unnamed_14 = 61;
pub const kBufOptModifiable: C2Rust_Unnamed_14 = 60;
pub const kBufOptModeline: C2Rust_Unnamed_14 = 59;
pub const kBufOptMatchpairs: C2Rust_Unnamed_14 = 58;
pub const kBufOptMakeprg: C2Rust_Unnamed_14 = 57;
pub const kBufOptMakeencoding: C2Rust_Unnamed_14 = 56;
pub const kBufOptLispwords: C2Rust_Unnamed_14 = 55;
pub const kBufOptLispoptions: C2Rust_Unnamed_14 = 54;
pub const kBufOptLisp: C2Rust_Unnamed_14 = 53;
pub const kBufOptKeywordprg: C2Rust_Unnamed_14 = 52;
pub const kBufOptKeymap: C2Rust_Unnamed_14 = 51;
pub const kBufOptIskeyword: C2Rust_Unnamed_14 = 50;
pub const kBufOptInfercase: C2Rust_Unnamed_14 = 49;
pub const kBufOptIndentkeys: C2Rust_Unnamed_14 = 48;
pub const kBufOptIndentexpr: C2Rust_Unnamed_14 = 47;
pub const kBufOptIncludeexpr: C2Rust_Unnamed_14 = 46;
pub const kBufOptInclude: C2Rust_Unnamed_14 = 45;
pub const kBufOptImsearch: C2Rust_Unnamed_14 = 44;
pub const kBufOptIminsert: C2Rust_Unnamed_14 = 43;
pub const kBufOptGrepprg: C2Rust_Unnamed_14 = 42;
pub const kBufOptGrepformat: C2Rust_Unnamed_14 = 41;
pub const kBufOptFsync: C2Rust_Unnamed_14 = 40;
pub const kBufOptFormatprg: C2Rust_Unnamed_14 = 39;
pub const kBufOptFormatoptions: C2Rust_Unnamed_14 = 38;
pub const kBufOptFormatlistpat: C2Rust_Unnamed_14 = 37;
pub const kBufOptFormatexpr: C2Rust_Unnamed_14 = 36;
pub const kBufOptFixendofline: C2Rust_Unnamed_14 = 35;
pub const kBufOptFindfunc: C2Rust_Unnamed_14 = 34;
pub const kBufOptFiletype: C2Rust_Unnamed_14 = 33;
pub const kBufOptFileformat: C2Rust_Unnamed_14 = 32;
pub const kBufOptFileencoding: C2Rust_Unnamed_14 = 31;
pub const kBufOptExpandtab: C2Rust_Unnamed_14 = 30;
pub const kBufOptErrorformat: C2Rust_Unnamed_14 = 29;
pub const kBufOptEqualprg: C2Rust_Unnamed_14 = 28;
pub const kBufOptEndofline: C2Rust_Unnamed_14 = 27;
pub const kBufOptEndoffile: C2Rust_Unnamed_14 = 26;
pub const kBufOptDiffanchors: C2Rust_Unnamed_14 = 25;
pub const kBufOptDictionary: C2Rust_Unnamed_14 = 24;
pub const kBufOptDefine: C2Rust_Unnamed_14 = 23;
pub const kBufOptCopyindent: C2Rust_Unnamed_14 = 22;
pub const kBufOptCompleteslash: C2Rust_Unnamed_14 = 21;
pub const kBufOptCompleteopt: C2Rust_Unnamed_14 = 20;
pub const kBufOptCompletefunc: C2Rust_Unnamed_14 = 19;
pub const kBufOptComplete: C2Rust_Unnamed_14 = 18;
pub const kBufOptCommentstring: C2Rust_Unnamed_14 = 17;
pub const kBufOptComments: C2Rust_Unnamed_14 = 16;
pub const kBufOptCinwords: C2Rust_Unnamed_14 = 15;
pub const kBufOptCinscopedecls: C2Rust_Unnamed_14 = 14;
pub const kBufOptCinoptions: C2Rust_Unnamed_14 = 13;
pub const kBufOptCinkeys: C2Rust_Unnamed_14 = 12;
pub const kBufOptCindent: C2Rust_Unnamed_14 = 11;
pub const kBufOptChannel: C2Rust_Unnamed_14 = 10;
pub const kBufOptBusy: C2Rust_Unnamed_14 = 9;
pub const kBufOptBuftype: C2Rust_Unnamed_14 = 8;
pub const kBufOptBuflisted: C2Rust_Unnamed_14 = 7;
pub const kBufOptBufhidden: C2Rust_Unnamed_14 = 6;
pub const kBufOptBomb: C2Rust_Unnamed_14 = 5;
pub const kBufOptBinary: C2Rust_Unnamed_14 = 4;
pub const kBufOptBackupcopy: C2Rust_Unnamed_14 = 3;
pub const kBufOptAutoread: C2Rust_Unnamed_14 = 2;
pub const kBufOptAutoindent: C2Rust_Unnamed_14 = 1;
pub const kBufOptAutocomplete: C2Rust_Unnamed_14 = 0;
pub const kBufOptInvalid: C2Rust_Unnamed_14 = -1;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_15 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_15 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_15 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_15 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_15 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_15 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_16 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_16 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_16 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_16 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_16 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_16 = 20;
pub const UPD_VALID: C2Rust_Unnamed_16 = 10;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    pub cmod_flags: ::core::ffi::c_int,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: regmatch_T,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const INDENT_DEC: C2Rust_Unnamed_18 = 3;
pub const INDENT_INC: C2Rust_Unnamed_18 = 2;
pub const INDENT_SET: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_19 = 4;
pub const BL_SOL: C2Rust_Unnamed_19 = 2;
pub const BL_WHITE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const INSCHAR_COM_LIST: C2Rust_Unnamed_20 = 16;
pub const INSCHAR_NO_FEX: C2Rust_Unnamed_20 = 8;
pub const INSCHAR_CTRLV: C2Rust_Unnamed_20 = 4;
pub const INSCHAR_DO_COM: C2Rust_Unnamed_20 = 2;
pub const INSCHAR_FORMAT: C2Rust_Unnamed_20 = 1;
pub type VimVarIndex = ::core::ffi::c_uint;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_21 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_21 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_21 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_21 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_21 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_21 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_21 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_21 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_21 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_21 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_21 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_21 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_21 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_21 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_21 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_21 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_21 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_21 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_21 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_21 = 1;
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
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const SIN_NOMARK: C2Rust_Unnamed_22 = 8;
pub const SIN_UNDO: C2Rust_Unnamed_22 = 4;
pub const SIN_INSERT: C2Rust_Unnamed_22 = 2;
pub const SIN_CHANGED: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_23 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_23 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_23 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_23 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_23 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_23 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_23 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FO_WRAP: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
pub const FO_WRAP_COMS: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const FO_Q_COMS: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const FO_Q_NUMBER: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const FO_Q_SECOND: ::core::ffi::c_int = '2' as ::core::ffi::c_int;
pub const FO_INS_VI: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_MBYTE_BREAK: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const FO_ONE_LETTER: ::core::ffi::c_int = '1' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_RIGOROUS_TW: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const FO_PERIOD_ABBR: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
static mut did_add_space: bool = false_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn has_format_option(mut x: ::core::ffi::c_int) -> bool {
    if p_paste != 0 {
        return false_0 != 0;
    }
    return !vim_strchr((*curbuf).b_p_fo, x).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn internal_format(
    mut textwidth: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut format_only: bool,
    mut c: ::core::ffi::c_int,
) {
    let mut cc: ::core::ffi::c_int = 0;
    let mut save_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut haveto_redraw: bool = false_0 != 0;
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    let fo_multibyte: bool = has_format_option(FO_MBYTE_BREAK);
    let fo_rigor_tw: bool = has_format_option(FO_RIGOROUS_TW);
    let fo_white_par: bool = has_format_option(FO_WHITE_PAR);
    let mut first_line: bool = true_0 != 0;
    let mut leader_len: colnr_T = 0;
    let mut no_leader: bool = false_0 != 0;
    let mut do_comments: bool = flags & INSCHAR_DO_COM as ::core::ffi::c_int != 0;
    let mut has_lbr: ::core::ffi::c_int = (*curwin).w_onebuf_opt.wo_lbr;
    (*curwin).w_onebuf_opt.wo_lbr = false_0;
    if (*curbuf).b_p_ai == 0 && State & VREPLACE_FLAG as ::core::ffi::c_int == 0 {
        cc = gchar_cursor();
        if ascii_iswhite(cc) {
            save_char = cc as ::core::ffi::c_char;
            pchar_cursor('x' as ::core::ffi::c_char);
        }
    }
    while !got_int {
        let mut startcol: ::core::ffi::c_int = 0;
        let mut wantcol: ::core::ffi::c_int = 0;
        let mut foundcol: ::core::ffi::c_int = 0;
        let mut end_foundcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut saved_text: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut col: colnr_T = 0;
        let mut did_do_comment: bool = false_0 != 0;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells((if c != NUL { c } else { gchar_cursor() }));
        if virtcol <= textwidth {
            break;
        }
        if no_leader {
            do_comments = false_0 != 0;
        } else if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        {
            do_comments = true_0 != 0;
        }
        if do_comments {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            leader_len = get_leader_len(
                line,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                false_0 != 0,
                true_0 != 0,
            ) as colnr_T;
            if leader_len == 0 as ::core::ffi::c_int && (*curbuf).b_p_cin != 0 {
                let mut comment_start: ::core::ffi::c_int = check_linecomment(line);
                if comment_start != MAXCOL as ::core::ffi::c_int {
                    leader_len = get_leader_len(
                        line.offset(comment_start as isize),
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        true_0 != 0,
                    ) as colnr_T;
                    if leader_len != 0 as ::core::ffi::c_int {
                        leader_len += comment_start;
                    }
                }
            }
        } else {
            leader_len = 0 as ::core::ffi::c_int as colnr_T;
        }
        if leader_len == 0 as ::core::ffi::c_int {
            no_leader = true_0 != 0;
        }
        if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && leader_len == 0 as ::core::ffi::c_int
            && !has_format_option(FO_WRAP)
        {
            break;
        }
        startcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
        if startcol == 0 as ::core::ffi::c_int {
            break;
        }
        coladvance(curwin, textwidth);
        wantcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
        (*curwin).w_cursor.col = startcol as colnr_T;
        foundcol = 0 as ::core::ffi::c_int;
        let mut skip_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !fo_ins_blank && !has_format_option(FO_INS_VI)
            || flags & INSCHAR_FORMAT as ::core::ffi::c_int != 0
            || (*curwin).w_cursor.lnum != Insstart.lnum
            || (*curwin).w_cursor.col >= Insstart.col
        {
            if (*curwin).w_cursor.col == startcol && c != NUL {
                cc = c;
            } else {
                cc = gchar_cursor();
            }
            if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                && !utf_iscomposing_first(utf_ptr2char(
                    get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                ))
            {
                let mut end_col: colnr_T = (*curwin).w_cursor.col;
                let mut wcc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    dec_cursor();
                    cc = gchar_cursor();
                    if wcc < 2 as ::core::ffi::c_int {
                        wcc += 1;
                    }
                }
                if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    break;
                } else {
                    if has_format_option(FO_PERIOD_ABBR) as ::core::ffi::c_int != 0
                        && cc == '.' as ::core::ffi::c_int
                        && wcc < 2 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if (*curwin).w_cursor.col < leader_len {
                        break;
                    }
                    if has_format_option(FO_ONE_LETTER) {
                        if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
                            break;
                        }
                        if (*curwin).w_cursor.col <= leader_len {
                            break;
                        }
                        col = (*curwin).w_cursor.col;
                        dec_cursor();
                        cc = gchar_cursor();
                        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                            && !utf_iscomposing_first(utf_ptr2char(
                                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                            ))
                        {
                            continue;
                        } else {
                            (*curwin).w_cursor.col = col;
                        }
                    }
                    inc_cursor();
                    end_foundcol = end_col as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    foundcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
                    if (*curwin).w_cursor.col <= wantcol {
                        break;
                    }
                }
            } else if (cc >= 0x100 as ::core::ffi::c_int || !utf_allow_break_before(cc))
                && fo_multibyte as ::core::ffi::c_int != 0
            {
                let mut ncc: ::core::ffi::c_int = 0;
                let mut allow_break: bool = false;
                if (*curwin).w_cursor.col != startcol {
                    if (*curwin).w_cursor.col < leader_len {
                        break;
                    }
                    col = (*curwin).w_cursor.col;
                    inc_cursor();
                    ncc = gchar_cursor();
                    allow_break = utf_allow_break(cc, ncc);
                    if (*curwin).w_cursor.col != skip_pos && allow_break as ::core::ffi::c_int != 0
                    {
                        foundcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                        if (*curwin).w_cursor.col <= wantcol {
                            break;
                        }
                    }
                    (*curwin).w_cursor.col = col;
                }
                if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
                    break;
                }
                ncc = cc;
                col = (*curwin).w_cursor.col;
                dec_cursor();
                cc = gchar_cursor();
                if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                {
                    continue;
                } else {
                    if (*curwin).w_cursor.col < leader_len {
                        break;
                    }
                    (*curwin).w_cursor.col = col;
                    skip_pos = (*curwin).w_cursor.col as ::core::ffi::c_int;
                    allow_break = utf_allow_break(cc, ncc);
                    if allow_break {
                        foundcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                    }
                    if (*curwin).w_cursor.col <= wantcol {
                        let ncc_allow_break: bool = utf_allow_break_before(ncc);
                        if allow_break {
                            break;
                        }
                        if !ncc_allow_break && !fo_rigor_tw {
                            if (*curwin).w_cursor.col == startcol {
                                foundcol = 0 as ::core::ffi::c_int;
                                end_foundcol = foundcol;
                                break;
                            } else {
                                col = (*curwin).w_cursor.col;
                                inc_cursor();
                                cc = ncc;
                                ncc = gchar_cursor();
                                ncc = if ncc != NUL { ncc } else { c };
                                allow_break = utf_allow_break(cc, ncc);
                                if allow_break {
                                    foundcol = if ncc == NUL {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        (*curwin).w_cursor.col as ::core::ffi::c_int
                                    };
                                    end_foundcol = foundcol;
                                    break;
                                } else {
                                    (*curwin).w_cursor.col = col;
                                }
                            }
                        }
                    }
                }
            }
            if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
                break;
            }
            dec_cursor();
        }
        if foundcol == 0 as ::core::ffi::c_int {
            (*curwin).w_cursor.col = startcol as colnr_T;
            break;
        } else {
            undisplay_dollar();
            if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                orig_col = startcol;
            } else {
                replace_offset = startcol - end_foundcol;
            }
            (*curwin).w_cursor.col = foundcol as colnr_T;
            loop {
                cc = gchar_cursor();
                if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                    && (!fo_white_par || (*curwin).w_cursor.col < startcol))
                {
                    break;
                }
                inc_cursor();
            }
            startcol -= (*curwin).w_cursor.col as ::core::ffi::c_int;
            startcol = if startcol > 0 as ::core::ffi::c_int {
                startcol
            } else {
                0 as ::core::ffi::c_int
            };
            if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                saved_text = xstrnsave(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
                (*curwin).w_cursor.col = orig_col as colnr_T;
                *saved_text.offset(startcol as isize) = NUL as ::core::ffi::c_char;
                if !fo_white_par {
                    backspace_until_column(foundcol);
                }
            } else if !fo_white_par {
                (*curwin).w_cursor.col = foundcol as colnr_T;
            }
            open_line(
                FORWARD as ::core::ffi::c_int,
                OPENLINE_DELSPACES as ::core::ffi::c_int
                    + OPENLINE_MARKFIX as ::core::ffi::c_int
                    + (if fo_white_par as ::core::ffi::c_int != 0 {
                        OPENLINE_KEEPTRAIL as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + (if do_comments as ::core::ffi::c_int != 0 {
                        OPENLINE_DO_COM as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + OPENLINE_FORMAT as ::core::ffi::c_int
                    + (if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                        OPENLINE_COM_LIST as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                    second_indent
                } else {
                    old_indent
                },
                &raw mut did_do_comment,
            );
            if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                old_indent = 0 as ::core::ffi::c_int;
            }
            if did_do_comment {
                no_leader = false_0 != 0;
            }
            replace_offset = 0 as ::core::ffi::c_int;
            if first_line {
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                    if second_indent < 0 as ::core::ffi::c_int
                        && has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
                    {
                        second_indent = get_number_indent((*curwin).w_cursor.lnum - 1 as linenr_T);
                    }
                    if second_indent >= 0 as ::core::ffi::c_int {
                        if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                            change_indent(
                                INDENT_SET as ::core::ffi::c_int,
                                second_indent,
                                false_0,
                                true_0 != 0,
                            );
                        } else if leader_len > 0 as ::core::ffi::c_int
                            && second_indent as colnr_T - leader_len > 0 as ::core::ffi::c_int
                        {
                            let mut padding: ::core::ffi::c_int =
                                second_indent - leader_len as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < padding {
                                ins_str(
                                    b" \0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                        .wrapping_sub(1 as size_t),
                                );
                                i += 1;
                            }
                        } else {
                            set_indent(second_indent, SIN_CHANGED as ::core::ffi::c_int);
                        }
                    }
                }
                first_line = false_0 != 0;
            }
            if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                ins_bytes(saved_text);
                xfree(saved_text as *mut ::core::ffi::c_void);
            } else {
                (*curwin).w_cursor.col += startcol;
                let mut len: colnr_T = get_cursor_line_len();
                (*curwin).w_cursor.col = if (*curwin).w_cursor.col < len {
                    (*curwin).w_cursor.col
                } else {
                    len
                };
            }
            haveto_redraw = true_0 != 0;
            set_can_cindent(true_0 != 0);
            did_ai = false_0 != 0;
            did_si = false_0 != 0;
            can_si = false_0 != 0;
            can_si_back = false_0 != 0;
            line_breakcheck();
        }
    }
    if save_char as ::core::ffi::c_int != NUL {
        pchar_cursor(save_char);
    }
    (*curwin).w_onebuf_opt.wo_lbr = has_lbr;
    if !format_only && haveto_redraw as ::core::ffi::c_int != 0 {
        update_topline(curwin);
        redraw_curbuf_later(UPD_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn fmt_check_par(
    mut lnum: linenr_T,
    mut leader_len: *mut ::core::ffi::c_int,
    mut leader_flags: *mut *mut ::core::ffi::c_char,
    mut do_comments: bool,
) -> ::core::ffi::c_int {
    let mut flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ml_get(lnum);
    if do_comments {
        *leader_len = get_leader_len(ptr, leader_flags, false_0 != 0, true_0 != 0);
    } else {
        *leader_len = 0 as ::core::ffi::c_int;
    }
    if *leader_len > 0 as ::core::ffi::c_int {
        flags = *leader_flags;
        while *flags as ::core::ffi::c_int != 0
            && *flags as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            && *flags as ::core::ffi::c_int != COM_END
        {
            flags = flags.offset(1);
        }
    }
    return (*skipwhite(ptr.offset(*leader_len as isize)) as ::core::ffi::c_int == NUL
        || *leader_len > 0 as ::core::ffi::c_int && *flags as ::core::ffi::c_int == COM_END
        || startPS(lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn ends_in_white(mut lnum: linenr_T) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut l: colnr_T = ml_get_len(lnum) - 1 as colnr_T;
    return ascii_iswhite(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int);
}
unsafe extern "C" fn same_leader(
    mut lnum: linenr_T,
    mut leader1_len: ::core::ffi::c_int,
    mut leader1_flags: *mut ::core::ffi::c_char,
    mut leader2_len: ::core::ffi::c_int,
    mut leader2_flags: *mut ::core::ffi::c_char,
) -> bool {
    let mut idx1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if leader1_len == 0 as ::core::ffi::c_int {
        return leader2_len == 0 as ::core::ffi::c_int;
    }
    if !leader1_flags.is_null() {
        let mut p: *mut ::core::ffi::c_char = leader1_flags;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == COM_FIRST {
                return leader2_len == 0 as ::core::ffi::c_int;
            }
            if *p as ::core::ffi::c_int == COM_END {
                return false_0 != 0;
            }
            if *p as ::core::ffi::c_int == COM_START {
                let mut line_len: ::core::ffi::c_int = ml_get_len(lnum);
                if line_len <= leader1_len {
                    return false_0 != 0;
                }
                if leader2_flags.is_null() || leader2_len == 0 as ::core::ffi::c_int {
                    return false_0 != 0;
                }
                p = leader2_flags;
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_MIDDLE {
                        return true_0 != 0;
                    }
                    p = p.offset(1);
                }
                return false_0 != 0;
            }
            p = p.offset(1);
        }
    }
    let mut line1: *mut ::core::ffi::c_char = xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
    idx1 = 0 as ::core::ffi::c_int;
    while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
        idx1 += 1;
    }
    let mut line2: *mut ::core::ffi::c_char = ml_get(lnum + 1 as linenr_T);
    idx2 = 0 as ::core::ffi::c_int;
    while idx2 < leader2_len {
        if !ascii_iswhite(*line2.offset(idx2 as isize) as ::core::ffi::c_int) {
            let c2rust_fresh0 = idx1;
            idx1 = idx1 + 1;
            if *line1.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int
                != *line2.offset(idx2 as isize) as ::core::ffi::c_int
            {
                break;
            }
        } else {
            while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
                idx1 += 1;
            }
        }
        idx2 += 1;
    }
    xfree(line1 as *mut ::core::ffi::c_void);
    return idx2 == leader2_len && idx1 == leader1_len;
}
unsafe extern "C" fn paragraph_start(mut lnum: linenr_T) -> bool {
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lnum <= 1 as linenr_T {
        return true_0 != 0;
    }
    let mut p: *mut ::core::ffi::c_char = ml_get(lnum - 1 as linenr_T);
    if *p as ::core::ffi::c_int == NUL {
        return true_0 != 0;
    }
    let do_comments: bool = has_format_option(FO_Q_COMS);
    if fmt_check_par(
        lnum - 1 as linenr_T,
        &raw mut leader_len,
        &raw mut leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if fmt_check_par(
        lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
        && !ends_in_white(lnum - 1 as linenr_T)
    {
        return true_0 != 0;
    }
    if has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
        && get_number_indent(lnum) > 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if !same_leader(
        lnum - 1 as linenr_T,
        leader_len,
        leader_flags,
        next_leader_len,
        next_leader_flags,
    ) {
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn auto_format(mut trailblank: bool, mut prev_line: bool) {
    if !has_format_option(FO_AUTO) {
        return;
    }
    let mut pos: pos_T = (*curwin).w_cursor;
    let mut old: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    check_auto_format(false_0 != 0);
    let mut wasatend: bool = pos.col == get_cursor_line_len();
    if *old as ::core::ffi::c_int != NUL && !trailblank && wasatend as ::core::ffi::c_int != 0 {
        dec_cursor();
        let mut cc: ::core::ffi::c_int = gchar_cursor();
        if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            )))
            && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
            && has_format_option(FO_ONE_LETTER) as ::core::ffi::c_int != 0
        {
            dec_cursor();
        }
        cc = gchar_cursor();
        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin).w_cursor = pos;
            return;
        }
        (*curwin).w_cursor = pos;
    }
    if *old as ::core::ffi::c_int != NUL
        && !trailblank
        && !wasatend
        && pos.col > 0 as ::core::ffi::c_int
        && State & MODE_INSERT as ::core::ffi::c_int != 0
    {
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if ascii_iswhite(
            *line.offset((pos.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin).w_cursor = pos;
            return;
        }
    }
    if has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        && !has_format_option(FO_WRAP)
        && get_leader_len(
            old,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if prev_line as ::core::ffi::c_int != 0 && !paragraph_start((*curwin).w_cursor.lnum) {
        (*curwin).w_cursor.lnum -= 1;
        if u_save_cursor() == FAIL {
            return;
        }
    }
    saved_cursor = pos;
    format_lines(-1 as linenr_T, false_0 != 0);
    (*curwin).w_cursor = saved_cursor;
    saved_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    if (*curwin).w_cursor.lnum > (*curbuf).b_ml.ml_line_count {
        (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
        coladvance(curwin, MAXCOL as ::core::ffi::c_int);
    } else {
        check_cursor_col(curwin);
    }
    if !wasatend && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0 {
        let mut linep: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut len: colnr_T = get_cursor_line_len();
        if (*curwin).w_cursor.col == len {
            let mut plinep: *mut ::core::ffi::c_char =
                xstrnsave(linep, (len as size_t).wrapping_add(2 as size_t));
            *plinep.offset(len as isize) = ' ' as ::core::ffi::c_char;
            *plinep.offset((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                NUL as ::core::ffi::c_char;
            ml_replace((*curwin).w_cursor.lnum, plinep, false_0 != 0);
            did_add_space = true_0 != 0;
        } else {
            check_auto_format(false_0 != 0);
        }
    }
    check_cursor(curwin);
}
#[no_mangle]
pub unsafe extern "C" fn check_auto_format(mut end_insert: bool) {
    if !did_add_space {
        return;
    }
    let mut cc: ::core::ffi::c_int = gchar_cursor();
    if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
        && !utf_iscomposing_first(utf_ptr2char(
            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
        )))
    {
        did_add_space = false_0 != 0;
    } else {
        let mut c: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
        if !end_insert {
            inc_cursor();
            c = gchar_cursor();
            dec_cursor();
        }
        if c != NUL {
            del_char(false_0 != 0);
            did_add_space = false_0 != 0;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn comp_textwidth(mut ff: bool) -> ::core::ffi::c_int {
    let mut textwidth: ::core::ffi::c_int = (*curbuf).b_p_tw as ::core::ffi::c_int;
    if textwidth == 0 as ::core::ffi::c_int && (*curbuf).b_p_wm != 0 {
        textwidth = (*curwin).w_view_width - (*curbuf).b_p_wm as ::core::ffi::c_int;
        if curbuf == cmdwin_buf {
            textwidth -= 1 as ::core::ffi::c_int;
        }
        textwidth -= win_fdccol_count(curwin);
        textwidth -= (*curwin).w_scwidth;
        if (*curwin).w_onebuf_opt.wo_nu != 0 || (*curwin).w_onebuf_opt.wo_rnu != 0 {
            textwidth -= 8 as ::core::ffi::c_int;
        }
    }
    textwidth = if textwidth > 0 as ::core::ffi::c_int {
        textwidth
    } else {
        0 as ::core::ffi::c_int
    };
    if ff as ::core::ffi::c_int != 0 && textwidth == 0 as ::core::ffi::c_int {
        textwidth = if ((*curwin).w_view_width - 1 as ::core::ffi::c_int) < 79 as ::core::ffi::c_int
        {
            (*curwin).w_view_width - 1 as ::core::ffi::c_int
        } else {
            79 as ::core::ffi::c_int
        };
    }
    return textwidth;
}
#[no_mangle]
pub unsafe extern "C" fn op_format(mut oap: *mut oparg_T, mut keep_cursor: bool) {
    let mut old_line_count: linenr_T = (*curbuf).b_ml.ml_line_count;
    (*curwin).w_cursor = (*oap).cursor_start;
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return;
    }
    (*curwin).w_cursor = (*oap).start;
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*curbuf).b_op_start = (*oap).start;
    }
    if keep_cursor {
        saved_cursor = (*oap).cursor_start;
    }
    format_lines((*oap).line_count, keep_cursor);
    if (*oap).end_adjusted as ::core::ffi::c_int != 0
        && (*curwin).w_cursor.lnum < (*curbuf).b_ml.ml_line_count
    {
        (*curwin).w_cursor.lnum += 1;
    }
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    old_line_count = (*curbuf).b_ml.ml_line_count - old_line_count;
    msgmore(old_line_count as ::core::ffi::c_int);
    if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*curbuf).b_op_end = (*curwin).w_cursor;
    }
    if keep_cursor {
        (*curwin).w_cursor = saved_cursor;
        saved_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
        check_cursor(curwin);
    }
    if (*oap).is_VIsual {
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_old_cursor_lnum != 0 as linenr_T {
                if (*wp).w_old_cursor_lnum > (*wp).w_old_visual_lnum {
                    (*wp).w_old_cursor_lnum += old_line_count;
                } else {
                    (*wp).w_old_visual_lnum += old_line_count;
                }
            }
            wp = (*wp).w_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn op_formatexpr(mut oap: *mut oparg_T) {
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if fex_format(
        (*oap).start.lnum,
        (*oap).line_count as ::core::ffi::c_long,
        NUL,
    ) != 0 as ::core::ffi::c_int
    {
        op_format(oap, false_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn fex_format(
    mut lnum: linenr_T,
    mut count: ::core::ffi::c_long,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut use_sandbox: bool =
        was_set_insecurely(curwin, kOptFormatexpr, OPT_LOCAL as ::core::ffi::c_int) != 0;
    let save_sctx: sctx_T = current_sctx;
    set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
    set_vim_var_nr(VV_COUNT, count as varnumber_T);
    set_vim_var_char(c);
    let mut fex: *mut ::core::ffi::c_char = xstrdup((*curbuf).b_p_fex);
    current_sctx = (*curbuf).b_p_script_ctx[kBufOptFormatexpr as ::core::ffi::c_int as usize];
    if use_sandbox {
        sandbox += 1;
    }
    let mut r: ::core::ffi::c_int = eval_to_number(fex, true_0 != 0) as ::core::ffi::c_int;
    if use_sandbox {
        sandbox -= 1;
    }
    set_vim_var_string(
        VV_CHAR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    xfree(fex as *mut ::core::ffi::c_void);
    current_sctx = save_sctx;
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn format_lines(mut line_count: linenr_T, mut avoid_fex: bool) {
    let mut is_not_par: bool = false;
    let mut next_is_not_par: bool = false;
    let mut is_end_par: bool = false;
    let mut prev_is_end_par: bool = false_0 != 0;
    let mut next_is_start_par: bool = false_0 != 0;
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_len: ::core::ffi::c_int = 0;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut advance: bool = true_0 != 0;
    let mut second_indent: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut first_par_line: bool = true_0 != 0;
    let mut smd_save: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_long = 0;
    let mut need_set_indent: bool = true_0 != 0;
    let mut first_line: linenr_T = (*curwin).w_cursor.lnum;
    let mut force_format: bool = false_0 != 0;
    let old_State: ::core::ffi::c_int = State;
    let max_len: ::core::ffi::c_int = comp_textwidth(true_0 != 0) * 3 as ::core::ffi::c_int;
    let do_comments: bool = has_format_option(FO_Q_COMS);
    let mut do_comments_list: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let do_second_indent: bool = has_format_option(FO_Q_SECOND);
    let do_number_indent: bool = has_format_option(FO_Q_NUMBER);
    let do_trail_white: bool = has_format_option(FO_WHITE_PAR);
    if (*curwin).w_cursor.lnum > 1 as linenr_T {
        is_not_par = fmt_check_par(
            (*curwin).w_cursor.lnum - 1 as linenr_T,
            &raw mut leader_len,
            &raw mut leader_flags,
            do_comments,
        ) != 0;
    } else {
        is_not_par = true_0 != 0;
    }
    next_is_not_par = fmt_check_par(
        (*curwin).w_cursor.lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0;
    is_end_par =
        is_not_par as ::core::ffi::c_int != 0 || next_is_not_par as ::core::ffi::c_int != 0;
    if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
        is_end_par = !ends_in_white((*curwin).w_cursor.lnum - 1 as linenr_T);
    }
    (*curwin).w_cursor.lnum -= 1;
    count = line_count as ::core::ffi::c_long;
    while count != 0 as ::core::ffi::c_long && !got_int {
        if advance {
            (*curwin).w_cursor.lnum += 1;
            prev_is_end_par = is_end_par;
            is_not_par = next_is_not_par;
            leader_len = next_leader_len;
            leader_flags = next_leader_flags;
        }
        if count == 1 as ::core::ffi::c_long
            || (*curwin).w_cursor.lnum == (*curbuf).b_ml.ml_line_count
        {
            next_is_not_par = true_0 != 0;
            next_leader_len = 0 as ::core::ffi::c_int;
            next_leader_flags = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            next_is_not_par = fmt_check_par(
                (*curwin).w_cursor.lnum + 1 as linenr_T,
                &raw mut next_leader_len,
                &raw mut next_leader_flags,
                do_comments,
            ) != 0;
            if do_number_indent {
                next_is_start_par = get_number_indent((*curwin).w_cursor.lnum + 1 as linenr_T)
                    > 0 as ::core::ffi::c_int;
            }
        }
        advance = true_0 != 0;
        is_end_par = is_not_par as ::core::ffi::c_int != 0
            || next_is_not_par as ::core::ffi::c_int != 0
            || next_is_start_par as ::core::ffi::c_int != 0;
        if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
            is_end_par = !ends_in_white((*curwin).w_cursor.lnum);
        }
        if is_not_par {
            if line_count < 0 as linenr_T {
                break;
            }
        } else {
            if first_par_line as ::core::ffi::c_int != 0
                && (do_second_indent as ::core::ffi::c_int != 0
                    || do_number_indent as ::core::ffi::c_int != 0)
                && prev_is_end_par as ::core::ffi::c_int != 0
                && (*curwin).w_cursor.lnum < (*curbuf).b_ml.ml_line_count
            {
                if do_second_indent as ::core::ffi::c_int != 0
                    && !(*ml_get((*curwin).w_cursor.lnum + 1 as linenr_T) as ::core::ffi::c_int
                        == NUL)
                {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent = get_indent_lnum((*curwin).w_cursor.lnum + 1 as linenr_T);
                    } else {
                        second_indent = next_leader_len;
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                } else if do_number_indent {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent = get_number_indent((*curwin).w_cursor.lnum);
                    } else {
                        second_indent = get_number_indent((*curwin).w_cursor.lnum);
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                }
            }
            if (*curwin).w_cursor.lnum >= (*curbuf).b_ml.ml_line_count
                || !same_leader(
                    (*curwin).w_cursor.lnum,
                    leader_len,
                    leader_flags,
                    next_leader_len,
                    next_leader_flags,
                )
            {
                if next_leader_flags.is_null()
                    || strncmp(
                        next_leader_flags,
                        b"://\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    || check_linecomment(get_cursor_line_ptr()) == MAXCOL as ::core::ffi::c_int
                {
                    is_end_par = true_0 != 0;
                }
            }
            if is_end_par as ::core::ffi::c_int != 0 || force_format as ::core::ffi::c_int != 0 {
                if need_set_indent {
                    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*curwin).w_cursor.lnum == first_line {
                        indent = get_indent();
                    } else if (*curbuf).b_p_lisp != 0 {
                        indent = get_lisp_indent();
                    } else if cindent_on() {
                        indent = if *(*curbuf).b_p_inde as ::core::ffi::c_int != NUL {
                            get_expr_indent()
                        } else {
                            get_c_indent()
                        };
                    } else {
                        indent = get_indent();
                    }
                    set_indent(indent, SIN_CHANGED as ::core::ffi::c_int);
                }
                State = MODE_NORMAL as ::core::ffi::c_int;
                coladvance(curwin, MAXCOL as ::core::ffi::c_int);
                while (*curwin).w_cursor.col != 0
                    && ascii_isspace(gchar_cursor()) as ::core::ffi::c_int != 0
                {
                    dec_cursor();
                }
                State = MODE_INSERT as ::core::ffi::c_int;
                smd_save = p_smd;
                p_smd = false_0;
                insertchar(
                    NUL,
                    INSCHAR_FORMAT as ::core::ffi::c_int
                        + (if do_comments as ::core::ffi::c_int != 0 {
                            INSCHAR_DO_COM as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if do_comments as ::core::ffi::c_int != 0 && do_comments_list != 0 {
                            INSCHAR_COM_LIST as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if avoid_fex as ::core::ffi::c_int != 0 {
                            INSCHAR_NO_FEX as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    second_indent,
                );
                State = old_State;
                p_smd = smd_save;
                ui_cursor_shape();
                second_indent = -1 as ::core::ffi::c_int;
                need_set_indent = is_end_par;
                if is_end_par {
                    if line_count < 0 as linenr_T {
                        break;
                    }
                    first_par_line = true_0 != 0;
                }
                force_format = false_0 != 0;
            }
            if !is_end_par {
                advance = false_0 != 0;
                (*curwin).w_cursor.lnum += 1;
                (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                if line_count < 0 as linenr_T && u_save_cursor() == FAIL {
                    break;
                }
                if next_leader_len > 0 as ::core::ffi::c_int {
                    del_bytes(next_leader_len as colnr_T, false_0 != 0, false_0 != 0);
                    mark_col_adjust(
                        (*curwin).w_cursor.lnum,
                        0 as colnr_T,
                        0 as linenr_T,
                        -(next_leader_len as colnr_T),
                        0 as ::core::ffi::c_int,
                    );
                } else if second_indent > 0 as ::core::ffi::c_int {
                    let mut indent_0: ::core::ffi::c_int =
                        getwhitecols_curline() as ::core::ffi::c_int;
                    if indent_0 > 0 as ::core::ffi::c_int {
                        del_bytes(indent_0 as colnr_T, false_0 != 0, false_0 != 0);
                        mark_col_adjust(
                            (*curwin).w_cursor.lnum,
                            0 as colnr_T,
                            0 as linenr_T,
                            -(indent_0 as colnr_T),
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
                (*curwin).w_cursor.lnum -= 1;
                if do_join(
                    2 as size_t,
                    true_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                ) == FAIL
                {
                    beep_flush();
                    break;
                } else {
                    first_par_line = false_0 != 0;
                    force_format = get_cursor_line_len() > max_len;
                }
            }
        }
        line_breakcheck();
        count -= 1;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
