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
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    static breakat_flags: GlobalCell<[::core::ffi::c_char; 256]>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static dy_flags: GlobalCell<::core::ffi::c_uint>;
    static p_sel: GlobalCell<*mut ::core::ffi::c_char>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn transstr_buf(
        s: *const ::core::ffi::c_char,
        slen: ssize_t,
        buf: *mut ::core::ffi::c_char,
        buflen: size_t,
        untab: bool,
    ) -> size_t;
    fn transchar_buf(buf: *const buf_T, c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn transchar_hex(buf: *mut ::core::ffi::c_char, c: ::core::ffi::c_int) -> size_t;
    fn rl_mirror_ascii(str: *mut ::core::ffi::c_char, end: *mut ::core::ffi::c_char);
    fn byte2cells(b: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_cursor_rel_lnum(wp: *mut win_T, lnum: linenr_T) -> linenr_T;
    fn cursor_is_block_during_visual(exclusive: bool) -> bool;
    static decor_state: GlobalCell<DecorState>;
    fn clear_virttext(text: *mut VirtText);
    fn next_virt_text_chunk(
        vt: VirtText,
        pos: *mut size_t,
        attr: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn decor_virt_pos(decor: *const DecorRange) -> bool;
    fn decor_virt_pos_kind(decor: *const DecorRange) -> VirtTextPos;
    fn decor_redraw_line(wp: *mut win_T, row: ::core::ffi::c_int, state: *mut DecorState);
    fn decor_has_more_decorations(state: *mut DecorState, row: ::core::ffi::c_int) -> bool;
    fn decor_init_draw_col(win_col: ::core::ffi::c_int, hidden: bool, item: *mut DecorRange);
    fn decor_recheck_draw_col(win_col: ::core::ffi::c_int, hidden: bool, state: *mut DecorState);
    fn decor_redraw_col_impl(
        wp: *mut win_T,
        col: ::core::ffi::c_int,
        win_col: ::core::ffi::c_int,
        hidden: bool,
        state: *mut DecorState,
        max_col_last: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn decor_redraw_signs(
        wp: *mut win_T,
        buf: *mut buf_T,
        row: ::core::ffi::c_int,
        sattrs: *mut SignTextAttrs,
        line_id: *mut ::core::ffi::c_int,
        cul_id: *mut ::core::ffi::c_int,
        num_id: *mut ::core::ffi::c_int,
    );
    fn decor_redraw_eol(
        wp: *mut win_T,
        state: *mut DecorState,
        eol_attr: *mut ::core::ffi::c_int,
        eol_col: ::core::ffi::c_int,
    ) -> bool;
    fn decor_virt_lines(
        wp: *mut win_T,
        start_row: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        num_below: *mut ::core::ffi::c_int,
        lines: *mut VirtLines,
        apply_folds: bool,
    ) -> ::core::ffi::c_int;
    fn decor_providers_invoke_line(wp: *mut win_T, row: ::core::ffi::c_int);
    fn decor_providers_invoke_range(
        wp: *mut win_T,
        start_row: ::core::ffi::c_int,
        start_col: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
    );
    fn diff_check_with_linestatus(
        wp: *mut win_T,
        lnum: linenr_T,
        linestatus: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn diff_change_parse(
        diffline: *mut diffline_T,
        change: *mut diffline_change_T,
        change_start: *mut ::core::ffi::c_int,
        change_end: *mut ::core::ffi::c_int,
    ) -> bool;
    fn diff_find_change(wp: *mut win_T, lnum: linenr_T, diffline: *mut diffline_T) -> bool;
    static win_extmark_arr: GlobalCell<C2Rust_Unnamed_30>;
    static screen_search_hl: GlobalCell<match_T>;
    fn win_draw_end(
        wp: *mut win_T,
        c1: schar_T,
        draw_margin: bool,
        startrow: ::core::ffi::c_int,
        endrow: ::core::ffi::c_int,
        hl: hlf_T,
    );
    fn compute_foldcolumn(wp: *mut win_T, col: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn number_width(wp: *mut win_T) -> ::core::ffi::c_int;
    fn conceal_cursor_line(wp: *const win_T) -> bool;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn get_foldtext(
        wp: *mut win_T,
        lnum: linenr_T,
        lnume: linenr_T,
        foldinfo: foldinfo_T,
        buf: *mut ::core::ffi::c_char,
        vt: *mut VirtText,
    ) -> *mut ::core::ffi::c_char;
    fn grid_adjust(
        grid: *mut GridView,
        row_off: *mut ::core::ffi::c_int,
        col_off: *mut ::core::ffi::c_int,
    ) -> *mut ScreenGrid;
    fn schar_get_adv(buf_out: *mut *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn schar_len(sc: schar_T) -> size_t;
    fn schar_cells(sc: schar_T) -> ::core::ffi::c_int;
    fn schar_get_first_codepoint(sc: schar_T) -> ::core::ffi::c_int;
    fn schar_get_ascii(sc: schar_T) -> ::core::ffi::c_char;
    fn linebuf_mirror(
        firstp: *mut ::core::ffi::c_int,
        lastp: *mut ::core::ffi::c_int,
        clearp: *mut ::core::ffi::c_int,
        width: ::core::ffi::c_int,
    );
    fn grid_put_linebuf(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        coloff: ::core::ffi::c_int,
        col: ::core::ffi::c_int,
        endcol: ::core::ffi::c_int,
        clear_width: ::core::ffi::c_int,
        bg_attr: ::core::ffi::c_int,
        clear_attr: ::core::ffi::c_int,
        last_vcol: colnr_T,
        flags: ::core::ffi::c_int,
    );
    fn schar_from_char(c: ::core::ffi::c_int) -> schar_T;
    static linebuf_char: GlobalCell<*mut schar_T>;
    static linebuf_attr: GlobalCell<*mut sattr_T>;
    static linebuf_vcol: GlobalCell<*mut colnr_T>;
    fn win_bg_attr(wp: *mut win_T) -> ::core::ffi::c_int;
    fn hl_get_underline() -> ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn hl_blend_attrs(
        back_attr: ::core::ffi::c_int,
        front_attr: ::core::ffi::c_int,
        through: *mut bool,
    ) -> ::core::ffi::c_int;
    fn syn_attr2entry(attr: ::core::ffi::c_int) -> HlAttrs;
    static highlight_attr: GlobalCell<[::core::ffi::c_int; 76]>;
    static cterm_normal_bg_color: GlobalCell<::core::ffi::c_int>;
    static normal_bg: GlobalCell<RgbValue>;
    static ns_hl_fast: GlobalCell<NS>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn tabstop_padding(col: colnr_T, ts_arg: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int;
    fn get_breakindent_win(wp: *mut win_T, line: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ins_compl_col_range_attr(lnum: linenr_T, col: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ins_compl_lnum_in_range(lnum: linenr_T) -> bool;
    fn ins_compl_win_active(wp: *mut win_T) -> bool;
    fn prepare_search_hl_line(
        wp: *mut win_T,
        lnum: linenr_T,
        mincol: colnr_T,
        line: *mut *mut ::core::ffi::c_char,
        search_hl: *mut match_T,
        search_attr: *mut ::core::ffi::c_int,
        search_attr_from_match: *mut bool,
    ) -> bool;
    fn update_search_hl(
        wp: *mut win_T,
        lnum: linenr_T,
        col: colnr_T,
        line: *mut *mut ::core::ffi::c_char,
        search_hl: *mut match_T,
        has_match_conc: *mut ::core::ffi::c_int,
        match_conc: *mut ::core::ffi::c_int,
        lcs_eol_todo: bool,
        on_last_col: *mut bool,
        search_attr_from_match: *mut bool,
    ) -> ::core::ffi::c_int;
    fn get_prevcol_hl_flag(wp: *mut win_T, search_hl: *mut match_T, curcol: colnr_T) -> bool;
    fn get_search_match_hl(
        wp: *mut win_T,
        search_hl: *mut match_T,
        col: colnr_T,
        char_attr: *mut ::core::ffi::c_int,
    );
    static utf8len_tab: [uint8_t; 256];
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn mb_ptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2schar(p: *const ::core::ffi::c_char, firstc: *mut ::core::ffi::c_int) -> schar_T;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn mb_off_next(
        base: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int;
    fn validate_virtcol(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_col_off2(wp: *mut win_T) -> ::core::ffi::c_int;
    fn set_empty_rows(wp: *mut win_T, used: ::core::ffi::c_int);
    fn get_showbreak_value(win: *mut win_T) -> *mut ::core::ffi::c_char;
    fn init_charsize_arg(
        csarg: *mut CharsizeArg,
        wp: *mut win_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
    ) -> CSType;
    fn charsize_regular(
        csarg: *mut CharsizeArg,
        cur: *mut ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
    fn charsize_fast(
        csarg: *mut CharsizeArg,
        cur: *const ::core::ffi::c_char,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> CharSize;
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
    fn qf_current_entry(wp: *mut win_T) -> linenr_T;
    fn virtual_active(wp: *mut win_T) -> bool;
    fn spell_check(
        wp: *mut win_T,
        ptr: *mut ::core::ffi::c_char,
        attrp: *mut hlf_T,
        capcol: *mut ::core::ffi::c_int,
        docount: bool,
    ) -> size_t;
    fn spell_move_to(
        wp: *mut win_T,
        dir: ::core::ffi::c_int,
        behaviour: smt_T,
        curline: bool,
        attrp: *mut hlf_T,
    ) -> size_t;
    fn spell_cat_line(
        buf: *mut ::core::ffi::c_char,
        line: *mut ::core::ffi::c_char,
        maxlen: ::core::ffi::c_int,
    );
    fn check_need_cap(wp: *mut win_T, lnum: linenr_T, col: colnr_T) -> bool;
    fn spell_to_word_end(
        start: *mut ::core::ffi::c_char,
        win: *mut win_T,
    ) -> *mut ::core::ffi::c_char;
    fn build_statuscol_str(
        wp: *mut win_T,
        lnum: linenr_T,
        relnum: linenr_T,
        buf: *mut ::core::ffi::c_char,
        stcp: *mut statuscol_T,
    ) -> ::core::ffi::c_int;
    fn syntax_start(wp: *mut win_T, lnum: linenr_T);
    fn get_syntax_attr(col: colnr_T, can_spell: *mut bool, keep_state: bool) -> ::core::ffi::c_int;
    fn syntax_present(win: *mut win_T) -> bool;
    fn get_syntax_info(seqnrp: *mut ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn syn_get_sub_char() -> ::core::ffi::c_int;
    fn terminal_get_line_attributes(
        term: *mut Terminal,
        wp: *mut win_T,
        linenr: ::core::ffi::c_int,
        term_attrs: *mut ::core::ffi::c_int,
    );
    fn ui_rgb_attached() -> bool;
    static dollar_vcol: GlobalCell<colnr_T>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static highlight_match: GlobalCell<bool>;
    static search_match_lines: GlobalCell<linenr_T>;
    static search_match_endcol: GlobalCell<colnr_T>;
    static curwin: GlobalCell<*mut win_T>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_mode: GlobalCell<::core::ffi::c_int>;
    static State: GlobalCell<::core::ffi::c_int>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static cmdwin_win: GlobalCell<*mut win_T>;
    static spell_redraw_lnum: GlobalCell<linenr_T>;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
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
pub type NS = handle_T;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_14 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_14 = 1;
pub type DecorPriorityInternal = uint32_t;
pub type HlMode = ::core::ffi::c_uint;
pub const kHlModeBlend: HlMode = 3;
pub const kHlModeCombine: HlMode = 2;
pub const kHlModeReplace: HlMode = 1;
pub const kHlModeUnknown: HlMode = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_15 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_15 = 4;
pub const kVTHide: C2Rust_Unnamed_15 = 2;
pub const kVTIsLines: C2Rust_Unnamed_15 = 1;
pub type RgbValue = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
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
    pub s: [C2Rust_Unnamed_16; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_17 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_17 = 3;
pub const BACKWARD: C2Rust_Unnamed_17 = -1;
pub const FORWARD: C2Rust_Unnamed_17 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptFlagColon: C2Rust_Unnamed_18 = 33554432;
pub const kOptFlagFunc: C2Rust_Unnamed_18 = 16777216;
pub const kOptFlagMLE: C2Rust_Unnamed_18 = 8388608;
pub const kOptFlagHLOnly: C2Rust_Unnamed_18 = 4194304;
pub const kOptFlagNDname: C2Rust_Unnamed_18 = 2097152;
pub const kOptFlagCurswant: C2Rust_Unnamed_18 = 1048576;
pub const kOptFlagPriMkrc: C2Rust_Unnamed_18 = 524288;
pub const kOptFlagInsecure: C2Rust_Unnamed_18 = 262144;
pub const kOptFlagNFname: C2Rust_Unnamed_18 = 131072;
pub const kOptFlagNoGlob: C2Rust_Unnamed_18 = 65536;
pub const kOptFlagGettext: C2Rust_Unnamed_18 = 32768;
pub const kOptFlagSecure: C2Rust_Unnamed_18 = 16384;
pub const kOptFlagFlagList: C2Rust_Unnamed_18 = 8192;
pub const kOptFlagNoDup: C2Rust_Unnamed_18 = 4096;
pub const kOptFlagOneComma: C2Rust_Unnamed_18 = 3072;
pub const kOptFlagComma: C2Rust_Unnamed_18 = 1024;
pub const kOptFlagRedrClear: C2Rust_Unnamed_18 = 896;
pub const kOptFlagRedrAll: C2Rust_Unnamed_18 = 768;
pub const kOptFlagRedrBuf: C2Rust_Unnamed_18 = 512;
pub const kOptFlagRedrWin: C2Rust_Unnamed_18 = 256;
pub const kOptFlagRedrStat: C2Rust_Unnamed_18 = 128;
pub const kOptFlagRedrTabl: C2Rust_Unnamed_18 = 64;
pub const kOptFlagUIOption: C2Rust_Unnamed_18 = 32;
pub const kOptFlagNoMkrc: C2Rust_Unnamed_18 = 16;
pub const kOptFlagWasSet: C2Rust_Unnamed_18 = 8;
pub const kOptFlagNoDefault: C2Rust_Unnamed_18 = 4;
pub const kOptFlagNoDefExp: C2Rust_Unnamed_18 = 2;
pub const kOptFlagExpand: C2Rust_Unnamed_18 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct foldinfo_T {
    pub fi_lnum: linenr_T,
    pub fi_level: ::core::ffi::c_int,
    pub fi_low_level: ::core::ffi::c_int,
    pub fi_lines: linenr_T,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const FOLD_TEXT_LEN: C2Rust_Unnamed_19 = 51;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SIGN_SHOW_MAX: C2Rust_Unnamed_20 = 9;
pub type StlFlag = ::core::ffi::c_uint;
pub const STL_CLICK_FUNC: StlFlag = 64;
pub const STL_TABCLOSENR: StlFlag = 88;
pub const STL_TABPAGENR: StlFlag = 84;
pub const STL_HIGHLIGHT_COMB: StlFlag = 36;
pub const STL_HIGHLIGHT: StlFlag = 35;
pub const STL_USER_HL: StlFlag = 42;
pub const STL_TRUNCMARK: StlFlag = 60;
pub const STL_SEPARATE: StlFlag = 61;
pub const STL_VIM_EXPR: StlFlag = 123;
pub const STL_SIGNCOL: StlFlag = 115;
pub const STL_FOLDCOL: StlFlag = 67;
pub const STL_SHOWCMD: StlFlag = 83;
pub const STL_PAGENUM: StlFlag = 78;
pub const STL_ARGLISTSTAT: StlFlag = 97;
pub const STL_ALTPERCENT: StlFlag = 80;
pub const STL_PERCENTAGE: StlFlag = 112;
pub const STL_QUICKFIX: StlFlag = 113;
pub const STL_MODIFIED_ALT: StlFlag = 77;
pub const STL_MODIFIED: StlFlag = 109;
pub const STL_PREVIEWFLAG_ALT: StlFlag = 87;
pub const STL_PREVIEWFLAG: StlFlag = 119;
pub const STL_FILETYPE_ALT: StlFlag = 89;
pub const STL_FILETYPE: StlFlag = 121;
pub const STL_HELPFLAG_ALT: StlFlag = 72;
pub const STL_HELPFLAG: StlFlag = 104;
pub const STL_ROFLAG_ALT: StlFlag = 82;
pub const STL_ROFLAG: StlFlag = 114;
pub const STL_BYTEVAL_X: StlFlag = 66;
pub const STL_BYTEVAL: StlFlag = 98;
pub const STL_OFFSET_X: StlFlag = 79;
pub const STL_OFFSET: StlFlag = 111;
pub const STL_KEYMAP: StlFlag = 107;
pub const STL_BUFNO: StlFlag = 110;
pub const STL_NUMLINES: StlFlag = 76;
pub const STL_LINE: StlFlag = 108;
pub const STL_VIRTCOL_ALT: StlFlag = 86;
pub const STL_VIRTCOL: StlFlag = 118;
pub const STL_COLUMN: StlFlag = 99;
pub const STL_FILENAME: StlFlag = 116;
pub const STL_FULLPATH: StlFlag = 70;
pub const STL_FILEPATH: StlFlag = 102;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stl_hlrec {
    pub start: *mut ::core::ffi::c_char,
    pub userhl: ::core::ffi::c_int,
    pub item: StlFlag,
}
pub type stl_hlrec_t = stl_hlrec;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct statuscol_T {
    pub width: ::core::ffi::c_int,
    pub lnum: linenr_T,
    pub sign_cul_id: ::core::ffi::c_int,
    pub draw: bool,
    pub hlrec: *mut stl_hlrec_t,
    pub foldinfo: foldinfo_T,
    pub fold_vcol: [colnr_T; 9],
    pub sattrs: *mut SignTextAttrs,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffline_change_S {
    pub dc_start: [colnr_T; 8],
    pub dc_end: [colnr_T; 8],
    pub dc_start_lnum_off: [::core::ffi::c_int; 8],
    pub dc_end_lnum_off: [::core::ffi::c_int; 8],
}
pub type diffline_change_T = diffline_change_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffline_S {
    pub changes: *mut diffline_change_T,
    pub num_changes: ::core::ffi::c_int,
    pub bufidx: ::core::ffi::c_int,
    pub lineoff: ::core::ffi::c_int,
}
pub type diffline_T = diffline_S;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptCuloptFlagNumber: C2Rust_Unnamed_21 = 4;
pub const kOptCuloptFlagScreenline: C2Rust_Unnamed_21 = 2;
pub const kOptCuloptFlagLine: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kOptDyFlagMsgsep: C2Rust_Unnamed_22 = 8;
pub const kOptDyFlagUhex: C2Rust_Unnamed_22 = 4;
pub const kOptDyFlagTruncate: C2Rust_Unnamed_22 = 2;
pub const kOptDyFlagLastline: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const kOptSpoFlagNoplainbuffer: C2Rust_Unnamed_23 = 2;
pub const kOptSpoFlagCamel: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const kDecorKindUIWatched: C2Rust_Unnamed_24 = 4;
pub const kDecorKindVirtLines: C2Rust_Unnamed_24 = 3;
pub const kDecorKindVirtText: C2Rust_Unnamed_24 = 2;
pub const kDecorKindSign: C2Rust_Unnamed_24 = 1;
pub const kDecorKindHighlight: C2Rust_Unnamed_24 = 0;
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
    pub data: C2Rust_Unnamed_25,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_26,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
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
    pub slots: C2Rust_Unnamed_28,
    pub ranges_i: C2Rust_Unnamed_27,
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
pub struct C2Rust_Unnamed_27 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const TERM_ATTRS_MAX: C2Rust_Unnamed_29 = 1024;
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
pub struct C2Rust_Unnamed_30 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlinevars_T {
    pub lnum: linenr_T,
    pub foldinfo: foldinfo_T,
    pub startrow: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub vcol: colnr_T,
    pub col: ::core::ffi::c_int,
    pub boguscols: ::core::ffi::c_int,
    pub old_boguscols: ::core::ffi::c_int,
    pub vcol_off_co: ::core::ffi::c_int,
    pub off: ::core::ffi::c_int,
    pub cul_attr: ::core::ffi::c_int,
    pub line_attr: ::core::ffi::c_int,
    pub line_attr_lowprio: ::core::ffi::c_int,
    pub sign_num_attr: ::core::ffi::c_int,
    pub prev_num_attr: ::core::ffi::c_int,
    pub sign_cul_attr: ::core::ffi::c_int,
    pub fromcol: ::core::ffi::c_int,
    pub tocol: ::core::ffi::c_int,
    pub vcol_sbr: colnr_T,
    pub need_showbreak: bool,
    pub char_attr: ::core::ffi::c_int,
    pub n_extra: ::core::ffi::c_int,
    pub n_attr: ::core::ffi::c_int,
    pub p_extra: *mut ::core::ffi::c_char,
    pub extra_attr: ::core::ffi::c_int,
    pub sc_extra: schar_T,
    pub sc_final: schar_T,
    pub extra_for_extmark: bool,
    pub extra: [::core::ffi::c_char; 11],
    pub diff_hlf: hlf_T,
    pub n_virt_lines: ::core::ffi::c_int,
    pub n_virt_below: ::core::ffi::c_int,
    pub filler_lines: ::core::ffi::c_int,
    pub filler_todo: ::core::ffi::c_int,
    pub sattrs: [SignTextAttrs; 9],
    pub need_lbr: bool,
    pub virt_inline: VirtText,
    pub virt_inline_i: size_t,
    pub virt_inline_hl_mode: HlMode,
    pub reset_extra_attr: bool,
    pub skip_cells: ::core::ffi::c_int,
    pub skipped_cells: ::core::ffi::c_int,
    pub color_cols: *mut ::core::ffi::c_int,
}
pub const SLF_WRAP: C2Rust_Unnamed_32 = 2;
pub const SLF_RIGHTLEFT: C2Rust_Unnamed_32 = 1;
pub const SLF_INC_VCOL: C2Rust_Unnamed_32 = 4;
pub const HL_CONCEAL: C2Rust_Unnamed_34 = 131072;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
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
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
pub type CSType = bool;
pub const kCharsizeFast: C2Rust_Unnamed_33 = 1;
pub const MODE_INSERT: C2Rust_Unnamed_31 = 16;
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
pub type smt_T = ::core::ffi::c_uint;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_31 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_31 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_31 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_31 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_31 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_31 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_31 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_31 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_31 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_31 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_31 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_31 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_31 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_31 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_31 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_31 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_31 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_31 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_31 = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const HL_INCLUDED_TOPLEVEL: C2Rust_Unnamed_34 = 524288;
pub const HL_CONCEALENDS: C2Rust_Unnamed_34 = 262144;
pub const HL_TRANS_CONT: C2Rust_Unnamed_34 = 65536;
pub const HL_MATCHCONT: C2Rust_Unnamed_34 = 32768;
pub const HL_EXTEND: C2Rust_Unnamed_34 = 16384;
pub const HL_FOLD: C2Rust_Unnamed_34 = 8192;
pub const HL_DISPLAY: C2Rust_Unnamed_34 = 4096;
pub const HL_EXCLUDENL: C2Rust_Unnamed_34 = 2048;
pub const HL_KEEPEND: C2Rust_Unnamed_34 = 1024;
pub const HL_SKIPEMPTY: C2Rust_Unnamed_34 = 512;
pub const HL_SKIPWHITE: C2Rust_Unnamed_34 = 256;
pub const HL_SKIPNL: C2Rust_Unnamed_34 = 128;
pub const HL_MATCH: C2Rust_Unnamed_34 = 64;
pub const HL_SYNC_THERE: C2Rust_Unnamed_34 = 32;
pub const HL_SYNC_HERE: C2Rust_Unnamed_34 = 16;
pub const HL_HAS_EOL: C2Rust_Unnamed_34 = 8;
pub const HL_ONELINE: C2Rust_Unnamed_34 = 4;
pub const HL_TRANSP: C2Rust_Unnamed_34 = 2;
pub const HL_CONTAINED: C2Rust_Unnamed_34 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const VIRTTEXT_EMPTY: VirtText = VirtText {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<VirtTextChunk>(),
};
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const MAX_NUMBERWIDTH: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn vim_isbreak(mut c: ::core::ffi::c_int) -> bool {
    return (*breakat_flags.ptr())[c as uint8_t as usize] != 0;
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
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VALID_CHEIGHT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
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
static extra_buf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static extra_buf_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
unsafe extern "C" fn get_extra_buf(mut size: size_t) -> *mut ::core::ffi::c_char {
    size = if size > 64 as size_t {
        size
    } else {
        64 as size_t
    };
    if extra_buf_size.get() < size {
        xfree(extra_buf.get() as *mut ::core::ffi::c_void);
        extra_buf.set(xmalloc(size) as *mut ::core::ffi::c_char);
        extra_buf_size.set(size);
    }
    return extra_buf.get();
}
unsafe extern "C" fn get_lcs_ext(mut wp: *mut win_T) -> schar_T {
    if (*wp).w_onebuf_opt.wo_wrap != 0 {
        return NUL as schar_T;
    }
    if (*wp).w_onebuf_opt.wo_wrap_flags & kOptFlagInsecure as ::core::ffi::c_int as uint32_t != 0 {
        return '>' as ::core::ffi::c_int as schar_T;
    }
    return if (*wp).w_onebuf_opt.wo_list != 0 {
        (*wp).w_p_lcs_chars.ext
    } else {
        NUL as schar_T
    };
}
unsafe extern "C" fn advance_color_col(mut wlv: *mut winlinevars_T, mut vcol: ::core::ffi::c_int) {
    if !(*wlv).color_cols.is_null() {
        while *(*wlv).color_cols >= 0 as ::core::ffi::c_int && vcol > *(*wlv).color_cols {
            (*wlv).color_cols = (*wlv).color_cols.offset(1);
        }
        if *(*wlv).color_cols < 0 as ::core::ffi::c_int {
            (*wlv).color_cols = ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
    }
}
unsafe extern "C" fn margin_columns_win(
    mut wp: *mut win_T,
    mut left_col: *mut ::core::ffi::c_int,
    mut right_col: *mut ::core::ffi::c_int,
) {
    static saved_w_virtcol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static prev_wp: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static prev_width1: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static prev_width2: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static prev_left_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static prev_right_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    let mut cur_col_off: ::core::ffi::c_int = win_col_off(wp);
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - cur_col_off;
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    if saved_w_virtcol.get() == (*wp).w_virtcol
        && prev_wp.get() == wp
        && prev_width1.get() == width1
        && prev_width2.get() == width2
    {
        *right_col = prev_right_col.get();
        *left_col = prev_left_col.get();
        return;
    }
    *left_col = 0 as ::core::ffi::c_int;
    *right_col = width1;
    if (*wp).w_virtcol >= width1 && width2 > 0 as ::core::ffi::c_int {
        *right_col = width1
            + (((*wp).w_virtcol as ::core::ffi::c_int - width1) / width2 + 1 as ::core::ffi::c_int)
                * width2;
    }
    if (*wp).w_virtcol >= width1 && width2 > 0 as ::core::ffi::c_int {
        *left_col = ((*wp).w_virtcol as ::core::ffi::c_int - width1) / width2 * width2 + width1;
    }
    prev_left_col.set(*left_col);
    prev_right_col.set(*right_col);
    prev_wp.set(wp);
    prev_width1.set(width1);
    prev_width2.set(width2);
    saved_w_virtcol.set((*wp).w_virtcol as ::core::ffi::c_int);
}
unsafe extern "C" fn line_putchar(
    mut buf: *mut buf_T,
    mut pp: *mut *const ::core::ffi::c_char,
    mut dest: *mut schar_T,
    mut maxcells: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if *dest.offset(0 as ::core::ffi::c_int as isize) != 0 as schar_T {
        } else {
            __assert_fail(
                b"dest[0] != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                235 as ::core::ffi::c_uint,
                b"int line_putchar(buf_T *, const char **, schar_T *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut p: *const ::core::ffi::c_char = *pp;
    let mut cells: ::core::ffi::c_int = utf_ptr2cells(p);
    let mut c_len: ::core::ffi::c_int = utfc_ptr2len(p);
    '_c2rust_label_0: {
        if maxcells > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"maxcells > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                240 as ::core::ffi::c_uint,
                b"int line_putchar(buf_T *, const char **, schar_T *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if cells > maxcells {
        *dest.offset(0 as ::core::ffi::c_int as isize) = ' ' as ::core::ffi::c_int as schar_T;
        return 1 as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int == TAB {
        cells = tabstop_padding(vcol as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array);
        cells = if cells < maxcells { cells } else { maxcells };
    }
    if cells < maxcells && *dest.offset(cells as isize) == 0 as schar_T {
        *dest.offset(cells as isize) = ' ' as ::core::ffi::c_int as schar_T;
    }
    if *p as ::core::ffi::c_int == TAB {
        let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while c < cells {
            *dest.offset(c as isize) = ' ' as ::core::ffi::c_int as schar_T;
            c += 1;
        }
    } else {
        let mut u8c: ::core::ffi::c_int = 0;
        *dest.offset(0 as ::core::ffi::c_int as isize) = utfc_ptr2schar(p, &raw mut u8c);
        if cells > 1 as ::core::ffi::c_int {
            *dest.offset(1 as ::core::ffi::c_int as isize) = 0 as schar_T;
        }
    }
    *pp = (*pp).offset(c_len as isize);
    return cells;
}
unsafe extern "C" fn draw_virt_text(
    mut wp: *mut win_T,
    mut buf: *mut buf_T,
    mut col_off: ::core::ffi::c_int,
    mut end_col: *mut ::core::ffi::c_int,
    mut win_row: ::core::ffi::c_int,
) {
    let state: *mut DecorState = decor_state.ptr();
    let max_col: ::core::ffi::c_int = (*wp).w_view_width;
    let mut right_pos: ::core::ffi::c_int = max_col;
    let do_eol: bool = (*state).eol_col > -1 as ::core::ffi::c_int;
    let end: ::core::ffi::c_int = (*state).current_end;
    let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*state).slots.items;
    let mut totalWidthOfEolRightAlignedVirtText: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < end {
        let mut item: *mut DecorRange =
            &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
        if (*item).start_row == (*state).row && decor_virt_pos(item) as ::core::ffi::c_int != 0 {
            let mut vt: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
            if (*item).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
                '_c2rust_label: {
                    if !(*item).data.vt.is_null() {
                    } else {
                        __assert_fail(
                            b"item->data.vt\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            293 as ::core::ffi::c_uint,
                            b"void draw_virt_text(win_T *, buf_T *, int, int *, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                vt = (*item).data.vt;
            }
            if decor_virt_pos(item) as ::core::ffi::c_int != 0
                && (*item).draw_col == -1 as ::core::ffi::c_int
            {
                let mut updated: bool = true_0 != 0;
                let mut pos: VirtTextPos = decor_virt_pos_kind(item);
                if do_eol as ::core::ffi::c_int != 0
                    && pos as ::core::ffi::c_uint
                        == kVPosEndOfLineRightAlign as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut eolOffset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if totalWidthOfEolRightAlignedVirtText == 0 as ::core::ffi::c_int {
                        let mut j: ::core::ffi::c_int = i;
                        while j < end {
                            let mut lookaheadItem: *mut DecorRange = &raw mut (*slots
                                .offset(*indices.offset(j as isize) as isize))
                            .range;
                            if !((*lookaheadItem).start_row != (*state).row
                                || !decor_virt_pos(lookaheadItem)
                                || (*lookaheadItem).draw_col != -1 as ::core::ffi::c_int)
                            {
                                let mut lookaheadVt: *mut DecorVirtText =
                                    ::core::ptr::null_mut::<DecorVirtText>();
                                if (*lookaheadItem).kind as ::core::ffi::c_int
                                    == kDecorKindVirtText as ::core::ffi::c_int
                                {
                                    '_c2rust_label_0: {
                                        if !(*lookaheadItem).data.vt.is_null() {
                                        } else {
                                            __assert_fail(
                                                b"lookaheadItem->data.vt\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/drawline.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                317 as ::core::ffi::c_uint,
                                                b"void draw_virt_text(win_T *, buf_T *, int, int *, int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    lookaheadVt = (*lookaheadItem).data.vt;
                                }
                                if decor_virt_pos_kind(lookaheadItem) as ::core::ffi::c_uint
                                    == kVPosEndOfLineRightAlign as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    totalWidthOfEolRightAlignedVirtText +=
                                        (*lookaheadVt).width + 1 as ::core::ffi::c_int;
                                }
                            }
                            j += 1;
                        }
                        totalWidthOfEolRightAlignedVirtText -= 1;
                        if totalWidthOfEolRightAlignedVirtText <= right_pos - (*state).eol_col {
                            eolOffset =
                                right_pos - totalWidthOfEolRightAlignedVirtText - (*state).eol_col;
                        }
                    }
                    (*item).draw_col = (*state).eol_col + eolOffset;
                } else if pos as ::core::ffi::c_uint
                    == kVPosRightAlign as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    right_pos -= (*vt).width;
                    (*item).draw_col = right_pos;
                } else if pos as ::core::ffi::c_uint
                    == kVPosEndOfLine as ::core::ffi::c_int as ::core::ffi::c_uint
                    && do_eol as ::core::ffi::c_int != 0
                {
                    (*item).draw_col = (*state).eol_col;
                } else if pos as ::core::ffi::c_uint
                    == kVPosWinCol as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*item).draw_col = if col_off + (*vt).col > 0 as ::core::ffi::c_int {
                        col_off + (*vt).col
                    } else {
                        0 as ::core::ffi::c_int
                    };
                } else {
                    updated = false_0 != 0;
                }
                if updated as ::core::ffi::c_int != 0
                    && ((*item).draw_col < 0 as ::core::ffi::c_int
                        || (*item).draw_col >= (*wp).w_view_width)
                {
                    (*item).draw_col = INT_MIN;
                }
            }
            if (*item).draw_col >= 0 as ::core::ffi::c_int {
                if (*item).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int {
                    let mut m: WinExtmark = WinExtmark {
                        ns_id: (*item).data.ui.ns_id as NS,
                        mark_id: (*item).data.ui.mark_id as uint64_t,
                        win_row: win_row,
                        win_col: (*item).draw_col,
                    };
                    if (*win_extmark_arr.ptr()).size == (*win_extmark_arr.ptr()).capacity {
                        (*win_extmark_arr.ptr()).capacity =
                            if (*win_extmark_arr.ptr()).capacity != 0 {
                                (*win_extmark_arr.ptr()).capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                        (*win_extmark_arr.ptr()).items = xrealloc(
                            (*win_extmark_arr.ptr()).items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<WinExtmark>()
                                .wrapping_mul((*win_extmark_arr.ptr()).capacity),
                        )
                            as *mut WinExtmark;
                    } else {
                    };
                    let c2rust_fresh4 = (*win_extmark_arr.ptr()).size;
                    (*win_extmark_arr.ptr()).size = (*win_extmark_arr.ptr()).size.wrapping_add(1);
                    *(*win_extmark_arr.ptr())
                        .items
                        .offset(c2rust_fresh4 as isize) = m;
                }
                if !vt.is_null() {
                    let mut vcol: ::core::ffi::c_int = (*item).draw_col - col_off;
                    let mut col: ::core::ffi::c_int = draw_virt_text_item(
                        buf,
                        (*item).draw_col,
                        (*vt).data.virt_text,
                        (*vt).hl_mode as HlMode,
                        max_col,
                        vcol,
                        0 as ::core::ffi::c_int,
                    );
                    if do_eol as ::core::ffi::c_int != 0
                        && ((*vt).pos as ::core::ffi::c_uint
                            == kVPosEndOfLine as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*vt).pos as ::core::ffi::c_uint
                                == kVPosEndOfLineRightAlign as ::core::ffi::c_int
                                    as ::core::ffi::c_uint)
                    {
                        (*state).eol_col = col + 1 as ::core::ffi::c_int;
                    }
                    *end_col = if *end_col > col { *end_col } else { col };
                }
                if vt.is_null()
                    || (*vt).flags as ::core::ffi::c_int & kVTRepeatLinebreak as ::core::ffi::c_int
                        == 0
                {
                    (*item).draw_col = INT_MIN;
                }
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn draw_virt_text_item(
    mut buf: *mut buf_T,
    mut col: ::core::ffi::c_int,
    mut vt: VirtText,
    mut hl_mode: HlMode,
    mut max_col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
    mut skip_cells: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut virt_str: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut virt_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut virt_pos: size_t = 0 as size_t;
    while col < max_col {
        if skip_cells >= 0 as ::core::ffi::c_int && *virt_str as ::core::ffi::c_int == NUL {
            if virt_pos >= vt.size {
                break;
            }
            virt_attr = 0 as ::core::ffi::c_int;
            virt_str = next_virt_text_chunk(vt, &raw mut virt_pos, &raw mut virt_attr);
            if virt_str.is_null() {
                break;
            }
        }
        while skip_cells > 0 as ::core::ffi::c_int && *virt_str as ::core::ffi::c_int != NUL {
            let mut c_len: ::core::ffi::c_int = utfc_ptr2len(virt_str);
            let mut cells: ::core::ffi::c_int = if *virt_str as ::core::ffi::c_int == TAB {
                tabstop_padding(vcol as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array)
            } else {
                utf_ptr2cells(virt_str)
            };
            skip_cells -= cells;
            vcol += cells;
            virt_str = virt_str.offset(c_len as isize);
        }
        let mut draw_str: *const ::core::ffi::c_char = if skip_cells < 0 as ::core::ffi::c_int {
            b" \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            virt_str
        };
        if *draw_str as ::core::ffi::c_int == NUL {
            continue;
        }
        '_c2rust_label: {
            if skip_cells <= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"skip_cells <= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    407 as ::core::ffi::c_uint,
                    b"int draw_virt_text_item(buf_T *, int, VirtText, HlMode, int, int, int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut attr: ::core::ffi::c_int = 0;
        let mut through: bool = false_0 != 0;
        if hl_mode as ::core::ffi::c_uint
            == kHlModeCombine as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            attr = hl_combine_attr(
                *(*linebuf_attr.ptr()).offset(col as isize) as ::core::ffi::c_int,
                virt_attr,
            );
        } else if hl_mode as ::core::ffi::c_uint
            == kHlModeBlend as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            through = *draw_str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int;
            attr = hl_blend_attrs(
                *(*linebuf_attr.ptr()).offset(col as isize) as ::core::ffi::c_int,
                virt_attr,
                &raw mut through,
            );
        } else {
            attr = virt_attr;
        }
        let mut dummy: [schar_T; 2] = [
            ' ' as ::core::ffi::c_int as schar_T,
            ' ' as ::core::ffi::c_int as schar_T,
        ];
        let mut maxcells: ::core::ffi::c_int = max_col - col;
        if !through && *(*linebuf_char.ptr()).offset(col as isize) == 0 as schar_T {
            '_c2rust_label_0: {
                if col > 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"col > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        422 as ::core::ffi::c_uint,
                        b"int draw_virt_text_item(buf_T *, int, VirtText, HlMode, int, int, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            *(*linebuf_char.ptr()).offset((col - 1 as ::core::ffi::c_int) as isize) =
                ' ' as ::core::ffi::c_int as schar_T;
            *(*linebuf_char.ptr()).offset(col as isize) = ' ' as ::core::ffi::c_int as schar_T;
        }
        let mut cells_0: ::core::ffi::c_int = line_putchar(
            buf,
            &raw mut draw_str,
            if through as ::core::ffi::c_int != 0 {
                &raw mut dummy as *mut schar_T
            } else {
                (*linebuf_char.ptr()).offset(col as isize)
            },
            maxcells,
            vcol,
        );
        let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while c < cells_0 {
            *(*linebuf_attr.ptr()).offset(col as isize) = attr as sattr_T;
            col += 1;
            c += 1;
        }
        if skip_cells < 0 as ::core::ffi::c_int {
            skip_cells += 1;
        } else {
            vcol += cells_0;
            virt_str = draw_str;
        }
    }
    return col;
}
unsafe extern "C" fn draw_col_buf(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut text: *const ::core::ffi::c_char,
    mut len: size_t,
    mut attr: ::core::ffi::c_int,
    mut fold_vcol: *const colnr_T,
    mut inc_vcol: bool,
) {
    let mut ptr: *const ::core::ffi::c_char = text;
    while ptr < text.offset(len as isize) && (*wlv).off < (*wp).w_view_width {
        let mut cells: ::core::ffi::c_int = line_putchar(
            (*wp).w_buffer,
            &raw mut ptr,
            (*linebuf_char.ptr()).offset((*wlv).off as isize),
            (*wp).w_view_width - (*wlv).off,
            (*wlv).off,
        );
        let mut myattr: ::core::ffi::c_int = attr;
        if inc_vcol {
            advance_color_col(wlv, (*wlv).vcol as ::core::ffi::c_int);
            if !(*wlv).color_cols.is_null() && (*wlv).vcol == *(*wlv).color_cols {
                myattr = hl_combine_attr(win_hl_attr(wp, HLF_MC as ::core::ffi::c_int), myattr);
            }
        }
        let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while c < cells {
            *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = myattr as sattr_T;
            *(*linebuf_vcol.ptr()).offset((*wlv).off as isize) =
                (if inc_vcol as ::core::ffi::c_int != 0 {
                    let c2rust_fresh6 = (*wlv).vcol;
                    (*wlv).vcol = (*wlv).vcol + 1;
                    c2rust_fresh6
                } else if !fold_vcol.is_null() {
                    let c2rust_fresh7 = fold_vcol;
                    fold_vcol = fold_vcol.offset(1);
                    *c2rust_fresh7 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                }) as colnr_T;
            (*wlv).off += 1;
            c += 1;
        }
    }
}
unsafe extern "C" fn draw_col_fill(
    mut wlv: *mut winlinevars_T,
    mut fillchar: schar_T,
    mut width: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < width {
        *(*linebuf_char.ptr()).offset((*wlv).off as isize) = fillchar;
        *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = attr as sattr_T;
        (*wlv).off += 1;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn use_cursor_line_highlight(mut wp: *mut win_T, mut lnum: linenr_T) -> bool {
    return (*wp).w_onebuf_opt.wo_cul != 0
        && lnum == (*wp).w_cursorline
        && (*wp).w_p_culopt_flags as ::core::ffi::c_int
            & kOptCuloptFlagNumber as ::core::ffi::c_int
            != 0;
}
unsafe extern "C" fn draw_foldcolumn(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    let mut fdc: ::core::ffi::c_int = compute_foldcolumn(wp, 0 as ::core::ffi::c_int);
    if fdc > 0 as ::core::ffi::c_int {
        let mut attr: ::core::ffi::c_int = win_hl_attr(
            wp,
            if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
                HLF_CLF as ::core::ffi::c_int
            } else {
                HLF_FC as ::core::ffi::c_int
            },
        );
        let mut is_virt: bool = (*wlv).filler_todo > 0 as ::core::ffi::c_int;
        fill_foldcolumn(
            wp,
            (*wlv).foldinfo,
            (*wlv).lnum,
            attr,
            fdc,
            is_virt,
            &raw mut (*wlv).off,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<schar_T>(),
        );
    }
}
#[inline]
unsafe extern "C" fn foldcolumn_sep_char(
    mut first_level: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
    mut wp: *mut win_T,
) -> schar_T {
    if first_level == 1 as ::core::ffi::c_int {
        return (*wp).w_p_fcs_chars.foldsep;
    } else if (*wp).w_p_fcs_chars.foldinner != NUL as schar_T {
        return (*wp).w_p_fcs_chars.foldinner;
    } else if first_level + i <= 9 as ::core::ffi::c_int {
        return ('0' as ::core::ffi::c_int + first_level + i) as schar_T;
    } else {
        return '>' as ::core::ffi::c_int as schar_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn fill_foldcolumn(
    mut wp: *mut win_T,
    mut foldinfo: foldinfo_T,
    mut lnum: linenr_T,
    mut attr: ::core::ffi::c_int,
    mut fdc: ::core::ffi::c_int,
    mut is_virt: bool,
    mut wlv_off: *mut ::core::ffi::c_int,
    mut out_vcol: *mut colnr_T,
    mut out_buffer: *mut schar_T,
) {
    let mut closed: bool =
        foldinfo.fi_level != 0 as ::core::ffi::c_int && foldinfo.fi_lines > 0 as linenr_T;
    let mut level: ::core::ffi::c_int = foldinfo.fi_level;
    let mut first_level: ::core::ffi::c_int = if level - fdc - closed as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        > 1 as ::core::ffi::c_int
    {
        level - fdc - closed as ::core::ffi::c_int + 1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    let mut closedcol: ::core::ffi::c_int = if fdc < level { fdc } else { level };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < fdc {
        let mut symbol: schar_T = 0 as schar_T;
        if i >= level {
            symbol = ' ' as ::core::ffi::c_int as schar_T;
        } else if i == closedcol - 1 as ::core::ffi::c_int && closed as ::core::ffi::c_int != 0 {
            symbol = (*wp).w_p_fcs_chars.foldclosed;
        } else if foldinfo.fi_lnum == lnum && first_level + i >= foldinfo.fi_low_level {
            symbol = (*wp).w_p_fcs_chars.foldopen;
        } else {
            symbol = foldcolumn_sep_char(first_level, i, wp);
        }
        if is_virt as ::core::ffi::c_int != 0
            && foldinfo.fi_level != 0 as ::core::ffi::c_int
            && foldinfo.fi_lnum == lnum
        {
            let mut outer_level: ::core::ffi::c_int =
                if foldinfo.fi_low_level - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                    foldinfo.fi_low_level - 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            let mut outer_first_level: ::core::ffi::c_int =
                if outer_level - fdc + 1 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                    outer_level - fdc + 1 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                };
            if i >= outer_level {
                symbol = ' ' as ::core::ffi::c_int as schar_T;
            } else {
                symbol = foldcolumn_sep_char(outer_first_level, i, wp);
            }
        }
        let mut vcol: ::core::ffi::c_int = if i >= level {
            -1 as ::core::ffi::c_int
        } else if i == closedcol - 1 as ::core::ffi::c_int && closed as ::core::ffi::c_int != 0 {
            -2 as ::core::ffi::c_int
        } else {
            -3 as ::core::ffi::c_int
        };
        if !out_buffer.is_null() {
            *out_vcol.offset(i as isize) = vcol as colnr_T;
            *out_buffer.offset(i as isize) = symbol;
        } else {
            *(*linebuf_vcol.ptr()).offset(*wlv_off as isize) = vcol as colnr_T;
            *(*linebuf_attr.ptr()).offset(*wlv_off as isize) = attr as sattr_T;
            let c2rust_fresh0 = *wlv_off;
            *wlv_off = *wlv_off + 1;
            *(*linebuf_char.ptr()).offset(c2rust_fresh0 as isize) = symbol;
        }
        i += 1;
    }
}
unsafe extern "C" fn draw_sign(
    mut nrcol: bool,
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut sign_idx: ::core::ffi::c_int,
) {
    let mut sattr: SignTextAttrs = (*wlv).sattrs[sign_idx as usize];
    let mut scl_attr: ::core::ffi::c_int = win_hl_attr(
        wp,
        if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
            HLF_CLS as ::core::ffi::c_int
        } else {
            HLF_SC as ::core::ffi::c_int
        },
    );
    if sattr.text[0 as ::core::ffi::c_int as usize] != 0
        && (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
        && (*wlv).filler_todo <= 0 as ::core::ffi::c_int
    {
        let mut fill: ::core::ffi::c_int = if nrcol as ::core::ffi::c_int != 0 {
            number_width(wp) + 1 as ::core::ffi::c_int
        } else {
            SIGN_WIDTH as ::core::ffi::c_int
        };
        let mut attr: ::core::ffi::c_int = if (*wlv).sign_cul_attr != 0 {
            (*wlv).sign_cul_attr
        } else if sattr.hl_id != 0 {
            syn_id2attr(sattr.hl_id)
        } else {
            0 as ::core::ffi::c_int
        };
        attr = hl_combine_attr(scl_attr, attr);
        draw_col_fill(wlv, ' ' as ::core::ffi::c_int as schar_T, fill, attr);
        let mut sign_pos: ::core::ffi::c_int =
            (*wlv).off - SIGN_WIDTH as ::core::ffi::c_int - nrcol as ::core::ffi::c_int;
        '_c2rust_label: {
            if sign_pos >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"sign_pos >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    580 as ::core::ffi::c_uint,
                    b"void draw_sign(_Bool, win_T *, winlinevars_T *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        *(*linebuf_char.ptr()).offset(sign_pos as isize) =
            sattr.text[0 as ::core::ffi::c_int as usize];
        *(*linebuf_char.ptr()).offset((sign_pos + 1 as ::core::ffi::c_int) as isize) =
            sattr.text[1 as ::core::ffi::c_int as usize];
    } else {
        '_c2rust_label_0: {
            if !nrcol {
            } else {
                __assert_fail(
                    b"!nrcol\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    584 as ::core::ffi::c_uint,
                    b"void draw_sign(_Bool, win_T *, winlinevars_T *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        draw_col_fill(
            wlv,
            ' ' as ::core::ffi::c_int as schar_T,
            SIGN_WIDTH as ::core::ffi::c_int,
            scl_attr,
        );
    };
}
#[inline]
unsafe extern "C" fn get_line_number_str(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
) {
    let mut num: linenr_T = 0;
    let mut fmt: *mut ::core::ffi::c_char =
        b"%*d \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu == 0 {
        num = lnum;
    } else {
        num = abs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_int) as linenr_T;
        if num == 0 as linenr_T && (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0 {
            num = lnum;
            fmt = b"%-*d \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    }
    snprintf(buf, buf_len, fmt, number_width(wp), num);
}
unsafe extern "C" fn use_cursor_line_nr(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) -> bool {
    return (*wp).w_onebuf_opt.wo_cul != 0
        && (*wlv).lnum == (*wp).w_cursorline
        && (*wp).w_p_culopt_flags as ::core::ffi::c_int
            & kOptCuloptFlagNumber as ::core::ffi::c_int
            != 0
        && ((*wlv).row == (*wlv).startrow + (*wlv).filler_lines
            || (*wlv).row > (*wlv).startrow + (*wlv).filler_lines
                && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                    & kOptCuloptFlagLine as ::core::ffi::c_int
                    != 0);
}
unsafe extern "C" fn get_line_number_attr(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) -> ::core::ffi::c_int {
    let mut numhl_attr: ::core::ffi::c_int = (*wlv).sign_num_attr;
    if (*wlv).n_virt_lines - (*wlv).filler_todo < (*wlv).n_virt_below {
        if (*wlv).prev_num_attr == -1 as ::core::ffi::c_int {
            decor_redraw_signs(
                wp,
                (*wp).w_buffer,
                (*wlv).lnum as ::core::ffi::c_int - 2 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<SignTextAttrs>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut (*wlv).prev_num_attr,
            );
            if (*wlv).prev_num_attr > 0 as ::core::ffi::c_int {
                (*wlv).prev_num_attr = syn_id2attr((*wlv).prev_num_attr);
            }
        }
        numhl_attr = (*wlv).prev_num_attr;
    }
    if use_cursor_line_nr(wp, wlv) {
        return hl_combine_attr(win_hl_attr(wp, HLF_CLN as ::core::ffi::c_int), numhl_attr);
    }
    if (*wp).w_onebuf_opt.wo_rnu != 0 {
        if (*wlv).lnum < (*wp).w_cursor.lnum {
            return hl_combine_attr(win_hl_attr(wp, HLF_LNA as ::core::ffi::c_int), numhl_attr);
        }
        if (*wlv).lnum > (*wp).w_cursor.lnum {
            return hl_combine_attr(win_hl_attr(wp, HLF_LNB as ::core::ffi::c_int), numhl_attr);
        }
    }
    return hl_combine_attr(win_hl_attr(wp, HLF_N as ::core::ffi::c_int), numhl_attr);
}
unsafe extern "C" fn draw_lnum_col(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    let mut has_cpo_n: bool = !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null();
    if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
        && ((*wlv).row == (*wlv).startrow + (*wlv).filler_lines || !has_cpo_n)
        && !(has_cpo_n as ::core::ffi::c_int != 0
            && (*wp).w_onebuf_opt.wo_bri == 0
            && (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wlv).lnum == (*wp).w_topline)
    {
        if (*wp).w_minscwidth == SCL_NUM
            && (*wlv).sattrs[0 as ::core::ffi::c_int as usize].text
                [0 as ::core::ffi::c_int as usize]
                != 0
            && (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
            && (*wlv).filler_todo <= 0 as ::core::ffi::c_int
        {
            draw_sign(true_0 != 0, wp, wlv, 0 as ::core::ffi::c_int);
        } else {
            let mut width: ::core::ffi::c_int = number_width(wp) + 1 as ::core::ffi::c_int;
            let mut attr: ::core::ffi::c_int = get_line_number_attr(wp, wlv);
            if (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
                && ((*wp).w_skipcol == 0 as ::core::ffi::c_int
                    || (*wlv).row > 0 as ::core::ffi::c_int
                    || (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0)
            {
                let mut buf: [::core::ffi::c_char; 32] = [0; 32];
                get_line_number_str(
                    wp,
                    (*wlv).lnum,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                );
                if (*wp).w_skipcol > 0 as ::core::ffi::c_int
                    && (*wlv).startrow == 0 as ::core::ffi::c_int
                {
                    let mut c: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
                    while *c as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                        *c = '-' as ::core::ffi::c_char;
                        c = c.offset(1);
                    }
                }
                if (*wp).w_onebuf_opt.wo_rl != 0 {
                    let mut num: *mut ::core::ffi::c_char =
                        skipwhite(&raw mut buf as *mut ::core::ffi::c_char);
                    rl_mirror_ascii(num, skiptowhite(num));
                }
                draw_col_buf(
                    wp,
                    wlv,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    width as size_t,
                    attr,
                    ::core::ptr::null::<colnr_T>(),
                    false_0 != 0,
                );
            } else {
                draw_col_fill(wlv, ' ' as ::core::ffi::c_int as schar_T, width, attr);
            }
        }
    }
}
unsafe extern "C" fn draw_statuscol(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut virtnum: ::core::ffi::c_int,
    mut col_rows: ::core::ffi::c_int,
    mut stcp: *mut statuscol_T,
) {
    let mut lnum: linenr_T = (*wlv).lnum
        - ((*wlv).n_virt_lines - (*wlv).filler_todo < (*wlv).n_virt_below) as ::core::ffi::c_int;
    let mut relnum: linenr_T = if virtnum == -(*wlv).filler_lines
        || virtnum == 0 as ::core::ffi::c_int
        || virtnum == (*wlv).n_virt_below - (*wlv).filler_lines
    {
        abs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_int) as linenr_T
    } else {
        -1 as linenr_T
    };
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if (*wp).w_statuscol_line_count != (*wp).w_nrwidth_line_count {
        (*wp).w_statuscol_line_count = (*wp).w_nrwidth_line_count;
        set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
        let mut width: ::core::ffi::c_int = build_statuscol_str(
            wp,
            (*wp).w_nrwidth_line_count,
            (*wp).w_nrwidth_line_count,
            &raw mut buf as *mut ::core::ffi::c_char,
            stcp,
        );
        if width > (*stcp).width {
            let mut addwidth: ::core::ffi::c_int = if width - (*stcp).width
                < 20 as ::core::ffi::c_int
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int
                    - (*stcp).width
            {
                width - (*stcp).width
            } else {
                20 as ::core::ffi::c_int
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int
                    - (*stcp).width
            };
            (*wp).w_nrwidth += addwidth;
            (*wp).w_nrwidth_width = (*wp).w_nrwidth;
            if col_rows > 0 as ::core::ffi::c_int {
                (*wp).w_redr_statuscol = true_0 != 0;
                return;
            }
            (*stcp).width += addwidth;
            (*wp).w_valid &= !VALID_WCOL;
        }
    }
    set_vim_var_nr(VV_VIRTNUM, virtnum as varnumber_T);
    let mut width_0: ::core::ffi::c_int = build_statuscol_str(
        wp,
        lnum,
        relnum,
        &raw mut buf as *mut ::core::ffi::c_char,
        stcp,
    );
    if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL
        || width_0 > (*stcp).width
            && (*stcp).width
                < MAX_NUMBERWIDTH
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int
    {
        if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL {
            (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
            (*wp).w_nrwidth = ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                as ::core::ffi::c_int
                * number_width(wp);
        } else {
            (*wp).w_nrwidth += if width_0 - (*stcp).width
                < 20 as ::core::ffi::c_int
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int
                    - (*stcp).width
            {
                width_0 - (*stcp).width
            } else {
                20 as ::core::ffi::c_int
                    + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                    + 9 as ::core::ffi::c_int
                    - (*stcp).width
            };
            (*wp).w_nrwidth_width = (*wp).w_nrwidth;
        }
        (*wp).w_redr_statuscol = true_0 != 0;
        return;
    }
    let mut p: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    let mut transbuf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut fold_vcol: *mut colnr_T = ::core::ptr::null_mut::<colnr_T>();
    let mut len: size_t = strlen(&raw mut buf as *mut ::core::ffi::c_char);
    let mut scl_attr: ::core::ffi::c_int = win_hl_attr(
        wp,
        if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
            HLF_CLS as ::core::ffi::c_int
        } else {
            HLF_SC as ::core::ffi::c_int
        },
    );
    let mut num_attr: ::core::ffi::c_int = get_line_number_attr(wp, wlv);
    let mut cur_attr: ::core::ffi::c_int = num_attr;
    let mut sp: *mut stl_hlrec_t = (*stcp).hlrec;
    while !(*sp).start.is_null() {
        let mut textlen: ptrdiff_t = (*sp).start.offset_from(p);
        let mut translen: size_t = transstr_buf(
            p,
            textlen as ssize_t,
            &raw mut transbuf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        draw_col_buf(
            wp,
            wlv,
            &raw mut transbuf as *mut ::core::ffi::c_char,
            translen,
            cur_attr,
            fold_vcol,
            false_0 != 0,
        );
        let mut attr: ::core::ffi::c_int = if (*sp).item as ::core::ffi::c_uint
            == STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            scl_attr
        } else if (*sp).item as ::core::ffi::c_uint
            == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            0 as ::core::ffi::c_int
        } else {
            num_attr
        };
        cur_attr = hl_combine_attr(
            attr,
            if (*sp).userhl < 0 as ::core::ffi::c_int {
                syn_id2attr(-(*sp).userhl)
            } else {
                0 as ::core::ffi::c_int
            },
        );
        fold_vcol = if (*sp).item as ::core::ffi::c_uint
            == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            &raw mut (*stcp).fold_vcol as *mut colnr_T
        } else {
            ::core::ptr::null_mut::<colnr_T>()
        };
        p = (*sp).start;
        sp = sp.offset(1);
    }
    let mut translen_0: size_t = transstr_buf(
        p,
        (&raw mut buf as *mut ::core::ffi::c_char)
            .offset(len as isize)
            .offset_from(p) as ssize_t,
        &raw mut transbuf as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        true_0 != 0,
    );
    draw_col_buf(
        wp,
        wlv,
        &raw mut transbuf as *mut ::core::ffi::c_char,
        translen_0,
        cur_attr,
        fold_vcol,
        false_0 != 0,
    );
    draw_col_fill(
        wlv,
        ' ' as ::core::ffi::c_int as schar_T,
        (*stcp).width - width_0,
        cur_attr,
    );
}
unsafe extern "C" fn handle_breakindent(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    if (*wp).w_onebuf_opt.wo_bri != 0
        && ((*wlv).row > (*wlv).startrow + (*wlv).filler_lines
            || (*wlv).need_showbreak as ::core::ffi::c_int != 0)
    {
        let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*wlv).diff_hlf as ::core::ffi::c_uint != HLF_NONE as ::core::ffi::c_uint {
            attr = win_hl_attr(wp, (*wlv).diff_hlf as ::core::ffi::c_int);
        }
        let mut num: ::core::ffi::c_int =
            get_breakindent_win(wp, ml_get_buf((*wp).w_buffer, (*wlv).lnum));
        if (*wlv).row == (*wlv).startrow {
            num -= win_col_off2(wp);
            if (*wlv).n_extra < 0 as ::core::ffi::c_int {
                num = 0 as ::core::ffi::c_int;
            }
        }
        let mut vcol_before: colnr_T = (*wlv).vcol;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num {
            *(*linebuf_char.ptr()).offset((*wlv).off as isize) =
                ' ' as ::core::ffi::c_int as schar_T;
            advance_color_col(wlv, (*wlv).vcol as ::core::ffi::c_int);
            let mut myattr: ::core::ffi::c_int = attr;
            if !(*wlv).color_cols.is_null() && (*wlv).vcol == *(*wlv).color_cols {
                myattr = hl_combine_attr(win_hl_attr(wp, HLF_MC as ::core::ffi::c_int), myattr);
            }
            *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = myattr as sattr_T;
            let c2rust_fresh5 = (*wlv).vcol;
            (*wlv).vcol = (*wlv).vcol + 1;
            *(*linebuf_vcol.ptr()).offset((*wlv).off as isize) = c2rust_fresh5;
            (*wlv).off += 1;
            i += 1;
        }
        if (*wlv).fromcol >= vcol_before && (*wlv).fromcol < (*wlv).vcol {
            (*wlv).fromcol = (*wlv).vcol as ::core::ffi::c_int;
        }
        if (*wlv).tocol == vcol_before {
            (*wlv).tocol = (*wlv).vcol as ::core::ffi::c_int;
        }
    }
    if (*wp).w_skipcol > 0 as ::core::ffi::c_int
        && (*wlv).startrow == 0 as ::core::ffi::c_int
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && (*wp).w_briopt_sbr as ::core::ffi::c_int != 0
    {
        (*wlv).need_showbreak = false_0 != 0;
    }
}
unsafe extern "C" fn handle_showbreak_and_filler(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    let mut remaining: ::core::ffi::c_int = (*wp).w_view_width - (*wlv).off;
    if (*wlv).filler_todo > (*wlv).filler_lines - (*wlv).n_virt_lines {
        draw_col_fill(
            wlv,
            ' ' as ::core::ffi::c_int as schar_T,
            remaining,
            0 as ::core::ffi::c_int,
        );
    } else if (*wlv).filler_todo > 0 as ::core::ffi::c_int {
        let mut c: schar_T = (*wp).w_p_fcs_chars.diff;
        draw_col_fill(
            wlv,
            c,
            remaining,
            win_hl_attr(wp, HLF_DED as ::core::ffi::c_int),
        );
    }
    let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
    if *sbr as ::core::ffi::c_int != NUL && (*wlv).need_showbreak as ::core::ffi::c_int != 0 {
        let mut attr: ::core::ffi::c_int = hl_combine_attr(
            (*wlv).cul_attr,
            win_hl_attr(wp, HLF_AT as ::core::ffi::c_int),
        );
        let mut vcol_before: colnr_T = (*wlv).vcol;
        draw_col_buf(
            wp,
            wlv,
            sbr,
            strlen(sbr),
            attr,
            ::core::ptr::null::<colnr_T>(),
            true_0 != 0,
        );
        (*wlv).vcol_sbr = (*wlv).vcol;
        if (*wlv).fromcol >= vcol_before && (*wlv).fromcol < (*wlv).vcol {
            (*wlv).fromcol = (*wlv).vcol as ::core::ffi::c_int;
        }
        if (*wlv).tocol == vcol_before {
            (*wlv).tocol = (*wlv).vcol as ::core::ffi::c_int;
        }
    }
    if (*wp).w_skipcol == 0 as ::core::ffi::c_int
        || (*wlv).startrow > 0 as ::core::ffi::c_int
        || (*wp).w_onebuf_opt.wo_wrap == 0
        || !(*wp).w_briopt_sbr
    {
        (*wlv).need_showbreak = false_0 != 0;
    }
}
unsafe extern "C" fn apply_cursorline_highlight(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    (*wlv).cul_attr = win_hl_attr(wp, HLF_CUL as ::core::ffi::c_int);
    let mut ae: HlAttrs = syn_attr2entry((*wlv).cul_attr);
    if ae.rgb_fg_color == -1 as RgbValue
        && ae.cterm_fg_color as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*wlv).line_attr_lowprio = (*wlv).cul_attr;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int == 0
        && bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
        && qf_current_entry(wp) == (*wlv).lnum
    {
        (*wlv).line_attr = hl_combine_attr((*wlv).cul_attr, (*wlv).line_attr);
    } else {
        (*wlv).line_attr = (*wlv).cul_attr;
    };
}
unsafe extern "C" fn set_line_attr_for_diff(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    (*wlv).line_attr = win_hl_attr(wp, (*wlv).diff_hlf as ::core::ffi::c_int);
    if (*wlv).cul_attr != 0 {
        (*wlv).line_attr = if 0 as ::core::ffi::c_int != (*wlv).line_attr_lowprio {
            hl_combine_attr(
                hl_combine_attr((*wlv).cul_attr, (*wlv).line_attr),
                hl_get_underline(),
            )
        } else {
            hl_combine_attr((*wlv).line_attr, (*wlv).cul_attr)
        };
    }
}
unsafe extern "C" fn has_more_inline_virt(mut wlv: *mut winlinevars_T, mut v: ptrdiff_t) -> bool {
    if (*wlv).virt_inline_i < (*wlv).virt_inline.size {
        return true_0 != 0;
    }
    let count: ::core::ffi::c_int = (*decor_state.ptr()).ranges_i.size as ::core::ffi::c_int;
    let cur_end: ::core::ffi::c_int = (*decor_state.ptr()).current_end;
    let fut_beg: ::core::ffi::c_int = (*decor_state.ptr()).future_begin;
    let indices: *mut ::core::ffi::c_int = (*decor_state.ptr()).ranges_i.items;
    let slots: *mut DecorRangeSlot = (*decor_state.ptr()).slots.items;
    let beg_pos: [::core::ffi::c_int; 2] = [0 as ::core::ffi::c_int, fut_beg];
    let end_pos: [::core::ffi::c_int; 2] = [cur_end, count];
    let mut pos_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while pos_i < 2 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = beg_pos[pos_i as usize];
        while i < end_pos[pos_i as usize] {
            let mut item: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
            if !((*item).start_row != (*decor_state.ptr()).row
                || (*item).kind as ::core::ffi::c_int != kDecorKindVirtText as ::core::ffi::c_int
                || (*(*item).data.vt).pos as ::core::ffi::c_uint
                    != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*(*item).data.vt).width == 0 as ::core::ffi::c_int)
            {
                if (*item).draw_col >= -1 as ::core::ffi::c_int
                    && (*item).start_col as ptrdiff_t >= v
                {
                    return true_0 != 0;
                }
            }
            i += 1;
        }
        pos_i += 1;
    }
    return false_0 != 0;
}
unsafe extern "C" fn handle_inline_virtual_text(
    mut _wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut v: ptrdiff_t,
    mut selected: bool,
) {
    while (*wlv).n_extra == 0 as ::core::ffi::c_int {
        if (*wlv).virt_inline_i >= (*wlv).virt_inline.size {
            (*wlv).virt_inline = VIRTTEXT_EMPTY;
            (*wlv).virt_inline_i = 0 as size_t;
            let mut state: *mut DecorState = decor_state.ptr();
            let end: ::core::ffi::c_int = (*state).current_end;
            let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
            let slots: *mut DecorRangeSlot = (*state).slots.items;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < end {
                let mut item: *mut DecorRange =
                    &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
                if (*item).draw_col == -3 as ::core::ffi::c_int {
                    decor_init_draw_col((*wlv).off, selected, item);
                }
                if !((*item).start_row != (*state).row
                    || (*item).kind as ::core::ffi::c_int
                        != kDecorKindVirtText as ::core::ffi::c_int
                    || (*(*item).data.vt).pos as ::core::ffi::c_uint
                        != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*(*item).data.vt).width == 0 as ::core::ffi::c_int)
                {
                    if (*item).draw_col >= -1 as ::core::ffi::c_int
                        && (*item).start_col as ptrdiff_t == v
                    {
                        (*wlv).virt_inline = (*(*item).data.vt).data.virt_text;
                        (*wlv).virt_inline_hl_mode = (*(*item).data.vt).hl_mode as HlMode;
                        (*item).draw_col = INT_MIN;
                        break;
                    }
                }
                i += 1;
            }
            if (*wlv).virt_inline.size == 0 {
                break;
            }
        } else {
            let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut text: *mut ::core::ffi::c_char = next_virt_text_chunk(
                (*wlv).virt_inline,
                &raw mut (*wlv).virt_inline_i,
                &raw mut attr,
            );
            if text.is_null() {
                continue;
            }
            (*wlv).p_extra = text;
            (*wlv).n_extra = strlen(text) as ::core::ffi::c_int;
            if (*wlv).n_extra == 0 as ::core::ffi::c_int {
                continue;
            }
            (*wlv).sc_extra = NUL as schar_T;
            (*wlv).sc_final = NUL as schar_T;
            (*wlv).extra_attr = attr;
            (*wlv).n_attr = mb_charlen(text);
            if (*wlv).skip_cells > 0 as ::core::ffi::c_int {
                let mut virt_text_width: ::core::ffi::c_int =
                    mb_string2cells((*wlv).p_extra) as ::core::ffi::c_int;
                if virt_text_width > (*wlv).skip_cells {
                    let mut skip_cells_remaining: ::core::ffi::c_int = (*wlv).skip_cells;
                    while skip_cells_remaining > 0 as ::core::ffi::c_int {
                        let mut cells: ::core::ffi::c_int = utf_ptr2cells((*wlv).p_extra);
                        if cells > skip_cells_remaining {
                            break;
                        }
                        let mut c_len: ::core::ffi::c_int = utfc_ptr2len((*wlv).p_extra);
                        skip_cells_remaining -= cells;
                        (*wlv).p_extra = (*wlv).p_extra.offset(c_len as isize);
                        (*wlv).n_extra -= c_len;
                        (*wlv).n_attr -= 1;
                    }
                    (*wlv).skipped_cells += (*wlv).skip_cells - skip_cells_remaining;
                    (*wlv).skip_cells = skip_cells_remaining;
                } else {
                    (*wlv).skip_cells -= virt_text_width;
                    (*wlv).skipped_cells += virt_text_width;
                    (*wlv).n_attr = 0 as ::core::ffi::c_int;
                    (*wlv).n_extra = 0 as ::core::ffi::c_int;
                    continue;
                }
            }
            '_c2rust_label: {
                if (*wlv).n_extra > 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"wlv->n_extra > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1017 as ::core::ffi::c_uint,
                        b"void handle_inline_virtual_text(win_T *, winlinevars_T *, ptrdiff_t, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*wlv).extra_for_extmark = true_0 != 0;
        }
    }
}
unsafe extern "C" fn win_line_start(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    (*wlv).col = 0 as ::core::ffi::c_int;
    (*wlv).off = 0 as ::core::ffi::c_int;
    (*wlv).need_lbr = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_view_width {
        *(*linebuf_char.ptr()).offset(i as isize) = ' ' as ::core::ffi::c_int as schar_T;
        *(*linebuf_attr.ptr()).offset(i as isize) = 0 as ::core::ffi::c_int as sattr_T;
        *(*linebuf_vcol.ptr()).offset(i as isize) = -1 as ::core::ffi::c_int as colnr_T;
        i += 1;
    }
}
unsafe extern "C" fn fix_for_boguscols(mut wlv: *mut winlinevars_T) {
    (*wlv).n_extra += (*wlv).vcol_off_co;
    (*wlv).vcol -= (*wlv).vcol_off_co;
    (*wlv).vcol_off_co = 0 as ::core::ffi::c_int;
    (*wlv).col -= (*wlv).boguscols;
    (*wlv).old_boguscols = (*wlv).boguscols;
    (*wlv).boguscols = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn get_rightmost_vcol(
    mut wp: *mut win_T,
    mut color_cols: *const ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*wp).w_onebuf_opt.wo_cuc != 0 {
        ret = (*wp).w_virtcol as ::core::ffi::c_int;
    }
    if !color_cols.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *color_cols.offset(i as isize) >= 0 as ::core::ffi::c_int {
            ret = if ret > *color_cols.offset(i as isize) {
                ret
            } else {
                *color_cols.offset(i as isize)
            };
            i += 1;
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn win_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut startrow: ::core::ffi::c_int,
    mut endrow: ::core::ffi::c_int,
    mut col_rows: ::core::ffi::c_int,
    mut concealed: bool,
    mut spv: *mut spellvars_T,
    mut foldinfo: foldinfo_T,
) -> ::core::ffi::c_int {
    let mut vcol_prev: colnr_T = -1 as colnr_T;
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let view_width: ::core::ffi::c_int = (*wp).w_view_width;
    let view_height: ::core::ffi::c_int = (*wp).w_view_height;
    let in_curline: bool = wp == curwin.get() && lnum == (*curwin.get()).w_cursor.lnum;
    let has_fold: bool =
        foldinfo.fi_level != 0 as ::core::ffi::c_int && foldinfo.fi_lines > 0 as linenr_T;
    let has_foldtext: bool = has_fold as ::core::ffi::c_int != 0
        && *(*wp).w_onebuf_opt.wo_fdt as ::core::ffi::c_int != NUL;
    let is_wrapped: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && !has_fold;
    let mut saved_attr2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_attr3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_attr3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fromcol_prev: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
    let mut noinvcur: bool = false_0 != 0;
    let mut lnum_in_visual_area: bool = false_0 != 0;
    let mut char_attr_pri: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut char_attr_base: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut area_highlighting: bool = false_0 != 0;
    let mut vi_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut area_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vcol_save_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut decor_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_syntax: bool = false_0 != 0;
    let mut folded_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eol_hl_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut nextline: [::core::ffi::c_char; 300] = [0; 300];
    let mut nextlinecol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut nextline_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut spell_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut word_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_checked_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut extra_check: bool = false_0 != 0;
    let mut multi_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mb_l: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut mb_c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mb_schar: schar_T = 0 as schar_T;
    let mut change_start: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
    let mut change_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut in_multispace: bool = false_0 != 0;
    let mut multispace_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_extra_next: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut extra_attr_next: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut search_attr_from_match: bool = false_0 != 0;
    let mut has_decor: bool = false_0 != 0;
    let mut saved_search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_area_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_decor_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_search_attr_from_match: bool = false_0 != 0;
    let mut win_col_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut area_active: bool = false_0 != 0;
    let mut decor_need_recheck: bool = false_0 != 0;
    let mut buf_fold: [::core::ffi::c_char; 51] = [0; 51];
    let mut fold_vt: VirtText = VIRTTEXT_EMPTY;
    let mut foldtext_free: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cul_screenline: bool = false_0 != 0;
    let mut left_curline_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut right_curline_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut match_conc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut on_last_col: bool = false_0 != 0;
    let mut syntax_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut syntax_seqnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_syntax_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut conceal_attr: ::core::ffi::c_int = win_hl_attr(wp, HLF_CONCEAL as ::core::ffi::c_int);
    let mut is_concealing: bool = false_0 != 0;
    let mut did_wcol: bool = false_0 != 0;
    '_c2rust_label: {
        if startrow < endrow {
        } else {
            __assert_fail(
                b"startrow < endrow\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1168 as ::core::ffi::c_uint,
                b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut wlv: winlinevars_T = winlinevars_T {
        lnum: lnum,
        foldinfo: foldinfo,
        startrow: startrow,
        row: startrow,
        vcol: 0,
        col: 0,
        boguscols: 0,
        old_boguscols: 0 as ::core::ffi::c_int,
        vcol_off_co: 0,
        off: 0,
        cul_attr: 0,
        line_attr: 0,
        line_attr_lowprio: 0,
        sign_num_attr: 0,
        prev_num_attr: -1 as ::core::ffi::c_int,
        sign_cul_attr: 0,
        fromcol: -10 as ::core::ffi::c_int,
        tocol: MAXCOL as ::core::ffi::c_int,
        vcol_sbr: -1 as colnr_T,
        need_showbreak: false,
        char_attr: 0,
        n_extra: 0,
        n_attr: 0,
        p_extra: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        extra_attr: 0,
        sc_extra: 0,
        sc_final: 0,
        extra_for_extmark: false,
        extra: [0; 11],
        diff_hlf: HLF_NONE,
        n_virt_lines: 0,
        n_virt_below: 0,
        filler_lines: 0,
        filler_todo: 0,
        sattrs: [SignTextAttrs {
            text: [0; 2],
            hl_id: 0,
        }; 9],
        need_lbr: false,
        virt_inline: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        virt_inline_i: 0,
        virt_inline_hl_mode: kHlModeUnknown,
        reset_extra_attr: false,
        skip_cells: 0,
        skipped_cells: 0,
        color_cols: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    };
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let draw_text: bool = !concealed && lnum != (*buf).b_ml.ml_line_count + 1 as linenr_T;
    let mut decor_provider_end_col: ::core::ffi::c_int = 0;
    let mut check_decor_providers: bool = false_0 != 0;
    if col_rows == 0 as ::core::ffi::c_int && draw_text as ::core::ffi::c_int != 0 {
        extra_check = (*wp).w_onebuf_opt.wo_lbr != 0;
        if syntax_present(wp) as ::core::ffi::c_int != 0
            && !(*(*wp).w_s).b_syn_error
            && !(*(*wp).w_s).b_syn_slow
            && !has_foldtext
        {
            let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
            did_emsg.set(false_0);
            syntax_start(wp, lnum);
            if did_emsg.get() != 0 {
                (*(*wp).w_s).b_syn_error = true_0 != 0;
            } else {
                did_emsg.set(save_did_emsg);
                if !(*(*wp).w_s).b_syn_slow {
                    has_syntax = true_0 != 0;
                    extra_check = true_0 != 0;
                }
            }
        }
        check_decor_providers = true_0 != 0;
        wlv.color_cols = if !(*(*wp).w_buffer).terminal.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_int>()
        } else {
            (*wp).w_p_cc_cols
        };
        advance_color_col(
            &raw mut wlv,
            wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co,
        );
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && (*wp).w_buffer == (*curwin.get()).w_buffer
        {
            let mut top: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
            let mut bot: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
            if ltoreq((*curwin.get()).w_cursor, VIsual.get()) {
                top = &raw mut (*curwin.get()).w_cursor;
                bot = VIsual.ptr();
            } else {
                top = VIsual.ptr();
                bot = &raw mut (*curwin.get()).w_cursor;
            }
            lnum_in_visual_area = lnum >= (*top).lnum && lnum <= (*bot).lnum;
            if VIsual_mode.get() == Ctrl_V {
                if lnum_in_visual_area {
                    wlv.fromcol = (*wp).w_old_cursor_fcol as ::core::ffi::c_int;
                    wlv.tocol = (*wp).w_old_cursor_lcol as ::core::ffi::c_int;
                }
            } else {
                if lnum > (*top).lnum && lnum <= (*bot).lnum {
                    wlv.fromcol = 0 as ::core::ffi::c_int;
                } else if lnum == (*top).lnum {
                    if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                        wlv.fromcol = 0 as ::core::ffi::c_int;
                    } else {
                        getvvcol(
                            wp,
                            top,
                            &raw mut wlv.fromcol as *mut colnr_T,
                            ::core::ptr::null_mut::<colnr_T>(),
                            ::core::ptr::null_mut::<colnr_T>(),
                        );
                        if gchar_pos(top) == NUL {
                            wlv.tocol = wlv.fromcol + 1 as ::core::ffi::c_int;
                        }
                    }
                }
                if VIsual_mode.get() != 'V' as ::core::ffi::c_int && lnum == (*bot).lnum {
                    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                        && (*bot).col == 0 as ::core::ffi::c_int
                        && (*bot).coladd == 0 as ::core::ffi::c_int
                    {
                        wlv.fromcol = -10 as ::core::ffi::c_int;
                        wlv.tocol = MAXCOL as ::core::ffi::c_int;
                    } else if (*bot).col == MAXCOL as ::core::ffi::c_int {
                        wlv.tocol = MAXCOL as ::core::ffi::c_int;
                    } else {
                        let mut pos: pos_T = *bot;
                        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                            getvvcol(
                                wp,
                                &raw mut pos,
                                &raw mut wlv.tocol as *mut colnr_T,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                            );
                        } else {
                            getvvcol(
                                wp,
                                &raw mut pos,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut wlv.tocol as *mut colnr_T,
                            );
                            wlv.tocol += 1;
                        }
                    }
                }
            }
            if !highlight_match.get()
                && in_curline as ::core::ffi::c_int != 0
                && cursor_is_block_during_visual(
                    *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                noinvcur = true_0 != 0;
            }
            if wlv.fromcol >= 0 as ::core::ffi::c_int {
                area_highlighting = true_0 != 0;
                vi_attr = win_hl_attr(wp, HLF_V as ::core::ffi::c_int);
            }
        } else if highlight_match.get() as ::core::ffi::c_int != 0
            && wp == curwin.get()
            && !has_foldtext
            && lnum >= (*curwin.get()).w_cursor.lnum
            && lnum <= (*curwin.get()).w_cursor.lnum + search_match_lines.get()
        {
            if lnum == (*curwin.get()).w_cursor.lnum {
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut wlv.fromcol as *mut colnr_T,
                    ::core::ptr::null_mut::<colnr_T>(),
                    ::core::ptr::null_mut::<colnr_T>(),
                );
            } else {
                wlv.fromcol = 0 as ::core::ffi::c_int;
            }
            if lnum == (*curwin.get()).w_cursor.lnum + search_match_lines.get() {
                let mut pos_0: pos_T = pos_T {
                    lnum: lnum,
                    col: search_match_endcol.get(),
                    coladd: 0,
                };
                getvcol(
                    curwin.get(),
                    &raw mut pos_0,
                    &raw mut wlv.tocol as *mut colnr_T,
                    ::core::ptr::null_mut::<colnr_T>(),
                    ::core::ptr::null_mut::<colnr_T>(),
                );
            }
            if wlv.fromcol == wlv.tocol && search_match_endcol.get() != 0 {
                wlv.tocol = wlv.fromcol + 1 as ::core::ffi::c_int;
            }
            area_highlighting = true_0 != 0;
            vi_attr = win_hl_attr(wp, HLF_I as ::core::ffi::c_int);
        }
    }
    let mut bg_attr: ::core::ffi::c_int = win_bg_attr(wp);
    let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    wlv.filler_lines = diff_check_with_linestatus(wp, lnum, &raw mut linestatus);
    let mut line_changes: diffline_T = diffline_S {
        changes: ::core::ptr::null_mut::<diffline_change_T>(),
        num_changes: 0,
        bufidx: 0,
        lineoff: 0,
    };
    let mut change_index: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if linestatus < 0 as ::core::ffi::c_int {
        if linestatus == -1 as ::core::ffi::c_int {
            if diff_find_change(wp, lnum, &raw mut line_changes) {
                wlv.diff_hlf = HLF_ADD;
            } else if line_changes.num_changes > 0 as ::core::ffi::c_int {
                let mut added: bool = diff_change_parse(
                    &raw mut line_changes,
                    line_changes
                        .changes
                        .offset(0 as ::core::ffi::c_int as isize),
                    &raw mut change_start,
                    &raw mut change_end,
                );
                if change_start == 0 as ::core::ffi::c_int {
                    if added {
                        wlv.diff_hlf = HLF_TXA;
                    } else {
                        wlv.diff_hlf = HLF_TXD;
                    }
                } else {
                    wlv.diff_hlf = HLF_CHD;
                }
                change_index = 0 as ::core::ffi::c_int;
            } else {
                wlv.diff_hlf = HLF_CHD;
                change_index = 0 as ::core::ffi::c_int;
            }
        } else {
            wlv.diff_hlf = HLF_ADD;
        }
        area_highlighting = true_0 != 0;
    }
    let mut virt_lines: VirtLines = VirtLines {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<virt_line>(),
    };
    wlv.n_virt_lines = decor_virt_lines(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        lnum as ::core::ffi::c_int,
        &raw mut wlv.n_virt_below,
        &raw mut virt_lines,
        true_0 != 0,
    );
    wlv.filler_lines += wlv.n_virt_lines;
    if lnum == (*wp).w_topline {
        wlv.filler_lines = (*wp).w_topfill;
        wlv.n_virt_lines = if wlv.n_virt_lines < wlv.filler_lines {
            wlv.n_virt_lines
        } else {
            wlv.filler_lines
        };
    }
    wlv.filler_todo = wlv.filler_lines;
    if (*wp).w_onebuf_opt.wo_cul != 0
        && (*wp).w_p_culopt_flags as ::core::ffi::c_int
            != kOptCuloptFlagNumber as ::core::ffi::c_int
        && lnum == (*wp).w_cursorline
        && !(wp == curwin.get() && VIsual_active.get() as ::core::ffi::c_int != 0)
    {
        cul_screenline = is_wrapped as ::core::ffi::c_int != 0
            && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagScreenline as ::core::ffi::c_int
                != 0;
        if !cul_screenline {
            apply_cursorline_highlight(wp, &raw mut wlv);
        } else {
            margin_columns_win(wp, &raw mut left_curline_col, &raw mut right_curline_col);
        }
        area_highlighting = true_0 != 0;
    }
    let mut sign_line_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    decor_redraw_signs(
        wp,
        buf,
        wlv.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        &raw mut wlv.sattrs as *mut SignTextAttrs,
        &raw mut sign_line_attr,
        &raw mut wlv.sign_cul_attr,
        &raw mut wlv.sign_num_attr,
    );
    let mut statuscol: statuscol_T = statuscol_T {
        width: 0 as ::core::ffi::c_int,
        lnum: 0,
        sign_cul_id: 0,
        draw: false,
        hlrec: ::core::ptr::null_mut::<stl_hlrec_t>(),
        foldinfo: foldinfo_T {
            fi_lnum: 0,
            fi_level: 0,
            fi_low_level: 0,
            fi_lines: 0,
        },
        fold_vcol: [0; 9],
        sattrs: ::core::ptr::null_mut::<SignTextAttrs>(),
    };
    if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL {
        statuscol.draw = true_0 != 0;
        statuscol.sattrs = &raw mut wlv.sattrs as *mut SignTextAttrs;
        statuscol.lnum = lnum;
        statuscol.foldinfo = foldinfo;
        statuscol.width = win_col_off(wp) - (wp == cmdwin_win.get()) as ::core::ffi::c_int;
        statuscol.sign_cul_id = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            wlv.sign_cul_attr
        } else {
            0 as ::core::ffi::c_int
        };
    } else if wlv.sign_cul_attr > 0 as ::core::ffi::c_int {
        wlv.sign_cul_attr = if use_cursor_line_highlight(wp, lnum) as ::core::ffi::c_int != 0 {
            syn_id2attr(wlv.sign_cul_attr)
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if wlv.sign_num_attr > 0 as ::core::ffi::c_int {
        wlv.sign_num_attr = syn_id2attr(wlv.sign_num_attr);
    }
    if sign_line_attr > 0 as ::core::ffi::c_int {
        wlv.line_attr = syn_id2attr(sign_line_attr);
    }
    if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && qf_current_entry(wp) == lnum {
        wlv.line_attr = win_hl_attr(wp, HLF_QFL as ::core::ffi::c_int);
    }
    if wlv.line_attr_lowprio != 0 || wlv.line_attr != 0 {
        area_highlighting = true_0 != 0;
    }
    let mut line_attr_save: ::core::ffi::c_int = wlv.line_attr;
    let mut line_attr_lowprio_save: ::core::ffi::c_int = wlv.line_attr_lowprio;
    if (*spv).spv_has_spell as ::core::ffi::c_int != 0
        && col_rows == 0 as ::core::ffi::c_int
        && draw_text as ::core::ffi::c_int != 0
    {
        extra_check = true_0 != 0;
        if lnum == (*spv).spv_checked_lnum {
            cur_checked_col = (*spv).spv_checked_col;
        }
        if (*spv).spv_capcol_lnum == 0 as linenr_T
            && check_need_cap(wp, lnum, 0 as colnr_T) as ::core::ffi::c_int != 0
        {
            (*spv).spv_cap_col = 0 as ::core::ffi::c_int;
        } else if lnum != (*spv).spv_capcol_lnum {
            (*spv).spv_cap_col = -1 as ::core::ffi::c_int;
        }
        (*spv).spv_checked_lnum = 0 as ::core::ffi::c_int as linenr_T;
        nextline[SPWORDLEN as usize] = NUL as ::core::ffi::c_char;
        if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
            let mut line: *mut ::core::ffi::c_char =
                ml_get_buf((*wp).w_buffer, lnum + 1 as linenr_T);
            spell_cat_line(
                (&raw mut nextline as *mut ::core::ffi::c_char).offset(SPWORDLEN as isize),
                line,
                SPWORDLEN,
            );
        }
        let mut line_0: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
        let mut ptr: *mut ::core::ffi::c_char = skipwhite(line_0);
        if *ptr as ::core::ffi::c_int == NUL {
            (*spv).spv_cap_col = 0 as ::core::ffi::c_int;
            (*spv).spv_capcol_lnum = lnum + 1 as linenr_T;
        } else if (*spv).spv_cap_col == 0 as ::core::ffi::c_int {
            (*spv).spv_cap_col = ptr.offset_from(line_0) as ::core::ffi::c_int;
        }
        if nextline[SPWORDLEN as usize] as ::core::ffi::c_int == NUL {
            nextlinecol = MAXCOL as ::core::ffi::c_int;
            nextline_idx = 0 as ::core::ffi::c_int;
        } else {
            let line_len: colnr_T = ml_get_buf_len((*wp).w_buffer, lnum);
            if line_len < SPWORDLEN {
                nextlinecol = 0 as ::core::ffi::c_int;
                memmove(
                    &raw mut nextline as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    line_0 as *const ::core::ffi::c_void,
                    line_len as size_t,
                );
                memmove(
                    (&raw mut nextline as *mut ::core::ffi::c_char).offset(line_len as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut nextline as *mut ::core::ffi::c_char)
                        .offset(150 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    strlen(
                        (&raw mut nextline as *mut ::core::ffi::c_char)
                            .offset(150 as ::core::ffi::c_int as isize),
                    )
                    .wrapping_add(1 as size_t),
                );
                nextline_idx = line_len as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            } else {
                nextlinecol = line_len as ::core::ffi::c_int - SPWORDLEN;
                memmove(
                    &raw mut nextline as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    line_0.offset(nextlinecol as isize) as *const ::core::ffi::c_void,
                    SPWORDLEN as size_t,
                );
                nextline_idx = SPWORDLEN + 1 as ::core::ffi::c_int;
            }
        }
    }
    let mut line_1: *mut ::core::ffi::c_char = (if draw_text as ::core::ffi::c_int != 0 {
        ml_get_buf((*wp).w_buffer, lnum) as *const ::core::ffi::c_char
    } else {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    let mut ptr_0: *mut ::core::ffi::c_char = line_1;
    let mut trailcol: colnr_T = MAXCOL as ::core::ffi::c_int;
    let mut leadcol: colnr_T = 0 as colnr_T;
    let mut lcs_eol_todo: bool = true_0 != 0;
    let lcs_eol: schar_T = (*wp).w_p_lcs_chars.eol;
    let mut lcs_prec_todo: schar_T = (*wp).w_p_lcs_chars.prec;
    if (*wp).w_onebuf_opt.wo_list != 0 && !has_foldtext && draw_text as ::core::ffi::c_int != 0 {
        if (*wp).w_p_lcs_chars.space != 0
            || !(*wp).w_p_lcs_chars.multispace.is_null()
            || !(*wp).w_p_lcs_chars.leadmultispace.is_null()
            || (*wp).w_p_lcs_chars.trail != 0
            || (*wp).w_p_lcs_chars.lead != 0
            || (*wp).w_p_lcs_chars.nbsp != 0
        {
            extra_check = true_0 != 0;
        }
        if (*wp).w_p_lcs_chars.trail != 0 {
            trailcol = ml_get_buf_len((*wp).w_buffer, lnum);
            while trailcol > 0 as ::core::ffi::c_int
                && ascii_iswhite(
                    *ptr_0
                        .offset((trailcol as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                trailcol -= 1;
            }
            trailcol += ptr_0.offset_from(line_1) as colnr_T;
        }
        if (*wp).w_p_lcs_chars.lead != 0
            || !(*wp).w_p_lcs_chars.leadmultispace.is_null()
            || (*wp).w_p_lcs_chars.leadtab1 != NUL as schar_T
        {
            leadcol = 0 as ::core::ffi::c_int as colnr_T;
            while ascii_iswhite(*ptr_0.offset(leadcol as isize) as ::core::ffi::c_int) {
                leadcol += 1;
            }
            if *ptr_0.offset(leadcol as isize) as ::core::ffi::c_int == NUL {
                leadcol = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                leadcol += (ptr_0.offset_from(line_1) + 1 as isize) as colnr_T;
            }
        }
    }
    let start_vcol: ::core::ffi::c_int = if (*wp).w_onebuf_opt.wo_wrap != 0 {
        if startrow == 0 as ::core::ffi::c_int {
            (*wp).w_skipcol as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }
    } else {
        (*wp).w_leftcol as ::core::ffi::c_int
    };
    if has_foldtext {
        wlv.vcol = start_vcol as colnr_T;
    } else if start_vcol > 0 as ::core::ffi::c_int && col_rows == 0 as ::core::ffi::c_int {
        let mut prev_ptr: *mut ::core::ffi::c_char = ptr_0;
        let mut cs: CharSize = CharSize {
            width: 0 as ::core::ffi::c_int,
            head: 0,
        };
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
                s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1],
        };
        let mut cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line_1);
        csarg.max_head_vcol = start_vcol;
        let mut vcol: ::core::ffi::c_int = wlv.vcol as ::core::ffi::c_int;
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(ptr_0);
        while vcol < start_vcol {
            cs = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg);
            vcol += cs.width;
            prev_ptr = ci.ptr;
            if *prev_ptr as ::core::ffi::c_int == NUL {
                break;
            }
            ci = utfc_next(ci);
            if (*wp).w_onebuf_opt.wo_list != 0 {
                in_multispace = *prev_ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                    && (*ci.ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                        || prev_ptr > line_1
                            && *prev_ptr.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int);
                if !in_multispace {
                    multispace_pos = 0 as ::core::ffi::c_int;
                } else if ci.ptr >= line_1.offset(leadcol as isize)
                    && !(*wp).w_p_lcs_chars.multispace.is_null()
                {
                    multispace_pos += 1;
                    if *(*wp)
                        .w_p_lcs_chars
                        .multispace
                        .offset(multispace_pos as isize)
                        == NUL as schar_T
                    {
                        multispace_pos = 0 as ::core::ffi::c_int;
                    }
                } else if ci.ptr < line_1.offset(leadcol as isize)
                    && !(*wp).w_p_lcs_chars.leadmultispace.is_null()
                {
                    multispace_pos += 1;
                    if *(*wp)
                        .w_p_lcs_chars
                        .leadmultispace
                        .offset(multispace_pos as isize)
                        == NUL as schar_T
                    {
                        multispace_pos = 0 as ::core::ffi::c_int;
                    }
                }
            }
        }
        wlv.vcol = vcol as colnr_T;
        ptr_0 = ci.ptr;
        let mut charsize: ::core::ffi::c_int = cs.width;
        let mut head: ::core::ffi::c_int = cs.head;
        if wlv.vcol < start_vcol
            && ((*wp).w_onebuf_opt.wo_cuc != 0
                || !wlv.color_cols.is_null()
                || virtual_active(wp) as ::core::ffi::c_int != 0
                || VIsual_active.get() as ::core::ffi::c_int != 0
                    && (*wp).w_buffer == (*curwin.get()).w_buffer
                || has_fold as ::core::ffi::c_int != 0)
        {
            wlv.vcol = start_vcol as colnr_T;
        }
        if wlv.vcol > start_vcol {
            wlv.vcol -= charsize;
            ptr_0 = prev_ptr;
        }
        if start_vcol > wlv.vcol {
            wlv.skip_cells = start_vcol - wlv.vcol as ::core::ffi::c_int - head;
        }
        if wlv.tocol <= wlv.vcol {
            wlv.fromcol = 0 as ::core::ffi::c_int;
        } else if wlv.fromcol >= 0 as ::core::ffi::c_int && wlv.fromcol < wlv.vcol {
            wlv.fromcol = wlv.vcol as ::core::ffi::c_int;
        }
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            wlv.need_showbreak = true_0 != 0;
        }
        if (*spv).spv_has_spell {
            let mut linecol: colnr_T = ptr_0.offset_from(line_1) as colnr_T;
            let mut spell_hlf: hlf_T = HLF_COUNT;
            let mut pos_1: pos_T = (*wp).w_cursor;
            (*wp).w_cursor.lnum = lnum;
            (*wp).w_cursor.col = linecol;
            let mut len: size_t = spell_move_to(
                wp,
                FORWARD as ::core::ffi::c_int,
                SMT_ALL,
                true_0 != 0,
                &raw mut spell_hlf,
            );
            line_1 = ml_get_buf((*wp).w_buffer, lnum);
            ptr_0 = line_1.offset(linecol as isize);
            if len == 0 as size_t || (*wp).w_cursor.col > linecol {
                spell_hlf = HLF_COUNT;
                word_end = (spell_to_word_end(ptr_0, wp).offset_from(line_1) + 1 as isize)
                    as ::core::ffi::c_int;
            } else {
                '_c2rust_label_0: {
                    if len <= 2147483647 as ::core::ffi::c_int as size_t {
                    } else {
                        __assert_fail(
                            b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/drawline.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1617 as ::core::ffi::c_uint,
                            b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                word_end = (*wp).w_cursor.col as ::core::ffi::c_int
                    + len as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int;
                if spell_hlf as ::core::ffi::c_uint
                    != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    spell_attr = (*highlight_attr.ptr())[spell_hlf as usize];
                }
            }
            (*wp).w_cursor = pos_1;
            if has_syntax {
                syntax_start(wp, lnum);
            }
        }
    }
    if check_decor_providers {
        let col: ::core::ffi::c_int = ptr_0.offset_from(line_1) as ::core::ffi::c_int;
        decor_provider_end_col = decor_providers_setup(
            endrow - startrow,
            start_vcol == 0 as ::core::ffi::c_int,
            lnum,
            col as colnr_T,
            wp,
        );
        line_1 = ml_get_buf((*wp).w_buffer, lnum);
        ptr_0 = line_1.offset(col as isize);
    }
    decor_redraw_line(
        wp,
        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        decor_state.ptr(),
    );
    if !has_decor
        && decor_has_more_decorations(
            decor_state.ptr(),
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
    {
        has_decor = true_0 != 0;
        extra_check = true_0 != 0;
    }
    if wlv.fromcol >= 0 as ::core::ffi::c_int {
        if noinvcur {
            if wlv.fromcol == (*wp).w_virtcol {
                fromcol_prev = wlv.fromcol;
                wlv.fromcol = -1 as ::core::ffi::c_int;
            } else if wlv.fromcol < (*wp).w_virtcol {
                fromcol_prev = (*wp).w_virtcol as ::core::ffi::c_int;
            }
        }
        if wlv.fromcol >= wlv.tocol {
            wlv.fromcol = -1 as ::core::ffi::c_int;
        }
    }
    if col_rows == 0 as ::core::ffi::c_int && draw_text as ::core::ffi::c_int != 0 && !has_foldtext
    {
        let v: ::core::ffi::c_int = ptr_0.offset_from(line_1) as ::core::ffi::c_int;
        area_highlighting = area_highlighting as ::core::ffi::c_int
            | prepare_search_hl_line(
                wp,
                lnum,
                v as colnr_T,
                &raw mut line_1,
                screen_search_hl.ptr(),
                &raw mut search_attr,
                &raw mut search_attr_from_match,
            ) as ::core::ffi::c_int
            != 0;
        ptr_0 = line_1.offset(v as isize);
    }
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        && ins_compl_win_active(wp) as ::core::ffi::c_int != 0
        && (in_curline as ::core::ffi::c_int != 0
            || ins_compl_lnum_in_range(lnum) as ::core::ffi::c_int != 0)
    {
        area_highlighting = true_0 != 0;
    }
    win_line_start(wp, &raw mut wlv);
    let mut draw_cols: bool = true_0 != 0;
    let mut leftcols_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut term_attrs: [::core::ffi::c_int; 1024] = [0 as ::core::ffi::c_int; 1024];
    if !(*(*wp).w_buffer).terminal.is_null() {
        terminal_get_line_attributes(
            (*(*wp).w_buffer).terminal,
            wp,
            lnum as ::core::ffi::c_int,
            &raw mut term_attrs as *mut ::core::ffi::c_int,
        );
        extra_check = true_0 != 0;
    }
    let may_have_inline_virt: bool =
        !has_foldtext && buf_meta_total((*wp).w_buffer, kMTMetaInline) > 0 as uint32_t;
    let mut virt_line_index: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut virt_line_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut draw_folded: bool = false;
    let mut extmark_attr: ::core::ffi::c_int = 0;
    let mut lcs_ext: schar_T = 0;
    's_5143: loop {
        let mut has_match_conc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut decor_conceal: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut did_decrement_ptr: bool = false_0 != 0;
        if check_decor_providers as ::core::ffi::c_int != 0
            && ptr_0.offset_from(line_1) as ::core::ffi::c_int >= decor_provider_end_col
        {
            let col_0: ::core::ffi::c_int = ptr_0.offset_from(line_1) as ::core::ffi::c_int;
            decor_provider_end_col = invoke_range_next(
                wp,
                lnum as ::core::ffi::c_int,
                col_0 as colnr_T,
                100 as colnr_T,
            );
            line_1 = ml_get_buf((*wp).w_buffer, lnum);
            ptr_0 = line_1.offset(col_0 as isize);
            if !has_decor
                && decor_has_more_decorations(
                    decor_state.ptr(),
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                has_decor = true_0 != 0;
                extra_check = true_0 != 0;
            }
        }
        '_end_check: {
            if draw_cols {
                if cul_screenline {
                    wlv.cul_attr = 0 as ::core::ffi::c_int;
                    wlv.line_attr = line_attr_save;
                    wlv.line_attr_lowprio = line_attr_lowprio_save;
                }
                '_c2rust_label_1: {
                    if wlv.off == 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"wlv.off == 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/drawline.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1726 as ::core::ffi::c_uint,
                            b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if wp == cmdwin_win.get() {
                    draw_col_fill(
                        &raw mut wlv,
                        cmdwin_type.get() as schar_T,
                        1 as ::core::ffi::c_int,
                        win_hl_attr(wp, HLF_AT as ::core::ffi::c_int),
                    );
                }
                if wlv.filler_todo > 0 as ::core::ffi::c_int {
                    let mut index: ::core::ffi::c_int =
                        wlv.filler_todo - (wlv.filler_lines - wlv.n_virt_lines);
                    if index > 0 as ::core::ffi::c_int {
                        virt_line_index = virt_lines.size as ::core::ffi::c_int - index;
                        '_c2rust_label_2: {
                            if virt_line_index >= 0 as ::core::ffi::c_int {
                            } else {
                                __assert_fail(
                                    b"virt_line_index >= 0\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    1737 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        virt_line_flags =
                            (*virt_lines.items.offset(virt_line_index as isize)).flags;
                    }
                }
                if !(virt_line_index >= 0 as ::core::ffi::c_int
                    && virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0)
                {
                    if statuscol.draw {
                        let v_0: ::core::ffi::c_int =
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                        draw_statuscol(
                            wp,
                            &raw mut wlv,
                            wlv.row - startrow - wlv.filler_lines,
                            col_rows,
                            &raw mut statuscol,
                        );
                        if (*wp).w_redr_statuscol {
                            break 's_5143;
                        }
                        if draw_text {
                            line_1 = ml_get_buf((*wp).w_buffer, lnum);
                            ptr_0 = line_1.offset(v_0 as isize);
                        }
                    } else {
                        draw_foldcolumn(wp, &raw mut wlv);
                        let mut sign_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while sign_idx < (*wp).w_scwidth {
                            draw_sign(false_0 != 0, wp, &raw mut wlv, sign_idx);
                            sign_idx += 1;
                        }
                        draw_lnum_col(wp, &raw mut wlv);
                    }
                }
                win_col_offset = wlv.off;
                if col_rows > 0 as ::core::ffi::c_int {
                    wlv_put_linebuf(
                        wp,
                        &raw mut wlv,
                        if wlv.off < view_width {
                            wlv.off
                        } else {
                            view_width
                        },
                        false_0 != 0,
                        bg_attr,
                        0 as ::core::ffi::c_int,
                    );
                    if !(wlv.row + 1 as ::core::ffi::c_int - wlv.startrow < col_rows
                        && (statuscol.draw as ::core::ffi::c_int != 0
                            || win_hl_attr(wp, HLF_LNA as ::core::ffi::c_int)
                                != win_hl_attr(wp, HLF_N as ::core::ffi::c_int)
                            || win_hl_attr(wp, HLF_LNB as ::core::ffi::c_int)
                                != win_hl_attr(wp, HLF_N as ::core::ffi::c_int))
                        || wlv.filler_todo > 0 as ::core::ffi::c_int)
                    {
                        break 's_5143;
                    }
                    wlv.row += 1;
                    if wlv.row == endrow {
                        break 's_5143;
                    }
                    wlv.filler_todo -= 1;
                    virt_line_index = -1 as ::core::ffi::c_int;
                    if wlv.filler_todo == 0 as ::core::ffi::c_int
                        && ((*wp).w_botfill as ::core::ffi::c_int != 0 || !draw_text)
                    {
                        break 's_5143;
                    }
                    wlv.col = 0 as ::core::ffi::c_int;
                    wlv.off = 0 as ::core::ffi::c_int;
                    continue 's_5143;
                } else {
                    if !(*wp).w_briopt_sbr {
                        handle_breakindent(wp, &raw mut wlv);
                    }
                    handle_showbreak_and_filler(wp, &raw mut wlv);
                    if (*wp).w_briopt_sbr {
                        handle_breakindent(wp, &raw mut wlv);
                    }
                    wlv.col = wlv.off;
                    draw_cols = false_0 != 0;
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        leftcols_width = wlv.off;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.row == startrow + wlv.filler_lines
                    {
                        decor_redraw_col(
                            wp,
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                            wlv.off,
                            true_0 != 0,
                            decor_state.ptr(),
                            decor_provider_end_col - 1 as ::core::ffi::c_int,
                        );
                    }
                    if wlv.col >= view_width {
                        wlv.off = view_width;
                        wlv.col = wlv.off;
                        break '_end_check;
                    }
                }
            }
            if cul_screenline as ::core::ffi::c_int != 0
                && wlv.filler_todo <= 0 as ::core::ffi::c_int
                && wlv.vcol >= left_curline_col
                && wlv.vcol < right_curline_col
            {
                apply_cursorline_highlight(wp, &raw mut wlv);
            }
            if dollar_vcol.get() >= 0 as ::core::ffi::c_int
                && in_curline as ::core::ffi::c_int != 0
                && wlv.vcol >= (*wp).w_virtcol
            {
                draw_virt_text(wp, buf, win_col_offset, &raw mut wlv.col, wlv.row);
                wlv_put_linebuf(
                    wp,
                    &raw mut wlv,
                    wlv.col,
                    false_0 != 0,
                    bg_attr,
                    0 as ::core::ffi::c_int,
                );
                if (*wp).w_onebuf_opt.wo_cuc != 0 {
                    wlv.row = (*wp).w_cline_row + (*wp).w_cline_height;
                } else {
                    wlv.row = view_height;
                }
                break 's_5143;
            } else {
                draw_folded =
                    has_fold as ::core::ffi::c_int != 0 && wlv.row == startrow + wlv.filler_lines;
                if draw_folded as ::core::ffi::c_int != 0 && wlv.n_extra == 0 as ::core::ffi::c_int
                {
                    folded_attr = win_hl_attr(wp, HLF_FL as ::core::ffi::c_int);
                    wlv.char_attr = folded_attr;
                    decor_attr = 0 as ::core::ffi::c_int;
                }
                extmark_attr = 0 as ::core::ffi::c_int;
                if wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && (area_highlighting as ::core::ffi::c_int != 0
                        || (*spv).spv_has_spell as ::core::ffi::c_int != 0
                        || extra_check as ::core::ffi::c_int != 0)
                {
                    if wlv.n_extra == 0 as ::core::ffi::c_int || !wlv.extra_for_extmark {
                        wlv.reset_extra_attr = false_0 != 0;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.n_extra == 0 as ::core::ffi::c_int
                    {
                        if wlv.vcol == wlv.fromcol
                            || wlv.vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                                == wlv.fromcol
                                && (wlv.n_extra == 0 as ::core::ffi::c_int
                                    && utf_ptr2cells(ptr_0) > 1 as ::core::ffi::c_int)
                            || vcol_prev == fromcol_prev
                                && vcol_prev < wlv.vcol
                                && wlv.vcol < wlv.tocol
                        {
                            area_active = true_0 != 0;
                        } else if area_active as ::core::ffi::c_int != 0
                            && (wlv.vcol == wlv.tocol
                                || noinvcur as ::core::ffi::c_int != 0
                                    && wlv.vcol == (*wp).w_virtcol)
                        {
                            area_active = false_0 != 0;
                        }
                        let mut selected: bool = area_active as ::core::ffi::c_int != 0
                            || area_highlighting as ::core::ffi::c_int != 0
                                && noinvcur as ::core::ffi::c_int != 0
                                && wlv.vcol == (*wp).w_virtcol;
                        if decor_need_recheck {
                            if !may_have_inline_virt {
                                decor_recheck_draw_col(wlv.off, selected, decor_state.ptr());
                            }
                            decor_need_recheck = false_0 != 0;
                        }
                        extmark_attr = decor_redraw_col(
                            wp,
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int,
                            if may_have_inline_virt as ::core::ffi::c_int != 0 {
                                -3 as ::core::ffi::c_int
                            } else {
                                wlv.off
                            },
                            selected,
                            decor_state.ptr(),
                            decor_provider_end_col - 1 as ::core::ffi::c_int,
                        );
                        if may_have_inline_virt {
                            handle_inline_virtual_text(
                                wp,
                                &raw mut wlv,
                                ptr_0.offset_from(line_1),
                                selected,
                            );
                            if wlv.n_extra > 0 as ::core::ffi::c_int
                                && wlv.virt_inline_hl_mode as ::core::ffi::c_uint
                                    <= kHlModeReplace as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                saved_search_attr = search_attr;
                                saved_area_attr = area_attr;
                                saved_decor_attr = decor_attr;
                                saved_search_attr_from_match = search_attr_from_match;
                                search_attr = 0 as ::core::ffi::c_int;
                                area_attr = 0 as ::core::ffi::c_int;
                                decor_attr = 0 as ::core::ffi::c_int;
                                search_attr_from_match = false_0 != 0;
                            }
                        }
                    }
                    let mut area_attr_p: *mut ::core::ffi::c_int =
                        if wlv.extra_for_extmark as ::core::ffi::c_int != 0
                            && wlv.virt_inline_hl_mode as ::core::ffi::c_uint
                                <= kHlModeReplace as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            &raw mut saved_area_attr
                        } else {
                            &raw mut area_attr
                        };
                    if wlv.vcol == wlv.fromcol
                        || wlv.vcol as ::core::ffi::c_int + 1 as ::core::ffi::c_int == wlv.fromcol
                            && (wlv.n_extra == 0 as ::core::ffi::c_int
                                && utf_ptr2cells(ptr_0) > 1 as ::core::ffi::c_int
                                || wlv.n_extra > 0 as ::core::ffi::c_int
                                    && !wlv.p_extra.is_null()
                                    && utf_ptr2cells(wlv.p_extra) > 1 as ::core::ffi::c_int)
                        || vcol_prev == fromcol_prev && vcol_prev < wlv.vcol && wlv.vcol < wlv.tocol
                    {
                        *area_attr_p = vi_attr;
                        area_active = true_0 != 0;
                    } else if *area_attr_p != 0 as ::core::ffi::c_int
                        && (wlv.vcol == wlv.tocol
                            || noinvcur as ::core::ffi::c_int != 0 && wlv.vcol == (*wp).w_virtcol)
                    {
                        *area_attr_p = 0 as ::core::ffi::c_int;
                        area_active = false_0 != 0;
                    }
                    if !has_foldtext && wlv.n_extra == 0 as ::core::ffi::c_int {
                        let v_1: ::core::ffi::c_int =
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                        search_attr = update_search_hl(
                            wp,
                            lnum,
                            v_1 as colnr_T,
                            &raw mut line_1,
                            screen_search_hl.ptr(),
                            &raw mut has_match_conc,
                            &raw mut match_conc,
                            lcs_eol_todo,
                            &raw mut on_last_col,
                            &raw mut search_attr_from_match,
                        );
                        ptr_0 = line_1.offset(v_1 as isize);
                        if *ptr_0 as ::core::ffi::c_int == NUL {
                            has_match_conc = 0 as ::core::ffi::c_int;
                        }
                        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
                            && ins_compl_win_active(wp) as ::core::ffi::c_int != 0
                            && (in_curline as ::core::ffi::c_int != 0
                                || ins_compl_lnum_in_range(lnum) as ::core::ffi::c_int != 0)
                        {
                            let mut ins_match_attr: ::core::ffi::c_int = ins_compl_col_range_attr(
                                lnum,
                                ptr_0.offset_from(line_1) as ::core::ffi::c_int,
                            );
                            if ins_match_attr > 0 as ::core::ffi::c_int {
                                search_attr = hl_combine_attr(search_attr, ins_match_attr);
                            }
                        }
                    }
                    if wlv.diff_hlf as ::core::ffi::c_uint != HLF_NONE as ::core::ffi::c_uint {
                        if line_changes.num_changes > 0 as ::core::ffi::c_int
                            && change_index >= 0 as ::core::ffi::c_int
                            && change_index < line_changes.num_changes - 1 as ::core::ffi::c_int
                        {
                            if ptr_0.offset_from(line_1)
                                >= (*line_changes
                                    .changes
                                    .offset((change_index + 1 as ::core::ffi::c_int) as isize))
                                .dc_start[line_changes.bufidx as usize]
                                    as isize
                            {
                                change_index += 1 as ::core::ffi::c_int;
                            }
                        }
                        let mut added_0: bool = false_0 != 0;
                        if line_changes.num_changes > 0 as ::core::ffi::c_int
                            && change_index >= 0 as ::core::ffi::c_int
                            && change_index < line_changes.num_changes
                        {
                            added_0 = diff_change_parse(
                                &raw mut line_changes,
                                line_changes.changes.offset(change_index as isize),
                                &raw mut change_start,
                                &raw mut change_end,
                            );
                        }
                        if wlv.diff_hlf as ::core::ffi::c_uint
                            == HLF_CHD as ::core::ffi::c_int as ::core::ffi::c_uint
                            && ptr_0.offset_from(line_1) >= change_start as isize
                            && wlv.n_extra == 0 as ::core::ffi::c_int
                        {
                            wlv.diff_hlf = (if added_0 as ::core::ffi::c_int != 0 {
                                HLF_TXA as ::core::ffi::c_int
                            } else {
                                HLF_TXD as ::core::ffi::c_int
                            }) as hlf_T;
                        }
                        if (wlv.diff_hlf as ::core::ffi::c_uint
                            == HLF_TXD as ::core::ffi::c_int as ::core::ffi::c_uint
                            || wlv.diff_hlf as ::core::ffi::c_uint
                                == HLF_TXA as ::core::ffi::c_int as ::core::ffi::c_uint)
                            && (ptr_0.offset_from(line_1) >= change_end as isize
                                && wlv.n_extra == 0 as ::core::ffi::c_int
                                || wlv.n_extra > 0 as ::core::ffi::c_int
                                    && wlv.extra_for_extmark as ::core::ffi::c_int != 0)
                        {
                            wlv.diff_hlf = HLF_CHD;
                        }
                        set_line_attr_for_diff(wp, &raw mut wlv);
                    }
                    if area_attr != 0 as ::core::ffi::c_int {
                        char_attr_pri = hl_combine_attr(wlv.line_attr, area_attr);
                        if !highlight_match.get() {
                            char_attr_pri = hl_combine_attr(search_attr, char_attr_pri);
                        }
                    } else if search_attr != 0 as ::core::ffi::c_int {
                        char_attr_pri = hl_combine_attr(wlv.line_attr, search_attr);
                    } else if wlv.line_attr != 0 as ::core::ffi::c_int
                        && (wlv.fromcol == -10 as ::core::ffi::c_int
                            && wlv.tocol == MAXCOL as ::core::ffi::c_int
                            || wlv.vcol < wlv.fromcol
                            || vcol_prev < fromcol_prev
                            || wlv.vcol >= wlv.tocol)
                    {
                        char_attr_pri = wlv.line_attr;
                    } else {
                        char_attr_pri = 0 as ::core::ffi::c_int;
                    }
                    char_attr_base = hl_combine_attr(folded_attr, decor_attr);
                    wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && has_foldtext as ::core::ffi::c_int != 0
                    && wlv.n_extra == 0 as ::core::ffi::c_int
                    && wlv.col == win_col_offset
                {
                    let v_2: ::core::ffi::c_int = ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                    let mut lnume: linenr_T = lnum + foldinfo.fi_lines - 1 as linenr_T;
                    memset(
                        &raw mut buf_fold as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        ' ' as ::core::ffi::c_int,
                        FOLD_TEXT_LEN as ::core::ffi::c_int as size_t,
                    );
                    wlv.p_extra = get_foldtext(
                        wp,
                        lnum,
                        lnume,
                        foldinfo,
                        &raw mut buf_fold as *mut ::core::ffi::c_char,
                        &raw mut fold_vt,
                    );
                    wlv.n_extra = strlen(wlv.p_extra) as ::core::ffi::c_int;
                    if wlv.p_extra != &raw mut buf_fold as *mut ::core::ffi::c_char {
                        '_c2rust_label_3: {
                            if foldtext_free.is_null() {
                            } else {
                                __assert_fail(
                                    b"foldtext_free == NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2012 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        foldtext_free = wlv.p_extra;
                    }
                    wlv.sc_extra = NUL as schar_T;
                    wlv.sc_final = NUL as schar_T;
                    *wlv.p_extra.offset(wlv.n_extra as isize) = NUL as ::core::ffi::c_char;
                    line_1 = ml_get_buf((*wp).w_buffer, lnum);
                    ptr_0 = line_1.offset(v_2 as isize);
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && wlv.n_extra == 0 as ::core::ffi::c_int
                    && wlv.col < view_width
                    && (has_foldtext as ::core::ffi::c_int != 0
                        || *ptr_0 as ::core::ffi::c_int == NUL
                            && ((*wp).w_onebuf_opt.wo_list == 0
                                || !lcs_eol_todo
                                || lcs_eol == NUL as schar_T))
                {
                    wlv.sc_extra = (*wp).w_p_fcs_chars.fold;
                    wlv.sc_final = NUL as schar_T;
                    wlv.n_extra = view_width - wlv.col;
                    search_attr = 0 as ::core::ffi::c_int;
                }
                if draw_folded as ::core::ffi::c_int != 0
                    && wlv.n_extra != 0 as ::core::ffi::c_int
                    && wlv.col >= view_width
                {
                    wlv.n_extra = 0 as ::core::ffi::c_int;
                }
                if wlv.n_extra > 0 as ::core::ffi::c_int {
                    if wlv.sc_extra != NUL as schar_T
                        || wlv.n_extra == 1 as ::core::ffi::c_int && wlv.sc_final != NUL as schar_T
                    {
                        mb_schar = if wlv.n_extra == 1 as ::core::ffi::c_int
                            && wlv.sc_final != NUL as schar_T
                        {
                            wlv.sc_final
                        } else {
                            wlv.sc_extra
                        };
                        mb_c = schar_get_first_codepoint(mb_schar);
                        wlv.n_extra -= 1;
                    } else {
                        '_c2rust_label_4: {
                            if !wlv.p_extra.is_null() {
                            } else {
                                __assert_fail(
                                    b"wlv.p_extra != NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/drawline.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2055 as ::core::ffi::c_uint,
                                    b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        mb_l = utfc_ptr2len(wlv.p_extra);
                        mb_schar = utfc_ptr2schar(wlv.p_extra, &raw mut mb_c);
                        if mb_l > wlv.n_extra || mb_l == 0 as ::core::ffi::c_int {
                            mb_l = 1 as ::core::ffi::c_int;
                        }
                        if wlv.col >= view_width - 1 as ::core::ffi::c_int
                            && schar_cells(mb_schar) == 2 as ::core::ffi::c_int
                        {
                            mb_c = '>' as ::core::ffi::c_int;
                            mb_l = 1 as ::core::ffi::c_int;
                            mb_schar = mb_c as schar_T;
                            multi_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                            if wlv.cul_attr != 0 {
                                multi_attr = if 0 as ::core::ffi::c_int != wlv.line_attr_lowprio {
                                    hl_combine_attr(wlv.cul_attr, multi_attr)
                                } else {
                                    hl_combine_attr(multi_attr, wlv.cul_attr)
                                };
                            }
                        } else {
                            wlv.n_extra -= mb_l;
                            wlv.p_extra = wlv.p_extra.offset(mb_l as isize);
                        }
                        if wlv.filler_todo <= 0 as ::core::ffi::c_int
                            && wlv.skip_cells > 0 as ::core::ffi::c_int
                            && mb_l > 1 as ::core::ffi::c_int
                        {
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                n_extra_next = wlv.n_extra;
                                extra_attr_next = wlv.extra_attr;
                            }
                            wlv.n_extra = 1 as ::core::ffi::c_int;
                            wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            mb_c = ' ' as ::core::ffi::c_int;
                            mb_l = 1 as ::core::ffi::c_int;
                            wlv.n_attr += 1;
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                        }
                    }
                    if wlv.n_extra <= 0 as ::core::ffi::c_int {
                        if n_extra_next <= 0 as ::core::ffi::c_int {
                            if search_attr == 0 as ::core::ffi::c_int {
                                search_attr = saved_search_attr;
                                saved_search_attr = 0 as ::core::ffi::c_int;
                            }
                            if area_attr == 0 as ::core::ffi::c_int
                                && *ptr_0 as ::core::ffi::c_int != NUL
                            {
                                area_attr = saved_area_attr;
                                saved_area_attr = 0 as ::core::ffi::c_int;
                            }
                            if decor_attr == 0 as ::core::ffi::c_int {
                                decor_attr = saved_decor_attr;
                                saved_decor_attr = 0 as ::core::ffi::c_int;
                            }
                            if wlv.extra_for_extmark {
                                wlv.reset_extra_attr = true_0 != 0;
                                extra_attr_next = -1 as ::core::ffi::c_int;
                            }
                            wlv.extra_for_extmark = false_0 != 0;
                        } else {
                            '_c2rust_label_5: {
                                if wlv.sc_extra != '\0' as schar_T
                                    || wlv.sc_final != '\0' as schar_T
                                {
                                } else {
                                    __assert_fail(
                                        b"wlv.sc_extra != NUL || wlv.sc_final != NUL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2121 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            '_c2rust_label_6: {
                                if !wlv.p_extra.is_null() {
                                } else {
                                    __assert_fail(
                                        b"wlv.p_extra != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2122 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            wlv.sc_extra = NUL as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            wlv.n_extra = n_extra_next;
                            n_extra_next = 0 as ::core::ffi::c_int;
                            wlv.reset_extra_attr = true_0 != 0;
                            '_c2rust_label_7: {
                                if extra_attr_next >= 0 as ::core::ffi::c_int {
                                } else {
                                    __assert_fail(
                                        b"extra_attr_next >= 0\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2130 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                        }
                    }
                } else if wlv.filler_todo > 0 as ::core::ffi::c_int {
                    mb_c = ' ' as ::core::ffi::c_int;
                    mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                } else if has_foldtext as ::core::ffi::c_int != 0
                    || has_fold as ::core::ffi::c_int != 0 && wlv.col >= view_width
                {
                    mb_schar = NUL as schar_T;
                } else {
                    let mut prev_ptr_0: *const ::core::ffi::c_char = ptr_0;
                    let mut c0: ::core::ffi::c_int = *ptr_0 as uint8_t as ::core::ffi::c_int;
                    if c0 == NUL {
                        wlv.skip_cells = 0 as ::core::ffi::c_int;
                    }
                    mb_l = utfc_ptr2len(ptr_0);
                    mb_schar = utfc_ptr2schar(ptr_0, &raw mut mb_c);
                    if mb_l > 1 as ::core::ffi::c_int && mb_c < 0x80 as ::core::ffi::c_int {
                        c0 = mb_c;
                    }
                    if mb_l == 1 as ::core::ffi::c_int && c0 >= 0x80 as ::core::ffi::c_int
                        || mb_l >= 1 as ::core::ffi::c_int && mb_c == 0 as ::core::ffi::c_int
                        || mb_l > 1 as ::core::ffi::c_int && !vim_isprintc(mb_c)
                    {
                        transchar_hex(&raw mut wlv.extra as *mut ::core::ffi::c_char, mb_c);
                        if (*wp).w_onebuf_opt.wo_rl != 0 {
                            rl_mirror_ascii(
                                &raw mut wlv.extra as *mut ::core::ffi::c_char,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            );
                        }
                        wlv.p_extra = &raw mut wlv.extra as *mut ::core::ffi::c_char;
                        mb_c = mb_ptr2char_adv(
                            &raw mut wlv.p_extra as *mut *const ::core::ffi::c_char,
                        );
                        mb_schar = schar_from_char(mb_c);
                        wlv.n_extra = strlen(wlv.p_extra) as ::core::ffi::c_int;
                        wlv.sc_extra = NUL as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        if area_attr == 0 as ::core::ffi::c_int
                            && search_attr == 0 as ::core::ffi::c_int
                        {
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_8 as ::core::ffi::c_int);
                            saved_attr2 = wlv.char_attr;
                        }
                    } else if mb_l == 0 as ::core::ffi::c_int {
                        mb_l = 1 as ::core::ffi::c_int;
                    }
                    if wlv.col >= view_width - 1 as ::core::ffi::c_int
                        && schar_cells(mb_schar) == 2 as ::core::ffi::c_int
                    {
                        mb_schar = '>' as ::core::ffi::c_int as schar_T;
                        mb_c = '>' as ::core::ffi::c_int;
                        mb_l = 1 as ::core::ffi::c_int;
                        multi_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                        ptr_0 = ptr_0.offset(-1);
                        did_decrement_ptr = true_0 != 0;
                    } else if *ptr_0 as ::core::ffi::c_int != NUL {
                        ptr_0 = ptr_0.offset((mb_l - 1 as ::core::ffi::c_int) as isize);
                    }
                    if wlv.skip_cells > 0 as ::core::ffi::c_int
                        && mb_l > 1 as ::core::ffi::c_int
                        && wlv.n_extra == 0 as ::core::ffi::c_int
                    {
                        wlv.n_extra = 1 as ::core::ffi::c_int;
                        wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                        mb_c = ' ' as ::core::ffi::c_int;
                        mb_l = 1 as ::core::ffi::c_int;
                        if area_attr == 0 as ::core::ffi::c_int
                            && search_attr == 0 as ::core::ffi::c_int
                        {
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                            saved_attr2 = wlv.char_attr;
                        }
                    }
                    ptr_0 = ptr_0.offset(1);
                    decor_attr = 0 as ::core::ffi::c_int;
                    if extra_check {
                        let no_plain_buffer: bool = (*(*wp).w_s).b_p_spo_flags
                            & kOptSpoFlagNoplainbuffer as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0 as ::core::ffi::c_uint;
                        let mut can_spell: bool = !no_plain_buffer;
                        let v_3: ::core::ffi::c_int =
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                        let prev_v: ptrdiff_t = prev_ptr_0.offset_from(line_1);
                        if has_syntax as ::core::ffi::c_int != 0 && v_3 > 0 as ::core::ffi::c_int {
                            let mut save_did_emsg_0: ::core::ffi::c_int = did_emsg.get();
                            did_emsg.set(false_0);
                            decor_attr = get_syntax_attr(
                                v_3 as colnr_T - 1 as colnr_T,
                                if (*spv).spv_has_spell as ::core::ffi::c_int != 0 {
                                    &raw mut can_spell
                                } else {
                                    ::core::ptr::null_mut::<bool>()
                                },
                                false_0 != 0,
                            );
                            if did_emsg.get() != 0 {
                                (*(*wp).w_s).b_syn_error = true_0 != 0;
                                has_syntax = false_0 != 0;
                            } else {
                                did_emsg.set(save_did_emsg_0);
                            }
                            if (*(*wp).w_s).b_syn_slow {
                                has_syntax = false_0 != 0;
                            }
                            line_1 = ml_get_buf((*wp).w_buffer, lnum);
                            ptr_0 = line_1.offset(v_3 as isize);
                            prev_ptr_0 = line_1.offset(prev_v as isize);
                            syntax_flags = if mb_schar == 0 as schar_T {
                                0 as ::core::ffi::c_int
                            } else {
                                get_syntax_info(&raw mut syntax_seqnr)
                            };
                        }
                        if has_decor as ::core::ffi::c_int != 0 && v_3 > 0 as ::core::ffi::c_int {
                            decor_attr = hl_combine_attr(decor_attr, extmark_attr);
                            decor_conceal = (*decor_state.ptr()).conceal;
                            can_spell = if (*decor_state.ptr()).spell as ::core::ffi::c_int
                                == kTrue as ::core::ffi::c_int
                            {
                                true_0
                            } else if (*decor_state.ptr()).spell as ::core::ffi::c_int
                                == kFalse as ::core::ffi::c_int
                            {
                                false_0
                            } else {
                                can_spell as ::core::ffi::c_int
                            } != 0;
                        }
                        char_attr_base = hl_combine_attr(folded_attr, decor_attr);
                        wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                        let mut v1: ::core::ffi::c_int =
                            ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                        if (*spv).spv_has_spell as ::core::ffi::c_int != 0
                            && v1 >= word_end
                            && v1 > cur_checked_col
                        {
                            spell_attr = 0 as ::core::ffi::c_int;
                            if mb_schar != 0 as schar_T
                                && *skipwhite(prev_ptr_0) as ::core::ffi::c_int != NUL
                                && can_spell as ::core::ffi::c_int != 0
                            {
                                let mut p: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut spell_hlf_0: hlf_T = HLF_COUNT;
                                v1 -= mb_l - 1 as ::core::ffi::c_int;
                                if prev_ptr_0.offset_from(line_1) - nextlinecol as isize
                                    >= 0 as isize
                                {
                                    p = (&raw mut nextline as *mut ::core::ffi::c_char).offset(
                                        (prev_ptr_0.offset_from(line_1) - nextlinecol as isize)
                                            as isize,
                                    );
                                } else {
                                    p = prev_ptr_0 as *mut ::core::ffi::c_char;
                                }
                                (*spv).spv_cap_col -=
                                    prev_ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                                let mut tmplen: size_t = spell_check(
                                    wp,
                                    p,
                                    &raw mut spell_hlf_0,
                                    &raw mut (*spv).spv_cap_col,
                                    (*spv).spv_unchanged,
                                );
                                '_c2rust_label_8: {
                                    if tmplen <= 2147483647 as ::core::ffi::c_int as size_t {
                                    } else {
                                        __assert_fail(
                                            b"tmplen <= INT_MAX\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            b"src/nvim/drawline.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            2290 as ::core::ffi::c_uint,
                                            b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                let mut len_0: ::core::ffi::c_int = tmplen as ::core::ffi::c_int;
                                word_end = v1 + len_0;
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && State.get() & MODE_INSERT as ::core::ffi::c_int != 0
                                    && (*wp).w_cursor.lnum == lnum
                                    && (*wp).w_cursor.col
                                        >= prev_ptr_0.offset_from(line_1) as colnr_T
                                    && (*wp).w_cursor.col < word_end
                                {
                                    spell_hlf_0 = HLF_COUNT;
                                    spell_redraw_lnum.set(lnum);
                                }
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    == HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && p != prev_ptr_0 as *mut ::core::ffi::c_char
                                    && p.offset_from(&raw mut nextline as *mut ::core::ffi::c_char)
                                        + len_0 as isize
                                        > nextline_idx as isize
                                {
                                    (*spv).spv_checked_lnum = lnum + 1 as linenr_T;
                                    (*spv).spv_checked_col = (p
                                        .offset_from(&raw mut nextline as *mut ::core::ffi::c_char)
                                        + len_0 as isize
                                        - nextline_idx as isize)
                                        as ::core::ffi::c_int;
                                }
                                if spell_hlf_0 as ::core::ffi::c_uint
                                    != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    spell_attr = (*highlight_attr.ptr())[spell_hlf_0 as usize];
                                }
                                if (*spv).spv_cap_col > 0 as ::core::ffi::c_int {
                                    if p != prev_ptr_0 as *mut ::core::ffi::c_char
                                        && p.offset_from(
                                            &raw mut nextline as *mut ::core::ffi::c_char,
                                        ) + (*spv).spv_cap_col as isize
                                            >= nextline_idx as isize
                                    {
                                        (*spv).spv_capcol_lnum = lnum + 1 as linenr_T;
                                        (*spv).spv_cap_col = (p.offset_from(
                                            &raw mut nextline as *mut ::core::ffi::c_char,
                                        ) + (*spv).spv_cap_col as isize
                                            - nextline_idx as isize)
                                            as ::core::ffi::c_int;
                                    } else {
                                        (*spv).spv_cap_col +=
                                            prev_ptr_0.offset_from(line_1) as ::core::ffi::c_int;
                                    }
                                }
                            }
                        }
                        if spell_attr != 0 as ::core::ffi::c_int {
                            char_attr_base = hl_combine_attr(char_attr_base, spell_attr);
                            wlv.char_attr = hl_combine_attr(char_attr_base, char_attr_pri);
                        }
                        if !(*(*wp).w_buffer).terminal.is_null() {
                            wlv.char_attr = hl_combine_attr(
                                if wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int {
                                    term_attrs[wlv.vcol as usize]
                                } else {
                                    0 as ::core::ffi::c_int
                                },
                                wlv.char_attr,
                            );
                        }
                        if (*wp).w_onebuf_opt.wo_lbr != 0
                            && !wlv.need_lbr
                            && mb_schar != NUL as schar_T
                            && !vim_isbreak(*ptr_0 as uint8_t as ::core::ffi::c_int)
                        {
                            wlv.need_lbr = true_0 != 0;
                        }
                        if (*wp).w_onebuf_opt.wo_lbr != 0
                            && c0 == mb_c
                            && mb_c < 128 as ::core::ffi::c_int
                            && wlv.need_lbr as ::core::ffi::c_int != 0
                            && vim_isbreak(mb_c) as ::core::ffi::c_int != 0
                            && !vim_isbreak(*ptr_0 as uint8_t as ::core::ffi::c_int)
                        {
                            let mut mb_off: ::core::ffi::c_int = utf_head_off(
                                line_1,
                                ptr_0.offset(-(1 as ::core::ffi::c_int as isize)),
                            );
                            let mut p_0: *mut ::core::ffi::c_char =
                                ptr_0.offset(-((mb_off + 1 as ::core::ffi::c_int) as isize));
                            let mut csarg_0: CharsizeArg = CharsizeArg {
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
                                    s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                                    intersect_idx: 0,
                                    intersect_pos: MTPos { row: 0, col: 0 },
                                    intersect_pos_x: MTPos { row: 0, col: 0 },
                                }; 1],
                            };
                            let mut cstype_0: CSType =
                                init_charsize_arg(&raw mut csarg_0, wp, 0 as linenr_T, line_1);
                            wlv.n_extra = win_charsize(
                                cstype_0,
                                wlv.vcol as ::core::ffi::c_int,
                                p_0,
                                utf_ptr2CharInfo(p_0).value,
                                &raw mut csarg_0,
                            )
                            .width
                                - 1 as ::core::ffi::c_int;
                            if on_last_col as ::core::ffi::c_int != 0 && mb_c != TAB {
                                search_attr = 0 as ::core::ffi::c_int;
                            }
                            if mb_c == TAB && wlv.n_extra + wlv.col > view_width {
                                wlv.n_extra = tabstop_padding(
                                    wlv.vcol,
                                    (*(*wp).w_buffer).b_p_ts,
                                    (*(*wp).w_buffer).b_p_vts_array,
                                ) - 1 as ::core::ffi::c_int;
                            }
                            wlv.sc_extra = (if mb_off > 0 as ::core::ffi::c_int {
                                '<' as ::core::ffi::c_int
                            } else {
                                ' ' as ::core::ffi::c_int
                            }) as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            if mb_c < 128 as ::core::ffi::c_int
                                && ascii_iswhite(mb_c) as ::core::ffi::c_int != 0
                            {
                                if mb_c == TAB {
                                    fix_for_boguscols(&raw mut wlv);
                                }
                                if (*wp).w_onebuf_opt.wo_list == 0 {
                                    mb_c = ' ' as ::core::ffi::c_int;
                                    mb_schar = mb_c as schar_T;
                                }
                            }
                        }
                        if (*wp).w_onebuf_opt.wo_list != 0 {
                            in_multispace = mb_c == ' ' as ::core::ffi::c_int
                                && (*ptr_0 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                    || prev_ptr_0 > line_1 as *const ::core::ffi::c_char
                                        && *prev_ptr_0.offset(-1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == ' ' as ::core::ffi::c_int);
                            if !in_multispace {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                        }
                        if (*wp).w_onebuf_opt.wo_list != 0
                            && ((mb_c == 160 as ::core::ffi::c_int
                                && mb_l == 2 as ::core::ffi::c_int
                                || mb_c == 0x202f as ::core::ffi::c_int
                                    && mb_l == 3 as ::core::ffi::c_int)
                                && (*wp).w_p_lcs_chars.nbsp != 0
                                || mb_c == ' ' as ::core::ffi::c_int
                                    && mb_l == 1 as ::core::ffi::c_int
                                    && ((*wp).w_p_lcs_chars.space != 0
                                        || in_multispace as ::core::ffi::c_int != 0
                                            && !(*wp).w_p_lcs_chars.multispace.is_null())
                                    && ptr_0.offset_from(line_1) >= leadcol as isize
                                    && ptr_0.offset_from(line_1) <= trailcol as isize)
                        {
                            if in_multispace as ::core::ffi::c_int != 0
                                && !(*wp).w_p_lcs_chars.multispace.is_null()
                            {
                                let c2rust_fresh1 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                mb_schar = *(*wp)
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(c2rust_fresh1 as isize);
                                if *(*wp)
                                    .w_p_lcs_chars
                                    .multispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                            } else {
                                mb_schar = if mb_c == ' ' as ::core::ffi::c_int {
                                    (*wp).w_p_lcs_chars.space
                                } else {
                                    (*wp).w_p_lcs_chars.nbsp
                                };
                            }
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_0 as ::core::ffi::c_int);
                            saved_attr2 = wlv.char_attr;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                        if mb_c == ' ' as ::core::ffi::c_int
                            && mb_l == 1 as ::core::ffi::c_int
                            && (trailcol != MAXCOL as ::core::ffi::c_int
                                && ptr_0 > line_1.offset(trailcol as isize)
                                || leadcol != 0 as ::core::ffi::c_int
                                    && ptr_0 < line_1.offset(leadcol as isize))
                        {
                            if leadcol != 0 as ::core::ffi::c_int
                                && in_multispace as ::core::ffi::c_int != 0
                                && ptr_0 < line_1.offset(leadcol as isize)
                                && !(*wp).w_p_lcs_chars.leadmultispace.is_null()
                            {
                                let c2rust_fresh2 = multispace_pos;
                                multispace_pos = multispace_pos + 1;
                                mb_schar = *(*wp)
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(c2rust_fresh2 as isize);
                                if *(*wp)
                                    .w_p_lcs_chars
                                    .leadmultispace
                                    .offset(multispace_pos as isize)
                                    == NUL as schar_T
                                {
                                    multispace_pos = 0 as ::core::ffi::c_int;
                                }
                            } else if ptr_0 > line_1.offset(trailcol as isize)
                                && (*wp).w_p_lcs_chars.trail != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.trail;
                            } else if ptr_0 < line_1.offset(leadcol as isize)
                                && (*wp).w_p_lcs_chars.lead != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.lead;
                            } else if leadcol != 0 as ::core::ffi::c_int
                                && (*wp).w_p_lcs_chars.space != 0
                            {
                                mb_schar = (*wp).w_p_lcs_chars.space;
                            }
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_0 as ::core::ffi::c_int);
                            saved_attr2 = wlv.char_attr;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                    }
                    if !vim_isprintc(mb_c) {
                        if mb_c == TAB
                            && ((*wp).w_onebuf_opt.wo_list == 0 || (*wp).w_p_lcs_chars.tab1 != 0)
                        {
                            let mut tab_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut vcol_adjusted: colnr_T = wlv.vcol;
                            let mut lcs_tab1: schar_T = (*wp).w_p_lcs_chars.tab1;
                            let mut lcs_tab2: schar_T = (*wp).w_p_lcs_chars.tab2;
                            let mut lcs_tab3: schar_T = (*wp).w_p_lcs_chars.tab3;
                            if (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.leadtab1 != NUL as schar_T
                                && ptr_0 < line_1.offset(leadcol as isize)
                            {
                                lcs_tab1 = (*wp).w_p_lcs_chars.leadtab1;
                                lcs_tab2 = (*wp).w_p_lcs_chars.leadtab2;
                                lcs_tab3 = (*wp).w_p_lcs_chars.leadtab3;
                            }
                            let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
                            if *sbr as ::core::ffi::c_int != NUL
                                && wlv.vcol == wlv.vcol_sbr
                                && (*wp).w_onebuf_opt.wo_wrap != 0
                            {
                                vcol_adjusted =
                                    (wlv.vcol as ::core::ffi::c_int - mb_charlen(sbr)) as colnr_T;
                            }
                            tab_len = tabstop_padding(
                                vcol_adjusted,
                                (*(*wp).w_buffer).b_p_ts,
                                (*(*wp).w_buffer).b_p_vts_array,
                            ) - 1 as ::core::ffi::c_int;
                            if (*wp).w_onebuf_opt.wo_lbr == 0 || (*wp).w_onebuf_opt.wo_list == 0 {
                                wlv.n_extra = tab_len;
                            } else {
                                let mut saved_nextra: ::core::ffi::c_int = wlv.n_extra;
                                if wlv.vcol_off_co > 0 as ::core::ffi::c_int {
                                    tab_len += wlv.vcol_off_co;
                                }
                                if lcs_tab1 != 0
                                    && wlv.old_boguscols > 0 as ::core::ffi::c_int
                                    && wlv.n_extra > tab_len
                                {
                                    tab_len += wlv.n_extra - tab_len;
                                }
                                if tab_len > 0 as ::core::ffi::c_int {
                                    let mut tab2_len: size_t = schar_len(lcs_tab2);
                                    let mut len_1: size_t =
                                        (tab_len as size_t).wrapping_mul(tab2_len);
                                    if lcs_tab3 != 0 {
                                        len_1 = len_1.wrapping_add(
                                            schar_len(lcs_tab3).wrapping_sub(tab2_len),
                                        );
                                    }
                                    if wlv.n_extra > 0 as ::core::ffi::c_int {
                                        len_1 =
                                            len_1.wrapping_add((wlv.n_extra - tab_len) as size_t);
                                    }
                                    mb_schar = lcs_tab1;
                                    mb_c = schar_get_first_codepoint(mb_schar);
                                    let mut p_1: *mut ::core::ffi::c_char =
                                        get_extra_buf(len_1.wrapping_add(1 as size_t));
                                    memset(
                                        p_1 as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        len_1,
                                    );
                                    *p_1.offset(len_1 as isize) = NUL as ::core::ffi::c_char;
                                    wlv.p_extra = p_1;
                                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while i < tab_len {
                                        if *p_1 as ::core::ffi::c_int == NUL {
                                            tab_len = i;
                                            break;
                                        } else {
                                            let mut lcs: schar_T = lcs_tab2;
                                            if lcs_tab3 != 0
                                                && i == tab_len - 1 as ::core::ffi::c_int
                                            {
                                                lcs = lcs_tab3;
                                            }
                                            let mut slen: size_t = schar_get_adv(&raw mut p_1, lcs);
                                            wlv.n_extra += slen as ::core::ffi::c_int
                                                - (if saved_nextra > 0 as ::core::ffi::c_int {
                                                    1 as ::core::ffi::c_int
                                                } else {
                                                    0 as ::core::ffi::c_int
                                                });
                                            i += 1;
                                        }
                                    }
                                    if wlv.vcol_off_co > 0 as ::core::ffi::c_int {
                                        wlv.n_extra -= wlv.vcol_off_co;
                                    }
                                }
                            }
                            let mut vc_saved: ::core::ffi::c_int = wlv.vcol_off_co;
                            fix_for_boguscols(&raw mut wlv);
                            if wlv.n_extra == tab_len + vc_saved
                                && (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.tab1 != 0
                            {
                                tab_len += vc_saved;
                            }
                            if (*wp).w_onebuf_opt.wo_list != 0 {
                                mb_schar =
                                    if wlv.n_extra == 0 as ::core::ffi::c_int && lcs_tab3 != 0 {
                                        lcs_tab3
                                    } else {
                                        lcs_tab1
                                    };
                                if (*wp).w_onebuf_opt.wo_lbr != 0
                                    && !wlv.p_extra.is_null()
                                    && *wlv.p_extra as ::core::ffi::c_int != NUL
                                {
                                    wlv.sc_extra = NUL as schar_T;
                                } else {
                                    wlv.sc_extra = lcs_tab2;
                                }
                                wlv.sc_final = lcs_tab3;
                                wlv.n_attr = tab_len + 1 as ::core::ffi::c_int;
                                wlv.extra_attr = win_hl_attr(wp, HLF_0 as ::core::ffi::c_int);
                                saved_attr2 = wlv.char_attr;
                            } else {
                                wlv.sc_final = NUL as schar_T;
                                wlv.sc_extra = ' ' as ::core::ffi::c_int as schar_T;
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            mb_c = schar_get_first_codepoint(mb_schar);
                        } else if mb_schar == NUL as schar_T
                            && ((*wp).w_onebuf_opt.wo_list != 0
                                || (wlv.fromcol >= 0 as ::core::ffi::c_int
                                    || fromcol_prev >= 0 as ::core::ffi::c_int)
                                    && wlv.tocol > wlv.vcol
                                    && VIsual_mode.get() != Ctrl_V
                                    && wlv.col < view_width
                                    && !(noinvcur as ::core::ffi::c_int != 0
                                        && lnum == (*wp).w_cursor.lnum
                                        && wlv.vcol == (*wp).w_virtcol))
                            && lcs_eol_todo as ::core::ffi::c_int != 0
                            && lcs_eol != NUL as schar_T
                        {
                            if wlv.diff_hlf as ::core::ffi::c_uint
                                == HLF_NONE as ::core::ffi::c_uint
                                && wlv.line_attr == 0 as ::core::ffi::c_int
                                && wlv.line_attr_lowprio == 0 as ::core::ffi::c_int
                            {
                                if !(area_highlighting as ::core::ffi::c_int != 0
                                    && virtual_active(wp) as ::core::ffi::c_int != 0
                                    && wlv.tocol != MAXCOL as ::core::ffi::c_int
                                    && wlv.vcol < wlv.tocol)
                                {
                                    wlv.p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                }
                                wlv.n_extra = 0 as ::core::ffi::c_int;
                            }
                            if (*wp).w_onebuf_opt.wo_list != 0
                                && (*wp).w_p_lcs_chars.eol > 0 as schar_T
                            {
                                mb_schar = (*wp).w_p_lcs_chars.eol;
                            } else {
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            lcs_eol_todo = false_0 != 0;
                            ptr_0 = ptr_0.offset(-1);
                            wlv.extra_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                            wlv.n_attr = 1 as ::core::ffi::c_int;
                            mb_c = schar_get_first_codepoint(mb_schar);
                        } else if mb_schar != NUL as schar_T {
                            wlv.p_extra = transchar_buf((*wp).w_buffer, mb_c);
                            if wlv.n_extra == 0 as ::core::ffi::c_int {
                                wlv.n_extra = byte2cells(mb_c) - 1 as ::core::ffi::c_int;
                            }
                            if dy_flags.get()
                                & kOptDyFlagUhex as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0
                                && (*wp).w_onebuf_opt.wo_rl != 0
                            {
                                rl_mirror_ascii(
                                    wlv.p_extra,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                );
                            }
                            wlv.sc_extra = NUL as schar_T;
                            wlv.sc_final = NUL as schar_T;
                            if (*wp).w_onebuf_opt.wo_lbr != 0 {
                                mb_c = *wlv.p_extra as uint8_t as ::core::ffi::c_int;
                                let mut p_2: *mut ::core::ffi::c_char = get_extra_buf(
                                    (wlv.n_extra as size_t).wrapping_add(1 as size_t),
                                );
                                memset(
                                    p_2 as *mut ::core::ffi::c_void,
                                    ' ' as ::core::ffi::c_int,
                                    wlv.n_extra as size_t,
                                );
                                memcpy(
                                    p_2 as *mut ::core::ffi::c_void,
                                    wlv.p_extra.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    strlen(wlv.p_extra).wrapping_sub(1 as size_t),
                                );
                                *p_2.offset(wlv.n_extra as isize) = NUL as ::core::ffi::c_char;
                                wlv.p_extra = p_2;
                            } else {
                                wlv.n_extra = byte2cells(mb_c) - 1 as ::core::ffi::c_int;
                                let c2rust_fresh3 = wlv.p_extra;
                                wlv.p_extra = wlv.p_extra.offset(1);
                                mb_c = *c2rust_fresh3 as uint8_t as ::core::ffi::c_int;
                            }
                            wlv.n_attr = wlv.n_extra + 1 as ::core::ffi::c_int;
                            wlv.extra_attr = win_hl_attr(wp, HLF_8 as ::core::ffi::c_int);
                            saved_attr2 = wlv.char_attr;
                            mb_schar = mb_c as schar_T;
                        } else if VIsual_active.get() as ::core::ffi::c_int != 0
                            && (VIsual_mode.get() == Ctrl_V
                                || VIsual_mode.get() == 'v' as ::core::ffi::c_int)
                            && virtual_active(wp) as ::core::ffi::c_int != 0
                            && wlv.tocol != MAXCOL as ::core::ffi::c_int
                            && wlv.vcol < wlv.tocol
                            && wlv.col < view_width
                        {
                            mb_c = ' ' as ::core::ffi::c_int;
                            mb_schar = schar_from_char(mb_c);
                            ptr_0 = ptr_0.offset(-1);
                        }
                    }
                    if (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
                        && (wp != curwin.get()
                            || lnum != (*wp).w_cursor.lnum
                            || conceal_cursor_line(wp) as ::core::ffi::c_int != 0)
                        && (syntax_flags & HL_CONCEAL as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                            || has_match_conc > 0 as ::core::ffi::c_int
                            || decor_conceal > 0 as ::core::ffi::c_int)
                        && !(lnum_in_visual_area as ::core::ffi::c_int != 0
                            && vim_strchr((*wp).w_onebuf_opt.wo_cocu, 'v' as ::core::ffi::c_int)
                                .is_null())
                    {
                        let mut syntax_conceal: bool = syntax_flags
                            & HL_CONCEAL as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int;
                        wlv.char_attr = conceal_attr;
                        if (prev_syntax_id != syntax_seqnr
                            && syntax_conceal as ::core::ffi::c_int != 0
                            || has_match_conc > 1 as ::core::ffi::c_int
                            || decor_conceal > 1 as ::core::ffi::c_int)
                            && (syntax_conceal as ::core::ffi::c_int != 0
                                && syn_get_sub_char() != NUL
                                || has_match_conc != 0 && match_conc != 0
                                || decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0
                                || (*wp).w_onebuf_opt.wo_cole == 1 as OptInt)
                            && (*wp).w_onebuf_opt.wo_cole != 3 as OptInt
                        {
                            if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                                wlv.n_extra += 1;
                            }
                            if has_match_conc != 0 && match_conc != 0 {
                                mb_schar = schar_from_char(match_conc);
                            } else if decor_conceal != 0 && (*decor_state.ptr()).conceal_char != 0 {
                                mb_schar = (*decor_state.ptr()).conceal_char;
                                if (*decor_state.ptr()).conceal_attr != 0 {
                                    wlv.char_attr = (*decor_state.ptr()).conceal_attr;
                                }
                            } else if syntax_conceal as ::core::ffi::c_int != 0
                                && syn_get_sub_char() != NUL
                            {
                                mb_schar = schar_from_char(syn_get_sub_char());
                            } else if (*wp).w_p_lcs_chars.conceal != NUL as schar_T {
                                mb_schar = (*wp).w_p_lcs_chars.conceal;
                            } else {
                                mb_schar = ' ' as ::core::ffi::c_int as schar_T;
                            }
                            mb_c = schar_get_first_codepoint(mb_schar);
                            prev_syntax_id = syntax_seqnr;
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol_off_co += wlv.n_extra;
                            }
                            wlv.vcol += wlv.n_extra;
                            if is_wrapped as ::core::ffi::c_int != 0
                                && wlv.n_extra > 0 as ::core::ffi::c_int
                            {
                                wlv.boguscols += wlv.n_extra;
                                wlv.col += wlv.n_extra;
                            }
                            wlv.n_extra = 0 as ::core::ffi::c_int;
                            wlv.n_attr = 0 as ::core::ffi::c_int;
                        } else if wlv.skip_cells == 0 as ::core::ffi::c_int {
                            is_concealing = true_0 != 0;
                            wlv.skip_cells = 1 as ::core::ffi::c_int;
                        }
                    } else {
                        prev_syntax_id = 0 as ::core::ffi::c_int;
                        is_concealing = false_0 != 0;
                    }
                    if wlv.skip_cells > 0 as ::core::ffi::c_int
                        && did_decrement_ptr as ::core::ffi::c_int != 0
                    {
                        ptr_0 = ptr_0.offset(1);
                    }
                }
                if !did_wcol
                    && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && in_curline as ::core::ffi::c_int != 0
                    && conceal_cursor_line(wp) as ::core::ffi::c_int != 0
                    && (wlv.vcol as ::core::ffi::c_int + wlv.skip_cells >= (*wp).w_virtcol
                        || mb_schar == NUL as schar_T)
                {
                    (*wp).w_wcol = wlv.col - wlv.boguscols;
                    if wlv.vcol as ::core::ffi::c_int + wlv.skip_cells < (*wp).w_virtcol {
                        (*wp).w_wcol += (*wp).w_virtcol as ::core::ffi::c_int
                            - wlv.vcol as ::core::ffi::c_int
                            - wlv.skip_cells;
                    }
                    (*wp).w_wrow = wlv.row;
                    did_wcol = true_0 != 0;
                    (*wp).w_valid |= VALID_WCOL | VALID_WROW | VALID_VIRTCOL;
                }
                if wlv.n_attr > 0 as ::core::ffi::c_int && !search_attr_from_match {
                    wlv.char_attr = hl_combine_attr(wlv.char_attr, wlv.extra_attr);
                    if wlv.reset_extra_attr {
                        wlv.reset_extra_attr = false_0 != 0;
                        if extra_attr_next >= 0 as ::core::ffi::c_int {
                            wlv.extra_attr = extra_attr_next;
                            extra_attr_next = -1 as ::core::ffi::c_int;
                        } else {
                            wlv.extra_attr = 0 as ::core::ffi::c_int;
                            search_attr_from_match = saved_search_attr_from_match;
                        }
                    }
                }
                if lcs_prec_todo != NUL as schar_T
                    && (*wp).w_onebuf_opt.wo_list != 0
                    && (if (*wp).w_onebuf_opt.wo_wrap != 0 {
                        ((*wp).w_skipcol > 0 as ::core::ffi::c_int
                            && wlv.row == 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } else {
                        ((*wp).w_leftcol > 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    }) != 0
                    && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    && wlv.skip_cells <= 0 as ::core::ffi::c_int
                    && mb_schar != NUL as schar_T
                {
                    lcs_prec_todo = NUL as schar_T;
                    if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                        wlv.sc_extra = '<' as ::core::ffi::c_int as schar_T;
                        wlv.sc_final = NUL as schar_T;
                        if wlv.n_extra > 0 as ::core::ffi::c_int {
                            '_c2rust_label_9: {
                                if !wlv.p_extra.is_null() {
                                } else {
                                    __assert_fail(
                                        b"wlv.p_extra != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/drawline.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2749 as ::core::ffi::c_uint,
                                        b"int win_line(win_T *, linenr_T, int, int, int, _Bool, spellvars_T *, foldinfo_T)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            n_extra_next = wlv.n_extra;
                            extra_attr_next = wlv.extra_attr;
                            wlv.n_attr =
                                if wlv.n_attr + 1 as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
                                    wlv.n_attr + 1 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                };
                        } else {
                            wlv.n_attr = 2 as ::core::ffi::c_int;
                        }
                        wlv.n_extra = 1 as ::core::ffi::c_int;
                        wlv.extra_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                    }
                    mb_schar = (*wp).w_p_lcs_chars.prec;
                    mb_c = schar_get_first_codepoint(mb_schar);
                    saved_attr3 = wlv.char_attr;
                    wlv.char_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                    n_attr3 = 1 as ::core::ffi::c_int;
                }
                if mb_schar == NUL as schar_T && eol_hl_off == 0 as ::core::ffi::c_int {
                    let prevcol_hl_flag: bool = get_prevcol_hl_flag(
                        wp,
                        screen_search_hl.ptr(),
                        ptr_0.offset_from(line_1) as colnr_T - 1 as colnr_T,
                    );
                    if lcs_eol_todo as ::core::ffi::c_int != 0
                        && (area_attr != 0 as ::core::ffi::c_int
                            && wlv.vcol == wlv.fromcol
                            && (VIsual_mode.get() != Ctrl_V
                                || lnum == (*VIsual.ptr()).lnum
                                || lnum == (*curwin.get()).w_cursor.lnum)
                            || prevcol_hl_flag as ::core::ffi::c_int != 0)
                    {
                        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if wlv.col >= view_width {
                            n = -1 as ::core::ffi::c_int;
                        }
                        if n != 0 as ::core::ffi::c_int {
                            wlv.off += n;
                            wlv.col += n;
                        } else {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                                ' ' as ::core::ffi::c_int as schar_T;
                        }
                        if area_attr == 0 as ::core::ffi::c_int && !has_fold {
                            get_search_match_hl(
                                wp,
                                screen_search_hl.ptr(),
                                ptr_0.offset_from(line_1) as colnr_T,
                                &raw mut wlv.char_attr,
                            );
                        }
                        let eol_attr: ::core::ffi::c_int = if wlv.cul_attr != 0 {
                            hl_combine_attr(wlv.cul_attr, wlv.char_attr)
                        } else {
                            wlv.char_attr
                        };
                        *(*linebuf_attr.ptr()).offset(wlv.off as isize) = eol_attr as sattr_T;
                        *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                        wlv.col += 1;
                        wlv.off += 1;
                        wlv.vcol += 1;
                        eol_hl_off = 1 as ::core::ffi::c_int;
                    }
                }
                if mb_schar == NUL as schar_T {
                    wlv.vcol = (if wlv.vcol > start_vcol + wlv.col - win_col_off(wp) {
                        wlv.vcol as ::core::ffi::c_int
                    } else {
                        start_vcol + wlv.col - win_col_off(wp)
                    }) as colnr_T;
                    wlv.col -= wlv.boguscols;
                    wlv.boguscols = 0 as ::core::ffi::c_int;
                    advance_color_col(
                        &raw mut wlv,
                        wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co,
                    );
                    let eol_skip: ::core::ffi::c_int = if lcs_eol_todo as ::core::ffi::c_int != 0
                        && eol_hl_off == 0 as ::core::ffi::c_int
                    {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    if has_decor {
                        decor_redraw_eol(
                            wp,
                            decor_state.ptr(),
                            &raw mut wlv.line_attr,
                            wlv.col + eol_skip,
                        );
                    }
                    let mut i_0: ::core::ffi::c_int = wlv.col;
                    while i_0 < view_width {
                        *(*linebuf_vcol.ptr()).offset((wlv.off + (i_0 - wlv.col)) as isize) =
                            (wlv.vcol as ::core::ffi::c_int + (i_0 - wlv.col)) as colnr_T;
                        i_0 += 1;
                    }
                    if (*wp).w_onebuf_opt.wo_cuc != 0
                        && (*wp).w_virtcol
                            >= wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co - eol_hl_off
                        && ((*wp).w_virtcol as ptrdiff_t)
                            < view_width as ptrdiff_t
                                * (wlv.row - startrow + 1 as ::core::ffi::c_int) as ptrdiff_t
                                + start_vcol as ptrdiff_t
                        && lnum != (*wp).w_cursor.lnum
                        || !wlv.color_cols.is_null()
                        || wlv.line_attr_lowprio != 0
                        || wlv.line_attr != 0
                        || wlv.diff_hlf as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint
                        || !(*(*wp).w_buffer).terminal.is_null()
                    {
                        let mut rightmost_vcol: ::core::ffi::c_int =
                            get_rightmost_vcol(wp, wlv.color_cols);
                        let cuc_attr: ::core::ffi::c_int =
                            win_hl_attr(wp, HLF_CUC as ::core::ffi::c_int);
                        let mc_attr: ::core::ffi::c_int =
                            win_hl_attr(wp, HLF_MC as ::core::ffi::c_int);
                        if wlv.diff_hlf as ::core::ffi::c_uint
                            == HLF_TXD as ::core::ffi::c_int as ::core::ffi::c_uint
                            || wlv.diff_hlf as ::core::ffi::c_uint
                                == HLF_TXA as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            wlv.diff_hlf = HLF_CHD;
                            set_line_attr_for_diff(wp, &raw mut wlv);
                        }
                        let diff_attr: ::core::ffi::c_int =
                            if wlv.diff_hlf as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint {
                                win_hl_attr(wp, wlv.diff_hlf as ::core::ffi::c_int)
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        let base_attr: ::core::ffi::c_int =
                            hl_combine_attr(wlv.line_attr_lowprio, diff_attr);
                        if base_attr != 0
                            || wlv.line_attr != 0
                            || !(*(*wp).w_buffer).terminal.is_null()
                        {
                            rightmost_vcol = INT_MAX;
                        }
                        while wlv.col < view_width {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                                ' ' as ::core::ffi::c_int as schar_T;
                            advance_color_col(
                                &raw mut wlv,
                                wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co,
                            );
                            let mut col_attr: ::core::ffi::c_int = base_attr;
                            if (*wp).w_onebuf_opt.wo_cuc != 0
                                && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co
                                    == (*wp).w_virtcol
                                && lnum != (*wp).w_cursor.lnum
                            {
                                col_attr = hl_combine_attr(col_attr, cuc_attr);
                            } else if !wlv.color_cols.is_null()
                                && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co
                                    == *wlv.color_cols
                            {
                                col_attr = hl_combine_attr(col_attr, mc_attr);
                            }
                            if !(*(*wp).w_buffer).terminal.is_null()
                                && wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int
                            {
                                col_attr = hl_combine_attr(col_attr, term_attrs[wlv.vcol as usize]);
                            }
                            col_attr = hl_combine_attr(col_attr, wlv.line_attr);
                            *(*linebuf_attr.ptr()).offset(wlv.off as isize) = col_attr as sattr_T;
                            wlv.off += 1;
                            wlv.col += 1;
                            wlv.vcol += 1;
                            if wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co > rightmost_vcol {
                                break;
                            }
                        }
                    }
                    if fold_vt.size > 0 as size_t {
                        draw_virt_text_item(
                            buf,
                            win_col_offset,
                            fold_vt,
                            kHlModeCombine,
                            view_width,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    draw_virt_text(wp, buf, win_col_offset, &raw mut wlv.col, wlv.row);
                    wlv_put_linebuf(
                        wp,
                        &raw mut wlv,
                        wlv.col,
                        true_0 != 0,
                        bg_attr,
                        SLF_INC_VCOL as ::core::ffi::c_int,
                    );
                    wlv.row += 1;
                    if in_curline {
                        (*curwin.get()).w_cline_row = startrow;
                        (*curwin.get()).w_cline_height = wlv.row - startrow;
                        (*curwin.get()).w_cline_folded = has_fold;
                        (*curwin.get()).w_valid |= VALID_CHEIGHT | VALID_CROW;
                    }
                    break 's_5143;
                } else {
                    lcs_ext = get_lcs_ext(wp);
                    if lcs_ext != NUL as schar_T
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                        && wlv.col == view_width - 1 as ::core::ffi::c_int
                        && !has_foldtext
                    {
                        if has_decor as ::core::ffi::c_int != 0
                            && *ptr_0 as ::core::ffi::c_int == NUL
                            && lcs_eol == 0 as schar_T
                            && lcs_eol_todo as ::core::ffi::c_int != 0
                        {
                            decor_redraw_col(
                                wp,
                                ptr_0.offset_from(line_1) as ::core::ffi::c_int,
                                -1 as ::core::ffi::c_int,
                                false_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                        }
                        if *ptr_0 as ::core::ffi::c_int != NUL
                            || lcs_eol > 0 as schar_T && lcs_eol_todo as ::core::ffi::c_int != 0
                            || wlv.n_extra > 0 as ::core::ffi::c_int
                                && (wlv.sc_extra != NUL as schar_T
                                    || *wlv.p_extra as ::core::ffi::c_int != NUL)
                            || may_have_inline_virt as ::core::ffi::c_int != 0
                                && has_more_inline_virt(&raw mut wlv, ptr_0.offset_from(line_1))
                                    as ::core::ffi::c_int
                                    != 0
                        {
                            mb_schar = lcs_ext;
                            wlv.char_attr = win_hl_attr(wp, HLF_AT as ::core::ffi::c_int);
                            mb_c = schar_get_first_codepoint(mb_schar);
                        }
                    }
                    advance_color_col(
                        &raw mut wlv,
                        wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co,
                    );
                    vcol_save_attr = -1 as ::core::ffi::c_int;
                    if !lnum_in_visual_area
                        && search_attr == 0 as ::core::ffi::c_int
                        && area_attr == 0 as ::core::ffi::c_int
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                    {
                        if (*wp).w_onebuf_opt.wo_cuc != 0
                            && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co == (*wp).w_virtcol
                            && lnum != (*wp).w_cursor.lnum
                        {
                            vcol_save_attr = wlv.char_attr;
                            wlv.char_attr = hl_combine_attr(
                                win_hl_attr(wp, HLF_CUC as ::core::ffi::c_int),
                                wlv.char_attr,
                            );
                        } else if !wlv.color_cols.is_null()
                            && wlv.vcol as ::core::ffi::c_int - wlv.vcol_off_co == *wlv.color_cols
                        {
                            vcol_save_attr = wlv.char_attr;
                            wlv.char_attr = hl_combine_attr(
                                win_hl_attr(wp, HLF_MC as ::core::ffi::c_int),
                                wlv.char_attr,
                            );
                        }
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        let mut low: ::core::ffi::c_int = wlv.line_attr_lowprio;
                        let mut high: ::core::ffi::c_int = wlv.char_attr;
                        if wlv.line_attr_lowprio != 0 as ::core::ffi::c_int {
                            let mut line_ae: HlAttrs = syn_attr2entry(wlv.line_attr_lowprio);
                            let mut char_ae: HlAttrs = syn_attr2entry(wlv.char_attr);
                            let mut win_normal_bg: ::core::ffi::c_int =
                                normal_bg.get() as ::core::ffi::c_int;
                            let mut win_normal_cterm_bg: ::core::ffi::c_int =
                                cterm_normal_bg_color.get();
                            if bg_attr != 0 as ::core::ffi::c_int {
                                let mut norm_ae: HlAttrs = syn_attr2entry(bg_attr);
                                win_normal_bg = norm_ae.rgb_bg_color as ::core::ffi::c_int;
                                win_normal_cterm_bg = norm_ae.cterm_bg_color as ::core::ffi::c_int;
                            }
                            let mut char_is_normal_bg: bool =
                                if ui_rgb_attached() as ::core::ffi::c_int != 0 {
                                    (char_ae.rgb_bg_color == win_normal_bg as RgbValue)
                                        as ::core::ffi::c_int
                                } else {
                                    (char_ae.cterm_bg_color as ::core::ffi::c_int
                                        == win_normal_cterm_bg)
                                        as ::core::ffi::c_int
                                } != 0;
                            if (line_ae.rgb_bg_color >= 0 as RgbValue
                                || line_ae.cterm_bg_color as ::core::ffi::c_int
                                    > 0 as ::core::ffi::c_int)
                                && char_is_normal_bg as ::core::ffi::c_int != 0
                            {
                                low = wlv.char_attr;
                                high = wlv.line_attr_lowprio;
                            }
                        }
                        wlv.char_attr = hl_combine_attr(low, high);
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        vcol_prev = wlv.vcol;
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        if wlv.skip_cells <= 0 as ::core::ffi::c_int {
                            *(*linebuf_char.ptr()).offset(wlv.off as isize) = mb_schar;
                            if multi_attr != 0 {
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) =
                                    multi_attr as sattr_T;
                                multi_attr = 0 as ::core::ffi::c_int;
                            } else {
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) =
                                    wlv.char_attr as sattr_T;
                            }
                            *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                            if schar_cells(mb_schar) > 1 as ::core::ffi::c_int {
                                wlv.off += 1;
                                wlv.col += 1;
                                *(*linebuf_char.ptr()).offset(wlv.off as isize) = 0 as schar_T;
                                *(*linebuf_attr.ptr()).offset(wlv.off as isize) = *(*linebuf_attr
                                    .ptr())
                                .offset((wlv.off - 1 as ::core::ffi::c_int) as isize);
                                wlv.vcol += 1;
                                *(*linebuf_vcol.ptr()).offset(wlv.off as isize) = wlv.vcol;
                                if wlv.tocol == wlv.vcol {
                                    wlv.tocol += 1;
                                }
                            }
                            wlv.off += 1;
                            wlv.col += 1;
                        } else if (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
                            && is_concealing as ::core::ffi::c_int != 0
                        {
                            let mut concealed_wide: bool =
                                schar_cells(mb_schar) > 1 as ::core::ffi::c_int;
                            wlv.skip_cells -= 1;
                            wlv.vcol_off_co += 1;
                            if concealed_wide {
                                wlv.vcol += 1;
                                wlv.vcol_off_co += 1;
                            }
                            if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol_off_co += wlv.n_extra;
                            }
                            if is_wrapped {
                                if wlv.n_extra > 0 as ::core::ffi::c_int {
                                    wlv.vcol += wlv.n_extra;
                                    wlv.col += wlv.n_extra;
                                    wlv.boguscols += wlv.n_extra;
                                    wlv.n_extra = 0 as ::core::ffi::c_int;
                                    wlv.n_attr = 0 as ::core::ffi::c_int;
                                }
                                if concealed_wide {
                                    wlv.boguscols += 1;
                                    wlv.col += 1;
                                }
                                wlv.boguscols += 1;
                                wlv.col += 1;
                            } else if wlv.n_extra > 0 as ::core::ffi::c_int {
                                wlv.vcol += wlv.n_extra;
                                wlv.n_extra = 0 as ::core::ffi::c_int;
                                wlv.n_attr = 0 as ::core::ffi::c_int;
                            }
                        } else {
                            wlv.skip_cells -= 1;
                        }
                    }
                    if wlv.skipped_cells > 0 as ::core::ffi::c_int {
                        wlv.vcol += wlv.skipped_cells;
                        wlv.skipped_cells = 0 as ::core::ffi::c_int;
                    }
                    if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                        wlv.vcol += 1;
                    }
                    if vcol_save_attr >= 0 as ::core::ffi::c_int {
                        wlv.char_attr = vcol_save_attr;
                    }
                    if n_attr3 > 0 as ::core::ffi::c_int && {
                        n_attr3 -= 1;
                        n_attr3 == 0 as ::core::ffi::c_int
                    } {
                        wlv.char_attr = saved_attr3;
                    }
                    if wlv.n_attr > 0 as ::core::ffi::c_int && {
                        wlv.n_attr -= 1;
                        wlv.n_attr == 0 as ::core::ffi::c_int
                    } {
                        wlv.char_attr = saved_attr2;
                    }
                    if has_decor as ::core::ffi::c_int != 0
                        && wlv.filler_todo <= 0 as ::core::ffi::c_int
                        && wlv.col >= view_width
                    {
                        if is_wrapped as ::core::ffi::c_int != 0
                            && wlv.n_extra == 0 as ::core::ffi::c_int
                        {
                            decor_redraw_col(
                                wp,
                                ptr_0.offset_from(line_1) as ::core::ffi::c_int,
                                -3 as ::core::ffi::c_int,
                                false_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                            decor_need_recheck = true_0 != 0;
                        } else if !is_wrapped {
                            decor_recheck_draw_col(
                                -1 as ::core::ffi::c_int,
                                true_0 != 0,
                                decor_state.ptr(),
                            );
                            decor_redraw_col(
                                wp,
                                MAXCOL as ::core::ffi::c_int,
                                -1 as ::core::ffi::c_int,
                                true_0 != 0,
                                decor_state.ptr(),
                                decor_provider_end_col - 1 as ::core::ffi::c_int,
                            );
                        }
                    }
                }
            }
        }
        if !(wlv.col >= view_width
            && (!has_foldtext || wlv.filler_todo > 0 as ::core::ffi::c_int)
            && (wlv.col <= leftcols_width
                || *ptr_0 as ::core::ffi::c_int != NUL
                || wlv.filler_todo > 0 as ::core::ffi::c_int
                || (*wp).w_onebuf_opt.wo_list != 0
                    && (*wp).w_p_lcs_chars.eol != NUL as schar_T
                    && lcs_eol_todo as ::core::ffi::c_int != 0
                || wlv.n_extra != 0 as ::core::ffi::c_int
                    && (wlv.sc_extra != NUL as schar_T
                        || *wlv.p_extra as ::core::ffi::c_int != NUL)
                || may_have_inline_virt as ::core::ffi::c_int != 0
                    && has_more_inline_virt(&raw mut wlv, ptr_0.offset_from(line_1))
                        as ::core::ffi::c_int
                        != 0))
        {
            continue;
        }
        let mut grid_width: ::core::ffi::c_int = (*(*wp).w_grid.target).cols;
        let wrap: bool = is_wrapped as ::core::ffi::c_int != 0
            && wlv.filler_todo <= 0 as ::core::ffi::c_int
            && lcs_eol_todo as ::core::ffi::c_int != 0
            && wlv.row != endrow - 1 as ::core::ffi::c_int
            && view_width == grid_width
            && (*wp).w_onebuf_opt.wo_rl == 0;
        let mut draw_col: ::core::ffi::c_int = wlv.col - wlv.boguscols;
        let mut i_1: ::core::ffi::c_int = draw_col;
        while i_1 < view_width {
            *(*linebuf_vcol.ptr()).offset((wlv.off + (i_1 - draw_col)) as isize) =
                (wlv.vcol as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            i_1 += 1;
        }
        if wlv.boguscols != 0 as ::core::ffi::c_int
            && (wlv.line_attr_lowprio != 0 as ::core::ffi::c_int
                || wlv.line_attr != 0 as ::core::ffi::c_int)
        {
            let mut attr: ::core::ffi::c_int =
                hl_combine_attr(wlv.line_attr_lowprio, wlv.line_attr);
            while draw_col < view_width {
                *(*linebuf_char.ptr()).offset(wlv.off as isize) =
                    schar_from_char(' ' as ::core::ffi::c_int);
                *(*linebuf_attr.ptr()).offset(wlv.off as isize) = attr as sattr_T;
                wlv.off += 1;
                draw_col += 1;
            }
        }
        if virt_line_index >= 0 as ::core::ffi::c_int {
            draw_virt_text_item(
                buf,
                if virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    win_col_offset
                },
                (*virt_lines.items.offset(virt_line_index as isize)).line,
                kHlModeReplace,
                view_width,
                0 as ::core::ffi::c_int,
                if virt_line_flags & kVLScroll as ::core::ffi::c_int != 0 {
                    (*wp).w_leftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        } else if wlv.filler_todo <= 0 as ::core::ffi::c_int {
            draw_virt_text(wp, buf, win_col_offset, &raw mut draw_col, wlv.row);
        }
        wlv_put_linebuf(
            wp,
            &raw mut wlv,
            draw_col,
            true_0 != 0,
            bg_attr,
            if wrap as ::core::ffi::c_int != 0 {
                SLF_WRAP as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if wrap {
            let mut current_row: ::core::ffi::c_int = wlv.row;
            let mut dummy_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut current_grid: *mut ScreenGrid =
                grid_adjust(grid, &raw mut current_row, &raw mut dummy_col);
            *(*current_grid).attrs.offset(
                *(*current_grid)
                    .line_offset
                    .offset((current_row + 1 as ::core::ffi::c_int) as isize)
                    as isize,
            ) = -1 as ::core::ffi::c_int as sattr_T;
        }
        wlv.boguscols = 0 as ::core::ffi::c_int;
        wlv.vcol_off_co = 0 as ::core::ffi::c_int;
        wlv.row += 1;
        if !is_wrapped && wlv.filler_todo <= 0 as ::core::ffi::c_int {
            break;
        }
        if wlv.col <= leftcols_width {
            win_draw_end(
                wp,
                '@' as ::core::ffi::c_int as schar_T,
                true_0 != 0,
                wlv.row,
                (*wp).w_view_height,
                HLF_AT,
            );
            set_empty_rows(wp, wlv.row);
            wlv.row = endrow;
        }
        if wlv.row == endrow {
            wlv.row += 1;
            break;
        } else {
            win_line_start(wp, &raw mut wlv);
            draw_cols = true_0 != 0;
            lcs_prec_todo = (*wp).w_p_lcs_chars.prec;
            if wlv.filler_todo <= 0 as ::core::ffi::c_int {
                wlv.need_showbreak = true_0 != 0;
            }
            if statuscol.draw as ::core::ffi::c_int != 0
                && !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
                && wlv.row > startrow + wlv.filler_lines
            {
                statuscol.draw = false_0 != 0;
            }
            wlv.filler_todo -= 1;
            virt_line_index = -1 as ::core::ffi::c_int;
            virt_line_flags = 0 as ::core::ffi::c_int;
            if wlv.filler_todo == 0 as ::core::ffi::c_int
                && ((*wp).w_botfill as ::core::ffi::c_int != 0 || !draw_text)
            {
                break;
            }
        }
    }
    clear_virttext(&raw mut fold_vt);
    xfree(virt_lines.items as *mut ::core::ffi::c_void);
    virt_lines.capacity = 0 as size_t;
    virt_lines.size = virt_lines.capacity;
    virt_lines.items = ::core::ptr::null_mut::<virt_line>();
    xfree(foldtext_free as *mut ::core::ffi::c_void);
    return wlv.row;
}
pub const SPWORDLEN: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
unsafe extern "C" fn wlv_put_linebuf(
    mut wp: *mut win_T,
    mut wlv: *const winlinevars_T,
    mut endcol: ::core::ffi::c_int,
    mut clear_end: bool,
    mut bg_attr: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut grid: *mut GridView = &raw mut (*wp).w_grid;
    let mut startcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut clear_width: ::core::ffi::c_int = if clear_end as ::core::ffi::c_int != 0 {
        (*wp).w_view_width
    } else {
        endcol
    };
    '_c2rust_label: {
        if flags & SLF_RIGHTLEFT as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"!(flags & SLF_RIGHTLEFT)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3253 as ::core::ffi::c_uint,
                b"void wlv_put_linebuf(win_T *, const winlinevars_T *, int, _Bool, int, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*wp).w_onebuf_opt.wo_rl != 0 {
        linebuf_mirror(
            &raw mut startcol,
            &raw mut endcol,
            &raw mut clear_width,
            (*wp).w_view_width,
        );
        flags |= SLF_RIGHTLEFT as ::core::ffi::c_int;
    }
    if (*wlv).row == 0 as ::core::ffi::c_int
        && (*wp).w_skipcol > 0 as ::core::ffi::c_int
        && *get_showbreak_value(wp) as ::core::ffi::c_int == NUL
        && !((*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0 as schar_T)
    {
        let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0 {
            while off < (*wp).w_view_width
                && ascii_isdigit(schar_get_ascii(*(*linebuf_char.ptr()).offset(off as isize))
                    as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0
            {
                off += 1;
            }
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 3 as ::core::ffi::c_int && off < (*wp).w_view_width {
            if (off + 1 as ::core::ffi::c_int) < (*wp).w_view_width
                && *(*linebuf_char.ptr()).offset((off + 1 as ::core::ffi::c_int) as isize)
                    == NUL as schar_T
            {
                *(*linebuf_char.ptr()).offset((off + 1 as ::core::ffi::c_int) as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
            }
            *(*linebuf_char.ptr()).offset(off as isize) = '<' as ::core::ffi::c_int as schar_T;
            *(*linebuf_attr.ptr()).offset(off as isize) =
                *(*hl_attr_active.ptr()).offset(HLF_AT as ::core::ffi::c_int as isize) as sattr_T;
            off += 1;
            i += 1;
        }
    }
    let mut row: ::core::ffi::c_int = (*wlv).row;
    let mut coloff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g: *mut ScreenGrid = grid_adjust(grid, &raw mut row, &raw mut coloff);
    grid_put_linebuf(
        g,
        row,
        coloff,
        startcol,
        endcol,
        clear_width,
        bg_attr,
        0 as ::core::ffi::c_int,
        (*wlv).vcol - 1 as colnr_T,
        flags,
    );
}
unsafe extern "C" fn decor_providers_setup(
    mut rows_to_draw: ::core::ffi::c_int,
    mut draw_from_line_start: bool,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut wp: *mut win_T,
) -> ::core::ffi::c_int {
    let mut rem_vcols: ::core::ffi::c_int = 0;
    if (*wp).w_onebuf_opt.wo_wrap != 0 {
        let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width + win_col_off2(wp);
        let mut first_row_width: ::core::ffi::c_int =
            if draw_from_line_start as ::core::ffi::c_int != 0 {
                width
            } else {
                width2
            };
        rem_vcols = first_row_width + (rows_to_draw - 1 as ::core::ffi::c_int) * width2;
    } else {
        rem_vcols = (*wp).w_view_width - win_col_off(wp);
    }
    decor_providers_invoke_line(wp, lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    validate_virtcol(wp);
    return invoke_range_next(
        wp,
        lnum as ::core::ffi::c_int,
        col,
        rem_vcols as colnr_T + 1 as colnr_T,
    );
}
unsafe extern "C" fn invoke_range_next(
    mut wp: *mut win_T,
    mut lnum: ::core::ffi::c_int,
    mut begin_col: colnr_T,
    mut col_off: colnr_T,
) -> ::core::ffi::c_int {
    let line: *const ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum as linenr_T);
    let line_len: ::core::ffi::c_int = ml_get_buf_len((*wp).w_buffer, lnum as linenr_T);
    col_off = (if col_off > 1 as ::core::ffi::c_int {
        col_off as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    }) as colnr_T;
    let mut new_col: colnr_T = 0;
    if col_off <= line_len as colnr_T - begin_col {
        let mut end_col: ::core::ffi::c_int =
            begin_col as ::core::ffi::c_int + col_off as ::core::ffi::c_int;
        end_col += mb_off_next(line, line.offset(end_col as isize));
        decor_providers_invoke_range(
            wp,
            lnum - 1 as ::core::ffi::c_int,
            begin_col as ::core::ffi::c_int,
            lnum - 1 as ::core::ffi::c_int,
            end_col,
        );
        validate_virtcol(wp);
        new_col = end_col as colnr_T;
    } else {
        decor_providers_invoke_range(
            wp,
            lnum - 1 as ::core::ffi::c_int,
            begin_col as ::core::ffi::c_int,
            lnum,
            0 as ::core::ffi::c_int,
        );
        validate_virtcol(wp);
        new_col = INT_MAX as colnr_T;
    }
    return new_col as ::core::ffi::c_int;
}
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
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
