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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn nvim_buf_set_lines(
        channel_id: uint64_t,
        buf: Buffer,
        start: Integer,
        end: Integer,
        strict_indexing: Boolean,
        replacement: Array,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn api_free_array(value: Array);
    fn api_clear_error(value: *mut Error);
    fn parse_winborder(
        fconfig: *mut WinConfig,
        border_opt: *mut ::core::ffi::c_char,
        err: *mut Error,
    ) -> bool;
    fn block_autocmds();
    fn unblock_autocmds();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buf_clear();
    fn bt_nofile(buf: *const buf_T) -> bool;
    static mut opt_winborder_values: [*const ::core::ffi::c_char; 9];
    static mut cia_flags: ::core::ffi::c_uint;
    static mut p_pumborder: *mut ::core::ffi::c_char;
    static mut p_pb: OptInt;
    static mut p_ph: OptInt;
    static mut p_pw: OptInt;
    static mut p_pmw: OptInt;
    static mut p_mousemev: ::core::ffi::c_int;
    static mut p_pvh: OptInt;
    fn reverse_text(s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn transstr(s: *const ::core::ffi::c_char, untab: bool) -> *mut ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn cmdline_compl_pattern() -> *mut ::core::ffi::c_char;
    fn cmdline_compl_is_fuzzy() -> bool;
    static e_menu_only_exists_in_another_mode: [::core::ffi::c_char; 0];
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor_mayforce(wp: *mut win_T, force: bool);
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_float(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: float_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_bool(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: BoolVarValue,
    ) -> ::core::ffi::c_int;
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn fuzzy_match_str_with_pos(
        str: *mut ::core::ffi::c_char,
        pat: *const ::core::ffi::c_char,
    ) -> *mut garray_T;
    fn ga_clear(gap: *mut garray_T);
    fn vgetc() -> ::core::ffi::c_int;
    fn vungetc(c: ::core::ffi::c_int);
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut mouse_grid: ::core::ffi::c_int;
    static mut mouse_row: ::core::ffi::c_int;
    static mut mouse_col: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut textlock: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut no_u_sync: ::core::ffi::c_int;
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut must_redraw_pum: bool;
    static mut g_do_tagpreview: ::core::ffi::c_int;
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut cmdline_win: *mut win_T;
    static mut default_grid: ScreenGrid;
    static mut linebuf_char: *mut schar_T;
    static mut linebuf_attr: *mut sattr_T;
    fn schar_from_str(str: *const ::core::ffi::c_char) -> schar_T;
    fn grid_invalidate(grid: *mut ScreenGrid);
    fn screengrid_line_start(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
    );
    fn grid_line_put_schar(
        col: ::core::ffi::c_int,
        schar: schar_T,
        attr: ::core::ffi::c_int,
    );
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_fill(
        start_col: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        sc: schar_T,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_flush();
    fn grid_alloc(
        grid: *mut ScreenGrid,
        rows: ::core::ffi::c_int,
        columns: ::core::ffi::c_int,
        copy: bool,
        valid: bool,
    );
    fn grid_free(grid: *mut ScreenGrid);
    fn grid_assign_handle(grid: *mut ScreenGrid);
    fn grid_draw_border(
        grid: *mut ScreenGrid,
        config: *mut WinConfig,
        adj: *mut ::core::ffi::c_int,
        winbl: ::core::ffi::c_int,
        hl_attr: *mut ::core::ffi::c_int,
    );
    fn get_win_by_grid_handle(handle: handle_T) -> *mut win_T;
    static mut ns_hl_fast: NS;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn hl_get_ui_attr(
        ns_id: ::core::ffi::c_int,
        idx: ::core::ffi::c_int,
        final_id: ::core::ffi::c_int,
        optional: bool,
    ) -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn syn_check_group(
        name: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn ins_compl_leader() -> *mut ::core::ffi::c_char;
    fn get_cot_flags() -> ::core::ffi::c_uint;
    fn compl_match_curr_select(selected: ::core::ffi::c_int) -> bool;
    fn ins_compl_active() -> bool;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
    fn menu_is_separator(name: *mut ::core::ffi::c_char) -> bool;
    fn get_menu_mode_flag() -> ::core::ffi::c_int;
    fn execute_menu(
        eap: *const exarg_T,
        menu: *mut vimmenu_T,
        mode_idx: ::core::ffi::c_int,
    );
    fn menu_find(path_name: *const ::core::ffi::c_char) -> *mut vimmenu_T;
    fn mouse_find_win_outer(
        gridp: *mut ::core::ffi::c_int,
        rowp: *mut ::core::ffi::c_int,
        colp: *mut ::core::ffi::c_int,
    ) -> *mut win_T;
    fn update_topline(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn validate_cheight(wp: *mut win_T);
    fn validate_cursor_col(wp: *mut win_T);
    fn set_option_value_give_err(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
    );
    fn init_charsize_arg(
        csarg: *mut CharsizeArg,
        wp: *mut win_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
    ) -> CSType;
    fn linesize_regular(
        csarg: *mut CharsizeArg,
        vcol_arg: ::core::ffi::c_int,
        len: colnr_T,
    ) -> ::core::ffi::c_int;
    fn linesize_fast(
        csarg: *const CharsizeArg,
        vcol_arg: ::core::ffi::c_int,
        len: colnr_T,
    ) -> ::core::ffi::c_int;
    fn plines_m_win(
        wp: *mut win_T,
        first: linenr_T,
        last: linenr_T,
        max: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static mut pum_grid: ScreenGrid;
    static mut pum_want: C2Rust_Unnamed_24;
    fn ui_pum_get_height() -> ::core::ffi::c_int;
    fn ui_pum_get_pos(
        pwidth: *mut ::core::ffi::c_double,
        pheight: *mut ::core::ffi::c_double,
        prow: *mut ::core::ffi::c_double,
        pcol: *mut ::core::ffi::c_double,
    ) -> bool;
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_option_set(name: String_0, value: Object);
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_call_grid_destroy(grid: Integer);
    fn ui_call_win_float_pos(
        grid: Integer,
        win: Window,
        anchor: String_0,
        anchor_grid: Integer,
        anchor_row: Float,
        anchor_col: Float,
        mouse_enabled: Boolean,
        zindex: Integer,
        compindex: Integer,
        screen_row: Integer,
        screen_col: Integer,
    );
    fn ui_call_win_close(grid: Integer);
    fn ui_call_popupmenu_show(
        items: Array,
        selected: Integer,
        row: Integer,
        col: Integer,
        grid: Integer,
    );
    fn ui_call_popupmenu_hide();
    fn ui_call_popupmenu_select(selected: Integer);
    fn ui_comp_put_grid(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
        height: ::core::ffi::c_int,
        width: ::core::ffi::c_int,
        valid: bool,
        on_top: bool,
    ) -> bool;
    fn ui_comp_remove_grid(grid: *mut ScreenGrid);
    fn ui_comp_compose_grid(grid: *mut ScreenGrid);
    fn win_valid(win: *const win_T) -> bool;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_setheight(height: ::core::ffi::c_int);
    fn win_config_float(wp: *mut win_T, fconfig: WinConfig);
    fn win_float_find_preview() -> *mut win_T;
    fn win_float_create_preview(enter: bool, new_buf: bool) -> *mut win_T;
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
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
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
pub type Buffer = handle_T;
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
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_14 = 250;
pub const kZIndexMessages: C2Rust_Unnamed_14 = 200;
pub const kZIndexPopupMenu: C2Rust_Unnamed_14 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_14 = 50;
pub const kZIndexDefaultGrid: C2Rust_Unnamed_14 = 0;
pub type hlf_T = ::core::ffi::c_uint;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_15; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
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
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
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
    pub cs_pend: C2Rust_Unnamed_16,
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
pub union C2Rust_Unnamed_16 {
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptCotFlagNearest: C2Rust_Unnamed_17 = 1024;
pub const kOptCotFlagPreinsert: C2Rust_Unnamed_17 = 512;
pub const kOptCotFlagNosort: C2Rust_Unnamed_17 = 256;
pub const kOptCotFlagFuzzy: C2Rust_Unnamed_17 = 128;
pub const kOptCotFlagNoselect: C2Rust_Unnamed_17 = 64;
pub const kOptCotFlagNoinsert: C2Rust_Unnamed_17 = 32;
pub const kOptCotFlagPopup: C2Rust_Unnamed_17 = 16;
pub const kOptCotFlagPreview: C2Rust_Unnamed_17 = 8;
pub const kOptCotFlagLongest: C2Rust_Unnamed_17 = 4;
pub const kOptCotFlagMenuone: C2Rust_Unnamed_17 = 2;
pub const kOptCotFlagMenu: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_18 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_18 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_18 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_18 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_18 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_18 = 20;
pub const UPD_VALID: C2Rust_Unnamed_18 = 10;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_19 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_19 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_19 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VimMenu {
    pub modes: ::core::ffi::c_int,
    pub enabled: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
    pub dname: *mut ::core::ffi::c_char,
    pub en_name: *mut ::core::ffi::c_char,
    pub en_dname: *mut ::core::ffi::c_char,
    pub mnemonic: ::core::ffi::c_int,
    pub actext: *mut ::core::ffi::c_char,
    pub priority: ::core::ffi::c_int,
    pub strings: [*mut ::core::ffi::c_char; 8],
    pub noremap: [::core::ffi::c_int; 8],
    pub silent: [bool; 8],
    pub children: *mut vimmenu_T,
    pub parent: *mut vimmenu_T,
    pub next: *mut vimmenu_T,
}
pub type vimmenu_T = VimMenu;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_20 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_20 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_20 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_20 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_20 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_20 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_20 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_20 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_20 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_20 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_20 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_20 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_20 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_20 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_20 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_20 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_20 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_20 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_20 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_20 = 1;
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
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CPT_COUNT: C2Rust_Unnamed_21 = 4;
pub const CPT_INFO: C2Rust_Unnamed_21 = 3;
pub const CPT_MENU: C2Rust_Unnamed_21 = 2;
pub const CPT_KIND: C2Rust_Unnamed_21 = 1;
pub const CPT_ABBR: C2Rust_Unnamed_21 = 0;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_22 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_22 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_22 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_22 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_22 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_22 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_22 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_22 = 1;
pub type CSType = bool;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const kCharsizeFast: C2Rust_Unnamed_23 = 1;
pub const kCharsizeRegular: C2Rust_Unnamed_23 = 0;
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
pub struct pumitem_T {
    pub pum_text: *mut ::core::ffi::c_char,
    pub pum_kind: *mut ::core::ffi::c_char,
    pub pum_extra: *mut ::core::ffi::c_char,
    pub pum_info: *mut ::core::ffi::c_char,
    pub pum_cpt_source_idx: ::core::ffi::c_int,
    pub pum_user_abbr_hlattr: ::core::ffi::c_int,
    pub pum_user_kind_hlattr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_24 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active
    }
        .offset(hlf as isize);
}
pub const K_UP: ::core::ffi::c_int = -('k' as ::core::ffi::c_int
    + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DOWN: ::core::ffi::c_int = -('k' as ::core::ffi::c_int
    + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
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
            s: [C2Rust_Unnamed_15 {
                oldcol: 0,
                i: 0,
            }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return linesize_fast(&raw mut csarg, 0 as ::core::ffi::c_int, len)
    } else {
        return linesize_regular(&raw mut csarg, 0 as ::core::ffi::c_int, len)
    };
}
static mut pum_array: *mut pumitem_T = ::core::ptr::null_mut::<pumitem_T>();
static mut pum_size: ::core::ffi::c_int = 0;
static mut pum_selected: ::core::ffi::c_int = 0;
static mut pum_first: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut pum_height: ::core::ffi::c_int = 0;
static mut pum_width: ::core::ffi::c_int = 0;
static mut pum_base_width: ::core::ffi::c_int = 0;
static mut pum_kind_width: ::core::ffi::c_int = 0;
static mut pum_extra_width: ::core::ffi::c_int = 0;
static mut pum_scrollbar: ::core::ffi::c_int = 0;
static mut pum_rl: bool = false;
static mut pum_anchor_grid: ::core::ffi::c_int = 0;
static mut pum_row: ::core::ffi::c_int = 0;
static mut pum_col: ::core::ffi::c_int = 0;
static mut pum_win_row_offset: ::core::ffi::c_int = 0;
static mut pum_win_col_offset: ::core::ffi::c_int = 0;
static mut pum_left_col: ::core::ffi::c_int = 0;
static mut pum_right_col: ::core::ffi::c_int = 0;
static mut pum_above: bool = false;
static mut pum_is_visible: bool = false_0 != 0;
static mut pum_is_drawn: bool = false_0 != 0;
static mut pum_external: bool = false_0 != 0;
static mut pum_invalid: bool = false_0 != 0;
unsafe extern "C" fn pum_compute_size() {
    pum_base_width = 0 as ::core::ffi::c_int;
    pum_kind_width = 0 as ::core::ffi::c_int;
    pum_extra_width = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < pum_size {
        if !(*pum_array.offset(i as isize)).pum_text.is_null() {
            let mut w: ::core::ffi::c_int = vim_strsize(
                (*pum_array.offset(i as isize)).pum_text,
            );
            if pum_base_width < w {
                pum_base_width = w;
            }
        }
        if !(*pum_array.offset(i as isize)).pum_kind.is_null() {
            let mut w_0: ::core::ffi::c_int = vim_strsize(
                (*pum_array.offset(i as isize)).pum_kind,
            ) + 1 as ::core::ffi::c_int;
            if pum_kind_width < w_0 {
                pum_kind_width = w_0;
            }
        }
        if !(*pum_array.offset(i as isize)).pum_extra.is_null() {
            let mut w_1: ::core::ffi::c_int = vim_strsize(
                (*pum_array.offset(i as isize)).pum_extra,
            ) + 1 as ::core::ffi::c_int;
            if pum_extra_width < w_1 {
                pum_extra_width = w_1;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn pum_compute_vertical_placement(
    mut size: ::core::ffi::c_int,
    mut target_win: *mut win_T,
    mut pum_win_row: ::core::ffi::c_int,
    mut above_row: ::core::ffi::c_int,
    mut below_row: ::core::ffi::c_int,
    mut pum_border_size: ::core::ffi::c_int,
) {
    let mut context_lines: ::core::ffi::c_int = 0;
    pum_height = if size < 10 as ::core::ffi::c_int {
        size
    } else {
        10 as ::core::ffi::c_int
    };
    if p_ph > 0 as OptInt && pum_height as OptInt > p_ph {
        pum_height = p_ph as ::core::ffi::c_int;
    }
    if pum_win_row + 2 as ::core::ffi::c_int + pum_border_size >= below_row - pum_height
        && pum_win_row - above_row > (below_row - above_row) / 2 as ::core::ffi::c_int
    {
        pum_above = true_0 != 0;
        if State & MODE_CMDLINE as ::core::ffi::c_int != 0 && target_win.is_null() {
            context_lines = 0 as ::core::ffi::c_int;
        } else {
            context_lines = if (2 as ::core::ffi::c_int)
                < (*target_win).w_wrow - (*target_win).w_cline_row
            {
                2 as ::core::ffi::c_int
            } else {
                (*target_win).w_wrow - (*target_win).w_cline_row
            };
        }
        if pum_win_row >= size + context_lines {
            pum_row = pum_win_row - size - context_lines;
            pum_height = size;
        } else {
            pum_row = 0 as ::core::ffi::c_int;
            pum_height = pum_win_row - context_lines;
        }
        if p_ph > 0 as OptInt && pum_height as OptInt > p_ph {
            pum_row += pum_height - p_ph as ::core::ffi::c_int;
            pum_height = p_ph as ::core::ffi::c_int;
        }
        if pum_border_size > 0 as ::core::ffi::c_int
            && pum_border_size + pum_row + pum_height >= pum_win_row
        {
            if pum_row < 2 as ::core::ffi::c_int {
                pum_height -= pum_border_size;
            } else {
                pum_row -= pum_border_size;
            }
        }
    } else {
        pum_above = false_0 != 0;
        if State & MODE_CMDLINE as ::core::ffi::c_int != 0 && target_win.is_null() {
            context_lines = 0 as ::core::ffi::c_int;
        } else {
            validate_cheight(target_win);
            let mut cline_visible_offset: ::core::ffi::c_int = (*target_win).w_cline_row
                + (*target_win).w_cline_height - (*target_win).w_wrow;
            context_lines = if (3 as ::core::ffi::c_int) < cline_visible_offset {
                3 as ::core::ffi::c_int
            } else {
                cline_visible_offset
            };
        }
        pum_row = pum_win_row + context_lines;
        pum_height = if below_row - pum_row < size { below_row - pum_row } else { size };
        if p_ph > 0 as OptInt && pum_height as OptInt > p_ph {
            pum_height = p_ph as ::core::ffi::c_int;
        }
        if pum_row + pum_height + pum_border_size >= cmdline_row {
            pum_height -= pum_border_size;
        }
    }
    if above_row > 0 as ::core::ffi::c_int && pum_row < above_row
        && pum_height > above_row
    {
        pum_row = above_row;
        pum_height = pum_win_row - above_row;
    }
}
unsafe extern "C" fn set_pum_width_aligned_with_cursor(
    mut width: ::core::ffi::c_int,
    mut available_width: ::core::ffi::c_int,
) -> bool {
    let mut end_padding: bool = true_0 != 0;
    if (width as OptInt) < p_pw {
        width = p_pw as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    if p_pmw > 0 as OptInt && width as OptInt > p_pmw {
        width = p_pmw as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    pum_width = width
        + (if end_padding as ::core::ffi::c_int != 0 && width as OptInt >= p_pw {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    return available_width >= pum_width;
}
unsafe extern "C" fn pum_compute_horizontal_placement(
    mut target_win: *mut win_T,
    mut cursor_col: ::core::ffi::c_int,
    mut border_width: ::core::ffi::c_int,
) {
    let mut max_col: ::core::ffi::c_int = if Columns
        > (if !target_win.is_null() {
            (*target_win).w_wincol + (*target_win).w_view_width
        } else {
            0 as ::core::ffi::c_int
        })
    {
        Columns
    } else if !target_win.is_null() {
        (*target_win).w_wincol + (*target_win).w_view_width
    } else {
        0 as ::core::ffi::c_int
    };
    let mut desired_width: ::core::ffi::c_int = pum_base_width + pum_kind_width
        + pum_extra_width;
    let mut available_width: ::core::ffi::c_int = 0;
    if pum_rl {
        available_width = cursor_col - pum_scrollbar + 1 as ::core::ffi::c_int
            - border_width;
    } else {
        available_width = max_col - cursor_col - pum_scrollbar - border_width;
    }
    pum_col = cursor_col;
    if set_pum_width_aligned_with_cursor(desired_width, available_width) {
        return;
    }
    if available_width as OptInt > p_pw {
        pum_width = available_width;
        return;
    }
    if pum_rl {
        available_width = max_col - pum_scrollbar - border_width;
    } else {
        available_width += cursor_col;
    }
    if available_width as OptInt > p_pw {
        pum_width = p_pw as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        if pum_rl {
            pum_col = pum_width + pum_scrollbar + border_width;
        } else {
            pum_col = max_col - pum_width - pum_scrollbar - border_width;
        }
        return;
    }
    if pum_rl {
        pum_col = max_col - 1 as ::core::ffi::c_int;
    } else {
        pum_col = 0 as ::core::ffi::c_int;
    }
    pum_width = max_col - pum_scrollbar - border_width;
}
#[inline]
unsafe extern "C" fn pum_border_width() -> ::core::ffi::c_int {
    if *p_pumborder as ::core::ffi::c_int == NUL
        || strequal(
            p_pumborder,
            opt_winborder_values[7 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return if strequal(
        p_pumborder,
        opt_winborder_values[3 as ::core::ffi::c_int as usize]
            as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int != 0
    {
        1 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn pum_display(
    mut array: *mut pumitem_T,
    mut size: ::core::ffi::c_int,
    mut selected: ::core::ffi::c_int,
    mut array_changed: bool,
    mut cmd_startcol: ::core::ffi::c_int,
) {
    let mut redo_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pum_win_row: ::core::ffi::c_int = 0;
    let mut cursor_col: ::core::ffi::c_int = 0;
    if !pum_is_visible {
        pum_external = ui_has(kUIPopupmenu) as ::core::ffi::c_int != 0
            || State & MODE_CMDLINE as ::core::ffi::c_int != 0
                && ui_has(kUIWildmenu) as ::core::ffi::c_int != 0;
    }
    pum_rl = State & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (*curwin).w_onebuf_opt.wo_rl != 0;
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    loop {
        pum_is_visible = true_0 != 0;
        pum_is_drawn = true_0 != 0;
        validate_cursor_col(curwin);
        let mut above_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut below_row: ::core::ffi::c_int = if cmdline_row
            > (*curwin).w_winrow + (*curwin).w_view_height
        {
            cmdline_row
        } else {
            (*curwin).w_winrow + (*curwin).w_view_height
        };
        if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
            below_row = cmdline_row;
        }
        let mut target_win: *mut win_T = if State & MODE_CMDLINE as ::core::ffi::c_int
            != 0
        {
            cmdline_win
        } else {
            curwin
        };
        pum_win_row_offset = 0 as ::core::ffi::c_int;
        pum_win_col_offset = 0 as ::core::ffi::c_int;
        if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
            pum_win_row = if !cmdline_win.is_null() {
                (*cmdline_win).w_wrow
            } else if ui_has(kUICmdline) as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                cmdline_row
            };
            cursor_col = (if !cmdline_win.is_null() {
                (*cmdline_win).w_config._cmdline_offset
            } else {
                0 as ::core::ffi::c_int
            }) + cmd_startcol;
            cursor_col
                %= if !cmdline_win.is_null() {
                    (*cmdline_win).w_view_width
                } else {
                    Columns
                };
            pum_anchor_grid = if ui_has(kUICmdline) as ::core::ffi::c_int != 0 {
                -1 as ::core::ffi::c_int
            } else {
                DEFAULT_GRID_HANDLE
            };
        } else {
            pum_win_row = (*curwin).w_wrow;
            if pum_rl {
                cursor_col = (*curwin).w_view_width - (*curwin).w_wcol
                    - 1 as ::core::ffi::c_int;
            } else {
                cursor_col = (*curwin).w_wcol;
            }
        }
        if !target_win.is_null() {
            pum_anchor_grid = (*(*target_win).w_grid.target).handle
                as ::core::ffi::c_int;
            pum_win_row += (*target_win).w_grid.row_offset;
            cursor_col += (*target_win).w_grid.col_offset;
            if (*target_win).w_grid.target != &raw mut default_grid {
                pum_win_row += (*target_win).w_winrow;
                cursor_col += (*target_win).w_wincol;
                if !ui_has(kUIMultigrid) {
                    pum_anchor_grid = DEFAULT_GRID_HANDLE;
                } else {
                    pum_win_row_offset = (*target_win).w_winrow;
                    pum_win_col_offset = (*target_win).w_wincol;
                }
            }
        }
        if pum_external {
            if array_changed {
                let mut arena: Arena = ARENA_EMPTY;
                let mut arr: Array = arena_array(&raw mut arena, size as size_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < size {
                    let mut item: Array = arena_array(&raw mut arena, 4 as size_t);
                    let c2rust_fresh0 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh0 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_text),
                        },
                    };
                    let c2rust_fresh1 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_kind),
                        },
                    };
                    let c2rust_fresh2 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_extra),
                        },
                    };
                    let c2rust_fresh3 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh3 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_info),
                        },
                    };
                    let c2rust_fresh4 = arr.size;
                    arr.size = arr.size.wrapping_add(1);
                    *arr.items.offset(c2rust_fresh4 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_12 { array: item },
                    };
                    i += 1;
                }
                ui_call_popupmenu_show(
                    arr,
                    selected as Integer,
                    (pum_win_row - pum_win_row_offset) as Integer,
                    (cursor_col - pum_win_col_offset) as Integer,
                    pum_anchor_grid as Integer,
                );
                arena_mem_free(arena_finish(&raw mut arena));
            } else {
                ui_call_popupmenu_select(selected as Integer);
                return;
            }
        }
        let mut pvwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_pvw != 0 {
                pvwin = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if !pvwin.is_null() {
            if (*pvwin).w_winrow < (*curwin).w_winrow {
                above_row = (*pvwin).w_winrow + (*pvwin).w_height;
            } else if (*pvwin).w_winrow > (*curwin).w_winrow + (*curwin).w_height {
                below_row = (*pvwin).w_winrow;
            }
        }
        pum_compute_vertical_placement(
            size,
            target_win,
            pum_win_row,
            above_row,
            below_row,
            border_width,
        );
        if border_width == 0 as ::core::ffi::c_int
            && (pum_height < 1 as ::core::ffi::c_int
                || pum_height == 1 as ::core::ffi::c_int
                    && size > 1 as ::core::ffi::c_int)
        {
            return;
        }
        pum_array = array;
        pum_size = size;
        if pum_external {
            return;
        }
        pum_compute_size();
        pum_scrollbar = if pum_height < size {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        pum_compute_horizontal_placement(target_win, cursor_col, border_width);
        if !(pum_set_selected(selected, redo_count) as ::core::ffi::c_int != 0
            && {
                redo_count += 1;
                redo_count <= 2 as ::core::ffi::c_int
            })
        {
            break;
        }
    }
    pum_grid.zindex = if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        kZIndexCmdlinePopupMenu as ::core::ffi::c_int
    } else {
        kZIndexPopupMenu as ::core::ffi::c_int
    };
    pum_redraw();
}
unsafe extern "C" fn pum_compute_text_attrs(
    mut text: *mut ::core::ffi::c_char,
    mut hlf: hlf_T,
    mut user_hlattr: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_int {
    if *text as ::core::ffi::c_int == NUL
        || hlf as ::core::ffi::c_uint
            != HLF_PSI as ::core::ffi::c_int as ::core::ffi::c_uint
            && hlf as ::core::ffi::c_uint
                != HLF_PNI as ::core::ffi::c_int as ::core::ffi::c_uint
        || win_hl_attr(curwin, HLF_PMSI as ::core::ffi::c_int)
            == win_hl_attr(curwin, HLF_PSI as ::core::ffi::c_int)
            && win_hl_attr(curwin, HLF_PMNI as ::core::ffi::c_int)
                == win_hl_attr(curwin, HLF_PNI as ::core::ffi::c_int)
    {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    let mut leader: *mut ::core::ffi::c_char = if State
        & MODE_CMDLINE as ::core::ffi::c_int != 0
    {
        cmdline_compl_pattern()
    } else {
        ins_compl_leader()
    };
    if leader.is_null() || *leader as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    let mut attrs: *mut ::core::ffi::c_int = xmalloc(
        ::core::mem::size_of::<::core::ffi::c_int>()
            .wrapping_mul(vim_strsize(text) as size_t),
    ) as *mut ::core::ffi::c_int;
    let mut in_fuzzy: bool = if State & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        cmdline_compl_is_fuzzy() as ::core::ffi::c_int
    } else {
        (get_cot_flags() & kOptCotFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
    } != 0;
    let mut leader_len: size_t = strlen(leader);
    let mut ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut matched_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if in_fuzzy {
        ga = fuzzy_match_str_with_pos(text, leader);
        if ga.is_null() {
            xfree(attrs as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
    }
    let mut ptr: *const ::core::ffi::c_char = text;
    let mut cell_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut char_pos: uint32_t = 0 as uint32_t;
    let mut is_select: bool = hlf as ::core::ffi::c_uint
        == HLF_PSI as ::core::ffi::c_int as ::core::ffi::c_uint;
    while *ptr as ::core::ffi::c_int != NUL {
        let mut new_attr: ::core::ffi::c_int = win_hl_attr(
            curwin,
            hlf as ::core::ffi::c_int,
        );
        if !ga.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*ga).ga_len {
                if char_pos == *((*ga).ga_data as *mut uint32_t).offset(i as isize) {
                    new_attr = win_hl_attr(
                        curwin,
                        if is_select as ::core::ffi::c_int != 0 {
                            HLF_PMSI as ::core::ffi::c_int
                        } else {
                            HLF_PMNI as ::core::ffi::c_int
                        },
                    );
                    new_attr = hl_combine_attr(
                        win_hl_attr(curwin, HLF_PMNI as ::core::ffi::c_int),
                        new_attr,
                    );
                    new_attr = hl_combine_attr(
                        win_hl_attr(curwin, hlf as ::core::ffi::c_int),
                        new_attr,
                    );
                    break;
                } else {
                    i += 1;
                }
            }
        } else {
            if matched_len < 0 as ::core::ffi::c_int
                && mb_strnicmp(ptr, leader, leader_len) == 0 as ::core::ffi::c_int
            {
                matched_len = leader_len as ::core::ffi::c_int;
            }
            if matched_len > 0 as ::core::ffi::c_int {
                new_attr = win_hl_attr(
                    curwin,
                    if is_select as ::core::ffi::c_int != 0 {
                        HLF_PMSI as ::core::ffi::c_int
                    } else {
                        HLF_PMNI as ::core::ffi::c_int
                    },
                );
                new_attr = hl_combine_attr(
                    win_hl_attr(curwin, HLF_PMNI as ::core::ffi::c_int),
                    new_attr,
                );
                new_attr = hl_combine_attr(
                    win_hl_attr(curwin, hlf as ::core::ffi::c_int),
                    new_attr,
                );
                matched_len -= 1;
            }
        }
        new_attr = hl_combine_attr(
            win_hl_attr(curwin, HLF_PNI as ::core::ffi::c_int),
            new_attr,
        );
        if user_hlattr > 0 as ::core::ffi::c_int {
            new_attr = hl_combine_attr(new_attr, user_hlattr);
        }
        let mut char_cells: ::core::ffi::c_int = utf_ptr2cells(ptr);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < char_cells {
            *attrs.offset((cell_idx + i_0) as isize) = new_attr;
            i_0 += 1;
        }
        cell_idx += char_cells;
        ptr = ptr.offset(utfc_ptr2len(ptr as *mut ::core::ffi::c_char) as isize);
        char_pos = char_pos.wrapping_add(1);
    }
    if !ga.is_null() {
        ga_clear(ga);
        xfree(ga as *mut ::core::ffi::c_void);
    }
    return attrs;
}
unsafe extern "C" fn pum_grid_puts_with_attrs(
    mut col: ::core::ffi::c_int,
    mut cells: ::core::ffi::c_int,
    mut text: *const ::core::ffi::c_char,
    mut textlen: ::core::ffi::c_int,
    mut attrs: *const ::core::ffi::c_int,
) {
    let col_start: ::core::ffi::c_int = col;
    let mut ptr: *const ::core::ffi::c_char = text;
    while *ptr as ::core::ffi::c_int != NUL
        && (textlen < 0 as ::core::ffi::c_int || ptr < text.offset(textlen as isize))
    {
        let mut char_len: ::core::ffi::c_int = utfc_ptr2len(ptr);
        let mut attr: ::core::ffi::c_int = *attrs
            .offset(
                (if pum_rl as ::core::ffi::c_int != 0 {
                    col_start + cells - col - 1 as ::core::ffi::c_int
                } else {
                    col - col_start
                }) as isize,
            );
        grid_line_puts(col, ptr, char_len, attr);
        col += utf_ptr2cells(ptr);
        ptr = ptr.offset(char_len as isize);
    }
}
#[inline]
unsafe extern "C" fn pum_align_order(mut order: *mut ::core::ffi::c_int) {
    let mut is_default: bool = cia_flags == 0 as ::core::ffi::c_uint;
    *order.offset(0 as ::core::ffi::c_int as isize) = (if is_default
        as ::core::ffi::c_int != 0
    {
        CPT_ABBR as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        cia_flags.wrapping_div(100 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
    *order.offset(1 as ::core::ffi::c_int as isize) = (if is_default
        as ::core::ffi::c_int != 0
    {
        CPT_KIND as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        cia_flags
            .wrapping_div(10 as ::core::ffi::c_uint)
            .wrapping_rem(10 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
    *order.offset(2 as ::core::ffi::c_int as isize) = (if is_default
        as ::core::ffi::c_int != 0
    {
        CPT_MENU as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        cia_flags.wrapping_rem(10 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn pum_get_item(
    mut index: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match type_0 {
        0 => return (*pum_array.offset(index as isize)).pum_text,
        1 => return (*pum_array.offset(index as isize)).pum_kind,
        2 => return (*pum_array.offset(index as isize)).pum_extra,
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[inline]
unsafe extern "C" fn pum_user_attr_combine(
    mut idx: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut user_attr: [::core::ffi::c_int; 2] = [
        (*pum_array.offset(idx as isize)).pum_user_abbr_hlattr,
        (*pum_array.offset(idx as isize)).pum_user_kind_hlattr,
    ];
    return if user_attr[type_0 as usize] > 0 as ::core::ffi::c_int {
        hl_combine_attr(attr, user_attr[type_0 as usize])
    } else {
        attr
    };
}
#[no_mangle]
pub unsafe extern "C" fn pum_redraw() {
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr_scroll: ::core::ffi::c_int = win_hl_attr(
        curwin,
        HLF_PSB as ::core::ffi::c_int,
    );
    let mut attr_thumb: ::core::ffi::c_int = win_hl_attr(
        curwin,
        HLF_PST as ::core::ffi::c_int,
    );
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut thumb_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut thumb_height: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let fcs_trunc: schar_T = if pum_rl as ::core::ffi::c_int != 0 {
        (*curwin).w_p_fcs_chars.truncrl
    } else {
        (*curwin).w_p_fcs_chars.trunc
    };
    let hlfsNorm: [hlf_T; 3] = [HLF_PNI, HLF_PNK, HLF_PNX];
    let hlfsSel: [hlf_T; 3] = [HLF_PSI, HLF_PSK, HLF_PSX];
    let mut grid_width: ::core::ffi::c_int = pum_width;
    let mut col_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut extra_space: bool = false_0 != 0;
    if pum_rl {
        col_off = pum_width - 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if State & MODE_CMDLINE as ::core::ffi::c_int == 0 {} else {
                __assert_fail(
                    b"!(State & MODE_CMDLINE)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/popupmenu.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    584 as ::core::ffi::c_uint,
                    b"void pum_redraw(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut win_end_col: ::core::ffi::c_int = (*curwin).w_wincol + (*curwin).w_width;
        if pum_col < win_end_col - 1 as ::core::ffi::c_int {
            grid_width += 1 as ::core::ffi::c_int;
            extra_space = true_0 != 0;
        }
    } else {
        let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pum_col > min_col {
            grid_width += 1 as ::core::ffi::c_int;
            col_off = 1 as ::core::ffi::c_int;
            extra_space = true_0 != 0;
        }
    }
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut border_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut border_char: schar_T = 0 as schar_T;
    let mut fill_char: schar_T = ' ' as ::core::ffi::c_int as schar_T;
    let mut has_border: bool = border_width > 0 as ::core::ffi::c_int;
    if border_width > 0 as ::core::ffi::c_int {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if !parse_winborder(&raw mut fconfig, p_pumborder, &raw mut err) {
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                emsg(err.msg);
            }
            api_clear_error(&raw mut err);
            return;
        }
        if strequal(
            p_pumborder,
            opt_winborder_values[3 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
        ) {
            fconfig.shadow = true_0 != 0;
            let mut blend: ::core::ffi::c_int = syn_check_group(
                b"PmenuShadow\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>()
                    .wrapping_sub(1 as size_t),
            );
            let mut through: ::core::ffi::c_int = syn_check_group(
                b"PmenuShadowThrough\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                    .wrapping_sub(1 as size_t),
            );
            fconfig.border_hl_ids[2 as ::core::ffi::c_int as usize] = through;
            fconfig.border_hl_ids[3 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[4 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[5 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[6 as ::core::ffi::c_int as usize] = through;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 8 as ::core::ffi::c_int {
            let mut attr: ::core::ffi::c_int = *hl_attr_active
                .offset(HLF_PBR as ::core::ffi::c_int as isize);
            if fconfig.border_hl_ids[i as usize] != 0 {
                attr = hl_get_ui_attr(
                    -1 as ::core::ffi::c_int,
                    HLF_PBR as ::core::ffi::c_int,
                    fconfig.border_hl_ids[i as usize],
                    false_0 != 0,
                );
            }
            fconfig.border_attr[i as usize] = attr;
            i += 1;
        }
        api_clear_error(&raw mut err);
        if pum_scrollbar != 0 {
            border_char = schar_from_str(
                &raw mut *(&raw mut fconfig.border_chars
                    as *mut [::core::ffi::c_char; 32])
                    .offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
            );
            border_attr = fconfig.border_attr[3 as ::core::ffi::c_int as usize];
        }
    }
    if pum_scrollbar > 0 as ::core::ffi::c_int
        && (!fconfig.border || fconfig.shadow as ::core::ffi::c_int != 0)
    {
        grid_width += 1;
        if pum_rl {
            col_off += 1;
        }
    }
    pum_grid.blending = p_pb > 0 as OptInt || fconfig.shadow as ::core::ffi::c_int != 0;
    grid_assign_handle(&raw mut pum_grid);
    pum_left_col = pum_col - col_off;
    pum_right_col = pum_left_col + grid_width;
    let mut moved: bool = ui_comp_put_grid(
        &raw mut pum_grid,
        pum_row,
        pum_left_col,
        pum_height + border_width,
        grid_width + border_width,
        false_0 != 0,
        true_0 != 0,
    );
    let mut invalid_grid: bool = moved as ::core::ffi::c_int != 0
        || pum_invalid as ::core::ffi::c_int != 0;
    pum_invalid = false_0 != 0;
    must_redraw_pum = false_0 != 0;
    if pum_grid.chars.is_null() || pum_grid.rows != pum_height + border_width
        || pum_grid.cols != grid_width + border_width
    {
        grid_alloc(
            &raw mut pum_grid,
            pum_height + border_width,
            grid_width + border_width,
            !invalid_grid,
            false_0 != 0,
        );
        ui_call_grid_resize(
            pum_grid.handle as Integer,
            pum_grid.cols as Integer,
            pum_grid.rows as Integer,
        );
    } else if invalid_grid {
        grid_invalidate(&raw mut pum_grid);
    }
    if ui_has(kUIMultigrid) {
        let mut anchor: *const ::core::ffi::c_char = if pum_above as ::core::ffi::c_int
            != 0
        {
            b"SW\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NW\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut row_off: ::core::ffi::c_int = if pum_above as ::core::ffi::c_int != 0 {
            -pum_height
        } else {
            0 as ::core::ffi::c_int
        };
        ui_call_win_float_pos(
            pum_grid.handle as Integer,
            -1 as Window,
            cstr_as_string(anchor),
            pum_anchor_grid as Integer,
            (pum_row - row_off - pum_win_row_offset) as Float,
            (pum_left_col - pum_win_col_offset) as Float,
            false_0 != 0,
            pum_grid.zindex as Integer,
            pum_grid.comp_index as ::core::ffi::c_int as Integer,
            pum_grid.comp_row as Integer,
            pum_grid.comp_col as Integer,
        );
    }
    let mut scroll_range: ::core::ffi::c_int = pum_size - pum_height;
    if fconfig.border {
        grid_draw_border(
            &raw mut pum_grid,
            &raw mut fconfig,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if !fconfig.shadow {
            row += 1;
            col_off += 1;
        }
    }
    pum_first = if pum_first < scroll_range { pum_first } else { scroll_range };
    if pum_scrollbar != 0 {
        thumb_height = pum_height * pum_height / pum_size;
        if thumb_height == 0 as ::core::ffi::c_int {
            thumb_height = 1 as ::core::ffi::c_int;
        }
        thumb_pos = (pum_first * (pum_height - thumb_height)
            + scroll_range / 2 as ::core::ffi::c_int) / scroll_range;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < pum_height {
        let mut idx: ::core::ffi::c_int = i_0 + pum_first;
        let selected: bool = idx == pum_selected;
        let hlfs: *const hlf_T = if selected as ::core::ffi::c_int != 0 {
            &raw const hlfsSel as *const hlf_T
        } else {
            &raw const hlfsNorm as *const hlf_T
        };
        let trunc_attr: ::core::ffi::c_int = win_hl_attr(
            curwin,
            if selected as ::core::ffi::c_int != 0 {
                HLF_PSI as ::core::ffi::c_int
            } else {
                HLF_PNI as ::core::ffi::c_int
            },
        );
        let mut hlf: hlf_T = *hlfs.offset(0 as ::core::ffi::c_int as isize);
        let mut attr_0: ::core::ffi::c_int = win_hl_attr(
            curwin,
            hlf as ::core::ffi::c_int,
        );
        attr_0 = hl_combine_attr(
            win_hl_attr(curwin, HLF_PNI as ::core::ffi::c_int),
            attr_0,
        );
        screengrid_line_start(&raw mut pum_grid, row, 0 as ::core::ffi::c_int);
        if extra_space {
            if pum_rl {
                grid_line_puts(
                    col_off + 1 as ::core::ffi::c_int,
                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    attr_0,
                );
            } else {
                grid_line_puts(
                    col_off - 1 as ::core::ffi::c_int,
                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    attr_0,
                );
            }
        }
        let mut grid_col: ::core::ffi::c_int = col_off;
        let mut totwidth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut need_fcs_trunc: bool = false_0 != 0;
        let mut order: [::core::ffi::c_int; 3] = [0; 3];
        let mut items_width_array: [::core::ffi::c_int; 3] = [
            pum_base_width,
            pum_kind_width,
            pum_extra_width,
        ];
        pum_align_order(&raw mut order as *mut ::core::ffi::c_int);
        let mut basic_width: ::core::ffi::c_int = items_width_array[order[0
            as ::core::ffi::c_int as usize] as usize];
        let mut last_isabbr: bool = order[2 as ::core::ffi::c_int as usize]
            == CPT_ABBR as ::core::ffi::c_int;
        let mut orig_attr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 3 as ::core::ffi::c_int {
            let mut item_type: ::core::ffi::c_int = order[j as usize];
            hlf = *hlfs.offset(item_type as isize);
            attr_0 = win_hl_attr(curwin, hlf as ::core::ffi::c_int);
            attr_0 = hl_combine_attr(
                win_hl_attr(curwin, HLF_PNI as ::core::ffi::c_int),
                attr_0,
            );
            orig_attr = attr_0;
            if item_type < 2 as ::core::ffi::c_int {
                attr_0 = pum_user_attr_combine(idx, item_type, attr_0);
            }
            let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            p = pum_get_item(idx, item_type);
            let next_isempty: bool = j + 1 as ::core::ffi::c_int
                >= 3 as ::core::ffi::c_int
                || pum_get_item(idx, order[(j + 1 as ::core::ffi::c_int) as usize])
                    .is_null();
            if !p.is_null() {
                loop {
                    if s.is_null() {
                        s = p;
                    }
                    let mut w: ::core::ffi::c_int = ptr2cells(p);
                    if *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != TAB
                        && totwidth + w <= pum_width
                    {
                        width += w;
                    } else {
                        let width_limit: ::core::ffi::c_int = pum_width;
                        let mut saved: ::core::ffi::c_char = *p;
                        if saved as ::core::ffi::c_int != NUL {
                            *p = NUL as ::core::ffi::c_char;
                        }
                        let mut st: *mut ::core::ffi::c_char = transstr(s, true_0 != 0);
                        if saved as ::core::ffi::c_int != NUL {
                            *p = saved;
                        }
                        let mut attrs: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<
                            ::core::ffi::c_int,
                        >();
                        if item_type == CPT_ABBR as ::core::ffi::c_int {
                            attrs = pum_compute_text_attrs(
                                st,
                                hlf,
                                (*pum_array.offset(idx as isize)).pum_user_abbr_hlattr,
                            );
                        }
                        if pum_rl {
                            let mut rt: *mut ::core::ffi::c_char = reverse_text(st);
                            let mut rt_start: *mut ::core::ffi::c_char = rt;
                            let mut cells: ::core::ffi::c_int = mb_string2cells(rt)
                                as ::core::ffi::c_int;
                            let mut pad: ::core::ffi::c_int = if next_isempty
                                as ::core::ffi::c_int != 0
                            {
                                0 as ::core::ffi::c_int
                            } else {
                                2 as ::core::ffi::c_int
                            };
                            if width_limit - totwidth < cells + pad {
                                need_fcs_trunc = true_0 != 0;
                            }
                            if grid_col - cells < col_off - width_limit {
                                loop {
                                    cells -= utf_ptr2cells(rt);
                                    rt = rt.offset(utfc_ptr2len(rt) as isize);
                                    if grid_col - cells >= col_off - width_limit {
                                        break;
                                    }
                                }
                                if grid_col - cells > col_off - width_limit {
                                    rt = rt.offset(-1);
                                    *rt = '<' as ::core::ffi::c_char;
                                    cells += 1;
                                }
                            }
                            if attrs.is_null() {
                                grid_line_puts(
                                    grid_col - cells + 1 as ::core::ffi::c_int,
                                    rt,
                                    -1 as ::core::ffi::c_int,
                                    attr_0,
                                );
                            } else {
                                pum_grid_puts_with_attrs(
                                    grid_col - cells + 1 as ::core::ffi::c_int,
                                    cells,
                                    rt,
                                    -1 as ::core::ffi::c_int,
                                    attrs,
                                );
                            }
                            xfree(rt_start as *mut ::core::ffi::c_void);
                            xfree(st as *mut ::core::ffi::c_void);
                            grid_col -= width;
                        } else {
                            let mut cells_0: ::core::ffi::c_int = mb_string2cells(st)
                                as ::core::ffi::c_int;
                            let mut pad_0: ::core::ffi::c_int = if next_isempty
                                as ::core::ffi::c_int != 0
                            {
                                0 as ::core::ffi::c_int
                            } else {
                                2 as ::core::ffi::c_int
                            };
                            if width_limit - totwidth < cells_0 + pad_0 {
                                need_fcs_trunc = true_0 != 0;
                            }
                            if need_fcs_trunc {
                                let mut available_cells: ::core::ffi::c_int = width_limit
                                    - totwidth;
                                let mut p_end: *mut ::core::ffi::c_char = st;
                                let mut displayed: ::core::ffi::c_int = 0
                                    as ::core::ffi::c_int;
                                while *p_end as ::core::ffi::c_int != NUL {
                                    let mut char_cells: ::core::ffi::c_int = utf_ptr2cells(
                                        p_end,
                                    );
                                    if displayed + char_cells > available_cells {
                                        break;
                                    }
                                    displayed += char_cells;
                                    p_end = p_end.offset(utfc_ptr2len(p_end) as isize);
                                }
                                *p_end = NUL as ::core::ffi::c_char;
                                cells_0 = displayed;
                                width = displayed;
                            }
                            if attrs.is_null() {
                                grid_line_puts(
                                    grid_col,
                                    st,
                                    -1 as ::core::ffi::c_int,
                                    attr_0,
                                );
                            } else {
                                pum_grid_puts_with_attrs(
                                    grid_col,
                                    cells_0,
                                    st,
                                    -1 as ::core::ffi::c_int,
                                    attrs,
                                );
                            }
                            xfree(st as *mut ::core::ffi::c_void);
                            grid_col += width;
                        }
                        if !attrs.is_null() {
                            let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut attrs
                                as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            *ptr_;
                        }
                        if *p as ::core::ffi::c_int != TAB {
                            break;
                        }
                        if pum_rl {
                            grid_line_puts(
                                grid_col - 1 as ::core::ffi::c_int,
                                b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                                attr_0,
                            );
                            grid_col -= 2 as ::core::ffi::c_int;
                        } else {
                            grid_line_puts(
                                grid_col,
                                b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                                attr_0,
                            );
                            grid_col += 2 as ::core::ffi::c_int;
                        }
                        totwidth += 2 as ::core::ffi::c_int;
                        s = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        width = 0 as ::core::ffi::c_int;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            }
            if j > 0 as ::core::ffi::c_int {
                n = items_width_array[order[1 as ::core::ffi::c_int as usize] as usize]
                    + (if last_isabbr as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    });
            } else {
                n = if order[j as usize] == CPT_ABBR as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            if j == 2 as ::core::ffi::c_int
                || next_isempty as ::core::ffi::c_int != 0
                    && (j == 1 as ::core::ffi::c_int
                        || j == 0 as ::core::ffi::c_int
                            && pum_get_item(
                                    idx,
                                    order[(j + 2 as ::core::ffi::c_int) as usize],
                                )
                                .is_null()) || basic_width + n >= pum_width
            {
                break;
            }
            if pum_rl {
                grid_line_fill(
                    col_off - basic_width - n + 1 as ::core::ffi::c_int,
                    grid_col + 1 as ::core::ffi::c_int,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                grid_col = col_off - basic_width - n;
            } else {
                grid_line_fill(
                    grid_col,
                    col_off + basic_width + n,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                grid_col = col_off + basic_width + n;
            }
            totwidth = basic_width + n;
            j += 1;
        }
        if pum_rl {
            let lcol: ::core::ffi::c_int = col_off - pum_width + 1 as ::core::ffi::c_int;
            grid_line_fill(
                lcol,
                grid_col + 1 as ::core::ffi::c_int,
                ' ' as ::core::ffi::c_int as schar_T,
                orig_attr,
            );
            if need_fcs_trunc {
                *linebuf_char.offset(lcol as isize) = if fcs_trunc != NUL as schar_T {
                    fcs_trunc
                } else {
                    '<' as ::core::ffi::c_int as schar_T
                };
                *linebuf_attr.offset(lcol as isize) = trunc_attr as sattr_T;
                if pum_width > 1 as ::core::ffi::c_int
                    && *linebuf_char.offset((lcol + 1 as ::core::ffi::c_int) as isize)
                        == NUL as schar_T
                {
                    *linebuf_char.offset((lcol + 1 as ::core::ffi::c_int) as isize) = ' '
                        as ::core::ffi::c_int as schar_T;
                }
            }
        } else {
            let rcol: ::core::ffi::c_int = col_off + pum_width;
            grid_line_fill(
                grid_col,
                rcol,
                ' ' as ::core::ffi::c_int as schar_T,
                orig_attr,
            );
            if need_fcs_trunc {
                if pum_width > 1 as ::core::ffi::c_int
                    && *linebuf_char.offset((rcol - 1 as ::core::ffi::c_int) as isize)
                        == NUL as schar_T
                {
                    *linebuf_char.offset((rcol - 2 as ::core::ffi::c_int) as isize) = ' '
                        as ::core::ffi::c_int as schar_T;
                }
                *linebuf_char.offset((rcol - 1 as ::core::ffi::c_int) as isize) = if fcs_trunc
                    != NUL as schar_T
                {
                    fcs_trunc
                } else {
                    '>' as ::core::ffi::c_int as schar_T
                };
                *linebuf_attr.offset((rcol - 1 as ::core::ffi::c_int) as isize) = trunc_attr
                    as sattr_T;
            }
        }
        if pum_scrollbar > 0 as ::core::ffi::c_int {
            let mut thumb: bool = i_0 >= thumb_pos && i_0 < thumb_pos + thumb_height;
            let mut scrollbar_col: ::core::ffi::c_int = col_off
                + (if pum_rl as ::core::ffi::c_int != 0 {
                    -pum_width
                } else {
                    pum_width
                });
            let mut use_border_style: bool = has_border as ::core::ffi::c_int != 0
                && !fconfig.shadow;
            grid_line_put_schar(
                scrollbar_col,
                if use_border_style as ::core::ffi::c_int != 0 && !thumb {
                    border_char
                } else {
                    fill_char
                },
                if thumb as ::core::ffi::c_int != 0 {
                    attr_thumb
                } else if use_border_style as ::core::ffi::c_int != 0 {
                    border_attr
                } else {
                    attr_scroll
                },
            );
        }
        grid_line_flush();
        row += 1;
        i_0 += 1;
    }
}
unsafe extern "C" fn pum_preview_set_text(
    mut win: *mut win_T,
    mut info: *mut ::core::ffi::c_char,
    mut lnum: *mut linenr_T,
    mut max_width: *mut ::core::ffi::c_int,
) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut replacement: Array = ARRAY_DICT_INIT;
    let mut buf: *mut buf_T = (*win).w_buffer;
    (*buf).b_p_ma = true_0;
    let mut curr: *mut ::core::ffi::c_char = info;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    while !curr.is_null() {
        next = strchr(curr, '\n' as ::core::ffi::c_int);
        if !next.is_null() {
            *next = NUL as ::core::ffi::c_char;
        }
        if *curr as ::core::ffi::c_int == NUL && next.is_null() {
            break;
        }
        let mut save_wrap: bool = (*win).w_onebuf_opt.wo_wrap != 0;
        (*win).w_onebuf_opt.wo_wrap = false_0;
        let mut line_width: ::core::ffi::c_int = win_linetabsize(
            win,
            0 as linenr_T,
            curr,
            MAXCOL as ::core::ffi::c_int,
        );
        (*win).w_onebuf_opt.wo_wrap = save_wrap as ::core::ffi::c_int;
        *max_width = if *max_width > line_width { *max_width } else { line_width };
        if replacement.size == replacement.capacity {
            replacement.capacity = (if replacement.capacity != 0 {
                replacement.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            });
            replacement.items = xrealloc(
                replacement.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(replacement.capacity),
            ) as *mut Object;
        } else {};
        let c2rust_fresh5 = replacement.size;
        replacement.size = replacement.size.wrapping_add(1);
        *replacement.items.offset(c2rust_fresh5 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_12 {
                string: cstr_to_string(curr),
            },
        };
        *lnum += 1;
        if !next.is_null() {
            *next = '\n' as ::core::ffi::c_char;
        }
        curr = if !next.is_null() {
            next.offset(1 as ::core::ffi::c_int as isize)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
    let mut original_textlock: ::core::ffi::c_int = textlock;
    textlock = 0 as ::core::ffi::c_int;
    nvim_buf_set_lines(
        0 as uint64_t,
        (*buf).handle as Buffer,
        0 as Integer,
        -1 as Integer,
        false_0 != 0,
        replacement,
        &raw mut arena,
        &raw mut err,
    );
    textlock = original_textlock;
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg(err.msg);
        api_clear_error(&raw mut err);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    api_free_array(replacement);
    (*buf).b_p_ma = false_0;
}
unsafe extern "C" fn pum_adjust_info_position(
    mut wp: *mut win_T,
    mut width: ::core::ffi::c_int,
) -> bool {
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut col: ::core::ffi::c_int = pum_col + pum_width + 1 as ::core::ffi::c_int
        + (if border_width > pum_scrollbar { border_width } else { pum_scrollbar });
    let mut right_extra: ::core::ffi::c_int = Columns - col;
    let mut left_extra: ::core::ffi::c_int = pum_col - 2 as ::core::ffi::c_int;
    let mut max_extra: ::core::ffi::c_int = if right_extra > left_extra {
        right_extra
    } else {
        left_extra
    };
    if max_extra < 10 as ::core::ffi::c_int {
        (*wp).w_config.hide = true_0 != 0;
        return false_0 != 0;
    }
    if right_extra > width {
        (*wp).w_config.width = width;
        (*wp).w_config.col = (col - 1 as ::core::ffi::c_int) as ::core::ffi::c_double;
    } else if left_extra > width {
        (*wp).w_config.width = width;
        (*wp).w_config.col = (pum_col - (*wp).w_config.width - 1 as ::core::ffi::c_int)
            as ::core::ffi::c_double;
    } else {
        let place_in_right: bool = right_extra > left_extra;
        (*wp).w_config.width = max_extra;
        (*wp).w_config.col = (if place_in_right as ::core::ffi::c_int != 0 {
            col - 1 as ::core::ffi::c_int
        } else {
            pum_col - (*wp).w_config.width - 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_double;
    }
    (*wp).w_config.anchor = 0 as ::core::ffi::c_int as FloatAnchor;
    let mut count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
    (*wp).w_view_width = (*wp).w_config.width;
    (*wp).w_config.height = plines_m_win(wp, (*wp).w_topline, count, Rows);
    (*wp).w_config.row = pum_row as ::core::ffi::c_double;
    (*wp).w_config.hide = false_0 != 0;
    win_config_float(wp, (*wp).w_config);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn pum_set_info(
    mut selected: ::core::ffi::c_int,
    mut info: *mut ::core::ffi::c_char,
) -> *mut win_T {
    if !pum_is_visible || !compl_match_curr_select(selected) {
        return ::core::ptr::null_mut::<win_T>();
    }
    block_autocmds();
    RedrawingDisabled += 1;
    no_u_sync += 1;
    let mut wp: *mut win_T = win_float_find_preview();
    if wp.is_null() {
        wp = win_float_create_preview(false_0 != 0, true_0 != 0);
        if wp.is_null() {
            return ::core::ptr::null_mut::<win_T>();
        }
        (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_onebuf_opt.wo_wfb = true_0;
    }
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    pum_preview_set_text(wp, info, &raw mut lnum, &raw mut max_info_width);
    no_u_sync -= 1;
    RedrawingDisabled -= 1;
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    if !pum_adjust_info_position(wp, max_info_width) {
        wp = ::core::ptr::null_mut::<win_T>();
    }
    unblock_autocmds();
    return wp;
}
unsafe extern "C" fn pum_set_selected(
    mut n: ::core::ffi::c_int,
    mut repeat: ::core::ffi::c_int,
) -> bool {
    let mut resized: bool = false_0 != 0;
    let mut context: ::core::ffi::c_int = pum_height / 2 as ::core::ffi::c_int;
    let mut prev_selected: ::core::ffi::c_int = pum_selected;
    pum_selected = n;
    let mut scroll_offset: ::core::ffi::c_int = pum_selected - pum_height;
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    let mut use_float: bool = cur_cot_flags
        & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    if use_float as ::core::ffi::c_int != 0
        && (pum_selected < 0 as ::core::ffi::c_int
            || (*pum_array.offset(pum_selected as isize)).pum_info.is_null())
    {
        let mut wp: *mut win_T = win_float_find_preview();
        if !wp.is_null() {
            (*wp).w_config.hide = true_0 != 0;
            win_config_float(wp, (*wp).w_config);
        }
    }
    if pum_selected >= 0 as ::core::ffi::c_int && pum_selected < pum_size {
        if pum_first > pum_selected - 4 as ::core::ffi::c_int {
            if pum_first > pum_selected - 2 as ::core::ffi::c_int {
                pum_first -= pum_height - 2 as ::core::ffi::c_int;
                if pum_first < 0 as ::core::ffi::c_int {
                    pum_first = 0 as ::core::ffi::c_int;
                } else if pum_first > pum_selected {
                    pum_first = pum_selected;
                }
            } else {
                pum_first = pum_selected;
            }
        } else if pum_first < scroll_offset + 5 as ::core::ffi::c_int {
            if pum_first < scroll_offset + 3 as ::core::ffi::c_int {
                pum_first = if pum_first + pum_height - 2 as ::core::ffi::c_int
                    > scroll_offset + 1 as ::core::ffi::c_int
                {
                    pum_first + pum_height - 2 as ::core::ffi::c_int
                } else {
                    scroll_offset + 1 as ::core::ffi::c_int
                };
            } else {
                pum_first = scroll_offset + 1 as ::core::ffi::c_int;
            }
        }
        context = if context < 3 as ::core::ffi::c_int {
            context
        } else {
            3 as ::core::ffi::c_int
        };
        if pum_height > 2 as ::core::ffi::c_int {
            if pum_first > pum_selected - context {
                pum_first = if pum_selected - context > 0 as ::core::ffi::c_int {
                    pum_selected - context
                } else {
                    0 as ::core::ffi::c_int
                };
            } else if pum_first
                < pum_selected + context - pum_height + 1 as ::core::ffi::c_int
            {
                pum_first = pum_selected + context - pum_height
                    + 1 as ::core::ffi::c_int;
            }
        }
        pum_first = if pum_first < pum_size - pum_height {
            pum_first
        } else {
            pum_size - pum_height
        };
        if !(*pum_array.offset(pum_selected as isize)).pum_info.is_null()
            && Rows > 10 as ::core::ffi::c_int && repeat <= 1 as ::core::ffi::c_int
            && cur_cot_flags
                & (kOptCotFlagPreview as ::core::ffi::c_int
                    | kOptCotFlagPopup as ::core::ffi::c_int) as ::core::ffi::c_uint != 0
            && !(cur_cot_flags
                & kOptCotFlagPreview as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                && cmdwin_type != 0 as ::core::ffi::c_int)
        {
            let mut curwin_save: *mut win_T = curwin;
            let mut curtab_save: *mut tabpage_T = curtab;
            if use_float {
                block_autocmds();
            }
            g_do_tagpreview = 3 as ::core::ffi::c_int;
            if p_pvh > 0 as OptInt && p_pvh < g_do_tagpreview as OptInt {
                g_do_tagpreview = p_pvh as ::core::ffi::c_int;
            }
            RedrawingDisabled += 1;
            no_u_sync += 1;
            if !use_float {
                resized = prepare_tagpreview(false_0 != 0);
            } else {
                let mut wp_0: *mut win_T = win_float_find_preview();
                if !wp_0.is_null() {
                    win_enter(wp_0, false_0 != 0);
                } else {
                    wp_0 = win_float_create_preview(true_0 != 0, true_0 != 0);
                    if !wp_0.is_null() {
                        resized = true_0 != 0;
                    }
                }
            }
            no_u_sync -= 1;
            RedrawingDisabled -= 1;
            g_do_tagpreview = 0 as ::core::ffi::c_int;
            if (*curwin).w_onebuf_opt.wo_pvw != 0
                || (*curwin).w_float_is_info as ::core::ffi::c_int != 0
            {
                let mut res: ::core::ffi::c_int = OK;
                if !resized && (*curbuf).b_nwindows == 1 as ::core::ffi::c_int
                    && (*curbuf).b_fname.is_null()
                    && bt_nofile(curbuf) as ::core::ffi::c_int != 0
                    && *(*curbuf).b_p_bh.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int == 'w' as ::core::ffi::c_int
                {
                    buf_clear();
                } else {
                    no_u_sync += 1;
                    res = do_ecmd(
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<exarg_T>(),
                        ECMD_ONE as ::core::ffi::c_int as linenr_T,
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<win_T>(),
                    );
                    no_u_sync -= 1;
                    if res == OK {
                        set_option_value_give_err(
                            kOptSwapfile,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBuflisted,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBuftype,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: String_0 {
                                        data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                        size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                            .wrapping_sub(1 as size_t),
                                    },
                                },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBufhidden,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: String_0 {
                                        data: b"wipe\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                            .wrapping_sub(1 as size_t),
                                    },
                                },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptDiff,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                    }
                }
                if res == OK {
                    let mut lnum: linenr_T = 0 as linenr_T;
                    let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    pum_preview_set_text(
                        curwin,
                        (*pum_array.offset(pum_selected as isize)).pum_info,
                        &raw mut lnum,
                        &raw mut max_info_width,
                    );
                    if repeat == 0 as ::core::ffi::c_int && !use_float {
                        lnum = if lnum < p_pvh as linenr_T {
                            lnum
                        } else {
                            p_pvh as linenr_T
                        };
                        if ((*curwin).w_height as linenr_T) < lnum {
                            win_setheight(lnum as ::core::ffi::c_int);
                            resized = true_0 != 0;
                        }
                    }
                    (*curbuf).b_changed = false_0;
                    (*curbuf).b_p_ma = false_0;
                    if pum_selected != prev_selected {
                        (*curwin).w_topline = 1 as ::core::ffi::c_int as linenr_T;
                    } else if (*curwin).w_topline > (*curbuf).b_ml.ml_line_count {
                        (*curwin).w_topline = (*curbuf).b_ml.ml_line_count;
                    }
                    (*curwin).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    if use_float {
                        if !pum_adjust_info_position(curwin, max_info_width)
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            win_enter(curwin_save, false_0 != 0);
                        }
                    }
                    if curwin != curwin_save
                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        || curtab != curtab_save
                            && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                    {
                        if curtab != curtab_save
                            && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                        {
                            goto_tabpage_tp(curtab_save, false_0 != 0, false_0 != 0);
                        }
                        if ins_compl_active() as ::core::ffi::c_int != 0 && !resized {
                            (*curwin).w_redr_status = false_0 != 0;
                        }
                        validate_cursor(curwin);
                        redraw_later(curwin, UPD_SOME_VALID as ::core::ffi::c_int);
                        if resized as ::core::ffi::c_int != 0
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            no_u_sync += 1;
                            win_enter(curwin_save, true_0 != 0);
                            no_u_sync -= 1;
                            update_topline(curwin);
                        }
                        pum_is_visible = false_0 != 0;
                        update_screen();
                        pum_is_visible = true_0 != 0;
                        if !resized && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            no_u_sync += 1;
                            win_enter(curwin_save, true_0 != 0);
                            no_u_sync -= 1;
                        }
                        pum_is_visible = false_0 != 0;
                        update_screen();
                        pum_is_visible = true_0 != 0;
                    }
                }
            }
            if use_float {
                unblock_autocmds();
            }
        }
    }
    return resized;
}
#[no_mangle]
pub unsafe extern "C" fn pum_undisplay(mut immediate: bool) {
    pum_is_visible = false_0 != 0;
    pum_array = ::core::ptr::null_mut::<pumitem_T>();
    must_redraw_pum = false_0 != 0;
    if immediate {
        pum_check_clear();
    }
}
#[no_mangle]
pub unsafe extern "C" fn pum_check_clear() {
    if !pum_is_visible && pum_is_drawn as ::core::ffi::c_int != 0 {
        if pum_external {
            ui_call_popupmenu_hide();
        } else {
            ui_comp_remove_grid(&raw mut pum_grid);
            if ui_has(kUIMultigrid) {
                ui_call_win_close(pum_grid.handle as Integer);
                ui_call_grid_destroy(pum_grid.handle as Integer);
            }
            grid_free(&raw mut pum_grid);
        }
        pum_is_drawn = false_0 != 0;
        pum_external = false_0 != 0;
        let mut wp: *mut win_T = win_float_find_preview();
        if !wp.is_null() {
            win_close(wp, false_0 != 0, false_0 != 0);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pum_clear() {
    pum_first = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn pum_visible() -> bool {
    return pum_is_visible;
}
#[no_mangle]
pub unsafe extern "C" fn pum_drawn() -> bool {
    return pum_visible() as ::core::ffi::c_int != 0 && !pum_external;
}
#[no_mangle]
pub unsafe extern "C" fn pum_invalidate() {
    pum_invalid = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn pum_recompose() {
    ui_comp_compose_grid(&raw mut pum_grid);
}
#[no_mangle]
pub unsafe extern "C" fn pum_ext_select_item(
    mut item: ::core::ffi::c_int,
    mut insert: bool,
    mut finish: bool,
) {
    if !pum_visible() || item < -1 as ::core::ffi::c_int || item >= pum_size {
        return;
    }
    pum_want.active = true_0 != 0;
    pum_want.item = item;
    pum_want.insert = insert;
    pum_want.finish = finish;
}
#[no_mangle]
pub unsafe extern "C" fn pum_get_height() -> ::core::ffi::c_int {
    if pum_external {
        let mut ui_pum_height: ::core::ffi::c_int = ui_pum_get_height();
        if ui_pum_height != 0 {
            return ui_pum_height;
        }
    }
    return pum_height;
}
#[no_mangle]
pub unsafe extern "C" fn pum_set_event_info(mut dict: *mut dict_T) {
    if !pum_visible() {
        return;
    }
    let mut w: ::core::ffi::c_double = 0.;
    let mut h: ::core::ffi::c_double = 0.;
    let mut r: ::core::ffi::c_double = 0.;
    let mut c: ::core::ffi::c_double = 0.;
    if !ui_pum_get_pos(&raw mut w, &raw mut h, &raw mut r, &raw mut c) {
        w = pum_width as ::core::ffi::c_double;
        h = pum_height as ::core::ffi::c_double;
        r = pum_row as ::core::ffi::c_double;
        c = pum_col as ::core::ffi::c_double;
    }
    tv_dict_add_float(
        dict,
        b"height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        h as float_T,
    );
    tv_dict_add_float(
        dict,
        b"width\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        w as float_T,
    );
    tv_dict_add_float(
        dict,
        b"row\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        r as float_T,
    );
    tv_dict_add_float(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        c as float_T,
    );
    tv_dict_add_nr(
        dict,
        b"size\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        pum_size as varnumber_T,
    );
    tv_dict_add_bool(
        dict,
        b"scrollbar\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (if pum_scrollbar != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
}
unsafe extern "C" fn pum_position_at_mouse(mut min_width: ::core::ffi::c_int) {
    let mut min_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut max_row: ::core::ffi::c_int = Rows;
    let mut max_col: ::core::ffi::c_int = Columns;
    let mut grid: ::core::ffi::c_int = mouse_grid;
    let mut row: ::core::ffi::c_int = mouse_row;
    let mut col: ::core::ffi::c_int = mouse_col;
    pum_win_row_offset = 0 as ::core::ffi::c_int;
    pum_win_col_offset = 0 as ::core::ffi::c_int;
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && grid == 0 as ::core::ffi::c_int
    {
        mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
    }
    if grid > 1 as ::core::ffi::c_int {
        let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
        if !wp.is_null() {
            row += (*wp).w_winrow;
            col += (*wp).w_wincol;
            pum_win_row_offset = (*wp).w_winrow;
            pum_win_col_offset = (*wp).w_wincol;
            if (*wp).w_view_height > 0 as ::core::ffi::c_int
                || (*wp).w_view_width > 0 as ::core::ffi::c_int
            {
                max_row = if Rows - (*wp).w_winrow > (*wp).w_winrow + (*wp).w_view_height
                {
                    Rows - (*wp).w_winrow
                } else {
                    (*wp).w_winrow + (*wp).w_view_height
                };
                max_col = if Columns - (*wp).w_wincol
                    > (*wp).w_wincol + (*wp).w_view_width
                {
                    Columns - (*wp).w_wincol
                } else {
                    (*wp).w_wincol + (*wp).w_view_width
                };
            }
        }
    }
    if pum_grid.handle != 0 as ::core::ffi::c_int && grid == pum_grid.handle {
        row += pum_row;
        col += pum_left_col;
    } else {
        pum_anchor_grid = grid;
    }
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut border_height: ::core::ffi::c_int = border_width;
    if max_row - row > pum_size + border_height || max_row - row > row - min_row {
        pum_above = false_0 != 0;
        pum_row = row + 1 as ::core::ffi::c_int;
        if pum_height + border_height > max_row - pum_row {
            pum_height = max_row - pum_row - border_height;
        }
    } else {
        pum_above = true_0 != 0;
        pum_row = row - pum_size - border_height;
        if pum_row < min_row {
            pum_height += pum_row - min_row;
            pum_row = min_row;
        }
    }
    if pum_rl {
        if col - min_col + 1 as ::core::ffi::c_int >= pum_base_width + border_width
            || col - min_col + 1 as ::core::ffi::c_int > min_width + border_width
        {
            pum_col = col;
        } else {
            pum_col = min_col
                + (if pum_base_width + border_width < min_width + border_width {
                    pum_base_width + border_width
                } else {
                    min_width + border_width
                }) - 1 as ::core::ffi::c_int;
        }
        pum_width = pum_col - min_col + 1 as ::core::ffi::c_int - border_width;
    } else {
        if max_col - col >= pum_base_width + border_width
            || max_col - col > min_width + border_width
        {
            pum_col = col;
        } else {
            pum_col = max_col
                - (if pum_base_width + border_width < min_width + border_width {
                    pum_base_width + border_width
                } else {
                    min_width + border_width
                });
        }
        pum_width = max_col - pum_col - border_width;
    }
    pum_width = if pum_width < pum_base_width + 1 as ::core::ffi::c_int {
        pum_width
    } else {
        pum_base_width + 1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn pum_select_mouse_pos() {
    let mut grid: ::core::ffi::c_int = mouse_grid;
    let mut row: ::core::ffi::c_int = mouse_row;
    let mut col: ::core::ffi::c_int = mouse_col;
    if grid == 0 as ::core::ffi::c_int {
        mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
    }
    if grid == pum_grid.handle {
        let mut border_offset: ::core::ffi::c_int = if pum_border_width()
            == 2 as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        let mut item: ::core::ffi::c_int = row - border_offset;
        pum_selected = if item >= 0 as ::core::ffi::c_int && item < pum_height {
            item
        } else {
            -1 as ::core::ffi::c_int
        };
        return;
    }
    if grid != pum_anchor_grid || col < pum_left_col - pum_win_col_offset
        || col >= pum_right_col - pum_win_col_offset
    {
        pum_selected = -1 as ::core::ffi::c_int;
        return;
    }
    let mut idx: ::core::ffi::c_int = row - (pum_row - pum_win_row_offset);
    if idx < 0 as ::core::ffi::c_int || idx >= pum_height {
        pum_selected = -1 as ::core::ffi::c_int;
    } else if *(*pum_array.offset(idx as isize)).pum_text as ::core::ffi::c_int != NUL {
        pum_selected = idx;
    }
}
unsafe extern "C" fn pum_execute_menu(
    mut menu: *mut vimmenu_T,
    mut mode: ::core::ffi::c_int,
) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut mp: *mut vimmenu_T = (*menu).children;
    while !mp.is_null() {
        if (*mp).modes & (*mp).enabled & mode != 0
            && {
                let c2rust_fresh7 = idx;
                idx = idx + 1;
                c2rust_fresh7 == pum_selected
            }
        {
            memset(
                &raw mut ea as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<exarg_T>(),
            );
            execute_menu(&raw mut ea, mp, -1 as ::core::ffi::c_int);
            break;
        } else {
            mp = (*mp).next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pum_show_popupmenu(mut menu: *mut vimmenu_T) {
    pum_undisplay(true_0 != 0);
    pum_size = 0 as ::core::ffi::c_int;
    let mut mode: ::core::ffi::c_int = get_menu_mode_flag();
    let mut mp: *mut vimmenu_T = (*menu).children;
    while !mp.is_null() {
        if menu_is_separator((*mp).dname) as ::core::ffi::c_int != 0
            || (*mp).modes & (*mp).enabled & mode != 0
        {
            pum_size += 1;
        }
        mp = (*mp).next;
    }
    if pum_size <= 0 as ::core::ffi::c_int {
        emsg(
            gettext(
                &raw const e_menu_only_exists_in_another_mode
                    as *const ::core::ffi::c_char,
            ),
        );
        return;
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut array: *mut pumitem_T = xcalloc(
        pum_size as size_t,
        ::core::mem::size_of::<pumitem_T>(),
    ) as *mut pumitem_T;
    let mut mp_0: *mut vimmenu_T = (*menu).children;
    while !mp_0.is_null() {
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
            ::core::ffi::c_char,
        >();
        if menu_is_separator((*mp_0).dname) {
            s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if (*mp_0).modes & (*mp_0).enabled & mode != 0 {
            s = (*mp_0).dname;
        }
        if !s.is_null() {
            s = xstrdup(s);
            let c2rust_fresh6 = idx;
            idx = idx + 1;
            let c2rust_lvalue_ptr = &raw mut (*array.offset(c2rust_fresh6 as isize))
                .pum_text;
            *c2rust_lvalue_ptr = s;
        }
        mp_0 = (*mp_0).next;
    }
    pum_array = array;
    pum_compute_size();
    pum_scrollbar = 0 as ::core::ffi::c_int;
    pum_height = pum_size;
    pum_rl = (*curwin).w_onebuf_opt.wo_rl != 0;
    pum_position_at_mouse(20 as ::core::ffi::c_int);
    pum_selected = -1 as ::core::ffi::c_int;
    pum_first = 0 as ::core::ffi::c_int;
    if p_mousemev == 0 {
        ui_call_option_set(
            String_0 {
                data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>()
                    .wrapping_sub(1 as size_t),
            },
            object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 { boolean: true },
            },
        );
    }
    loop {
        pum_is_visible = true_0 != 0;
        pum_is_drawn = true_0 != 0;
        pum_grid.zindex = kZIndexCmdlinePopupMenu as ::core::ffi::c_int;
        pum_redraw();
        setcursor_mayforce(curwin, true_0 != 0);
        let mut c: ::core::ffi::c_int = vgetc();
        if c == ESC || c == Ctrl_C || pum_array.is_null() {
            break;
        }
        if c == CAR || c == NL {
            pum_execute_menu(menu, mode);
            break;
        } else if c == 'k' as ::core::ffi::c_int || c == K_UP
            || c
                == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            while pum_selected > 0 as ::core::ffi::c_int {
                pum_selected -= 1;
                if *(*array.offset(pum_selected as isize)).pum_text as ::core::ffi::c_int
                    != NUL
                {
                    break;
                }
            }
        } else if c == 'j' as ::core::ffi::c_int || c == K_DOWN
            || c
                == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            while pum_selected < pum_size - 1 as ::core::ffi::c_int {
                pum_selected += 1;
                if *(*array.offset(pum_selected as isize)).pum_text as ::core::ffi::c_int
                    != NUL
                {
                    break;
                }
            }
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            vungetc(c);
            break;
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c
                == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c
                == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            pum_select_mouse_pos();
        } else {
            if !(c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int))
                || c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTRELEASE as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int)))
            {
                continue;
            }
            pum_select_mouse_pos();
            if pum_selected >= 0 as ::core::ffi::c_int {
                pum_execute_menu(menu, mode);
                break;
            } else if c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c
                    == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int)
                            << 8 as ::core::ffi::c_int))
            {
                break;
            }
        }
    }
    idx = 0 as ::core::ffi::c_int;
    while idx < pum_size {
        xfree((*array.offset(idx as isize)).pum_text as *mut ::core::ffi::c_void);
        idx += 1;
    }
    xfree(array as *mut ::core::ffi::c_void);
    pum_undisplay(true_0 != 0);
    if p_mousemev == 0 {
        ui_call_option_set(
            String_0 {
                data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>()
                    .wrapping_sub(1 as size_t),
            },
            object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 {
                    boolean: false,
                },
            },
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn pum_make_popup(
    mut path_name: *const ::core::ffi::c_char,
    mut use_mouse_pos: ::core::ffi::c_int,
) {
    if use_mouse_pos == 0 {
        mouse_row = (*curwin).w_grid.row_offset + (*curwin).w_wrow;
        mouse_col = (*curwin).w_grid.col_offset
            + (if (*curwin).w_onebuf_opt.wo_rl != 0 {
                (*curwin).w_view_width - (*curwin).w_wcol - 1 as ::core::ffi::c_int
            } else {
                (*curwin).w_wcol
            });
        if ui_has(kUIMultigrid) {
            mouse_grid = (*(*curwin).w_grid.target).handle as ::core::ffi::c_int;
        } else if (*curwin).w_grid.target != &raw mut default_grid {
            mouse_grid = 0 as ::core::ffi::c_int;
            mouse_row += (*curwin).w_winrow;
            mouse_col += (*curwin).w_wincol;
        }
    }
    let mut menu: *mut vimmenu_T = menu_find(path_name);
    if !menu.is_null() {
        pum_show_popupmenu(menu);
    }
}
#[no_mangle]
pub unsafe extern "C" fn pum_ui_flush() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        && pum_is_drawn as ::core::ffi::c_int != 0 && !pum_external
        && pum_grid.handle != 0 as ::core::ffi::c_int
        && pum_grid.pending_comp_index_update as ::core::ffi::c_int != 0
    {
        let mut anchor: *const ::core::ffi::c_char = if pum_above as ::core::ffi::c_int
            != 0
        {
            b"SW\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NW\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut row_off: ::core::ffi::c_int = if pum_above as ::core::ffi::c_int != 0 {
            -pum_height
        } else {
            0 as ::core::ffi::c_int
        };
        ui_call_win_float_pos(
            pum_grid.handle as Integer,
            -1 as Window,
            cstr_as_string(anchor),
            pum_anchor_grid as Integer,
            (pum_row - row_off - pum_win_row_offset) as Float,
            (pum_left_col - pum_win_col_offset) as Float,
            false_0 != 0,
            pum_grid.zindex as Integer,
            pum_grid.comp_index as ::core::ffi::c_int as Integer,
            pum_grid.comp_row as Integer,
            pum_grid.comp_col as Integer,
        );
        pum_grid.pending_comp_index_update = false_0 != 0;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
