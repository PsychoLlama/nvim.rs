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
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn maketitle();
    static p_ch: GlobalCell<OptInt>;
    static p_columns: GlobalCell<OptInt>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static dy_flags: GlobalCell<::core::ffi::c_uint>;
    static p_hls: GlobalCell<::core::ffi::c_int>;
    static p_icon: GlobalCell<::core::ffi::c_int>;
    static p_lines: GlobalCell<OptInt>;
    static p_lz: GlobalCell<::core::ffi::c_int>;
    static p_paste: GlobalCell<::core::ffi::c_int>;
    static p_rdt: GlobalCell<OptInt>;
    static p_ri: GlobalCell<::core::ffi::c_int>;
    static p_ru: GlobalCell<::core::ffi::c_int>;
    static p_wbr: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sc: GlobalCell<::core::ffi::c_int>;
    static p_sloc: GlobalCell<*mut ::core::ffi::c_char>;
    static p_smd: GlobalCell<::core::ffi::c_int>;
    static p_title: GlobalCell<::core::ffi::c_int>;
    static p_wmw: GlobalCell<OptInt>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn cmdline_pum_display(changed_array: bool);
    static decor_state: GlobalCell<DecorState>;
    fn decor_redraw_reset(wp: *mut win_T, state: *mut DecorState) -> bool;
    fn decor_range_add_virt(
        state: *mut DecorState,
        start_row: ::core::ffi::c_int,
        start_col: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        vt: *mut DecorVirtText,
        owned: bool,
    );
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor: bool) -> bool;
    fn win_lines_concealed(wp: *mut win_T) -> bool;
    fn buf_signcols_count_range(
        buf: *mut buf_T,
        row1: ::core::ffi::c_int,
        row2: ::core::ffi::c_int,
        add: ::core::ffi::c_int,
        clear: TriState,
    );
    fn decor_virt_lines(
        wp: *mut win_T,
        start_row: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        num_below: *mut ::core::ffi::c_int,
        lines: *mut VirtLines,
        apply_folds: bool,
    ) -> ::core::ffi::c_int;
    fn decor_providers_start();
    fn decor_providers_invoke_win(wp: *mut win_T);
    fn decor_providers_invoke_buf(buf: *mut buf_T);
    fn decor_providers_invoke_end();
    static need_diff_redraw: GlobalCell<bool>;
    fn get_keymap_str(
        wp: *mut win_T,
        fmt: *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static win_extmark_arr: GlobalCell<C2Rust_Unnamed_23>;
    fn diff_redraw(dofold: bool);
    fn win_line(
        wp: *mut win_T,
        lnum: linenr_T,
        startrow: ::core::ffi::c_int,
        endrow: ::core::ffi::c_int,
        col_rows: ::core::ffi::c_int,
        concealed: bool,
        spv: *mut spellvars_T,
        foldinfo: foldinfo_T,
    ) -> ::core::ffi::c_int;
    static updating_screen: GlobalCell<bool>;
    static redraw_not_allowed: GlobalCell<bool>;
    static screen_search_hl: GlobalCell<match_T>;
    static search_hl_has_cursor_lnum: GlobalCell<linenr_T>;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn cmdline_screen_cleared();
    fn redrawcmdline();
    fn compute_cmdrow();
    fn get_cmdline_info() -> *mut CmdlineInfo;
    fn hasAnyFolding(win: *mut win_T) -> ::core::ffi::c_int;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn fold_info(win: *mut win_T, lnum: linenr_T) -> foldinfo_T;
    fn foldmethodIsSyntax(wp: *mut win_T) -> bool;
    fn char_avail() -> bool;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static redraw_cmdline: GlobalCell<bool>;
    static redraw_mode: GlobalCell<bool>;
    static clear_cmdline: GlobalCell<bool>;
    static mode_displayed: GlobalCell<bool>;
    static cmdline_was_last_drawn: GlobalCell<bool>;
    static dollar_vcol: GlobalCell<colnr_T>;
    static edit_submode: GlobalCell<*mut ::core::ffi::c_char>;
    static edit_submode_pre: GlobalCell<*mut ::core::ffi::c_char>;
    static edit_submode_extra: GlobalCell<*mut ::core::ffi::c_char>;
    static edit_submode_highl: GlobalCell<hlf_T>;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scrolled: GlobalCell<::core::ffi::c_int>;
    static msg_did_scroll: GlobalCell<bool>;
    static msg_didout: GlobalCell<bool>;
    static msg_didany: GlobalCell<bool>;
    static need_wait_return: GlobalCell<bool>;
    static need_maketitle: GlobalCell<bool>;
    static lines_left: GlobalCell<::core::ffi::c_int>;
    static msg_no_more: GlobalCell<bool>;
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static redraw_tabline: GlobalCell<bool>;
    static curbuf: GlobalCell<*mut buf_T>;
    static ru_col: GlobalCell<::core::ffi::c_int>;
    static ru_wid: GlobalCell<::core::ffi::c_int>;
    static sc_col: GlobalCell<::core::ffi::c_int>;
    static starting: GlobalCell<::core::ffi::c_int>;
    static exiting: GlobalCell<bool>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_select: GlobalCell<bool>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static State: GlobalCell<::core::ffi::c_int>;
    static exmode_active: GlobalCell<bool>;
    static reg_recording: GlobalCell<::core::ffi::c_int>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static RedrawingDisabled: GlobalCell<::core::ffi::c_int>;
    static KeyTyped: GlobalCell<bool>;
    static must_redraw: GlobalCell<::core::ffi::c_int>;
    static do_redraw: GlobalCell<bool>;
    static must_redraw_pum: GlobalCell<bool>;
    static need_highlight_changed: GlobalCell<bool>;
    static got_int: GlobalCell<bool>;
    static global_busy: GlobalCell<::core::ffi::c_int>;
    static stl_syntax: GlobalCell<::core::ffi::c_int>;
    static no_hlsearch: GlobalCell<bool>;
    static display_tick: GlobalCell<disptick_T>;
    static default_grid: GlobalCell<ScreenGrid>;
    static default_gridview: GlobalCell<GridView>;
    static resizing_screen: GlobalCell<bool>;
    static ns_hl_fast: GlobalCell<NS>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn win_check_ns_hl(wp: *mut win_T) -> bool;
    fn update_window_hl(wp: *mut win_T, invalid: bool);
    fn win_bg_attr(wp: *mut win_T) -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ins_compl_show_pum();
    fn highlight_changed();
    fn init_search_hl(wp: *mut win_T, search_hl: *mut match_T);
    fn prepare_search_hl(wp: *mut win_T, search_hl: *mut match_T, lnum: linenr_T);
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    static msg_grid: GlobalCell<ScreenGrid>;
    static msg_scrolled_at_flush: GlobalCell<::core::ffi::c_int>;
    static msg_grid_scroll_discount: GlobalCell<::core::ffi::c_int>;
    fn msg_grid_set_pos(row: ::core::ffi::c_int, scrolled: bool);
    fn msg_use_grid() -> bool;
    fn msg_grid_validate();
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_scrollsize() -> ::core::ffi::c_int;
    fn msg_reset_scroll();
    fn repeat_message();
    fn msg_clr_eos();
    fn msg_clr_cmdline();
    fn msg_ext_ui_flush();
    fn msg_ext_flush_showmode();
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn plines_correct_topline(
        wp: *mut win_T,
        lnum: linenr_T,
        nextp: *mut linenr_T,
        limit_winheight: bool,
        foldedp: *mut bool,
    ) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn update_curswant();
    fn changed_window_setting(wp: *mut win_T);
    fn changed_line_abv_curs();
    fn changed_line_abv_curs_win(wp: *mut win_T);
    fn invalidate_botline_win(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn validate_virtcol(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_col_off2(wp: *mut win_T) -> ::core::ffi::c_int;
    fn curs_columns(wp: *mut win_T, may_scroll: ::core::ffi::c_int);
    fn set_empty_rows(wp: *mut win_T, used: ::core::ffi::c_int);
    fn clear_showcmd();
    fn do_check_scrollbind(check: bool);
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn get_ve_flags(wp: *mut win_T) -> ::core::ffi::c_uint;
    fn getvvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvcols(
        wp: *mut win_T,
        pos1: *mut pos_T,
        pos2: *mut pos_T,
        left: *mut colnr_T,
        right: *mut colnr_T,
    );
    fn win_may_fill(wp: *mut win_T) -> bool;
    fn win_get_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn plines_win(wp: *mut win_T, lnum: linenr_T, limit_winheight: bool) -> ::core::ffi::c_int;
    fn plines_m_win(
        wp: *mut win_T,
        first: linenr_T,
        last: linenr_T,
        max: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn profile_setlimit(msec: int64_t) -> proftime_T;
    fn pum_redraw();
    fn pum_check_clear();
    fn pum_drawn() -> bool;
    fn pum_invalidate();
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regfree(prog: *mut regprog_T);
    fn last_pat_prog(regmatch: *mut regmmatch_T);
    fn spell_check_window(wp: *mut win_T) -> bool;
    fn get_real_state() -> ::core::ffi::c_int;
    fn win_redr_status(wp: *mut win_T);
    fn stl_clear_click_defs(click_defs: *mut StlClickDefinition, click_defs_size: size_t);
    fn stl_alloc_click_defs(
        cdp: *mut StlClickDefinition,
        width: ::core::ffi::c_int,
        size: *mut size_t,
    ) -> *mut StlClickDefinition;
    fn win_redr_winbar(wp: *mut win_T);
    fn redraw_ruler();
    fn draw_tabline();
    static tab_page_click_defs: GlobalCell<*mut StlClickDefinition>;
    static tab_page_click_defs_size: GlobalCell<size_t>;
    fn syn_set_timeout(tm: *mut proftime_T);
    fn syn_stack_apply_changes(buf: *mut buf_T);
    fn syntax_end_parsing(wp: *mut win_T, lnum: linenr_T);
    fn syntax_check_changed(lnum: linenr_T) -> bool;
    fn syntax_present(win: *mut win_T) -> bool;
    fn terminal_check_size(term: *mut Terminal);
    fn terminal_suspended(term: *const Terminal) -> bool;
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_call_grid_clear(grid: Integer);
    fn ui_call_win_extmark(
        grid: Integer,
        win: Window,
        ns_id: Integer,
        mark_id: Integer,
        row: Integer,
        col: Integer,
    );
    fn ui_call_msg_clear();
    fn ui_grid_cursor_goto(
        grid_handle: handle_T,
        new_row: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_comp_set_screen_valid(valid: bool) -> bool;
    fn may_show_intro() -> bool;
    fn intro_message(colon: bool);
    fn win_fdccol_count(wp: *mut win_T) -> ::core::ffi::c_int;
    fn frame2win(frp: *mut frame_T) -> *mut win_T;
    fn win_new_screensize();
    fn global_stl_height() -> ::core::ffi::c_int;
    fn last_stl_height(morewin: bool) -> ::core::ffi::c_int;
    fn min_rows(tp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn min_rows_for_all_tabpages() -> ::core::ffi::c_int;
    fn win_ui_flush(validate: bool);
    fn grid_adjust(
        grid: *mut GridView,
        row_off: *mut ::core::ffi::c_int,
        col_off: *mut ::core::ffi::c_int,
    ) -> *mut ScreenGrid;
    fn schar_cache_clear_if_full() -> bool;
    fn grid_clear_line(grid: *mut ScreenGrid, off: size_t, width: ::core::ffi::c_int, valid: bool);
    fn grid_invalidate(grid: *mut ScreenGrid);
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn grid_line_getchar(col: ::core::ffi::c_int, attr: *mut ::core::ffi::c_int) -> schar_T;
    fn grid_line_put_schar(col: ::core::ffi::c_int, schar: schar_T, attr: ::core::ffi::c_int);
    fn grid_line_fill(
        start_col: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        sc: schar_T,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_clear_end(
        start_col: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        bg_attr: ::core::ffi::c_int,
        clear_attr: ::core::ffi::c_int,
    );
    fn grid_line_mirror(width: ::core::ffi::c_int);
    fn grid_line_flush();
    fn grid_clear(
        grid: *mut GridView,
        start_row: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        start_col: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    );
    fn grid_alloc(
        grid: *mut ScreenGrid,
        rows: ::core::ffi::c_int,
        columns: ::core::ffi::c_int,
        copy: bool,
        valid: bool,
    );
    fn win_grid_alloc(wp: *mut win_T);
    fn grid_ins_lines(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        line_count: ::core::ffi::c_int,
        end: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
        width: ::core::ffi::c_int,
    );
    fn grid_del_lines(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        line_count: ::core::ffi::c_int,
        end: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
        width: ::core::ffi::c_int,
    );
    fn grid_draw_border(
        grid: *mut ScreenGrid,
        config: *mut WinConfig,
        adj: *mut ::core::ffi::c_int,
        winbl: ::core::ffi::c_int,
        hl_attr: *mut ::core::ffi::c_int,
    );
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub type DecorPriorityInternal = uint32_t;
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
pub type MetaIndex = ::core::ffi::c_uint;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
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
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
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
pub type auto_event = ::core::ffi::c_uint;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
pub type event_T = auto_event;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptDyFlagMsgsep: C2Rust_Unnamed_16 = 8;
pub const kOptDyFlagUhex: C2Rust_Unnamed_16 = 4;
pub const kOptDyFlagTruncate: C2Rust_Unnamed_16 = 2;
pub const kOptDyFlagLastline: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_17 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_17 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_17 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_17 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_17 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_18 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_18 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_18 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_18 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_18 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_18 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_18 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_18 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_18 = 79;
pub const SHM_OVER: C2Rust_Unnamed_18 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_18 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_18 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_18 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_18 = 97;
pub const SHM_WRI: C2Rust_Unnamed_18 = 119;
pub const SHM_LINES: C2Rust_Unnamed_18 = 108;
pub const SHM_MOD: C2Rust_Unnamed_18 = 109;
pub const SHM_RO: C2Rust_Unnamed_18 = 114;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CmdlineColorChunk {
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CmdlineColors {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CmdlineColorChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColoredCmdline {
    pub prompt_id: ::core::ffi::c_uint,
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub colors: CmdlineColors,
}
pub type CmdRedraw = ::core::ffi::c_uint;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdline_info {
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub cmdbufflen: ::core::ffi::c_int,
    pub cmdlen: ::core::ffi::c_int,
    pub cmdpos: ::core::ffi::c_int,
    pub cmdspos: ::core::ffi::c_int,
    pub cmdfirstc: ::core::ffi::c_int,
    pub cmdindent: ::core::ffi::c_int,
    pub cmdprompt: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
    pub overstrike: ::core::ffi::c_int,
    pub xpc: *mut expand_T,
    pub xp_context: ::core::ffi::c_int,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub input_fn: ::core::ffi::c_int,
    pub cmdbuff_replaced: bool,
    pub prompt_id: ::core::ffi::c_uint,
    pub highlight_callback: Callback,
    pub last_colors: ColoredCmdline,
    pub level: ::core::ffi::c_int,
    pub prev_ccline: *mut CmdlineInfo,
    pub special_char: ::core::ffi::c_char,
    pub special_shift: bool,
    pub redraw_state: CmdRedraw,
    pub one_key: bool,
    pub mouse_used: *mut bool,
}
pub type CmdlineInfo = cmdline_info;
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
    pub data: C2Rust_Unnamed_19,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_20,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
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
    pub slots: C2Rust_Unnamed_22,
    pub ranges_i: C2Rust_Unnamed_21,
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
pub struct C2Rust_Unnamed_21 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinExtmark {
    pub ns_id: NS,
    pub mark_id: uint64_t,
    pub win_row: ::core::ffi::c_int,
    pub win_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spellvars_T {
    pub spv_has_spell: bool,
    pub spv_unchanged: bool,
    pub spv_checked_col: ::core::ffi::c_int,
    pub spv_checked_lnum: linenr_T,
    pub spv_cap_col: ::core::ffi::c_int,
    pub spv_capcol_lnum: linenr_T,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_24 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_24 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_24 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_24 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_24 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_24 = 20;
pub const UPD_VALID: C2Rust_Unnamed_24 = 10;
pub const MODE_CMDLINE: C2Rust_Unnamed_26 = 8;
pub const MODE_NORMAL: C2Rust_Unnamed_26 = 1;
pub const MODE_INSERT: C2Rust_Unnamed_26 = 16;
pub const MODE_VISUAL: C2Rust_Unnamed_26 = 2;
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
pub const MODE_LANGMAP: C2Rust_Unnamed_26 = 32;
pub const REPLACE_FLAG: C2Rust_Unnamed_26 = 256;
pub const VREPLACE_FLAG: C2Rust_Unnamed_26 = 512;
pub const MODE_TERMINAL: C2Rust_Unnamed_26 = 128;
pub type WindowCorner = ::core::ffi::c_uint;
pub const WC_BOTTOM_RIGHT: WindowCorner = 3;
pub const WC_BOTTOM_LEFT: WindowCorner = 2;
pub const WC_TOP_RIGHT: WindowCorner = 1;
pub const WC_TOP_LEFT: WindowCorner = 0;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const DID_FOLD: C2Rust_Unnamed_25 = 3;
pub const DID_LINE: C2Rust_Unnamed_25 = 2;
pub const DID_NONE: C2Rust_Unnamed_25 = 1;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_26 = 20480;
pub const MODE_ASKMORE: C2Rust_Unnamed_26 = 12288;
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
pub const SHOWCMD_COLS: C2Rust_Unnamed_27 = 10;
pub const MIN_COLUMNS: C2Rust_Unnamed_28 = 12;
pub const MODE_SETWSIZE: C2Rust_Unnamed_26 = 16384;
pub const MODE_HITRETURN: C2Rust_Unnamed_26 = 8193;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_26 = 24592;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_26 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_26 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_26 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_26 = 272;
pub const MAP_ALL_MODES: C2Rust_Unnamed_26 = 255;
pub const MODE_SELECT: C2Rust_Unnamed_26 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_26 = 4;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_28 = 1;
pub const MIN_LINES: C2Rust_Unnamed_28 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
static redraw_popupmenu: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_grid_invalid: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static resizing_autocmd: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static conceal_cursor_used: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
#[no_mangle]
pub unsafe extern "C" fn conceal_check_cursor_line() {
    let mut should_conceal: bool = conceal_cursor_line(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_cole <= 0 as OptInt
        || conceal_cursor_used.get() as ::core::ffi::c_int == should_conceal as ::core::ffi::c_int
    {
        return;
    }
    redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    if decor_conceal_line(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        true_0 != 0,
    ) {
        changed_window_setting(curwin.get());
    }
    curs_columns(curwin.get(), true_0);
}
#[no_mangle]
pub unsafe extern "C" fn default_grid_alloc() -> bool {
    static resizing: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if resizing.get() {
        return false_0 != 0;
    }
    resizing.set(true_0 != 0);
    if !(*default_grid.ptr()).chars.is_null()
        && Rows.get() == (*default_grid.ptr()).rows
        && Columns.get() == (*default_grid.ptr()).cols
        || Rows.get() == 0 as ::core::ffi::c_int
        || Columns.get() == 0 as ::core::ffi::c_int
    {
        resizing.set(false_0 != 0);
        return false_0 != 0;
    }
    grid_alloc(
        default_grid.ptr(),
        Rows.get(),
        Columns.get(),
        true_0 != 0,
        true_0 != 0,
    );
    stl_clear_click_defs(tab_page_click_defs.get(), tab_page_click_defs_size.get());
    tab_page_click_defs.set(stl_alloc_click_defs(
        tab_page_click_defs.get(),
        Columns.get(),
        tab_page_click_defs_size.ptr(),
    ));
    (*default_grid.ptr()).comp_height = Rows.get();
    (*default_grid.ptr()).comp_width = Columns.get();
    (*default_grid.ptr()).handle = DEFAULT_GRID_HANDLE as handle_T;
    resizing.set(false_0 != 0);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn screenclear() {
    msg_check_for_delay(false_0 != 0);
    if starting.get() == NO_SCREEN || (*default_grid.ptr()).chars.is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*default_grid.ptr()).rows {
        grid_clear_line(
            default_grid.ptr(),
            *(*default_grid.ptr()).line_offset.offset(i as isize),
            (*default_grid.ptr()).cols,
            true_0 != 0,
        );
        i += 1;
    }
    ui_call_grid_clear(1 as Integer);
    ui_comp_set_screen_valid(true_0 != 0);
    ns_hl_fast.set(-1 as ::core::ffi::c_int as NS);
    clear_cmdline.set(false_0 != 0);
    mode_displayed.set(false_0 != 0);
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    cmdline_was_last_drawn.set(false_0 != 0);
    redraw_cmdline.set(true_0 != 0);
    redraw_tabline.set(true_0 != 0);
    redraw_popupmenu.set(true_0 != 0);
    pum_invalidate();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_floating {
            (*wp).w_redr_type = UPD_CLEAR as ::core::ffi::c_int;
        }
        wp = (*wp).w_next;
    }
    if must_redraw.get() == UPD_CLEAR as ::core::ffi::c_int {
        must_redraw.set(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    compute_cmdrow();
    msg_row.set(cmdline_row.get());
    msg_col.set(0 as ::core::ffi::c_int);
    msg_reset_scroll();
    msg_didany.set(false_0 != 0);
    msg_didout.set(false_0 != 0);
    if *(*hl_attr_active.ptr()).offset(HLF_MSG as ::core::ffi::c_int as isize)
        > 0 as ::core::ffi::c_int
        && msg_use_grid() as ::core::ffi::c_int != 0
        && !(*msg_grid.ptr()).chars.is_null()
    {
        grid_invalidate(msg_grid.ptr());
        msg_grid_validate();
        msg_grid_invalid.set(false_0 != 0);
        clear_cmdline.set(true_0 != 0);
    }
}
unsafe extern "C" fn cmdline_number_prompt() -> bool {
    return !ui_has(kUIMessages)
        && State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
        && !(*get_cmdline_info()).mouse_used.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    if updating_screen.get() as ::core::ffi::c_int != 0
        || resizing_screen.get() as ::core::ffi::c_int != 0
        || cmdline_number_prompt() as ::core::ffi::c_int != 0
    {
        return;
    }
    if width < 0 as ::core::ffi::c_int || height < 0 as ::core::ffi::c_int {
        return;
    }
    if State.get() == MODE_HITRETURN as ::core::ffi::c_int
        || State.get() == MODE_SETWSIZE as ::core::ffi::c_int
    {
        State.set(MODE_SETWSIZE as ::core::ffi::c_int);
        return;
    }
    resizing_screen.set(true_0 != 0);
    Rows.set(height);
    Columns.set(width);
    check_screensize();
    if !ui_has(kUIMessages) {
        let mut max_p_ch: ::core::ffi::c_int =
            Rows.get() - min_rows(curtab.get()) + 1 as ::core::ffi::c_int;
        if p_ch.get() > 0 as OptInt && p_ch.get() > max_p_ch as OptInt {
            p_ch.set(
                (if max_p_ch > 1 as ::core::ffi::c_int {
                    max_p_ch
                } else {
                    1 as ::core::ffi::c_int
                }) as OptInt,
            );
            (*curtab.get()).tp_ch_used = p_ch.get();
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                let mut max_tp_ch: ::core::ffi::c_int =
                    Rows.get() - min_rows(tp as *mut tabpage_T) + 1 as ::core::ffi::c_int;
                if (*tp).tp_ch_used > 0 as OptInt && (*tp).tp_ch_used > max_tp_ch as OptInt {
                    (*tp).tp_ch_used = (if max_tp_ch > 1 as ::core::ffi::c_int {
                        max_tp_ch
                    } else {
                        1 as ::core::ffi::c_int
                    }) as OptInt;
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    height = Rows.get();
    width = Columns.get();
    p_lines.set(Rows.get() as OptInt);
    p_columns.set(Columns.get() as OptInt);
    ui_call_grid_resize(1 as Integer, width as Integer, height as Integer);
    let mut retry_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    resizing_autocmd.set(true_0 != 0);
    while default_grid_alloc() {
        ui_comp_set_screen_valid(false_0 != 0);
        if !(*msg_grid.ptr()).chars.is_null() {
            msg_grid_invalid.set(true_0 != 0);
        }
        (*RedrawingDisabled.ptr()) += 1;
        win_new_screensize();
        comp_col();
        (*RedrawingDisabled.ptr()) -= 1;
        retry_count += 1;
        if retry_count > 3 as ::core::ffi::c_int {
            break;
        }
        apply_autocmds(
            EVENT_VIMRESIZED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    resizing_autocmd.set(false_0 != 0);
    redraw_all_later(UPD_CLEAR as ::core::ffi::c_int);
    if State.get() != MODE_ASKMORE as ::core::ffi::c_int
        && State.get() != MODE_EXTERNCMD as ::core::ffi::c_int
    {
        screenclear();
    }
    if starting.get() != NO_SCREEN {
        maketitle();
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
        if State.get() == MODE_ASKMORE as ::core::ffi::c_int
            || State.get() == MODE_EXTERNCMD as ::core::ffi::c_int
            || exmode_active.get() as ::core::ffi::c_int != 0
            || State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
                && (*get_cmdline_info()).one_key as ::core::ffi::c_int != 0
        {
            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                update_screen();
            }
            if !(*msg_grid.ptr()).chars.is_null() {
                msg_grid_validate();
            }
            ui_comp_set_screen_valid(true_0 != 0);
            repeat_message();
        } else {
            if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                do_check_scrollbind(true_0 != 0);
            }
            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                redraw_popupmenu.set(false_0 != 0);
                update_screen();
                redrawcmdline();
                if pum_drawn() {
                    cmdline_pum_display(false_0 != 0);
                }
            } else {
                update_topline(curwin.get());
                if pum_drawn() {
                    redraw_popupmenu.set(false_0 != 0);
                    ins_compl_show_pum();
                }
                update_screen();
                if redrawing() {
                    setcursor();
                }
            }
        }
        ui_flush();
    }
    resizing_screen.set(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn check_screensize() {
    Rows.set(
        if (if Rows.get() > min_rows_for_all_tabpages() {
            Rows.get()
        } else {
            min_rows_for_all_tabpages()
        }) < 1000 as ::core::ffi::c_int
        {
            if Rows.get() > min_rows_for_all_tabpages() {
                Rows.get()
            } else {
                min_rows_for_all_tabpages()
            }
        } else {
            1000 as ::core::ffi::c_int
        },
    );
    Columns.set(
        if (if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
            Columns.get()
        } else {
            MIN_COLUMNS as ::core::ffi::c_int
        }) < 10000 as ::core::ffi::c_int
        {
            if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
                Columns.get()
            } else {
                MIN_COLUMNS as ::core::ffi::c_int
            }
        } else {
            10000 as ::core::ffi::c_int
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn redrawing() -> bool {
    return RedrawingDisabled.get() == 0
        && !(p_lz.get() != 0
            && char_avail() as ::core::ffi::c_int != 0
            && !KeyTyped.get()
            && !do_redraw.get());
}
#[no_mangle]
pub unsafe extern "C" fn update_screen() -> ::core::ffi::c_int {
    static still_may_intro: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
    if still_may_intro.get() {
        if !may_show_intro() {
            redraw_later(firstwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
            still_may_intro.set(false_0 != 0);
        }
    }
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    if resizing_autocmd.get() as ::core::ffi::c_int != 0 || (*default_grid.ptr()).chars.is_null() {
        return FAIL;
    }
    if need_diff_redraw.get() {
        diff_redraw(true_0 != 0);
    }
    if !redrawing()
        || updating_screen.get() as ::core::ffi::c_int != 0
        || cmdline_number_prompt() as ::core::ffi::c_int != 0
    {
        return FAIL;
    }
    let mut type_0: ::core::ffi::c_int = must_redraw.get();
    must_redraw.set(0 as ::core::ffi::c_int);
    updating_screen.set(true_0 != 0);
    display_tick.set((*display_tick.ptr()).wrapping_add(1));
    if schar_cache_clear_if_full() {
        type_0 = if type_0 > UPD_CLEAR as ::core::ffi::c_int {
            type_0
        } else {
            UPD_CLEAR as ::core::ffi::c_int
        };
    }
    if msg_did_scroll.get() {
        msg_did_scroll.set(false_0 != 0);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
    }
    if type_0 >= UPD_CLEAR as ::core::ffi::c_int || !(*default_grid.ptr()).valid {
        ui_comp_set_screen_valid(false_0 != 0);
    }
    if msg_scrolled.get() != 0 || msg_grid_invalid.get() as ::core::ffi::c_int != 0 {
        clear_cmdline.set(true_0 != 0);
        let mut valid: ::core::ffi::c_int =
            if Rows.get() - msg_scrollsize() > 0 as ::core::ffi::c_int {
                Rows.get() - msg_scrollsize()
            } else {
                0 as ::core::ffi::c_int
            };
        if !(*msg_grid.ptr()).chars.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i
                < (if msg_scrollsize() < (*msg_grid.ptr()).rows {
                    msg_scrollsize()
                } else {
                    (*msg_grid.ptr()).rows
                })
            {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset.offset(i as isize),
                    (*msg_grid.ptr()).cols,
                    (i as OptInt) < p_ch.get(),
                );
                i += 1;
            }
        }
        (*msg_grid.ptr()).throttled = false_0 != 0;
        let mut was_invalidated: bool = false_0 != 0;
        if type_0 == UPD_NOT_VALID as ::core::ffi::c_int
            && !ui_has(kUIMultigrid)
            && msg_scrolled.get() != 0
        {
            was_invalidated = ui_comp_set_screen_valid(false_0 != 0);
            let mut i_0: ::core::ffi::c_int = valid;
            while (i_0 as OptInt) < Rows.get() as OptInt - p_ch.get() {
                grid_clear_line(
                    default_grid.ptr(),
                    *(*default_grid.ptr()).line_offset.offset(i_0 as isize),
                    Columns.get(),
                    false_0 != 0,
                );
                i_0 += 1;
            }
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if !(*wp).w_floating {
                    if (*wp).w_winrow + (*wp).w_height > valid {
                        (*wp).w_redr_type =
                            if (*wp).w_redr_type > UPD_NOT_VALID as ::core::ffi::c_int {
                                (*wp).w_redr_type
                            } else {
                                UPD_NOT_VALID as ::core::ffi::c_int
                            };
                    }
                    if !is_stl_global
                        && (*wp).w_winrow + (*wp).w_height + (*wp).w_status_height > valid
                    {
                        (*wp).w_redr_status = true_0 != 0;
                    }
                }
                wp = (*wp).w_next;
            }
            if is_stl_global as ::core::ffi::c_int != 0
                && Rows.get() as OptInt - p_ch.get() - 1 as OptInt > valid as OptInt
            {
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
        }
        msg_grid_set_pos(Rows.get() - p_ch.get() as ::core::ffi::c_int, false_0 != 0);
        msg_grid_invalid.set(false_0 != 0);
        if was_invalidated {
            ui_comp_set_screen_valid(true_0 != 0);
        }
        msg_scrolled.set(0 as ::core::ffi::c_int);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
        msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
        need_wait_return.set(false_0 != 0);
    }
    win_ui_flush(true_0 != 0);
    compute_cmdrow();
    let mut hl_changed: bool = false_0 != 0;
    if need_highlight_changed.get() {
        highlight_changed();
        hl_changed = true_0 != 0;
    }
    if type_0 == UPD_CLEAR as ::core::ffi::c_int {
        screenclear();
        cmdline_screen_cleared();
        if ui_has(kUIMessages) {
            ui_call_msg_clear();
        }
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        must_redraw.set(0 as ::core::ffi::c_int);
    } else if !(*default_grid.ptr()).valid {
        grid_invalidate(default_grid.ptr());
        (*default_grid.ptr()).valid = true_0 != 0;
    }
    if type_0 == UPD_NOT_VALID as ::core::ffi::c_int
        && clear_cmdline.get() as ::core::ffi::c_int != 0
        && !ui_has(kUIMessages)
    {
        grid_clear(
            default_gridview.ptr(),
            Rows.get() - p_ch.get() as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            0 as ::core::ffi::c_int,
        );
    }
    ui_comp_set_screen_valid(true_0 != 0);
    decor_providers_start();
    if win_check_ns_hl(::core::ptr::null_mut::<win_T>()) {
        redraw_cmdline.set(true_0 != 0);
        redraw_tabline.set(true_0 != 0);
    }
    if clear_cmdline.get() {
        msg_check_for_delay(false_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_NOT_VALID as ::core::ffi::c_int
        && (*curwin.get()).w_nrwidth
            != (if (*curwin.get()).w_onebuf_opt.wo_nu != 0
                || (*curwin.get()).w_onebuf_opt.wo_rnu != 0
                || *(*curwin.get()).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
            {
                number_width(curwin.get())
            } else {
                0 as ::core::ffi::c_int
            })
    {
        (*curwin.get()).w_redr_type = UPD_NOT_VALID as ::core::ffi::c_int;
    }
    if (*curwin.get()).w_redr_type == UPD_INVERTED as ::core::ffi::c_int {
        update_curswant();
    }
    if redraw_tabline.get() as ::core::ffi::c_int != 0
        || type_0 >= UPD_NOT_VALID as ::core::ffi::c_int
    {
        update_window_hl(curwin.get(), type_0 >= UPD_NOT_VALID as ::core::ffi::c_int);
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                update_window_hl(
                    (*tp).tp_curwin,
                    type_0 >= UPD_NOT_VALID as ::core::ffi::c_int,
                );
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        draw_tabline();
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        update_window_hl(
            wp_0,
            type_0 >= UPD_NOT_VALID as ::core::ffi::c_int || hl_changed as ::core::ffi::c_int != 0,
        );
        let mut buf: *mut buf_T = (*wp_0).w_buffer;
        if (*buf).b_mod_set {
            if (*buf).b_mod_tick_syn < display_tick.get()
                && syntax_present(wp_0) as ::core::ffi::c_int != 0
            {
                syn_stack_apply_changes(buf);
                (*buf).b_mod_tick_syn = display_tick.get();
            }
            if (*buf).b_mod_tick_decor < display_tick.get() {
                decor_providers_invoke_buf(buf);
                (*buf).b_mod_tick_decor = display_tick.get();
            }
        }
        wp_0 = (*wp_0).w_next;
    }
    let mut did_one: bool = false_0 != 0;
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut wp_1: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_1.is_null() {
        if (*wp_1).w_redr_type == UPD_CLEAR as ::core::ffi::c_int
            && (*wp_1).w_floating as ::core::ffi::c_int != 0
            && !(*wp_1).w_grid_alloc.chars.is_null()
        {
            grid_invalidate(&raw mut (*wp_1).w_grid_alloc);
            (*wp_1).w_redr_type = UPD_NOT_VALID as ::core::ffi::c_int;
        }
        win_check_ns_hl(wp_1);
        win_grid_alloc(wp_1);
        if (*wp_1).w_redr_border as ::core::ffi::c_int != 0
            || (*wp_1).w_redr_type >= UPD_NOT_VALID as ::core::ffi::c_int
        {
            grid_draw_border(
                &raw mut (*wp_1).w_grid_alloc,
                &raw mut (*wp_1).w_config,
                &raw mut (*wp_1).w_border_adj as *mut ::core::ffi::c_int,
                (*wp_1).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
                (*wp_1).w_ns_hl_attr,
            );
        }
        if (*wp_1).w_redr_type != 0 as ::core::ffi::c_int {
            if !did_one {
                did_one = true_0 != 0;
                start_search_hl();
            }
            win_update(wp_1);
        }
        if (*wp_1).w_redr_status {
            win_redr_winbar(wp_1);
            win_redr_status(wp_1);
        }
        wp_1 = (*wp_1).w_next;
    }
    if did_one {
        let mut wp_2: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp_2.is_null() {
            draw_sep_connectors_win(wp_2);
            wp_2 = (*wp_2).w_next;
        }
    }
    end_search_hl();
    if pum_drawn() as ::core::ffi::c_int != 0 && must_redraw_pum.get() as ::core::ffi::c_int != 0 {
        win_check_ns_hl(curwin.get());
        pum_redraw();
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        pum_check_clear();
    }
    win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    let mut wp_3: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_3.is_null() {
        (*(*wp_3).w_buffer).b_mod_set = false_0 != 0;
        wp_3 = (*wp_3).w_next;
    }
    updating_screen.set(false_0 != 0);
    if need_maketitle.get() {
        maketitle();
    }
    if clear_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_mode.get() as ::core::ffi::c_int != 0
    {
        showmode();
    }
    if still_may_intro.get() {
        intro_message(false_0 != 0);
    }
    repeat_message();
    decor_providers_invoke_end();
    if !ui_has(kUICmdline) {
        cmdline_was_last_drawn.set(false_0 != 0);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn start_search_hl() {
    if p_hls.get() == 0 || no_hlsearch.get() as ::core::ffi::c_int != 0 {
        return;
    }
    end_search_hl();
    last_pat_prog(&raw mut (*screen_search_hl.ptr()).rm);
    (*screen_search_hl.ptr()).tm = profile_setlimit(p_rdt.get() as int64_t);
}
#[no_mangle]
pub unsafe extern "C" fn end_search_hl() {
    if (*screen_search_hl.ptr()).rm.regprog.is_null() {
        return;
    }
    vim_regfree((*screen_search_hl.ptr()).rm.regprog);
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
}
#[no_mangle]
pub unsafe extern "C" fn setcursor() {
    setcursor_mayforce(curwin.get(), false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn setcursor_mayforce(mut wp: *mut win_T, mut force: bool) {
    if force as ::core::ffi::c_int != 0 || redrawing() as ::core::ffi::c_int != 0 {
        validate_cursor(wp);
        let mut row: ::core::ffi::c_int = (*wp).w_wrow;
        let mut col: ::core::ffi::c_int = (*wp).w_wcol;
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            let mut cursor: *mut ::core::ffi::c_char =
                ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
            col = (*wp).w_view_width
                - (*wp).w_wcol
                - (if utf_ptr2cells(cursor) == 2 as ::core::ffi::c_int
                    && vim_isprintc(utf_ptr2char(cursor)) as ::core::ffi::c_int != 0
                {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                });
        }
        let mut grid: *mut ScreenGrid =
            grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
        if !grid.is_null() {
            ui_grid_cursor_goto((*grid).handle, row, col);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_custom_title_later() -> bool {
    if p_icon.get() != 0 && stl_syntax.get() & STL_IN_ICON != 0
        || p_title.get() != 0 && stl_syntax.get() & STL_IN_TITLE != 0
    {
        need_maketitle.set(true_0 != 0);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn show_cursor_info_later(mut force: bool) {
    let mut state: ::core::ffi::c_int = get_real_state();
    let mut empty_line: ::core::ffi::c_int = (State.get() & MODE_INSERT as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum)
            as ::core::ffi::c_int
            == NUL) as ::core::ffi::c_int;
    validate_virtcol(curwin.get());
    if force as ::core::ffi::c_int != 0
        || (*curwin.get()).w_cursor.lnum != (*curwin.get()).w_stl_cursor.lnum
        || (*curwin.get()).w_cursor.col != (*curwin.get()).w_stl_cursor.col
        || (*curwin.get()).w_virtcol != (*curwin.get()).w_stl_virtcol
        || (*curwin.get()).w_cursor.coladd != (*curwin.get()).w_stl_cursor.coladd
        || (*curwin.get()).w_topline != (*curwin.get()).w_stl_topline
        || (*(*curwin.get()).w_buffer).b_ml.ml_line_count != (*curwin.get()).w_stl_line_count
        || (*curwin.get()).w_topfill != (*curwin.get()).w_stl_topfill
        || empty_line != (*curwin.get()).w_stl_empty as ::core::ffi::c_int
        || reg_recording.get() != (*curwin.get()).w_stl_recording
        || state != (*curwin.get()).w_stl_state
        || VIsual_active.get() as ::core::ffi::c_int != 0
            && (VIsual_mode.get() != (*curwin.get()).w_stl_visual_mode
                || (*VIsual.ptr()).lnum != (*curwin.get()).w_stl_visual_pos.lnum
                || (*VIsual.ptr()).col != (*curwin.get()).w_stl_visual_pos.col
                || (*VIsual.ptr()).coladd != (*curwin.get()).w_stl_visual_pos.coladd)
    {
        if (*curwin.get()).w_status_height != 0 || global_stl_height() != 0 {
            (*curwin.get()).w_redr_status = true_0 != 0;
        } else {
            redraw_cmdline.set(true_0 != 0);
        }
        if *p_wbr.get() as ::core::ffi::c_int != NUL
            || *(*curwin.get()).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
        {
            (*curwin.get()).w_redr_status = true_0 != 0;
        }
        redraw_custom_title_later();
    }
    (*curwin.get()).w_stl_cursor = (*curwin.get()).w_cursor;
    (*curwin.get()).w_stl_virtcol = (*curwin.get()).w_virtcol;
    (*curwin.get()).w_stl_empty = empty_line as ::core::ffi::c_char;
    (*curwin.get()).w_stl_topline = (*curwin.get()).w_topline;
    (*curwin.get()).w_stl_line_count = (*(*curwin.get()).w_buffer).b_ml.ml_line_count;
    (*curwin.get()).w_stl_topfill = (*curwin.get()).w_topfill;
    (*curwin.get()).w_stl_recording = reg_recording.get();
    (*curwin.get()).w_stl_state = state;
    if VIsual_active.get() {
        (*curwin.get()).w_stl_visual_mode = VIsual_mode.get();
        (*curwin.get()).w_stl_visual_pos = VIsual.get();
    }
}
#[no_mangle]
pub unsafe extern "C" fn skip_showmode() -> bool {
    if global_busy.get() != 0
        || msg_silent.get() != 0 as ::core::ffi::c_int
        || !redrawing()
        || char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get()
    {
        redraw_mode.set(true_0 != 0);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn showmode() -> ::core::ffi::c_int {
    let mut length: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    msg_ext_ui_flush();
    msg_grid_validate();
    let mut do_mode: bool = p_smd.get() != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && (State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
            || State.get() & MODE_INSERT as ::core::ffi::c_int != 0
            || restart_edit.get() != NUL
            || VIsual_active.get() as ::core::ffi::c_int != 0);
    let mut can_show_mode: bool =
        p_ch.get() != 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0;
    if (do_mode as ::core::ffi::c_int != 0 || reg_recording.get() != 0 as ::core::ffi::c_int)
        && can_show_mode as ::core::ffi::c_int != 0
    {
        if skip_showmode() {
            return 0 as ::core::ffi::c_int;
        }
        let mut nwr_save: bool = need_wait_return.get();
        msg_check_for_delay(false_0 != 0);
        let mut need_clear: bool = clear_cmdline.get();
        if clear_cmdline.get() as ::core::ffi::c_int != 0
            && cmdline_row.get() < Rows.get() - 1 as ::core::ffi::c_int
        {
            msg_clr_cmdline();
        }
        msg_pos_mode();
        let mut hl_id: ::core::ffi::c_int = HLF_CM as ::core::ffi::c_int;
        msg_no_more.set(true_0 != 0);
        let mut save_lines_left: ::core::ffi::c_int = lines_left.get();
        lines_left.set(0 as ::core::ffi::c_int);
        if do_mode {
            msg_puts_hl(
                b"--\0".as_ptr() as *const ::core::ffi::c_char,
                hl_id,
                false_0 != 0,
            );
            if !(*edit_submode.ptr()).is_null()
                && !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int)
            {
                if ui_has(kUIMessages) {
                    length = INT_MAX;
                } else {
                    length = (Rows.get() - msg_row.get()) * Columns.get() - 3 as ::core::ffi::c_int;
                }
                if !(*edit_submode_extra.ptr()).is_null() {
                    length -= vim_strsize(edit_submode_extra.get());
                }
                if length > 0 as ::core::ffi::c_int {
                    if !(*edit_submode_pre.ptr()).is_null() {
                        length -= vim_strsize(edit_submode_pre.get());
                    }
                    if length - vim_strsize(edit_submode.get()) > 0 as ::core::ffi::c_int {
                        if !(*edit_submode_pre.ptr()).is_null() {
                            msg_puts_hl(edit_submode_pre.get(), hl_id, false_0 != 0);
                        }
                        msg_puts_hl(edit_submode.get(), hl_id, false_0 != 0);
                    }
                    if !(*edit_submode_extra.ptr()).is_null() {
                        msg_puts_hl(
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                        let mut sub_id: ::core::ffi::c_int = if (edit_submode_highl.get()
                            as ::core::ffi::c_uint)
                            < HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            edit_submode_highl.get() as ::core::ffi::c_int
                        } else {
                            hl_id
                        };
                        msg_puts_hl(edit_submode_extra.get(), sub_id, false_0 != 0);
                    }
                }
            } else {
                if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" TERMINAL\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" VREPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" REPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
                    if p_ri.get() != 0 {
                        msg_puts_hl(
                            gettext(b" REVERSE\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                    msg_puts_hl(
                        gettext(b" INSERT\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if restart_edit.get() == 'I' as ::core::ffi::c_int
                    || restart_edit.get() == 'i' as ::core::ffi::c_int
                    || restart_edit.get() == 'a' as ::core::ffi::c_int
                    || restart_edit.get() == 'A' as ::core::ffi::c_int
                {
                    if !(*curbuf.get()).terminal.is_null() {
                        msg_puts_hl(
                            gettext(b" (terminal)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else {
                        msg_puts_hl(
                            gettext(b" (insert)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                } else if restart_edit.get() == 'R' as ::core::ffi::c_int {
                    msg_puts_hl(
                        gettext(b" (replace)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if restart_edit.get() == 'V' as ::core::ffi::c_int {
                    msg_puts_hl(
                        gettext(b" (vreplace)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                }
                if State.get() & MODE_LANGMAP as ::core::ffi::c_int != 0 {
                    if (*curwin.get()).w_onebuf_opt.wo_arab != 0 {
                        msg_puts_hl(
                            gettext(b" Arabic\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if get_keymap_str(
                        curwin.get(),
                        b" (%s)\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                        MAXPATHL,
                    ) > 0 as ::core::ffi::c_int
                    {
                        msg_puts_hl(
                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                    }
                }
                if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 && p_paste.get() != 0 {
                    msg_puts_hl(
                        gettext(b" (paste)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                }
                if VIsual_active.get() {
                    let mut p: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    match (if VIsual_select.get() as ::core::ffi::c_int != 0 {
                        4 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) + (VIsual_mode.get() == Ctrl_V) as ::core::ffi::c_int
                        * 2 as ::core::ffi::c_int
                        + (VIsual_mode.get() == 'V' as ::core::ffi::c_int) as ::core::ffi::c_int
                    {
                        0 => {
                            p = b" VISUAL\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        1 => {
                            p = b" VISUAL LINE\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        2 => {
                            p = b" VISUAL BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        4 => {
                            p = b" SELECT\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        5 => {
                            p = b" SELECT LINE\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        _ => {
                            p = b" SELECT BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    msg_puts_hl(gettext(p), hl_id, false_0 != 0);
                }
                msg_puts_hl(
                    b" --\0".as_ptr() as *const ::core::ffi::c_char,
                    hl_id,
                    false_0 != 0,
                );
            }
            need_clear = true_0 != 0;
        }
        if reg_recording.get() != 0 as ::core::ffi::c_int && (*edit_submode.ptr()).is_null() {
            recording_mode(hl_id);
            need_clear = true_0 != 0;
        }
        mode_displayed.set(true_0 != 0);
        if need_clear as ::core::ffi::c_int != 0
            || clear_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_mode.get() as ::core::ffi::c_int != 0
        {
            msg_clr_eos();
        }
        msg_didout.set(false_0 != 0);
        length = msg_col.get();
        msg_col.set(0 as ::core::ffi::c_int);
        msg_no_more.set(false_0 != 0);
        lines_left.set(save_lines_left);
        need_wait_return.set(nwr_save);
    } else if clear_cmdline.get() as ::core::ffi::c_int != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
    {
        msg_clr_cmdline();
    } else if redraw_mode.get() {
        msg_pos_mode();
        msg_clr_eos();
    }
    msg_ext_flush_showmode();
    if VIsual_active.get() {
        clear_showcmd();
    }
    redraw_ruler();
    redraw_cmdline.set(false_0 != 0);
    redraw_mode.set(false_0 != 0);
    clear_cmdline.set(false_0 != 0);
    return length;
}
unsafe extern "C" fn msg_pos_mode() {
    msg_col.set(0 as ::core::ffi::c_int);
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn unshowmode(mut force: bool) {
    if !redrawing() || !force && char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get() {
        redraw_cmdline.set(true_0 != 0);
    } else {
        clearmode();
    };
}
#[no_mangle]
pub unsafe extern "C" fn clearmode() {
    let save_msg_row: ::core::ffi::c_int = msg_row.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    msg_ext_ui_flush();
    msg_pos_mode();
    if reg_recording.get() != 0 as ::core::ffi::c_int {
        recording_mode(HLF_CM as ::core::ffi::c_int);
    }
    msg_clr_eos();
    msg_ext_flush_showmode();
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);
}
unsafe extern "C" fn recording_mode(mut hl_id: ::core::ffi::c_int) {
    if shortmess(SHM_RECORDING as ::core::ffi::c_int) {
        return;
    }
    msg_puts_hl(
        gettext(b"recording\0".as_ptr() as *const ::core::ffi::c_char),
        hl_id,
        false_0 != 0,
    );
    let mut s: [::core::ffi::c_char; 4] = [0; 4];
    snprintf(
        &raw mut s as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        b" @%c\0".as_ptr() as *const ::core::ffi::c_char,
        reg_recording.get(),
    );
    msg_puts_hl(&raw mut s as *mut ::core::ffi::c_char, hl_id, false_0 != 0);
}
pub const COL_RULER: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn comp_col() {
    let mut last_has_status: bool = last_stl_height(false_0 != 0) > 0 as ::core::ffi::c_int;
    sc_col.set(0 as ::core::ffi::c_int);
    ru_col.set(0 as ::core::ffi::c_int);
    if p_ru.get() != 0 {
        ru_col.set(
            (if ru_wid.get() != 0 {
                ru_wid.get()
            } else {
                COL_RULER
            }) + 1 as ::core::ffi::c_int,
        );
        if !last_has_status {
            sc_col.set(ru_col.get());
        }
    }
    if p_sc.get() != 0 && *p_sloc.get() as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
        (*sc_col.ptr()) += SHOWCMD_COLS as ::core::ffi::c_int;
        if p_ru.get() == 0 || last_has_status as ::core::ffi::c_int != 0 {
            (*sc_col.ptr()) += 1;
        }
    }
    '_c2rust_label: {
        if sc_col.get() >= 0 as ::core::ffi::c_int
            && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + sc_col.get()
                <= Columns.get()
        {
        } else {
            __assert_fail(
                b"sc_col >= 0 && INT_MIN + sc_col <= Columns\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1128 as ::core::ffi::c_uint,
                b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    sc_col.set(Columns.get() - sc_col.get());
    '_c2rust_label_0: {
        if ru_col.get() >= 0 as ::core::ffi::c_int
            && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + ru_col.get()
                <= Columns.get()
        {
        } else {
            __assert_fail(
                b"ru_col >= 0 && INT_MIN + ru_col <= Columns\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1131 as ::core::ffi::c_uint,
                b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    ru_col.set(Columns.get() - ru_col.get());
    if sc_col.get() <= 0 as ::core::ffi::c_int {
        sc_col.set(1 as ::core::ffi::c_int);
    }
    if ru_col.get() <= 0 as ::core::ffi::c_int {
        ru_col.set(1 as ::core::ffi::c_int);
    }
    set_vim_var_nr(
        VV_ECHOSPACE,
        (sc_col.get() - 1 as ::core::ffi::c_int) as varnumber_T,
    );
}
unsafe extern "C" fn win_redraw_signcols(mut wp: *mut win_T) -> bool {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if !(*buf).b_signcols.autom
        && (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
            || (*wp).w_maxscwidth > 1 as ::core::ffi::c_int
                && (*wp).w_minscwidth != (*wp).w_maxscwidth)
    {
        (*buf).b_signcols.autom = true_0 != 0;
        buf_signcols_count_range(
            buf,
            0 as ::core::ffi::c_int,
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            MAXLNUM as ::core::ffi::c_int,
            kFalse,
        );
    }
    while (*buf).b_signcols.max > 0 as ::core::ffi::c_int
        && (*buf).b_signcols.count[((*buf).b_signcols.max - 1 as ::core::ffi::c_int) as usize]
            == 0 as ::core::ffi::c_int
    {
        (*buf).b_signcols.max -= 1;
    }
    let mut width: ::core::ffi::c_int = if (*wp).w_maxscwidth < (*buf).b_signcols.max {
        (*wp).w_maxscwidth
    } else {
        (*buf).b_signcols.max
    };
    let mut rebuild_stc: bool = (*buf).b_signcols.max != (*buf).b_signcols.last_max
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL;
    if rebuild_stc {
        (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
    } else if (*wp).w_minscwidth == 0 as ::core::ffi::c_int
        && (*wp).w_maxscwidth == 1 as ::core::ffi::c_int
    {
        width = (buf_meta_total(buf, kMTMetaSignText) > 0 as uint32_t) as ::core::ffi::c_int;
    }
    let mut scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
    (*wp).w_scwidth = if (if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
        0 as ::core::ffi::c_int
    } else {
        (*wp).w_minscwidth
    }) > width
    {
        if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_minscwidth
        }
    } else {
        width
    };
    return (*wp).w_scwidth != scwidth || rebuild_stc as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn hsep_connected(mut wp: *mut win_T, mut corner: WindowCorner) -> bool {
    let mut before: bool = corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut sep_row: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*wp).w_winrow - 1 as ::core::ffi::c_int
    } else {
        (*wp).w_winrow + (*wp).w_height
    };
    let mut fr: *mut frame_T = (*wp).w_frame;
    while !(*fr).fr_parent.is_null() {
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            })
            .is_null()
        {
            fr = if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            };
            break;
        } else {
            fr = (*fr).fr_parent;
        }
    }
    if (*fr).fr_parent.is_null() {
        return false_0 != 0;
    }
    while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
        fr = (*fr).fr_child;
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && before as ::core::ffi::c_int != 0
        {
            while !(*fr).fr_next.is_null() {
                fr = (*fr).fr_next;
            }
        } else {
            while !(*fr).fr_next.is_null() && (*frame2win(fr)).w_winrow + (*fr).fr_height < sep_row
            {
                fr = (*fr).fr_next;
            }
        }
    }
    return sep_row == (*(*fr).fr_win).w_winrow - 1 as ::core::ffi::c_int
        || sep_row == (*(*fr).fr_win).w_winrow + (*(*fr).fr_win).w_height;
}
unsafe extern "C" fn vsep_connected(mut wp: *mut win_T, mut corner: WindowCorner) -> bool {
    let mut before: bool = corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut sep_col: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*wp).w_wincol - 1 as ::core::ffi::c_int
    } else {
        (*wp).w_wincol + (*wp).w_width
    };
    let mut fr: *mut frame_T = (*wp).w_frame;
    while !(*fr).fr_parent.is_null() {
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            })
            .is_null()
        {
            fr = if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            };
            break;
        } else {
            fr = (*fr).fr_parent;
        }
    }
    if (*fr).fr_parent.is_null() {
        return false_0 != 0;
    }
    while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
        fr = (*fr).fr_child;
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && before as ::core::ffi::c_int != 0
        {
            while !(*fr).fr_next.is_null() {
                fr = (*fr).fr_next;
            }
        } else {
            while !(*fr).fr_next.is_null() && (*frame2win(fr)).w_wincol + (*fr).fr_width < sep_col {
                fr = (*fr).fr_next;
            }
        }
    }
    return sep_col == (*(*fr).fr_win).w_wincol - 1 as ::core::ffi::c_int
        || sep_col == (*(*fr).fr_win).w_wincol + (*(*fr).fr_win).w_width;
}
unsafe extern "C" fn draw_vsep_win(mut wp: *mut win_T) {
    if (*wp).w_vsep_width == 0 {
        return;
    }
    let mut row: ::core::ffi::c_int = (*wp).w_winrow;
    while row < (*wp).w_winrow + (*wp).w_height {
        grid_line_start(default_gridview.ptr(), row);
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            (*wp).w_p_fcs_chars.vert,
            win_hl_attr(wp, HLF_C as ::core::ffi::c_int),
        );
        grid_line_flush();
        row += 1;
    }
}
unsafe extern "C" fn draw_hsep_win(mut wp: *mut win_T) {
    if (*wp).w_hsep_height == 0 {
        return;
    }
    grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
    grid_line_fill(
        (*wp).w_wincol,
        (*wp).w_wincol + (*wp).w_width,
        (*wp).w_p_fcs_chars.horiz,
        win_hl_attr(wp, HLF_C as ::core::ffi::c_int),
    );
    grid_line_flush();
}
unsafe extern "C" fn get_corner_sep_connector(
    mut wp: *mut win_T,
    mut corner: WindowCorner,
) -> schar_T {
    if vsep_connected(wp, corner) {
        if hsep_connected(wp, corner) {
            return (*wp).w_p_fcs_chars.verthoriz;
        } else if corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*wp).w_p_fcs_chars.vertright;
        } else {
            return (*wp).w_p_fcs_chars.vertleft;
        }
    } else if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*wp).w_p_fcs_chars.horizdown;
    } else {
        return (*wp).w_p_fcs_chars.horizup;
    };
}
unsafe extern "C" fn draw_sep_connectors_win(mut wp: *mut win_T) {
    if global_stl_height() == 0 as ::core::ffi::c_int
        || !((*wp).w_hsep_height == 1 as ::core::ffi::c_int
            || (*wp).w_vsep_width == 1 as ::core::ffi::c_int)
    {
        return;
    }
    let mut hl: ::core::ffi::c_int = win_hl_attr(wp, HLF_C as ::core::ffi::c_int);
    let mut win_at_top: bool = false;
    let mut win_at_bottom: bool = (*wp).w_hsep_height == 0 as ::core::ffi::c_int;
    let mut win_at_left: bool = false;
    let mut win_at_right: bool = (*wp).w_vsep_width == 0 as ::core::ffi::c_int;
    let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frp = (*wp).w_frame;
    while !(*frp).fr_parent.is_null() {
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(*frp).fr_prev.is_null()
        {
            break;
        }
        frp = (*frp).fr_parent;
    }
    win_at_top = (*frp).fr_parent.is_null();
    frp = (*wp).w_frame;
    while !(*frp).fr_parent.is_null() {
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(*frp).fr_prev.is_null()
        {
            break;
        }
        frp = (*frp).fr_parent;
    }
    win_at_left = (*frp).fr_parent.is_null();
    let mut top_left: bool =
        !(win_at_top as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
    let mut top_right: bool =
        !(win_at_top as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
    let mut bot_left: bool =
        !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
    let mut bot_right: bool =
        !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
    if top_left {
        grid_line_start(
            default_gridview.ptr(),
            (*wp).w_winrow - 1 as ::core::ffi::c_int,
        );
        grid_line_put_schar(
            (*wp).w_wincol - 1 as ::core::ffi::c_int,
            get_corner_sep_connector(wp, WC_TOP_LEFT),
            hl,
        );
        grid_line_flush();
    }
    if top_right {
        grid_line_start(
            default_gridview.ptr(),
            (*wp).w_winrow - 1 as ::core::ffi::c_int,
        );
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            get_corner_sep_connector(wp, WC_TOP_RIGHT),
            hl,
        );
        grid_line_flush();
    }
    if bot_left {
        grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
        grid_line_put_schar(
            (*wp).w_wincol - 1 as ::core::ffi::c_int,
            get_corner_sep_connector(wp, WC_BOTTOM_LEFT),
            hl,
        );
        grid_line_flush();
    }
    if bot_right {
        grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            get_corner_sep_connector(wp, WC_BOTTOM_RIGHT),
            hl,
        );
        grid_line_flush();
    }
}
unsafe extern "C" fn win_update(mut wp: *mut win_T) {
    let mut old_botline: linenr_T = 0;
    if (*wp).w_grid.target == default_grid.ptr() && (*wp).w_wincol >= Columns.get() {
        return;
    }
    let mut top_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mid_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut mid_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bot_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut scrolled_down: bool = false_0 != 0;
    let mut scrolled_for_mod: bool = false_0 != 0;
    let mut top_to_mod: bool = false_0 != 0;
    let mut bot_scroll_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut did_update: C2Rust_Unnamed_25 = DID_NONE;
    let mut syntax_last_parsed: linenr_T = 0 as linenr_T;
    let mut mod_top: linenr_T = 0 as linenr_T;
    let mut mod_bot: linenr_T = 0 as linenr_T;
    let mut type_0: ::core::ffi::c_int = (*wp).w_redr_type;
    if type_0 >= UPD_NOT_VALID as ::core::ffi::c_int {
        (*wp).w_redr_status = true_0 != 0;
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_view_height == 0 as ::core::ffi::c_int {
        draw_hsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        draw_vsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false);
    let mut syntax_tm: proftime_T = profile_setlimit(p_rdt.get() as int64_t);
    syn_set_timeout(&raw mut syntax_tm);
    (*win_extmark_arr.ptr()).size = 0 as size_t;
    decor_redraw_reset(wp, decor_state.ptr());
    decor_providers_invoke_win(wp);
    if !(*buf).terminal.is_null() && terminal_suspended((*buf).terminal) as ::core::ffi::c_int != 0
    {
        static chunk: GlobalCell<VirtTextChunk> = GlobalCell::new(VirtTextChunk {
            text: b"[Process suspended]\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            hl_id: -1 as ::core::ffi::c_int,
        });
        static virt_text: GlobalCell<DecorVirtText> = GlobalCell::new(unsafe {
            DecorVirtText {
                flags: 0,
                hl_mode: 0,
                priority: DECOR_PRIORITY_BASE as DecorPriority,
                width: 0,
                col: 0,
                pos: kVPosWinCol,
                data: C2Rust_Unnamed_2 {
                    virt_text: VirtText {
                        size: 1 as size_t,
                        capacity: 0,
                        items: (chunk.as_raw() as *const _) as *mut VirtTextChunk,
                    },
                },
                next: ::core::ptr::null_mut::<DecorVirtText>(),
            }
        });
        decor_range_add_virt(
            decor_state.ptr(),
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            virt_text.ptr(),
            false_0 != 0,
        );
    }
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_buffer == (*wp).w_buffer && win_redraw_signcols(win) as ::core::ffi::c_int != 0
        {
            changed_line_abv_curs_win(win);
            redraw_later(win, UPD_NOT_VALID as ::core::ffi::c_int);
        }
        win = (*win).w_next;
    }
    (*buf).b_signcols.last_max = (*buf).b_signcols.max;
    validate_virtcol(wp);
    type_0 = (*wp).w_redr_type;
    init_search_hl(wp, screen_search_hl.ptr());
    if (*wp).w_skipcol > 0 as ::core::ffi::c_int && (*wp).w_view_width > win_col_off(wp) {
        let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut add: ::core::ffi::c_int = width1;
        while w < (*wp).w_skipcol {
            if w > 0 as ::core::ffi::c_int {
                add = width2;
            }
            w += add;
        }
        if w != (*wp).w_skipcol {
            (*wp).w_skipcol = (w - add) as colnr_T;
        }
    }
    let nrwidth_before: ::core::ffi::c_int = (*wp).w_nrwidth;
    let mut nrwidth_new: ::core::ffi::c_int = if (*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
    {
        number_width(wp)
    } else {
        0 as ::core::ffi::c_int
    };
    if (*wp).w_nrwidth != nrwidth_new {
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        changed_line_abv_curs_win(wp);
        (*wp).w_nrwidth = nrwidth_new;
    } else {
        mod_top = (*wp).w_redraw_top;
        if (*wp).w_redraw_bot != 0 as linenr_T {
            mod_bot = (*wp).w_redraw_bot + 1 as linenr_T;
        } else {
            mod_bot = 0 as ::core::ffi::c_int as linenr_T;
        }
        if (*buf).b_mod_set {
            if mod_top == 0 as linenr_T || mod_top > (*buf).b_mod_top {
                mod_top = (*buf).b_mod_top;
                if syntax_present(wp) {
                    mod_top -= (*buf).b_s.b_syn_sync_linebreaks;
                    mod_top = if mod_top > 1 as linenr_T {
                        mod_top
                    } else {
                        1 as linenr_T
                    };
                }
            }
            if mod_bot == 0 as linenr_T || mod_bot < (*buf).b_mod_bot {
                mod_bot = (*buf).b_mod_bot;
            }
            if !(*screen_search_hl.ptr()).rm.regprog.is_null()
                && re_multiline((*screen_search_hl.ptr()).rm.regprog) != 0
            {
                top_to_mod = true_0 != 0;
            } else {
                let mut cur: *const matchitem_T = (*wp).w_match_head;
                while !cur.is_null() {
                    if !(*cur).mit_match.regprog.is_null()
                        && re_multiline((*cur).mit_match.regprog) != 0
                    {
                        top_to_mod = true_0 != 0;
                        break;
                    } else {
                        cur = (*cur).mit_next;
                    }
                }
            }
        }
        if search_hl_has_cursor_lnum.get() > 0 as linenr_T {
            if mod_top == 0 as linenr_T || mod_top > search_hl_has_cursor_lnum.get() {
                mod_top = search_hl_has_cursor_lnum.get();
            }
            if mod_bot == 0 as linenr_T || mod_bot < search_hl_has_cursor_lnum.get() + 1 as linenr_T
            {
                mod_bot = search_hl_has_cursor_lnum.get() + 1 as linenr_T;
            }
        }
        if mod_top != 0 as linenr_T && win_lines_concealed(wp) as ::core::ffi::c_int != 0 {
            let mut lnumt: linenr_T = (*wp).w_topline;
            let mut lnumb: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(i as isize)).wl_valid {
                    if (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < mod_top {
                        lnumt = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                    }
                    if lnumb == MAXLNUM as ::core::ffi::c_int as linenr_T
                        && (*(*wp).w_lines.offset(i as isize)).wl_lnum >= mod_bot
                    {
                        lnumb = (*(*wp).w_lines.offset(i as isize)).wl_lnum;
                        if compute_foldcolumn(wp, 0 as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
                        {
                            lnumb += 1;
                        }
                    }
                }
                i += 1;
            }
            hasFolding(
                wp,
                mod_top,
                &raw mut mod_top,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            mod_top = if mod_top < lnumt { mod_top } else { lnumt };
            mod_bot -= 1;
            hasFolding(
                wp,
                mod_bot,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut mod_bot,
            );
            mod_bot += 1;
            mod_bot = if mod_bot > lnumb { mod_bot } else { lnumb };
        }
        if mod_top != 0 as linenr_T && mod_top < (*wp).w_topline {
            if mod_bot > (*wp).w_topline {
                mod_top = (*wp).w_topline;
            } else if syntax_present(wp) {
                top_end = 1 as ::core::ffi::c_int;
            }
        }
    }
    (*wp).w_redraw_top = 0 as ::core::ffi::c_int as linenr_T;
    (*wp).w_redraw_bot = 0 as ::core::ffi::c_int as linenr_T;
    search_hl_has_cursor_lnum.set(0 as ::core::ffi::c_int as linenr_T);
    if type_0 == UPD_REDRAW_TOP as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*wp).w_lines_valid {
            j += (*(*wp).w_lines.offset(i_0 as isize)).wl_size as ::core::ffi::c_int;
            if j >= (*wp).w_upd_rows {
                top_end = j;
                break;
            } else {
                i_0 += 1;
            }
        }
        if top_end == 0 as ::core::ffi::c_int {
            type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        } else {
            type_0 = UPD_VALID as ::core::ffi::c_int;
        }
    }
    let mut topline_conceal: linenr_T = (*wp).w_topline;
    while topline_conceal < (*buf).b_ml.ml_line_count
        && decor_conceal_line(
            wp,
            topline_conceal as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        topline_conceal += 1;
        hasFolding(
            wp,
            topline_conceal,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut topline_conceal,
        );
    }
    if (type_0 == UPD_VALID as ::core::ffi::c_int
        || type_0 == UPD_SOME_VALID as ::core::ffi::c_int
        || type_0 == UPD_INVERTED as ::core::ffi::c_int
        || type_0 == UPD_INVERTED_ALL as ::core::ffi::c_int)
        && !(*wp).w_botfill
        && !(*wp).w_old_botfill
    {
        if !(mod_top != 0 as linenr_T
            && (*wp).w_topline == mod_top
            && (!(*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                || topline_conceal
                    == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum))
        {
            if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                as ::core::ffi::c_int
                != 0
                && (topline_conceal
                    < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                    || topline_conceal
                        == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        && (*wp).w_topfill > (*wp).w_old_topfill)
            {
                let mut j_0: ::core::ffi::c_int = 0;
                if win_lines_concealed(wp) {
                    j_0 = 0 as ::core::ffi::c_int;
                    let mut ln: linenr_T = (*wp).w_topline;
                    while ln < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum {
                        j_0 += !decor_conceal_line(
                            wp,
                            ln as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            false_0 != 0,
                        ) as ::core::ffi::c_int;
                        if j_0 >= (*wp).w_view_height - 2 as ::core::ffi::c_int {
                            break;
                        }
                        hasFolding(wp, ln, ::core::ptr::null_mut::<linenr_T>(), &raw mut ln);
                        ln += 1;
                    }
                } else {
                    j_0 = ((*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        - (*wp).w_topline) as ::core::ffi::c_int;
                }
                if j_0 < (*wp).w_view_height - 2 as ::core::ffi::c_int {
                    let mut i_1: ::core::ffi::c_int = plines_m_win(
                        wp,
                        (*wp).w_topline,
                        (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                            - 1 as linenr_T,
                        (*wp).w_view_height,
                    );
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        != (*wp).w_topline
                    {
                        i_1 += win_get_fill(
                            wp,
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum,
                        ) - (*wp).w_old_topfill;
                    }
                    if i_1 != 0 as ::core::ffi::c_int
                        && i_1 < (*wp).w_view_height - 2 as ::core::ffi::c_int
                    {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, i_1);
                        bot_scroll_start = 0 as ::core::ffi::c_int;
                        if (*wp).w_lines_valid != 0 as ::core::ffi::c_int {
                            top_end = i_1;
                            scrolled_down = true_0 != 0;
                            (*wp).w_lines_valid += j_0 as linenr_T as ::core::ffi::c_int;
                            if (*wp).w_lines_valid > (*wp).w_view_height {
                                (*wp).w_lines_valid = (*wp).w_view_height;
                            }
                            let mut idx: ::core::ffi::c_int = 0;
                            idx = (*wp).w_lines_valid;
                            while idx - j_0 >= 0 as ::core::ffi::c_int {
                                *(*wp).w_lines.offset(idx as isize) =
                                    *(*wp).w_lines.offset((idx - j_0) as isize);
                                idx -= 1;
                            }
                            while idx >= 0 as ::core::ffi::c_int {
                                let c2rust_fresh0 = idx;
                                idx = idx - 1;
                                (*(*wp).w_lines.offset(c2rust_fresh0 as isize)).wl_valid =
                                    false_0 != 0;
                            }
                        }
                    } else {
                        mid_start = 0 as ::core::ffi::c_int;
                    }
                } else {
                    mid_start = 0 as ::core::ffi::c_int;
                }
            } else {
                let mut j_1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_2 < (*wp).w_lines_valid {
                    if (*(*wp).w_lines.offset(i_2 as isize)).wl_valid as ::core::ffi::c_int != 0
                        && (*(*wp).w_lines.offset(i_2 as isize)).wl_lnum == (*wp).w_topline
                    {
                        j_1 = i_2;
                        break;
                    } else {
                        row += (*(*wp).w_lines.offset(i_2 as isize)).wl_size as ::core::ffi::c_int;
                        i_2 += 1;
                    }
                }
                if j_1 == -1 as ::core::ffi::c_int {
                    mid_start = 0 as ::core::ffi::c_int;
                } else {
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        == (*wp).w_topline
                    {
                        row += (*wp).w_old_topfill;
                    } else {
                        row += win_get_fill(wp, (*wp).w_topline);
                    }
                    row -= (*wp).w_topfill;
                    if row > 0 as ::core::ffi::c_int {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, -row);
                        bot_start = (*wp).w_view_height - row;
                        bot_scroll_start = bot_start;
                    }
                    if (row == 0 as ::core::ffi::c_int || bot_start < 999 as ::core::ffi::c_int)
                        && (*wp).w_lines_valid != 0 as ::core::ffi::c_int
                    {
                        bot_start = 0 as ::core::ffi::c_int;
                        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        loop {
                            *(*wp).w_lines.offset(idx_0 as isize) =
                                *(*wp).w_lines.offset(j_1 as isize);
                            if row > 0 as ::core::ffi::c_int
                                && bot_start
                                    + row
                                    + (*(*wp).w_lines.offset(j_1 as isize)).wl_size
                                        as ::core::ffi::c_int
                                    > (*wp).w_view_height
                            {
                                (*wp).w_lines_valid = idx_0 + 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                let c2rust_fresh1 = idx_0;
                                idx_0 = idx_0 + 1;
                                bot_start += (*(*wp).w_lines.offset(c2rust_fresh1 as isize)).wl_size
                                    as ::core::ffi::c_int;
                                j_1 += 1;
                                if j_1 < (*wp).w_lines_valid {
                                    continue;
                                }
                                (*wp).w_lines_valid = idx_0;
                                break;
                            }
                        }
                        if win_may_fill(wp) as ::core::ffi::c_int != 0
                            && bot_start > 0 as ::core::ffi::c_int
                        {
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_size =
                                plines_correct_topline(
                                    wp,
                                    (*wp).w_topline,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    true_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                ) as uint16_t;
                        }
                    }
                }
            }
        }
        if mid_start == 0 as ::core::ffi::c_int {
            mid_end = (*wp).w_view_height;
        }
    } else {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
    }
    if type_0 == UPD_SOME_VALID as ::core::ffi::c_int {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer
        || (*wp).w_old_cursor_lnum != 0 as linenr_T && type_0 != UPD_NOT_VALID as ::core::ffi::c_int
    {
        let mut from: linenr_T = 0;
        let mut to: linenr_T = 0;
        if VIsual_active.get() {
            if VIsual_mode.get() != (*wp).w_old_visual_mode as ::core::ffi::c_int
                || type_0 == UPD_INVERTED_ALL as ::core::ffi::c_int
            {
                if (*curwin.get()).w_cursor.lnum < (*VIsual.ptr()).lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*VIsual.ptr()).lnum;
                } else {
                    from = (*VIsual.ptr()).lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                }
                from = if (if from < (*wp).w_old_cursor_lnum {
                    from
                } else {
                    (*wp).w_old_cursor_lnum
                }) < (*wp).w_old_visual_lnum
                {
                    if from < (*wp).w_old_cursor_lnum {
                        from
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
                to = if (if to > (*wp).w_old_cursor_lnum {
                    to
                } else {
                    (*wp).w_old_cursor_lnum
                }) > (*wp).w_old_visual_lnum
                {
                    if to > (*wp).w_old_cursor_lnum {
                        to
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
            } else {
                if (*curwin.get()).w_cursor.lnum < (*wp).w_old_cursor_lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*wp).w_old_cursor_lnum;
                } else {
                    from = (*wp).w_old_cursor_lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                    if from == 0 as linenr_T {
                        from = to;
                    }
                }
                if (*VIsual.ptr()).lnum != (*wp).w_old_visual_lnum
                    || (*VIsual.ptr()).col != (*wp).w_old_visual_col
                {
                    if (*wp).w_old_visual_lnum < from && (*wp).w_old_visual_lnum != 0 as linenr_T {
                        from = (*wp).w_old_visual_lnum;
                    }
                    to = if (if to > (*wp).w_old_visual_lnum {
                        to
                    } else {
                        (*wp).w_old_visual_lnum
                    }) > (*VIsual.ptr()).lnum
                    {
                        if to > (*wp).w_old_visual_lnum {
                            to
                        } else {
                            (*wp).w_old_visual_lnum
                        }
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
            }
            if VIsual_mode.get() == Ctrl_V {
                let mut fromc: colnr_T = 0;
                let mut toc: colnr_T = 0;
                let mut save_ve_flags: ::core::ffi::c_uint =
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags;
                if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 {
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags =
                        kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint;
                }
                getvcols(
                    wp,
                    VIsual.ptr(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut fromc,
                    &raw mut toc,
                );
                toc += 1;
                (*curwin.get()).w_onebuf_opt.wo_ve_flags = save_ve_flags;
                if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                    if get_ve_flags(curwin.get())
                        & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        let mut pos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        let mut cursor_above: ::core::ffi::c_int = ((*curwin.get()).w_cursor.lnum
                            < (*VIsual.ptr()).lnum)
                            as ::core::ffi::c_int;
                        toc = 0 as ::core::ffi::c_int as colnr_T;
                        pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        pos.lnum = (*curwin.get()).w_cursor.lnum;
                        while if cursor_above != 0 {
                            (pos.lnum <= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } else {
                            (pos.lnum >= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } != 0
                        {
                            let mut t: colnr_T = 0;
                            pos.col = ml_get_buf_len((*wp).w_buffer, pos.lnum);
                            getvvcol(
                                wp,
                                &raw mut pos,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut t,
                            );
                            toc = if toc > t { toc } else { t };
                            pos.lnum = (pos.lnum as ::core::ffi::c_int
                                + if cursor_above != 0 {
                                    1 as ::core::ffi::c_int
                                } else {
                                    -1 as ::core::ffi::c_int
                                }) as linenr_T;
                        }
                        toc += 1;
                    } else {
                        toc = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                }
                if fromc != (*wp).w_old_cursor_fcol || toc != (*wp).w_old_cursor_lcol {
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    to = if to > (*VIsual.ptr()).lnum {
                        to
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
                (*wp).w_old_cursor_fcol = fromc;
                (*wp).w_old_cursor_lcol = toc;
            }
        } else if (*wp).w_old_cursor_lnum < (*wp).w_old_visual_lnum {
            from = (*wp).w_old_cursor_lnum;
            to = (*wp).w_old_visual_lnum;
        } else {
            from = (*wp).w_old_visual_lnum;
            to = (*wp).w_old_cursor_lnum;
        }
        from = if from > (*wp).w_topline {
            from
        } else {
            (*wp).w_topline
        };
        if (*wp).w_valid & VALID_BOTLINE != 0 {
            from = if from < (*wp).w_botline - 1 as linenr_T {
                from
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
            to = if to < (*wp).w_botline - 1 as linenr_T {
                to
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
        }
        if mid_start > 0 as ::core::ffi::c_int {
            let mut lnum: linenr_T = (*wp).w_topline;
            let mut idx_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut srow: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if scrolled_down {
                mid_start = top_end;
            } else {
                mid_start = 0 as ::core::ffi::c_int;
            }
            while lnum < from && idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid {
                    mid_start +=
                        (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                } else if !scrolled_down {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                }
                idx_1 += 1;
                if idx_1 < (*wp).w_lines_valid
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                {
                    lnum = (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum;
                } else {
                    lnum += 1;
                }
            }
            srow += mid_start;
            mid_end = (*wp).w_view_height;
            while idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum >= to + 1 as linenr_T
                {
                    mid_end = srow;
                    break;
                } else {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                    idx_1 += 1;
                }
            }
        }
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer {
        (*wp).w_old_visual_mode = VIsual_mode.get() as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*wp).w_old_visual_lnum = (*VIsual.ptr()).lnum;
        (*wp).w_old_visual_col = (*VIsual.ptr()).col;
        (*wp).w_old_curswant = (*curwin.get()).w_curswant;
    } else {
        (*wp).w_old_visual_mode = 0 as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_col = 0 as ::core::ffi::c_int as colnr_T;
    }
    let mut cursorline_fi: foldinfo_T = foldinfo_T {
        fi_lnum: 0 as linenr_T,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    win_update_cursorline(wp, &raw mut cursorline_fi);
    if wp == curwin.get() {
        conceal_cursor_used.set(conceal_cursor_line(curwin.get()));
    }
    win_check_ns_hl(wp);
    let mut spv: spellvars_T = spellvars_T {
        spv_has_spell: false,
        spv_unchanged: false,
        spv_checked_col: 0,
        spv_checked_lnum: 0,
        spv_cap_col: 0,
        spv_capcol_lnum: 0,
    };
    let mut lnum_0: linenr_T = (*wp).w_topline;
    if spell_check_window(wp) {
        spv.spv_has_spell = true_0 != 0;
        spv.spv_unchanged = mod_top == 0 as linenr_T;
    }
    let mut idx_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut srow_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eof: bool = false_0 != 0;
    let mut didline: bool = false_0 != 0;
    's_2363: {
        's_2327: loop {
            '_redr_statuscol: {
                's_2139: {
                    if row_0 == (*wp).w_view_height {
                        didline = true_0 != 0;
                    } else if lnum_0 > (*buf).b_ml.ml_line_count {
                        eof = true_0 != 0;
                    } else {
                        srow_0 = row_0;
                        if row_0 < top_end
                            || row_0 >= mid_start && row_0 < mid_end
                            || top_to_mod as ::core::ffi::c_int != 0
                            || idx_2 >= (*wp).w_lines_valid
                            || row_0
                                + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                    as ::core::ffi::c_int
                                > bot_start
                            || mod_top != 0 as linenr_T
                                && (lnum_0 == mod_top
                                    || lnum_0 >= mod_top
                                        && (lnum_0 < mod_bot
                                            || did_update as ::core::ffi::c_uint
                                                == DID_FOLD as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || did_update as ::core::ffi::c_uint
                                                == DID_LINE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && syntax_present(wp) as ::core::ffi::c_int != 0
                                                && (foldmethodIsSyntax(wp) as ::core::ffi::c_int
                                                    != 0
                                                    && hasAnyFolding(wp) != 0
                                                    || syntax_check_changed(lnum_0)
                                                        as ::core::ffi::c_int
                                                        != 0)
                                            || !(*wp).w_match_head.is_null()
                                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                                && (*buf).b_mod_xlines != 0 as linenr_T))
                            || lnum_0 == (*wp).w_cursorline
                            || lnum_0 == (*wp).w_last_cursorline
                        {
                            if lnum_0 == mod_top {
                                top_to_mod = false_0 != 0;
                            }
                            let mut foldinfo: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                && lnum_0 == (*wp).w_cursor.lnum
                            {
                                cursorline_fi
                            } else {
                                fold_info(wp, lnum_0)
                            };
                            let mut concealed: bool = decor_conceal_line(
                                wp,
                                lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                            if concealed as ::core::ffi::c_int != 0
                                && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                            {
                                if lnum_0 == mod_top && lnum_0 < mod_bot {
                                    mod_top = (mod_top as ::core::ffi::c_int
                                        + (if foldinfo.fi_lines != 0 {
                                            foldinfo.fi_lines
                                        } else {
                                            1 as linenr_T
                                        })
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                }
                                lnum_0 = (lnum_0 as ::core::ffi::c_int
                                    + (if foldinfo.fi_lines != 0 {
                                        foldinfo.fi_lines
                                    } else {
                                        1 as linenr_T
                                    }) as ::core::ffi::c_int)
                                    as linenr_T;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                continue 's_2327;
                            } else {
                                if !scrolled_for_mod
                                    && mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                    && lnum_0 >= mod_top
                                    && lnum_0
                                        < (if mod_bot > mod_top + 1 as linenr_T {
                                            mod_bot
                                        } else {
                                            mod_top + 1 as linenr_T
                                        })
                                    && (!scrolled_down || row_0 >= top_end)
                                {
                                    scrolled_for_mod = true_0 != 0;
                                    let mut old_cline_height: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let mut old_rows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut l: linenr_T = 0;
                                    let mut i_3: ::core::ffi::c_int = 0;
                                    i_3 = idx_2;
                                    while i_3 < (*wp).w_lines_valid {
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                                == mod_bot
                                        {
                                            break;
                                        }
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                            == (*wp).w_cursor.lnum
                                        {
                                            old_cline_height = (*(*wp).w_lines.offset(i_3 as isize))
                                                .wl_size
                                                as ::core::ffi::c_int;
                                        }
                                        old_rows += (*(*wp).w_lines.offset(i_3 as isize)).wl_size
                                            as ::core::ffi::c_int;
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lastlnum
                                                + 1 as linenr_T
                                                == mod_bot
                                        {
                                            i_3 += 1;
                                            while i_3 < (*wp).w_lines_valid
                                                && !(*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            {
                                                let c2rust_fresh2 = i_3;
                                                i_3 = i_3 + 1;
                                                old_rows +=
                                                    (*(*wp).w_lines.offset(c2rust_fresh2 as isize))
                                                        .wl_size
                                                        as ::core::ffi::c_int;
                                            }
                                            break;
                                        } else {
                                            i_3 += 1;
                                        }
                                    }
                                    if i_3 >= (*wp).w_lines_valid {
                                        bot_start = 0 as ::core::ffi::c_int;
                                        bot_scroll_start = 0 as ::core::ffi::c_int;
                                    } else {
                                        let mut new_rows: ::core::ffi::c_int =
                                            0 as ::core::ffi::c_int;
                                        let mut j_2: ::core::ffi::c_int = idx_2;
                                        l = lnum_0;
                                        while l < mod_bot {
                                            if dollar_vcol.get() >= 0 as ::core::ffi::c_int
                                                && wp == curwin.get()
                                                && old_cline_height > 0 as ::core::ffi::c_int
                                                && l == (*wp).w_cursor.lnum
                                            {
                                                new_rows += old_cline_height;
                                                j_2 += 1;
                                            } else {
                                                let mut n: ::core::ffi::c_int =
                                                    plines_correct_topline(
                                                        wp,
                                                        l,
                                                        &raw mut l,
                                                        true_0 != 0,
                                                        ::core::ptr::null_mut::<bool>(),
                                                    );
                                                new_rows += n;
                                                j_2 += (n > 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            }
                                            if new_rows
                                                > (*wp).w_view_height
                                                    - row_0
                                                    - 2 as ::core::ffi::c_int
                                            {
                                                new_rows = 9999 as ::core::ffi::c_int;
                                                break;
                                            } else {
                                                l += 1;
                                            }
                                        }
                                        let mut xtra_rows: ::core::ffi::c_int = new_rows - old_rows;
                                        if xtra_rows < 0 as ::core::ffi::c_int {
                                            if row_0 - xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0, xtra_rows);
                                                bot_start = (*wp).w_view_height + xtra_rows;
                                                bot_scroll_start = bot_start;
                                            }
                                        } else if xtra_rows > 0 as ::core::ffi::c_int {
                                            if row_0 + xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0 + old_rows, xtra_rows);
                                                bot_scroll_start = 0 as ::core::ffi::c_int;
                                                if top_end > row_0 + old_rows {
                                                    top_end += xtra_rows;
                                                }
                                            }
                                        }
                                        if mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                            && i_3 != j_2
                                        {
                                            if j_2 < i_3 {
                                                let mut x: ::core::ffi::c_int = row_0 + new_rows;
                                                loop {
                                                    if i_3 >= (*wp).w_lines_valid {
                                                        (*wp).w_lines_valid = j_2;
                                                        break;
                                                    } else {
                                                        *(*wp).w_lines.offset(j_2 as isize) =
                                                            *(*wp).w_lines.offset(i_3 as isize);
                                                        if x + (*(*wp).w_lines.offset(j_2 as isize))
                                                            .wl_size
                                                            as ::core::ffi::c_int
                                                            > (*wp).w_view_height
                                                        {
                                                            (*wp).w_lines_valid =
                                                                j_2 + 1 as ::core::ffi::c_int;
                                                            break;
                                                        } else {
                                                            let c2rust_fresh3 = j_2;
                                                            j_2 = j_2 + 1;
                                                            x += (*(*wp)
                                                                .w_lines
                                                                .offset(c2rust_fresh3 as isize))
                                                            .wl_size
                                                                as ::core::ffi::c_int;
                                                            i_3 += 1;
                                                        }
                                                    }
                                                }
                                                bot_start =
                                                    if bot_start < x { bot_start } else { x };
                                            } else {
                                                j_2 -= i_3;
                                                (*wp).w_lines_valid +=
                                                    j_2 as linenr_T as ::core::ffi::c_int;
                                                (*wp).w_lines_valid =
                                                    if (*wp).w_lines_valid < (*wp).w_view_height {
                                                        (*wp).w_lines_valid
                                                    } else {
                                                        (*wp).w_view_height
                                                    };
                                                i_3 = (*wp).w_lines_valid;
                                                while i_3 - j_2 >= idx_2 {
                                                    *(*wp).w_lines.offset(i_3 as isize) =
                                                        *(*wp).w_lines.offset((i_3 - j_2) as isize);
                                                    i_3 -= 1;
                                                }
                                                while i_3 >= idx_2 {
                                                    (*(*wp).w_lines.offset(i_3 as isize)).wl_size =
                                                        0 as uint16_t;
                                                    let c2rust_fresh4 = i_3;
                                                    i_3 = i_3 - 1;
                                                    (*(*wp)
                                                        .w_lines
                                                        .offset(c2rust_fresh4 as isize))
                                                    .wl_valid = false_0 != 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                if foldinfo.fi_lines == 0 as linenr_T
                                    && idx_2 < (*wp).w_lines_valid
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid
                                        as ::core::ffi::c_int
                                        != 0
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum == lnum_0
                                    && lnum_0 > (*wp).w_topline
                                    && dy_flags.get()
                                        & (kOptDyFlagLastline as ::core::ffi::c_int
                                            | kOptDyFlagTruncate as ::core::ffi::c_int)
                                            as ::core::ffi::c_uint
                                        == 0
                                    && srow_0
                                        + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                            as ::core::ffi::c_int
                                        > (*wp).w_view_height
                                    && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                                {
                                    row_0 = (*wp).w_view_height + 1 as ::core::ffi::c_int;
                                } else {
                                    prepare_search_hl(wp, screen_search_hl.ptr(), lnum_0);
                                    if syntax_last_parsed != 0 as linenr_T
                                        && (syntax_last_parsed + 1 as linenr_T) < lnum_0
                                        && syntax_present(wp) as ::core::ffi::c_int != 0
                                    {
                                        syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                                    }
                                    let mut display_buf_line: bool = !concealed
                                        && (foldinfo.fi_lines == 0 as linenr_T
                                            || *(*wp).w_onebuf_opt.wo_fdt as ::core::ffi::c_int
                                                == NUL);
                                    let mut zero_spv: spellvars_T = spellvars_T {
                                        spv_has_spell: false,
                                        spv_unchanged: false,
                                        spv_checked_col: 0,
                                        spv_checked_lnum: 0,
                                        spv_cap_col: 0,
                                        spv_capcol_lnum: 0,
                                    };
                                    row_0 = win_line(
                                        wp,
                                        lnum_0,
                                        srow_0,
                                        (*wp).w_view_height,
                                        0 as ::core::ffi::c_int,
                                        concealed,
                                        if display_buf_line as ::core::ffi::c_int != 0 {
                                            &raw mut spv
                                        } else {
                                            &raw mut zero_spv
                                        },
                                        foldinfo,
                                    );
                                    if display_buf_line {
                                        syntax_last_parsed = lnum_0;
                                    } else {
                                        spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    }
                                    let mut lastlnum: linenr_T = lnum_0 + foldinfo.fi_lines
                                        - (foldinfo.fi_lines > 0 as linenr_T) as ::core::ffi::c_int;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_folded =
                                        foldinfo.fi_lines > 0 as linenr_T;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_foldend = lastlnum;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum = lastlnum;
                                    did_update = (if foldinfo.fi_lines > 0 as linenr_T {
                                        DID_FOLD as ::core::ffi::c_int
                                    } else {
                                        DID_LINE as ::core::ffi::c_int
                                    })
                                        as C2Rust_Unnamed_25;
                                    let mut virt_below: bool = decor_virt_lines(
                                        wp,
                                        lastlnum as ::core::ffi::c_int,
                                        lastlnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        ::core::ptr::null_mut::<VirtLines>(),
                                        true_0 != 0,
                                    ) > 0 as ::core::ffi::c_int;
                                    while !virt_below
                                        && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                            < (*buf).b_ml.ml_line_count
                                        && decor_conceal_line(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                                as ::core::ffi::c_int,
                                            false_0 != 0,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        virt_below = false_0 != 0;
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum += 1;
                                        hasFolding(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            &raw mut (*(*wp).w_lines.offset(idx_2 as isize))
                                                .wl_lastlnum,
                                        );
                                    }
                                }
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum = lnum_0;
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid = true_0 != 0;
                                let mut is_curline: bool =
                                    wp == curwin.get() && lnum_0 == (*wp).w_cursor.lnum;
                                if row_0 > (*wp).w_view_height {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            plines_win(wp, lnum_0, true_0 != 0) as uint16_t;
                                    }
                                    idx_2 += 1;
                                    break 's_2139;
                                } else {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            (row_0 - srow_0) as uint16_t;
                                    }
                                    let c2rust_fresh5 = idx_2;
                                    idx_2 = idx_2 + 1;
                                    lnum_0 = (*(*wp).w_lines.offset(c2rust_fresh5 as isize))
                                        .wl_lastlnum
                                        + 1 as linenr_T;
                                }
                            }
                        } else {
                            if (*wp).w_onebuf_opt.wo_nu != 0
                                && mod_top != 0 as linenr_T
                                && lnum_0 >= mod_bot
                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                && (*buf).b_mod_xlines != 0 as linenr_T
                                || (*wp).w_onebuf_opt.wo_rnu != 0
                                    && (*wp).w_last_cursor_lnum_rnu != (*wp).w_cursor.lnum
                            {
                                let mut info: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                    && lnum_0 == (*wp).w_cursor.lnum
                                {
                                    cursorline_fi
                                } else {
                                    fold_info(wp, lnum_0)
                                };
                                win_line(
                                    wp,
                                    lnum_0,
                                    srow_0,
                                    (*wp).w_view_height,
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                        as ::core::ffi::c_int,
                                    false_0 != 0,
                                    &raw mut spv,
                                    info,
                                );
                            }
                            let c2rust_fresh6 = idx_2;
                            idx_2 = idx_2 + 1;
                            row_0 += (*(*wp).w_lines.offset(c2rust_fresh6 as isize)).wl_size
                                as ::core::ffi::c_int;
                            if row_0 > (*wp).w_view_height {
                                break 's_2139;
                            } else {
                                lnum_0 = (*(*wp)
                                    .w_lines
                                    .offset((idx_2 - 1 as ::core::ffi::c_int) as isize))
                                .wl_lastlnum
                                    + 1 as linenr_T;
                                did_update = DID_NONE;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                        if (*wp).w_redr_statuscol {
                            break '_redr_statuscol;
                        } else {
                            if lnum_0 <= (*buf).b_ml.ml_line_count {
                                continue 's_2327;
                            }
                            eof = true_0 != 0;
                        }
                    }
                }
                (*wp).w_last_cursorline = (*wp).w_cursorline;
                (*wp).w_last_cursor_lnum_rnu = if (*wp).w_onebuf_opt.wo_rnu != 0 {
                    (*wp).w_cursor.lnum
                } else {
                    0 as linenr_T
                };
                (*wp).w_lines_valid = if (*wp).w_lines_valid > idx_2 {
                    (*wp).w_lines_valid
                } else {
                    idx_2
                };
                (*wp).w_display_tick = display_tick.get();
                if syntax_last_parsed != 0 as linenr_T
                    && syntax_present(wp) as ::core::ffi::c_int != 0
                {
                    syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                }
                old_botline = (*wp).w_botline;
                (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
                (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
                if !eof && !didline {
                    let mut at_attr: ::core::ffi::c_int = hl_combine_attr(
                        win_bg_attr(wp),
                        win_hl_attr(wp, HLF_AT as ::core::ffi::c_int),
                    );
                    if lnum_0 == (*wp).w_topline {
                        (*wp).w_botline = lnum_0 + 1 as linenr_T;
                    } else if win_get_fill(wp, lnum_0) >= (*wp).w_view_height - srow_0 {
                        (*wp).w_botline = lnum_0;
                        (*wp).w_filler_rows = (*wp).w_view_height - srow_0;
                    } else if dy_flags.get()
                        & kOptDyFlagTruncate as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        grid_line_fill(
                            0 as ::core::ffi::c_int,
                            if (*wp).w_view_width < 3 as ::core::ffi::c_int {
                                (*wp).w_view_width
                            } else {
                                3 as ::core::ffi::c_int
                            },
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_fill(
                            3 as ::core::ffi::c_int,
                            (*wp).w_view_width,
                            ' ' as ::core::ffi::c_int as schar_T,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else if dy_flags.get()
                        & kOptDyFlagLastline as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        let mut width: ::core::ffi::c_int = if grid_line_getchar(
                            if (*wp).w_view_width - 3 as ::core::ffi::c_int
                                > 0 as ::core::ffi::c_int
                            {
                                (*wp).w_view_width - 3 as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ) == NUL as schar_T
                        {
                            4 as ::core::ffi::c_int
                        } else {
                            3 as ::core::ffi::c_int
                        };
                        grid_line_fill(
                            if (*wp).w_view_width - width > 0 as ::core::ffi::c_int {
                                (*wp).w_view_width - width
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            (*wp).w_view_width,
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else {
                        win_draw_end(
                            wp,
                            (*wp).w_p_fcs_chars.lastline,
                            true_0 != 0,
                            srow_0,
                            (*wp).w_view_height,
                            HLF_AT,
                        );
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2363;
                } else if eof {
                    (*wp).w_botline = (*buf).b_ml.ml_line_count + 1 as linenr_T;
                    let mut j_3: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_botline);
                    if !(j_3 > 0 as ::core::ffi::c_int
                        && !(*wp).w_botfill
                        && row_0 < (*wp).w_view_height)
                    {
                        break 's_2327;
                    }
                    let mut zero_spv_0: spellvars_T = spellvars_T {
                        spv_has_spell: false,
                        spv_unchanged: false,
                        spv_checked_col: 0,
                        spv_checked_lnum: 0,
                        spv_cap_col: 0,
                        spv_capcol_lnum: 0,
                    };
                    let mut zero_foldinfo: foldinfo_T = foldinfo_T {
                        fi_lnum: 0 as linenr_T,
                        fi_level: 0,
                        fi_low_level: 0,
                        fi_lines: 0,
                    };
                    row_0 = win_line(
                        wp,
                        (*wp).w_botline,
                        row_0,
                        (*wp).w_view_height,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        &raw mut zero_spv_0,
                        zero_foldinfo,
                    );
                    if !(*wp).w_redr_statuscol {
                        break 's_2327;
                    }
                    eof = false_0 != 0;
                } else {
                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2327;
                }
            }
            (*wp).w_redr_statuscol = false_0 != 0;
            idx_2 = 0 as ::core::ffi::c_int;
            row_0 = 0 as ::core::ffi::c_int;
            lnum_0 = (*wp).w_topline;
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
            (*wp).w_valid &= !VALID_WCOL;
            decor_redraw_reset(wp, decor_state.ptr());
            decor_providers_invoke_win(wp);
        }
        let mut lastline: ::core::ffi::c_int = bot_scroll_start;
        if mid_end >= row_0 {
            lastline = if lastline < mid_start {
                lastline
            } else {
                mid_start
            };
        }
        if mod_bot > (*buf).b_ml.ml_line_count {
            lastline = 0 as ::core::ffi::c_int;
        }
        win_draw_end(
            wp,
            (*wp).w_p_fcs_chars.eob,
            false_0 != 0,
            if lastline > row_0 { lastline } else { row_0 },
            (*wp).w_view_height,
            HLF_EOB,
        );
        set_empty_rows(wp, row_0);
    }
    if (*wp).w_redr_type >= UPD_REDRAW_TOP as ::core::ffi::c_int {
        draw_vsep_win(wp);
        draw_hsep_win(wp);
    }
    syn_set_timeout(::core::ptr::null_mut::<proftime_T>());
    (*wp).w_redr_type = 0 as ::core::ffi::c_int;
    (*wp).w_old_topfill = (*wp).w_topfill;
    (*wp).w_old_botfill = (*wp).w_botfill;
    let mut n_0: size_t = 0 as size_t;
    while n_0 < (*win_extmark_arr.ptr()).size {
        ui_call_win_extmark(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).ns_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).mark_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_row as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_col as Integer,
        );
        n_0 = n_0.wrapping_add(1);
    }
    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
        (*wp).w_valid |= VALID_BOTLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
        if wp == curwin.get() && (*wp).w_botline != old_botline && !recursive.get() {
            recursive.set(true_0 != 0);
            (*curwin.get()).w_valid &= !VALID_TOPLINE;
            update_topline(curwin.get());
            if must_redraw.get() != 0 as ::core::ffi::c_int {
                let mut mod_set: ::core::ffi::c_int =
                    (*curbuf.get()).b_mod_set as ::core::ffi::c_int;
                (*curbuf.get()).b_mod_set = false_0 != 0;
                curs_columns(curwin.get(), true_0);
                win_update(curwin.get());
                must_redraw.set(0 as ::core::ffi::c_int);
                (*curbuf.get()).b_mod_set = mod_set != 0;
            }
            recursive.set(false_0 != 0);
        }
    }
    if nrwidth_before != (*wp).w_nrwidth && !(*buf).terminal.is_null() {
        terminal_check_size((*buf).terminal);
    }
    if !got_int.get() {
        got_int.set(save_got_int != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn win_scroll_lines(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
) {
    if !redrawing() || line_count == 0 as ::core::ffi::c_int {
        return;
    }
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid =
        grid_adjust(&raw mut (*wp).w_grid, &raw mut row_off, &raw mut col);
    let mut checked_width: ::core::ffi::c_int = if (*grid).cols - col < (*wp).w_view_width {
        (*grid).cols - col
    } else {
        (*wp).w_view_width
    };
    let mut checked_height: ::core::ffi::c_int = if (*grid).rows - row_off < (*wp).w_view_height {
        (*grid).rows - row_off
    } else {
        (*wp).w_view_height
    };
    if row + abs(line_count) >= checked_height {
        return;
    }
    if line_count < 0 as ::core::ffi::c_int {
        grid_del_lines(
            grid,
            row + row_off,
            -line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    } else {
        grid_ins_lines(
            grid,
            row + row_off,
            line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn win_draw_end(
    mut wp: *mut win_T,
    mut c1: schar_T,
    mut draw_margin: bool,
    mut startrow: ::core::ffi::c_int,
    mut endrow: ::core::ffi::c_int,
    mut hl: hlf_T,
) {
    '_c2rust_label: {
        if hl as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
            && (hl as ::core::ffi::c_uint) < HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"hl >= 0 && hl < HLF_COUNT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2513 as ::core::ffi::c_uint,
                b"void win_draw_end(win_T *, schar_T, _Bool, int, int, hlf_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let view_width: ::core::ffi::c_int = (*wp).w_view_width;
    let fdc: ::core::ffi::c_int = compute_foldcolumn(wp, 0 as ::core::ffi::c_int);
    let scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
    let mut row: ::core::ffi::c_int = startrow;
    while row < endrow {
        grid_line_start(&raw mut (*wp).w_grid, row);
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if draw_margin {
            if fdc > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + fdc {
                        view_width
                    } else {
                        n + fdc
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_FC as ::core::ffi::c_int),
                );
            }
            if scwidth > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + scwidth * SIGN_WIDTH as ::core::ffi::c_int {
                        view_width
                    } else {
                        n + scwidth * SIGN_WIDTH as ::core::ffi::c_int
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_SC as ::core::ffi::c_int),
                );
            }
            if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                && vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
            {
                let mut width: ::core::ffi::c_int = number_width(wp) + 1 as ::core::ffi::c_int;
                n = grid_line_fill(
                    n,
                    if view_width < n + width {
                        view_width
                    } else {
                        n + width
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_N as ::core::ffi::c_int),
                );
            }
        }
        let mut attr: ::core::ffi::c_int = win_hl_attr(wp, hl as ::core::ffi::c_int);
        if n < view_width {
            grid_line_put_schar(n, c1, attr);
            n += 1;
        }
        grid_line_clear_end(n, view_width, win_bg_attr(wp), attr);
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            grid_line_mirror(view_width);
        }
        grid_line_flush();
        row += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn compute_foldcolumn(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut fdc: ::core::ffi::c_int = win_fdccol_count(wp);
    let mut wmw: ::core::ffi::c_int = if wp == curwin.get() && p_wmw.get() == 0 as OptInt {
        1 as ::core::ffi::c_int
    } else {
        p_wmw.get() as ::core::ffi::c_int
    };
    let mut n: ::core::ffi::c_int = (*wp).w_view_width - (col + wmw);
    return if fdc < n { fdc } else { n };
}
#[no_mangle]
pub unsafe extern "C" fn number_width(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = 0;
    if (*wp).w_onebuf_opt.wo_rnu != 0 && (*wp).w_onebuf_opt.wo_nu == 0 {
        lnum = (*wp).w_view_height as linenr_T;
    } else {
        lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
    }
    if lnum == (*wp).w_nrwidth_line_count {
        return (*wp).w_nrwidth_width;
    }
    (*wp).w_nrwidth_line_count = lnum;
    if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL {
        (*wp).w_statuscol_line_count = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_nrwidth_width = ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
            as ::core::ffi::c_int
            * (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int;
        return (*wp).w_nrwidth_width;
    }
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        lnum = (lnum as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as linenr_T;
        n += 1;
        if lnum <= 0 as linenr_T {
            break;
        }
    }
    n = if n > (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        n
    } else {
        (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    if n < 2 as ::core::ffi::c_int
        && buf_meta_total((*wp).w_buffer, kMTMetaSignText) != 0
        && (*wp).w_minscwidth == SCL_NUM
    {
        n = 2 as ::core::ffi::c_int;
    }
    (*wp).w_nrwidth_width = n;
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_later(mut wp: *mut win_T, mut type_0: ::core::ffi::c_int) {
    '_c2rust_label: {
        if !wp.is_null() || exiting.get() as ::core::ffi::c_int != 0 {
        } else {
            __assert_fail(
                b"wp != NULL || exiting\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2623 as ::core::ffi::c_uint,
                b"void redraw_later(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !exiting.get() && !redraw_not_allowed.get() && (*wp).w_redr_type < type_0 {
        (*wp).w_redr_type = type_0;
        if type_0 >= UPD_NOT_VALID as ::core::ffi::c_int {
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        }
        must_redraw.set(if must_redraw.get() > type_0 {
            must_redraw.get()
        } else {
            type_0
        });
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_all_later(mut type_0: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        redraw_later(wp, type_0);
        wp = (*wp).w_next;
    }
    set_must_redraw(type_0);
}
#[no_mangle]
pub unsafe extern "C" fn set_must_redraw(mut type_0: ::core::ffi::c_int) {
    if !redraw_not_allowed.get() {
        must_redraw.set(if must_redraw.get() > type_0 {
            must_redraw.get()
        } else {
            type_0
        });
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_invalidate_highlights() {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        (*wp).w_grid_alloc.valid = false_0 != 0;
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_curbuf_later(mut type_0: ::core::ffi::c_int) {
    redraw_buf_later(curbuf.get(), type_0);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_later(mut buf: *mut buf_T, mut type_0: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redraw_later(wp, type_0);
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_line_later(
    mut buf: *mut buf_T,
    mut line: linenr_T,
    mut force: bool,
) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redrawWinline(
                wp,
                if line < (*buf).b_ml.ml_line_count {
                    line
                } else {
                    (*buf).b_ml.ml_line_count
                },
            );
            if force as ::core::ffi::c_int != 0 && line > (*buf).b_ml.ml_line_count {
                (*wp).w_redraw_bot = line;
            }
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_win_range_later(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    if last >= (*wp).w_topline && first < (*wp).w_botline {
        if (*wp).w_redraw_top == 0 as linenr_T || (*wp).w_redraw_top > first {
            (*wp).w_redraw_top = first;
        }
        if (*wp).w_redraw_bot == 0 as linenr_T || (*wp).w_redraw_bot < last {
            (*wp).w_redraw_bot = last;
        }
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn redrawWinline(mut wp: *mut win_T, mut lnum: linenr_T) {
    redraw_win_range_later(wp, lnum, lnum);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_range_later(
    mut buf: *mut buf_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redraw_win_range_later(wp, first, last);
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_status_later(mut buf: *mut buf_T) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf
            && ((*wp).w_status_height != 0
                || wp == curwin.get() && global_stl_height() != 0
                || (*wp).w_winbar_height != 0)
        {
            (*wp).w_redr_status = true_0 != 0;
            set_must_redraw(UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_redraw_all() {
    let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !is_stl_global && (*wp).w_status_height != 0
            || wp == curwin.get()
            || (*wp).w_winbar_height != 0
        {
            (*wp).w_redr_status = true_0 != 0;
            redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_redraw_curbuf() {
    status_redraw_buf(curbuf.get());
}
#[no_mangle]
pub unsafe extern "C" fn status_redraw_buf(mut buf: *mut buf_T) {
    let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf
            && (!is_stl_global && (*wp).w_status_height != 0
                || is_stl_global as ::core::ffi::c_int != 0 && wp == curwin.get()
                || (*wp).w_winbar_height != 0)
        {
            (*wp).w_redr_status = true_0 != 0;
            redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
    if p_ru.get() != 0 && (*curwin.get()).w_status_height == 0 && !(*curwin.get()).w_redr_status {
        redraw_cmdline.set(true_0 != 0);
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_statuslines() {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_redr_status {
            win_check_ns_hl(wp);
            win_redr_winbar(wp);
            win_redr_status(wp);
        }
        wp = (*wp).w_next;
    }
    win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    if redraw_tabline.get() {
        draw_tabline();
    }
    if need_maketitle.get() {
        maketitle();
    }
}
#[no_mangle]
pub unsafe extern "C" fn win_redraw_last_status(mut frp: *const frame_T) {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        (*(*frp).fr_win).w_redr_status = true_0 != 0;
    } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            win_redraw_last_status(frp);
            frp = (*frp).fr_next;
        }
    } else {
        '_c2rust_label: {
            if (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"frp->fr_layout == FR_COL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2806 as ::core::ffi::c_uint,
                    b"void win_redraw_last_status(const frame_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frp = (*frp).fr_child;
        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        win_redraw_last_status(frp);
    };
}
#[no_mangle]
pub unsafe extern "C" fn conceal_cursor_line(mut wp: *const win_T) -> bool {
    let mut c: ::core::ffi::c_int = 0;
    if *(*wp).w_onebuf_opt.wo_cocu as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if get_real_state() & MODE_VISUAL as ::core::ffi::c_int != 0 {
        c = 'v' as ::core::ffi::c_int;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        c = 'i' as ::core::ffi::c_int;
    } else if State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        c = 'n' as ::core::ffi::c_int;
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        c = 'c' as ::core::ffi::c_int;
    } else {
        return false_0 != 0;
    }
    return !vim_strchr((*wp).w_onebuf_opt.wo_cocu, c).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn win_cursorline_standout(mut wp: *const win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_cul != 0
        || wp == curwin.get() as *const win_T
            && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
            && !conceal_cursor_line(wp);
}
#[no_mangle]
pub unsafe extern "C" fn win_update_cursorline(mut wp: *mut win_T, mut foldinfo: *mut foldinfo_T) {
    (*wp).w_cursorline = if win_cursorline_standout(wp) as ::core::ffi::c_int != 0 {
        (*wp).w_cursor.lnum
    } else {
        0 as linenr_T
    };
    if (*wp).w_onebuf_opt.wo_cul != 0 {
        *foldinfo = fold_info(wp, (*wp).w_cursor.lnum);
        if (*foldinfo).fi_level != 0 as ::core::ffi::c_int && (*foldinfo).fi_lines > 0 as linenr_T {
            (*wp).w_cursorline = (*foldinfo).fi_lnum;
        }
    }
}
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active.get()
    }
    .offset(hlf as isize);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
