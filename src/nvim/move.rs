extern "C" {
    pub type MsgpackRpcRequestHandler;
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
    fn labs(__x: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor_0: bool) -> bool;
    fn win_lines_concealed(wp: *mut win_T) -> bool;
    fn diff_get_corresponding_line(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T;
    fn redrawing() -> bool;
    fn win_scroll_lines(wp: *mut win_T, row: ::core::ffi::c_int, line_count: ::core::ffi::c_int);
    fn number_width(wp: *mut win_T) -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn redrawWinline(wp: *mut win_T, lnum: linenr_T);
    fn conceal_cursor_line(wp: *const win_T) -> bool;
    fn win_cursorline_standout(wp: *const win_T) -> bool;
    fn beginline(flags: ::core::ffi::c_int);
    fn cursor_up_inner(wp: *mut win_T, n: linenr_T, skip_conceal: bool);
    fn cursor_up(n: linenr_T, upd_topline: bool) -> ::core::ffi::c_int;
    fn cursor_down_inner(wp: *mut win_T, n: ::core::ffi::c_int, skip_conceal: bool);
    fn cursor_down(n: ::core::ffi::c_int, upd_topline: bool) -> ::core::ffi::c_int;
    static e_invalid_line_number_nr: [::core::ffi::c_char; 0];
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_number_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn foldAdjustCursor(wp: *mut win_T);
    fn beep_flush();
    static mut Rows: ::core::ffi::c_int;
    static mut dollar_vcol: colnr_T;
    static mut mouse_dragging: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut lastwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual_active: bool;
    static mut VIsual_select: bool;
    static mut restart_edit: ::core::ffi::c_int;
    static mut cmdwin_win: *mut win_T;
    static mut skip_update_topline: bool;
    static mut default_grid: ScreenGrid;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_adjust_cursor();
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn vcol2col(wp: *mut win_T, lnum: linenr_T, vcol: colnr_T, coladdp: *mut colnr_T) -> colnr_T;
    fn nv_screengo(
        oap: *mut oparg_T,
        dir: ::core::ffi::c_int,
        dist: ::core::ffi::c_int,
        skip_conceal: bool,
    ) -> bool;
    fn nv_g_home_m_cmd(cap: *mut cmdarg_T);
    fn get_showbreak_value(win: *mut win_T) -> *mut ::core::ffi::c_char;
    fn get_scrolloff_value(wp: *mut win_T) -> int64_t;
    fn get_sidescrolloff_value(wp: *mut win_T) -> int64_t;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_sj: OptInt;
    static mut p_so: OptInt;
    static mut p_ss: OptInt;
    static mut p_sol: ::core::ffi::c_int;
    static mut p_window: OptInt;
    fn linetabsize_eol(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn win_may_fill(wp: *mut win_T) -> bool;
    fn win_get_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn plines_win(wp: *mut win_T, lnum: linenr_T, limit_winheight: bool) -> ::core::ffi::c_int;
    fn plines_win_nofill(
        wp: *mut win_T,
        lnum: linenr_T,
        limit_winheight: bool,
    ) -> ::core::ffi::c_int;
    fn plines_win_full(
        wp: *mut win_T,
        lnum: linenr_T,
        nextp: *mut linenr_T,
        foldedp: *mut bool,
        cache: bool,
        limit_winheight: bool,
    ) -> ::core::ffi::c_int;
    fn plines_m_win(
        wp: *mut win_T,
        first: linenr_T,
        last: linenr_T,
        max: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn win_fdccol_count(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_check_anchored_floats(win: *mut win_T);
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type proftime_T = uint64_t;
pub type OptInt = int64_t;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed_0 = 2;
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
    pub b_signcols: C2Rust_Unnamed_4,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_2,
    pub update_callbacks: C2Rust_Unnamed_1,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub data: C2Rust_Unnamed_3,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
pub struct C2Rust_Unnamed_4 {
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
    pub sst_union: C2Rust_Unnamed_5,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub data: C2Rust_Unnamed_6,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub fc_fixvar: [C2Rust_Unnamed_7; 12],
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
pub struct C2Rust_Unnamed_7 {
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_15 = 4;
pub const BL_SOL: C2Rust_Unnamed_15 = 2;
pub const BL_WHITE: C2Rust_Unnamed_15 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdarg_T {
    pub oap: *mut oparg_T,
    pub prechar: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub nchar: ::core::ffi::c_int,
    pub nchar_composing: [::core::ffi::c_char; 32],
    pub nchar_len: ::core::ffi::c_int,
    pub extra_char: ::core::ffi::c_int,
    pub opcount: ::core::ffi::c_int,
    pub count0: ::core::ffi::c_int,
    pub count1: ::core::ffi::c_int,
    pub arg: ::core::ffi::c_int,
    pub retval: ::core::ffi::c_int,
    pub searchbuf: *mut ::core::ffi::c_char,
}
pub const kOptCuloptFlagScreenline: C2Rust_Unnamed_16 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lineoff_T {
    pub lnum: linenr_T,
    pub fill: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptCuloptFlagNumber: C2Rust_Unnamed_16 = 4;
pub const kOptCuloptFlagLine: C2Rust_Unnamed_16 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VALID_CHEIGHT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
unsafe extern "C" fn adjust_plines_for_skipcol(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut w2: ::core::ffi::c_int = width + win_col_off2(wp);
    if (*wp).w_skipcol >= width && w2 > 0 as ::core::ffi::c_int {
        return ((*wp).w_skipcol as ::core::ffi::c_int - width) / w2 + 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn plines_correct_topline(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut nextp: *mut linenr_T,
    mut limit_winheight: bool,
    mut foldedp: *mut bool,
) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int =
        plines_win_full(wp, lnum, nextp, foldedp, true_0 != 0, false_0 != 0);
    if lnum == (*wp).w_topline {
        n -= adjust_plines_for_skipcol(wp);
    }
    if limit_winheight as ::core::ffi::c_int != 0 && n > (*wp).w_view_height {
        return (*wp).w_view_height;
    }
    return n;
}
unsafe extern "C" fn comp_botline(mut wp: *mut win_T) {
    let mut lnum: linenr_T = 0;
    let mut done: ::core::ffi::c_int = 0;
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CROW != 0 {
        lnum = (*wp).w_cursor.lnum;
        done = (*wp).w_cline_row;
    } else {
        lnum = (*wp).w_topline;
        done = 0 as ::core::ffi::c_int;
    }
    while lnum <= (*(*wp).w_buffer).b_ml.ml_line_count {
        let mut last: linenr_T = lnum;
        let mut folded: bool = false;
        let mut n: ::core::ffi::c_int =
            plines_correct_topline(wp, lnum, &raw mut last, true_0 != 0, &raw mut folded);
        if lnum <= (*wp).w_cursor.lnum && last >= (*wp).w_cursor.lnum {
            (*wp).w_cline_row = done;
            (*wp).w_cline_height = n;
            (*wp).w_cline_folded = folded;
            redraw_for_cursorline(wp);
            (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
        }
        if done + n > (*wp).w_view_height {
            break;
        }
        done += n;
        lnum = last;
        lnum += 1;
    }
    (*wp).w_botline = lnum;
    (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
    (*wp).w_viewport_invalid = true_0 != 0;
    set_empty_rows(wp, done);
    win_check_anchored_floats(wp);
}
unsafe extern "C" fn redraw_for_cursorline(mut wp: *mut win_T) {
    if (*wp).w_valid & VALID_CROW != 0 {
        return;
    }
    if (*wp).w_onebuf_opt.wo_rnu != 0 || win_cursorline_standout(wp) as ::core::ffi::c_int != 0 {
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn redraw_for_cursorcolumn(mut wp: *mut win_T) {
    if wp == curwin
        && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
        && conceal_cursor_line(wp) as ::core::ffi::c_int != 0
    {
        redrawWinline(wp, (*wp).w_cursor.lnum);
    }
    if (*wp).w_valid & VALID_VIRTCOL != 0 {
        return;
    }
    if (*wp).w_onebuf_opt.wo_cuc != 0 {
        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    } else if (*wp).w_onebuf_opt.wo_cul != 0
        && (*wp).w_p_culopt_flags as ::core::ffi::c_int
            & kOptCuloptFlagScreenline as ::core::ffi::c_int
            != 0
    {
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
    if VIsual_active as ::core::ffi::c_int != 0 && (*wp).w_buffer == curbuf {
        redraw_buf_later(curbuf, UPD_INVERTED as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_valid_virtcol(mut wp: *mut win_T, mut vcol: colnr_T) {
    (*wp).w_virtcol = vcol;
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid |= VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn sms_marker_overlap(
    mut wp: *mut win_T,
    mut extra2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if extra2 == -1 as ::core::ffi::c_int {
        extra2 = win_col_off(wp) - win_col_off2(wp);
    }
    if *get_showbreak_value(wp) as ::core::ffi::c_int != NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return if extra2 > 3 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int - extra2
    };
}
unsafe extern "C" fn skipcol_from_plines(
    mut wp: *mut win_T,
    mut plines_off: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut skipcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if plines_off > 0 as ::core::ffi::c_int {
        skipcol += width1;
    }
    if plines_off > 1 as ::core::ffi::c_int {
        skipcol += (width1 + win_col_off2(wp)) * (plines_off - 1 as ::core::ffi::c_int);
    }
    return skipcol;
}
unsafe extern "C" fn reset_skipcol(mut wp: *mut win_T) {
    if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        return;
    }
    (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn update_topline(mut wp: *mut win_T) {
    let mut check_botline: bool = false_0 != 0;
    let mut so_ptr: *mut OptInt = if (*wp).w_onebuf_opt.wo_so >= 0 as OptInt {
        &raw mut (*wp).w_onebuf_opt.wo_so
    } else {
        &raw mut p_so
    };
    let mut save_so: OptInt = *so_ptr;
    if skip_update_topline {
        return;
    }
    if default_grid.chars.is_null() || (*wp).w_view_height == 0 as ::core::ffi::c_int {
        check_cursor_lnum(wp);
        (*wp).w_topline = (*wp).w_cursor.lnum;
        (*wp).w_botline = (*wp).w_topline;
        (*wp).w_viewport_invalid = true_0 != 0;
        (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
        return;
    }
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_TOPLINE != 0 {
        return;
    }
    if mouse_dragging > 0 as ::core::ffi::c_int {
        *so_ptr = (mouse_dragging - 1 as ::core::ffi::c_int) as OptInt;
    }
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
    if buf_is_empty((*wp).w_buffer) {
        if (*wp).w_topline != 1 as linenr_T {
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        }
        (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        (*wp).w_viewport_invalid = true_0 != 0;
        (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
    } else {
        let mut check_topline: bool = false_0 != 0;
        if (*wp).w_topline > 1 as linenr_T || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
            if (*wp).w_cursor.lnum < (*wp).w_topline {
                check_topline = true_0 != 0;
            } else if check_top_offset(wp) {
                check_topline = true_0 != 0;
            } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int
                && (*wp).w_cursor.lnum == (*wp).w_topline
            {
                let mut vcol: colnr_T = 0;
                getvvcol(
                    wp,
                    &raw mut (*wp).w_cursor,
                    &raw mut vcol,
                    ::core::ptr::null_mut::<colnr_T>(),
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                let mut overlap: ::core::ffi::c_int =
                    sms_marker_overlap(wp, -1 as ::core::ffi::c_int);
                if (*wp).w_skipcol as ::core::ffi::c_int + overlap > vcol {
                    check_topline = true_0 != 0;
                }
            }
        }
        if !check_topline && (*wp).w_topfill > win_get_fill(wp, (*wp).w_topline) {
            check_topline = true_0 != 0;
        }
        if check_topline {
            let mut halfheight: ::core::ffi::c_int =
                (*wp).w_view_height / 2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            if halfheight < 2 as ::core::ffi::c_int {
                halfheight = 2 as ::core::ffi::c_int;
            }
            let mut n: int64_t = 0;
            if win_lines_concealed(wp) {
                n = 0 as int64_t;
                let mut lnum: linenr_T = (*wp).w_cursor.lnum;
                while (lnum as OptInt) < (*wp).w_topline as OptInt + *so_ptr {
                    '_c2rust_label: {
                        if !(*wp).w_buffer.is_null() {
                        } else {
                            __assert_fail(
                                b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                338 as ::core::ffi::c_uint,
                                b"void update_topline(win_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count || {
                        n += !decor_conceal_line(wp, lnum as ::core::ffi::c_int, false_0 != 0)
                            as ::core::ffi::c_int as int64_t;
                        n >= halfheight as int64_t
                    } {
                        break;
                    }
                    hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                    lnum += 1;
                }
            } else {
                n = ((*wp).w_topline as OptInt + *so_ptr - (*wp).w_cursor.lnum as OptInt)
                    as int64_t;
            }
            if n >= halfheight as int64_t {
                scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
            } else {
                scroll_cursor_top(wp, scrolljump_value(wp), false_0);
                check_botline = true_0 != 0;
            }
        } else {
            hasFolding(
                wp,
                (*wp).w_topline,
                &raw mut (*wp).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            check_botline = true_0 != 0;
        }
    }
    if check_botline {
        if (*wp).w_valid & VALID_BOTLINE_AP == 0 {
            validate_botline_win(wp);
        }
        '_c2rust_label_0: {
            if !(*wp).w_buffer.is_null() {
            } else {
                __assert_fail(
                    b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    376 as ::core::ffi::c_uint,
                    b"void update_topline(win_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            if (*wp).w_cursor.lnum < (*wp).w_botline {
                if (*wp).w_cursor.lnum as OptInt >= (*wp).w_botline as OptInt - *so_ptr
                    || win_lines_concealed(wp) as ::core::ffi::c_int != 0
                {
                    let mut loff: lineoff_T = lineoff_T {
                        lnum: 0,
                        fill: 0,
                        height: 0,
                    };
                    let mut n_0: ::core::ffi::c_int = (*wp).w_empty_rows;
                    loff.lnum = (*wp).w_cursor.lnum;
                    hasFolding(
                        wp,
                        loff.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut loff.lnum,
                    );
                    loff.fill = 0 as ::core::ffi::c_int;
                    n_0 += (*wp).w_filler_rows;
                    loff.height = 0 as ::core::ffi::c_int;
                    while loff.lnum < (*wp).w_botline
                        && ((loff.lnum + 1 as linenr_T) < (*wp).w_botline
                            || loff.fill == 0 as ::core::ffi::c_int)
                    {
                        n_0 += loff.height;
                        if n_0 as OptInt >= *so_ptr {
                            break;
                        }
                        botline_forw(wp, &raw mut loff);
                    }
                    if n_0 as OptInt >= *so_ptr {
                        check_botline = false_0 != 0;
                    }
                } else {
                    check_botline = false_0 != 0;
                }
            }
            if check_botline {
                let mut n_1: int64_t = 0 as int64_t;
                if win_lines_concealed(wp) {
                    let mut lnum_0: linenr_T = (*wp).w_cursor.lnum;
                    while (lnum_0 as OptInt) >= (*wp).w_botline as OptInt - *so_ptr {
                        if lnum_0 <= 0 as linenr_T || {
                            n_1 += !decor_conceal_line(
                                wp,
                                lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                false_0 != 0,
                            ) as ::core::ffi::c_int as int64_t;
                            n_1 > ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t
                        } {
                            break;
                        }
                        hasFolding(
                            wp,
                            lnum_0,
                            &raw mut lnum_0,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        lnum_0 -= 1;
                    }
                } else {
                    n_1 = (((*wp).w_cursor.lnum - (*wp).w_botline + 1 as linenr_T) as OptInt
                        + *so_ptr) as int64_t;
                }
                if n_1 <= ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t {
                    scroll_cursor_bot(wp, scrolljump_value(wp), false_0 != 0);
                } else {
                    scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
                }
            }
        }
    }
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
    win_check_anchored_floats(wp);
    if (*wp).w_topline != old_topline || (*wp).w_topfill != old_topfill {
        dollar_vcol = -1 as ::core::ffi::c_int as colnr_T;
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        if (*wp).w_onebuf_opt.wo_sms == 0 {
            reset_skipcol(wp);
        } else if (*wp).w_skipcol != 0 as ::core::ffi::c_int {
            redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
        }
        if (*wp).w_cursor.lnum == (*wp).w_topline {
            validate_cursor(wp);
        }
    }
    *so_ptr = save_so;
}
unsafe extern "C" fn scrolljump_value(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = if p_sj >= 0 as OptInt {
        p_sj as ::core::ffi::c_int
    } else {
        (*wp).w_view_height * -p_sj as ::core::ffi::c_int / 100 as ::core::ffi::c_int
    };
    return result;
}
unsafe extern "C" fn check_top_offset(mut wp: *mut win_T) -> bool {
    let mut so: int64_t = get_scrolloff_value(wp);
    if ((*wp).w_cursor.lnum as int64_t) < (*wp).w_topline as int64_t + so
        || win_lines_concealed(wp) as ::core::ffi::c_int != 0
    {
        let mut loff: lineoff_T = lineoff_T {
            lnum: 0,
            fill: 0,
            height: 0,
        };
        loff.lnum = (*wp).w_cursor.lnum;
        loff.fill = 0 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = (*wp).w_topfill;
        while (n as int64_t) < so {
            topline_back(wp, &raw mut loff);
            if loff.lnum < (*wp).w_topline
                || loff.lnum == (*wp).w_topline && loff.fill > 0 as ::core::ffi::c_int
            {
                break;
            }
            n += loff.height;
        }
        if (n as int64_t) < so {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn update_curswant_force() {
    validate_virtcol(curwin);
    (*curwin).w_curswant = (*curwin).w_virtcol;
    (*curwin).w_set_curswant = false_0;
}
#[no_mangle]
pub unsafe extern "C" fn update_curswant() {
    if (*curwin).w_set_curswant != 0 {
        update_curswant_force();
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_cursor_moved(mut wp: *mut win_T) {
    if (*wp).w_cursor.lnum != (*wp).w_valid_cursor.lnum {
        (*wp).w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CHEIGHT | VALID_CROW | VALID_TOPLINE);
        if wp == curwin
            && (*wp).w_valid_cursor.lnum > 0 as linenr_T
            && (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt
            && !conceal_cursor_line(wp)
            && (decor_conceal_line(
                wp,
                (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0
                || decor_conceal_line(
                    wp,
                    (*wp).w_valid_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0)
        {
            changed_window_setting(wp);
        }
        (*wp).w_valid_cursor = (*wp).w_cursor;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_skipcol = (*wp).w_skipcol;
        (*wp).w_viewport_invalid = true_0 != 0;
    } else if (*wp).w_skipcol != (*wp).w_valid_skipcol {
        (*wp).w_valid &= !(VALID_WROW
            | VALID_WCOL
            | VALID_VIRTCOL
            | VALID_CHEIGHT
            | VALID_CROW
            | VALID_BOTLINE
            | VALID_BOTLINE_AP);
        (*wp).w_valid_cursor = (*wp).w_cursor;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_skipcol = (*wp).w_skipcol;
    } else if (*wp).w_cursor.col != (*wp).w_valid_cursor.col
        || (*wp).w_leftcol != (*wp).w_valid_leftcol
        || (*wp).w_cursor.coladd != (*wp).w_valid_cursor.coladd
    {
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        (*wp).w_valid_cursor.col = (*wp).w_cursor.col;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_cursor.coladd = (*wp).w_cursor.coladd;
        (*wp).w_viewport_invalid = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn changed_window_setting(mut wp: *mut win_T) {
    (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    changed_line_abv_curs_win(wp);
    (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn changed_window_setting_all() {
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            changed_window_setting(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_topline(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut prev_topline: linenr_T = (*wp).w_topline;
    hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
    (*wp).w_botline += lnum - (*wp).w_topline;
    if (*wp).w_botline > (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        (*wp).w_botline = (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T;
    }
    (*wp).w_topline = lnum;
    (*wp).w_topline_was_set = true_0 as ::core::ffi::c_char;
    if lnum != prev_topline {
        (*wp).w_topfill = 0 as ::core::ffi::c_int;
    }
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_TOPLINE);
    redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn changed_cline_bef_curs(mut wp: *mut win_T) {
    (*wp).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn changed_line_abv_curs() {
    (*curwin).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn changed_line_abv_curs_win(mut wp: *mut win_T) {
    (*wp).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn validate_botline_win(mut wp: *mut win_T) {
    if (*wp).w_valid & VALID_BOTLINE == 0 {
        comp_botline(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn invalidate_botline_win(mut wp: *mut win_T) {
    (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP);
}
#[no_mangle]
pub unsafe extern "C" fn approximate_botline_win(mut wp: *mut win_T) {
    (*wp).w_valid &= !VALID_BOTLINE;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_valid(mut wp: *mut win_T) -> ::core::ffi::c_int {
    check_cursor_moved(wp);
    return ((*wp).w_valid & (VALID_WROW | VALID_WCOL) == VALID_WROW | VALID_WCOL)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cursor(mut wp: *mut win_T) {
    check_cursor_lnum(wp);
    check_cursor_moved(wp);
    if (*wp).w_valid & (VALID_WCOL | VALID_WROW) != VALID_WCOL | VALID_WROW {
        curs_columns(wp, true_0);
    }
}
unsafe extern "C" fn curs_rows(mut wp: *mut win_T) {
    let mut all_invalid: bool = !redrawing()
        || (*wp).w_lines_valid == 0 as ::core::ffi::c_int
        || (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum > (*wp).w_topline;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*wp).w_cline_row = 0 as ::core::ffi::c_int;
    let mut lnum: linenr_T = (*wp).w_topline;
    's_111: while lnum < (*wp).w_cursor.lnum {
        let mut valid: bool = false_0 != 0;
        's_11: {
            if !all_invalid && i < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(i as isize)).wl_lnum < lnum
                    || !(*(*wp).w_lines.offset(i as isize)).wl_valid
                {
                    break 's_11;
                } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum == lnum {
                    if !(*(*wp).w_buffer).b_mod_set
                        || (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < (*wp).w_cursor.lnum
                        || (*(*wp).w_buffer).b_mod_top
                            > (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T
                    {
                        valid = true_0 != 0;
                    }
                } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum > lnum {
                    i -= 1;
                }
            }
            if valid as ::core::ffi::c_int != 0
                && (lnum != (*wp).w_topline
                    || (*wp).w_skipcol == 0 as ::core::ffi::c_int && !win_may_fill(wp))
            {
                lnum = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                if lnum > (*wp).w_cursor.lnum {
                    break 's_111;
                }
                (*wp).w_cline_row +=
                    (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
            } else {
                let mut last: linenr_T = lnum;
                let mut folded: bool = false;
                let mut n: ::core::ffi::c_int =
                    plines_correct_topline(wp, lnum, &raw mut last, true_0 != 0, &raw mut folded);
                lnum = last + 1 as linenr_T;
                if lnum
                    + decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        false_0 != 0,
                    ) as linenr_T
                    > (*wp).w_cursor.lnum
                {
                    break 's_111;
                }
                (*wp).w_cline_row += n;
            }
        }
        i += 1;
    }
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CHEIGHT == 0 {
        if all_invalid as ::core::ffi::c_int != 0
            || i == (*wp).w_lines_valid
            || i < (*wp).w_lines_valid
                && (!(*(*wp).w_lines.offset(i as isize)).wl_valid
                    || (*(*wp).w_lines.offset(i as isize)).wl_lnum != (*wp).w_cursor.lnum)
        {
            (*wp).w_cline_height = plines_win_full(
                wp,
                (*wp).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*wp).w_cline_folded,
                true_0 != 0,
                true_0 != 0,
            );
        } else if i > (*wp).w_lines_valid {
            (*wp).w_cline_height = 0 as ::core::ffi::c_int;
            (*wp).w_cline_folded = hasFolding(
                wp,
                (*wp).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                ::core::ptr::null_mut::<linenr_T>(),
            );
        } else {
            (*wp).w_cline_height =
                (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
            (*wp).w_cline_folded = (*(*wp).w_lines.offset(i as isize)).wl_folded;
        }
    }
    redraw_for_cursorline(wp);
    (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
}
#[no_mangle]
pub unsafe extern "C" fn validate_virtcol(mut wp: *mut win_T) {
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_VIRTCOL != 0 {
        return;
    }
    getvvcol(
        wp,
        &raw mut (*wp).w_cursor,
        ::core::ptr::null_mut::<colnr_T>(),
        &raw mut (*wp).w_virtcol,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid |= VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cheight(mut wp: *mut win_T) {
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CHEIGHT != 0 {
        return;
    }
    (*wp).w_cline_height = plines_win_full(
        wp,
        (*wp).w_cursor.lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut (*wp).w_cline_folded,
        true_0 != 0,
        true_0 != 0,
    );
    (*wp).w_valid |= VALID_CHEIGHT;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cursor_col(mut wp: *mut win_T) {
    validate_virtcol(wp);
    if (*wp).w_valid & VALID_WCOL != 0 {
        return;
    }
    let mut col: colnr_T = (*wp).w_virtcol;
    let mut off: colnr_T = win_col_off(wp);
    col += off;
    let mut width: ::core::ffi::c_int =
        (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
    if (*wp).w_onebuf_opt.wo_wrap != 0
        && col >= (*wp).w_view_width
        && width > 0 as ::core::ffi::c_int
    {
        col -= ((col as ::core::ffi::c_int - (*wp).w_view_width) / width + 1 as ::core::ffi::c_int)
            * width;
    }
    if col > (*wp).w_leftcol {
        col -= (*wp).w_leftcol;
    } else {
        col = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*wp).w_wcol = col as ::core::ffi::c_int;
    (*wp).w_valid |= VALID_WCOL;
}
#[no_mangle]
pub unsafe extern "C" fn win_col_off(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return (if (*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
    {
        number_width(wp)
            + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) + (if wp != cmdwin_win {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    }) + win_fdccol_count(wp)
        + (*wp).w_scwidth * SIGN_WIDTH as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn win_col_off2(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if ((*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL)
        && !vim_strchr(p_cpo, CPO_NUMCOL).is_null()
    {
        return number_width(wp)
            + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn curs_columns(mut wp: *mut win_T, mut may_scroll: ::core::ffi::c_int) {
    let mut startcol: colnr_T = 0;
    let mut endcol: colnr_T = 0;
    update_topline(wp);
    if (*wp).w_valid & VALID_CROW == 0 {
        curs_rows(wp);
    }
    if (*wp).w_cline_folded {
        endcol = (*wp).w_leftcol;
        (*wp).w_virtcol = endcol;
        startcol = (*wp).w_virtcol;
    } else {
        getvvcol(
            wp,
            &raw mut (*wp).w_cursor,
            &raw mut startcol,
            &raw mut (*wp).w_virtcol,
            &raw mut endcol,
        );
    }
    if startcol > dollar_vcol {
        dollar_vcol = -1 as ::core::ffi::c_int as colnr_T;
    }
    let mut extra: ::core::ffi::c_int = win_col_off(wp);
    (*wp).w_wcol = (*wp).w_virtcol as ::core::ffi::c_int + extra;
    endcol += extra;
    (*wp).w_wrow = (*wp).w_cline_row;
    let mut n: ::core::ffi::c_int = 0;
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - extra;
    let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_sub_skipcol: bool = false_0 != 0;
    if width1 <= 0 as ::core::ffi::c_int {
        (*wp).w_wcol = (*wp).w_view_width - 1 as ::core::ffi::c_int;
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int;
        } else {
            (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int - (*wp).w_empty_rows;
        }
    } else if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
        width2 = width1 + win_col_off2(wp);
        if (*wp).w_cursor.lnum == (*wp).w_topline
            && (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wp).w_wcol >= (*wp).w_skipcol
        {
            if (*wp).w_skipcol <= width1 {
                (*wp).w_wcol -= width2;
            } else {
                (*wp).w_wcol -= width2
                    * (((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                        + 1 as ::core::ffi::c_int);
            }
            did_sub_skipcol = true_0 != 0;
        }
        if (*wp).w_wcol >= (*wp).w_view_width {
            n = ((*wp).w_wcol - (*wp).w_view_width) / width2 + 1 as ::core::ffi::c_int;
            (*wp).w_wcol -= n * width2;
            (*wp).w_wrow += n;
        }
    } else if may_scroll != 0 && !(*wp).w_cline_folded {
        let mut siso: int64_t = get_sidescrolloff_value(wp);
        let mut off_left: int64_t = (startcol - (*wp).w_leftcol) as int64_t - siso;
        let mut off_right: int64_t = (endcol - (*wp).w_leftcol) as int64_t
            - ((*wp).w_view_width as int64_t - siso)
            + 1 as int64_t;
        if off_left < 0 as int64_t || off_right > 0 as int64_t {
            let mut diff: int64_t = if off_left < 0 as int64_t {
                -off_left
            } else {
                off_right
            };
            let mut new_leftcol: ::core::ffi::c_int = 0;
            if p_ss == 0 as OptInt
                || diff >= (width1 / 2 as ::core::ffi::c_int) as int64_t
                || off_right >= off_left
            {
                new_leftcol = (*wp).w_wcol - extra - width1 / 2 as ::core::ffi::c_int;
            } else {
                if diff < p_ss {
                    '_c2rust_label: {
                        if p_ss <= 2147483647 as OptInt {
                        } else {
                            __assert_fail(
                                b"p_ss <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                903 as ::core::ffi::c_uint,
                                b"void curs_columns(win_T *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    diff = p_ss as int64_t;
                }
                if off_left < 0 as int64_t {
                    new_leftcol =
                        (*wp).w_leftcol as ::core::ffi::c_int - diff as ::core::ffi::c_int;
                } else {
                    new_leftcol =
                        (*wp).w_leftcol as ::core::ffi::c_int + diff as ::core::ffi::c_int;
                }
            }
            new_leftcol = if new_leftcol > 0 as ::core::ffi::c_int {
                new_leftcol
            } else {
                0 as ::core::ffi::c_int
            };
            if new_leftcol != (*wp).w_leftcol {
                (*wp).w_leftcol = new_leftcol as colnr_T;
                win_check_anchored_floats(wp);
                redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            }
        }
        (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
    } else if (*wp).w_wcol > (*wp).w_leftcol {
        (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
    } else {
        (*wp).w_wcol = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_cursor.lnum == (*wp).w_topline {
        (*wp).w_wrow += (*wp).w_topfill;
    } else {
        (*wp).w_wrow += win_get_fill(wp, (*wp).w_cursor.lnum);
    }
    let mut plines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut so: int64_t = get_scrolloff_value(wp);
    let mut prev_skipcol: colnr_T = (*wp).w_skipcol;
    if ((*wp).w_wrow >= (*wp).w_view_height
        || (prev_skipcol > 0 as ::core::ffi::c_int
            || (*wp).w_wrow as int64_t + so >= (*wp).w_view_height as int64_t)
            && {
                plines = plines_win_nofill(wp, (*wp).w_cursor.lnum, false_0 != 0);
                plines - 1 as ::core::ffi::c_int >= (*wp).w_view_height
            })
        && (*wp).w_view_height != 0 as ::core::ffi::c_int
        && (*wp).w_cursor.lnum == (*wp).w_topline
        && width2 > 0 as ::core::ffi::c_int
        && (*wp).w_view_width != 0 as ::core::ffi::c_int
    {
        extra = 0 as ::core::ffi::c_int;
        if (*wp).w_skipcol as int64_t + so * width2 as int64_t > (*wp).w_virtcol as int64_t {
            extra = 1 as ::core::ffi::c_int;
        }
        if plines == 0 as ::core::ffi::c_int {
            plines = plines_win(wp, (*wp).w_cursor.lnum, false_0 != 0);
        }
        plines -= 1;
        if plines as int64_t > (*wp).w_wrow as int64_t + so {
            '_c2rust_label_0: {
                if (*wp).w_wrow as int64_t + so <= 2147483647 as int64_t {
                } else {
                    __assert_fail(
                        b"wp->w_wrow + so <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        964 as ::core::ffi::c_uint,
                        b"void curs_columns(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            n = ((*wp).w_wrow as int64_t + so) as ::core::ffi::c_int;
        } else {
            n = plines;
        }
        if n as int64_t
            >= ((*wp).w_view_height + (*wp).w_skipcol as ::core::ffi::c_int / width2) as int64_t
                - so
        {
            extra += 2 as ::core::ffi::c_int;
        }
        if extra == 3 as ::core::ffi::c_int || (*wp).w_view_height as int64_t <= so * 2 as int64_t {
            n = (*wp).w_virtcol as ::core::ffi::c_int / width2;
            if n > (*wp).w_view_height / 2 as ::core::ffi::c_int {
                n -= (*wp).w_view_height / 2 as ::core::ffi::c_int;
            } else {
                n = 0 as ::core::ffi::c_int;
            }
            if n > plines - (*wp).w_view_height + 1 as ::core::ffi::c_int {
                n = plines - (*wp).w_view_height + 1 as ::core::ffi::c_int;
            }
            (*wp).w_skipcol = (if n > 0 as ::core::ffi::c_int {
                width1 + (n - 1 as ::core::ffi::c_int) * width2
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
        } else if extra == 1 as ::core::ffi::c_int {
            '_c2rust_label_1: {
                if so <= 2147483647 as int64_t {
                } else {
                    __assert_fail(
                        b"so <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        989 as ::core::ffi::c_uint,
                        b"void curs_columns(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            extra = (((*wp).w_skipcol as int64_t + so * width2 as int64_t
                - (*wp).w_virtcol as int64_t
                + width2 as int64_t
                - 1 as int64_t)
                / width2 as int64_t) as ::core::ffi::c_int;
            if extra > 0 as ::core::ffi::c_int {
                if extra * width2 > (*wp).w_skipcol {
                    extra = (*wp).w_skipcol as ::core::ffi::c_int / width2;
                }
                (*wp).w_skipcol -= extra * width2;
            }
        } else if extra == 2 as ::core::ffi::c_int {
            endcol = ((n - (*wp).w_view_height + 1 as ::core::ffi::c_int) * width2) as colnr_T;
            while endcol > (*wp).w_virtcol {
                endcol -= width2;
            }
            (*wp).w_skipcol = if (*wp).w_skipcol > endcol {
                (*wp).w_skipcol
            } else {
                endcol
            };
        }
        if did_sub_skipcol {
            (*wp).w_wrow -= ((*wp).w_skipcol as ::core::ffi::c_int
                - prev_skipcol as ::core::ffi::c_int)
                / width2;
        } else {
            (*wp).w_wrow -= (*wp).w_skipcol as ::core::ffi::c_int / width2;
        }
        if (*wp).w_wrow >= (*wp).w_view_height {
            extra = (*wp).w_wrow - (*wp).w_view_height + 1 as ::core::ffi::c_int;
            (*wp).w_skipcol += extra * width2;
            (*wp).w_wrow -= extra;
        }
        extra =
            (prev_skipcol as ::core::ffi::c_int - (*wp).w_skipcol as ::core::ffi::c_int) / width2;
        if !(*wp).w_grid.target.is_null() {
            win_scroll_lines(wp, 0 as ::core::ffi::c_int, extra);
        }
    } else if (*wp).w_onebuf_opt.wo_sms == 0 {
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    }
    if prev_skipcol != (*wp).w_skipcol {
        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    }
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid_leftcol = (*wp).w_leftcol;
    (*wp).w_valid_skipcol = (*wp).w_skipcol;
    (*wp).w_valid |= VALID_WCOL | VALID_WROW | VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn textpos2screenpos(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut scolp: *mut ::core::ffi::c_int,
    mut ccolp: *mut ::core::ffi::c_int,
    mut ecolp: *mut ::core::ffi::c_int,
    mut local: bool,
) {
    let mut scol: colnr_T = 0 as colnr_T;
    let mut ccol: colnr_T = 0 as colnr_T;
    let mut ecol: colnr_T = 0 as colnr_T;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut coloff: colnr_T = 0 as colnr_T;
    let mut visible_row: bool = false_0 != 0;
    let mut is_folded: bool = false_0 != 0;
    let mut lnum: linenr_T = (*pos).lnum;
    if lnum >= (*wp).w_topline && lnum <= (*wp).w_botline {
        is_folded = hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
        row = plines_m_win(wp, (*wp).w_topline, lnum - 1 as linenr_T, INT_MAX);
        row -= adjust_plines_for_skipcol(wp);
        row += if lnum == (*wp).w_topline {
            (*wp).w_topfill
        } else {
            win_get_fill(wp, lnum)
        };
        visible_row = true_0 != 0;
    } else if !local || lnum < (*wp).w_topline {
        row = 0 as ::core::ffi::c_int;
    } else {
        row = (*wp).w_view_height - 1 as ::core::ffi::c_int;
    }
    let mut existing_row: bool =
        lnum > 0 as linenr_T && lnum <= (*(*wp).w_buffer).b_ml.ml_line_count;
    if (local as ::core::ffi::c_int != 0 || visible_row as ::core::ffi::c_int != 0)
        && existing_row as ::core::ffi::c_int != 0
    {
        let off: colnr_T = win_col_off(wp);
        if is_folded {
            row += (if local as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                (*wp).w_winrow + (*wp).w_winrow_off
            }) + 1 as ::core::ffi::c_int;
            coloff = (if local as ::core::ffi::c_int != 0 {
                0 as colnr_T
            } else {
                (*wp).w_wincol as colnr_T + (*wp).w_wincol_off as colnr_T
            }) + 1 as colnr_T
                + off;
        } else {
            '_c2rust_label: {
                if lnum == (*pos).lnum {
                } else {
                    __assert_fail(
                        b"lnum == pos->lnum\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1087 as ::core::ffi::c_uint,
                        b"void textpos2screenpos(win_T *, pos_T *, int *, int *, int *, int *, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            getvcol(wp, pos, &raw mut scol, &raw mut ccol, &raw mut ecol);
            let mut col: colnr_T = scol;
            col += off;
            let mut width: ::core::ffi::c_int =
                (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
            if (*wp).w_onebuf_opt.wo_wrap != 0
                && col >= (*wp).w_view_width
                && width > 0 as ::core::ffi::c_int
            {
                let mut rowoff: ::core::ffi::c_int = if visible_row as ::core::ffi::c_int != 0 {
                    (col as ::core::ffi::c_int - (*wp).w_view_width) / width
                        + 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                col -= rowoff * width;
                row += rowoff;
            }
            col -= (*wp).w_leftcol;
            if col >= 0 as ::core::ffi::c_int
                && col < (*wp).w_view_width
                && row >= 0 as ::core::ffi::c_int
                && row < (*wp).w_view_height
            {
                coloff = (col as ::core::ffi::c_int - scol as ::core::ffi::c_int
                    + (if local as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*wp).w_wincol + (*wp).w_wincol_off
                    })
                    + 1 as ::core::ffi::c_int) as colnr_T;
                row += (if local as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    (*wp).w_winrow + (*wp).w_winrow_off
                }) + 1 as ::core::ffi::c_int;
            } else {
                ecol = 0 as ::core::ffi::c_int as colnr_T;
                ccol = ecol;
                scol = ccol;
                if local {
                    coloff = (if col < 0 as ::core::ffi::c_int {
                        -1 as ::core::ffi::c_int
                    } else {
                        (*wp).w_view_width + 1 as ::core::ffi::c_int
                    }) as colnr_T;
                } else {
                    row = 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    *rowp = row;
    *scolp = (scol + coloff) as ::core::ffi::c_int;
    *ccolp = (ccol + coloff) as ::core::ffi::c_int;
    *ecolp = (ecol + coloff) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_screenpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    let mut pos: pos_T = pos_T {
        lnum: tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as linenr_T,
        col: tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as colnr_T
            - 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    if pos.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            pos.lnum,
        );
        return;
    }
    pos.col = (if pos.col > 0 as ::core::ffi::c_int {
        pos.col as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as colnr_T;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ccol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ecol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    textpos2screenpos(
        wp,
        &raw mut pos,
        &raw mut row,
        &raw mut scol,
        &raw mut ccol,
        &raw mut ecol,
        false_0 != 0,
    );
    tv_dict_add_nr(
        dict,
        b"row\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        row as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        scol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"curscol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ccol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"endcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ecol as varnumber_T,
    );
}
unsafe extern "C" fn virtcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut offset: ::core::ffi::c_int = vcol2col(
        wp,
        lnum,
        vcol as colnr_T - 1 as colnr_T,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut p: *mut ::core::ffi::c_char = line.offset(offset as isize);
    if *p as ::core::ffi::c_int == NUL {
        if p == line {
            return 0 as ::core::ffi::c_int;
        }
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return (p.offset_from(line) + 1 as isize) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_virtcol2col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_number_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut lnum: linenr_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as linenr_T;
    if error as ::core::ffi::c_int != 0
        || lnum < 0 as linenr_T
        || lnum > (*(*wp).w_buffer).b_ml.ml_line_count
    {
        return;
    }
    let mut screencol: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as ::core::ffi::c_int;
    if error as ::core::ffi::c_int != 0 || screencol < 0 as ::core::ffi::c_int {
        return;
    }
    (*rettv).vval.v_number = virtcol2col(wp, lnum, screencol) as varnumber_T;
}
unsafe extern "C" fn cursor_correct_sms(mut wp: *mut win_T) {
    if (*wp).w_onebuf_opt.wo_sms == 0
        || (*wp).w_onebuf_opt.wo_wrap == 0
        || (*wp).w_cursor.lnum != (*wp).w_topline
    {
        return;
    }
    let mut so: int64_t = get_scrolloff_value(wp);
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    let mut so_cols: int64_t = if so == 0 as int64_t {
        0 as int64_t
    } else {
        width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
    };
    let mut space_cols: ::core::ffi::c_int =
        ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2;
    let mut size: ::core::ffi::c_int = if so == 0 as int64_t {
        0 as ::core::ffi::c_int
    } else {
        linetabsize_eol(wp, (*wp).w_topline)
    };
    if (*wp).w_topline == 1 as linenr_T && (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        so_cols = 0 as int64_t;
    } else if so_cols > (space_cols / 2 as ::core::ffi::c_int) as int64_t {
        so_cols = (space_cols / 2 as ::core::ffi::c_int) as int64_t;
    }
    while so_cols > size as int64_t
        && so_cols - width2 as int64_t >= width1 as int64_t
        && width1 > 0 as ::core::ffi::c_int
    {
        so_cols -= width2 as int64_t;
    }
    if so_cols >= width1 as int64_t && so_cols > size as int64_t {
        so_cols -= width1 as int64_t;
    }
    let mut overlap: ::core::ffi::c_int = if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        sms_marker_overlap(wp, (*wp).w_view_width - width2)
    };
    let mut top: int64_t = (*wp).w_skipcol as int64_t
        + (if so_cols != 0 as int64_t {
            so_cols
        } else {
            overlap as int64_t
        });
    let mut bot: int64_t = ((*wp).w_skipcol as ::core::ffi::c_int
        + width1
        + ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2)
        as int64_t
        - so_cols;
    validate_virtcol(wp);
    let mut col: colnr_T = (*wp).w_virtcol;
    if (col as int64_t) < top {
        if col < width1 {
            col += width1;
        }
        while width2 > 0 as ::core::ffi::c_int && (col as int64_t) < top {
            col += width2;
        }
    } else {
        while width2 > 0 as ::core::ffi::c_int && col as int64_t >= bot {
            col -= width2;
        }
    }
    if col != (*wp).w_virtcol {
        (*wp).w_curswant = col;
        let mut rc: ::core::ffi::c_int = coladvance(wp, (*wp).w_curswant);
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        if rc == FAIL
            && (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wp).w_cursor.lnum < (*(*wp).w_buffer).b_ml.ml_line_count
        {
            validate_virtcol(wp);
            if (*wp).w_virtcol < (*wp).w_skipcol as ::core::ffi::c_int + overlap {
                (*wp).w_cursor.lnum += 1;
                (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_valid &= !VALID_VIRTCOL;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn scroll_redraw(mut up: ::core::ffi::c_int, mut count: linenr_T) {
    let mut prev_topline: linenr_T = (*curwin).w_topline;
    let mut prev_skipcol: ::core::ffi::c_int = (*curwin).w_skipcol as ::core::ffi::c_int;
    let mut prev_topfill: ::core::ffi::c_int = (*curwin).w_topfill;
    let mut prev_lnum: linenr_T = (*curwin).w_cursor.lnum;
    let mut moved: bool = if up != 0 {
        scrollup(curwin, count, true_0 != 0) as ::core::ffi::c_int
    } else {
        scrolldown(curwin, count, true_0) as ::core::ffi::c_int
    } != 0;
    if get_scrolloff_value(curwin) > 0 as int64_t {
        cursor_correct(curwin);
        check_cursor_moved(curwin);
        (*curwin).w_valid |= VALID_TOPLINE;
        while (*curwin).w_topline == prev_topline
            && (*curwin).w_skipcol == prev_skipcol
            && (*curwin).w_topfill == prev_topfill
        {
            if up != 0 {
                if (*curwin).w_cursor.lnum > prev_lnum
                    || cursor_down(1 as ::core::ffi::c_int, false_0 != 0) == FAIL
                {
                    break;
                }
            } else if (*curwin).w_cursor.lnum < prev_lnum
                || prev_topline as ::core::ffi::c_long == 1 as ::core::ffi::c_long
                || cursor_up(1 as linenr_T, false_0 != 0) == FAIL
            {
                break;
            }
            check_cursor_moved(curwin);
            (*curwin).w_valid |= VALID_TOPLINE;
        }
    }
    if moved {
        (*curwin).w_viewport_invalid = true_0 != 0;
    }
    cursor_correct_sms(curwin);
    if (*curwin).w_cursor.lnum != prev_lnum {
        coladvance(curwin, (*curwin).w_curswant);
    }
    redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn scrolldown(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: ::core::ffi::c_int,
) -> bool {
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms {
        width1 = (*wp).w_view_width - win_col_off(wp);
        width2 = width1 + win_col_off2(wp);
    }
    hasFolding(
        wp,
        (*wp).w_topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
    validate_cursor(wp);
    let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
    while todo > 0 as ::core::ffi::c_int {
        let mut can_fill: bool = (*wp).w_topfill < (*wp).w_view_height - 1 as ::core::ffi::c_int
            && (*wp).w_topfill < win_get_fill(wp, (*wp).w_topline);
        if (*wp).w_topline == 1 as linenr_T && !can_fill && (!do_sms || (*wp).w_skipcol < width1) {
            break;
        }
        if do_sms as ::core::ffi::c_int != 0 && (*wp).w_skipcol >= width1 {
            if (*wp).w_skipcol >= width1 + width2 {
                (*wp).w_skipcol -= width2;
            } else {
                (*wp).w_skipcol -= width1;
            }
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            done += 1;
        } else if can_fill {
            (*wp).w_topfill += 1;
            done += 1;
        } else {
            (*wp).w_topline -= 1;
            (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
            (*wp).w_topfill = 0 as ::core::ffi::c_int;
            let mut first: linenr_T = 0;
            if hasFolding(
                wp,
                (*wp).w_topline,
                &raw mut first,
                ::core::ptr::null_mut::<linenr_T>(),
            ) {
                done += !decor_conceal_line(
                    wp,
                    first as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
                if byfold == 0 {
                    todo -= ((*wp).w_topline - first - 1 as linenr_T) as ::core::ffi::c_int;
                }
                (*wp).w_botline -= (*wp).w_topline - first;
                (*wp).w_topline = first;
            } else if decor_conceal_line(
                wp,
                (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) {
                todo += 1;
            } else if do_sms {
                let mut size: ::core::ffi::c_int = linetabsize_eol(wp, (*wp).w_topline);
                if size > width1 {
                    (*wp).w_skipcol = width1 as colnr_T;
                    size -= width1;
                    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
                }
                while size > width2 {
                    (*wp).w_skipcol += width2;
                    size -= width2;
                }
                done += 1;
            } else {
                done += plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
            }
        }
        (*wp).w_botline -= 1;
        invalidate_botline_win(wp);
        todo -= 1;
    }
    while (*wp).w_topline > 1 as linenr_T
        && decor_conceal_line(
            wp,
            (*wp).w_topline as ::core::ffi::c_int - 2 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        (*wp).w_topline -= 1;
        hasFolding(
            wp,
            (*wp).w_topline,
            &raw mut (*wp).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    }
    (*wp).w_wrow += done;
    (*wp).w_cline_row += done;
    if (*wp).w_cursor.lnum == (*wp).w_topline {
        (*wp).w_cline_row = 0 as ::core::ffi::c_int;
    }
    check_topfill(wp, true_0 != 0);
    let mut wrow: ::core::ffi::c_int = (*wp).w_wrow;
    if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
        validate_virtcol(wp);
        validate_cheight(wp);
        wrow += (*wp).w_cline_height
            - 1 as ::core::ffi::c_int
            - (*wp).w_virtcol as ::core::ffi::c_int / (*wp).w_view_width;
    }
    let mut moved: bool = false_0 != 0;
    while wrow >= (*wp).w_view_height && (*wp).w_cursor.lnum > 1 as linenr_T {
        let mut first_0: linenr_T = 0;
        if hasFolding(
            wp,
            (*wp).w_cursor.lnum,
            &raw mut first_0,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            wrow -= !decor_conceal_line(
                wp,
                (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
            (*wp).w_cursor.lnum = if first_0 - 1 as linenr_T > 1 as linenr_T {
                first_0 - 1 as linenr_T
            } else {
                1 as linenr_T
            };
        } else {
            let c2rust_fresh0 = (*wp).w_cursor.lnum;
            (*wp).w_cursor.lnum = (*wp).w_cursor.lnum - 1;
            wrow -= plines_win(wp, c2rust_fresh0, true_0 != 0);
        }
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        moved = true_0 != 0;
    }
    if moved {
        foldAdjustCursor(wp);
        coladvance(wp, (*wp).w_curswant);
    }
    (*wp).w_cursor.lnum = if (*wp).w_cursor.lnum > (*wp).w_topline {
        (*wp).w_cursor.lnum
    } else {
        (*wp).w_topline
    };
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn scrollup(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: bool,
) -> bool {
    let mut topline: linenr_T = (*wp).w_topline;
    let mut botline: linenr_T = (*wp).w_botline;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms as ::core::ffi::c_int != 0
        || byfold as ::core::ffi::c_int != 0 && win_lines_concealed(wp) as ::core::ffi::c_int != 0
        || win_may_fill(wp) as ::core::ffi::c_int != 0
    {
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let prev_skipcol: colnr_T = (*wp).w_skipcol;
        if do_sms {
            size = linetabsize_eol(wp, (*wp).w_topline);
        }
        let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
        while todo > 0 as ::core::ffi::c_int {
            todo += decor_conceal_line(
                wp,
                (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
            if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                (*wp).w_topfill -= 1;
            } else {
                let mut lnum: linenr_T = (*wp).w_topline;
                if byfold {
                    hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                }
                if lnum == (*wp).w_topline && do_sms as ::core::ffi::c_int != 0 {
                    let mut add: ::core::ffi::c_int = if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                        width2
                    } else {
                        width1
                    };
                    (*wp).w_skipcol += add;
                    if (*wp).w_skipcol >= size {
                        if lnum == (*(*wp).w_buffer).b_ml.ml_line_count {
                            (*wp).w_skipcol -= add;
                            break;
                        } else {
                            lnum += 1;
                        }
                    }
                } else {
                    if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
                        break;
                    }
                    lnum += 1;
                }
                if lnum > (*wp).w_topline {
                    (*wp).w_botline += lnum - (*wp).w_topline;
                    (*wp).w_topline = lnum;
                    (*wp).w_topfill = win_get_fill(wp, lnum);
                    (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                    if todo > 1 as ::core::ffi::c_int && do_sms as ::core::ffi::c_int != 0 {
                        size = linetabsize_eol(wp, (*wp).w_topline);
                    }
                }
            }
            todo -= 1;
        }
        if prev_skipcol > 0 as ::core::ffi::c_int || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        }
    } else {
        (*wp).w_topline += line_count;
        (*wp).w_botline += line_count;
    }
    (*wp).w_topline = if (*wp).w_topline < (*(*wp).w_buffer).b_ml.ml_line_count {
        (*wp).w_topline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    (*wp).w_botline = if (*wp).w_botline < (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        (*wp).w_botline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
    };
    check_topfill(wp, false_0 != 0);
    hasFolding(
        wp,
        (*wp).w_topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    if (*wp).w_cursor.lnum < (*wp).w_topline {
        (*wp).w_cursor.lnum = (*wp).w_topline;
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        coladvance(wp, (*wp).w_curswant);
    }
    let mut moved: bool = topline != (*wp).w_topline || botline != (*wp).w_botline;
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn adjust_skipcol() {
    if (*curwin).w_onebuf_opt.wo_wrap == 0
        || (*curwin).w_onebuf_opt.wo_sms == 0
        || (*curwin).w_cursor.lnum != (*curwin).w_topline
    {
        return;
    }
    let mut width1: ::core::ffi::c_int = (*curwin).w_view_width - win_col_off(curwin);
    if width1 <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin);
    let mut so: int64_t = get_scrolloff_value(curwin);
    let mut scrolloff_cols: int64_t = if so == 0 as int64_t {
        0 as int64_t
    } else {
        width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
    };
    let mut scrolled: bool = false_0 != 0;
    validate_cheight(curwin);
    if (*curwin).w_cline_height == (*curwin).w_view_height
        && plines_win(curwin, (*curwin).w_cursor.lnum, false_0 != 0) <= (*curwin).w_view_height
    {
        reset_skipcol(curwin);
        return;
    }
    validate_virtcol(curwin);
    let mut overlap: ::core::ffi::c_int =
        sms_marker_overlap(curwin, (*curwin).w_view_width - width2);
    while (*curwin).w_skipcol > 0 as ::core::ffi::c_int
        && ((*curwin).w_virtcol as int64_t)
            < ((*curwin).w_skipcol as ::core::ffi::c_int + overlap) as int64_t + scrolloff_cols
    {
        if (*curwin).w_skipcol >= width1 + width2 {
            (*curwin).w_skipcol -= width2;
        } else {
            (*curwin).w_skipcol -= width1;
        }
        scrolled = true_0 != 0;
    }
    if scrolled {
        validate_virtcol(curwin);
        redraw_later(curwin, UPD_NOT_VALID as ::core::ffi::c_int);
        return;
    }
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut col: int64_t = (*curwin).w_virtcol as int64_t + scrolloff_cols;
    if scrolloff_cols > 0 as int64_t {
        let mut size: ::core::ffi::c_int = linetabsize_eol(curwin, (*curwin).w_topline);
        size = width1 + width2 * ((size - width1 + width2 - 1 as ::core::ffi::c_int) / width2);
        while col > size as int64_t {
            col -= width2 as int64_t;
        }
    }
    col -= (*curwin).w_skipcol as int64_t;
    if col >= width1 as int64_t {
        col -= width1 as int64_t;
        row += 1;
    }
    if col > width2 as int64_t {
        row += (col / width2 as int64_t) as ::core::ffi::c_int;
    }
    if row >= (*curwin).w_view_height {
        if (*curwin).w_skipcol == 0 as ::core::ffi::c_int {
            (*curwin).w_skipcol += width1;
            row -= 1;
        }
        if row >= (*curwin).w_view_height {
            (*curwin).w_skipcol += (row - (*curwin).w_view_height) * width2;
        }
        redraw_later(curwin, UPD_NOT_VALID as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_topfill(mut wp: *mut win_T, mut down: bool) {
    if (*wp).w_topfill > 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
        if (*wp).w_topfill + n > (*wp).w_view_height {
            if down as ::core::ffi::c_int != 0 && (*wp).w_topline > 1 as linenr_T {
                (*wp).w_topline -= 1;
                (*wp).w_topfill = 0 as ::core::ffi::c_int;
            } else {
                (*wp).w_topfill = (*wp).w_view_height - n;
                (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                    (*wp).w_topfill
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        }
    }
    win_check_anchored_floats(wp);
}
#[no_mangle]
pub unsafe extern "C" fn scrolldown_clamp() {
    let mut can_fill: bool = (*curwin).w_topfill < win_get_fill(curwin, (*curwin).w_topline);
    if (*curwin).w_topline <= 1 as linenr_T && !can_fill {
        return;
    }
    validate_cursor(curwin);
    let mut end_row: ::core::ffi::c_int = (*curwin).w_wrow;
    if can_fill {
        end_row += 1;
    } else {
        end_row += plines_win_nofill(curwin, (*curwin).w_topline - 1 as linenr_T, true_0 != 0);
    }
    if (*curwin).w_onebuf_opt.wo_wrap != 0 && (*curwin).w_view_width != 0 as ::core::ffi::c_int {
        validate_cheight(curwin);
        validate_virtcol(curwin);
        end_row += (*curwin).w_cline_height
            - 1 as ::core::ffi::c_int
            - (*curwin).w_virtcol as ::core::ffi::c_int / (*curwin).w_view_width;
    }
    if (end_row as int64_t) < (*curwin).w_view_height as int64_t - get_scrolloff_value(curwin) {
        if can_fill {
            (*curwin).w_topfill += 1;
            check_topfill(curwin, true_0 != 0);
        } else {
            (*curwin).w_topline -= 1;
            (*curwin).w_topfill = 0 as ::core::ffi::c_int;
        }
        hasFolding(
            curwin,
            (*curwin).w_topline,
            &raw mut (*curwin).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        (*curwin).w_botline -= 1;
        (*curwin).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}
#[no_mangle]
pub unsafe extern "C" fn scrollup_clamp() {
    if (*curwin).w_topline == (*curbuf).b_ml.ml_line_count
        && (*curwin).w_topfill == 0 as ::core::ffi::c_int
    {
        return;
    }
    validate_cursor(curwin);
    let mut start_row: ::core::ffi::c_int = (*curwin).w_wrow
        - plines_win_nofill(curwin, (*curwin).w_topline, true_0 != 0)
        - (*curwin).w_topfill;
    if (*curwin).w_onebuf_opt.wo_wrap != 0 && (*curwin).w_view_width != 0 as ::core::ffi::c_int {
        validate_virtcol(curwin);
        start_row -= (*curwin).w_virtcol as ::core::ffi::c_int / (*curwin).w_view_width;
    }
    if start_row as int64_t >= get_scrolloff_value(curwin) {
        if (*curwin).w_topfill > 0 as ::core::ffi::c_int {
            (*curwin).w_topfill -= 1;
        } else {
            hasFolding(
                curwin,
                (*curwin).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*curwin).w_topline,
            );
            (*curwin).w_topline += 1;
        }
        (*curwin).w_botline += 1;
        (*curwin).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}
unsafe extern "C" fn topline_back_winheight(
    mut wp: *mut win_T,
    mut lp: *mut lineoff_T,
    mut winheight: ::core::ffi::c_int,
) {
    if (*lp).fill < win_get_fill(wp, (*lp).lnum) {
        (*lp).fill += 1;
        (*lp).height = 1 as ::core::ffi::c_int;
    } else {
        (*lp).lnum -= 1;
        (*lp).fill = 0 as ::core::ffi::c_int;
        if (*lp).lnum < 1 as linenr_T {
            (*lp).height = MAXCOL as ::core::ffi::c_int;
        } else if hasFolding(
            wp,
            (*lp).lnum,
            &raw mut (*lp).lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*lp).height = !decor_conceal_line(
                wp,
                (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
        } else {
            (*lp).height = plines_win_nofill(wp, (*lp).lnum, winheight != 0);
        }
    };
}
unsafe extern "C" fn topline_back(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    topline_back_winheight(wp, lp, true_0);
}
unsafe extern "C" fn botline_forw(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    if (*lp).fill < win_get_fill(wp, (*lp).lnum + 1 as linenr_T) {
        (*lp).fill += 1;
        (*lp).height = 1 as ::core::ffi::c_int;
    } else {
        (*lp).lnum += 1;
        (*lp).fill = 0 as ::core::ffi::c_int;
        '_c2rust_label: {
            if !(*wp).w_buffer.is_null() {
            } else {
                __assert_fail(
                    b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1768 as ::core::ffi::c_uint,
                    b"void botline_forw(win_T *, lineoff_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*lp).lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            (*lp).height = MAXCOL as ::core::ffi::c_int;
        } else if hasFolding(
            wp,
            (*lp).lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*lp).lnum,
        ) {
            (*lp).height = !decor_conceal_line(
                wp,
                (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
        } else {
            (*lp).height = plines_win_nofill(wp, (*lp).lnum, true_0 != 0);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_top(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut always: ::core::ffi::c_int,
) {
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
    let mut old_topfill: linenr_T = (*wp).w_topfill as linenr_T;
    let mut off: int64_t = get_scrolloff_value(wp);
    if mouse_dragging > 0 as ::core::ffi::c_int {
        off = (mouse_dragging - 1 as ::core::ffi::c_int) as int64_t;
    }
    validate_cheight(wp);
    let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut used: ::core::ffi::c_int = (*wp).w_cline_height;
    if (*wp).w_cursor.lnum < (*wp).w_topline {
        scrolled = used;
    }
    let mut top: linenr_T = 0;
    let mut bot: linenr_T = 0;
    if hasFolding(wp, (*wp).w_cursor.lnum, &raw mut top, &raw mut bot) {
        top -= 1;
        bot += 1;
    } else {
        top = (*wp).w_cursor.lnum - 1 as linenr_T;
        bot = (*wp).w_cursor.lnum + 1 as linenr_T;
    }
    let mut new_topline: linenr_T = top + 1 as linenr_T;
    let mut extra: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_cursor.lnum);
    while top > 0 as linenr_T {
        let mut i: ::core::ffi::c_int = plines_win_nofill(wp, top, true_0 != 0);
        hasFolding(wp, top, &raw mut top, ::core::ptr::null_mut::<linenr_T>());
        if top < (*wp).w_topline {
            scrolled += i;
        }
        if (new_topline >= (*wp).w_topline || scrolled > min_scroll) && extra as int64_t >= off {
            break;
        }
        used += i;
        if (extra + i) as int64_t <= off && bot < (*(*wp).w_buffer).b_ml.ml_line_count {
            used += plines_win_full(
                wp,
                bot,
                &raw mut bot,
                ::core::ptr::null_mut::<bool>(),
                true_0 != 0,
                true_0 != 0,
            );
        }
        if used > (*wp).w_view_height {
            break;
        }
        extra += i;
        new_topline = top;
        top -= 1;
        bot += 1;
    }
    if used > (*wp).w_view_height {
        scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
    } else {
        if new_topline < (*wp).w_topline || always != 0 {
            (*wp).w_topline = new_topline;
        }
        (*wp).w_topline = if (*wp).w_topline < (*wp).w_cursor.lnum {
            (*wp).w_topline
        } else {
            (*wp).w_cursor.lnum
        };
        (*wp).w_topfill = win_get_fill(wp, (*wp).w_topline);
        if (*wp).w_topfill > 0 as ::core::ffi::c_int && extra as int64_t > off {
            (*wp).w_topfill -= extra - off as ::core::ffi::c_int;
            (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                (*wp).w_topfill
            } else {
                0 as ::core::ffi::c_int
            };
        }
        check_topfill(wp, false_0 != 0);
        if (*wp).w_topline != old_topline {
            reset_skipcol(wp);
        } else if (*wp).w_topline == (*wp).w_cursor.lnum {
            validate_virtcol(wp);
            if (*wp).w_skipcol >= (*wp).w_virtcol {
                reset_skipcol(wp);
            }
        }
        if (*wp).w_topline != old_topline
            || (*wp).w_skipcol != old_skipcol
            || (*wp).w_topfill as linenr_T != old_topfill
        {
            (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
        }
        (*wp).w_valid |= VALID_TOPLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_empty_rows(mut wp: *mut win_T, mut used: ::core::ffi::c_int) {
    (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
    if used == 0 as ::core::ffi::c_int {
        (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
    } else {
        (*wp).w_empty_rows = (*wp).w_view_height - used;
        if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            (*wp).w_filler_rows = win_get_fill(wp, (*wp).w_botline);
            if (*wp).w_empty_rows > (*wp).w_filler_rows {
                (*wp).w_empty_rows -= (*wp).w_filler_rows;
            } else {
                (*wp).w_filler_rows = (*wp).w_empty_rows;
                (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_bot(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut set_topbot: bool,
) {
    let mut loff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
    let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
    let mut old_botline: linenr_T = (*wp).w_botline;
    let mut old_valid: ::core::ffi::c_int = (*wp).w_valid;
    let mut old_empty_rows: ::core::ffi::c_int = (*wp).w_empty_rows;
    let mut cln: linenr_T = (*wp).w_cursor.lnum;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if set_topbot {
        let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cln_last: linenr_T = cln;
        hasFolding(
            wp,
            cln,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut cln_last,
        );
        (*wp).w_botline = cln_last + 1 as linenr_T;
        loff.lnum = cln_last + 1 as linenr_T;
        loff.fill = 0 as ::core::ffi::c_int;
        loop {
            topline_back_winheight(wp, &raw mut loff, false_0);
            if loff.height == MAXCOL as ::core::ffi::c_int {
                break;
            }
            if used + loff.height > (*wp).w_view_height {
                if do_sms {
                    if used < (*wp).w_view_height {
                        let mut plines_offset: ::core::ffi::c_int =
                            used + loff.height - (*wp).w_view_height;
                        used = (*wp).w_view_height;
                        (*wp).w_topfill = loff.fill;
                        (*wp).w_topline = loff.lnum;
                        (*wp).w_skipcol = skipcol_from_plines(wp, plines_offset) as colnr_T;
                    }
                }
                break;
            } else {
                (*wp).w_topfill = loff.fill;
                (*wp).w_topline = loff.lnum;
                used += loff.height;
            }
        }
        set_empty_rows(wp, used);
        (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        if (*wp).w_topline != old_topline
            || (*wp).w_topfill != old_topfill
            || (*wp).w_skipcol != old_skipcol
            || (*wp).w_skipcol != 0 as ::core::ffi::c_int
        {
            (*wp).w_valid &= !(VALID_WROW | VALID_CROW);
            if (*wp).w_skipcol != old_skipcol {
                redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            } else {
                reset_skipcol(wp);
            }
        }
    } else {
        validate_botline_win(wp);
    }
    let mut used_0: ::core::ffi::c_int = plines_win_nofill(wp, cln, true_0 != 0);
    let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cln >= (*wp).w_botline {
        scrolled = used_0;
        if cln == (*wp).w_botline {
            scrolled -= (*wp).w_empty_rows;
        }
        if do_sms {
            let mut top_plines: ::core::ffi::c_int =
                plines_win_nofill(wp, (*wp).w_topline, false_0 != 0);
            let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
            if width1 > 0 as ::core::ffi::c_int {
                let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
                let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if (*wp).w_skipcol > width1 {
                    skip_lines += ((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                        + 1 as ::core::ffi::c_int;
                } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                    skip_lines = 1 as ::core::ffi::c_int;
                }
                top_plines -= skip_lines;
                if top_plines > (*wp).w_view_height {
                    scrolled += top_plines - (*wp).w_view_height;
                }
            }
        }
    }
    let mut boff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    if !hasFolding(
        wp,
        (*wp).w_cursor.lnum,
        &raw mut loff.lnum,
        &raw mut boff.lnum,
    ) {
        loff.lnum = cln;
        boff.lnum = cln;
    }
    loff.fill = 0 as ::core::ffi::c_int;
    boff.fill = 0 as ::core::ffi::c_int;
    let mut fill_below_window: ::core::ffi::c_int =
        win_get_fill(wp, (*wp).w_botline) - (*wp).w_filler_rows;
    let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut so: int64_t = get_scrolloff_value(wp);
    while loff.lnum > 1 as linenr_T {
        if ((scrolled <= 0 as ::core::ffi::c_int || scrolled >= min_scroll)
            && extra as int64_t
                >= (if mouse_dragging > 0 as ::core::ffi::c_int {
                    (mouse_dragging - 1 as ::core::ffi::c_int) as int64_t
                } else {
                    so
                })
            || boff.lnum + 1 as linenr_T > (*(*wp).w_buffer).b_ml.ml_line_count)
            && loff.lnum <= (*wp).w_botline
            && (loff.lnum < (*wp).w_botline || loff.fill >= fill_below_window)
        {
            break;
        }
        topline_back(wp, &raw mut loff);
        if loff.height == MAXCOL as ::core::ffi::c_int {
            used_0 = MAXCOL as ::core::ffi::c_int;
        } else {
            used_0 += loff.height;
        }
        if used_0 > (*wp).w_view_height {
            break;
        }
        if loff.lnum >= (*wp).w_botline
            && (loff.lnum > (*wp).w_botline || loff.fill <= fill_below_window)
        {
            scrolled += loff.height;
            if loff.lnum == (*wp).w_botline && loff.fill == 0 as ::core::ffi::c_int {
                scrolled -= (*wp).w_empty_rows;
            }
        }
        if boff.lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
            continue;
        }
        botline_forw(wp, &raw mut boff);
        '_c2rust_label: {
            if boff.height != MAXCOL as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"boff.height != MAXCOL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/move.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    2067 as ::core::ffi::c_uint,
                    b"void scroll_cursor_bot(win_T *, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        used_0 += boff.height;
        if used_0 > (*wp).w_view_height {
            break;
        }
        if (extra as int64_t)
            < (if mouse_dragging > 0 as ::core::ffi::c_int {
                (mouse_dragging - 1 as ::core::ffi::c_int) as int64_t
            } else {
                so
            })
            || scrolled < min_scroll
        {
            extra += boff.height;
            if boff.lnum >= (*wp).w_botline
                || boff.lnum + 1 as linenr_T == (*wp).w_botline && boff.fill > (*wp).w_filler_rows
            {
                scrolled += boff.height;
                if boff.lnum == (*wp).w_botline && boff.fill == 0 as ::core::ffi::c_int {
                    scrolled -= (*wp).w_empty_rows;
                }
            }
        }
    }
    let mut line_count: linenr_T = 0;
    if scrolled <= 0 as ::core::ffi::c_int {
        line_count = 0 as ::core::ffi::c_int as linenr_T;
    } else if used_0 > (*wp).w_view_height {
        line_count = used_0 as linenr_T;
    } else {
        line_count = 0 as ::core::ffi::c_int as linenr_T;
        boff.fill = (*wp).w_topfill;
        boff.lnum = (*wp).w_topline - 1 as linenr_T;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < scrolled && boff.lnum < (*wp).w_botline {
            botline_forw(wp, &raw mut boff);
            i += boff.height;
            line_count += 1;
        }
        if i < scrolled {
            line_count = 9999 as ::core::ffi::c_int as linenr_T;
        }
    }
    if line_count >= (*wp).w_view_height as linenr_T && line_count > min_scroll as linenr_T {
        scroll_cursor_halfway(wp, false_0 != 0, true_0 != 0);
    } else if line_count > 0 as linenr_T {
        if do_sms {
            scrollup(wp, scrolled as linenr_T, true_0 != 0);
        } else {
            scrollup(wp, line_count, true_0 != 0);
        }
    }
    if (*wp).w_topline == old_topline
        && (*wp).w_skipcol == old_skipcol
        && set_topbot as ::core::ffi::c_int != 0
    {
        (*wp).w_botline = old_botline;
        (*wp).w_empty_rows = old_empty_rows;
        (*wp).w_valid = old_valid;
    }
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
    if set_topbot {
        cursor_correct_sms(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_halfway(
    mut wp: *mut win_T,
    mut atend: bool,
    mut prefer_above: bool,
) {
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut loff: lineoff_T = lineoff_T {
        lnum: (*wp).w_cursor.lnum,
        fill: 0,
        height: 0,
    };
    let mut boff: lineoff_T = lineoff_T {
        lnum: (*wp).w_cursor.lnum,
        fill: 0,
        height: 0,
    };
    hasFolding(wp, loff.lnum, &raw mut loff.lnum, &raw mut boff.lnum);
    let mut used: ::core::ffi::c_int = plines_win_nofill(wp, loff.lnum, true_0 != 0);
    loff.fill = 0 as ::core::ffi::c_int;
    boff.fill = 0 as ::core::ffi::c_int;
    let mut topline: linenr_T = loff.lnum;
    let mut skipcol: colnr_T = 0 as colnr_T;
    let mut want_height: ::core::ffi::c_int = 0;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms {
        if atend {
            want_height = ((*wp).w_view_height - used) / 2 as ::core::ffi::c_int;
            used = 0 as ::core::ffi::c_int;
        } else {
            want_height = (*wp).w_view_height;
        }
    }
    let mut topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while topline > 1 as linenr_T {
        if do_sms {
            topline_back_winheight(wp, &raw mut loff, false_0);
            if loff.height == MAXCOL as ::core::ffi::c_int {
                break;
            }
            used += loff.height;
            if !atend && boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                botline_forw(wp, &raw mut boff);
                used += boff.height;
            }
            if used > want_height {
                if used - loff.height < want_height {
                    topline = loff.lnum;
                    topfill = loff.fill;
                    skipcol = skipcol_from_plines(wp, used - want_height) as colnr_T;
                }
                break;
            } else {
                topline = loff.lnum;
                topfill = loff.fill;
            }
        } else {
            let mut done: bool = false_0 != 0;
            let mut above: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut below: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                if if prefer_above as ::core::ffi::c_int != 0 {
                    (round == 2 as ::core::ffi::c_int && below < above) as ::core::ffi::c_int
                } else {
                    (round == 1 as ::core::ffi::c_int && below <= above) as ::core::ffi::c_int
                } != 0
                {
                    if boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                        botline_forw(wp, &raw mut boff);
                        used += boff.height;
                        if used > (*wp).w_view_height {
                            done = true_0 != 0;
                            break;
                        } else {
                            below += boff.height;
                        }
                    } else {
                        below += 1;
                        if atend {
                            used += 1;
                        }
                    }
                }
                if if prefer_above as ::core::ffi::c_int != 0 {
                    (round == 1 as ::core::ffi::c_int && below >= above) as ::core::ffi::c_int
                } else {
                    (round == 1 as ::core::ffi::c_int && below > above) as ::core::ffi::c_int
                } != 0
                {
                    topline_back(wp, &raw mut loff);
                    if loff.height == MAXCOL as ::core::ffi::c_int {
                        used = MAXCOL as ::core::ffi::c_int;
                    } else {
                        used += loff.height;
                    }
                    if used > (*wp).w_view_height {
                        done = true_0 != 0;
                        break;
                    } else {
                        above += loff.height;
                        topline = loff.lnum;
                        topfill = loff.fill;
                    }
                }
                round += 1;
            }
            if done {
                break;
            }
        }
    }
    if !hasFolding(
        wp,
        topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    ) && ((*wp).w_topline != topline
        || skipcol != 0 as ::core::ffi::c_int
        || (*wp).w_skipcol != 0 as ::core::ffi::c_int)
    {
        (*wp).w_topline = topline;
        if skipcol != 0 as ::core::ffi::c_int {
            (*wp).w_skipcol = skipcol;
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        } else if do_sms {
            reset_skipcol(wp);
        }
    }
    (*wp).w_topfill = topfill;
    if old_topline > (*wp).w_topline + (*wp).w_view_height as linenr_T {
        (*wp).w_botfill = false_0 != 0;
    }
    check_topfill(wp, false_0 != 0);
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
    (*wp).w_valid |= VALID_TOPLINE;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_correct(mut wp: *mut win_T) {
    let mut above_wanted: int64_t = get_scrolloff_value(wp);
    let mut below_wanted: int64_t = get_scrolloff_value(wp);
    if mouse_dragging > 0 as ::core::ffi::c_int {
        above_wanted = (mouse_dragging - 1 as ::core::ffi::c_int) as int64_t;
        below_wanted = (mouse_dragging - 1 as ::core::ffi::c_int) as int64_t;
    }
    if (*wp).w_topline == 1 as linenr_T {
        above_wanted = 0 as int64_t;
        let mut max_off: ::core::ffi::c_int = (*wp).w_view_height / 2 as ::core::ffi::c_int;
        below_wanted = if below_wanted < max_off as int64_t {
            below_wanted
        } else {
            max_off as int64_t
        };
    }
    validate_botline_win(wp);
    if (*wp).w_botline == (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
        && mouse_dragging == 0 as ::core::ffi::c_int
    {
        below_wanted = 0 as int64_t;
        let mut max_off_0: ::core::ffi::c_int =
            ((*wp).w_view_height - 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
        above_wanted = if above_wanted < max_off_0 as int64_t {
            above_wanted
        } else {
            max_off_0 as int64_t
        };
    }
    let mut cln: linenr_T = (*wp).w_cursor.lnum;
    if cln as int64_t >= (*wp).w_topline as int64_t + above_wanted
        && (cln as int64_t) < (*wp).w_botline as int64_t - below_wanted
        && !win_lines_concealed(wp)
    {
        return;
    }
    if (*wp).w_onebuf_opt.wo_sms != 0 && (*wp).w_onebuf_opt.wo_wrap == 0 {
        if (*wp).w_cline_height == (*wp).w_view_height {
            reset_skipcol(wp);
            return;
        }
    }
    let mut topline: linenr_T = (*wp).w_topline;
    let mut botline: linenr_T = (*wp).w_botline - 1 as linenr_T;
    let mut above: ::core::ffi::c_int = (*wp).w_topfill;
    let mut below: ::core::ffi::c_int = (*wp).w_filler_rows;
    while ((above as int64_t) < above_wanted || (below as int64_t) < below_wanted)
        && topline < botline
    {
        if (below as int64_t) < below_wanted && (below <= above || above as int64_t >= above_wanted)
        {
            below += plines_win_full(
                wp,
                botline,
                ::core::ptr::null_mut::<linenr_T>(),
                ::core::ptr::null_mut::<bool>(),
                true_0 != 0,
                true_0 != 0,
            );
            hasFolding(
                wp,
                botline,
                &raw mut botline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            botline -= 1;
        }
        if (above as int64_t) < above_wanted && (above < below || below as int64_t >= below_wanted)
        {
            above += plines_win_nofill(wp, topline, true_0 != 0);
            hasFolding(
                wp,
                topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut topline,
            );
            if topline < botline {
                above += win_get_fill(wp, topline + 1 as linenr_T);
            }
            topline += 1;
        }
    }
    if topline == botline || botline == 0 as linenr_T {
        (*wp).w_cursor.lnum = topline;
    } else if topline > botline {
        (*wp).w_cursor.lnum = botline;
    } else {
        if cln < topline && (*wp).w_topline > 1 as linenr_T {
            (*wp).w_cursor.lnum = topline;
            (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
        if cln > botline && (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            (*wp).w_cursor.lnum = botline;
            (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
    }
    check_cursor_moved(wp);
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
}
unsafe extern "C" fn get_scroll_overlap(mut dir: Direction) -> ::core::ffi::c_int {
    let mut loff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    let mut min_height: ::core::ffi::c_int = (*curwin).w_view_height - 2 as ::core::ffi::c_int;
    validate_botline_win(curwin);
    if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
        && (*curwin).w_topline == 1 as linenr_T
        || dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
            && (*curwin).w_botline > (*curbuf).b_ml.ml_line_count
    {
        return min_height + 2 as ::core::ffi::c_int;
    }
    loff.lnum = if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        (*curwin).w_botline
    } else {
        (*curwin).w_topline - 1 as linenr_T
    };
    loff.fill = win_get_fill(
        curwin,
        loff.lnum
            + (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
    ) - (if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        (*curwin).w_filler_rows
    } else {
        (*curwin).w_topfill
    });
    loff.height = if loff.fill > 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        plines_win_nofill(curwin, loff.lnum, true_0 != 0)
    };
    let mut h1: ::core::ffi::c_int = loff.height;
    if h1 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin, &raw mut loff);
    } else {
        botline_forw(curwin, &raw mut loff);
    }
    let mut h2: ::core::ffi::c_int = loff.height;
    if h2 == MAXCOL as ::core::ffi::c_int || h2 + h1 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin, &raw mut loff);
    } else {
        botline_forw(curwin, &raw mut loff);
    }
    let mut h3: ::core::ffi::c_int = loff.height;
    if h3 == MAXCOL as ::core::ffi::c_int || h3 + h2 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin, &raw mut loff);
    } else {
        botline_forw(curwin, &raw mut loff);
    }
    let mut h4: ::core::ffi::c_int = loff.height;
    if h4 == MAXCOL as ::core::ffi::c_int || h4 + h3 + h2 > min_height || h3 + h2 + h1 > min_height
    {
        return min_height + 1 as ::core::ffi::c_int;
    } else {
        return min_height;
    };
}
unsafe extern "C" fn scroll_with_sms(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut curscount: *mut ::core::ffi::c_int,
) -> bool {
    let mut prev_sms: ::core::ffi::c_int = (*curwin).w_onebuf_opt.wo_sms;
    let mut prev_skipcol: colnr_T = (*curwin).w_skipcol;
    let mut prev_topline: linenr_T = (*curwin).w_topline;
    let mut prev_topfill: ::core::ffi::c_int = (*curwin).w_topfill;
    (*curwin).w_onebuf_opt.wo_sms = true_0;
    scroll_redraw(
        (dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
        count as linenr_T,
    );
    if prev_sms == 0 && (*curwin).w_skipcol > 0 as ::core::ffi::c_int {
        let mut fixdir: ::core::ffi::c_int = dir as ::core::ffi::c_int;
        if labs(((*curwin).w_topline - prev_topline) as ::core::ffi::c_long)
            > (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int) as ::core::ffi::c_int
                as ::core::ffi::c_long
        {
            fixdir = dir as ::core::ffi::c_int * -1 as ::core::ffi::c_int;
        }
        let mut width1: ::core::ffi::c_int = (*curwin).w_view_width - win_col_off(curwin);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin);
        count = 1 as ::core::ffi::c_int
            + ((*curwin).w_skipcol as ::core::ffi::c_int - width1 - 1 as ::core::ffi::c_int)
                / width2;
        if fixdir == FORWARD as ::core::ffi::c_int {
            count = 1 as ::core::ffi::c_int
                + (linetabsize_eol(curwin, (*curwin).w_topline)
                    - (*curwin).w_skipcol as ::core::ffi::c_int
                    - width1
                    + width2
                    - 1 as ::core::ffi::c_int)
                    / width2;
        }
        scroll_redraw(
            (fixdir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
            count as linenr_T,
        );
        *curscount += count
            * (if fixdir == dir as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            });
    }
    (*curwin).w_onebuf_opt.wo_sms = prev_sms;
    return (*curwin).w_topline != prev_topline
        || (*curwin).w_topfill != prev_topfill
        || (*curwin).w_skipcol != prev_skipcol;
}
#[no_mangle]
pub unsafe extern "C" fn pagescroll(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut half: bool,
) -> ::core::ffi::c_int {
    let mut did_move: bool = false_0 != 0;
    let mut buflen: ::core::ffi::c_int = (*curbuf).b_ml.ml_line_count as ::core::ffi::c_int;
    let mut prev_col: colnr_T = (*curwin).w_cursor.col;
    let mut prev_curswant: colnr_T = (*curwin).w_curswant;
    let mut prev_lnum: linenr_T = (*curwin).w_cursor.lnum;
    let mut oa: oparg_T = oparg_T {
        op_type: 0 as ::core::ffi::c_int,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut ca: cmdarg_T = cmdarg_T {
        oap: ::core::ptr::null_mut::<oparg_T>(),
        prechar: 0,
        cmdchar: 0,
        nchar: 0,
        nchar_composing: [0; 32],
        nchar_len: 0,
        extra_char: 0,
        opcount: 0,
        count0: 0,
        count1: 0,
        arg: 0,
        retval: 0,
        searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    ca.oap = &raw mut oa;
    if half {
        if count != 0 {
            (*curwin).w_onebuf_opt.wo_scr = (if (*curwin).w_view_height < count {
                (*curwin).w_view_height
            } else {
                count
            }) as OptInt;
        }
        count = if (*curwin).w_view_height < (*curwin).w_onebuf_opt.wo_scr as ::core::ffi::c_int {
            (*curwin).w_view_height
        } else {
            (*curwin).w_onebuf_opt.wo_scr as ::core::ffi::c_int
        };
        let mut curscount: ::core::ffi::c_int = count;
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
            && ((*curwin).w_topline + (*curwin).w_view_height as linenr_T + count as linenr_T
                > buflen as linenr_T
                || win_lines_concealed(curwin) as ::core::ffi::c_int != 0)
        {
            let mut n: ::core::ffi::c_int = plines_correct_topline(
                curwin,
                (*curwin).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            if n - count < (*curwin).w_view_height && (*curwin).w_topline < buflen as linenr_T {
                n += plines_m_win(
                    curwin,
                    (*curwin).w_topline + 1 as linenr_T,
                    buflen as linenr_T,
                    (*curwin).w_view_height + count,
                );
            }
            if n < (*curwin).w_view_height + count {
                count = n - (*curwin).w_view_height;
            }
        }
        if count > 0 as ::core::ffi::c_int {
            did_move = scroll_with_sms(dir, count, &raw mut curscount);
            (*curwin).w_cursor.lnum = prev_lnum;
            (*curwin).w_cursor.col = prev_col;
            (*curwin).w_curswant = prev_curswant;
        }
        if (*curwin).w_onebuf_opt.wo_wrap != 0 {
            nv_screengo(
                &raw mut oa,
                dir as ::core::ffi::c_int,
                curscount,
                true_0 != 0,
            );
        } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            cursor_down_inner(curwin, curscount, true_0 != 0);
        } else {
            cursor_up_inner(curwin, curscount as linenr_T, true_0 != 0);
        }
    } else {
        count *= if firstwin == lastwin
            && p_window > 0 as OptInt
            && p_window < (Rows - 1 as ::core::ffi::c_int) as OptInt
        {
            if 1 as ::core::ffi::c_int > p_window as ::core::ffi::c_int - 2 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                p_window as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            }
        } else {
            get_scroll_overlap(dir)
        };
        did_move = scroll_with_sms(dir, count, &raw mut count);
        if did_move {
            validate_botline_win(curwin);
            let mut lnum: linenr_T = if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                (*curwin).w_topline
            } else {
                (*curwin).w_botline - 1 as linenr_T
            };
            (*curwin).w_cursor.lnum = if lnum > 1 as linenr_T {
                lnum
            } else {
                1 as linenr_T
            };
        }
    }
    if get_scrolloff_value(curwin) > 0 as int64_t {
        cursor_correct(curwin);
    }
    foldAdjustCursor(curwin);
    did_move = did_move as ::core::ffi::c_int != 0
        || prev_col != (*curwin).w_cursor.col
        || prev_lnum != (*curwin).w_cursor.lnum;
    if !did_move {
        beep_flush();
    } else if (*curwin).w_onebuf_opt.wo_sms == 0 {
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else if p_sol != 0 {
        nv_g_home_m_cmd(&raw mut ca);
    }
    return if did_move as ::core::ffi::c_int != 0 {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_check_cursorbind() {
    static mut prev_curwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    static mut prev_cursor: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    if curwin == prev_curwin && equalpos((*curwin).w_cursor, prev_cursor) as ::core::ffi::c_int != 0
    {
        return;
    }
    prev_curwin = curwin;
    prev_cursor = (*curwin).w_cursor;
    let mut line: linenr_T = (*curwin).w_cursor.lnum;
    let mut col: colnr_T = (*curwin).w_cursor.col;
    let mut coladd: colnr_T = (*curwin).w_cursor.coladd;
    let mut curswant: colnr_T = (*curwin).w_curswant;
    let mut set_curswant: bool = (*curwin).w_set_curswant != 0;
    let mut old_curwin: *mut win_T = curwin;
    let mut old_curbuf: *mut buf_T = curbuf;
    let mut old_VIsual_select: ::core::ffi::c_int = VIsual_select as ::core::ffi::c_int;
    let mut old_VIsual_active: ::core::ffi::c_int = VIsual_active as ::core::ffi::c_int;
    VIsual_active = false_0 != 0;
    VIsual_select = VIsual_active;
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        curwin = wp;
        curbuf = (*curwin).w_buffer;
        if curwin != old_curwin && (*curwin).w_onebuf_opt.wo_crb != 0 {
            if (*curwin).w_onebuf_opt.wo_diff != 0 {
                (*curwin).w_cursor.lnum = diff_get_corresponding_line(old_curbuf, line);
            } else {
                (*curwin).w_cursor.lnum = line;
            }
            (*curwin).w_cursor.col = col;
            (*curwin).w_cursor.coladd = coladd;
            (*curwin).w_curswant = curswant;
            (*curwin).w_set_curswant = set_curswant as ::core::ffi::c_int;
            let mut restart_edit_save: ::core::ffi::c_int = restart_edit;
            restart_edit = true_0;
            check_cursor(curwin);
            if (*curwin).w_onebuf_opt.wo_scb == 0 {
                validate_cursor(curwin);
            }
            restart_edit = restart_edit_save;
            mb_adjust_cursor();
            redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
            if (*curwin).w_onebuf_opt.wo_scb == 0 {
                update_topline(curwin);
            }
            (*curwin).w_redr_status = true_0 != 0;
        }
        wp = (*wp).w_next;
    }
    VIsual_select = old_VIsual_select != 0;
    VIsual_active = old_VIsual_active != 0;
    curwin = old_curwin;
    curbuf = old_curbuf;
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
