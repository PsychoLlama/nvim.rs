extern "C" {
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strnequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char, n: size_t) -> bool;
    static mut last_cursormoved_win: *mut win_T;
    static mut last_cursormoved: pos_T;
    static mut did_cursorhold: bool;
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn change_warning(buf: *mut buf_T, col: ::core::ffi::c_int);
    fn changed_bytes(lnum: linenr_T, col: colnr_T);
    fn inserted_bytes(
        lnum: linenr_T,
        start_col: colnr_T,
        old_col: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn appended_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn ins_bytes_len(p: *mut ::core::ffi::c_char, len: size_t);
    fn ins_char(c: ::core::ffi::c_int);
    fn ins_char_bytes(buf: *mut ::core::ffi::c_char, charlen: size_t);
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
    static mut p_deco: ::core::ffi::c_int;
    static mut p_ch: OptInt;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut fdo_flags: ::core::ffi::c_uint;
    static mut p_langmap: *mut ::core::ffi::c_char;
    static mut p_lrm: ::core::ffi::c_int;
    static mut p_paste: ::core::ffi::c_int;
    static mut p_ari: ::core::ffi::c_int;
    static mut p_ri: ::core::ffi::c_int;
    static mut p_smd: ::core::ffi::c_int;
    static mut p_sta: ::core::ffi::c_int;
    static mut p_sol: ::core::ffi::c_int;
    static mut p_ww: *mut ::core::ffi::c_char;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn byte2cells(b: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn hex2nr(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getviscol() -> ::core::ffi::c_int;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn inc_cursor() -> ::core::ffi::c_int;
    fn dec_cursor() -> ::core::ffi::c_int;
    fn check_cursor_col(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn check_visual_pos();
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn char_before_cursor() -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn get_cursor_pos_len() -> colnr_T;
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor_0: bool) -> bool;
    fn win_lines_concealed(wp: *mut win_T) -> bool;
    fn redrawing() -> bool;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn show_cursor_info_later(force: bool);
    fn skip_showmode() -> bool;
    fn showmode() -> ::core::ffi::c_int;
    fn unshowmode(force: bool);
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redrawWinline(wp: *mut win_T, lnum: linenr_T);
    fn status_redraw_curbuf();
    fn redraw_statuslines();
    fn do_digraph(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn digraph_get(
        char1: ::core::ffi::c_int,
        char2: ::core::ffi::c_int,
        meta_char: bool,
    ) -> ::core::ffi::c_int;
    static e_noinstext: [::core::ffi::c_char; 0];
    static e_sandbox: [::core::ffi::c_char; 0];
    static e_textlock: [::core::ffi::c_char; 0];
    fn prompt_invoke_callback();
    fn invoke_prompt_interrupt() -> bool;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn expr_map_locked() -> bool;
    fn check_timestamps(focus: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static mut disable_fold_update: ::core::ffi::c_int;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn hasFoldingWin(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
        cache: bool,
        infop: *mut foldinfo_T,
    ) -> bool;
    fn foldOpenCursor();
    fn foldCheckClose();
    fn foldUpdateAfterInsert();
    static mut test_disable_char_avail: bool;
    fn get_inserted() -> String_0;
    fn stuff_empty() -> bool;
    fn ResetRedobuff();
    fn AppendToRedobuff(s: *const ::core::ffi::c_char);
    fn AppendToRedobuffLit(str: *const ::core::ffi::c_char, len: ::core::ffi::c_int);
    fn AppendCharToRedobuff(c: ::core::ffi::c_int);
    fn AppendNumberToRedobuff(n: ::core::ffi::c_int);
    fn stuffRedoReadbuff(s: *const ::core::ffi::c_char);
    fn stuffReadbuffLen(s: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn start_redo_ins() -> ::core::ffi::c_int;
    fn stop_redo_ins();
    fn typebuf_maplen() -> ::core::ffi::c_int;
    fn merge_modifiers(
        c_arg: ::core::ffi::c_int,
        modifiers: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vgetc() -> ::core::ffi::c_int;
    fn plain_vgetc() -> ::core::ffi::c_int;
    fn vpeekc() -> ::core::ffi::c_int;
    fn char_avail() -> bool;
    fn vungetc(c: ::core::ffi::c_int);
    fn getcmdkeycmd(
        promptc: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn map_execute_lua(may_repeat: bool, discard: bool) -> bool;
    fn paste_repeat(count: ::core::ffi::c_int);
    static mut mod_mask: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut redraw_mode: bool;
    static mut clear_cmdline: bool;
    static mut dollar_vcol: colnr_T;
    static mut edit_submode_extra: *mut ::core::ffi::c_char;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut emsg_on_display: bool;
    static mut vgetc_busy: ::core::ffi::c_int;
    static mut need_check_timestamps: bool;
    static mut did_check_timestamps: bool;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut textlock: ::core::ffi::c_int;
    static mut sandbox: ::core::ffi::c_int;
    static mut VIsual_active: bool;
    static mut restart_VIsual_select: ::core::ffi::c_int;
    static mut where_paste_started: pos_T;
    static mut did_ai: bool;
    static mut ai_col: colnr_T;
    static mut end_comment_pending: ::core::ffi::c_int;
    static mut did_si: bool;
    static mut can_si: bool;
    static mut can_si_back: bool;
    static mut old_indent: ::core::ffi::c_int;
    static mut Insstart: pos_T;
    static mut Insstart_orig: pos_T;
    static mut orig_line_count: linenr_T;
    static mut vr_lines_changed: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut reg_recording: ::core::ffi::c_int;
    static mut no_mapping: ::core::ffi::c_int;
    static mut allow_keys: ::core::ffi::c_int;
    static mut no_u_sync: ::core::ffi::c_int;
    static mut u_sync_once: ::core::ffi::c_int;
    static mut force_restart_edit: bool;
    static mut restart_edit: ::core::ffi::c_int;
    static mut arrow_used: bool;
    static mut ins_at_eol: bool;
    static mut no_abbr: bool;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut ex_normal_busy: ::core::ffi::c_int;
    static mut stop_insert_mode: bool;
    static mut KeyTyped: bool;
    static mut KeyStuffed: ::core::ffi::c_int;
    static mut must_redraw: ::core::ffi::c_int;
    static mut need_highlight_changed: bool;
    static mut got_int: bool;
    static mut need_start_insertmode: bool;
    static mut replace_offset: ::core::ffi::c_int;
    static mut langmap_mapchar: [uint8_t; 256];
    static mut km_startsel: bool;
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut cmdwin_result: ::core::ffi::c_int;
    static mut spell_redraw_lnum: linenr_T;
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn grid_line_getchar(col: ::core::ffi::c_int, attr: *mut ::core::ffi::c_int) -> schar_T;
    fn grid_line_put_schar(col: ::core::ffi::c_int, schar: schar_T, attr: ::core::ffi::c_int);
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_flush();
    static mut default_grid: ScreenGrid;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn highlight_changed();
    fn tabstop_padding(col: colnr_T, ts_arg: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int;
    fn tabstop_at(col: colnr_T, ts: OptInt, vts: *const colnr_T, left: bool) -> ::core::ffi::c_int;
    fn tabstop_start(col: colnr_T, ts: ::core::ffi::c_int, vts: *mut colnr_T) -> colnr_T;
    fn tabstop_count(ts: *mut colnr_T) -> ::core::ffi::c_int;
    fn tabstop_first(ts: *mut colnr_T) -> ::core::ffi::c_int;
    fn get_sw_value(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn get_sts_value() -> ::core::ffi::c_int;
    fn get_indent() -> ::core::ffi::c_int;
    fn inindent(extra: ::core::ffi::c_int) -> bool;
    fn may_do_si() -> bool;
    fn ins_try_si(c: ::core::ffi::c_int);
    fn change_indent(
        type_0: ::core::ffi::c_int,
        amount: ::core::ffi::c_int,
        round: ::core::ffi::c_int,
        call_changed_bytes: bool,
    );
    fn fix_indent();
    fn ins_ctrl_x();
    fn ctrl_x_mode_none() -> bool;
    fn ctrl_x_mode_normal() -> bool;
    fn ctrl_x_mode_scroll() -> bool;
    fn ctrl_x_mode_whole_line() -> bool;
    fn ctrl_x_mode_files() -> bool;
    fn ctrl_x_mode_tags() -> bool;
    fn ctrl_x_mode_path_patterns() -> bool;
    fn ctrl_x_mode_path_defines() -> bool;
    fn ctrl_x_mode_dictionary() -> bool;
    fn ctrl_x_mode_thesaurus() -> bool;
    fn ctrl_x_mode_cmdline() -> bool;
    fn ctrl_x_mode_function() -> bool;
    fn ctrl_x_mode_omni() -> bool;
    fn ctrl_x_mode_spell() -> bool;
    fn ctrl_x_mode_line_or_eval() -> bool;
    fn ctrl_x_mode_register() -> bool;
    fn compl_status_local() -> bool;
    fn compl_status_clear();
    fn check_compl_option(dict_opt: bool) -> bool;
    fn ins_compl_accept_char(c: ::core::ffi::c_int) -> bool;
    fn ins_compl_is_match_selected() -> bool;
    fn ins_compl_preinsert_longest() -> bool;
    fn ins_compl_has_shown_match() -> bool;
    fn ins_compl_long_shown_match() -> bool;
    fn pum_wanted() -> bool;
    fn ins_compl_clear();
    fn ins_compl_active() -> bool;
    fn ins_compl_win_active(wp: *mut win_T) -> bool;
    fn ins_compl_used_match() -> bool;
    fn ins_compl_init_get_longest();
    fn ins_compl_enter_selects() -> bool;
    fn ins_compl_col() -> colnr_T;
    fn ins_compl_preinsert_effect() -> bool;
    fn ins_compl_bs() -> ::core::ffi::c_int;
    fn ins_compl_has_autocomplete() -> bool;
    fn ins_compl_addleader(c: ::core::ffi::c_int);
    fn ins_compl_addfrommatch();
    fn ins_compl_cancel() -> bool;
    fn ins_compl_prep(c: ::core::ffi::c_int) -> bool;
    fn ins_compl_delete(new_leader: bool);
    fn ins_compl_insert(move_cursor: bool, insert_prefix: bool);
    fn ins_complete(c: ::core::ffi::c_int, enable_pum: bool) -> ::core::ffi::c_int;
    fn ins_compl_enable_autocomplete();
    fn cindent_on() -> bool;
    fn in_cinkeys(
        keytyped: ::core::ffi::c_int,
        when: ::core::ffi::c_int,
        line_is_empty: bool,
    ) -> bool;
    fn do_c_expr_indent();
    fn get_special_key_name(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn add_char2buf(c: ::core::ffi::c_int, s: *mut ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
    fn map_to_exists_mode(
        rhs: *const ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        abbr: bool,
    ) -> bool;
    fn check_abbr(
        c: ::core::ffi::c_int,
        ptr: *mut ::core::ffi::c_char,
        col: ::core::ffi::c_int,
        mincol: ::core::ffi::c_int,
    ) -> bool;
    fn langmap_adjust_mb(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_time() -> Timestamp;
    fn free_fmark(fm: fmark_T);
    fn mark_view_make(wp: *const win_T, pos: pos_T) -> fmarkv_T;
    static utf8len_tab: [uint8_t; 256];
    fn mb_get_class(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2CharInfo_impl(p: *const uint8_t, len: uintptr_t) -> int32_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_composinglike(
        p1: *const ::core::ffi::c_char,
        p2: *const ::core::ffi::c_char,
        state: *mut GraphemeState,
    ) -> bool;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utfc_next_impl(cur: StrCharInfo) -> StrCharInfo;
    fn mb_adjust_cursor();
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int;
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace(lnum: linenr_T, line: *mut ::core::ffi::c_char, copy: bool)
        -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn ins_mouse(c: ::core::ffi::c_int);
    fn ins_mousescroll(dir: ::core::ffi::c_int);
    fn setmouse();
    fn update_topline(wp: *mut win_T);
    fn update_curswant();
    fn set_topline(wp: *mut win_T, lnum: linenr_T);
    fn validate_cursor(wp: *mut win_T);
    fn validate_virtcol(wp: *mut win_T);
    fn validate_cursor_col(wp: *mut win_T);
    fn curs_columns(wp: *mut win_T, may_scroll: ::core::ffi::c_int);
    fn adjust_skipcol();
    fn scrolldown_clamp();
    fn scrollup_clamp();
    fn pagescroll(dir: Direction, count: ::core::ffi::c_int, half: bool) -> ::core::ffi::c_int;
    fn do_check_cursorbind();
    fn end_visual_mode();
    fn clear_showcmd();
    fn add_to_showcmd(c: ::core::ffi::c_int) -> bool;
    fn add_to_showcmd_c(c: ::core::ffi::c_int);
    fn do_check_scrollbind(check: bool);
    fn start_selection();
    fn do_join(
        count: size_t,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark: bool,
    ) -> ::core::ffi::c_int;
    fn set_iminsert_global(buf: *mut buf_T);
    fn can_bs(what: ::core::ffi::c_int) -> bool;
    fn get_ve_flags(wp: *mut win_T) -> ::core::ffi::c_uint;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn get_scrolloff_value(wp: *mut win_T) -> int64_t;
    fn line_breakcheck();
    fn win_chartabsize(
        wp: *mut win_T,
        p: *mut ::core::ffi::c_char,
        col: colnr_T,
    ) -> ::core::ffi::c_int;
    fn linetabsize_col(
        startvcol: ::core::ffi::c_int,
        s: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
    fn charsize_nowrap(
        buf: *mut buf_T,
        cur: *const ::core::ffi::c_char,
        use_tabstop: bool,
        vcol: colnr_T,
        cur_char: int32_t,
    ) -> ::core::ffi::c_int;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvcol_nolist(posp: *mut pos_T) -> colnr_T;
    fn pum_check_clear();
    fn pum_visible() -> bool;
    static mut pum_want: C2Rust_Unnamed_27;
    fn get_expr_register() -> ::core::ffi::c_int;
    fn valid_yank_reg(regname: ::core::ffi::c_int, writing: bool) -> bool;
    fn get_yank_register(regname: ::core::ffi::c_int, mode: ::core::ffi::c_int) -> *mut yankreg_T;
    fn insert_reg(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        literally_arg: bool,
    ) -> ::core::ffi::c_int;
    fn do_put(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    );
    fn state_enter(s: *mut VimState);
    fn state_handle_k_event();
    fn virtual_active(wp: *mut win_T) -> bool;
    fn may_trigger_modechanged();
    fn may_trigger_safestate(safe: bool);
    fn syntax_present(win: *mut win_T) -> bool;
    fn terminal_enter() -> bool;
    fn fwd_word(count: ::core::ffi::c_int, bigword: bool, eol: bool) -> ::core::ffi::c_int;
    fn bck_word(count: ::core::ffi::c_int, bigword: bool, stop: bool) -> ::core::ffi::c_int;
    fn has_format_option(x: ::core::ffi::c_int) -> bool;
    fn internal_format(
        textwidth: ::core::ffi::c_int,
        second_indent: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
        format_only: bool,
        c: ::core::ffi::c_int,
    );
    fn auto_format(trailblank: bool, prev_line: bool);
    fn check_auto_format(end_insert: bool);
    fn comp_textwidth(ff: bool) -> ::core::ffi::c_int;
    fn fex_format(
        lnum: linenr_T,
        count: ::core::ffi::c_long,
        c: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_flush();
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn u_save_cursor() -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn u_sync(force: bool);
    fn u_clearallandblockfree(buf: *mut buf_T);
    fn goto_tabpage(n: ::core::ffi::c_int);
    fn may_trigger_win_scrolled_resized();
}
pub type __time_t = ::core::ffi::c_long;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintptr_t = usize;
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_PRE: C2Rust_Unnamed_15 = 75;
pub const HLF_OK: C2Rust_Unnamed_15 = 74;
pub const HLF_SO: C2Rust_Unnamed_15 = 73;
pub const HLF_SE: C2Rust_Unnamed_15 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_15 = 71;
pub const HLF_TS: C2Rust_Unnamed_15 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_15 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_15 = 68;
pub const HLF_CU: C2Rust_Unnamed_15 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_15 = 66;
pub const HLF_WBR: C2Rust_Unnamed_15 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_15 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_0: C2Rust_Unnamed_15 = 59;
pub const HLF_QFL: C2Rust_Unnamed_15 = 58;
pub const HLF_MC: C2Rust_Unnamed_15 = 57;
pub const HLF_CUL: C2Rust_Unnamed_15 = 56;
pub const HLF_CUC: C2Rust_Unnamed_15 = 55;
pub const HLF_TPF: C2Rust_Unnamed_15 = 54;
pub const HLF_TPS: C2Rust_Unnamed_15 = 53;
pub const HLF_TP: C2Rust_Unnamed_15 = 52;
pub const HLF_PBR: C2Rust_Unnamed_15 = 51;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PSB: C2Rust_Unnamed_15 = 49;
pub const HLF_PSX: C2Rust_Unnamed_15 = 48;
pub const HLF_PNX: C2Rust_Unnamed_15 = 47;
pub const HLF_PSK: C2Rust_Unnamed_15 = 46;
pub const HLF_PNK: C2Rust_Unnamed_15 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_15 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_15 = 43;
pub const HLF_PSI: C2Rust_Unnamed_15 = 42;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_SPL: C2Rust_Unnamed_15 = 40;
pub const HLF_SPR: C2Rust_Unnamed_15 = 39;
pub const HLF_SPC: C2Rust_Unnamed_15 = 38;
pub const HLF_SPB: C2Rust_Unnamed_15 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_15 = 36;
pub const HLF_SC: C2Rust_Unnamed_15 = 35;
pub const HLF_TXA: C2Rust_Unnamed_15 = 34;
pub const HLF_TXD: C2Rust_Unnamed_15 = 33;
pub const HLF_DED: C2Rust_Unnamed_15 = 32;
pub const HLF_CHD: C2Rust_Unnamed_15 = 31;
pub const HLF_ADD: C2Rust_Unnamed_15 = 30;
pub const HLF_FC: C2Rust_Unnamed_15 = 29;
pub const HLF_FL: C2Rust_Unnamed_15 = 28;
pub const HLF_WM: C2Rust_Unnamed_15 = 27;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_VNC: C2Rust_Unnamed_15 = 25;
pub const HLF_V: C2Rust_Unnamed_15 = 24;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_VSP: C2Rust_Unnamed_15 = 22;
pub const HLF_C: C2Rust_Unnamed_15 = 21;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_R: C2Rust_Unnamed_15 = 18;
pub const HLF_CLF: C2Rust_Unnamed_15 = 17;
pub const HLF_CLS: C2Rust_Unnamed_15 = 16;
pub const HLF_CLN: C2Rust_Unnamed_15 = 15;
pub const HLF_LNB: C2Rust_Unnamed_15 = 14;
pub const HLF_LNA: C2Rust_Unnamed_15 = 13;
pub const HLF_N: C2Rust_Unnamed_15 = 12;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_M: C2Rust_Unnamed_15 = 10;
pub const HLF_LC: C2Rust_Unnamed_15 = 9;
pub const HLF_L: C2Rust_Unnamed_15 = 8;
pub const HLF_I: C2Rust_Unnamed_15 = 7;
pub const HLF_E: C2Rust_Unnamed_15 = 6;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_AT: C2Rust_Unnamed_15 = 4;
pub const HLF_TERM: C2Rust_Unnamed_15 = 3;
pub const HLF_EOB: C2Rust_Unnamed_15 = 2;
pub const HLF_8: C2Rust_Unnamed_15 = 1;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
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
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
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
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
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
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_18 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_18 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_18 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_18 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_18 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_18 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_19 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_19 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_19 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_19 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_19 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_19 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_19 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_19 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_19 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_19 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_19 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_19 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_19 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_19 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_19 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_19 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_19 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_19 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_19 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_20 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_20 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_20 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_20 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_20 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_20 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_20 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_20 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_20 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_20 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_21 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_21 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_21 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_21 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_21 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_21 = 4;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_22 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_22 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_22 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_22 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_22 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_22 = 20;
pub const UPD_VALID: C2Rust_Unnamed_22 = 10;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const INDENT_DEC: C2Rust_Unnamed_23 = 3;
pub const INDENT_INC: C2Rust_Unnamed_23 = 2;
pub const INDENT_SET: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_24 = 4;
pub const BL_SOL: C2Rust_Unnamed_24 = 2;
pub const BL_WHITE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const INSCHAR_COM_LIST: C2Rust_Unnamed_25 = 16;
pub const INSCHAR_NO_FEX: C2Rust_Unnamed_25 = 8;
pub const INSCHAR_CTRLV: C2Rust_Unnamed_25 = 4;
pub const INSCHAR_DO_COM: C2Rust_Unnamed_25 = 2;
pub const INSCHAR_FORMAT: C2Rust_Unnamed_25 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InsertState {
    pub state: VimState,
    pub ca: *mut cmdarg_T,
    pub mincol: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub cmdchar_todo: ::core::ffi::c_int,
    pub ins_just_started: bool,
    pub startln: ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub lastc: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
    pub did_backspace: bool,
    pub line_is_white: bool,
    pub old_topline: linenr_T,
    pub old_topfill: ::core::ffi::c_int,
    pub inserted_space: ::core::ffi::c_int,
    pub replaceState: ::core::ffi::c_int,
    pub did_restart_edit: ::core::ffi::c_int,
    pub nomove: bool,
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
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type VimState = vim_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_state {
    pub check: state_check_callback,
    pub execute: state_execute_callback,
}
pub type state_execute_callback =
    Option<unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int>;
pub type state_check_callback = Option<unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int>;
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
pub const MODE_NORMAL: C2Rust_Unnamed_29 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub const REPLACE_FLAG: C2Rust_Unnamed_29 = 256;
pub const MODE_INSERT: C2Rust_Unnamed_29 = 16;
pub const MODE_LANGMAP: C2Rust_Unnamed_29 = 32;
pub const MODE_VREPLACE: C2Rust_Unnamed_29 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_29 = 272;
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
pub const VREPLACE_FLAG: C2Rust_Unnamed_29 = 512;
pub const KE_EVENT: key_extra = 102;
pub const KE_NOP: key_extra = 97;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
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
pub type CSType = bool;
pub const kCharsizeFast: C2Rust_Unnamed_33 = 1;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_27 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const KE_LUA: key_extra = 103;
pub const KE_COMMAND: key_extra = 104;
pub const KE_IGNORE: key_extra = 53;
pub const MSCR_RIGHT: C2Rust_Unnamed_30 = -2;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const MSCR_LEFT: C2Rust_Unnamed_30 = -1;
pub const KE_MOUSELEFT: key_extra = 77;
pub const MSCR_UP: C2Rust_Unnamed_30 = 1;
pub const KE_MOUSEUP: key_extra = 76;
pub const MSCR_DOWN: C2Rust_Unnamed_30 = 0;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const BACKSPACE_LINE: C2Rust_Unnamed_34 = 4;
pub const BACKSPACE_CHAR: C2Rust_Unnamed_34 = 1;
pub type GraphemeState = utf8proc_int32_t;
pub type utf8proc_int32_t = int32_t;
pub const BACKSPACE_WORD_NOT_SPACE: C2Rust_Unnamed_34 = 3;
pub const BACKSPACE_WORD: C2Rust_Unnamed_34 = 2;
pub const KE_KDEL: key_extra = 80;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub const PUT_CURSEND: C2Rust_Unnamed_31 = 2;
pub const YREG_PASTE: C2Rust_Unnamed_32 = 0;
pub const PUT_FIXINDENT: C2Rust_Unnamed_31 = 1;
pub const KE_XF1: key_extra = 57;
pub const KE_KINS: key_extra = 79;
pub const MODE_CMDLINE: C2Rust_Unnamed_29 = 8;
pub const MB_MAXBYTES: C2Rust_Unnamed_28 = 21;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_28 = 6;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_29 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_29 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_29 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_29 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_29 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_29 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_29 = 288;
pub const MAP_ALL_MODES: C2Rust_Unnamed_29 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_29 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_29 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_29 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_29 = 2;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_DROP: key_extra = 95;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
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
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
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
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_31 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_31 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_31 = 16;
pub const PUT_LINE: C2Rust_Unnamed_31 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_31 = 4;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_32 = 2;
pub const YREG_YANK: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_26 = C2Rust_Unnamed_26 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ML_LINE_DIRTY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const ML_ALLOCATED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const NL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\n\0") };
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const ESC_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x1B\0") };
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const CTRL_V_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x16\0") };
pub const Ctrl_A: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const Ctrl_E: ::core::ffi::c_int = 5;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_J: ::core::ffi::c_int = 10;
pub const Ctrl_K: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16;
pub const Ctrl_Q: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_S: ::core::ffi::c_int = 19;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const Ctrl_HAT: ::core::ffi::c_int = 30;
pub const Ctrl__: ::core::ffi::c_int = 31;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_iswhite_nl_or_nul(mut c: ::core::ffi::c_int) -> bool {
    return ascii_iswhite(c) as ::core::ffi::c_int != 0
        || c == '\n' as ::core::ffi::c_int
        || c == NUL;
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
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const FO_RET_COMS: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const FO_INS_LONG: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const CPO_INDENT: ::core::ffi::c_int = 'I' as ::core::ffi::c_int;
pub const CPO_LISTWM: ::core::ffi::c_int = 'L' as ::core::ffi::c_int;
pub const CPO_BACKSPACE: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const CPO_REPLCNT: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const BS_INDENT: ::core::ffi::c_int = 'i' as ::core::ffi::c_int;
pub const BS_EOL: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const BS_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const BS_NOSTOP: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
static mut compl_busy: bool = false_0 != 0;
static mut Insstart_textlen: colnr_T = 0;
static mut Insstart_blank_vcol: colnr_T = 0;
static mut update_Insstart_orig: bool = true_0 != 0;
static mut last_insert: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
static mut last_insert_skip: ::core::ffi::c_int = 0;
static mut new_insert_skip: ::core::ffi::c_int = 0;
static mut did_restart_edit: ::core::ffi::c_int = 0;
static mut can_cindent: bool = false;
static mut revins_on: bool = false;
static mut revins_chars: ::core::ffi::c_int = 0;
static mut revins_legal: ::core::ffi::c_int = 0;
static mut revins_scol: ::core::ffi::c_int = 0;
static mut ins_need_undo: bool = false;
static mut dont_sync_undo: TriState = kFalse;
static mut o_lnum: linenr_T = 0 as linenr_T;
static mut replace_stack: C2Rust_Unnamed_26 = KV_INITIAL_VALUE;
unsafe extern "C" fn insert_enter(mut s: *mut InsertState) {
    (*s).did_backspace = true_0 != 0;
    (*s).old_topfill = -1 as ::core::ffi::c_int;
    (*s).replaceState = MODE_REPLACE as ::core::ffi::c_int;
    (*s).cmdchar_todo = (*s).cmdchar;
    (*s).ins_just_started = true_0 != 0;
    did_restart_edit = restart_edit;
    msg_check_for_delay(true_0 != 0);
    update_Insstart_orig = true_0 != 0;
    ins_compl_clear();
    if (*s).cmdchar != 'r' as ::core::ffi::c_int && (*s).cmdchar != 'v' as ::core::ffi::c_int {
        let mut save_cursor: pos_T = (*curwin).w_cursor;
        let ptr: *const ::core::ffi::c_char = if (*s).cmdchar == 'R' as ::core::ffi::c_int {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        } else if (*s).cmdchar == 'V' as ::core::ffi::c_int {
            b"v\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        };
        set_vim_var_string(VV_INSERTMODE, ptr, 1 as ptrdiff_t);
        set_vim_var_string(
            VV_CHAR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        ins_apply_autocmds(EVENT_INSERTENTER);
        if need_highlight_changed {
            highlight_changed();
        }
        if !equalpos((*curwin).w_cursor, save_cursor)
            && *get_vim_var_str(VV_CHAR) as ::core::ffi::c_int == NUL
            && save_cursor.lnum <= (*curbuf).b_ml.ml_line_count
        {
            let mut save_state: ::core::ffi::c_int = State;
            (*curwin).w_cursor = save_cursor;
            State = MODE_INSERT as ::core::ffi::c_int;
            check_cursor_col(curwin);
            State = save_state;
        }
    }
    if where_paste_started.lnum != 0 as linenr_T {
        Insstart = where_paste_started;
    } else {
        Insstart = (*curwin).w_cursor;
        if (*s).startln != 0 {
            Insstart.col = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
    Insstart_textlen = linetabsize_str(get_cursor_line_ptr()) as colnr_T;
    Insstart_blank_vcol = MAXCOL as ::core::ffi::c_int as colnr_T;
    if !did_ai {
        ai_col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*s).cmdchar != NUL && restart_edit == 0 as ::core::ffi::c_int {
        ResetRedobuff();
        AppendNumberToRedobuff((*s).count);
        if (*s).cmdchar == 'V' as ::core::ffi::c_int || (*s).cmdchar == 'v' as ::core::ffi::c_int {
            AppendCharToRedobuff('g' as ::core::ffi::c_int);
            AppendCharToRedobuff(if (*s).cmdchar == 'v' as ::core::ffi::c_int {
                'r' as ::core::ffi::c_int
            } else {
                'R' as ::core::ffi::c_int
            });
        } else {
            AppendCharToRedobuff((*s).cmdchar);
            if (*s).cmdchar == 'g' as ::core::ffi::c_int {
                AppendCharToRedobuff('I' as ::core::ffi::c_int);
            } else if (*s).cmdchar == 'r' as ::core::ffi::c_int {
                (*s).count = 1 as ::core::ffi::c_int;
            }
        }
    }
    if (*s).cmdchar == 'R' as ::core::ffi::c_int {
        State = MODE_REPLACE as ::core::ffi::c_int;
    } else if (*s).cmdchar == 'V' as ::core::ffi::c_int || (*s).cmdchar == 'v' as ::core::ffi::c_int
    {
        State = MODE_VREPLACE as ::core::ffi::c_int;
        (*s).replaceState = MODE_VREPLACE as ::core::ffi::c_int;
        orig_line_count = (*curbuf).b_ml.ml_line_count;
        vr_lines_changed = 1 as ::core::ffi::c_int;
    } else {
        State = MODE_INSERT as ::core::ffi::c_int;
    }
    may_trigger_modechanged();
    stop_insert_mode = false_0 != 0;
    if gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0 as uint32_t {
        (*curwin).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
    if (*curbuf).b_p_iminsert == B_IMODE_LMAP as OptInt {
        State |= MODE_LANGMAP as ::core::ffi::c_int;
    }
    setmouse();
    clear_showcmd();
    revins_on = State == MODE_INSERT as ::core::ffi::c_int && p_ri != 0;
    if revins_on {
        undisplay_dollar();
    }
    revins_chars = 0 as ::core::ffi::c_int;
    revins_legal = 0 as ::core::ffi::c_int;
    revins_scol = -1 as ::core::ffi::c_int;
    if restart_edit != 0 as ::core::ffi::c_int && stuff_empty() as ::core::ffi::c_int != 0 {
        arrow_used = where_paste_started.lnum == 0 as linenr_T;
        restart_edit = 0 as ::core::ffi::c_int;
        validate_virtcol(curwin);
        update_curswant();
        let mut ptr_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (ins_at_eol as ::core::ffi::c_int != 0 && (*curwin).w_cursor.lnum == o_lnum
            || (*curwin).w_curswant > (*curwin).w_virtcol)
            && {
                ptr_0 = get_cursor_line_ptr().offset((*curwin).w_cursor.col as isize);
                *ptr_0 as ::core::ffi::c_int != NUL
            }
        {
            if *ptr_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                (*curwin).w_cursor.col += 1;
            } else {
                (*s).i = utfc_ptr2len(ptr_0);
                if *ptr_0.offset((*s).i as isize) as ::core::ffi::c_int == NUL {
                    (*curwin).w_cursor.col += (*s).i;
                }
            }
        }
        ins_at_eol = false_0 != 0;
    } else {
        arrow_used = false_0 != 0;
    }
    need_start_insertmode = false_0 != 0;
    ins_need_undo = true_0 != 0;
    where_paste_started.lnum = 0 as ::core::ffi::c_int as linenr_T;
    can_cindent = true_0 != 0;
    if did_restart_edit == 0 as ::core::ffi::c_int {
        foldOpenCursor();
    }
    (*s).i = 0 as ::core::ffi::c_int;
    if p_smd != 0 && msg_silent == 0 as ::core::ffi::c_int {
        (*s).i = showmode();
    }
    if did_restart_edit == 0 as ::core::ffi::c_int {
        change_warning(
            curbuf,
            if (*s).i == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                (*s).i + 1 as ::core::ffi::c_int
            },
        );
    }
    ui_cursor_shape();
    do_digraph(-1 as ::core::ffi::c_int);
    let mut inserted: String_0 = get_inserted();
    new_insert_skip = inserted.size as ::core::ffi::c_int;
    if !inserted.data.is_null() {
        xfree(inserted.data as *mut ::core::ffi::c_void);
    }
    old_indent = 0 as ::core::ffi::c_int;
    loop {
        state_enter(&raw mut (*s).state);
        if ins_esc(&raw mut (*s).count, (*s).cmdchar, (*s).nomove) {
            break;
        }
    }
    if ins_at_eol {
        o_lnum = (*curwin).w_cursor.lnum;
    }
    pum_check_clear();
    foldUpdateAfterInsert();
    if (*s).cmdchar != 'r' as ::core::ffi::c_int
        && (*s).cmdchar != 'v' as ::core::ffi::c_int
        && (*s).c != Ctrl_C
    {
        ins_apply_autocmds(EVENT_INSERTLEAVE);
    }
    did_cursorhold = false_0 != 0;
    if !char_avail() && (*curbuf).b_last_changedtick_i == buf_get_changedtick(curbuf) {
        (*curbuf).b_last_changedtick = buf_get_changedtick(curbuf);
    }
}
unsafe extern "C" fn insert_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    let mut s: *mut InsertState = state as *mut InsertState;
    if revins_legal == 0 {
        revins_scol = -1 as ::core::ffi::c_int;
    } else {
        revins_legal = 0 as ::core::ffi::c_int;
    }
    if arrow_used {
        (*s).count = 0 as ::core::ffi::c_int;
    }
    if update_Insstart_orig {
        Insstart_orig = Insstart;
    }
    if !(*curbuf).terminal.is_null() && !stop_insert_mode {
        stop_insert_mode = true_0 != 0;
        restart_edit = 'I' as ::core::ffi::c_int;
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    }
    if stop_insert_mode as ::core::ffi::c_int != 0 && !ins_compl_active() {
        (*s).count = 0 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    }
    if !arrow_used {
        (*curwin).w_set_curswant = true_0;
    }
    if stuff_empty() {
        did_check_timestamps = false_0 != 0;
        if need_check_timestamps {
            check_timestamps(false_0);
        }
    }
    msg_scroll = false_0;
    if fdo_flags & kOptFdoFlagInsert as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        foldOpenCursor();
    }
    if !char_avail() {
        foldCheckClose();
    }
    if bt_prompt(curbuf) {
        init_prompt((*s).cmdchar_todo);
        (*s).cmdchar_todo = NUL;
    }
    if (*curbuf).b_mod_set as ::core::ffi::c_int != 0
        && (*curwin).w_onebuf_opt.wo_wrap != 0
        && (*curwin).w_onebuf_opt.wo_sms == 0
        && !(*s).did_backspace
        && (*curwin).w_topline == (*s).old_topline
        && (*curwin).w_topfill == (*s).old_topfill
        && (*s).count <= 1 as ::core::ffi::c_int
    {
        (*s).mincol = (*curwin).w_wcol;
        validate_cursor_col(curwin);
        if (*curwin).w_wcol
            < (*s).mincol
                - tabstop_at(
                    get_nolist_virtcol(),
                    (*curbuf).b_p_ts,
                    (*curbuf).b_p_vts_array,
                    false_0 != 0,
                )
            && (*curwin).w_wrow as int64_t
                == ((*curwin).w_view_height - 1 as ::core::ffi::c_int) as int64_t
                    - get_scrolloff_value(curwin)
            && ((*curwin).w_cursor.lnum != (*curwin).w_topline
                || (*curwin).w_topfill > 0 as ::core::ffi::c_int)
        {
            if (*curwin).w_topfill > 0 as ::core::ffi::c_int {
                (*curwin).w_topfill -= 1;
            } else if hasFolding(
                curwin,
                (*curwin).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*s).old_topline,
            ) {
                set_topline(curwin, (*s).old_topline + 1 as linenr_T);
            } else {
                set_topline(curwin, (*curwin).w_topline + 1 as linenr_T);
            }
        }
    }
    if (*s).count <= 1 as ::core::ffi::c_int {
        update_topline(curwin);
    }
    (*s).did_backspace = false_0 != 0;
    if (*s).count <= 1 as ::core::ffi::c_int {
        validate_cursor(curwin);
    }
    ins_redraw(true_0 != 0);
    if (*curwin).w_onebuf_opt.wo_scb != 0 {
        do_check_scrollbind(true_0 != 0);
    }
    if (*curwin).w_onebuf_opt.wo_crb != 0 {
        do_check_cursorbind();
    }
    if (*s).count <= 1 as ::core::ffi::c_int {
        update_curswant();
    }
    (*s).old_topline = (*curwin).w_topline;
    (*s).old_topfill = (*curwin).w_topfill;
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).lastc = (*s).c;
    }
    if dont_sync_undo as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        dont_sync_undo = kTrue;
    } else {
        dont_sync_undo = kFalse;
    }
    if (*s).ins_just_started {
        (*s).ins_just_started = false_0 != 0;
        if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
            && !char_avail()
            && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
        {
            (*s).c = char_before_cursor();
            if vim_isprintc((*s).c) {
                ins_compl_enable_autocomplete();
                ins_compl_init_get_longest();
                insert_do_complete(s);
                insert_handle_key_post(s);
                return 1 as ::core::ffi::c_int;
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn insert_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s: *mut InsertState = state as *mut InsertState;
    if stop_insert_mode {
        if key
            != -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            && key
                != -(253 as ::core::ffi::c_int
                    + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            vungetc(key);
        }
        (*s).count = 0 as ::core::ffi::c_int;
        (*s).nomove = true_0 != 0;
        ins_compl_prep(ESC);
        return 0 as ::core::ffi::c_int;
    }
    if key
        == -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || key
            == -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return -1 as ::core::ffi::c_int;
    }
    (*s).c = key;
    if key
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        did_cursorhold = true_0 != 0;
    }
    if ins_compl_active() as ::core::ffi::c_int != 0
        && (*curwin).w_cursor.col >= ins_compl_col()
        && ins_compl_has_shown_match() as ::core::ffi::c_int != 0
        && pum_wanted() as ::core::ffi::c_int != 0
    {
        if ((*s).c == K_BS || (*s).c == Ctrl_H) && (*curwin).w_cursor.col > ins_compl_col() && {
            (*s).c = ins_compl_bs();
            (*s).c == NUL
        } {
            return 1 as ::core::ffi::c_int;
        }
        if !ins_compl_used_match() {
            if (*s).c == Ctrl_L
                && (!ctrl_x_mode_line_or_eval()
                    || ins_compl_long_shown_match() as ::core::ffi::c_int != 0)
            {
                ins_compl_addfrommatch();
                return 1 as ::core::ffi::c_int;
            }
            if ins_compl_accept_char((*s).c) {
                let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
                if !str.is_null() {
                    let mut p: *mut ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != NUL {
                        ins_compl_addleader(utf_ptr2char(p));
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    xfree(str as *mut ::core::ffi::c_void);
                } else {
                    ins_compl_addleader((*s).c);
                }
                return 1 as ::core::ffi::c_int;
            }
            if ((*s).c == Ctrl_Y
                || ins_compl_enter_selects() as ::core::ffi::c_int != 0
                    && ((*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL))
                && stop_arrow() == OK
            {
                ins_compl_delete(false_0 != 0);
                if ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
                    && !ins_compl_is_match_selected()
                {
                    ins_compl_insert(false_0 != 0, true_0 != 0);
                    ins_compl_init_get_longest();
                    return 1 as ::core::ffi::c_int;
                } else {
                    ins_compl_insert(false_0 != 0, false_0 != 0);
                }
            } else if ascii_iswhite_nl_or_nul((*s).c) as ::core::ffi::c_int != 0
                && ins_compl_preinsert_effect() as ::core::ffi::c_int != 0
            {
                ins_compl_delete(false_0 != 0);
            }
        }
    }
    ins_compl_init_get_longest();
    if ins_compl_prep((*s).c) {
        return 1 as ::core::ffi::c_int;
    }
    if (*s).c == Ctrl_BSL {
        ins_redraw(false_0 != 0);
        no_mapping += 1;
        allow_keys += 1;
        (*s).c = plain_vgetc();
        no_mapping -= 1;
        allow_keys -= 1;
        if (*s).c != Ctrl_N && (*s).c != Ctrl_G && (*s).c != Ctrl_O {
            vungetc((*s).c);
            (*s).c = Ctrl_BSL;
        } else {
            if (*s).c == Ctrl_O {
                ins_ctrl_o();
                ins_at_eol = false_0 != 0;
                (*s).nomove = true_0 != 0;
            }
            (*s).count = 0 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).c = do_digraph((*s).c);
    }
    if ((*s).c == Ctrl_V || (*s).c == Ctrl_Q) && ctrl_x_mode_cmdline() as ::core::ffi::c_int != 0 {
        insert_do_complete(s);
        insert_handle_key_post(s);
        return 1 as ::core::ffi::c_int;
    }
    if (*s).c == Ctrl_V || (*s).c == Ctrl_Q {
        ins_ctrl_v();
        (*s).c = Ctrl_V;
        return 1 as ::core::ffi::c_int;
    }
    if cindent_on() as ::core::ffi::c_int != 0 && ctrl_x_mode_none() as ::core::ffi::c_int != 0 {
        (*s).line_is_white = inindent(0 as ::core::ffi::c_int);
        if in_cinkeys((*s).c, '!' as ::core::ffi::c_int, (*s).line_is_white) as ::core::ffi::c_int
            != 0
            && stop_arrow() == OK
        {
            do_c_expr_indent();
            return 1 as ::core::ffi::c_int;
        }
        if can_cindent as ::core::ffi::c_int != 0
            && in_cinkeys((*s).c, '*' as ::core::ffi::c_int, (*s).line_is_white)
                as ::core::ffi::c_int
                != 0
            && stop_arrow() == OK
        {
            do_c_expr_indent();
        }
    }
    if (*curwin).w_onebuf_opt.wo_rl != 0 {
        match (*s).c {
            K_LEFT => {
                (*s).c = K_RIGHT;
            }
            K_S_LEFT => {
                (*s).c = K_S_RIGHT;
            }
            -22013 => {
                (*s).c = -(253 as ::core::ffi::c_int
                    + ((KE_C_RIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            K_RIGHT => {
                (*s).c = K_LEFT;
            }
            K_S_RIGHT => {
                (*s).c = K_S_LEFT;
            }
            -22269 => {
                (*s).c = -(253 as ::core::ffi::c_int
                    + ((KE_C_LEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            _ => {}
        }
    }
    if ins_start_select((*s).c) {
        return 1 as ::core::ffi::c_int;
    }
    return insert_handle_key(s);
}
unsafe extern "C" fn insert_handle_key(mut s: *mut InsertState) -> ::core::ffi::c_int {
    's_1398: {
        '_normalchar: {
            '_check_pum: {
                'c_31045: {
                    'c_42507: {
                        'c_31081: {
                            'c_31145: {
                                'c_35097: {
                                    match (*s).c {
                                        ESC => {
                                            if echeck_abbr(ESC + ABBR_OFF) {
                                                break 's_1398;
                                            } else {
                                                break 'c_31045;
                                            }
                                        }
                                        Ctrl_C => {
                                            break 'c_31045;
                                        }
                                        Ctrl_O => {
                                            if ctrl_x_mode_omni() {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else if echeck_abbr(Ctrl_O + ABBR_OFF) {
                                                break 's_1398;
                                            } else {
                                                ins_ctrl_o();
                                                if get_ve_flags(curwin)
                                                    & kOptVeFlagOnemore as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    != 0
                                                {
                                                    ins_at_eol = false_0 != 0;
                                                    (*s).nomove = true_0 != 0;
                                                }
                                                (*s).count = 0 as ::core::ffi::c_int;
                                                return 0 as ::core::ffi::c_int;
                                            }
                                        }
                                        K_INS | K_KINS => {
                                            ins_insert((*s).replaceState);
                                            break 's_1398;
                                        }
                                        K_HELP | K_F1 | K_XF1 => {
                                            stuffcharReadbuff(K_HELP);
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        32 => {
                                            if mod_mask != MOD_MASK_CTRL {
                                                break '_normalchar;
                                            } else {
                                                break 'c_42507;
                                            }
                                        }
                                        K_ZERO | NUL | Ctrl_A => {
                                            break 'c_42507;
                                        }
                                        Ctrl_R => {
                                            if ctrl_x_mode_register() as ::core::ffi::c_int != 0
                                                && !ins_compl_active()
                                            {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else {
                                                ins_reg();
                                                auto_format(false_0 != 0, true_0 != 0);
                                                (*s).inserted_space = false_0;
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_G => {
                                            ins_ctrl_g();
                                            break 's_1398;
                                        }
                                        Ctrl_HAT => {
                                            ins_ctrl_hat();
                                            break 's_1398;
                                        }
                                        Ctrl__ => {
                                            if p_ari == 0 {
                                                break '_normalchar;
                                            } else {
                                                ins_ctrl_();
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_D => {
                                            if ctrl_x_mode_path_defines() {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            } else {
                                                break 'c_31081;
                                            }
                                        }
                                        Ctrl_T => {
                                            break 'c_31081;
                                        }
                                        K_DEL | K_KDEL => {
                                            ins_del();
                                            auto_format(false_0 != 0, true_0 != 0);
                                            break 's_1398;
                                        }
                                        K_BS | Ctrl_H => {
                                            (*s).did_backspace = ins_bs(
                                                (*s).c,
                                                BACKSPACE_CHAR as ::core::ffi::c_int,
                                                &raw mut (*s).inserted_space,
                                            );
                                            auto_format(false_0 != 0, true_0 != 0);
                                            if (*s).did_backspace {
                                                if ins_compl_has_autocomplete()
                                                    as ::core::ffi::c_int
                                                    != 0
                                                    && !char_avail()
                                                    && (*curwin).w_cursor.col
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    (*s).c = char_before_cursor();
                                                    if vim_isprintc((*s).c) {
                                                        redraw_later(
                                                            curwin,
                                                            UPD_VALID as ::core::ffi::c_int,
                                                        );
                                                        update_screen();
                                                        ui_flush();
                                                        ins_compl_enable_autocomplete();
                                                        insert_do_complete(s);
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        Ctrl_W => {
                                            if bt_prompt(curbuf) as ::core::ffi::c_int != 0
                                                && mod_mask & MOD_MASK_SHIFT
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                stuffcharReadbuff(Ctrl_W);
                                                restart_edit = 'A' as ::core::ffi::c_int;
                                                (*s).nomove = true_0 != 0;
                                                (*s).count = 0 as ::core::ffi::c_int;
                                                return 0 as ::core::ffi::c_int;
                                            }
                                            (*s).did_backspace = ins_bs(
                                                (*s).c,
                                                BACKSPACE_WORD as ::core::ffi::c_int,
                                                &raw mut (*s).inserted_space,
                                            );
                                            auto_format(false_0 != 0, true_0 != 0);
                                            if (*s).did_backspace {
                                                if ins_compl_has_autocomplete()
                                                    as ::core::ffi::c_int
                                                    != 0
                                                    && !char_avail()
                                                    && (*curwin).w_cursor.col
                                                        > 0 as ::core::ffi::c_int
                                                {
                                                    (*s).c = char_before_cursor();
                                                    if vim_isprintc((*s).c) {
                                                        redraw_later(
                                                            curwin,
                                                            UPD_VALID as ::core::ffi::c_int,
                                                        );
                                                        update_screen();
                                                        ui_flush();
                                                        ins_compl_enable_autocomplete();
                                                        insert_do_complete(s);
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        Ctrl_U => {
                                            if ctrl_x_mode_function() {
                                                insert_do_complete(s);
                                            } else {
                                                (*s).did_backspace = ins_bs(
                                                    (*s).c,
                                                    BACKSPACE_LINE as ::core::ffi::c_int,
                                                    &raw mut (*s).inserted_space,
                                                );
                                                auto_format(false_0 != 0, true_0 != 0);
                                                (*s).inserted_space = false_0;
                                                if (*s).did_backspace {
                                                    if ins_compl_has_autocomplete()
                                                        as ::core::ffi::c_int
                                                        != 0
                                                        && !char_avail()
                                                        && (*curwin).w_cursor.col
                                                            > 0 as ::core::ffi::c_int
                                                    {
                                                        (*s).c = char_before_cursor();
                                                        if vim_isprintc((*s).c) {
                                                            redraw_later(
                                                                curwin,
                                                                UPD_VALID as ::core::ffi::c_int,
                                                            );
                                                            update_screen();
                                                            ui_flush();
                                                            ins_compl_enable_autocomplete();
                                                            insert_do_complete(s);
                                                        }
                                                    }
                                                }
                                            }
                                            break 's_1398;
                                        }
                                        K_LEFTMOUSE | K_LEFTMOUSE_NM | K_LEFTDRAG
                                        | K_LEFTRELEASE | K_LEFTRELEASE_NM | K_MOUSEMOVE
                                        | K_MIDDLEMOUSE | K_MIDDLEDRAG | K_MIDDLERELEASE
                                        | K_RIGHTMOUSE | K_RIGHTDRAG | K_RIGHTRELEASE
                                        | K_X1MOUSE | K_X1DRAG | K_X1RELEASE | K_X2MOUSE
                                        | K_X2DRAG | K_X2RELEASE => {
                                            ins_mouse((*s).c);
                                            break 's_1398;
                                        }
                                        K_MOUSEDOWN => {
                                            ins_mousescroll(MSCR_DOWN as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSEUP => {
                                            ins_mousescroll(MSCR_UP as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSELEFT => {
                                            ins_mousescroll(MSCR_LEFT as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_MOUSERIGHT => {
                                            ins_mousescroll(MSCR_RIGHT as ::core::ffi::c_int);
                                            break 's_1398;
                                        }
                                        K_SELECT | -13821 => {
                                            break 's_1398;
                                        }
                                        K_PASTE_START => {
                                            paste_repeat(1 as ::core::ffi::c_int);
                                            break '_check_pum;
                                        }
                                        -26365 => {
                                            state_handle_k_event();
                                            if dont_sync_undo as ::core::ffi::c_int
                                                == kTrue as ::core::ffi::c_int
                                            {
                                                dont_sync_undo = kNone;
                                            }
                                            break '_check_pum;
                                        }
                                        K_COMMAND => {
                                            do_cmdline(
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                Some(
                                                    getcmdkeycmd
                                                        as unsafe extern "C" fn(
                                                            ::core::ffi::c_int,
                                                            *mut ::core::ffi::c_void,
                                                            ::core::ffi::c_int,
                                                            bool,
                                                        ) -> *mut ::core::ffi::c_char,
                                                ),
                                                NULL,
                                                0 as ::core::ffi::c_int,
                                            );
                                            break '_check_pum;
                                        }
                                        K_LUA => {
                                            map_execute_lua(false_0 != 0, false_0 != 0);
                                            break '_check_pum;
                                        }
                                        K_HOME | K_KHOME | K_S_HOME | -22525 => {
                                            ins_home((*s).c);
                                            break 's_1398;
                                        }
                                        K_END | K_KEND | K_S_END | -22781 => {
                                            ins_end((*s).c);
                                            break 's_1398;
                                        }
                                        K_LEFT => {
                                            if mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                                                ins_s_left();
                                            } else {
                                                ins_left();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_LEFT | -22013 => {
                                            ins_s_left();
                                            break 's_1398;
                                        }
                                        K_RIGHT => {
                                            if mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0 {
                                                ins_s_right();
                                            } else {
                                                ins_right();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_RIGHT | -22269 => {
                                            ins_s_right();
                                            break 's_1398;
                                        }
                                        K_UP => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else if mod_mask & MOD_MASK_SHIFT != 0 {
                                                ins_pageup();
                                            } else {
                                                ins_up(false_0 != 0);
                                            }
                                            break 's_1398;
                                        }
                                        K_S_UP | K_PAGEUP | K_KPAGEUP => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else {
                                                ins_pageup();
                                            }
                                            break 's_1398;
                                        }
                                        K_DOWN => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else if mod_mask & MOD_MASK_SHIFT != 0 {
                                                ins_pagedown();
                                            } else {
                                                ins_down(false_0 != 0);
                                            }
                                            break 's_1398;
                                        }
                                        K_S_DOWN | K_PAGEDOWN | K_KPAGEDOWN => {
                                            if pum_visible() {
                                                insert_do_complete(s);
                                            } else {
                                                ins_pagedown();
                                            }
                                            break 's_1398;
                                        }
                                        K_S_TAB => {
                                            (*s).c = TAB;
                                            break 'c_31145;
                                        }
                                        TAB => {
                                            break 'c_31145;
                                        }
                                        K_KENTER => {
                                            (*s).c = CAR;
                                            break 'c_35097;
                                        }
                                        CAR | NL => {
                                            break 'c_35097;
                                        }
                                        Ctrl_K => {
                                            if ctrl_x_mode_dictionary() {
                                                if check_compl_option(true_0 != 0) {
                                                    insert_do_complete(s);
                                                }
                                                break 's_1398;
                                            } else {
                                                (*s).c = ins_digraph();
                                                if (*s).c == NUL {
                                                    break 's_1398;
                                                } else {
                                                    break '_normalchar;
                                                }
                                            }
                                        }
                                        Ctrl_X => {
                                            ins_ctrl_x();
                                            break 's_1398;
                                        }
                                        Ctrl_RSB => {
                                            if !ctrl_x_mode_tags() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_F => {
                                            if !ctrl_x_mode_files() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        115 | Ctrl_S => {
                                            if !ctrl_x_mode_spell() {
                                                break '_normalchar;
                                            } else {
                                                insert_do_complete(s);
                                                break 's_1398;
                                            }
                                        }
                                        Ctrl_L => {
                                            if !ctrl_x_mode_whole_line() {
                                                break '_normalchar;
                                            }
                                        }
                                        Ctrl_P | Ctrl_N => {}
                                        Ctrl_Y | Ctrl_E => {
                                            (*s).c = ins_ctrl_ey((*s).c);
                                            break 's_1398;
                                        }
                                        Ctrl_Z | _ => {
                                            break '_normalchar;
                                        }
                                    }
                                    if *(*curbuf).b_p_cpt as ::core::ffi::c_int == NUL
                                        && (ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                                            || ctrl_x_mode_whole_line() as ::core::ffi::c_int != 0)
                                        && !compl_status_local()
                                    {
                                        break '_normalchar;
                                    } else {
                                        insert_do_complete(s);
                                        break 's_1398;
                                    }
                                }
                                if bt_quickfix(curbuf) as ::core::ffi::c_int != 0 && (*s).c == CAR {
                                    if (*curwin).w_llist_ref.is_null() {
                                        do_cmdline_cmd(
                                            b".cc\0".as_ptr() as *const ::core::ffi::c_char
                                        );
                                    } else {
                                        do_cmdline_cmd(
                                            b".ll\0".as_ptr() as *const ::core::ffi::c_char
                                        );
                                    }
                                    break 's_1398;
                                } else {
                                    if cmdwin_type != 0 as ::core::ffi::c_int {
                                        cmdwin_result = CAR;
                                        return 0 as ::core::ffi::c_int;
                                    }
                                    if mod_mask & MOD_MASK_SHIFT == 0 as ::core::ffi::c_int
                                        && bt_prompt(curbuf) as ::core::ffi::c_int != 0
                                    {
                                        prompt_invoke_callback();
                                        if !bt_prompt(curbuf) {
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        break 's_1398;
                                    } else {
                                        if !ins_eol((*s).c) {
                                            return 0 as ::core::ffi::c_int;
                                        }
                                        auto_format(false_0 != 0, false_0 != 0);
                                        (*s).inserted_space = false_0;
                                        break 's_1398;
                                    }
                                }
                            }
                            if ctrl_x_mode_path_patterns() {
                                insert_do_complete(s);
                                break 's_1398;
                            } else {
                                (*s).inserted_space = false_0;
                                if ins_tab() {
                                    break '_normalchar;
                                } else {
                                    auto_format(false_0 != 0, true_0 != 0);
                                    break 's_1398;
                                }
                            }
                        }
                        if (*s).c == Ctrl_T && ctrl_x_mode_thesaurus() as ::core::ffi::c_int != 0 {
                            if check_compl_option(false_0 != 0) {
                                insert_do_complete(s);
                            }
                            break 's_1398;
                        } else {
                            ins_shift((*s).c, (*s).lastc);
                            auto_format(false_0 != 0, true_0 != 0);
                            (*s).inserted_space = false_0;
                            break 's_1398;
                        }
                    }
                    if stuff_inserted(
                        NUL,
                        1 as ::core::ffi::c_int,
                        ((*s).c == Ctrl_A) as ::core::ffi::c_int,
                    ) == FAIL
                        && (*s).c != Ctrl_A
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                    (*s).inserted_space = false_0;
                    break 's_1398;
                }
                if (*s).c == Ctrl_C && cmdwin_type != 0 as ::core::ffi::c_int {
                    cmdwin_result = -(253 as ::core::ffi::c_int
                        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                    got_int = false_0 != 0;
                    (*s).nomove = true_0 != 0;
                    return 0 as ::core::ffi::c_int;
                }
                if (*s).c == Ctrl_C && bt_prompt(curbuf) as ::core::ffi::c_int != 0 {
                    if invoke_prompt_interrupt() {
                        if !bt_prompt(curbuf) {
                            return 0 as ::core::ffi::c_int;
                        }
                        break 's_1398;
                    }
                }
                return 0 as ::core::ffi::c_int;
            }
            if pum_want.active {
                if pum_visible() {
                    edit_submode_extra = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    insert_do_complete(s);
                    if pum_want.finish {
                        ins_compl_prep(Ctrl_Y);
                    }
                }
                pum_want.active = false_0 != 0;
            }
            if (*curbuf).b_u_synced {
                ins_need_undo = true_0 != 0;
            }
            break 's_1398;
        }
        if p_paste == 0 {
            let mut str: *mut ::core::ffi::c_char = do_insert_char_pre((*s).c);
            if !str.is_null() {
                if *str as ::core::ffi::c_int != NUL && stop_arrow() != FAIL {
                    let mut p: *mut ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != NUL {
                        (*s).c = utf_ptr2char(p);
                        if (*s).c == CAR || (*s).c == K_KENTER || (*s).c == NL {
                            ins_eol((*s).c);
                        } else {
                            ins_char((*s).c);
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                    AppendToRedobuffLit(str, -1 as ::core::ffi::c_int);
                }
                xfree(str as *mut ::core::ffi::c_void);
                (*s).c = NUL;
            }
            if (*s).c == NUL {
                break 's_1398;
            }
        }
        ins_try_si((*s).c);
        if (*s).c == ' ' as ::core::ffi::c_int {
            (*s).inserted_space = true_0;
            if inindent(0 as ::core::ffi::c_int) {
                can_cindent = false_0 != 0;
            }
            if Insstart_blank_vcol == MAXCOL as ::core::ffi::c_int
                && (*curwin).w_cursor.lnum == Insstart.lnum
            {
                Insstart_blank_vcol = get_nolist_virtcol();
            }
        }
        if vim_iswordc((*s).c) as ::core::ffi::c_int != 0
            || !echeck_abbr(
                (if (*s).c >= 0x100 as ::core::ffi::c_int {
                    (*s).c + ABBR_OFF
                } else {
                    (*s).c
                }),
            ) && (*s).c != Ctrl_RSB
        {
            insert_special((*s).c, false_0, false_0);
            revins_legal += 1;
            revins_chars += 1;
        }
        auto_format(false_0 != 0, true_0 != 0);
        foldOpenCursor();
        if ins_compl_has_autocomplete() as ::core::ffi::c_int != 0
            && !char_avail()
            && vim_isprintc((*s).c) as ::core::ffi::c_int != 0
        {
            redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
            update_screen();
            ui_flush();
            ins_compl_enable_autocomplete();
            insert_do_complete(s);
        }
    }
    insert_handle_key_post(s);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn insert_do_complete(mut s: *mut InsertState) {
    compl_busy = true_0 != 0;
    disable_fold_update += 1;
    if ins_complete((*s).c, true_0 != 0) == FAIL {
        compl_status_clear();
    }
    disable_fold_update -= 1;
    compl_busy = false_0 != 0;
    can_si = may_do_si();
}
unsafe extern "C" fn insert_do_cindent(mut s: *mut InsertState) {
    if in_cinkeys((*s).c, ' ' as ::core::ffi::c_int, (*s).line_is_white) {
        if stop_arrow() == OK {
            do_c_expr_indent();
        }
    }
}
unsafe extern "C" fn insert_handle_key_post(mut s: *mut InsertState) {
    if (*s).c
        != -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
    {
        did_cursorhold = false_0 != 0;
    }
    if ins_compl_active() as ::core::ffi::c_int != 0 && !ins_compl_win_active(curwin) {
        ins_compl_cancel();
    }
    if arrow_used {
        (*s).inserted_space = false_0;
    }
    if can_cindent as ::core::ffi::c_int != 0
        && cindent_on() as ::core::ffi::c_int != 0
        && ctrl_x_mode_normal() as ::core::ffi::c_int != 0
    {
        insert_do_cindent(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn edit(
    mut cmdchar: ::core::ffi::c_int,
    mut startln: bool,
    mut count: ::core::ffi::c_int,
) -> bool {
    if !(*curbuf).terminal.is_null() {
        if ex_normal_busy != 0 {
            restart_edit = 'i' as ::core::ffi::c_int;
            force_restart_edit = true_0 != 0;
            return false_0 != 0;
        }
        return terminal_enter();
    }
    if sandbox != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    if textlock != 0 as ::core::ffi::c_int
        || ins_compl_active() as ::core::ffi::c_int != 0
        || compl_busy as ::core::ffi::c_int != 0
        || pum_visible() as ::core::ffi::c_int != 0
        || expr_map_locked() as ::core::ffi::c_int != 0
    {
        emsg(gettext(&raw const e_textlock as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    let mut s: [InsertState; 1] = [InsertState {
        state: VimState {
            check: None,
            execute: None,
        },
        ca: ::core::ptr::null_mut::<cmdarg_T>(),
        mincol: 0,
        cmdchar: 0,
        cmdchar_todo: 0,
        ins_just_started: false,
        startln: 0,
        count: 0,
        c: 0,
        lastc: 0,
        i: 0,
        did_backspace: false,
        line_is_white: false,
        old_topline: 0,
        old_topfill: 0,
        inserted_space: 0,
        replaceState: 0,
        did_restart_edit: 0,
        nomove: false,
    }; 1];
    memset(
        &raw mut s as *mut InsertState as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<InsertState>(),
    );
    (*(&raw mut s as *mut InsertState)).state.execute = Some(
        insert_execute
            as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
    ) as state_execute_callback;
    (*(&raw mut s as *mut InsertState)).state.check =
        Some(insert_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
            as state_check_callback;
    (*(&raw mut s as *mut InsertState)).cmdchar = cmdchar;
    (*(&raw mut s as *mut InsertState)).startln = startln as ::core::ffi::c_int;
    (*(&raw mut s as *mut InsertState)).count = count;
    insert_enter(&raw mut s as *mut InsertState);
    return (*(&raw mut s as *mut InsertState)).c == Ctrl_O;
}
#[no_mangle]
pub unsafe extern "C" fn ins_need_undo_get() -> bool {
    return ins_need_undo;
}
#[no_mangle]
pub unsafe extern "C" fn ins_redraw(mut ready: bool) {
    if char_avail() {
        return;
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_CURSORMOVEDI) as ::core::ffi::c_int != 0
        && (last_cursormoved_win != curwin || !equalpos(last_cursormoved, (*curwin).w_cursor))
        && !pum_visible()
    {
        if syntax_present(curwin) as ::core::ffi::c_int != 0 && must_redraw != 0 {
            update_screen();
        }
        update_curswant();
        ins_apply_autocmds(EVENT_CURSORMOVEDI);
        last_cursormoved_win = curwin;
        last_cursormoved = (*curwin).w_cursor;
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_TEXTCHANGEDI) as ::core::ffi::c_int != 0
        && (*curbuf).b_last_changedtick_i != buf_get_changedtick(curbuf)
        && !pum_visible()
    {
        let mut aco: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        let mut tick: varnumber_T = buf_get_changedtick(curbuf);
        aucmd_prepbuf(&raw mut aco, curbuf);
        apply_autocmds(
            EVENT_TEXTCHANGEDI,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
        aucmd_restbuf(&raw mut aco);
        (*curbuf).b_last_changedtick_i = buf_get_changedtick(curbuf);
        if tick != buf_get_changedtick(curbuf) {
            u_save(
                (*curwin).w_cursor.lnum,
                (*curwin).w_cursor.lnum + 1 as linenr_T,
            );
        }
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_TEXTCHANGEDP) as ::core::ffi::c_int != 0
        && (*curbuf).b_last_changedtick_pum != buf_get_changedtick(curbuf)
        && pum_visible() as ::core::ffi::c_int != 0
    {
        let mut aco_0: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        let mut tick_0: varnumber_T = buf_get_changedtick(curbuf);
        aucmd_prepbuf(&raw mut aco_0, curbuf);
        apply_autocmds(
            EVENT_TEXTCHANGEDP,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
        aucmd_restbuf(&raw mut aco_0);
        (*curbuf).b_last_changedtick_pum = buf_get_changedtick(curbuf);
        if tick_0 != buf_get_changedtick(curbuf) {
            u_save(
                (*curwin).w_cursor.lnum,
                (*curwin).w_cursor.lnum + 1 as linenr_T,
            );
        }
    }
    if ready {
        may_trigger_win_scrolled_resized();
    }
    if ready as ::core::ffi::c_int != 0
        && has_event(EVENT_BUFMODIFIEDSET) as ::core::ffi::c_int != 0
        && (*curbuf).b_changed_invalid as ::core::ffi::c_int == true_0
        && !pum_visible()
    {
        apply_autocmds(
            EVENT_BUFMODIFIEDSET,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
        (*curbuf).b_changed_invalid = false_0 != 0;
    }
    may_trigger_safestate(
        ready as ::core::ffi::c_int != 0 && !ins_compl_active() && !pum_visible(),
    );
    pum_check_clear();
    show_cursor_info_later(false_0 != 0);
    if must_redraw != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if clear_cmdline as ::core::ffi::c_int != 0
            || redraw_cmdline as ::core::ffi::c_int != 0
            || redraw_mode as ::core::ffi::c_int != 0
        {
            showmode();
        }
    }
    setcursor();
    emsg_on_display = false_0 != 0;
}
unsafe extern "C" fn ins_ctrl_v() {
    let mut did_putchar: bool = false_0 != 0;
    ins_redraw(false_0 != 0);
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        edit_putchar('^' as ::core::ffi::c_int, true_0 != 0);
        did_putchar = true_0 != 0;
    }
    AppendToRedobuff(CTRL_V_STR.as_ptr());
    add_to_showcmd_c(Ctrl_V);
    let mut c: ::core::ffi::c_int = get_literal(mod_mask & MOD_MASK_SHIFT != 0);
    if did_putchar {
        edit_unputchar();
    }
    clear_showcmd();
    insert_special(c, true_0, true_0);
    revins_chars += 1;
    revins_legal += 1;
}
static mut pc_status: ::core::ffi::c_int = 0;
pub const PC_STATUS_UNSET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PC_STATUS_RIGHT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PC_STATUS_LEFT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PC_STATUS_SET: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut pc_schar: schar_T = 0;
static mut pc_attr: ::core::ffi::c_int = 0;
static mut pc_row: ::core::ffi::c_int = 0;
static mut pc_col: ::core::ffi::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn edit_putchar(mut c: ::core::ffi::c_int, mut highlight: bool) {
    if (*curwin).w_grid_alloc.chars.is_null() && default_grid.chars.is_null() {
        return;
    }
    let mut attr: ::core::ffi::c_int = 0;
    update_topline(curwin);
    validate_cursor(curwin);
    if highlight {
        attr = *hl_attr_active.offset(HLF_8 as ::core::ffi::c_int as isize);
    } else {
        attr = 0 as ::core::ffi::c_int;
    }
    pc_row = (*curwin).w_wrow;
    pc_status = PC_STATUS_UNSET;
    grid_line_start(&raw mut (*curwin).w_grid, pc_row);
    if (*curwin).w_onebuf_opt.wo_rl != 0 {
        pc_col = (*curwin).w_view_width - 1 as ::core::ffi::c_int - (*curwin).w_wcol;
        if grid_line_getchar(pc_col, ::core::ptr::null_mut::<::core::ffi::c_int>())
            == NUL as schar_T
        {
            grid_line_put_schar(
                pc_col - 1 as ::core::ffi::c_int,
                ' ' as ::core::ffi::c_int as schar_T,
                attr,
            );
            (*curwin).w_wcol -= 1;
            pc_status = PC_STATUS_RIGHT;
        }
    } else {
        pc_col = (*curwin).w_wcol;
        if grid_line_getchar(
            pc_col + 1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == NUL as schar_T
        {
            pc_status = PC_STATUS_LEFT;
        }
    }
    if pc_status == PC_STATUS_UNSET {
        pc_schar = grid_line_getchar(pc_col, &raw mut pc_attr);
        pc_status = PC_STATUS_SET;
    }
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    grid_line_puts(
        pc_col,
        &raw mut buf as *mut ::core::ffi::c_char,
        utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char),
        attr,
    );
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn buf_prompt_text(buf: *const buf_T) -> *mut ::core::ffi::c_char {
    if (*buf).b_prompt_text.is_null() {
        return b"% \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return (*buf).b_prompt_text;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_text() -> *mut ::core::ffi::c_char {
    return buf_prompt_text(curbuf);
}
unsafe extern "C" fn init_prompt(mut cmdchar_todo: ::core::ffi::c_int) {
    let mut prompt: *mut ::core::ffi::c_char = prompt_text();
    let mut prompt_len: ::core::ffi::c_int = strlen(prompt) as ::core::ffi::c_int;
    if (*curbuf).b_prompt_start.mark.lnum < 1 as linenr_T
        || (*curbuf).b_prompt_start.mark.lnum > (*curbuf).b_ml.ml_line_count
    {
        (*curbuf).b_prompt_start.mark.lnum = if 1 as linenr_T
            > (if (*curbuf).b_prompt_start.mark.lnum < (*curbuf).b_ml.ml_line_count {
                (*curbuf).b_prompt_start.mark.lnum
            } else {
                (*curbuf).b_ml.ml_line_count
            }) {
            1 as linenr_T
        } else if (*curbuf).b_prompt_start.mark.lnum < (*curbuf).b_ml.ml_line_count {
            (*curbuf).b_prompt_start.mark.lnum
        } else {
            (*curbuf).b_ml.ml_line_count
        };
        (*curbuf).b_prompt_append_new_line = true_0 != 0;
    }
    (*curwin).w_cursor.lnum = if (*curwin).w_cursor.lnum > (*curbuf).b_prompt_start.mark.lnum {
        (*curwin).w_cursor.lnum
    } else {
        (*curbuf).b_prompt_start.mark.lnum
    };
    let mut text: *mut ::core::ffi::c_char = ml_get((*curbuf).b_prompt_start.mark.lnum);
    let mut text_len: colnr_T = ml_get_len((*curbuf).b_prompt_start.mark.lnum);
    if (*curbuf).b_prompt_start.mark.lnum == (*curwin).w_cursor.lnum
        && ((*curbuf).b_prompt_start.mark.col < prompt_len
            || (*curbuf).b_prompt_start.mark.col > text_len
            || !strnequal(
                text.offset((*curbuf).b_prompt_start.mark.col as isize)
                    .offset(-(prompt_len as isize)),
                prompt,
                prompt_len as size_t,
            ))
    {
        if *text as ::core::ffi::c_int == NUL {
            ml_replace((*curbuf).b_prompt_start.mark.lnum, prompt, true_0 != 0);
            inserted_bytes(
                (*curbuf).b_prompt_start.mark.lnum,
                0 as colnr_T,
                0 as ::core::ffi::c_int,
                prompt_len,
            );
        } else {
            let lnum: linenr_T = (*curbuf).b_ml.ml_line_count;
            ml_append(lnum, prompt, 0 as colnr_T, false_0 != 0);
            appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
            (*curbuf).b_prompt_start.mark.lnum = (*curbuf).b_ml.ml_line_count;
            (*curbuf).b_prompt_append_new_line = true_0 != 0;
            u_clearallandblockfree(curbuf);
        }
        (*curbuf).b_prompt_start.mark.col = prompt_len as colnr_T;
        (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
        coladvance(curwin, MAXCOL as ::core::ffi::c_int);
    }
    if Insstart_orig.lnum != (*curbuf).b_prompt_start.mark.lnum
        || Insstart_orig.col != (*curbuf).b_prompt_start.mark.col
    {
        Insstart.lnum = (*curbuf).b_prompt_start.mark.lnum;
        Insstart.col = (*curbuf).b_prompt_start.mark.col;
        Insstart_orig = Insstart;
        Insstart_textlen = Insstart.col;
        Insstart_blank_vcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        arrow_used = false_0 != 0;
    }
    if cmdchar_todo == 'A' as ::core::ffi::c_int {
        coladvance(curwin, MAXCOL as ::core::ffi::c_int);
    }
    if (*curbuf).b_prompt_start.mark.lnum == (*curwin).w_cursor.lnum {
        (*curwin).w_cursor.col = if (*curwin).w_cursor.col > (*curbuf).b_prompt_start.mark.col {
            (*curwin).w_cursor.col
        } else {
            (*curbuf).b_prompt_start.mark.col
        };
    }
    check_cursor(curwin);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_curpos_editable() -> bool {
    return (*curwin).w_cursor.lnum > (*curbuf).b_prompt_start.mark.lnum
        || (*curwin).w_cursor.lnum == (*curbuf).b_prompt_start.mark.lnum
            && (*curwin).w_cursor.col >= (*curbuf).b_prompt_start.mark.col;
}
#[no_mangle]
pub unsafe extern "C" fn edit_unputchar() {
    if pc_status != PC_STATUS_UNSET {
        if pc_status == PC_STATUS_RIGHT {
            (*curwin).w_wcol += 1;
        }
        if pc_status == PC_STATUS_RIGHT || pc_status == PC_STATUS_LEFT {
            redrawWinline(curwin, (*curwin).w_cursor.lnum);
        } else {
            grid_line_start(&raw mut (*curwin).w_grid, pc_row);
            grid_line_put_schar(pc_col, pc_schar, pc_attr);
            grid_line_flush();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn display_dollar(mut col_arg: colnr_T) {
    let mut col: colnr_T = if col_arg > 0 as ::core::ffi::c_int {
        col_arg
    } else {
        0 as colnr_T
    };
    if !redrawing() {
        return;
    }
    let mut save_col: colnr_T = (*curwin).w_cursor.col;
    (*curwin).w_cursor.col = col;
    let mut p: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    (*curwin).w_cursor.col -= utf_head_off(p, p.offset(col as isize));
    curs_columns(curwin, false_0);
    if (*curwin).w_wcol < (*curwin).w_view_width {
        edit_putchar('$' as ::core::ffi::c_int, false_0 != 0);
        dollar_vcol = (*curwin).w_virtcol;
    }
    (*curwin).w_cursor.col = save_col;
}
#[no_mangle]
pub unsafe extern "C" fn undisplay_dollar() {
    if dollar_vcol < 0 as ::core::ffi::c_int {
        return;
    }
    dollar_vcol = -1 as ::core::ffi::c_int as colnr_T;
    redrawWinline(curwin, (*curwin).w_cursor.lnum);
}
#[no_mangle]
pub unsafe extern "C" fn truncate_spaces(mut line: *mut ::core::ffi::c_char, mut len: size_t) {
    let mut i: ::core::ffi::c_int = 0;
    i = len as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int
        && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_join(0 as ::core::ffi::c_int);
        }
        i -= 1;
    }
    *line.offset((i + 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn backspace_until_column(mut col: ::core::ffi::c_int) {
    while (*curwin).w_cursor.col > col {
        (*curwin).w_cursor.col -= 1;
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_do_bs(col);
        } else if !del_char_after_col(col) {
            break;
        }
    }
}
unsafe extern "C" fn del_char_after_col(mut limit_col: ::core::ffi::c_int) -> bool {
    if limit_col >= 0 as ::core::ffi::c_int {
        let mut ecol: colnr_T = (*curwin).w_cursor.col + 1 as colnr_T;
        mb_adjust_cursor();
        while (*curwin).w_cursor.col < limit_col {
            let mut l: ::core::ffi::c_int = utf_ptr2len(get_cursor_pos_ptr());
            if l == 0 as ::core::ffi::c_int {
                break;
            }
            (*curwin).w_cursor.col += l;
        }
        if *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL || (*curwin).w_cursor.col == ecol {
            return false_0 != 0;
        }
        del_bytes(ecol - (*curwin).w_cursor.col, false_0 != 0, true_0 != 0);
    } else {
        del_char(false_0 != 0);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_literal(mut no_simplify: bool) -> ::core::ffi::c_int {
    let mut nc: ::core::ffi::c_int = 0;
    let mut hex: bool = false_0 != 0;
    let mut octal: bool = false_0 != 0;
    let mut unicode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if got_int {
        return Ctrl_C;
    }
    no_mapping += 1;
    let mut cc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        nc = plain_vgetc();
        if !no_simplify {
            nc = merge_modifiers(nc, &raw mut mod_mask);
        }
        if mod_mask & !MOD_MASK_SHIFT != 0 as ::core::ffi::c_int {
            break;
        }
        if State & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (if nc < 0 as ::core::ffi::c_int || nc > 255 as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                utf8len_tab[nc as usize] as ::core::ffi::c_int
            }) == 1 as ::core::ffi::c_int
        {
            add_to_showcmd(nc);
        }
        if nc == 'x' as ::core::ffi::c_int || nc == 'X' as ::core::ffi::c_int {
            hex = true_0 != 0;
        } else if nc == 'o' as ::core::ffi::c_int || nc == 'O' as ::core::ffi::c_int {
            octal = true_0 != 0;
        } else if nc == 'u' as ::core::ffi::c_int || nc == 'U' as ::core::ffi::c_int {
            unicode = nc;
        } else {
            if hex as ::core::ffi::c_int != 0 || unicode != 0 as ::core::ffi::c_int {
                if !ascii_isxdigit(nc) {
                    break;
                }
                cc = cc * 16 as ::core::ffi::c_int + hex2nr(nc);
            } else if octal {
                if nc < '0' as ::core::ffi::c_int || nc > '7' as ::core::ffi::c_int {
                    break;
                }
                cc = cc * 8 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
            } else {
                if !ascii_isdigit(nc) {
                    break;
                }
                cc = cc * 10 as ::core::ffi::c_int + nc - '0' as ::core::ffi::c_int;
            }
            i += 1;
        }
        if cc > 255 as ::core::ffi::c_int && unicode == 0 as ::core::ffi::c_int {
            cc = 255 as ::core::ffi::c_int;
        }
        nc = 0 as ::core::ffi::c_int;
        if hex {
            if i >= 2 as ::core::ffi::c_int {
                break;
            }
        } else if unicode != 0 {
            if unicode == 'u' as ::core::ffi::c_int && i >= 4 as ::core::ffi::c_int
                || unicode == 'U' as ::core::ffi::c_int && i >= 8 as ::core::ffi::c_int
            {
                break;
            }
        } else if i >= 3 as ::core::ffi::c_int {
            break;
        }
    }
    if i == 0 as ::core::ffi::c_int {
        if nc == K_ZERO {
            cc = '\n' as ::core::ffi::c_int;
            nc = 0 as ::core::ffi::c_int;
        } else {
            cc = nc;
            nc = 0 as ::core::ffi::c_int;
        }
    }
    if cc == 0 as ::core::ffi::c_int {
        cc = '\n' as ::core::ffi::c_int;
    }
    no_mapping -= 1;
    if nc != 0 {
        vungetc(nc);
        mod_mask = 0 as ::core::ffi::c_int;
    }
    got_int = false_0 != 0;
    return cc;
}
unsafe extern "C" fn insert_special(
    mut c: ::core::ffi::c_int,
    mut allow_modmask: ::core::ffi::c_int,
    mut ctrlv: ::core::ffi::c_int,
) {
    if mod_mask & MOD_MASK_CMD != 0 {
        allow_modmask = true_0;
    }
    if c < 0 as ::core::ffi::c_int || mod_mask != 0 && allow_modmask != 0 {
        let mut p: *mut ::core::ffi::c_char = get_special_key_name(c, mod_mask);
        let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
        c = *p.offset((len - 1 as ::core::ffi::c_int) as isize) as uint8_t as ::core::ffi::c_int;
        if len > 2 as ::core::ffi::c_int {
            if stop_arrow() == FAIL {
                return;
            }
            *p.offset((len - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
            ins_str(p, (len - 1 as ::core::ffi::c_int) as size_t);
            AppendToRedobuffLit(p, -1 as ::core::ffi::c_int);
            ctrlv = false_0;
        }
    }
    if stop_arrow() == OK {
        insertchar(
            c,
            if ctrlv != 0 {
                INSCHAR_CTRLV as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            -1 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn insertchar(
    mut c: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut force_format: ::core::ffi::c_int = flags & INSCHAR_FORMAT as ::core::ffi::c_int;
    let textwidth: ::core::ffi::c_int = comp_textwidth(force_format != 0);
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    if textwidth > 0 as ::core::ffi::c_int
        && (force_format != 0
            || !ascii_iswhite(c)
                && !(State & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && State & VREPLACE_FLAG as ::core::ffi::c_int == 0
                    && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL)
                && ((*curwin).w_cursor.lnum != Insstart.lnum
                    || (!has_format_option(FO_INS_LONG) || Insstart_textlen <= textwidth)
                        && (!fo_ins_blank || Insstart_blank_vcol <= textwidth)))
    {
        let mut do_internal: bool = true_0 != 0;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells((if c != NUL { c } else { gchar_cursor() }));
        if *(*curbuf).b_p_fex as ::core::ffi::c_int != NUL
            && flags & INSCHAR_NO_FEX as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (force_format != 0 || virtcol > textwidth)
        {
            do_internal = fex_format((*curwin).w_cursor.lnum, 1 as ::core::ffi::c_long, c)
                != 0 as ::core::ffi::c_int;
            ins_need_undo = true_0 != 0;
        }
        if do_internal {
            internal_format(textwidth, second_indent, flags, c == NUL, c);
        }
    }
    if c == NUL {
        return;
    }
    if did_ai as ::core::ffi::c_int != 0 && c == end_comment_pending {
        let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut i: ::core::ffi::c_int = get_leader_len(line, &raw mut p, false_0 != 0, true_0 != 0);
        if i > 0 as ::core::ffi::c_int && !vim_strchr(p, COM_MIDDLE).is_null() {
            while *p as ::core::ffi::c_int != 0
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ':' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            let mut middle_len: ::core::ffi::c_int = copy_option_part(
                &raw mut p,
                &raw mut lead_end as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            while middle_len > 0 as ::core::ffi::c_int
                && ascii_iswhite(
                    lead_end[(middle_len - 1 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                middle_len -= 1;
            }
            while *p as ::core::ffi::c_int != 0
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ':' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            let mut end_len: ::core::ffi::c_int = copy_option_part(
                &raw mut p,
                &raw mut lead_end as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            i = (*curwin).w_cursor.col as ::core::ffi::c_int;
            loop {
                i -= 1;
                if !(i >= 0 as ::core::ffi::c_int
                    && ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0)
                {
                    break;
                }
            }
            i += 1;
            i -= middle_len;
            if i >= 0 as ::core::ffi::c_int
                && end_len > 0 as ::core::ffi::c_int
                && lead_end[(end_len - 1 as ::core::ffi::c_int) as usize] as uint8_t
                    as ::core::ffi::c_int
                    == end_comment_pending
            {
                backspace_until_column(i);
                ins_bytes_len(
                    &raw mut lead_end as *mut ::core::ffi::c_char,
                    (end_len - 1 as ::core::ffi::c_int) as size_t,
                );
            }
        }
    }
    end_comment_pending = NUL;
    did_ai = false_0 != 0;
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    if !(c < ' ' as ::core::ffi::c_int
        || c >= DEL
        || c == '0' as ::core::ffi::c_int
        || c == '^' as ::core::ffi::c_int)
        && utf_char2len(c) == 1 as ::core::ffi::c_int
        && !has_event(EVENT_INSERTCHARPRE)
        && !test_disable_char_avail
        && vpeekc() != NUL
        && State & REPLACE_FLAG as ::core::ffi::c_int == 0
        && !cindent_on()
        && p_ri == 0
    {
        let mut buf: [::core::ffi::c_char; 101] = [0; 101];
        let mut virtcol_0: colnr_T = 0 as colnr_T;
        buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
        let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if textwidth > 0 as ::core::ffi::c_int {
            virtcol_0 = get_nolist_virtcol();
        }
        loop {
            c = vpeekc();
            if !(c != NUL
                && !(c < ' ' as ::core::ffi::c_int
                    || c >= DEL
                    || c == '0' as ::core::ffi::c_int
                    || c == '^' as ::core::ffi::c_int)
                && utf8len_tab[c as usize] as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && i_0 < INPUT_BUFLEN
                && (textwidth == 0 as ::core::ffi::c_int || {
                    virtcol_0 += byte2cells(
                        buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                            as ::core::ffi::c_int,
                    );
                    virtcol_0 < textwidth
                })
                && !(!no_abbr
                    && !vim_iswordc(c)
                    && vim_iswordc(
                        buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0))
            {
                break;
            }
            c = vgetc();
            let c2rust_fresh0 = i_0;
            i_0 = i_0 + 1;
            buf[c2rust_fresh0 as usize] = c as ::core::ffi::c_char;
        }
        do_digraph(-1 as ::core::ffi::c_int);
        do_digraph(buf[(i_0 - 1 as ::core::ffi::c_int) as usize] as uint8_t as ::core::ffi::c_int);
        buf[i_0 as usize] = NUL as ::core::ffi::c_char;
        ins_str(&raw mut buf as *mut ::core::ffi::c_char, i_0 as size_t);
        if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
            redo_literal(
                *(&raw mut buf as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int,
            );
            i_0 = 1 as ::core::ffi::c_int;
        } else {
            i_0 = 0 as ::core::ffi::c_int;
        }
        if buf[i_0 as usize] as ::core::ffi::c_int != NUL {
            AppendToRedobuffLit(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(i_0 as isize),
                -1 as ::core::ffi::c_int,
            );
        }
    } else {
        let mut cc: ::core::ffi::c_int = 0;
        cc = utf_char2len(c);
        if cc > 1 as ::core::ffi::c_int {
            let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
            utf_char2bytes(c, &raw mut buf_0 as *mut ::core::ffi::c_char);
            buf_0[cc as usize] = NUL as ::core::ffi::c_char;
            ins_char_bytes(&raw mut buf_0 as *mut ::core::ffi::c_char, cc as size_t);
            AppendCharToRedobuff(c);
        } else {
            ins_char(c);
            if flags & INSCHAR_CTRLV as ::core::ffi::c_int != 0 {
                redo_literal(c);
            } else {
                AppendCharToRedobuff(c);
            }
        }
    };
}
pub const INPUT_BUFLEN: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
unsafe extern "C" fn redo_literal(mut c: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 10] = [0; 10];
    if ascii_isdigit(c) {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            b"%03d\0".as_ptr() as *const ::core::ffi::c_char,
            c,
        );
        AppendToRedobuff(&raw mut buf as *mut ::core::ffi::c_char);
    } else {
        AppendCharToRedobuff(c);
    };
}
#[no_mangle]
pub unsafe extern "C" fn start_arrow(mut end_insert_pos: *mut pos_T) {
    start_arrow_common(end_insert_pos, true_0 != 0);
}
unsafe extern "C" fn start_arrow_with_change(mut end_insert_pos: *mut pos_T, mut end_change: bool) {
    start_arrow_common(end_insert_pos, end_change);
    if !end_change {
        AppendCharToRedobuff(Ctrl_G);
        AppendCharToRedobuff('U' as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn start_arrow_common(mut end_insert_pos: *mut pos_T, mut end_change: bool) {
    if !arrow_used && end_change as ::core::ffi::c_int != 0 {
        AppendToRedobuff(ESC_STR.as_ptr());
        stop_insert(end_insert_pos, false_0, false_0);
        arrow_used = true_0 != 0;
    }
    check_spell_redraw();
}
unsafe extern "C" fn check_spell_redraw() {
    if spell_redraw_lnum != 0 as linenr_T {
        let mut lnum: linenr_T = spell_redraw_lnum;
        spell_redraw_lnum = 0 as ::core::ffi::c_int as linenr_T;
        redrawWinline(curwin, lnum);
    }
}
#[no_mangle]
pub unsafe extern "C" fn stop_arrow() -> ::core::ffi::c_int {
    if arrow_used {
        Insstart = (*curwin).w_cursor;
        if Insstart.col > Insstart_orig.col && !ins_need_undo {
            update_Insstart_orig = false_0 != 0;
        }
        Insstart_textlen = linetabsize_str(get_cursor_line_ptr()) as colnr_T;
        if u_save_cursor() == OK {
            arrow_used = false_0 != 0;
            ins_need_undo = false_0 != 0;
        }
        ai_col = 0 as ::core::ffi::c_int as colnr_T;
        if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            orig_line_count = (*curbuf).b_ml.ml_line_count;
            vr_lines_changed = 1 as ::core::ffi::c_int;
        }
        ResetRedobuff();
        AppendToRedobuff(b"1i\0".as_ptr() as *const ::core::ffi::c_char);
        new_insert_skip = 2 as ::core::ffi::c_int;
    } else if ins_need_undo {
        if u_save_cursor() == OK {
            ins_need_undo = false_0 != 0;
        }
    }
    foldOpenCursor();
    return if arrow_used as ::core::ffi::c_int != 0 || ins_need_undo as ::core::ffi::c_int != 0 {
        FAIL
    } else {
        OK
    };
}
unsafe extern "C" fn stop_insert(
    mut end_insert_pos: *mut pos_T,
    mut esc: ::core::ffi::c_int,
    mut nomove: ::core::ffi::c_int,
) {
    stop_redo_ins();
    xfree(replace_stack.items as *mut ::core::ffi::c_void);
    replace_stack.capacity = 0 as size_t;
    replace_stack.size = replace_stack.capacity;
    replace_stack.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut inserted: String_0 = get_inserted();
    let mut added: ::core::ffi::c_int = if inserted.data.is_null() {
        0 as ::core::ffi::c_int
    } else {
        inserted.size as ::core::ffi::c_int - new_insert_skip
    };
    if did_restart_edit == 0 as ::core::ffi::c_int || added > 0 as ::core::ffi::c_int {
        xfree(last_insert.data as *mut ::core::ffi::c_void);
        last_insert = inserted;
        last_insert_skip = if added < 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            new_insert_skip
        };
    } else {
        xfree(inserted.data as *mut ::core::ffi::c_void);
    }
    if !arrow_used && !end_insert_pos.is_null() {
        let mut cc: ::core::ffi::c_int = 0;
        if !ins_need_undo && has_format_option(FO_AUTO) as ::core::ffi::c_int != 0 {
            let mut tpos: pos_T = (*curwin).w_cursor;
            cc = 'x' as ::core::ffi::c_int;
            if (*curwin).w_cursor.col > 0 as ::core::ffi::c_int && gchar_cursor() == NUL {
                dec_cursor();
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    (*curwin).w_cursor = tpos;
                }
            }
            auto_format(true_0 != 0, false_0 != 0);
            if ascii_iswhite(cc) {
                if gchar_cursor() != NUL {
                    inc_cursor();
                }
                if gchar_cursor() == NUL
                    && (*curwin).w_cursor.lnum == tpos.lnum
                    && (*curwin).w_cursor.col == tpos.col
                {
                    (*curwin).w_cursor.coladd = tpos.coladd;
                }
            }
        }
        check_auto_format(true_0 != 0);
        if nomove == 0
            && did_ai as ::core::ffi::c_int != 0
            && (esc != 0
                || vim_strchr(p_cpo, CPO_INDENT).is_null()
                    && (*curwin).w_cursor.lnum != (*end_insert_pos).lnum)
            && (*end_insert_pos).lnum <= (*curbuf).b_ml.ml_line_count
        {
            let mut tpos_0: pos_T = (*curwin).w_cursor;
            let mut prev_col: colnr_T = (*end_insert_pos).col;
            (*curwin).w_cursor = *end_insert_pos;
            check_cursor_col(curwin);
            loop {
                if gchar_cursor() == NUL && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int {
                    (*curwin).w_cursor.col -= 1;
                }
                cc = gchar_cursor();
                if !ascii_iswhite(cc) {
                    break;
                }
                if del_char(true_0 != 0) == FAIL {
                    break;
                }
            }
            if (*curwin).w_cursor.lnum != tpos_0.lnum {
                (*curwin).w_cursor = tpos_0;
            } else if (*curwin).w_cursor.col < prev_col {
                tpos_0 = (*curwin).w_cursor;
                tpos_0.col += 1;
                if cc != NUL && gchar_pos(&raw mut tpos_0) == NUL {
                    (*curwin).w_cursor.col += 1;
                }
            }
            if VIsual_active {
                check_visual_pos();
            }
        }
    }
    did_ai = false_0 != 0;
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    if !end_insert_pos.is_null() {
        (*curbuf).b_op_start = Insstart;
        (*curbuf).b_op_start_orig = Insstart_orig;
        (*curbuf).b_op_end = *end_insert_pos;
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_last_insert(mut c: ::core::ffi::c_int) {
    xfree(last_insert.data as *mut ::core::ffi::c_void);
    last_insert.data = xmalloc(
        (MB_MAXBYTES as ::core::ffi::c_int * 3 as ::core::ffi::c_int + 5 as ::core::ffi::c_int)
            as size_t,
    ) as *mut ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = last_insert.data;
    if c < ' ' as ::core::ffi::c_int || c == DEL {
        let c2rust_fresh5 = s;
        s = s.offset(1);
        *c2rust_fresh5 = Ctrl_V as ::core::ffi::c_char;
    }
    s = add_char2buf(c, s);
    let c2rust_fresh6 = s;
    s = s.offset(1);
    *c2rust_fresh6 = ESC as ::core::ffi::c_char;
    *s = NUL as ::core::ffi::c_char;
    last_insert.size = s.offset_from(last_insert.data) as size_t;
    last_insert_skip = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn beginline(mut flags: ::core::ffi::c_int) {
    if flags & BL_SOL as ::core::ffi::c_int != 0 && p_sol == 0 {
        coladvance(curwin, (*curwin).w_curswant);
    } else {
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        if flags & (BL_WHITE as ::core::ffi::c_int | BL_SOL as ::core::ffi::c_int) != 0 {
            let mut ptr: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            while ascii_iswhite(*ptr as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && !(flags & BL_FIX as ::core::ffi::c_int != 0
                    && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            {
                (*curwin).w_cursor.col += 1;
                ptr = ptr.offset(1);
            }
        }
        (*curwin).w_set_curswant = true_0;
    }
    adjust_skipcol();
}
#[no_mangle]
pub unsafe extern "C" fn oneright() -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if virtual_active(curwin) {
        let mut prevpos: pos_T = (*curwin).w_cursor;
        ptr = get_cursor_pos_ptr();
        coladvance(
            curwin,
            getviscol()
                + (if *ptr as ::core::ffi::c_int != TAB
                    && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                {
                    ptr2cells(ptr)
                } else {
                    1 as colnr_T
                }),
        );
        (*curwin).w_set_curswant = true_0;
        return if prevpos.col != (*curwin).w_cursor.col
            || prevpos.coladd != (*curwin).w_cursor.coladd
        {
            OK
        } else {
            FAIL
        };
    }
    ptr = get_cursor_pos_ptr();
    if *ptr as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    let mut l: ::core::ffi::c_int = utfc_ptr2len(ptr);
    if *ptr.offset(l as isize) as ::core::ffi::c_int == NUL
        && get_ve_flags(curwin) & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
            == 0 as ::core::ffi::c_uint
    {
        return FAIL;
    }
    (*curwin).w_cursor.col += l;
    (*curwin).w_set_curswant = true_0;
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn oneleft() -> ::core::ffi::c_int {
    if virtual_active(curwin) {
        let mut v: ::core::ffi::c_int = getviscol();
        if v == 0 as ::core::ffi::c_int {
            return FAIL;
        }
        let mut width: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        loop {
            coladvance(curwin, v as colnr_T - width as colnr_T);
            if getviscol() < v {
                break;
            }
            width += 1;
        }
        if (*curwin).w_cursor.coladd == 1 as ::core::ffi::c_int {
            let mut ptr: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            if *ptr as ::core::ffi::c_int != TAB
                && vim_isprintc(utf_ptr2char(ptr)) as ::core::ffi::c_int != 0
                && ptr2cells(ptr) > 1 as ::core::ffi::c_int
            {
                (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        (*curwin).w_set_curswant = true_0;
        adjust_skipcol();
        return OK;
    }
    if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
        return FAIL;
    }
    (*curwin).w_set_curswant = true_0;
    (*curwin).w_cursor.col -= 1;
    mb_adjust_cursor();
    adjust_skipcol();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_up_inner(
    mut wp: *mut win_T,
    mut n: linenr_T,
    mut skip_conceal: bool,
) {
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    if n >= lnum {
        lnum = 1 as ::core::ffi::c_int as linenr_T;
    } else if win_lines_concealed(wp) {
        hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
        loop {
            let c2rust_fresh3 = n;
            n = n - 1;
            if c2rust_fresh3 == 0 {
                break;
            }
            lnum -= 1;
            if lnum <= 1 as linenr_T {
                break;
            }
            n = (n as ::core::ffi::c_int
                + (skip_conceal as ::core::ffi::c_int != 0
                    && decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int) as linenr_T;
            if n > 0 as linenr_T
                || !(State & MODE_INSERT as ::core::ffi::c_int != 0
                    || fdo_flags & kOptFdoFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint != 0)
            {
                hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
            }
        }
        lnum = if lnum > 1 as linenr_T {
            lnum
        } else {
            1 as linenr_T
        };
    } else {
        lnum -= n;
    }
    (*wp).w_cursor.lnum = lnum;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_up(mut n: linenr_T, mut upd_topline: bool) -> ::core::ffi::c_int {
    if n > 0 as linenr_T && (*curwin).w_cursor.lnum <= 1 as linenr_T {
        return FAIL;
    }
    cursor_up_inner(curwin, n, false_0 != 0);
    coladvance(curwin, (*curwin).w_curswant);
    if upd_topline {
        update_topline(curwin);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_down_inner(
    mut wp: *mut win_T,
    mut n: ::core::ffi::c_int,
    mut skip_conceal: bool,
) {
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    let mut line_count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
    if lnum + n as linenr_T >= line_count {
        lnum = line_count;
    } else if win_lines_concealed(wp) {
        let mut last: linenr_T = 0;
        loop {
            let c2rust_fresh2 = n;
            n = n - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            if hasFoldingWin(
                wp,
                lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut last,
                true_0 != 0,
                ::core::ptr::null_mut::<foldinfo_T>(),
            ) {
                lnum = last + 1 as linenr_T;
            } else {
                lnum += 1;
            }
            if lnum >= line_count {
                break;
            }
            n += (skip_conceal as ::core::ffi::c_int != 0
                && decor_conceal_line(
                    wp,
                    lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int;
        }
        lnum = if lnum < line_count { lnum } else { line_count };
    } else {
        lnum += n as linenr_T;
    }
    (*wp).w_cursor.lnum = lnum;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_down(
    mut n: ::core::ffi::c_int,
    mut upd_topline: bool,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = (*curwin).w_cursor.lnum;
    hasFoldingWin(
        curwin,
        lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut lnum,
        true_0 != 0,
        ::core::ptr::null_mut::<foldinfo_T>(),
    );
    if n > 0 as ::core::ffi::c_int && lnum >= (*(*curwin).w_buffer).b_ml.ml_line_count {
        return FAIL;
    }
    cursor_down_inner(curwin, n, false_0 != 0);
    coladvance(curwin, (*curwin).w_curswant);
    if upd_topline {
        update_topline(curwin);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn stuff_inserted(
    mut c: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut no_esc: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut last: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut insert: String_0 = get_last_insert();
    if insert.data.is_null() {
        emsg(gettext(
            &raw const e_noinstext as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if c != NUL {
        stuffcharReadbuff(c);
    }
    if insert.size > 0 as size_t {
        let mut p: *mut ::core::ffi::c_char = insert
            .data
            .offset(insert.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        while p >= insert.data {
            if *p as ::core::ffi::c_int == ESC {
                insert.size = p.offset_from(insert.data) as size_t;
                break;
            } else {
                p = p.offset(-1);
            }
        }
    }
    if insert.size > 0 as size_t {
        let mut p_0: *mut ::core::ffi::c_char = insert
            .data
            .offset(insert.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if (*p_0 as ::core::ffi::c_int == '0' as ::core::ffi::c_int
            || *p_0 as ::core::ffi::c_int == '^' as ::core::ffi::c_int)
            && (no_esc != 0
                || *insert.data as ::core::ffi::c_int == Ctrl_D && count > 1 as ::core::ffi::c_int)
        {
            last = *p_0;
            insert.size = insert.size.wrapping_sub(1);
        }
    }
    loop {
        stuffReadbuffLen(insert.data, insert.size as ptrdiff_t);
        match last as ::core::ffi::c_int {
            48 => {
                stuffReadbuffLen(
                    b"\x16048\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
            }
            94 => {
                stuffReadbuffLen(
                    b"\x16^\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
            }
            _ => {}
        }
        count -= 1;
        if count <= 0 as ::core::ffi::c_int {
            break;
        }
    }
    if no_esc == 0 {
        stuffcharReadbuff(ESC);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_last_insert() -> String_0 {
    return if last_insert.data.is_null() {
        NULL_STRING
    } else {
        String_0 {
            data: last_insert.data.offset(last_insert_skip as isize),
            size: last_insert.size.wrapping_sub(last_insert_skip as size_t),
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_last_insert_save() -> *mut ::core::ffi::c_char {
    let mut insert: String_0 = get_last_insert();
    if insert.data.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut s: *mut ::core::ffi::c_char =
        xmemdupz(insert.data as *const ::core::ffi::c_void, insert.size)
            as *mut ::core::ffi::c_char;
    if insert.size > 0 as size_t
        && *s.offset(insert.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == ESC
    {
        insert.size = insert.size.wrapping_sub(1);
        *s.offset(insert.size as isize) = NUL as ::core::ffi::c_char;
    }
    return s;
}
unsafe extern "C" fn echeck_abbr(mut c: ::core::ffi::c_int) -> bool {
    if p_paste != 0 || no_abbr as ::core::ffi::c_int != 0 || arrow_used as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    return check_abbr(
        c,
        get_cursor_line_ptr(),
        (*curwin).w_cursor.col as ::core::ffi::c_int,
        if (*curwin).w_cursor.lnum == Insstart.lnum {
            Insstart.col as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn replace_push(mut str: *mut ::core::ffi::c_char, mut len: size_t) {
    if replace_stack.size < replace_offset as size_t {
        return;
    }
    if replace_stack.capacity < replace_stack.size.wrapping_add(len) {
        replace_stack.capacity = replace_stack.size.wrapping_add(len);
        replace_stack.capacity = replace_stack.capacity.wrapping_sub(1);
        replace_stack.capacity |= replace_stack.capacity >> 1 as ::core::ffi::c_int;
        replace_stack.capacity |= replace_stack.capacity >> 2 as ::core::ffi::c_int;
        replace_stack.capacity |= replace_stack.capacity >> 4 as ::core::ffi::c_int;
        replace_stack.capacity |= replace_stack.capacity >> 8 as ::core::ffi::c_int;
        replace_stack.capacity |= replace_stack.capacity >> 16 as ::core::ffi::c_int;
        replace_stack.capacity = replace_stack.capacity.wrapping_add(1);
        replace_stack.capacity = replace_stack.capacity;
        replace_stack.items = xrealloc(
            replace_stack.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(replace_stack.capacity),
        ) as *mut ::core::ffi::c_char;
    }
    let mut p: *mut ::core::ffi::c_char = replace_stack
        .items
        .offset(replace_stack.size as isize)
        .offset(-(replace_offset as isize));
    if replace_offset != 0 {
        memmove(
            p.offset(len as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            replace_offset as size_t,
        );
    }
    memcpy(
        p as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    replace_stack.size = replace_stack.size.wrapping_add(len);
}
#[no_mangle]
pub unsafe extern "C" fn replace_push_nul() {
    replace_push(
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        1 as size_t,
    );
}
unsafe extern "C" fn replace_pop_if_nul() -> ::core::ffi::c_int {
    let mut ch: ::core::ffi::c_int = if replace_stack.size != 0 {
        *replace_stack
            .items
            .offset(replace_stack.size.wrapping_sub(1 as size_t) as isize) as uint8_t
            as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    if ch == NUL {
        replace_stack.size = replace_stack.size.wrapping_sub(1);
    }
    return ch;
}
#[no_mangle]
pub unsafe extern "C" fn replace_join(mut off: ::core::ffi::c_int) {
    let mut i: ssize_t = replace_stack.size as ssize_t;
    loop {
        i -= 1;
        if i < 0 as ssize_t {
            break;
        }
        if *replace_stack.items.offset(i as isize) as ::core::ffi::c_int == NUL && {
            let c2rust_fresh1 = off;
            off = off - 1;
            c2rust_fresh1 <= 0 as ::core::ffi::c_int
        } {
            replace_stack.size = replace_stack.size.wrapping_sub(1);
            memmove(
                replace_stack.items.offset(i as isize) as *mut ::core::ffi::c_void,
                replace_stack.items.offset((i + 1 as ssize_t) as isize)
                    as *const ::core::ffi::c_void,
                replace_stack.size.wrapping_sub(i as size_t),
            );
            return;
        }
    }
}
unsafe extern "C" fn replace_pop_ins() {
    let mut oldState: ::core::ffi::c_int = State;
    State = MODE_NORMAL as ::core::ffi::c_int;
    while replace_pop_if_nul() > 0 as ::core::ffi::c_int {
        mb_replace_pop_ins();
        dec_cursor();
    }
    State = oldState;
}
unsafe extern "C" fn mb_replace_pop_ins() {
    let mut len: ::core::ffi::c_int = utf_head_off(
        replace_stack.items.offset(0 as ::core::ffi::c_int as isize),
        replace_stack
            .items
            .offset(replace_stack.size.wrapping_sub(1 as size_t) as isize),
    ) + 1 as ::core::ffi::c_int;
    replace_stack.size = replace_stack.size.wrapping_sub(len as size_t);
    ins_bytes_len(
        replace_stack.items.offset(replace_stack.size as isize),
        len as size_t,
    );
}
unsafe extern "C" fn replace_do_bs(mut limit_col: ::core::ffi::c_int) {
    let mut start_vcol: colnr_T = 0;
    let l_State: ::core::ffi::c_int = State;
    let mut cc: ::core::ffi::c_int = replace_pop_if_nul();
    if cc > 0 as ::core::ffi::c_int {
        let mut orig_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_vcols: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            getvcol(
                curwin,
                &raw mut (*curwin).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut start_vcol,
                ::core::ptr::null_mut::<colnr_T>(),
            );
            orig_vcols = win_chartabsize(curwin, get_cursor_pos_ptr(), start_vcol);
        }
        del_char_after_col(limit_col);
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            orig_len = get_cursor_pos_len() as ::core::ffi::c_int;
        }
        replace_pop_ins();
        if l_State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
            let mut ins_len: ::core::ffi::c_int = get_cursor_pos_len() - orig_len;
            let mut vcol: ::core::ffi::c_int = start_vcol as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ins_len {
                vcol += win_chartabsize(curwin, p.offset(i as isize), vcol as colnr_T);
                i += utfc_ptr2len(p) - 1 as ::core::ffi::c_int;
                i += 1;
            }
            vcol -= start_vcol as ::core::ffi::c_int;
            (*curwin).w_cursor.col += ins_len;
            while vcol > orig_vcols && gchar_cursor() == ' ' as ::core::ffi::c_int {
                del_char(false_0 != 0);
                orig_vcols += 1;
            }
            (*curwin).w_cursor.col -= ins_len;
        }
        changed_bytes((*curwin).w_cursor.lnum, (*curwin).w_cursor.col);
    } else if cc == 0 as ::core::ffi::c_int {
        del_char_after_col(limit_col);
    }
}
unsafe extern "C" fn ins_reg() {
    let mut need_redraw: bool = false_0 != 0;
    let mut literally: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vis_active: ::core::ffi::c_int = VIsual_active as ::core::ffi::c_int;
    pc_status = PC_STATUS_UNSET;
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        ins_redraw(false_0 != 0);
        edit_putchar('"' as ::core::ffi::c_int, true_0 != 0);
        add_to_showcmd_c(Ctrl_R);
    }
    no_mapping += 1;
    allow_keys += 1;
    let mut regname: ::core::ffi::c_int = plain_vgetc();
    if *p_langmap as ::core::ffi::c_int != 0
        && true
        && (p_lrm != 0
            || (if vgetc_busy != 0 {
                (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } else {
                KeyTyped as ::core::ffi::c_int
            }) != 0)
        && KeyStuffed == 0
        && regname >= 0 as ::core::ffi::c_int
    {
        if regname < 256 as ::core::ffi::c_int {
            regname = langmap_mapchar[regname as usize] as ::core::ffi::c_int;
        } else {
            regname = langmap_adjust_mb(regname);
        }
    }
    if regname == Ctrl_R || regname == Ctrl_O || regname == Ctrl_P {
        literally = regname;
        add_to_showcmd_c(literally);
        regname = plain_vgetc();
        if *p_langmap as ::core::ffi::c_int != 0
            && true
            && (p_lrm != 0
                || (if vgetc_busy != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed == 0
            && regname >= 0 as ::core::ffi::c_int
        {
            if regname < 256 as ::core::ffi::c_int {
                regname = langmap_mapchar[regname as usize] as ::core::ffi::c_int;
            } else {
                regname = langmap_adjust_mb(regname);
            }
        }
    }
    no_mapping -= 1;
    allow_keys -= 1;
    no_u_sync += 1;
    if regname == '=' as ::core::ffi::c_int {
        let mut curpos: pos_T = (*curwin).w_cursor;
        u_sync_once = 2 as ::core::ffi::c_int;
        regname = get_expr_register();
        (*curwin).w_cursor = curpos;
        check_cursor(curwin);
    }
    if regname == NUL || !valid_yank_reg(regname, false_0 != 0) {
        vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
        need_redraw = true_0 != 0;
    } else {
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        if literally == Ctrl_O || literally == Ctrl_P {
            AppendCharToRedobuff(Ctrl_R);
            AppendCharToRedobuff(literally);
            AppendCharToRedobuff(regname);
            do_put(
                regname,
                ::core::ptr::null_mut::<yankreg_T>(),
                BACKWARD as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                (if literally == Ctrl_P {
                    PUT_FIXINDENT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | PUT_CURSEND as ::core::ffi::c_int,
            );
        } else if (*reg).y_size > 1 as size_t
            && is_literal_register(regname) as ::core::ffi::c_int != 0
        {
            AppendCharToRedobuff(Ctrl_R);
            AppendCharToRedobuff(regname);
            do_put(
                regname,
                ::core::ptr::null_mut::<yankreg_T>(),
                BACKWARD as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                PUT_CURSEND as ::core::ffi::c_int,
            );
        } else if insert_reg(
            regname,
            ::core::ptr::null_mut::<yankreg_T>(),
            literally != 0,
        ) == FAIL
        {
            vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
            need_redraw = true_0 != 0;
        } else if stop_insert_mode {
            need_redraw = true_0 != 0;
        }
    }
    no_u_sync -= 1;
    if u_sync_once == 1 as ::core::ffi::c_int {
        ins_need_undo = true_0 != 0;
    }
    u_sync_once = 0 as ::core::ffi::c_int;
    if need_redraw as ::core::ffi::c_int != 0 || stuff_empty() as ::core::ffi::c_int != 0 {
        edit_unputchar();
    }
    clear_showcmd();
    if vis_active == 0 && VIsual_active as ::core::ffi::c_int != 0 {
        end_visual_mode();
    }
}
unsafe extern "C" fn ins_ctrl_g() {
    setcursor();
    no_mapping += 1;
    allow_keys += 1;
    let mut c: ::core::ffi::c_int = plain_vgetc();
    no_mapping -= 1;
    allow_keys -= 1;
    match c {
        K_UP | Ctrl_K | 107 => {
            ins_up(true_0 != 0);
        }
        K_DOWN | Ctrl_J | 106 => {
            ins_down(true_0 != 0);
        }
        117 => {
            u_sync(true_0 != 0);
            ins_need_undo = true_0 != 0;
            update_Insstart_orig = false_0 != 0;
            Insstart = (*curwin).w_cursor;
        }
        85 => {
            dont_sync_undo = kNone;
        }
        ESC => {}
        _ => {
            vim_beep(kOptBoFlagCtrlg as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
    };
}
unsafe extern "C" fn ins_ctrl_hat() {
    if map_to_exists_mode(
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        MODE_LANGMAP as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        if State & MODE_LANGMAP as ::core::ffi::c_int != 0 {
            (*curbuf).b_p_iminsert = B_IMODE_NONE as OptInt;
            State &= !(MODE_LANGMAP as ::core::ffi::c_int);
        } else {
            (*curbuf).b_p_iminsert = B_IMODE_LMAP as OptInt;
            State |= MODE_LANGMAP as ::core::ffi::c_int;
        }
    }
    set_iminsert_global(curbuf);
    showmode();
    status_redraw_curbuf();
}
unsafe extern "C" fn ins_esc(
    mut count: *mut ::core::ffi::c_int,
    mut cmdchar: ::core::ffi::c_int,
    mut nomove: bool,
) -> bool {
    static mut disabled_redraw: bool = false_0 != 0;
    check_spell_redraw();
    let mut temp: ::core::ffi::c_int = (*curwin).w_cursor.col as ::core::ffi::c_int;
    if disabled_redraw {
        RedrawingDisabled -= 1;
        disabled_redraw = false_0 != 0;
    }
    if !arrow_used {
        if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
            AppendToRedobuff(ESC_STR.as_ptr());
        }
        if *count > 0 as ::core::ffi::c_int {
            line_breakcheck();
            if got_int {
                *count = 0 as ::core::ffi::c_int;
            }
        }
        *count -= 1;
        if *count > 0 as ::core::ffi::c_int {
            if !vim_strchr(p_cpo, CPO_REPLCNT).is_null() {
                State &= !(REPLACE_FLAG as ::core::ffi::c_int);
            }
            start_redo_ins();
            if cmdchar == 'r' as ::core::ffi::c_int || cmdchar == 'v' as ::core::ffi::c_int {
                stuffRedoReadbuff(ESC_STR.as_ptr());
            }
            RedrawingDisabled += 1;
            disabled_redraw = true_0 != 0;
            return false_0 != 0;
        }
        stop_insert(
            &raw mut (*curwin).w_cursor,
            true_0,
            nomove as ::core::ffi::c_int,
        );
        undisplay_dollar();
    }
    if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
        ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
    }
    if restart_edit == NUL && temp == (*curwin).w_cursor.col {
        (*curwin).w_set_curswant = true_0;
    }
    if cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut view: fmarkv_T = mark_view_make(curwin, (*curwin).w_cursor);
        let fmarkp___: *mut fmark_T = &raw mut (*curbuf).b_last_insert;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = (*curwin).w_cursor;
        (*fmarkp__).fnum = (*curbuf).handle as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = view;
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
    if !nomove
        && ((*curwin).w_cursor.col != 0 as ::core::ffi::c_int
            || (*curwin).w_cursor.coladd > 0 as ::core::ffi::c_int)
        && (restart_edit == NUL || gchar_cursor() == NUL && !VIsual_active)
        && !revins_on
    {
        if (*curwin).w_cursor.coladd > 0 as ::core::ffi::c_int
            || get_ve_flags(curwin) == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            oneleft();
            if restart_edit != NUL {
                (*curwin).w_cursor.coladd += 1;
            }
        } else {
            (*curwin).w_cursor.col -= 1;
            (*curwin).w_valid &= !(VALID_WCOL | VALID_VIRTCOL);
            mb_adjust_cursor();
        }
    }
    State = MODE_NORMAL as ::core::ffi::c_int;
    may_trigger_modechanged();
    if gchar_cursor() == TAB || buf_meta_total(curbuf, kMTMetaInline) > 0 as uint32_t {
        (*curwin).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
    setmouse();
    ui_cursor_shape();
    if reg_recording != 0 as ::core::ffi::c_int || restart_edit != NUL {
        showmode();
    } else if p_smd != 0
        && (got_int as ::core::ffi::c_int != 0 || !skip_showmode())
        && !(p_ch == 0 as OptInt && !ui_has(kUIMessages))
    {
        unshowmode(false_0 != 0);
    }
    return true_0 != 0;
}
unsafe extern "C" fn ins_ctrl_() {
    if revins_on as ::core::ffi::c_int != 0
        && revins_chars != 0
        && revins_scol >= 0 as ::core::ffi::c_int
    {
        while gchar_cursor() != NUL && {
            let c2rust_fresh4 = revins_chars;
            revins_chars = revins_chars - 1;
            c2rust_fresh4 != 0
        } {
            (*curwin).w_cursor.col += 1;
        }
    }
    p_ri = (p_ri == 0) as ::core::ffi::c_int;
    revins_on = State == MODE_INSERT as ::core::ffi::c_int && p_ri != 0;
    if revins_on {
        revins_scol = (*curwin).w_cursor.col as ::core::ffi::c_int;
        revins_legal += 1;
        revins_chars = 0 as ::core::ffi::c_int;
        undisplay_dollar();
    } else {
        revins_scol = -1 as ::core::ffi::c_int;
    }
    showmode();
}
unsafe extern "C" fn ins_start_select(mut c: ::core::ffi::c_int) -> bool {
    if !km_startsel {
        return false_0 != 0;
    }
    's_78: {
        match c {
            K_KHOME | K_KEND | K_PAGEUP | K_KPAGEUP | K_PAGEDOWN | K_KPAGEDOWN => {
                if mod_mask & MOD_MASK_SHIFT == 0 {
                    break 's_78;
                }
            }
            K_S_LEFT | K_S_RIGHT | K_S_UP | K_S_DOWN | K_S_END | K_S_HOME => {}
            _ => {
                break 's_78;
            }
        }
        start_selection();
        stuffcharReadbuff(Ctrl_O);
        if mod_mask != 0 {
            let buf: [::core::ffi::c_char; 4] = [
                K_SPECIAL as ::core::ffi::c_char,
                KS_MODIFIER as ::core::ffi::c_char,
                mod_mask as uint8_t as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            stuffReadbuffLen(&raw const buf as *const ::core::ffi::c_char, 3 as ptrdiff_t);
        }
        stuffcharReadbuff(c);
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ins_insert(mut replaceState: ::core::ffi::c_int) {
    set_vim_var_string(
        VV_INSERTMODE,
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        } else if replaceState == MODE_VREPLACE as ::core::ffi::c_int {
            b"v\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        },
        1 as ptrdiff_t,
    );
    ins_apply_autocmds(EVENT_INSERTCHANGE);
    if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        State = MODE_INSERT as ::core::ffi::c_int | State & MODE_LANGMAP as ::core::ffi::c_int;
    } else {
        State = replaceState | State & MODE_LANGMAP as ::core::ffi::c_int;
    }
    may_trigger_modechanged();
    AppendCharToRedobuff(K_INS);
    showmode();
    ui_cursor_shape();
}
unsafe extern "C" fn ins_ctrl_o() {
    restart_VIsual_select = 0 as ::core::ffi::c_int;
    if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
        restart_edit = 'V' as ::core::ffi::c_int;
    } else if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        restart_edit = 'R' as ::core::ffi::c_int;
    } else {
        restart_edit = 'I' as ::core::ffi::c_int;
    }
    if virtual_active(curwin) {
        ins_at_eol = false_0 != 0;
    } else {
        ins_at_eol = gchar_cursor() == NUL;
    };
}
unsafe extern "C" fn ins_shift(mut c: ::core::ffi::c_int, mut lastc: ::core::ffi::c_int) {
    if stop_arrow() == FAIL {
        return;
    }
    AppendCharToRedobuff(c);
    if c == Ctrl_D
        && (lastc == '0' as ::core::ffi::c_int || lastc == '^' as ::core::ffi::c_int)
        && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
    {
        (*curwin).w_cursor.col -= 1;
        del_char(false_0 != 0);
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            replace_pop_ins();
        }
        if lastc == '^' as ::core::ffi::c_int {
            old_indent = get_indent();
        }
        change_indent(
            INDENT_SET as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0,
            true_0 != 0,
        );
    } else {
        change_indent(
            if c == Ctrl_D {
                INDENT_DEC as ::core::ffi::c_int
            } else {
                INDENT_INC as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
            true_0,
            true_0 != 0,
        );
    }
    if did_ai as ::core::ffi::c_int != 0
        && *skipwhite(get_cursor_line_ptr()) as ::core::ffi::c_int != NUL
    {
        did_ai = false_0 != 0;
    }
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    can_cindent = false_0 != 0;
}
unsafe extern "C" fn ins_del() {
    if stop_arrow() == FAIL {
        return;
    }
    if gchar_cursor() == NUL {
        let temp: ::core::ffi::c_int = (*curwin).w_cursor.col as ::core::ffi::c_int;
        if !can_bs(BS_EOL)
            || do_join(
                2 as size_t,
                false_0 != 0,
                true_0 != 0,
                false_0 != 0,
                false_0 != 0,
            ) == FAIL
        {
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
        } else {
            (*curwin).w_cursor.col = temp as colnr_T;
            if State & VREPLACE_FLAG as ::core::ffi::c_int != 0
                && orig_line_count > (*curbuf).b_ml.ml_line_count
            {
                orig_line_count = (*curbuf).b_ml.ml_line_count;
            }
        }
    } else if del_char(false_0 != 0) == FAIL {
        vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    did_ai = false_0 != 0;
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    AppendCharToRedobuff(K_DEL);
}
unsafe extern "C" fn ins_bs(
    mut c: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut inserted_space_p: *mut ::core::ffi::c_int,
) -> bool {
    let mut cc: ::core::ffi::c_int = 0;
    let mut temp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_backspace: bool = false_0 != 0;
    let mut call_fix_indent: bool = false_0 != 0;
    if buf_is_empty(curbuf) as ::core::ffi::c_int != 0
        || !revins_on
            && ((*curwin).w_cursor.lnum == 1 as linenr_T
                && (*curwin).w_cursor.col == 0 as ::core::ffi::c_int
                || !can_bs(BS_START)
                    && (arrow_used as ::core::ffi::c_int != 0 && !bt_prompt(curbuf)
                        || (*curwin).w_cursor.lnum == Insstart_orig.lnum
                            && (*curwin).w_cursor.col <= Insstart_orig.col)
                || !can_bs(BS_INDENT)
                    && !arrow_used
                    && ai_col > 0 as ::core::ffi::c_int
                    && (*curwin).w_cursor.col <= ai_col
                || !can_bs(BS_EOL) && (*curwin).w_cursor.col == 0 as ::core::ffi::c_int)
    {
        vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
        return false_0 != 0;
    }
    if stop_arrow() == FAIL {
        return false_0 != 0;
    }
    let mut in_indent: bool = inindent(0 as ::core::ffi::c_int);
    if in_indent {
        can_cindent = false_0 != 0;
    }
    end_comment_pending = NUL;
    if revins_on {
        inc_cursor();
    }
    if (*curwin).w_cursor.coladd > 0 as ::core::ffi::c_int {
        if mode == BACKSPACE_CHAR as ::core::ffi::c_int {
            (*curwin).w_cursor.coladd -= 1;
            return true_0 != 0;
        }
        if mode == BACKSPACE_WORD as ::core::ffi::c_int {
            (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        }
        (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*curwin).w_cursor.col == 0 as ::core::ffi::c_int {
        let mut lnum: linenr_T = Insstart.lnum;
        if (*curwin).w_cursor.lnum == lnum || revins_on as ::core::ffi::c_int != 0 {
            if u_save(
                (*curwin).w_cursor.lnum - 2 as linenr_T,
                (*curwin).w_cursor.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return false_0 != 0;
            }
            Insstart.lnum -= 1;
            Insstart.col = ml_get_len(Insstart.lnum);
        }
        cc = -1 as ::core::ffi::c_int;
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
            cc = replace_pop_if_nul();
        }
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0 && (*curwin).w_cursor.lnum <= lnum {
            dec_cursor();
        } else {
            if State & VREPLACE_FLAG as ::core::ffi::c_int == 0
                || (*curwin).w_cursor.lnum > orig_line_count
            {
                temp = gchar_cursor();
                (*curwin).w_cursor.lnum -= 1;
                if has_format_option(FO_AUTO) as ::core::ffi::c_int != 0
                    && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
                {
                    let mut ptr: *const ::core::ffi::c_char =
                        ml_get_buf(curbuf, (*curwin).w_cursor.lnum);
                    let mut len: ::core::ffi::c_int = get_cursor_line_len();
                    if len > 0 as ::core::ffi::c_int
                        && *ptr.offset((len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                    {
                        let mut newp: *mut ::core::ffi::c_char = xmemdupz(
                            ptr as *const ::core::ffi::c_void,
                            (len - 1 as ::core::ffi::c_int) as size_t,
                        )
                            as *mut ::core::ffi::c_char;
                        if (*curbuf).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                            xfree((*curbuf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                        }
                        (*curbuf).b_ml.ml_line_ptr = newp;
                        (*curbuf).b_ml.ml_line_textlen -= 1;
                        (*curbuf).b_ml.ml_flags |= ML_LINE_DIRTY;
                    }
                }
                do_join(
                    2 as size_t,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                );
                if temp == NUL && gchar_cursor() != NUL {
                    inc_cursor();
                }
            } else {
                dec_cursor();
            }
            if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                let mut oldState: ::core::ffi::c_int = State;
                State = MODE_NORMAL as ::core::ffi::c_int;
                while cc > 0 as ::core::ffi::c_int {
                    let mut save_col: colnr_T = (*curwin).w_cursor.col;
                    mb_replace_pop_ins();
                    (*curwin).w_cursor.col = save_col;
                    cc = replace_pop_if_nul();
                }
                replace_pop_ins();
                State = oldState;
            }
        }
        did_ai = false_0 != 0;
    } else {
        if revins_on {
            dec_cursor();
        }
        let mut mincol: colnr_T = 0 as colnr_T;
        if mode == BACKSPACE_LINE as ::core::ffi::c_int
            && ((*curbuf).b_p_ai != 0 || cindent_on() as ::core::ffi::c_int != 0)
            && !revins_on
        {
            let mut save_col_0: colnr_T = (*curwin).w_cursor.col;
            beginline(BL_WHITE as ::core::ffi::c_int);
            if (*curwin).w_cursor.col < save_col_0 {
                mincol = (*curwin).w_cursor.col;
                call_fix_indent = true_0 != 0;
            }
            (*curwin).w_cursor.col = save_col_0;
        }
        if mode == BACKSPACE_CHAR as ::core::ffi::c_int
            && (p_sta != 0 && in_indent as ::core::ffi::c_int != 0
                || (get_sts_value() != 0 as ::core::ffi::c_int
                    || tabstop_count((*curbuf).b_p_vsts_array) != 0)
                    && (*curwin).w_cursor.col > 0 as ::core::ffi::c_int
                    && (*get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_int
                        == TAB
                        || *get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                            && (*inserted_space_p == 0 || arrow_used as ::core::ffi::c_int != 0)))
        {
            *inserted_space_p = false_0;
            let use_ts: bool =
                (*curwin).w_onebuf_opt.wo_list == 0 || (*curwin).w_p_lcs_chars.tab1 != 0;
            let line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let cursor_ptr: *mut ::core::ffi::c_char = line.offset((*curwin).w_cursor.col as isize);
            let mut vcol: colnr_T = 0 as colnr_T;
            let mut space_vcol: colnr_T = 0 as colnr_T;
            let mut sci: StrCharInfo = utf_ptr2StrCharInfo(line);
            let mut space_sci: StrCharInfo = sci;
            let mut prev_space: bool = false_0 != 0;
            while sci.ptr < cursor_ptr {
                let mut cur_space: bool = ascii_iswhite(sci.chr.value as ::core::ffi::c_int);
                if !prev_space && cur_space as ::core::ffi::c_int != 0 {
                    space_sci = sci;
                    space_vcol = vcol;
                }
                vcol += charsize_nowrap(curbuf, sci.ptr, use_ts, vcol, sci.chr.value);
                sci = utfc_next(sci);
                prev_space = cur_space;
            }
            let mut want_vcol: colnr_T = if vcol > 0 as ::core::ffi::c_int {
                vcol - 1 as colnr_T
            } else {
                0 as colnr_T
            };
            if p_sta != 0 && in_indent as ::core::ffi::c_int != 0 {
                want_vcol -= want_vcol as ::core::ffi::c_int % get_sw_value(curbuf);
            } else {
                want_vcol = tabstop_start(want_vcol, get_sts_value(), (*curbuf).b_p_vsts_array);
            }
            loop {
                let mut size: ::core::ffi::c_int = charsize_nowrap(
                    curbuf,
                    space_sci.ptr,
                    use_ts,
                    space_vcol,
                    space_sci.chr.value,
                );
                if space_vcol as ::core::ffi::c_int + size > want_vcol {
                    break;
                }
                space_vcol += size;
                space_sci = utfc_next(space_sci);
            }
            let want_col: colnr_T = space_sci.ptr.offset_from(line) as colnr_T;
            while (*curwin).w_cursor.col > want_col {
                dec_cursor();
                if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    if (*curwin).w_cursor.lnum != Insstart.lnum
                        || (*curwin).w_cursor.col >= Insstart.col
                    {
                        replace_do_bs(-1 as ::core::ffi::c_int);
                    }
                } else {
                    del_char(false_0 != 0);
                }
            }
            while space_vcol < want_vcol {
                if (*curwin).w_cursor.lnum == Insstart_orig.lnum
                    && (*curwin).w_cursor.col < Insstart_orig.col
                {
                    Insstart_orig.col = (*curwin).w_cursor.col;
                }
                if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                    ins_char(' ' as ::core::ffi::c_int);
                } else {
                    ins_str(
                        b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    );
                    if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                        replace_push_nul();
                    }
                }
                space_vcol += 1;
            }
        } else {
            let mut cclass: ::core::ffi::c_int = mb_get_class(get_cursor_pos_ptr());
            loop {
                if !revins_on {
                    dec_cursor();
                }
                cc = gchar_cursor();
                let mut prev_cclass: ::core::ffi::c_int = cclass;
                cclass = mb_get_class(get_cursor_pos_ptr());
                if mode == BACKSPACE_WORD as ::core::ffi::c_int && !ascii_isspace(cc) {
                    mode = BACKSPACE_WORD_NOT_SPACE as ::core::ffi::c_int;
                    temp = vim_iswordc(cc) as ::core::ffi::c_int;
                } else if mode == BACKSPACE_WORD_NOT_SPACE as ::core::ffi::c_int
                    && (ascii_isspace(cc) as ::core::ffi::c_int != 0
                        || vim_iswordc(cc) as ::core::ffi::c_int != temp
                        || prev_cclass != cclass)
                {
                    if !revins_on {
                        inc_cursor();
                    } else if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                        dec_cursor();
                    }
                    break;
                }
                if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    replace_do_bs(-1 as ::core::ffi::c_int);
                } else {
                    let mut has_composing: bool = false_0 != 0;
                    if p_deco != 0 {
                        let mut p0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        has_composing = utf_composinglike(
                            p0,
                            p0.offset(utf_ptr2len(p0) as isize),
                            ::core::ptr::null_mut::<GraphemeState>(),
                        );
                    }
                    del_char(false_0 != 0);
                    if has_composing {
                        inc_cursor();
                    }
                    if revins_chars != 0 {
                        revins_chars -= 1;
                        revins_legal += 1;
                    }
                    if revins_on as ::core::ffi::c_int != 0 && gchar_cursor() == NUL {
                        break;
                    }
                }
                if mode == BACKSPACE_CHAR as ::core::ffi::c_int {
                    break;
                }
                if !(revins_on as ::core::ffi::c_int != 0
                    || (*curwin).w_cursor.col > mincol
                        && (can_bs(BS_NOSTOP) as ::core::ffi::c_int != 0
                            || ((*curwin).w_cursor.lnum != Insstart_orig.lnum
                                || (*curwin).w_cursor.col != Insstart_orig.col)))
                {
                    break;
                }
            }
        }
        did_backspace = true_0 != 0;
    }
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    if (*curwin).w_cursor.col <= 1 as ::core::ffi::c_int {
        did_ai = false_0 != 0;
    }
    if call_fix_indent {
        fix_indent();
    }
    AppendCharToRedobuff(c);
    if (*curwin).w_cursor.lnum == Insstart_orig.lnum && (*curwin).w_cursor.col < Insstart_orig.col {
        Insstart_orig.col = (*curwin).w_cursor.col;
    }
    if !vim_strchr(p_cpo, CPO_BACKSPACE).is_null() && dollar_vcol == -1 as ::core::ffi::c_int {
        dollar_vcol = (*curwin).w_virtcol;
    }
    if did_backspace {
        foldOpenCursor();
    }
    return did_backspace;
}
unsafe extern "C" fn ins_left() {
    let end_change: bool = dont_sync_undo as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if oneleft() == OK {
        start_arrow_with_change(&raw mut tpos, end_change);
        if !end_change {
            AppendCharToRedobuff(K_LEFT);
        }
        if revins_scol != -1 as ::core::ffi::c_int && (*curwin).w_cursor.col >= revins_scol {
            revins_legal += 1;
        }
        revins_chars += 1;
    } else if !vim_strchr(p_ww, '[' as ::core::ffi::c_int).is_null()
        && (*curwin).w_cursor.lnum > 1 as linenr_T
    {
        start_arrow(&raw mut tpos);
        (*curwin).w_cursor.lnum -= 1;
        coladvance(curwin, MAXCOL as ::core::ffi::c_int);
        (*curwin).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo = kFalse;
}
unsafe extern "C" fn ins_home(mut c: ::core::ffi::c_int) {
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if c == -(253 as ::core::ffi::c_int
        + ((KE_C_HOME as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*curwin).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    (*curwin).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
    start_arrow(&raw mut tpos);
}
unsafe extern "C" fn ins_end(mut c: ::core::ffi::c_int) {
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if c == -(253 as ::core::ffi::c_int
        + ((KE_C_END as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
    }
    coladvance(curwin, MAXCOL as ::core::ffi::c_int);
    (*curwin).w_curswant = MAXCOL as ::core::ffi::c_int as colnr_T;
    start_arrow(&raw mut tpos);
}
unsafe extern "C" fn ins_s_left() {
    let end_change: bool = dont_sync_undo as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if (*curwin).w_cursor.lnum > 1 as linenr_T || (*curwin).w_cursor.col > 0 as ::core::ffi::c_int {
        start_arrow_with_change(&raw mut (*curwin).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_LEFT);
        }
        bck_word(1 as ::core::ffi::c_int, false_0 != 0, false_0 != 0);
        (*curwin).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo = kFalse;
}
unsafe extern "C" fn ins_right() {
    let end_change: bool = dont_sync_undo as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if gchar_cursor() != NUL || virtual_active(curwin) as ::core::ffi::c_int != 0 {
        start_arrow_with_change(&raw mut (*curwin).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_RIGHT);
        }
        (*curwin).w_set_curswant = true_0;
        if virtual_active(curwin) {
            oneright();
        } else {
            (*curwin).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
        }
        revins_legal += 1;
        if revins_chars != 0 {
            revins_chars -= 1;
        }
    } else if !vim_strchr(p_ww, ']' as ::core::ffi::c_int).is_null()
        && (*curwin).w_cursor.lnum < (*curbuf).b_ml.ml_line_count
    {
        start_arrow(&raw mut (*curwin).w_cursor);
        (*curwin).w_set_curswant = true_0;
        (*curwin).w_cursor.lnum += 1;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo = kFalse;
}
unsafe extern "C" fn ins_s_right() {
    let end_change: bool = dont_sync_undo as ::core::ffi::c_int == kFalse as ::core::ffi::c_int;
    if fdo_flags & kOptFdoFlagHor as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    undisplay_dollar();
    if (*curwin).w_cursor.lnum < (*curbuf).b_ml.ml_line_count || gchar_cursor() != NUL {
        start_arrow_with_change(&raw mut (*curwin).w_cursor, end_change);
        if !end_change {
            AppendCharToRedobuff(K_S_RIGHT);
        }
        fwd_word(1 as ::core::ffi::c_int, false_0 != 0, false);
        (*curwin).w_set_curswant = true_0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    dont_sync_undo = kFalse;
}
unsafe extern "C" fn ins_up(mut startcol: bool) {
    let mut old_topline: linenr_T = (*curwin).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*curwin).w_topfill;
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if cursor_up(1 as linenr_T, true_0 != 0) == OK {
        if startcol {
            coladvance(curwin, getvcol_nolist(&raw mut Insstart));
        }
        if old_topline != (*curwin).w_topline || old_topfill != (*curwin).w_topfill {
            redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
        }
        start_arrow(&raw mut tpos);
        can_cindent = true_0 != 0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_pageup() {
    undisplay_dollar();
    if mod_mask & MOD_MASK_CTRL != 0 {
        if !(*first_tabpage).tp_next.is_null() {
            start_arrow(&raw mut (*curwin).w_cursor);
            goto_tabpage(-1 as ::core::ffi::c_int);
        }
        return;
    }
    let mut tpos: pos_T = (*curwin).w_cursor;
    if pagescroll(BACKWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
        start_arrow(&raw mut tpos);
        can_cindent = true_0 != 0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_down(mut startcol: bool) {
    let mut old_topline: linenr_T = (*curwin).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*curwin).w_topfill;
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin).w_cursor;
    if cursor_down(1 as ::core::ffi::c_int, true_0 != 0) == OK {
        if startcol {
            coladvance(curwin, getvcol_nolist(&raw mut Insstart));
        }
        if old_topline != (*curwin).w_topline || old_topfill != (*curwin).w_topfill {
            redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
        }
        start_arrow(&raw mut tpos);
        can_cindent = true_0 != 0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_pagedown() {
    undisplay_dollar();
    if mod_mask & MOD_MASK_CTRL != 0 {
        if !(*first_tabpage).tp_next.is_null() {
            start_arrow(&raw mut (*curwin).w_cursor);
            goto_tabpage(0 as ::core::ffi::c_int);
        }
        return;
    }
    let mut tpos: pos_T = (*curwin).w_cursor;
    if pagescroll(FORWARD, 1 as ::core::ffi::c_int, false_0 != 0) == OK {
        start_arrow(&raw mut tpos);
        can_cindent = true_0 != 0;
    } else {
        vim_beep(kOptBoFlagCursor as ::core::ffi::c_int as ::core::ffi::c_uint);
    };
}
unsafe extern "C" fn ins_tab() -> bool {
    let mut temp: ::core::ffi::c_int = 0;
    if Insstart_blank_vcol == MAXCOL as ::core::ffi::c_int
        && (*curwin).w_cursor.lnum == Insstart.lnum
    {
        Insstart_blank_vcol = get_nolist_virtcol();
    }
    if echeck_abbr(TAB + ABBR_OFF) {
        return false_0 != 0;
    }
    let mut ind: bool = inindent(0 as ::core::ffi::c_int);
    if ind {
        can_cindent = false_0 != 0;
    }
    if (*curbuf).b_p_et == 0
        && !(p_sta != 0
            && ind as ::core::ffi::c_int != 0
            && (tabstop_count((*curbuf).b_p_vts_array) > 1 as ::core::ffi::c_int
                || tabstop_count((*curbuf).b_p_vts_array) == 1 as ::core::ffi::c_int
                    && tabstop_first((*curbuf).b_p_vts_array) != get_sw_value(curbuf)
                || tabstop_count((*curbuf).b_p_vts_array) == 0 as ::core::ffi::c_int
                    && (*curbuf).b_p_ts != get_sw_value(curbuf) as OptInt))
        && tabstop_count((*curbuf).b_p_vsts_array) == 0 as ::core::ffi::c_int
        && get_sts_value() == 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if stop_arrow() == FAIL {
        return true_0 != 0;
    }
    did_ai = false_0 != 0;
    did_si = false_0 != 0;
    can_si = false_0 != 0;
    can_si_back = false_0 != 0;
    AppendToRedobuff(b"\t\0".as_ptr() as *const ::core::ffi::c_char);
    if p_sta != 0 && ind as ::core::ffi::c_int != 0 {
        temp = get_sw_value(curbuf);
        temp -= get_nolist_virtcol() % temp;
    } else if tabstop_count((*curbuf).b_p_vsts_array) > 0 as ::core::ffi::c_int
        || (*curbuf).b_p_sts != 0 as OptInt
    {
        temp = tabstop_padding(
            get_nolist_virtcol(),
            get_sts_value() as OptInt,
            (*curbuf).b_p_vsts_array,
        );
    } else {
        temp = tabstop_padding(
            get_nolist_virtcol(),
            (*curbuf).b_p_ts,
            (*curbuf).b_p_vts_array,
        );
    }
    ins_char(' ' as ::core::ffi::c_int);
    loop {
        temp -= 1;
        if temp <= 0 as ::core::ffi::c_int {
            break;
        }
        if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            ins_char(' ' as ::core::ffi::c_int);
        } else {
            ins_str(
                b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
            if State & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                replace_push_nul();
            }
        }
    }
    if (*curbuf).b_p_et == 0
        && (tabstop_count((*curbuf).b_p_vsts_array) > 0 as ::core::ffi::c_int
            || get_sts_value() > 0 as ::core::ffi::c_int
            || p_sta != 0 && ind as ::core::ffi::c_int != 0)
    {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut saved_line: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut cursor: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut want_vcol: colnr_T = 0;
        let mut vcol: colnr_T = 0;
        let mut change_col: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut save_list: ::core::ffi::c_int = (*curwin).w_onebuf_opt.wo_list;
        if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            pos = (*curwin).w_cursor;
            cursor = &raw mut pos;
            saved_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            ptr = saved_line.offset(pos.col as isize);
        } else {
            ptr = get_cursor_pos_ptr();
            cursor = &raw mut (*curwin).w_cursor;
        }
        if vim_strchr(p_cpo, CPO_LISTWM).is_null() {
            (*curwin).w_onebuf_opt.wo_list = false_0;
        }
        let mut fpos: pos_T = (*curwin).w_cursor;
        while fpos.col > 0 as ::core::ffi::c_int
            && ascii_iswhite(*ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            fpos.col -= 1;
            ptr = ptr.offset(-1);
        }
        if State & REPLACE_FLAG as ::core::ffi::c_int != 0
            && fpos.lnum == Insstart.lnum
            && fpos.col < Insstart.col
        {
            ptr = ptr.offset((Insstart.col - fpos.col) as isize);
            fpos.col = Insstart.col;
        }
        getvcol(
            curwin,
            &raw mut fpos,
            &raw mut vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        getvcol(
            curwin,
            cursor,
            &raw mut want_vcol,
            ::core::ptr::null_mut::<colnr_T>(),
            ::core::ptr::null_mut::<colnr_T>(),
        );
        let mut tab: *mut ::core::ffi::c_char =
            b"\t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut tab_v: int32_t = *tab as uint8_t as int32_t;
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
        let mut cstype: CSType = init_charsize_arg(&raw mut csarg, curwin, 0 as linenr_T, tab);
        while ascii_iswhite(*ptr as ::core::ffi::c_int) {
            let mut i: ::core::ffi::c_int = win_charsize(
                cstype,
                vcol as ::core::ffi::c_int,
                tab,
                tab_v,
                &raw mut csarg,
            )
            .width;
            if vcol as ::core::ffi::c_int + i > want_vcol {
                break;
            }
            if *ptr as ::core::ffi::c_int != TAB {
                *ptr = TAB as ::core::ffi::c_char;
                if change_col < 0 as ::core::ffi::c_int {
                    change_col = fpos.col as ::core::ffi::c_int;
                    if fpos.lnum == Insstart.lnum && fpos.col < Insstart.col {
                        Insstart.col = fpos.col;
                    }
                }
            }
            fpos.col += 1;
            ptr = ptr.offset(1);
            vcol += i;
        }
        if change_col >= 0 as ::core::ffi::c_int {
            let mut repl_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            cstype = init_charsize_arg(&raw mut csarg, curwin, 0 as linenr_T, ptr);
            while vcol < want_vcol && *ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                vcol += win_charsize(
                    cstype,
                    vcol as ::core::ffi::c_int,
                    ptr,
                    ' ' as ::core::ffi::c_int as uint8_t as int32_t,
                    &raw mut csarg,
                )
                .width;
                ptr = ptr.offset(1);
                repl_off += 1;
            }
            if vcol > want_vcol {
                ptr = ptr.offset(-1);
                repl_off -= 1;
            }
            fpos.col += repl_off;
            let mut i_0: ::core::ffi::c_int =
                (*cursor).col as ::core::ffi::c_int - fpos.col as ::core::ffi::c_int;
            if i_0 > 0 as ::core::ffi::c_int {
                if State & VREPLACE_FLAG as ::core::ffi::c_int == 0 {
                    let newp_len: colnr_T = (*curbuf).b_ml.ml_line_textlen - i_0 as colnr_T;
                    let mut newp: *mut ::core::ffi::c_char =
                        xmalloc(newp_len as size_t) as *mut ::core::ffi::c_char;
                    let mut col: ptrdiff_t = ptr.offset_from((*curbuf).b_ml.ml_line_ptr);
                    if col > 0 as ptrdiff_t {
                        memmove(
                            newp as *mut ::core::ffi::c_void,
                            ptr.offset(-(col as isize)) as *const ::core::ffi::c_void,
                            col as size_t,
                        );
                    }
                    memmove(
                        newp.offset(col as isize) as *mut ::core::ffi::c_void,
                        ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                        (newp_len as ptrdiff_t - col) as size_t,
                    );
                    if (*curbuf).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                        xfree((*curbuf).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                    }
                    (*curbuf).b_ml.ml_line_ptr = newp;
                    (*curbuf).b_ml.ml_line_textlen = newp_len;
                    (*curbuf).b_ml.ml_flags = ((*curbuf).b_ml.ml_flags | ML_LINE_DIRTY) & !ML_EMPTY;
                    inserted_bytes(
                        fpos.lnum,
                        change_col as colnr_T,
                        (*cursor).col as ::core::ffi::c_int - change_col,
                        fpos.col as ::core::ffi::c_int - change_col,
                    );
                } else {
                    memmove(
                        ptr as *mut ::core::ffi::c_void,
                        ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                        strlen(ptr.offset(i_0 as isize)).wrapping_add(1 as size_t),
                    );
                }
                if State & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && State & VREPLACE_FLAG as ::core::ffi::c_int == 0
                {
                    temp = i_0;
                    loop {
                        temp -= 1;
                        if temp < 0 as ::core::ffi::c_int {
                            break;
                        }
                        replace_join(repl_off);
                    }
                }
            }
            (*cursor).col -= i_0;
            if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                backspace_until_column(change_col);
                ins_bytes_len(
                    saved_line.offset(change_col as isize),
                    ((*cursor).col as ::core::ffi::c_int - change_col) as size_t,
                );
            }
        }
        if State & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            xfree(saved_line as *mut ::core::ffi::c_void);
        }
        (*curwin).w_onebuf_opt.wo_list = save_list;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ins_eol(mut c: ::core::ffi::c_int) -> bool {
    if echeck_abbr(c + ABBR_OFF) {
        return true_0 != 0;
    }
    if stop_arrow() == FAIL {
        return false_0 != 0;
    }
    undisplay_dollar();
    if State & REPLACE_FLAG as ::core::ffi::c_int != 0
        && State & VREPLACE_FLAG as ::core::ffi::c_int == 0
    {
        replace_push_nul();
    }
    if virtual_active(curwin) as ::core::ffi::c_int != 0
        && (*curwin).w_cursor.coladd > 0 as ::core::ffi::c_int
    {
        coladvance(curwin, getviscol());
    }
    if revins_on {
        (*curwin).w_cursor.col += get_cursor_pos_len();
    }
    AppendToRedobuff(NL_STR.as_ptr());
    let mut i: bool = open_line(
        FORWARD as ::core::ffi::c_int,
        if has_format_option(FO_RET_COMS) as ::core::ffi::c_int != 0 {
            OPENLINE_DO_COM as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        old_indent,
        ::core::ptr::null_mut::<bool>(),
    );
    old_indent = 0 as ::core::ffi::c_int;
    can_cindent = true_0 != 0;
    foldOpenCursor();
    return i;
}
unsafe extern "C" fn ins_digraph() -> ::core::ffi::c_int {
    let mut did_putchar: bool = false_0 != 0;
    pc_status = PC_STATUS_UNSET;
    if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
        ins_redraw(false_0 != 0);
        edit_putchar('?' as ::core::ffi::c_int, true_0 != 0);
        did_putchar = true_0 != 0;
        add_to_showcmd_c(Ctrl_K);
    }
    no_mapping += 1;
    allow_keys += 1;
    let mut c: ::core::ffi::c_int = plain_vgetc();
    no_mapping -= 1;
    allow_keys -= 1;
    if did_putchar {
        edit_unputchar();
    }
    if c < 0 as ::core::ffi::c_int || mod_mask != 0 {
        clear_showcmd();
        insert_special(c, true_0, false_0);
        return NUL;
    }
    if c != ESC {
        did_putchar = false_0 != 0;
        if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
            ins_redraw(false_0 != 0);
            if char2cells(c) == 1 as ::core::ffi::c_int {
                ins_redraw(false_0 != 0);
                edit_putchar(c, true_0 != 0);
                did_putchar = true_0 != 0;
            }
            add_to_showcmd_c(c);
        }
        no_mapping += 1;
        allow_keys += 1;
        let mut cc: ::core::ffi::c_int = plain_vgetc();
        no_mapping -= 1;
        allow_keys -= 1;
        if did_putchar {
            edit_unputchar();
        }
        if cc != ESC {
            AppendToRedobuff(CTRL_V_STR.as_ptr());
            c = digraph_get(c, cc, true_0 != 0);
            clear_showcmd();
            return c;
        }
    }
    clear_showcmd();
    return NUL;
}
#[no_mangle]
pub unsafe extern "C" fn ins_copychar(mut lnum: linenr_T) -> ::core::ffi::c_int {
    if lnum < 1 as linenr_T || lnum > (*curbuf).b_ml.ml_line_count {
        vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
        return NUL;
    }
    validate_virtcol(curwin);
    let end_vcol: ::core::ffi::c_int = (*curwin).w_virtcol as ::core::ffi::c_int;
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
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
    let mut cstype: CSType = init_charsize_arg(&raw mut csarg, curwin, lnum, line);
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while vcol < end_vcol && *ci.ptr as ::core::ffi::c_int != NUL {
        vcol += win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
        if vcol > end_vcol {
            break;
        }
        ci = utfc_next(ci);
    }
    let mut c: ::core::ffi::c_int = if ci.chr.value < 0 as int32_t {
        *ci.ptr as uint8_t as ::core::ffi::c_int
    } else {
        ci.chr.value as ::core::ffi::c_int
    };
    if c == NUL {
        vim_beep(kOptBoFlagCopy as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    return c;
}
unsafe extern "C" fn ins_ctrl_ey(mut tc: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = tc;
    if ctrl_x_mode_scroll() {
        if c == Ctrl_Y {
            scrolldown_clamp();
        } else {
            scrollup_clamp();
        }
        redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
    } else {
        c = ins_copychar(
            (*curwin).w_cursor.lnum
                + (if c == Ctrl_Y {
                    -1 as linenr_T
                } else {
                    1 as linenr_T
                }),
        );
        if c != NUL {
            if c < 256 as ::core::ffi::c_int
                && *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
                    & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
            {
                AppendToRedobuff(CTRL_V_STR.as_ptr());
            }
            let mut tw_save: OptInt = (*curbuf).b_p_tw;
            (*curbuf).b_p_tw = -1 as OptInt;
            insert_special(c, true_0, false_0);
            (*curbuf).b_p_tw = tw_save;
            revins_chars += 1;
            revins_legal += 1;
            c = Ctrl_V;
            auto_format(false_0 != 0, true_0 != 0);
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn get_nolist_virtcol() -> colnr_T {
    if (*curwin).w_buffer.is_null()
        || (*(*curwin).w_buffer).b_ml.ml_mfp.is_null()
        || (*curwin).w_cursor.lnum > (*(*curwin).w_buffer).b_ml.ml_line_count
    {
        return 0 as colnr_T;
    }
    if (*curwin).w_onebuf_opt.wo_list != 0 && vim_strchr(p_cpo, CPO_LISTWM).is_null() {
        return getvcol_nolist(&raw mut (*curwin).w_cursor);
    }
    validate_virtcol(curwin);
    return (*curwin).w_virtcol;
}
unsafe extern "C" fn do_insert_char_pre(mut c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
    let save_State: ::core::ffi::c_int = State;
    if c == Ctrl_RSB {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !has_event(EVENT_INSERTCHARPRE) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buflen: size_t = utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as size_t;
    buf[buflen as usize] = NUL as ::core::ffi::c_char;
    textlock += 1;
    set_vim_var_string(
        VV_CHAR,
        &raw mut buf as *mut ::core::ffi::c_char,
        buflen as ptrdiff_t,
    );
    let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if ins_apply_autocmds(EVENT_INSERTCHARPRE) != 0 {
        if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            get_vim_var_str(VV_CHAR),
        ) != 0 as ::core::ffi::c_int
        {
            res = xstrdup(get_vim_var_str(VV_CHAR));
        }
    }
    set_vim_var_string(
        VV_CHAR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    textlock -= 1;
    State = save_State;
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn get_can_cindent() -> bool {
    return can_cindent;
}
#[no_mangle]
pub unsafe extern "C" fn set_can_cindent(mut val: bool) {
    can_cindent = val;
}
#[no_mangle]
pub unsafe extern "C" fn ins_apply_autocmds(mut event: event_T) -> ::core::ffi::c_int {
    let mut tick: varnumber_T = buf_get_changedtick(curbuf);
    let mut r: ::core::ffi::c_int = apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    ) as ::core::ffi::c_int;
    if event as ::core::ffi::c_uint
        != EVENT_INSERTLEAVE as ::core::ffi::c_int as ::core::ffi::c_uint
        && tick != buf_get_changedtick(curbuf)
    {
        u_save(
            (*curwin).w_cursor.lnum,
            (*curwin).w_cursor.lnum + 1 as linenr_T,
        );
    }
    return r;
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int = -22783;
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_DOWN: ::core::ffi::c_int = -25707;
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_UP: ::core::ffi::c_int = -1277;
pub const K_S_DOWN: ::core::ffi::c_int = -1533;
pub const K_S_LEFT: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_RIGHT: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('i' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_HOME: ::core::ffi::c_int = -12835;
pub const K_S_END: ::core::ffi::c_int = -14122;
pub const K_S_TAB: ::core::ffi::c_int = -17003;
pub const K_XF1: ::core::ffi::c_int = -14845;
pub const K_F1: ::core::ffi::c_int = -12651;
pub const K_HELP: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('1' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_INS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('I' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KINS: ::core::ffi::c_int = -20477;
pub const K_DEL: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KDEL: ::core::ffi::c_int = -20733;
pub const K_HOME: ::core::ffi::c_int = -26731;
pub const K_KHOME: ::core::ffi::c_int = -12619;
pub const K_END: ::core::ffi::c_int = -14144;
pub const K_KEND: ::core::ffi::c_int = -13387;
pub const K_PAGEUP: ::core::ffi::c_int = -20587;
pub const K_PAGEDOWN: ::core::ffi::c_int = -20075;
pub const K_KPAGEUP: ::core::ffi::c_int = -13131;
pub const K_KPAGEDOWN: ::core::ffi::c_int = -13643;
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PASTE_START: ::core::ffi::c_int = -21328;
pub const K_SELECT: ::core::ffi::c_int = -22773;
pub const K_LEFTMOUSE: ::core::ffi::c_int = -11517;
pub const K_LEFTMOUSE_NM: ::core::ffi::c_int = -17917;
pub const K_LEFTDRAG: ::core::ffi::c_int = -11773;
pub const K_LEFTRELEASE: ::core::ffi::c_int = -12029;
pub const K_LEFTRELEASE_NM: ::core::ffi::c_int = -18173;
pub const K_MOUSEMOVE: ::core::ffi::c_int = -25853;
pub const K_MIDDLEMOUSE: ::core::ffi::c_int = -12285;
pub const K_MIDDLEDRAG: ::core::ffi::c_int = -12541;
pub const K_MIDDLERELEASE: ::core::ffi::c_int = -12797;
pub const K_RIGHTMOUSE: ::core::ffi::c_int = -13053;
pub const K_RIGHTDRAG: ::core::ffi::c_int = -13309;
pub const K_RIGHTRELEASE: ::core::ffi::c_int = -13565;
pub const K_X1MOUSE: ::core::ffi::c_int = -23037;
pub const K_X1DRAG: ::core::ffi::c_int = -23293;
pub const K_X1RELEASE: ::core::ffi::c_int = -23549;
pub const K_X2MOUSE: ::core::ffi::c_int = -23805;
pub const K_X2DRAG: ::core::ffi::c_int = -24061;
pub const K_X2RELEASE: ::core::ffi::c_int = -24317;
pub const K_MOUSEDOWN: ::core::ffi::c_int = -19453;
pub const K_MOUSEUP: ::core::ffi::c_int = -19709;
pub const K_MOUSELEFT: ::core::ffi::c_int = -19965;
pub const K_MOUSERIGHT: ::core::ffi::c_int = -20221;
pub const K_COMMAND: ::core::ffi::c_int = -26877;
pub const K_LUA: ::core::ffi::c_int = -26621;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_CMD: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
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
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
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
#[inline(always)]
unsafe extern "C" fn linetabsize_str(mut s: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    return linetabsize_col(0 as ::core::ffi::c_int, s);
}
#[inline]
unsafe extern "C" fn is_literal_register(regname: ::core::ffi::c_int) -> bool {
    return regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
        || (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
