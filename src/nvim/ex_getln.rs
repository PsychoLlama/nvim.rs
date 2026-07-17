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
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_mem_free(mem: ArenaMem);
    fn mh_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, new: *mut MHPutStatus) -> uint32_t;
    fn mh_get_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> uint32_t;
    fn nvim_create_namespace(name: String_0) -> Integer;
    fn nvim_create_buf(listed: Boolean, scratch: Boolean, err: *mut Error) -> Buffer;
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn api_free_array(value: Array);
    fn api_clear_error(value: *mut Error);
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
    fn block_autocmds();
    fn unblock_autocmds();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buf_valid(buf: *mut buf_T) -> bool;
    fn close_buffer(
        win: *mut win_T,
        buf: *mut buf_T,
        action: ::core::ffi::c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;
    fn buf_clear();
    fn do_buffer(
        action: ::core::ffi::c_int,
        start: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buf_open_scratch(bufnr: handle_T, bufname: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn buf_set_changedtick(buf: *mut buf_T, changedtick: varnumber_T);
    static mut p_cedit: *mut ::core::ffi::c_char;
    static mut p_cwh: OptInt;
    static mut p_ch: OptInt;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_hls: ::core::ffi::c_int;
    static mut p_arshape: ::core::ffi::c_int;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_is: ::core::ffi::c_int;
    static mut p_icm: *mut ::core::ffi::c_char;
    static mut p_paste: ::core::ffi::c_int;
    static mut p_ari: ::core::ffi::c_int;
    static mut p_ru: ::core::ffi::c_int;
    static mut p_stl: *mut ::core::ffi::c_char;
    static mut p_wbr: *mut ::core::ffi::c_char;
    static mut p_scs: ::core::ffi::c_int;
    static mut p_tal: *mut ::core::ffi::c_char;
    static mut p_tbidi: ::core::ffi::c_int;
    static mut p_wc: OptInt;
    static mut p_wcm: OptInt;
    static mut p_wim: *mut ::core::ffi::c_char;
    static mut p_wmnu: ::core::ffi::c_int;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_str2nr(
        start: *const ::core::ffi::c_char,
        prep: *mut ::core::ffi::c_int,
        len: *mut ::core::ffi::c_int,
        what: ::core::ffi::c_int,
        nptr: *mut varnumber_T,
        unptr: *mut uvarnumber_T,
        maxlen: ::core::ffi::c_int,
        strict: bool,
        overflow: *mut bool,
    );
    fn save_batch_count() -> ::core::ffi::c_int;
    fn restore_batch_count(save_count: ::core::ffi::c_int);
    fn nextwild(
        xp: *mut expand_T,
        type_0: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        escape: bool,
    ) -> ::core::ffi::c_int;
    fn cmdline_pum_active() -> bool;
    fn cmdline_pum_remove(defer_redraw: bool);
    fn cmdline_pum_cleanup(cclp: *mut CmdlineInfo);
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    fn ExpandCleanup(xp: *mut expand_T);
    fn clear_cmdline_orig();
    fn showmatches(
        xp: *mut expand_T,
        display_wildmenu: bool,
        display_list: bool,
        noselect: bool,
    ) -> ::core::ffi::c_int;
    fn set_expand_context(xp: *mut expand_T);
    fn wildmenu_translate_key(
        cclp: *mut CmdlineInfo,
        key: ::core::ffi::c_int,
        xp: *mut expand_T,
        did_wild_list: bool,
    ) -> ::core::ffi::c_int;
    fn wildmenu_process_key(
        cclp: *mut CmdlineInfo,
        key: ::core::ffi::c_int,
        xp: *mut expand_T,
    ) -> ::core::ffi::c_int;
    fn wildmenu_cleanup(cclp: *mut CmdlineInfo);
    fn get_hislen() -> ::core::ffi::c_int;
    fn get_histentry(hist_type: ::core::ffi::c_int) -> *mut histentry_T;
    fn get_hisidx(hist_type: ::core::ffi::c_int) -> *mut ::core::ffi::c_int;
    fn hist_char2type(c: ::core::ffi::c_int) -> HistoryType;
    fn init_history();
    fn add_to_history(
        histype: ::core::ffi::c_int,
        new_entry: *const ::core::ffi::c_char,
        new_entrylen: size_t,
        in_map: bool,
        sep: ::core::ffi::c_int,
    );
    fn do_digraph(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn get_digraph(cmdline: bool) -> ::core::ffi::c_int;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn gchar_cursor() -> ::core::ffi::c_int;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_line_len() -> colnr_T;
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor();
    fn redraw_custom_title_later() -> bool;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn set_must_redraw(type_0: ::core::ffi::c_int);
    fn status_redraw_all();
    fn status_redraw_curbuf();
    fn redraw_statuslines();
    static e_cmdwin: [::core::ffi::c_char; 0];
    static e_command_too_recursive: [::core::ffi::c_char; 0];
    static e_intern2: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_positive: [::core::ffi::c_char; 0];
    static e_textlock: [::core::ffi::c_char; 0];
    static e_cannot_edit_other_buf: [::core::ffi::c_char; 0];
    fn get_literal(no_simplify: bool) -> ::core::ffi::c_int;
    fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T;
    fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T);
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn get_echo_hl_id() -> ::core::ffi::c_int;
    fn eval_has_provider(feat: *const ::core::ffi::c_char, throw_if_fast: bool) -> bool;
    fn msg_grid_validate();
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn msg_start();
    fn msg_starthere();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans_len(
        msgstr: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_puts_len(
        str: *const ::core::ffi::c_char,
        len: ptrdiff_t,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    );
    fn msg_cursor_goto(row: ::core::ffi::c_int, col: ::core::ffi::c_int);
    fn sb_text_start_cmdline();
    fn sb_text_restart_cmdline();
    fn sb_text_end_cmdline();
    fn msg_clr_eos();
    fn msg_check();
    fn tv_list_free(l: *mut list_T);
    fn callback_free(callback: *mut Callback);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
    fn tv_dict_get_string_buf_chk(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        key_len: ptrdiff_t,
        numbuf: *mut ::core::ffi::c_char,
        def: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_dict_get_callback(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: ptrdiff_t,
        result: *mut Callback,
    ) -> bool;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_bool(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: BoolVarValue,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_str(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_dict_set_keys_readonly(dict: *mut dict_T);
    fn tv_clear(tv: *mut typval_T);
    fn tv_copy(from: *const typval_T, to: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_string_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_check_for_opt_number_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string_buf_chk(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn heredoc_get(
        eap: *mut exarg_T,
        cmd: *mut ::core::ffi::c_char,
        script_get_0: bool,
    ) -> *mut list_T;
    fn get_globvar_dict() -> *mut dict_T;
    fn set_vim_var_char(c: ::core::ffi::c_int);
    fn rename_buffer(new_fname: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn parse_cmdline(
        cmdline: *mut *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        cmdinfo: *mut CmdParseInfo,
        errormsg: *mut *const ::core::ffi::c_char,
    ) -> bool;
    fn execute_cmd(
        eap: *mut exarg_T,
        cmdinfo: *mut CmdParseInfo,
        preview: bool,
    ) -> ::core::ffi::c_int;
    fn parse_command_modifiers(
        eap: *mut exarg_T,
        errormsg: *mut *const ::core::ffi::c_char,
        cmod: *mut cmdmod_T,
        skip_only: bool,
    ) -> ::core::ffi::c_int;
    fn undo_cmdmod(cmod: *mut cmdmod_T);
    fn parse_cmd_address(
        eap: *mut exarg_T,
        errormsg: *mut *const ::core::ffi::c_char,
        silent: bool,
    ) -> ::core::ffi::c_int;
    fn skip_range(
        cmd: *const ::core::ffi::c_char,
        ctx: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn expr_map_locked() -> bool;
    fn set_no_hlsearch(flag: bool);
    fn aborting() -> bool;
    fn extmark_clear(
        buf: *mut buf_T,
        ns_id: uint32_t,
        l_row: ::core::ffi::c_int,
        l_col: colnr_T,
        u_row: ::core::ffi::c_int,
        u_col: colnr_T,
    ) -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn stuff_empty() -> bool;
    fn beep_flush();
    fn stuffReadbuff(s: *const ::core::ffi::c_char);
    fn stuffReadbuffSpec(s: *const ::core::ffi::c_char);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn ins_typebuf(
        str: *mut ::core::ffi::c_char,
        noremap: ::core::ffi::c_int,
        offset: ::core::ffi::c_int,
        nottyped: bool,
        silent: bool,
    ) -> ::core::ffi::c_int;
    fn vgetc() -> ::core::ffi::c_int;
    fn plain_vgetc() -> ::core::ffi::c_int;
    fn vpeekc() -> ::core::ffi::c_int;
    fn vpeekc_any() -> ::core::ffi::c_int;
    fn char_avail() -> bool;
    fn vungetc(c: ::core::ffi::c_int);
    fn getcmdkeycmd(
        promptc: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn map_execute_lua(may_repeat: bool, discard: bool) -> bool;
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut mod_mask: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut cmdline_star: ::core::ffi::c_int;
    static mut redrawing_cmdline: bool;
    static mut cmdline_was_last_drawn: bool;
    static mut exec_from_reg: bool;
    static mut cmdmsg_rl: bool;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didout: bool;
    static mut emsg_off: ::core::ffi::c_int;
    static mut did_emsg: ::core::ffi::c_int;
    static mut emsg_on_display: bool;
    static mut need_wait_return: bool;
    static mut quit_more: bool;
    static mut lines_left: ::core::ffi::c_int;
    static mut msg_no_more: bool;
    static mut current_sctx: sctx_T;
    static mut highlight_match: bool;
    static mut search_match_lines: linenr_T;
    static mut search_match_endcol: colnr_T;
    static mut search_first_line: linenr_T;
    static mut search_last_line: linenr_T;
    static mut mouse_row: ::core::ffi::c_int;
    static mut mouse_col: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut redraw_tabline: bool;
    static mut curbuf: *mut buf_T;
    static mut textlock: ::core::ffi::c_int;
    static mut allbuf_lock: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut exmode_active: bool;
    static mut cmdpreview: bool;
    static mut no_mapping: ::core::ffi::c_int;
    static mut allow_keys: ::core::ffi::c_int;
    static mut restart_edit: ::core::ffi::c_int;
    static mut no_abbr: bool;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut cmd_silent: bool;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut typebuf: typebuf_T;
    static mut ex_normal_busy: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut KeyStuffed: ::core::ffi::c_int;
    static mut skip_redraw: bool;
    static mut got_int: bool;
    static mut global_busy: ::core::ffi::c_int;
    static mut new_last_cmdline: *mut ::core::ffi::c_char;
    static mut redir_off: bool;
    static mut wild_menu_showing: ::core::ffi::c_int;
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut cmdwin_result: ::core::ffi::c_int;
    static mut cmdwin_level: ::core::ffi::c_int;
    static mut cmdwin_buf: *mut buf_T;
    static mut cmdwin_win: *mut win_T;
    static mut cmdwin_old_curwin: *mut win_T;
    static mut cmdline_win: *mut win_T;
    static mut wim_flags: [uint8_t; 4];
    static mut no_hlsearch: bool;
    static mut magic_overruled: optmagic_T;
    static mut skip_win_fix_cursor: bool;
    fn syn_name2id(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn get_special_key_name(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
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
    fn add_map(
        lhs: *mut ::core::ffi::c_char,
        rhs: *mut ::core::ffi::c_char,
        mode: ::core::ffi::c_int,
        buffer: bool,
    );
    fn setpcmark();
    fn mb_get_class(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_cptr2char_adv(pp: *mut *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_iscomposing_first(c: ::core::ffi::c_int) -> bool;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_off_next(
        base: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_prevptr(
        line: *mut ::core::ffi::c_char,
        p: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    static utf8len_tab_zero: [uint8_t; 256];
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace(lnum: linenr_T, line: *mut ::core::ffi::c_char, copy: bool)
        -> ::core::ffi::c_int;
    fn incl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn decl(lp: *mut pos_T) -> ::core::ffi::c_int;
    fn setmouse();
    fn update_topline(wp: *mut win_T);
    fn changed_cline_bef_curs(wp: *mut win_T);
    fn changed_line_abv_curs();
    fn invalidate_botline_win(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn normal_enter(cmdwin: bool, noexmode: bool);
    fn clear_showcmd();
    fn string_to_key(arg: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn set_iminsert_global(buf: *mut buf_T);
    fn set_imsearch_global(buf: *mut buf_T);
    fn magic_isset() -> bool;
    fn csh_like_shell() -> ::core::ffi::c_int;
    fn line_breakcheck();
    fn home_replace_save(
        buf: *mut buf_T,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    static mut pum_want: C2Rust_Unnamed_51;
    fn pum_undisplay(immediate: bool);
    fn pum_check_clear();
    fn skip_regexp_ex(
        startp: *mut ::core::ffi::c_char,
        dirc: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
        newp: *mut *mut ::core::ffi::c_char,
        dropped: *mut ::core::ffi::c_int,
        magic_val: *mut magic_T,
    ) -> *mut ::core::ffi::c_char;
    fn profile_setlimit(msec: int64_t) -> proftime_T;
    fn get_expr_register() -> ::core::ffi::c_int;
    fn get_expr_line() -> *mut ::core::ffi::c_char;
    fn valid_yank_reg(regname: ::core::ffi::c_int, writing: bool) -> bool;
    fn get_spec_reg(
        regname: ::core::ffi::c_int,
        argp: *mut *mut ::core::ffi::c_char,
        allocated: *mut bool,
        errmsg: bool,
    ) -> bool;
    fn cmdline_paste_reg(regname: ::core::ffi::c_int, literally_arg: bool, remcr: bool) -> bool;
    fn save_search_patterns();
    fn restore_search_patterns();
    fn save_last_search_pattern();
    fn restore_last_search_pattern();
    fn last_search_pattern() -> *mut ::core::ffi::c_char;
    fn last_search_pattern_len() -> size_t;
    fn pat_has_uppercase(pat: *mut ::core::ffi::c_char) -> bool;
    fn searchit(
        win: *mut win_T,
        buf: *mut buf_T,
        pos: *mut pos_T,
        end_pos: *mut pos_T,
        dir: Direction,
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        count: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        pat_use: ::core::ffi::c_int,
        extra_arg: *mut searchit_arg_T,
    ) -> ::core::ffi::c_int;
    fn do_search(
        oap: *mut oparg_T,
        dirc: ::core::ffi::c_int,
        search_delim: ::core::ffi::c_int,
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        count: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        sia: *mut searchit_arg_T,
    ) -> ::core::ffi::c_int;
    fn ui_busy_start();
    fn ui_busy_stop();
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_flush();
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn state_enter(s: *mut VimState);
    fn state_handle_k_event();
    fn may_trigger_modechanged();
    fn may_trigger_safestate(safe: bool);
    fn ui_call_cmdline_show(
        content: Array,
        pos: Integer,
        firstc: String_0,
        prompt: String_0,
        indent: Integer,
        level: Integer,
        hl_id: Integer,
    );
    fn ui_call_cmdline_pos(pos: Integer, level: Integer);
    fn ui_call_cmdline_special_char(c: String_0, shift: Boolean, level: Integer);
    fn ui_call_cmdline_hide(level: Integer, abort_0: Boolean);
    fn ui_call_cmdline_block_show(lines: Array);
    fn ui_call_cmdline_block_append(lines: Array);
    fn ui_call_cmdline_block_hide();
    fn cmdcomplete_type_to_str(
        expand: ::core::ffi::c_int,
        compl_arg: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn parse_compl_arg(
        value: *const ::core::ffi::c_char,
        vallen: ::core::ffi::c_int,
        complp: *mut ::core::ffi::c_int,
        argt: *mut uint32_t,
        compl_arg: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn u_undo_and_forget(count: ::core::ffi::c_int, do_buf_event: bool) -> bool;
    fn u_sync(force: bool);
    fn u_clearall(buf: *mut buf_T);
    fn u_blockfree(buf: *mut buf_T);
    fn parser_simple_get_line(cookie: *mut ::core::ffi::c_void, ret_pline: *mut ParserLine);
    fn viml_parser_destroy(pstate: *mut ParserState);
    fn viml_pexpr_free_ast(ast: ExprAST);
    fn viml_pexpr_parse(pstate: *mut ParserState, flags: ::core::ffi::c_int) -> ExprAST;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn close_windows(buf: *mut buf_T, keep_curwin: bool);
    fn last_window(win: *mut win_T) -> bool;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn win_goto(wp: *mut win_T);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_size_save(gap: *mut garray_T);
    fn win_size_restore(gap: *mut garray_T);
    fn global_stl_height() -> ::core::ffi::c_int;
    fn lastwin_nofloating(tp: *mut tabpage_T) -> *mut win_T;
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
pub type ptrdiff_t = isize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type ApiDispatchWrapper =
    Option<unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Float = ::core::ffi::c_double;
pub type Integer = int64_t;
pub type Boolean = bool;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
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
pub type Buffer = handle_T;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type uvarnumber_T = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_16 = 76;
pub const HLF_PRE: C2Rust_Unnamed_16 = 75;
pub const HLF_OK: C2Rust_Unnamed_16 = 74;
pub const HLF_SO: C2Rust_Unnamed_16 = 73;
pub const HLF_SE: C2Rust_Unnamed_16 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_16 = 71;
pub const HLF_TS: C2Rust_Unnamed_16 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_16 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_16 = 68;
pub const HLF_CU: C2Rust_Unnamed_16 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_16 = 66;
pub const HLF_WBR: C2Rust_Unnamed_16 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_16 = 64;
pub const HLF_MSG: C2Rust_Unnamed_16 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_16 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_16 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_16 = 60;
pub const HLF_0: C2Rust_Unnamed_16 = 59;
pub const HLF_QFL: C2Rust_Unnamed_16 = 58;
pub const HLF_MC: C2Rust_Unnamed_16 = 57;
pub const HLF_CUL: C2Rust_Unnamed_16 = 56;
pub const HLF_CUC: C2Rust_Unnamed_16 = 55;
pub const HLF_TPF: C2Rust_Unnamed_16 = 54;
pub const HLF_TPS: C2Rust_Unnamed_16 = 53;
pub const HLF_TP: C2Rust_Unnamed_16 = 52;
pub const HLF_PBR: C2Rust_Unnamed_16 = 51;
pub const HLF_PST: C2Rust_Unnamed_16 = 50;
pub const HLF_PSB: C2Rust_Unnamed_16 = 49;
pub const HLF_PSX: C2Rust_Unnamed_16 = 48;
pub const HLF_PNX: C2Rust_Unnamed_16 = 47;
pub const HLF_PSK: C2Rust_Unnamed_16 = 46;
pub const HLF_PNK: C2Rust_Unnamed_16 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_16 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_16 = 43;
pub const HLF_PSI: C2Rust_Unnamed_16 = 42;
pub const HLF_PNI: C2Rust_Unnamed_16 = 41;
pub const HLF_SPL: C2Rust_Unnamed_16 = 40;
pub const HLF_SPR: C2Rust_Unnamed_16 = 39;
pub const HLF_SPC: C2Rust_Unnamed_16 = 38;
pub const HLF_SPB: C2Rust_Unnamed_16 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_16 = 36;
pub const HLF_SC: C2Rust_Unnamed_16 = 35;
pub const HLF_TXA: C2Rust_Unnamed_16 = 34;
pub const HLF_TXD: C2Rust_Unnamed_16 = 33;
pub const HLF_DED: C2Rust_Unnamed_16 = 32;
pub const HLF_CHD: C2Rust_Unnamed_16 = 31;
pub const HLF_ADD: C2Rust_Unnamed_16 = 30;
pub const HLF_FC: C2Rust_Unnamed_16 = 29;
pub const HLF_FL: C2Rust_Unnamed_16 = 28;
pub const HLF_WM: C2Rust_Unnamed_16 = 27;
pub const HLF_W: C2Rust_Unnamed_16 = 26;
pub const HLF_VNC: C2Rust_Unnamed_16 = 25;
pub const HLF_V: C2Rust_Unnamed_16 = 24;
pub const HLF_T: C2Rust_Unnamed_16 = 23;
pub const HLF_VSP: C2Rust_Unnamed_16 = 22;
pub const HLF_C: C2Rust_Unnamed_16 = 21;
pub const HLF_SNC: C2Rust_Unnamed_16 = 20;
pub const HLF_S: C2Rust_Unnamed_16 = 19;
pub const HLF_R: C2Rust_Unnamed_16 = 18;
pub const HLF_CLF: C2Rust_Unnamed_16 = 17;
pub const HLF_CLS: C2Rust_Unnamed_16 = 16;
pub const HLF_CLN: C2Rust_Unnamed_16 = 15;
pub const HLF_LNB: C2Rust_Unnamed_16 = 14;
pub const HLF_LNA: C2Rust_Unnamed_16 = 13;
pub const HLF_N: C2Rust_Unnamed_16 = 12;
pub const HLF_CM: C2Rust_Unnamed_16 = 11;
pub const HLF_M: C2Rust_Unnamed_16 = 10;
pub const HLF_LC: C2Rust_Unnamed_16 = 9;
pub const HLF_L: C2Rust_Unnamed_16 = 8;
pub const HLF_I: C2Rust_Unnamed_16 = 7;
pub const HLF_E: C2Rust_Unnamed_16 = 6;
pub const HLF_D: C2Rust_Unnamed_16 = 5;
pub const HLF_AT: C2Rust_Unnamed_16 = 4;
pub const HLF_TERM: C2Rust_Unnamed_16 = 3;
pub const HLF_EOB: C2Rust_Unnamed_16 = 2;
pub const HLF_8: C2Rust_Unnamed_16 = 1;
pub const HLF_NONE: C2Rust_Unnamed_16 = 0;
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_ptr_t {
    pub h: MapHash,
    pub keys: *mut ptr_t,
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const XP_BS_COMMA: C2Rust_Unnamed_17 = 4;
pub const XP_BS_THREE: C2Rust_Unnamed_17 = 2;
pub const XP_BS_ONE: C2Rust_Unnamed_17 = 1;
pub const XP_BS_NONE: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_18 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_18 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_18 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_18 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_18 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_18 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_18 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_18 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_18 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_18 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_18 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_18 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_18 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_18 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_18 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_18 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_18 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_18 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_18 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_18 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_18 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_18 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_18 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_18 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_18 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_18 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_18 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_18 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_18 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_18 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_18 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_18 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_18 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_18 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_18 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_18 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_18 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_18 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_18 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_18 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_18 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_18 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_18 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_18 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_18 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_18 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_18 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_18 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_18 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_18 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_18 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_18 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_18 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_18 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_18 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_18 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_18 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_18 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_18 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_18 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_18 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_18 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_18 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_18 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_18 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_18 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_18 = -2;
pub type optmagic_T = ::core::ffi::c_uint;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub type magic_T = ::core::ffi::c_uint;
pub const MAGIC_ALL: magic_T = 4;
pub const MAGIC_ON: magic_T = 3;
pub const MAGIC_OFF: magic_T = 2;
pub const MAGIC_NONE: magic_T = 1;
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
pub struct optset_T {
    pub os_varp: *mut ::core::ffi::c_void,
    pub os_idx: OptIndex,
    pub os_flags: ::core::ffi::c_int,
    pub os_oldval: OptValData,
    pub os_newval: OptValData,
    pub os_value_checked: bool,
    pub os_value_changed: bool,
    pub os_restore_chartab: bool,
    pub os_errbuf: *mut ::core::ffi::c_char,
    pub os_errbuflen: size_t,
    pub os_win: *mut ::core::ffi::c_void,
    pub os_buf: *mut ::core::ffi::c_void,
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
    pub cs_pend: C2Rust_Unnamed_19,
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
pub union C2Rust_Unnamed_19 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
pub type except_T = vim_exception;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TryState {
    pub current_exception: *mut except_T,
    pub private_msg_list: *mut msglist_T,
    pub msg_list: *const *const msglist_T,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub did_emsg: ::core::ffi::c_int,
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
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_20 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_20 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_20 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_20 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_20 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_20 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_20 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_20 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_20 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_20 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_20 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_20 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_20 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_20 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CmdParseInfo {
    pub cmdmod: cmdmod_T,
    pub magic: C2Rust_Unnamed_21,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub file: bool,
    pub bar: bool,
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
pub type dobuf_action_values = ::core::ffi::c_uint;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub type dobuf_start_values = ::core::ffi::c_uint;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_22 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_22 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_22 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_22 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_22 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_22 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_22 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_22 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_22 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_22 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_22 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_22 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_22 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_22 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_22 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_22 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_22 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_22 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_22 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const kOptWimFlagNoselect: C2Rust_Unnamed_23 = 16;
pub const kOptWimFlagLastused: C2Rust_Unnamed_23 = 8;
pub const kOptWimFlagList: C2Rust_Unnamed_23 = 4;
pub const kOptWimFlagLongest: C2Rust_Unnamed_23 = 2;
pub const kOptWimFlagFull: C2Rust_Unnamed_23 = 1;
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
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_24 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_24 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_24 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_24 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_24 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_24 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_24 = 7;
pub const WILD_ALL: C2Rust_Unnamed_24 = 6;
pub const WILD_PREV: C2Rust_Unnamed_24 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_24 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_24 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_24 = 2;
pub const WILD_FREE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_25 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_25 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_25 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_25 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_25 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_25 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_25 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_25 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_25 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_25 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_25 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_25 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_25 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_25 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_25 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_25 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_25 = 1;
pub type HistoryType = ::core::ffi::c_int;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_INPUT: HistoryType = 3;
pub const HIST_EXPR: HistoryType = 2;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_INVALID: HistoryType = -1;
pub const HIST_DEFAULT: HistoryType = -2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct histentry_T {
    pub hisnum: ::core::ffi::c_int,
    pub hisstr: *mut ::core::ffi::c_char,
    pub hisstrlen: size_t,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_26 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_26 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_26 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_26 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_26 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_26 = 20;
pub const UPD_VALID: C2Rust_Unnamed_26 = 10;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_27 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_27 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_27 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_27 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_27 = 1;
pub const CONV_NONE: C2Rust_Unnamed_27 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typebuf_T {
    pub tb_buf: *mut uint8_t,
    pub tb_noremap: *mut uint8_t,
    pub tb_buflen: ::core::ffi::c_int,
    pub tb_off: ::core::ffi::c_int,
    pub tb_len: ::core::ffi::c_int,
    pub tb_maplen: ::core::ffi::c_int,
    pub tb_silent: ::core::ffi::c_int,
    pub tb_no_abbr_cnt: ::core::ffi::c_int,
    pub tb_change_cnt: ::core::ffi::c_int,
}
pub type RemapValues = ::core::ffi::c_int;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_28 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_28 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_28 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_28 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_28 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const VSE_BUFFER: C2Rust_Unnamed_29 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_29 = 1;
pub const VSE_NONE: C2Rust_Unnamed_29 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandLineState {
    pub state: VimState,
    pub firstc: ::core::ffi::c_int,
    pub count: ::core::ffi::c_int,
    pub indent: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
    pub gotesc: bool,
    pub do_abbr: bool,
    pub lookfor: *mut ::core::ffi::c_char,
    pub lookforlen: ::core::ffi::c_int,
    pub hiscnt: ::core::ffi::c_int,
    pub save_hiscnt: ::core::ffi::c_int,
    pub histype: ::core::ffi::c_int,
    pub is_state: incsearch_state_T,
    pub did_wild_list: bool,
    pub wim_index: ::core::ffi::c_int,
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_State: ::core::ffi::c_int,
    pub prev_cmdpos: ::core::ffi::c_int,
    pub prev_cmdbuff: *mut ::core::ffi::c_char,
    pub save_p_icm: *mut ::core::ffi::c_char,
    pub skip_pum_redraw: bool,
    pub some_key_typed: bool,
    pub ignore_drag_release: bool,
    pub break_ctrl_c: bool,
    pub xpc: expand_T,
    pub b_im_ptr: *mut OptInt,
    pub b_im_ptr_buf: *mut buf_T,
    pub cmdline_type: ::core::ffi::c_int,
    pub event_cmdlineleavepre_triggered: bool,
    pub did_hist_navigate: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct incsearch_state_T {
    pub search_start: pos_T,
    pub save_cursor: pos_T,
    pub winid: handle_T,
    pub init_viewstate: viewstate_T,
    pub old_viewstate: viewstate_T,
    pub match_start: pos_T,
    pub match_end: pos_T,
    pub did_incsearch: bool,
    pub incsearch_postponed: bool,
    pub magic_overruled_save: optmagic_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct viewstate_T {
    pub vs_curswant: colnr_T,
    pub vs_leftcol: colnr_T,
    pub vs_skipcol: colnr_T,
    pub vs_topline: linenr_T,
    pub vs_topfill: ::core::ffi::c_int,
    pub vs_botline: linenr_T,
    pub vs_empty_rows: ::core::ffi::c_int,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserHighlightChunk {
    pub start: ParserPosition,
    pub end_col: size_t,
    pub group: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserPosition {
    pub line: size_t,
    pub col: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserHighlight {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserHighlightChunk,
    pub init_array: [ParserHighlightChunk; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserState {
    pub reader: ParserInputReader,
    pub pos: ParserPosition,
    pub stack: C2Rust_Unnamed_30,
    pub colors: *mut ParserHighlight,
    pub can_continuate: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_30 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserStateItem,
    pub init_array: [ParserStateItem; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserStateItem {
    pub type_0: C2Rust_Unnamed_34,
    pub data: C2Rust_Unnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_31 {
    pub expr: C2Rust_Unnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_32 {
    pub type_0: C2Rust_Unnamed_33,
}
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const kExprUnknown: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const kPTopStateParsingExpression: C2Rust_Unnamed_34 = 1;
pub const kPTopStateParsingCommand: C2Rust_Unnamed_34 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserInputReader {
    pub get_line: ParserLineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub lines: C2Rust_Unnamed_35,
    pub conv: vimconv_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_35 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ParserLine,
    pub init_array: [ParserLine; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParserLine {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
    pub allocated: bool,
}
pub type ParserLineGetter =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ParserLine) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprAST {
    pub err: ExprASTError,
    pub root: *mut ExprASTNode,
}
pub type ExprASTNode = expr_ast_node;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expr_ast_node {
    pub type_0: ExprASTNodeType,
    pub children: *mut ExprASTNode,
    pub next: *mut ExprASTNode,
    pub start: ParserPosition,
    pub len: size_t,
    pub data: C2Rust_Unnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_36 {
    pub reg: C2Rust_Unnamed_48,
    pub fig: C2Rust_Unnamed_46,
    pub var: C2Rust_Unnamed_45,
    pub ter: C2Rust_Unnamed_44,
    pub cmp: C2Rust_Unnamed_43,
    pub num: C2Rust_Unnamed_42,
    pub flt: C2Rust_Unnamed_41,
    pub str: C2Rust_Unnamed_40,
    pub opt: C2Rust_Unnamed_39,
    pub env: C2Rust_Unnamed_38,
    pub ass: C2Rust_Unnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_37 {
    pub type_0: ExprAssignmentType,
}
pub type ExprAssignmentType = ::core::ffi::c_uint;
pub const kExprAsgnConcat: ExprAssignmentType = 3;
pub const kExprAsgnSubtract: ExprAssignmentType = 2;
pub const kExprAsgnAdd: ExprAssignmentType = 1;
pub const kExprAsgnPlain: ExprAssignmentType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_38 {
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_39 {
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
    pub scope: ExprOptScope,
}
pub type ExprOptScope = ::core::ffi::c_uint;
pub const kExprOptScopeLocal: ExprOptScope = 108;
pub const kExprOptScopeGlobal: ExprOptScope = 103;
pub const kExprOptScopeUnspecified: ExprOptScope = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_40 {
    pub value: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_41 {
    pub value: float_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_42 {
    pub value: uvarnumber_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_43 {
    pub type_0: ExprComparisonType,
    pub ccs: ExprCaseCompareStrategy,
    pub inv: bool,
}
pub type ExprCaseCompareStrategy = ::core::ffi::c_uint;
pub const kCCStrategyIgnoreCase: ExprCaseCompareStrategy = 63;
pub const kCCStrategyMatchCase: ExprCaseCompareStrategy = 35;
pub const kCCStrategyUseOption: ExprCaseCompareStrategy = 0;
pub type ExprComparisonType = ::core::ffi::c_uint;
pub const kExprCmpIdentical: ExprComparisonType = 4;
pub const kExprCmpGreaterOrEqual: ExprComparisonType = 3;
pub const kExprCmpGreater: ExprComparisonType = 2;
pub const kExprCmpMatches: ExprComparisonType = 1;
pub const kExprCmpEqual: ExprComparisonType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_44 {
    pub got_colon: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_45 {
    pub scope: ExprVarScope,
    pub ident: *const ::core::ffi::c_char,
    pub ident_len: size_t,
}
pub type ExprVarScope = ::core::ffi::c_uint;
pub const kExprVarScopeArguments: ExprVarScope = 97;
pub const kExprVarScopeLocal: ExprVarScope = 108;
pub const kExprVarScopeTabpage: ExprVarScope = 116;
pub const kExprVarScopeWindow: ExprVarScope = 119;
pub const kExprVarScopeBuffer: ExprVarScope = 98;
pub const kExprVarScopeVim: ExprVarScope = 118;
pub const kExprVarScopeGlobal: ExprVarScope = 103;
pub const kExprVarScopeScript: ExprVarScope = 115;
pub const kExprVarScopeMissing: ExprVarScope = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_46 {
    pub type_guesses: C2Rust_Unnamed_47,
    pub opening_hl_idx: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_47 {
    pub allow_dict: bool,
    pub allow_lambda: bool,
    pub allow_ident: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_48 {
    pub name: ::core::ffi::c_int,
}
pub type ExprASTNodeType = ::core::ffi::c_uint;
pub const kExprNodeAssignment: ExprASTNodeType = 38;
pub const kExprNodeEnvironment: ExprASTNodeType = 37;
pub const kExprNodeOption: ExprASTNodeType = 36;
pub const kExprNodeMod: ExprASTNodeType = 35;
pub const kExprNodeDivision: ExprASTNodeType = 34;
pub const kExprNodeMultiplication: ExprASTNodeType = 33;
pub const kExprNodeNot: ExprASTNodeType = 32;
pub const kExprNodeBinaryMinus: ExprASTNodeType = 31;
pub const kExprNodeUnaryMinus: ExprASTNodeType = 30;
pub const kExprNodeAnd: ExprASTNodeType = 29;
pub const kExprNodeOr: ExprASTNodeType = 28;
pub const kExprNodeDoubleQuotedString: ExprASTNodeType = 27;
pub const kExprNodeSingleQuotedString: ExprASTNodeType = 26;
pub const kExprNodeFloat: ExprASTNodeType = 25;
pub const kExprNodeInteger: ExprASTNodeType = 24;
pub const kExprNodeConcatOrSubscript: ExprASTNodeType = 23;
pub const kExprNodeConcat: ExprASTNodeType = 22;
pub const kExprNodeComparison: ExprASTNodeType = 21;
pub const kExprNodeArrow: ExprASTNodeType = 20;
pub const kExprNodeColon: ExprASTNodeType = 19;
pub const kExprNodeComma: ExprASTNodeType = 18;
pub const kExprNodeCurlyBracesIdentifier: ExprASTNodeType = 17;
pub const kExprNodeDictLiteral: ExprASTNodeType = 16;
pub const kExprNodeLambda: ExprASTNodeType = 15;
pub const kExprNodeUnknownFigure: ExprASTNodeType = 14;
pub const kExprNodeComplexIdentifier: ExprASTNodeType = 13;
pub const kExprNodePlainKey: ExprASTNodeType = 12;
pub const kExprNodePlainIdentifier: ExprASTNodeType = 11;
pub const kExprNodeCall: ExprASTNodeType = 10;
pub const kExprNodeNested: ExprASTNodeType = 9;
pub const kExprNodeBinaryPlus: ExprASTNodeType = 8;
pub const kExprNodeUnaryPlus: ExprASTNodeType = 7;
pub const kExprNodeListLiteral: ExprASTNodeType = 6;
pub const kExprNodeSubscript: ExprASTNodeType = 5;
pub const kExprNodeRegister: ExprASTNodeType = 4;
pub const kExprNodeTernaryValue: ExprASTNodeType = 3;
pub const kExprNodeTernary: ExprASTNodeType = 2;
pub const kExprNodeOpMissing: ExprASTNodeType = 1;
pub const kExprNodeMissing: ExprASTNodeType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExprASTError {
    pub msg: *const ::core::ffi::c_char,
    pub arg: *const ::core::ffi::c_char,
    pub arg_len: ::core::ffi::c_int,
}
pub const kExprFlagsDisallowEOC: ExprParserFlags = 2;
pub const MAX_CB_ERRORS: C2Rust_Unnamed_58 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchit_arg_T {
    pub sa_stop_lnum: linenr_T,
    pub sa_tm: *mut proftime_T,
    pub sa_timed_out: ::core::ffi::c_int,
    pub sa_wrapped: ::core::ffi::c_int,
}
pub const SEARCH_PEEK: C2Rust_Unnamed_54 = 2048;
pub const SEARCH_NOOF: C2Rust_Unnamed_54 = 128;
pub const SEARCH_OPT: C2Rust_Unnamed_54 = 16;
pub const SEARCH_START: C2Rust_Unnamed_54 = 256;
pub const SEARCH_KEEP: C2Rust_Unnamed_54 = 1024;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpInfo {
    pub win_info: C2Rust_Unnamed_50,
    pub buf_info: C2Rust_Unnamed_49,
    pub save_hls: bool,
    pub save_cmdmod: cmdmod_T,
    pub save_view: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_49 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpBufInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpBufInfo {
    pub buf: *mut buf_T,
    pub save_b_p_ul: OptInt,
    pub save_b_p_ma: ::core::ffi::c_int,
    pub save_b_changed: ::core::ffi::c_int,
    pub save_b_op_start: pos_T,
    pub save_b_op_end: pos_T,
    pub save_changedtick: varnumber_T,
    pub undo_info: CpUndoInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpUndoInfo {
    pub save_b_u_oldhead: *mut u_header_T,
    pub save_b_u_newhead: *mut u_header_T,
    pub save_b_u_curhead: *mut u_header_T,
    pub save_b_u_numhead: ::core::ffi::c_int,
    pub save_b_u_synced: bool,
    pub save_b_u_seq_last: ::core::ffi::c_int,
    pub save_b_u_save_nr_last: ::core::ffi::c_int,
    pub save_b_u_seq_cur: ::core::ffi::c_int,
    pub save_b_u_time_cur: time_t,
    pub save_b_u_save_nr_cur: ::core::ffi::c_int,
    pub save_b_u_line_ptr: *mut ::core::ffi::c_char,
    pub save_b_u_line_lnum: linenr_T,
    pub save_b_u_line_colnr: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_50 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CpWinInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CpWinInfo {
    pub win: *mut win_T,
    pub save_w_cursor: pos_T,
    pub save_viewstate: viewstate_T,
    pub save_w_p_cul: ::core::ffi::c_int,
    pub save_w_p_cuc: ::core::ffi::c_int,
}
pub const WSP_BOT: C2Rust_Unnamed_56 = 16;
pub const RE_SEARCH: C2Rust_Unnamed_55 = 0;
pub const SEARCH_COL: C2Rust_Unnamed_54 = 4096;
pub const GOTO_NORMAL_MODE: C2Rust_Unnamed_57 = 3;
pub const CMDLINE_CHANGED: C2Rust_Unnamed_57 = 2;
pub const CMDLINE_NOT_CHANGED: C2Rust_Unnamed_57 = 1;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_IGNORE: key_extra = 53;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_C_RIGHT: key_extra = 86;
pub const MODE_LANGMAP: C2Rust_Unnamed_52 = 32;
pub const KE_KINS: key_extra = 79;
pub const KE_KDEL: key_extra = 80;
pub const KE_WILD: key_extra = 108;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_NOP: key_extra = 97;
pub const MODE_NORMAL: C2Rust_Unnamed_52 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_53 = 2;
pub const MODE_INSERT: C2Rust_Unnamed_52 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_52 = 8;
pub const KE_CMDWIN: key_extra = 84;
pub const PROCESS_NEXT_KEY: C2Rust_Unnamed_57 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_51 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const KE_COMMAND: key_extra = 104;
pub const KE_EVENT: key_extra = 102;
pub const KE_LUA: key_extra = 103;
pub type C2Rust_Unnamed_52 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_52 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_52 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_52 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_52 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_52 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_52 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_52 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_52 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_52 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_52 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_52 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_52 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_52 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_52 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_52 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_52 = 2;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_DROP: key_extra = 95;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
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
pub type C2Rust_Unnamed_53 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_53 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_53 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_53 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_53 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_53 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_53 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_53 = 1;
pub type C2Rust_Unnamed_54 = ::core::ffi::c_uint;
pub const SEARCH_MARK: C2Rust_Unnamed_54 = 512;
pub const SEARCH_END: C2Rust_Unnamed_54 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_54 = 32;
pub const SEARCH_NFMSG: C2Rust_Unnamed_54 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_54 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_54 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_54 = 1;
pub type C2Rust_Unnamed_55 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_55 = 2;
pub const RE_BOTH: C2Rust_Unnamed_55 = 2;
pub const RE_SUBST: C2Rust_Unnamed_55 = 1;
pub type ExprParserFlags = ::core::ffi::c_uint;
pub const kExprFlagsParseLet: ExprParserFlags = 4;
pub const kExprFlagsMulti: ExprParserFlags = 1;
pub type C2Rust_Unnamed_56 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_56 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_56 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_56 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_56 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_56 = 64;
pub const WSP_HELP: C2Rust_Unnamed_56 = 32;
pub const WSP_TOP: C2Rust_Unnamed_56 = 8;
pub const WSP_HOR: C2Rust_Unnamed_56 = 4;
pub const WSP_VERT: C2Rust_Unnamed_56 = 2;
pub const WSP_ROOM: C2Rust_Unnamed_56 = 1;
pub type C2Rust_Unnamed_57 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_58 = ::core::ffi::c_uint;
static mut prev_prompt_id: ::core::ffi::c_uint = 0;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
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
pub const SET_INIT: Set_ptr_t = Set_ptr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4;
pub const Ctrl_E: ::core::ffi::c_int = 5;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_K: ::core::ffi::c_int = 11;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_Q: ::core::ffi::c_int = 17;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20;
pub const Ctrl_U: ::core::ffi::c_int = 21;
pub const Ctrl_V: ::core::ffi::c_int = 22;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const Ctrl_HAT: ::core::ffi::c_int = 30;
pub const Ctrl__: ::core::ffi::c_int = 31;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
pub const CPO_ESC: ::core::ffi::c_int = 'x' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
pub const B_IMODE_USE_INSERT: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const B_IMODE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut last_prompt_id: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
static mut ccline: CmdlineInfo = CmdlineInfo {
    cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    cmdbufflen: 0,
    cmdlen: 0,
    cmdpos: 0,
    cmdspos: 0,
    cmdfirstc: 0,
    cmdindent: 0,
    cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    hl_id: 0,
    overstrike: 0,
    xpc: ::core::ptr::null_mut::<expand_T>(),
    xp_context: 0,
    xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    input_fn: 0,
    cmdbuff_replaced: false,
    prompt_id: 0,
    highlight_callback: Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    },
    last_colors: ColoredCmdline {
        prompt_id: 0,
        cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        colors: CmdlineColors {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
        },
    },
    level: 0,
    prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
    special_char: 0,
    special_shift: false,
    redraw_state: kCmdRedrawNone,
    one_key: false,
    mouse_used: ::core::ptr::null_mut::<bool>(),
};
static mut new_cmdpos: ::core::ffi::c_int = 0;
static mut cmdline_block: Array = ARRAY_DICT_INIT;
static mut getln_interrupted_highlight: bool = false_0 != 0;
static mut cedit_key: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
static mut cmdpreview_bufnr: handle_T = 0 as handle_T;
static mut cmdpreview_ns: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut e_active_window_or_buffer_changed_or_deleted: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"E199: Active window or buffer changed or deleted\0",
    )
};
unsafe extern "C" fn trigger_cmd_autocmd(mut typechar: ::core::ffi::c_int, mut evt: event_T) {
    let mut typestr: [::core::ffi::c_char; 2] =
        [typechar as ::core::ffi::c_char, NUL as ::core::ffi::c_char];
    apply_autocmds(
        evt,
        &raw mut typestr as *mut ::core::ffi::c_char,
        &raw mut typestr as *mut ::core::ffi::c_char,
        false_0 != 0,
        curbuf,
    );
}
unsafe extern "C" fn save_viewstate(mut wp: *mut win_T, mut vs: *mut viewstate_T) {
    (*vs).vs_curswant = (*wp).w_curswant;
    (*vs).vs_leftcol = (*wp).w_leftcol;
    (*vs).vs_skipcol = (*wp).w_skipcol;
    (*vs).vs_topline = (*wp).w_topline;
    (*vs).vs_topfill = (*wp).w_topfill;
    (*vs).vs_botline = (*wp).w_botline;
    (*vs).vs_empty_rows = (*wp).w_empty_rows;
}
unsafe extern "C" fn restore_viewstate(mut wp: *mut win_T, mut vs: *mut viewstate_T) {
    (*wp).w_curswant = (*vs).vs_curswant;
    (*wp).w_leftcol = (*vs).vs_leftcol;
    (*wp).w_skipcol = (*vs).vs_skipcol;
    (*wp).w_topline = (*vs).vs_topline;
    (*wp).w_topfill = (*vs).vs_topfill;
    (*wp).w_botline = (*vs).vs_botline;
    (*wp).w_empty_rows = (*vs).vs_empty_rows;
}
unsafe extern "C" fn init_incsearch_state(mut s: *mut incsearch_state_T) {
    (*s).winid = (*curwin).handle;
    (*s).match_start = (*curwin).w_cursor;
    (*s).did_incsearch = false_0 != 0;
    (*s).incsearch_postponed = false_0 != 0;
    (*s).magic_overruled_save = magic_overruled;
    clearpos(&raw mut (*s).match_end);
    (*s).save_cursor = (*curwin).w_cursor;
    (*s).search_start = (*curwin).w_cursor;
    save_viewstate(curwin, &raw mut (*s).init_viewstate);
    save_viewstate(curwin, &raw mut (*s).old_viewstate);
}
unsafe extern "C" fn set_search_match(mut t: *mut pos_T) {
    (*t).lnum += search_match_lines;
    (*t).col = search_match_endcol;
    if (*t).lnum > (*curbuf).b_ml.ml_line_count {
        (*t).lnum = (*curbuf).b_ml.ml_line_count;
        coladvance(curwin, MAXCOL as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn parse_pattern_and_range(
    mut incsearch_start: *mut pos_T,
    mut search_delim: *mut ::core::ffi::c_int,
    mut skiplen: *mut ::core::ffi::c_int,
    mut patlen: *mut ::core::ffi::c_int,
) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut delim_optional: bool = false_0 != 0;
    let mut dummy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut magic: magic_T = 0 as magic_T;
    *skiplen = 0 as ::core::ffi::c_int;
    *patlen = ccline.cmdlen;
    search_first_line = 0 as ::core::ffi::c_int as linenr_T;
    search_last_line = MAXLNUM as ::core::ffi::c_int as linenr_T;
    let mut ea: exarg_T = exarg {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ccline.cmdbuff,
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 1 as linenr_T,
        line2: 1 as linenr_T,
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
    let mut dummy_cmdmod: cmdmod_T = cmdmod_T {
        cmod_flags: 0,
        cmod_split: 0,
        cmod_tab: 0,
        cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmod_filter_regmatch: regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        },
        cmod_filter_force: false,
        cmod_verbose: 0,
        cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmod_did_sandbox: 0,
        cmod_verbose_save: 0,
        cmod_save_msg_silent: 0,
        cmod_save_msg_scroll: 0,
        cmod_did_esilent: 0,
    };
    parse_command_modifiers(
        &raw mut ea,
        &raw mut dummy,
        &raw mut dummy_cmdmod,
        true_0 != 0,
    );
    let mut cmd: *mut ::core::ffi::c_char =
        skip_range(ea.cmd, ::core::ptr::null_mut::<::core::ffi::c_int>());
    if vim_strchr(
        b"sgvlu\0".as_ptr() as *const ::core::ffi::c_char,
        *cmd as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        return false_0 != 0;
    }
    p = cmd;
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        p = p.offset(1);
    }
    if *skipwhite(p) as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if strncmp(
        cmd,
        b"substitute\0".as_ptr() as *const ::core::ffi::c_char,
        p.offset_from(cmd) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"smagic\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(cmd) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"snomagic\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 3 as isize {
                p.offset_from(cmd)
            } else {
                3 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"vglobal\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(cmd) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'm' as ::core::ffi::c_int
        {
            magic_overruled = OPTION_MAGIC_ON;
        } else if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'n' as ::core::ffi::c_int
        {
            magic_overruled = OPTION_MAGIC_OFF;
        }
    } else if strncmp(
        cmd,
        b"sort\0".as_ptr() as *const ::core::ffi::c_char,
        (if p.offset_from(cmd) > 3 as isize {
            p.offset_from(cmd)
        } else {
            3 as isize
        }) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"uniq\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 3 as isize {
                p.offset_from(cmd)
            } else {
                3 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
        }
        loop {
            p = skipwhite(p);
            if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint && {
                p = skipwhite(p);
                *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            } || {
                p = skipwhite(p);
                *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint && {
                    p = skipwhite(p);
                    *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                }
            }) {
                break;
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            return false_0 != 0;
        }
    } else if strncmp(
        cmd,
        b"vimgrep\0".as_ptr() as *const ::core::ffi::c_char,
        (if p.offset_from(cmd) > 3 as isize {
            p.offset_from(cmd)
        } else {
            3 as isize
        }) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"vimgrepadd\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 8 as isize {
                p.offset_from(cmd)
            } else {
                8 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"lvimgrep\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 2 as isize {
                p.offset_from(cmd)
            } else {
                2 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"lvimgrepadd\0".as_ptr() as *const ::core::ffi::c_char,
            (if p.offset_from(cmd) > 9 as isize {
                p.offset_from(cmd)
            } else {
                9 as isize
            }) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            cmd,
            b"global\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(cmd) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
            p = p.offset(1);
            if *skipwhite(p) as ::core::ffi::c_int == NUL {
                return false_0 != 0;
            }
        }
        if *cmd as ::core::ffi::c_int != 'g' as ::core::ffi::c_int {
            delim_optional = true_0 != 0;
        }
    } else {
        return false_0 != 0;
    }
    p = skipwhite(p);
    let mut delim: ::core::ffi::c_int = if delim_optional as ::core::ffi::c_int != 0
        && vim_isIDc(*p as uint8_t as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        ' ' as ::core::ffi::c_int
    } else {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        *c2rust_fresh0 as ::core::ffi::c_int
    };
    *search_delim = delim;
    let mut end: *mut ::core::ffi::c_char = skip_regexp_ex(
        p,
        delim,
        magic_isset() as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        &raw mut magic,
    );
    let mut use_last_pat: bool = end == p && *end as ::core::ffi::c_int == delim;
    if end == p && !use_last_pat {
        return false_0 != 0;
    }
    if !use_last_pat {
        let mut c: ::core::ffi::c_char = *end;
        *end = NUL as ::core::ffi::c_char;
        let mut empty: bool = empty_pattern_magic(p, end.offset_from(p) as size_t, magic);
        *end = c;
        if empty {
            return false_0 != 0;
        }
    }
    *skiplen = p.offset_from(ccline.cmdbuff) as ::core::ffi::c_int;
    *patlen = end.offset_from(p) as ::core::ffi::c_int;
    let mut save_cursor: pos_T = (*curwin).w_cursor;
    (*curwin).w_cursor = *incsearch_start;
    parse_cmd_address(&raw mut ea, &raw mut dummy, true_0 != 0);
    if ea.addr_count > 0 as ::core::ffi::c_int {
        search_first_line = if ea.line2 < ea.line1 {
            ea.line2
        } else {
            ea.line1
        };
        search_last_line = if ea.line2 > ea.line1 {
            ea.line2
        } else {
            ea.line1
        };
    } else if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int
        && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 'o' as ::core::ffi::c_int
    {
        search_last_line = (*curwin).w_cursor.lnum;
        search_first_line = search_last_line;
    }
    (*curwin).w_cursor = save_cursor;
    return true_0 != 0;
}
unsafe extern "C" fn do_incsearch_highlighting(
    mut firstc: ::core::ffi::c_int,
    mut search_delim: *mut ::core::ffi::c_int,
    mut is_state: *mut incsearch_state_T,
    mut skiplen: *mut ::core::ffi::c_int,
    mut patlen: *mut ::core::ffi::c_int,
) -> bool {
    let mut retval: bool = false_0 != 0;
    *skiplen = 0 as ::core::ffi::c_int;
    *patlen = ccline.cmdlen;
    if p_is == 0 || cmd_silent as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    search_first_line = 0 as ::core::ffi::c_int as linenr_T;
    search_last_line = MAXLNUM as ::core::ffi::c_int as linenr_T;
    if firstc == '/' as ::core::ffi::c_int || firstc == '?' as ::core::ffi::c_int {
        *search_delim = firstc;
        return true_0 != 0;
    }
    if firstc != ':' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    emsg_off += 1;
    retval = parse_pattern_and_range(
        &raw mut (*is_state).search_start,
        search_delim,
        skiplen,
        patlen,
    );
    emsg_off -= 1;
    return retval;
}
unsafe extern "C" fn may_do_incsearch_highlighting(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
) {
    let mut skiplen: ::core::ffi::c_int = 0;
    let mut patlen: ::core::ffi::c_int = 0;
    let mut search_delim: ::core::ffi::c_int = 0;
    save_last_search_pattern();
    if !do_incsearch_highlighting(
        firstc,
        &raw mut search_delim,
        s,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        finish_incsearch_highlighting(false_0 != 0, s, true_0 != 0);
        return;
    }
    if char_avail() {
        restore_last_search_pattern();
        (*s).incsearch_postponed = true_0 != 0;
        return;
    }
    (*s).incsearch_postponed = false_0 != 0;
    let mut next_char: ::core::ffi::c_char = *ccline.cmdbuff.offset((skiplen + patlen) as isize);
    let mut use_last_pat: bool = patlen == 0 as ::core::ffi::c_int
        && skiplen > 0 as ::core::ffi::c_int
        && *ccline
            .cmdbuff
            .offset((skiplen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == next_char as ::core::ffi::c_int;
    if patlen != 0 as ::core::ffi::c_int || use_last_pat as ::core::ffi::c_int != 0 {
        ui_busy_start();
        ui_flush();
    }
    if search_first_line == 0 as linenr_T {
        (*curwin).w_cursor = (*s).search_start;
    } else if search_first_line > (*curbuf).b_ml.ml_line_count {
        (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
        (*curwin).w_cursor.col = MAXCOL as ::core::ffi::c_int as colnr_T;
    } else {
        (*curwin).w_cursor.lnum = search_first_line;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if patlen != 0 as ::core::ffi::c_int || use_last_pat as ::core::ffi::c_int != 0 {
        let mut search_flags: ::core::ffi::c_int = SEARCH_OPT as ::core::ffi::c_int
            + SEARCH_NOOF as ::core::ffi::c_int
            + SEARCH_PEEK as ::core::ffi::c_int;
        if p_hls == 0 {
            search_flags += SEARCH_KEEP as ::core::ffi::c_int;
        }
        if search_first_line != 0 as linenr_T {
            search_flags += SEARCH_START as ::core::ffi::c_int;
        }
        let mut tm: proftime_T = profile_setlimit(500 as int64_t);
        let mut sia: searchit_arg_T = searchit_arg_T {
            sa_stop_lnum: 0,
            sa_tm: &raw mut tm,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };
        *ccline.cmdbuff.offset((skiplen + patlen) as isize) = NUL as ::core::ffi::c_char;
        emsg_off += 1;
        found = do_search(
            ::core::ptr::null_mut::<oparg_T>(),
            if firstc == ':' as ::core::ffi::c_int {
                '/' as ::core::ffi::c_int
            } else {
                firstc
            },
            search_delim,
            ccline.cmdbuff.offset(skiplen as isize),
            patlen as size_t,
            count,
            search_flags,
            &raw mut sia,
        );
        emsg_off -= 1;
        *ccline.cmdbuff.offset((skiplen + patlen) as isize) = next_char;
        if (*curwin).w_cursor.lnum < search_first_line || (*curwin).w_cursor.lnum > search_last_line
        {
            found = 0 as ::core::ffi::c_int;
            (*curwin).w_cursor = (*s).search_start;
        }
        if got_int {
            vpeekc();
            got_int = false_0 != 0;
            found = 0 as ::core::ffi::c_int;
        } else if char_avail() {
            (*s).incsearch_postponed = true_0 != 0;
        }
        ui_busy_stop();
    } else {
        set_no_hlsearch(true_0 != 0);
        redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    }
    highlight_match = found != 0 as ::core::ffi::c_int;
    restore_viewstate(curwin, &raw mut (*s).old_viewstate);
    changed_cline_bef_curs(curwin);
    update_topline(curwin);
    let mut end_pos: pos_T = (*curwin).w_cursor;
    if found != 0 as ::core::ffi::c_int {
        (*s).match_start = (*curwin).w_cursor;
        set_search_match(&raw mut (*curwin).w_cursor);
        validate_cursor(curwin);
        (*s).match_end = (*curwin).w_cursor;
        (*curwin).w_cursor = end_pos;
        end_pos = (*s).match_end;
    }
    if !use_last_pat {
        next_char = *ccline.cmdbuff.offset((skiplen + patlen) as isize);
        *ccline.cmdbuff.offset((skiplen + patlen) as isize) = NUL as ::core::ffi::c_char;
        if empty_pattern(
            ccline.cmdbuff.offset(skiplen as isize),
            patlen as size_t,
            search_delim,
        ) as ::core::ffi::c_int
            != 0
            && !no_hlsearch
        {
            redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
            set_no_hlsearch(true_0 != 0);
        }
        *ccline.cmdbuff.offset((skiplen + patlen) as isize) = next_char;
    }
    validate_cursor(curwin);
    if p_ru != 0
        && ((*curwin).w_status_height > 0 as ::core::ffi::c_int
            || global_stl_height() > 0 as ::core::ffi::c_int)
    {
        (*curwin).w_redr_status = true_0 != 0;
    }
    redraw_later(curwin, UPD_SOME_VALID as ::core::ffi::c_int);
    update_screen();
    highlight_match = false_0 != 0;
    restore_last_search_pattern();
    if *ccline.cmdbuff.offset((skiplen + patlen) as isize) as ::core::ffi::c_int != NUL {
        (*curwin).w_cursor = (*s).search_start;
    } else if found != 0 as ::core::ffi::c_int {
        (*curwin).w_cursor = end_pos;
        (*curwin).w_valid_cursor = end_pos;
    }
    msg_starthere();
    redrawcmdline();
    (*s).did_incsearch = true_0 != 0;
}
unsafe extern "C" fn may_add_char_to_search(
    mut firstc: ::core::ffi::c_int,
    mut c: *mut ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
) -> ::core::ffi::c_int {
    let mut skiplen: ::core::ffi::c_int = 0;
    let mut patlen: ::core::ffi::c_int = 0;
    let mut search_delim: ::core::ffi::c_int = 0;
    save_last_search_pattern();
    if !do_incsearch_highlighting(
        firstc,
        &raw mut search_delim,
        s,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        return FAIL;
    }
    restore_last_search_pattern();
    if (*s).did_incsearch {
        (*curwin).w_cursor = (*s).match_end;
        *c = gchar_cursor();
        if *c != NUL {
            if p_ic != 0
                && p_scs != 0
                && !pat_has_uppercase(ccline.cmdbuff.offset(skiplen as isize))
            {
                *c = mb_tolower(*c);
            }
            if *c == search_delim
                || !vim_strchr(
                    if magic_isset() as ::core::ffi::c_int != 0 {
                        b"\\~^$.*[\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\\^$\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    *c,
                )
                .is_null()
            {
                stuffcharReadbuff(*c);
                *c = '\\' as ::core::ffi::c_int;
            }
            if utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                let save_c: ::core::ffi::c_int = *c;
                while utf_char2len(*c) != utfc_ptr2len(get_cursor_pos_ptr()) {
                    (*curwin).w_cursor.col += utf_char2len(*c);
                    *c = gchar_cursor();
                    stuffcharReadbuff(*c);
                }
                *c = save_c;
            }
            return FAIL;
        }
    }
    return OK;
}
unsafe extern "C" fn finish_incsearch_highlighting(
    mut gotesc: bool,
    mut s: *mut incsearch_state_T,
    mut call_update_screen: bool,
) {
    if !(*s).did_incsearch {
        return;
    }
    (*s).did_incsearch = false_0 != 0;
    if gotesc {
        (*curwin).w_cursor = (*s).save_cursor;
    } else {
        if !equalpos((*s).save_cursor, (*s).search_start) {
            (*curwin).w_cursor = (*s).save_cursor;
            setpcmark();
        }
        (*curwin).w_cursor = (*s).search_start;
    }
    restore_viewstate(curwin, &raw mut (*s).old_viewstate);
    highlight_match = false_0 != 0;
    search_first_line = 0 as ::core::ffi::c_int as linenr_T;
    search_last_line = MAXLNUM as ::core::ffi::c_int as linenr_T;
    magic_overruled = (*s).magic_overruled_save;
    validate_cursor(curwin);
    status_redraw_all();
    redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
    if call_update_screen {
        update_screen();
    }
}
unsafe extern "C" fn init_ccline(mut firstc: ::core::ffi::c_int, mut indent: ::core::ffi::c_int) {
    ccline.overstrike = false_0;
    '_c2rust_label: {
        if indent >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"indent >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                691 as ::core::ffi::c_uint,
                b"void init_ccline(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    ccline.cmdfirstc = if firstc == '@' as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        firstc
    };
    ccline.cmdindent = if firstc > 0 as ::core::ffi::c_int {
        indent
    } else {
        0 as ::core::ffi::c_int
    };
    alloc_cmdbuff(indent + 50 as ::core::ffi::c_int);
    ccline.cmdpos = 0 as ::core::ffi::c_int;
    ccline.cmdlen = ccline.cmdpos;
    *ccline.cmdbuff.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    ccline.last_colors = ColoredCmdline {
        prompt_id: 0,
        cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        colors: CmdlineColors {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
        },
    };
    sb_text_start_cmdline();
    if firstc <= 0 as ::core::ffi::c_int {
        memset(
            ccline.cmdbuff as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            indent as size_t,
        );
        *ccline.cmdbuff.offset(indent as isize) = NUL as ::core::ffi::c_char;
        ccline.cmdpos = indent;
        ccline.cmdspos = indent;
        ccline.cmdlen = indent;
    }
}
unsafe extern "C" fn ui_ext_cmdline_hide(mut abort_0: bool) {
    if ui_has(kUICmdline) {
        cmdline_was_last_drawn = false_0 != 0;
        ccline.redraw_state = kCmdRedrawNone;
        ui_call_cmdline_hide(ccline.level as Integer, abort_0 as Boolean);
    }
}
unsafe extern "C" fn command_line_enter(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut indent: ::core::ffi::c_int,
    mut clear_ccline: bool,
) -> *mut uint8_t {
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut firstcbuf: [::core::ffi::c_char; 2] = [0; 2];
    static mut cmdline_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cmdline_level += 1;
    let mut save_cmdpreview: bool = cmdpreview;
    cmdpreview = false_0 != 0;
    let mut state: CommandLineState = CommandLineState {
        state: VimState {
            check: None,
            execute: None,
        },
        firstc: firstc,
        count: count,
        indent: indent,
        c: 0,
        gotesc: false,
        do_abbr: false,
        lookfor: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lookforlen: 0,
        hiscnt: 0,
        save_hiscnt: 0,
        histype: 0,
        is_state: incsearch_state_T {
            search_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            save_cursor: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            winid: 0,
            init_viewstate: viewstate_T {
                vs_curswant: 0,
                vs_leftcol: 0,
                vs_skipcol: 0,
                vs_topline: 0,
                vs_topfill: 0,
                vs_botline: 0,
                vs_empty_rows: 0,
            },
            old_viewstate: viewstate_T {
                vs_curswant: 0,
                vs_leftcol: 0,
                vs_skipcol: 0,
                vs_topline: 0,
                vs_topfill: 0,
                vs_botline: 0,
                vs_empty_rows: 0,
            },
            match_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            match_end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            did_incsearch: false,
            incsearch_postponed: false,
            magic_overruled_save: OPTION_MAGIC_NOT_SET,
        },
        did_wild_list: false,
        wim_index: 0,
        save_msg_scroll: msg_scroll,
        save_State: State,
        prev_cmdpos: -1 as ::core::ffi::c_int,
        prev_cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_p_icm: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        skip_pum_redraw: false,
        some_key_typed: false,
        ignore_drag_release: true_0 != 0,
        break_ctrl_c: false,
        xpc: expand_T {
            xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_context: 0,
            xp_pattern_len: 0,
            xp_prefix: XP_PREFIX_NONE,
            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_luaref: 0,
            xp_script_ctx: sctx_T {
                sc_sid: 0,
                sc_seq: 0,
                sc_lnum: 0,
                sc_chan: 0,
            },
            xp_backslash: 0,
            xp_shell: false,
            xp_numfiles: 0,
            xp_col: 0,
            xp_selected: 0,
            xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            xp_buf: [0; 256],
            xp_search_dir: kDirectionNotSet,
            xp_pre_incsearch_pos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
        },
        b_im_ptr: ::core::ptr::null_mut::<OptInt>(),
        b_im_ptr_buf: ::core::ptr::null_mut::<buf_T>(),
        cmdline_type: 0,
        event_cmdlineleavepre_triggered: false,
        did_hist_navigate: false,
    };
    let mut s: *mut CommandLineState = &raw mut state;
    (*s).save_p_icm = xstrdup(p_icm);
    init_incsearch_state(&raw mut (*s).is_state);
    let mut save_ccline: CmdlineInfo = CmdlineInfo {
        cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdbufflen: 0,
        cmdlen: 0,
        cmdpos: 0,
        cmdspos: 0,
        cmdfirstc: 0,
        cmdindent: 0,
        cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        hl_id: 0,
        overstrike: 0,
        xpc: ::core::ptr::null_mut::<expand_T>(),
        xp_context: 0,
        xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        input_fn: 0,
        cmdbuff_replaced: false,
        prompt_id: 0,
        highlight_callback: Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        last_colors: ColoredCmdline {
            prompt_id: 0,
            cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            colors: CmdlineColors {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
            },
        },
        level: 0,
        prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
        special_char: 0,
        special_shift: false,
        redraw_state: kCmdRedrawNone,
        one_key: false,
        mouse_used: ::core::ptr::null_mut::<bool>(),
    };
    let mut did_save_ccline: bool = false_0 != 0;
    if !ccline.cmdbuff.is_null() {
        '_c2rust_label: {
            if clear_ccline {
            } else {
                __assert_fail(
                    b"clear_ccline\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    756 as ::core::ffi::c_uint,
                    b"uint8_t *command_line_enter(int, int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        save_cmdline(&raw mut save_ccline);
        did_save_ccline = true_0 != 0;
    } else if clear_ccline {
        memset(
            &raw mut ccline as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CmdlineInfo>(),
        );
    }
    if (*s).firstc == -1 as ::core::ffi::c_int {
        (*s).firstc = NUL;
        (*s).break_ctrl_c = true_0 != 0;
    }
    init_ccline((*s).firstc, (*s).indent);
    '_c2rust_label_0: {
        if !ccline.cmdbuff.is_null() {
        } else {
            __assert_fail(
                b"ccline.cmdbuff != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                771 as ::core::ffi::c_uint,
                b"uint8_t *command_line_enter(int, int, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh1 = last_prompt_id;
    last_prompt_id = last_prompt_id.wrapping_add(1);
    ccline.prompt_id = c2rust_fresh1;
    ccline.level = cmdline_level;
    if cmdline_level == 50 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_command_too_recursive as *const ::core::ffi::c_char,
        ));
    } else {
        ExpandInit(&raw mut (*s).xpc);
        ccline.xpc = &raw mut (*s).xpc;
        clear_cmdline_orig();
        cmdmsg_rl = (*curwin).w_onebuf_opt.wo_rl != 0
            && *(*curwin).w_onebuf_opt.wo_rlc as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && ((*s).firstc == '/' as ::core::ffi::c_int
                || (*s).firstc == '?' as ::core::ffi::c_int);
        msg_grid_validate();
        redir_off = true_0 != 0;
        if !cmd_silent {
            gotocmdline(true_0 != 0);
            redrawcmdprompt();
            ccline.cmdspos = cmd_startcol();
        }
        (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        (*s).xpc.xp_backslash = XP_BS_NONE as ::core::ffi::c_int;
        (*s).xpc.xp_shell = false_0 != 0;
        if ccline.input_fn != 0 {
            (*s).xpc.xp_context = ccline.xp_context;
            (*s).xpc.xp_pattern = ccline.cmdbuff;
            (*s).xpc.xp_arg = ccline.xp_arg;
        }
        msg_scroll = false_0;
        State = MODE_CMDLINE as ::core::ffi::c_int;
        if (*s).firstc == '/' as ::core::ffi::c_int
            || (*s).firstc == '?' as ::core::ffi::c_int
            || (*s).firstc == '@' as ::core::ffi::c_int
        {
            if (*curbuf).b_p_imsearch == B_IMODE_USE_INSERT as OptInt {
                (*s).b_im_ptr = &raw mut (*curbuf).b_p_iminsert;
            } else {
                (*s).b_im_ptr = &raw mut (*curbuf).b_p_imsearch;
            }
            (*s).b_im_ptr_buf = curbuf;
            if *(*s).b_im_ptr == B_IMODE_LMAP as OptInt {
                State |= MODE_LANGMAP as ::core::ffi::c_int;
            }
        }
        setmouse();
        (*s).cmdline_type = if firstc > 0 as ::core::ffi::c_int {
            firstc
        } else {
            '-' as ::core::ffi::c_int
        };
        err = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        firstcbuf = [0; 2];
        firstcbuf[0 as ::core::ffi::c_int as usize] = (*s).cmdline_type as ::core::ffi::c_char;
        firstcbuf[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        if has_event(EVENT_CMDLINEENTER) {
            let mut save_v_event: save_v_event_T = save_v_event_T {
                sve_did_save: false,
                sve_hashtab: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
            };
            let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
            tv_dict_add_str(
                dict,
                b"cmdtype\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
            );
            tv_dict_add_nr(
                dict,
                b"cmdlevel\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                ccline.level as varnumber_T,
            );
            tv_dict_set_keys_readonly(dict);
            let mut tstate: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate);
            apply_autocmds(
                EVENT_CMDLINEENTER,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                false,
                curbuf,
            );
            restore_v_event(dict, &raw mut save_v_event);
            try_leave(&raw mut tstate, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                if !ui_has(kUIMessages) {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                msg_scroll = true_0;
                msg_puts_hl(err.msg, HLF_E as ::core::ffi::c_int, true_0 != 0);
                api_clear_error(&raw mut err);
                redrawcmd();
            }
            err = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
        }
        may_trigger_modechanged();
        init_history();
        (*s).hiscnt = get_hislen();
        (*s).histype = hist_char2type((*s).firstc) as ::core::ffi::c_int;
        do_digraph(-1 as ::core::ffi::c_int);
        if did_emsg != 0 {
            redrawcmd();
        }
        if !cmd_silent && !exmode_active {
            let mut found_one: bool = false_0 != 0;
            let mut wp: *mut win_T = if curtab == curtab {
                firstwin
            } else {
                (*curtab).tp_firstwin
            };
            while !wp.is_null() {
                if *p_stl as ::core::ffi::c_int != NUL
                    || *(*wp).w_onebuf_opt.wo_stl as ::core::ffi::c_int != NUL
                    || *p_wbr as ::core::ffi::c_int != NUL
                    || *(*wp).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
                {
                    (*wp).w_redr_status = true_0 != 0;
                    found_one = true_0 != 0;
                }
                wp = (*wp).w_next;
            }
            if *p_tal as ::core::ffi::c_int != NUL {
                redraw_tabline = true_0 != 0;
                found_one = true_0 != 0;
            }
            if redraw_custom_title_later() {
                found_one = true_0 != 0;
            }
            if found_one {
                redraw_statuslines();
            }
        }
        did_emsg = false_0;
        got_int = false_0 != 0;
        (*s).state.check =
            Some(command_line_check as unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int)
                as state_check_callback;
        (*s).state.execute = Some(
            command_line_execute
                as unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int,
        ) as state_execute_callback;
        state_enter(&raw mut (*s).state);
        if !(*s).event_cmdlineleavepre_triggered {
            set_vim_var_char((*s).c);
            trigger_cmd_autocmd((*s).cmdline_type, EVENT_CMDLINELEAVEPRE);
        }
        if has_event(EVENT_CMDLINELEAVE) {
            let mut save_v_event_0: save_v_event_T = save_v_event_T {
                sve_did_save: false,
                sve_hashtab: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
            };
            let mut dict_0: *mut dict_T = get_v_event(&raw mut save_v_event_0);
            tv_dict_add_str(
                dict_0,
                b"cmdtype\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
            );
            tv_dict_add_nr(
                dict_0,
                b"cmdlevel\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                ccline.level as varnumber_T,
            );
            tv_dict_set_keys_readonly(dict_0);
            tv_dict_add_bool(
                dict_0,
                b"abort\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (if (*s).gotesc as ::core::ffi::c_int != 0 {
                    kBoolVarTrue as ::core::ffi::c_int
                } else {
                    kBoolVarFalse as ::core::ffi::c_int
                }) as BoolVarValue,
            );
            set_vim_var_char((*s).c);
            let mut tstate_0: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate_0);
            apply_autocmds(
                EVENT_CMDLINELEAVE,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                &raw mut firstcbuf as *mut ::core::ffi::c_char,
                false,
                curbuf,
            );
            try_leave(&raw mut tstate_0, &raw mut err);
            if tv_dict_get_number(dict_0, b"abort\0".as_ptr() as *const ::core::ffi::c_char)
                != 0 as varnumber_T
            {
                (*s).gotesc = true_0 != 0;
            }
            restore_v_event(dict_0, &raw mut save_v_event_0);
        }
        cmdmsg_rl = false_0 != 0;
        if cmdline_pum_active() {
            cmdline_pum_remove(false_0 != 0);
        } else {
            pum_check_clear();
        }
        wildmenu_cleanup(&raw mut ccline);
        (*s).did_wild_list = false_0 != 0;
        (*s).wim_index = 0 as ::core::ffi::c_int;
        ExpandCleanup(&raw mut (*s).xpc);
        ccline.xpc = ::core::ptr::null_mut::<expand_T>();
        clear_cmdline_orig();
        finish_incsearch_highlighting((*s).gotesc, &raw mut (*s).is_state, false_0 != 0);
        if !ccline.cmdbuff.is_null() {
            if (*s).histype != HIST_INVALID as ::core::ffi::c_int
                && ccline.cmdlen != 0
                && (*s).firstc != NUL
                && ((*s).some_key_typed as ::core::ffi::c_int != 0
                    || (*s).histype == HIST_SEARCH as ::core::ffi::c_int)
            {
                add_to_history(
                    (*s).histype,
                    ccline.cmdbuff,
                    ccline.cmdlen as size_t,
                    true_0 != 0,
                    if (*s).histype == HIST_SEARCH as ::core::ffi::c_int {
                        (*s).firstc
                    } else {
                        NUL
                    },
                );
                if (*s).firstc == ':' as ::core::ffi::c_int {
                    xfree(new_last_cmdline as *mut ::core::ffi::c_void);
                    new_last_cmdline = xstrnsave(ccline.cmdbuff, ccline.cmdlen as size_t);
                }
            }
            if (*s).gotesc {
                abandon_cmdline();
            }
        }
        msg_check();
        if p_ch == 0 as OptInt && !ui_has(kUIMessages) {
            set_must_redraw(UPD_VALID as ::core::ffi::c_int);
        }
        msg_scroll = (*s).save_msg_scroll;
        redir_off = false_0 != 0;
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            emsg(err.msg);
            did_emsg = false_0;
            api_clear_error(&raw mut err);
        }
        if (*s).some_key_typed as ::core::ffi::c_int != 0
            && !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        {
            need_wait_return = false_0 != 0;
        }
        set_option_direct(
            kOptInccommand,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string((*s).save_p_icm),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        State = (*s).save_State;
        if cmdpreview as ::core::ffi::c_int != save_cmdpreview as ::core::ffi::c_int {
            cmdpreview = save_cmdpreview;
            redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
        }
        may_trigger_modechanged();
        setmouse();
        sb_text_end_cmdline();
    }
    xfree((*s).save_p_icm as *mut ::core::ffi::c_void);
    xfree(ccline.last_colors.cmdbuff as *mut ::core::ffi::c_void);
    xfree(ccline.last_colors.colors.items as *mut ::core::ffi::c_void);
    ccline.last_colors.colors.capacity = 0 as size_t;
    ccline.last_colors.colors.size = ccline.last_colors.colors.capacity;
    ccline.last_colors.colors.items = ::core::ptr::null_mut::<CmdlineColorChunk>();
    let mut p: *mut ::core::ffi::c_char = ccline.cmdbuff;
    if ui_has(kUICmdline) {
        if exmode_active as ::core::ffi::c_int != 0 && !p.is_null() {
            ui_ext_cmdline_block_append(0 as size_t, p);
        }
        ui_ext_cmdline_hide((*s).gotesc);
    }
    if !cmd_silent {
        redraw_custom_title_later();
        status_redraw_all();
    }
    cmdline_level -= 1;
    if did_save_ccline {
        restore_cmdline(&raw mut save_ccline);
    } else {
        ccline.cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    xfree((*s).prev_cmdbuff as *mut ::core::ffi::c_void);
    return p as *mut uint8_t;
}
unsafe extern "C" fn command_line_check(mut state: *mut VimState) -> ::core::ffi::c_int {
    let mut s: *mut CommandLineState = state as *mut CommandLineState;
    (*s).prev_cmdpos = ccline.cmdpos;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*s).prev_cmdbuff as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    redir_off = true_0 != 0;
    quit_more = false_0 != 0;
    did_emsg = false_0;
    if ex_normal_busy == 0 as ::core::ffi::c_int
        && stuff_empty() as ::core::ffi::c_int != 0
        && typebuf.tb_len == 0 as ::core::ffi::c_int
    {
        (*s).some_key_typed = true_0 != 0;
    }
    may_trigger_safestate((*s).xpc.xp_numfiles <= 0 as ::core::ffi::c_int);
    if !ccline.cmdbuff.is_null() {
        (*s).prev_cmdbuff = xstrdup(ccline.cmdbuff);
    }
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && (*s).firstc != '@' as ::core::ffi::c_int
    {
        (*s).skip_pum_redraw = true_0 != 0;
    }
    cursorcmd();
    ui_cursor_shape();
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn command_line_handle_ctrl_bsl(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    no_mapping += 1;
    allow_keys += 1;
    (*s).c = plain_vgetc();
    no_mapping -= 1;
    allow_keys -= 1;
    if (*s).c != Ctrl_N
        && (*s).c != Ctrl_G
        && ((*s).c != 'e' as ::core::ffi::c_int
            || ccline.cmdfirstc == '=' as ::core::ffi::c_int && KeyTyped as ::core::ffi::c_int != 0
            || cmdline_star > 0 as ::core::ffi::c_int)
    {
        vungetc((*s).c);
        return PROCESS_NEXT_KEY as ::core::ffi::c_int;
    }
    if (*s).c == 'e' as ::core::ffi::c_int {
        if ccline.cmdpos == ccline.cmdlen {
            new_cmdpos = 99999 as ::core::ffi::c_int;
        } else {
            new_cmdpos = ccline.cmdpos;
        }
        (*s).c = get_expr_register();
        if (*s).c == '=' as ::core::ffi::c_int {
            textlock += 1;
            let mut p: *mut ::core::ffi::c_char = get_expr_line();
            textlock -= 1;
            if !p.is_null() {
                let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
                realloc_cmdbuff(len + 1 as ::core::ffi::c_int);
                ccline.cmdlen = len;
                strcpy(ccline.cmdbuff, p);
                xfree(p as *mut ::core::ffi::c_void);
                ccline.cmdpos = if ccline.cmdlen < new_cmdpos {
                    ccline.cmdlen
                } else {
                    new_cmdpos
                };
                KeyTyped = false_0 != 0;
                redrawcmd();
                return CMDLINE_CHANGED as ::core::ffi::c_int;
            }
        }
        beep_flush();
        got_int = false_0 != 0;
        did_emsg = false_0;
        emsg_on_display = false_0 != 0;
        redrawcmd();
        return CMDLINE_NOT_CHANGED as ::core::ffi::c_int;
    }
    (*s).gotesc = true_0 != 0;
    return GOTO_NORMAL_MODE as ::core::ffi::c_int;
}
unsafe extern "C" fn command_line_wildchar_complete(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    let mut res: ::core::ffi::c_int = 0;
    let mut options: ::core::ffi::c_int = WILD_NO_BEEP as ::core::ffi::c_int;
    let mut escape: bool = (*s).firstc != '@' as ::core::ffi::c_int;
    let mut redraw_if_menu_empty: bool = (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    let mut wim_noselect: bool = p_wmnu != 0
        && wim_flags[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & kOptWimFlagNoselect as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int;
    if wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
        & kOptWimFlagLastused as ::core::ffi::c_int
        != 0
    {
        options |= WILD_BUFLASTUSED as ::core::ffi::c_int;
    }
    if (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int {
        if (*s).xpc.xp_numfiles > 1 as ::core::ffi::c_int
            && !(*s).did_wild_list
            && wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
                & kOptWimFlagList as ::core::ffi::c_int
                != 0
        {
            showmatches(&raw mut (*s).xpc, false_0 != 0, true_0 != 0, wim_noselect);
            redrawcmd();
            (*s).did_wild_list = true_0 != 0;
        }
        if wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
            & kOptWimFlagLongest as ::core::ffi::c_int
            != 0
        {
            res = nextwild(
                &raw mut (*s).xpc,
                WILD_LONGEST as ::core::ffi::c_int,
                options,
                escape,
            );
        } else if wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
            & kOptWimFlagFull as ::core::ffi::c_int
            != 0
        {
            res = nextwild(
                &raw mut (*s).xpc,
                WILD_NEXT as ::core::ffi::c_int,
                options,
                escape,
            );
        } else {
            res = OK;
        }
    } else {
        let mut wim_longest: bool = wim_flags[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            & kOptWimFlagLongest as ::core::ffi::c_int
            != 0;
        let mut wim_list: bool = wim_flags[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & kOptWimFlagList as ::core::ffi::c_int
            != 0;
        let mut wim_full: bool = wim_flags[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & kOptWimFlagFull as ::core::ffi::c_int
            != 0;
        (*s).wim_index = 0 as ::core::ffi::c_int;
        if (*s).c as OptInt == p_wc
            || (*s).c as OptInt == p_wcm
            || (*s).c
                == -(253 as ::core::ffi::c_int
                    + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || (*s).c == Ctrl_Z
        {
            options |= WILD_MAY_EXPAND_PATTERN as ::core::ffi::c_int;
            if (*s).c
                == -(253 as ::core::ffi::c_int
                    + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                options |= WILD_FUNC_TRIGGER as ::core::ffi::c_int;
            }
            (*s).xpc.xp_pre_incsearch_pos = (*s).is_state.search_start;
        }
        let mut cmdpos_before: ::core::ffi::c_int = ccline.cmdpos;
        if wim_longest {
            res = nextwild(
                &raw mut (*s).xpc,
                WILD_LONGEST as ::core::ffi::c_int,
                options,
                escape,
            );
        } else {
            if wim_noselect as ::core::ffi::c_int != 0 || wim_list as ::core::ffi::c_int != 0 {
                options |= WILD_NOSELECT as ::core::ffi::c_int;
            }
            res = nextwild(
                &raw mut (*s).xpc,
                WILD_EXPAND_KEEP as ::core::ffi::c_int,
                options,
                escape,
            );
        }
        if redraw_if_menu_empty as ::core::ffi::c_int != 0
            && (*s).xpc.xp_numfiles <= 0 as ::core::ffi::c_int
        {
            pum_check_clear();
        }
        if got_int {
            vpeekc();
            got_int = false_0 != 0;
            ExpandOne(
                &raw mut (*s).xpc,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                WILD_FREE as ::core::ffi::c_int,
            );
            (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            return CMDLINE_CHANGED as ::core::ffi::c_int;
        }
        if res == OK
            && (*s).xpc.xp_numfiles
                > (if wim_noselect as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                })
        {
            if wim_longest {
                let mut found_longest_prefix: bool = ccline.cmdpos != cmdpos_before;
                if wim_list as ::core::ffi::c_int != 0
                    || p_wmnu != 0 && wim_full as ::core::ffi::c_int != 0
                {
                    showmatches(&raw mut (*s).xpc, p_wmnu != 0, wim_list, true_0 != 0);
                } else if !found_longest_prefix {
                    let mut wim_list_next: bool = wim_flags[1 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        & kOptWimFlagList as ::core::ffi::c_int
                        != 0;
                    let mut wim_full_next: bool = wim_flags[1 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        & kOptWimFlagFull as ::core::ffi::c_int
                        != 0;
                    let mut wim_noselect_next: bool = wim_flags[1 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        & kOptWimFlagNoselect as ::core::ffi::c_int
                        != 0;
                    if wim_list_next as ::core::ffi::c_int != 0
                        || p_wmnu != 0
                            && (wim_full_next as ::core::ffi::c_int != 0
                                || wim_noselect_next as ::core::ffi::c_int != 0)
                    {
                        if wim_full_next as ::core::ffi::c_int != 0 && !wim_noselect_next {
                            nextwild(
                                &raw mut (*s).xpc,
                                WILD_NEXT as ::core::ffi::c_int,
                                options,
                                escape,
                            );
                        } else {
                            showmatches(
                                &raw mut (*s).xpc,
                                p_wmnu != 0,
                                wim_list_next,
                                wim_noselect_next,
                            );
                        }
                        if wim_list_next {
                            (*s).did_wild_list = true_0 != 0;
                        }
                    }
                }
            } else if wim_list as ::core::ffi::c_int != 0
                || p_wmnu != 0
                    && (wim_full as ::core::ffi::c_int != 0
                        || wim_noselect as ::core::ffi::c_int != 0)
            {
                showmatches(&raw mut (*s).xpc, p_wmnu != 0, wim_list, wim_noselect);
            } else {
                vim_beep(kOptBoFlagWildmode as ::core::ffi::c_int as ::core::ffi::c_uint);
            }
            redrawcmd();
            if wim_list {
                (*s).did_wild_list = true_0 != 0;
            }
        } else if (*s).xpc.xp_numfiles == -1 as ::core::ffi::c_int {
            (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
    }
    if (*s).wim_index < 3 as ::core::ffi::c_int {
        (*s).wim_index += 1;
    }
    if (*s).c == ESC {
        (*s).gotesc = true_0 != 0;
    }
    return if res == OK {
        CMDLINE_CHANGED as ::core::ffi::c_int
    } else {
        CMDLINE_NOT_CHANGED as ::core::ffi::c_int
    };
}
unsafe extern "C" fn command_line_end_wildmenu(
    mut s: *mut CommandLineState,
    mut key_is_wc: bool,
    mut c: ::core::ffi::c_int,
) {
    if cmdline_pum_active() {
        if c != -1 as ::core::ffi::c_int {
            (*s).skip_pum_redraw = (*s).skip_pum_redraw as ::core::ffi::c_int != 0
                && !key_is_wc
                && !ascii_iswhite(c)
                && (vim_isprintc(c) as ::core::ffi::c_int != 0
                    || c == K_BS
                    || c == Ctrl_H
                    || c == K_DEL
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == Ctrl_W
                    || c == Ctrl_U);
        }
        cmdline_pum_remove(
            c != -1 as ::core::ffi::c_int && (*s).skip_pum_redraw as ::core::ffi::c_int != 0,
        );
    }
    if (*s).xpc.xp_numfiles != -1 as ::core::ffi::c_int {
        ExpandOne(
            &raw mut (*s).xpc,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            WILD_FREE as ::core::ffi::c_int,
        );
    }
    (*s).did_wild_list = false_0 != 0;
    if p_wmnu == 0 || c != K_UP && c != K_DOWN {
        (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    (*s).wim_index = 0 as ::core::ffi::c_int;
    wildmenu_cleanup(&raw mut ccline);
}
unsafe extern "C" fn command_line_execute(
    mut state: *mut VimState,
    mut key: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if key
        == -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || key
            == -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return -1 as ::core::ffi::c_int;
    }
    let mut display_tick_saved: disptick_T = (*curwin).w_display_tick;
    let mut s: *mut CommandLineState = state as *mut CommandLineState;
    (*s).c = key;
    if ccline.cmdbuff_replaced as ::core::ffi::c_int != 0
        && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
    {
        command_line_end_wildmenu(s, false_0 != 0, -1 as ::core::ffi::c_int);
    }
    ccline.cmdbuff_replaced = false_0 != 0;
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && (*s).did_hist_navigate as ::core::ffi::c_int != 0
    {
        (*s).did_hist_navigate = false_0 != 0;
        return 1 as ::core::ffi::c_int;
    }
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            state_handle_k_event();
        } else if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            do_cmdline(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                Some(
                    getcmdkeycmd
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                NULL_0,
                DOCMD_NOWAIT as ::core::ffi::c_int,
            );
        } else {
            map_execute_lua(false_0 != 0, false_0 != 0);
        }
        if (*s).is_state.winid != (*curwin).handle {
            init_incsearch_state(&raw mut (*s).is_state);
        }
        if (*curwin).w_display_tick > display_tick_saved
            && (*s).is_state.did_incsearch as ::core::ffi::c_int != 0
        {
            may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
        }
        if ccline.cmdbuff_replaced {
            command_line_changed(s);
        }
        if pum_want.active {
            if cmdline_pum_active() {
                nextwild(
                    &raw mut (*s).xpc,
                    WILD_PUM_WANT as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    (*s).firstc != '@' as ::core::ffi::c_int,
                );
                if pum_want.finish {
                    nextwild(
                        &raw mut (*s).xpc,
                        WILD_APPLY as ::core::ffi::c_int,
                        WILD_NO_BEEP as ::core::ffi::c_int,
                        (*s).firstc != '@' as ::core::ffi::c_int,
                    );
                    command_line_end_wildmenu(s, false_0 != 0, (*s).c);
                }
            }
            pum_want.active = false_0 != 0;
        }
        if !cmdline_was_last_drawn {
            redrawcmdline();
        }
        return 1 as ::core::ffi::c_int;
    }
    if KeyTyped {
        (*s).some_key_typed = true_0 != 0;
        if cmdmsg_rl as ::core::ffi::c_int != 0 && KeyStuffed == 0 {
            match (*s).c {
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
                _ => {}
            }
        }
    }
    if (*s).c == Ctrl_C
        && (*s).firstc != '@' as ::core::ffi::c_int
        && (!(*s).break_ctrl_c || exmode_active as ::core::ffi::c_int != 0)
        && global_busy == 0
    {
        got_int = false_0 != 0;
    }
    if !(*s).lookfor.is_null()
        && (*s).c
            != -(253 as ::core::ffi::c_int
                + ((KE_S_DOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && (*s).c
            != -(253 as ::core::ffi::c_int
                + ((KE_S_UP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        && (*s).c != K_DOWN
        && (*s).c != K_UP
        && (*s).c != K_PAGEDOWN
        && (*s).c != K_PAGEUP
        && (*s).c != K_KPAGEDOWN
        && (*s).c != K_KPAGEUP
        && (*s).c != K_LEFT
        && (*s).c != K_RIGHT
        && ((*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int || (*s).c != Ctrl_P && (*s).c != Ctrl_N)
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*s).lookfor as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        (*s).lookforlen = 0 as ::core::ffi::c_int;
    }
    if (*s).c as OptInt != p_wc
        && (*s).c == K_S_TAB
        && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
    {
        (*s).c = Ctrl_P;
    }
    if p_wmnu != 0 {
        (*s).c = wildmenu_translate_key(
            &raw mut ccline,
            (*s).c,
            &raw mut (*s).xpc,
            (*s).did_wild_list,
        );
    }
    let mut wild_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let key_is_wc: bool = (*s).c as OptInt == p_wc && KeyTyped as ::core::ffi::c_int != 0
        || (*s).c as OptInt == p_wcm;
    if (cmdline_pum_active() as ::core::ffi::c_int != 0
        || wild_menu_showing != 0
        || (*s).did_wild_list as ::core::ffi::c_int != 0)
        && !key_is_wc
        && (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int
    {
        if (*s).c == Ctrl_E || (*s).c == Ctrl_Y {
            wild_type = if (*s).c == Ctrl_E {
                WILD_CANCEL as ::core::ffi::c_int
            } else {
                WILD_APPLY as ::core::ffi::c_int
            };
            nextwild(
                &raw mut (*s).xpc,
                wild_type,
                WILD_NO_BEEP as ::core::ffi::c_int,
                (*s).firstc != '@' as ::core::ffi::c_int,
            );
        }
    }
    if KeyTyped as ::core::ffi::c_int != 0
        && ((*s).c == '\n' as ::core::ffi::c_int
            || (*s).c == '\r' as ::core::ffi::c_int
            || (*s).c == K_KENTER
            || (*s).c == ESC)
        || (*s).c == Ctrl_C
    {
        set_vim_var_char((*s).c);
        trigger_cmd_autocmd((*s).cmdline_type, EVENT_CMDLINELEAVEPRE);
        (*s).event_cmdlineleavepre_triggered = true_0 != 0;
        if ((*s).c == ESC || (*s).c == Ctrl_C)
            && wim_flags[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                & kOptWimFlagList as ::core::ffi::c_int
                != 0
        {
            set_no_hlsearch(true_0 != 0);
        }
    }
    let mut end_wildmenu: bool = !key_is_wc
        && (*s).c != Ctrl_Z
        && (*s).c != Ctrl_N
        && (*s).c != Ctrl_P
        && (*s).c != Ctrl_A
        && (*s).c != Ctrl_L;
    end_wildmenu = end_wildmenu as ::core::ffi::c_int != 0
        && (!cmdline_pum_active()
            || (*s).c != K_PAGEDOWN
                && (*s).c != K_PAGEUP
                && (*s).c != K_KPAGEDOWN
                && (*s).c != K_KPAGEUP);
    if end_wildmenu {
        command_line_end_wildmenu(s, key_is_wc, (*s).c);
    }
    if p_wmnu != 0 {
        (*s).c = wildmenu_process_key(&raw mut ccline, (*s).c, &raw mut (*s).xpc);
    }
    if (*s).c == Ctrl_BSL {
        match command_line_handle_ctrl_bsl(s) {
            2 => return command_line_changed(s),
            1 => return command_line_not_changed(s),
            3 => return 0 as ::core::ffi::c_int,
            _ => {
                (*s).c = Ctrl_BSL;
            }
        }
    }
    if (*s).c == cedit_key
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_CMDWIN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        if ((*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_CMDWIN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || ex_normal_busy == 0 as ::core::ffi::c_int)
            && got_int as ::core::ffi::c_int == false_0
        {
            (*s).c = open_cmdwin();
            (*s).some_key_typed = true_0 != 0;
        }
    } else {
        (*s).c = do_digraph((*s).c);
    }
    if (*s).c == '\n' as ::core::ffi::c_int
        || (*s).c == '\r' as ::core::ffi::c_int
        || (*s).c == K_KENTER
        || (*s).c == ESC && (!KeyTyped || !vim_strchr(p_cpo, CPO_ESC).is_null())
    {
        if exmode_active as ::core::ffi::c_int != 0
            && (*s).c != ESC
            && ccline.cmdpos == ccline.cmdlen
            && ccline.cmdpos > 0 as ::core::ffi::c_int
            && *ccline
                .cmdbuff
                .offset((ccline.cmdpos - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
        {
            if (*s).c == K_KENTER {
                (*s).c = '\n' as ::core::ffi::c_int;
            }
        } else {
            (*s).gotesc = false_0 != 0;
            if ccheck_abbr((*s).c + ABBR_OFF) != 0 {
                return command_line_changed(s);
            }
            if !cmd_silent {
                if !ui_has(kUICmdline) {
                    msg_cursor_goto(msg_row, 0 as ::core::ffi::c_int);
                }
                ui_flush();
            }
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*s).c as OptInt == p_wc && !(*s).gotesc && KeyTyped as ::core::ffi::c_int != 0
        || (*s).c as OptInt == p_wcm
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || (*s).c == Ctrl_Z
    {
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            emsg_silent += 1;
        }
        let mut res: ::core::ffi::c_int = command_line_wildchar_complete(s);
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            emsg_silent -= 1;
        }
        if res == CMDLINE_CHANGED as ::core::ffi::c_int {
            return command_line_changed(s);
        }
        if (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_WILD as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            return command_line_not_changed(s);
        }
    }
    (*s).gotesc = false_0 != 0;
    if (*s).c == K_S_TAB && KeyTyped as ::core::ffi::c_int != 0 {
        if nextwild(
            &raw mut (*s).xpc,
            WILD_EXPAND_KEEP as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*s).firstc != '@' as ::core::ffi::c_int,
        ) == OK
        {
            if (*s).xpc.xp_numfiles > 1 as ::core::ffi::c_int
                && (!(*s).did_wild_list
                    && wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
                        & kOptWimFlagList as ::core::ffi::c_int
                        != 0
                    || p_wmnu != 0)
            {
                showmatches(
                    &raw mut (*s).xpc,
                    p_wmnu != 0,
                    wim_flags[(*s).wim_index as usize] as ::core::ffi::c_int
                        & kOptWimFlagList as ::core::ffi::c_int
                        != 0,
                    wim_flags[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        & kOptWimFlagNoselect as ::core::ffi::c_int
                        != 0,
                );
            }
            nextwild(
                &raw mut (*s).xpc,
                WILD_PREV as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*s).firstc != '@' as ::core::ffi::c_int,
            );
            nextwild(
                &raw mut (*s).xpc,
                WILD_PREV as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*s).firstc != '@' as ::core::ffi::c_int,
            );
            return command_line_changed(s);
        }
    }
    if (*s).c == NUL || (*s).c == K_ZERO {
        (*s).c = NL;
    }
    (*s).do_abbr = true_0 != 0;
    if wild_type == WILD_CANCEL as ::core::ffi::c_int
        || wild_type == WILD_APPLY as ::core::ffi::c_int
    {
        if (*s).is_state.winid != (*curwin).handle {
            init_incsearch_state(&raw mut (*s).is_state);
        }
        if KeyTyped as ::core::ffi::c_int != 0 || vpeekc() == NUL {
            may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
        }
        return command_line_not_changed(s);
    }
    return command_line_handle_key(s);
}
unsafe extern "C" fn may_do_command_line_next_incsearch(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut s: *mut incsearch_state_T,
    mut next_match: bool,
) -> ::core::ffi::c_int {
    let mut skiplen: ::core::ffi::c_int = 0;
    let mut patlen: ::core::ffi::c_int = 0;
    let mut search_delim: ::core::ffi::c_int = 0;
    save_last_search_pattern();
    if !do_incsearch_highlighting(
        firstc,
        &raw mut search_delim,
        s,
        &raw mut skiplen,
        &raw mut patlen,
    ) {
        restore_last_search_pattern();
        return OK;
    }
    if patlen == 0 as ::core::ffi::c_int
        && *ccline.cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int == NUL
    {
        restore_last_search_pattern();
        return FAIL;
    }
    ui_busy_start();
    ui_flush();
    let mut t: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut search_flags: ::core::ffi::c_int = SEARCH_NOOF as ::core::ffi::c_int;
    if search_delim == *ccline.cmdbuff.offset(skiplen as isize) as ::core::ffi::c_int {
        pat = last_search_pattern();
        if pat.is_null() {
            restore_last_search_pattern();
            return FAIL;
        }
        skiplen = 0 as ::core::ffi::c_int;
        patlen = last_search_pattern_len() as ::core::ffi::c_int;
    } else {
        pat = ccline.cmdbuff.offset(skiplen as isize);
    }
    let mut bslsh: bool = false_0 != 0;
    if patlen > 2 as ::core::ffi::c_int
        && firstc == *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
    {
        patlen -= 1;
        if *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
        {
            *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) =
                firstc as uint8_t as ::core::ffi::c_char;
            bslsh = true_0 != 0;
        }
    }
    if next_match {
        t = (*s).match_end;
        if lt((*s).match_start, (*s).match_end) {
            decl(&raw mut t);
        }
        search_flags += SEARCH_COL as ::core::ffi::c_int;
    } else {
        t = (*s).match_start;
    }
    if p_hls == 0 {
        search_flags += SEARCH_KEEP as ::core::ffi::c_int;
    }
    emsg_off += 1;
    let mut save: ::core::ffi::c_char = *pat.offset(patlen as isize);
    *pat.offset(patlen as isize) = NUL as ::core::ffi::c_char;
    let mut found: ::core::ffi::c_int = searchit(
        curwin,
        curbuf,
        &raw mut t,
        ::core::ptr::null_mut::<pos_T>(),
        (if next_match as ::core::ffi::c_int != 0 {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        }) as Direction,
        pat,
        patlen as size_t,
        count,
        search_flags,
        RE_SEARCH as ::core::ffi::c_int,
        ::core::ptr::null_mut::<searchit_arg_T>(),
    );
    emsg_off -= 1;
    *pat.offset(patlen as isize) = save;
    if bslsh {
        *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) = '\\' as ::core::ffi::c_char;
    }
    ui_busy_stop();
    if found != 0 {
        (*s).search_start = (*s).match_start;
        (*s).match_end = t;
        (*s).match_start = t;
        if !next_match && firstc != '?' as ::core::ffi::c_int {
            (*s).search_start = t;
            decl(&raw mut (*s).search_start);
        } else if next_match as ::core::ffi::c_int != 0 && firstc == '?' as ::core::ffi::c_int {
            (*s).search_start = t;
            incl(&raw mut (*s).search_start);
        }
        if lt(t, (*s).search_start) as ::core::ffi::c_int != 0
            && next_match as ::core::ffi::c_int != 0
        {
            (*s).search_start = t;
            if firstc == '?' as ::core::ffi::c_int {
                incl(&raw mut (*s).search_start);
            } else {
                decl(&raw mut (*s).search_start);
            }
        }
        set_search_match(&raw mut (*s).match_end);
        (*curwin).w_cursor = (*s).match_start;
        changed_cline_bef_curs(curwin);
        update_topline(curwin);
        validate_cursor(curwin);
        highlight_match = true_0 != 0;
        save_viewstate(curwin, &raw mut (*s).old_viewstate);
        redraw_later(curwin, UPD_NOT_VALID as ::core::ffi::c_int);
        update_screen();
        highlight_match = false_0 != 0;
        redrawcmdline();
        (*curwin).w_cursor = (*s).match_end;
    } else {
        vim_beep(kOptBoFlagError as ::core::ffi::c_int as ::core::ffi::c_uint);
    }
    restore_last_search_pattern();
    return FAIL;
}
unsafe extern "C" fn command_line_erase_chars(mut s: *mut CommandLineState) -> ::core::ffi::c_int {
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_KDEL as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).c = K_DEL;
    }
    if (*s).c == K_DEL && ccline.cmdpos != ccline.cmdlen {
        ccline.cmdpos += 1;
    }
    if (*s).c == K_DEL {
        ccline.cmdpos += mb_off_next(
            ccline.cmdbuff,
            ccline.cmdbuff.offset(ccline.cmdpos as isize),
        );
    }
    if ccline.cmdpos > 0 as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = ccline.cmdpos;
        let mut p: *mut ::core::ffi::c_char =
            mb_prevptr(ccline.cmdbuff, ccline.cmdbuff.offset(j as isize));
        if (*s).c == Ctrl_W {
            while p > ccline.cmdbuff
                && ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                p = mb_prevptr(ccline.cmdbuff, p);
            }
            let mut i: ::core::ffi::c_int = mb_get_class(p);
            while p > ccline.cmdbuff && mb_get_class(p) == i {
                p = mb_prevptr(ccline.cmdbuff, p);
            }
            if mb_get_class(p) != i {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
        ccline.cmdpos = p.offset_from(ccline.cmdbuff) as ::core::ffi::c_int;
        ccline.cmdlen -= j - ccline.cmdpos;
        let mut i_0: ::core::ffi::c_int = ccline.cmdpos;
        while i_0 < ccline.cmdlen {
            let c2rust_fresh29 = j;
            j = j + 1;
            let c2rust_fresh30 = i_0;
            i_0 = i_0 + 1;
            *ccline.cmdbuff.offset(c2rust_fresh30 as isize) =
                *ccline.cmdbuff.offset(c2rust_fresh29 as isize);
        }
        *ccline.cmdbuff.offset(ccline.cmdlen as isize) = NUL as ::core::ffi::c_char;
        if ccline.cmdlen == 0 as ::core::ffi::c_int {
            (*s).is_state.search_start = (*s).is_state.save_cursor;
            (*s).is_state.old_viewstate = (*s).is_state.init_viewstate;
        }
        redrawcmd();
    } else if ccline.cmdlen == 0 as ::core::ffi::c_int
        && (*s).c != Ctrl_W
        && ccline.cmdprompt.is_null()
        && (*s).indent == 0 as ::core::ffi::c_int
    {
        if exmode_active as ::core::ffi::c_int != 0 || ccline.cmdfirstc == '>' as ::core::ffi::c_int
        {
            return CMDLINE_NOT_CHANGED as ::core::ffi::c_int;
        }
        dealloc_cmdbuff();
        if !cmd_silent && !ui_has(kUICmdline) {
            msg_col = 0 as ::core::ffi::c_int;
            msg_putchar(' ' as ::core::ffi::c_int);
        }
        (*s).is_state.search_start = (*s).is_state.save_cursor;
        redraw_cmdline = true_0 != 0;
        return GOTO_NORMAL_MODE as ::core::ffi::c_int;
    }
    return CMDLINE_CHANGED as ::core::ffi::c_int;
}
unsafe extern "C" fn command_line_toggle_langmap(mut s: *mut CommandLineState) {
    let mut b_im_ptr: *mut OptInt = if buf_valid((*s).b_im_ptr_buf) as ::core::ffi::c_int != 0 {
        (*s).b_im_ptr
    } else {
        ::core::ptr::null_mut::<OptInt>()
    };
    if map_to_exists_mode(
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        MODE_LANGMAP as ::core::ffi::c_int,
        false_0 != 0,
    ) {
        State ^= MODE_LANGMAP as ::core::ffi::c_int;
        if !b_im_ptr.is_null() {
            if State & MODE_LANGMAP as ::core::ffi::c_int != 0 {
                *b_im_ptr = B_IMODE_LMAP as OptInt;
            } else {
                *b_im_ptr = B_IMODE_NONE as OptInt;
            }
        }
    }
    if !b_im_ptr.is_null() {
        if b_im_ptr == &raw mut (*curbuf).b_p_iminsert {
            set_iminsert_global(curbuf);
        } else {
            set_imsearch_global(curbuf);
        }
    }
    ui_cursor_shape();
    status_redraw_curbuf();
}
unsafe extern "C" fn command_line_insert_reg(mut s: *mut CommandLineState) -> ::core::ffi::c_int {
    let save_new_cmdpos: ::core::ffi::c_int = new_cmdpos;
    putcmdline('"' as ::core::ffi::c_char, true_0 != 0);
    no_mapping += 1;
    allow_keys += 1;
    (*s).c = plain_vgetc();
    let mut i: ::core::ffi::c_int = (*s).c;
    if i == Ctrl_O {
        i = Ctrl_R;
    }
    if i == Ctrl_R {
        (*s).c = plain_vgetc();
    }
    no_mapping -= 1;
    allow_keys -= 1;
    new_cmdpos = -1 as ::core::ffi::c_int;
    if (*s).c == '=' as ::core::ffi::c_int {
        if ccline.cmdfirstc == '=' as ::core::ffi::c_int || cmdline_star > 0 as ::core::ffi::c_int {
            beep_flush();
            (*s).c = ESC;
        } else {
            (*s).c = get_expr_register();
        }
    }
    let mut literally: bool = false_0 != 0;
    if (*s).c != ESC {
        literally = i == Ctrl_R || is_literal_register((*s).c) as ::core::ffi::c_int != 0;
        cmdline_paste((*s).c, literally, false_0 != 0);
        if aborting() {
            (*s).gotesc = true_0 != 0;
            return GOTO_NORMAL_MODE as ::core::ffi::c_int;
        }
        KeyTyped = false_0 != 0;
        if new_cmdpos >= 0 as ::core::ffi::c_int {
            ccline.cmdpos = if ccline.cmdlen < new_cmdpos {
                ccline.cmdlen
            } else {
                new_cmdpos
            };
        }
    }
    new_cmdpos = save_new_cmdpos;
    ccline.special_char = NUL as ::core::ffi::c_char;
    redrawcmd();
    return if literally as ::core::ffi::c_int != 0 {
        CMDLINE_CHANGED as ::core::ffi::c_int
    } else {
        CMDLINE_NOT_CHANGED as ::core::ffi::c_int
    };
}
unsafe extern "C" fn command_line_left_right_mouse(mut s: *mut CommandLineState) {
    if (*s).c
        == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        (*s).ignore_drag_release = true_0 != 0;
    } else {
        (*s).ignore_drag_release = false_0 != 0;
    }
    ccline.cmdspos = cmd_startcol();
    ccline.cmdpos = 0 as ::core::ffi::c_int;
    while ccline.cmdpos < ccline.cmdlen {
        let mut cells: ::core::ffi::c_int = cmdline_charsize(ccline.cmdpos);
        if mouse_row <= cmdline_row + ccline.cmdspos / Columns
            && mouse_col < ccline.cmdspos % Columns + cells
        {
            break;
        }
        correct_screencol(ccline.cmdpos, cells, &raw mut ccline.cmdspos);
        ccline.cmdpos +=
            utfc_ptr2len(ccline.cmdbuff.offset(ccline.cmdpos as isize)) - 1 as ::core::ffi::c_int;
        ccline.cmdspos += cells;
        ccline.cmdpos += 1;
    }
}
unsafe extern "C" fn command_line_next_histidx(mut s: *mut CommandLineState, mut next_match: bool) {
    loop {
        if !next_match {
            if (*s).hiscnt == get_hislen() {
                (*s).hiscnt = *get_hisidx((*s).histype);
            } else if (*s).hiscnt == 0 as ::core::ffi::c_int
                && *get_hisidx((*s).histype) != get_hislen() - 1 as ::core::ffi::c_int
            {
                (*s).hiscnt = get_hislen() - 1 as ::core::ffi::c_int;
            } else if (*s).hiscnt != *get_hisidx((*s).histype) + 1 as ::core::ffi::c_int {
                (*s).hiscnt -= 1;
            } else {
                (*s).hiscnt = (*s).save_hiscnt;
                break;
            }
        } else if (*s).hiscnt == *get_hisidx((*s).histype) {
            (*s).hiscnt = get_hislen();
            break;
        } else {
            if (*s).hiscnt == get_hislen() {
                break;
            }
            if (*s).hiscnt == get_hislen() - 1 as ::core::ffi::c_int {
                (*s).hiscnt = 0 as ::core::ffi::c_int;
            } else {
                (*s).hiscnt += 1;
            }
        }
        if (*s).hiscnt < 0 as ::core::ffi::c_int
            || (*get_histentry((*s).histype).offset((*s).hiscnt as isize))
                .hisstr
                .is_null()
        {
            (*s).hiscnt = (*s).save_hiscnt;
            break;
        } else if (*s).c != K_UP && (*s).c != K_DOWN
            || (*s).hiscnt == (*s).save_hiscnt
            || strncmp(
                (*get_histentry((*s).histype).offset((*s).hiscnt as isize)).hisstr,
                (*s).lookfor,
                (*s).lookforlen as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
    }
}
unsafe extern "C" fn command_line_browse_history(
    mut s: *mut CommandLineState,
) -> ::core::ffi::c_int {
    if (*s).histype == HIST_INVALID as ::core::ffi::c_int
        || get_hislen() == 0 as ::core::ffi::c_int
        || (*s).firstc == NUL
    {
        return CMDLINE_NOT_CHANGED as ::core::ffi::c_int;
    }
    (*s).save_hiscnt = (*s).hiscnt;
    if (*s).lookfor.is_null() {
        (*s).lookfor = xstrnsave(ccline.cmdbuff, ccline.cmdlen as size_t);
        *(*s).lookfor.offset(ccline.cmdpos as isize) = NUL as ::core::ffi::c_char;
        (*s).lookforlen = ccline.cmdpos;
    }
    let mut next_match: bool = (*s).c == K_DOWN
        || (*s).c
            == -(253 as ::core::ffi::c_int
                + ((KE_S_DOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || (*s).c == Ctrl_N
        || (*s).c == K_PAGEDOWN
        || (*s).c == K_KPAGEDOWN;
    command_line_next_histidx(s, next_match);
    if (*s).hiscnt != (*s).save_hiscnt {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut plen: ::core::ffi::c_int = 0;
        let mut old_firstc: ::core::ffi::c_int = 0;
        dealloc_cmdbuff();
        (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        if (*s).hiscnt == get_hislen() {
            p = (*s).lookfor;
            plen = (*s).lookforlen;
        } else {
            p = (*get_histentry((*s).histype).offset((*s).hiscnt as isize)).hisstr;
            plen = (*get_histentry((*s).histype).offset((*s).hiscnt as isize)).hisstrlen
                as ::core::ffi::c_int;
        }
        if (*s).histype == HIST_SEARCH as ::core::ffi::c_int && p != (*s).lookfor && {
            old_firstc = *p.offset((plen + 1 as ::core::ffi::c_int) as isize) as uint8_t
                as ::core::ffi::c_int;
            old_firstc != (*s).firstc
        } {
            let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i <= 1 as ::core::ffi::c_int {
                len = 0 as ::core::ffi::c_int;
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while *p.offset(j as isize) as ::core::ffi::c_int != NUL {
                    if *p.offset(j as isize) as ::core::ffi::c_int == old_firstc
                        && (j == 0 as ::core::ffi::c_int
                            || *p.offset((j - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                != '\\' as ::core::ffi::c_int)
                    {
                        if i > 0 as ::core::ffi::c_int {
                            *ccline.cmdbuff.offset(len as isize) =
                                (*s).firstc as ::core::ffi::c_char;
                        }
                    } else {
                        if *p.offset(j as isize) as ::core::ffi::c_int == (*s).firstc
                            && (j == 0 as ::core::ffi::c_int
                                || *p.offset((j - 1 as ::core::ffi::c_int) as isize)
                                    as ::core::ffi::c_int
                                    != '\\' as ::core::ffi::c_int)
                        {
                            if i > 0 as ::core::ffi::c_int {
                                *ccline.cmdbuff.offset(len as isize) = '\\' as ::core::ffi::c_char;
                            }
                            len += 1;
                        }
                        if i > 0 as ::core::ffi::c_int {
                            *ccline.cmdbuff.offset(len as isize) = *p.offset(j as isize);
                        }
                    }
                    len += 1;
                    j += 1;
                }
                if i == 0 as ::core::ffi::c_int {
                    alloc_cmdbuff(len);
                }
                i += 1;
            }
            *ccline.cmdbuff.offset(len as isize) = NUL as ::core::ffi::c_char;
            ccline.cmdlen = len;
            ccline.cmdpos = ccline.cmdlen;
        } else {
            alloc_cmdbuff(plen);
            strcpy(ccline.cmdbuff, p);
            ccline.cmdlen = plen;
            ccline.cmdpos = ccline.cmdlen;
        }
        redrawcmd();
        return CMDLINE_CHANGED as ::core::ffi::c_int;
    }
    beep_flush();
    return CMDLINE_NOT_CHANGED as ::core::ffi::c_int;
}
unsafe extern "C" fn command_line_handle_key(mut s: *mut CommandLineState) -> ::core::ffi::c_int {
    if !(ccline.one_key as ::core::ffi::c_int != 0 && (*s).c != ESC && (*s).c != Ctrl_C) {
        's_680: {
            'c_46093: {
                'c_46136: {
                    'c_46090: {
                        match (*s).c {
                            K_BS | Ctrl_H | K_DEL | -20733 | Ctrl_W => {
                                match command_line_erase_chars(s) {
                                    1 => return command_line_not_changed(s),
                                    3 => return 0 as ::core::ffi::c_int,
                                    _ => return command_line_changed(s),
                                }
                            }
                            K_INS | K_KINS => {
                                ccline.overstrike = (ccline.overstrike == 0) as ::core::ffi::c_int;
                                ui_cursor_shape();
                                may_trigger_modechanged();
                                status_redraw_curbuf();
                                redraw_statuslines();
                                return command_line_not_changed(s);
                            }
                            Ctrl_HAT => {
                                command_line_toggle_langmap(s);
                                return command_line_not_changed(s);
                            }
                            Ctrl_U => {
                                let mut j: ::core::ffi::c_int = ccline.cmdpos;
                                ccline.cmdlen -= j;
                                ccline.cmdpos = 0 as ::core::ffi::c_int;
                                let mut i: ::core::ffi::c_int = ccline.cmdpos;
                                while i < ccline.cmdlen {
                                    let c2rust_fresh7 = j;
                                    j = j + 1;
                                    let c2rust_fresh8 = i;
                                    i = i + 1;
                                    *ccline.cmdbuff.offset(c2rust_fresh8 as isize) =
                                        *ccline.cmdbuff.offset(c2rust_fresh7 as isize);
                                }
                                *ccline.cmdbuff.offset(ccline.cmdlen as isize) =
                                    NUL as ::core::ffi::c_char;
                                if ccline.cmdlen == 0 as ::core::ffi::c_int {
                                    (*s).is_state.search_start = (*s).is_state.save_cursor;
                                }
                                redrawcmd();
                                return command_line_changed(s);
                            }
                            ESC | Ctrl_C => {
                                if exmode_active as ::core::ffi::c_int != 0
                                    && (ex_normal_busy == 0 as ::core::ffi::c_int
                                        || typebuf.tb_len > 0 as ::core::ffi::c_int)
                                    || getln_interrupted_highlight as ::core::ffi::c_int != 0
                                        && (*s).c == Ctrl_C
                                {
                                    getln_interrupted_highlight = false_0 != 0;
                                    return command_line_not_changed(s);
                                }
                                (*s).gotesc = true_0 != 0;
                                return 0 as ::core::ffi::c_int;
                            }
                            Ctrl_R => match command_line_insert_reg(s) {
                                3 => return 0 as ::core::ffi::c_int,
                                2 => return command_line_changed(s),
                                _ => return command_line_not_changed(s),
                            },
                            Ctrl_D => {
                                if showmatches(
                                    &raw mut (*s).xpc,
                                    false_0 != 0,
                                    true_0 != 0,
                                    wim_flags[0 as ::core::ffi::c_int as usize]
                                        as ::core::ffi::c_int
                                        & kOptWimFlagNoselect as ::core::ffi::c_int
                                        != 0,
                                ) == EXPAND_NOTHING as ::core::ffi::c_int
                                {
                                    break 's_680;
                                } else {
                                    redrawcmd();
                                    return 1 as ::core::ffi::c_int;
                                }
                            }
                            K_RIGHT | K_S_RIGHT | -22269 => {
                                while ccline.cmdpos < ccline.cmdlen {
                                    let mut cells: ::core::ffi::c_int =
                                        cmdline_charsize(ccline.cmdpos);
                                    if KeyTyped as ::core::ffi::c_int != 0
                                        && ccline.cmdspos + cells >= Columns * Rows
                                    {
                                        break;
                                    }
                                    ccline.cmdspos += cells;
                                    ccline.cmdpos +=
                                        utfc_ptr2len(ccline.cmdbuff.offset(ccline.cmdpos as isize));
                                    if !(((*s).c == K_S_RIGHT
                                        || (*s).c
                                            == -(253 as ::core::ffi::c_int
                                                + ((KE_C_RIGHT as ::core::ffi::c_int)
                                                    << 8 as ::core::ffi::c_int))
                                        || mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0)
                                        && *ccline.cmdbuff.offset(ccline.cmdpos as isize)
                                            as ::core::ffi::c_int
                                            != ' ' as ::core::ffi::c_int)
                                    {
                                        break;
                                    }
                                }
                                ccline.cmdspos = cmd_screencol(ccline.cmdpos);
                                return command_line_not_changed(s);
                            }
                            K_LEFT | K_S_LEFT | -22013 => {
                                if ccline.cmdpos == 0 as ::core::ffi::c_int {
                                    return command_line_not_changed(s);
                                }
                                loop {
                                    ccline.cmdpos -= 1;
                                    ccline.cmdpos -= utf_head_off(
                                        ccline.cmdbuff,
                                        ccline.cmdbuff.offset(ccline.cmdpos as isize),
                                    );
                                    ccline.cmdspos -= cmdline_charsize(ccline.cmdpos);
                                    if !(ccline.cmdpos > 0 as ::core::ffi::c_int
                                        && ((*s).c == K_S_LEFT
                                            || (*s).c
                                                == -(253 as ::core::ffi::c_int
                                                    + ((KE_C_LEFT as ::core::ffi::c_int)
                                                        << 8 as ::core::ffi::c_int))
                                            || mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0)
                                        && *ccline.cmdbuff.offset(
                                            (ccline.cmdpos - 1 as ::core::ffi::c_int) as isize,
                                        )
                                            as ::core::ffi::c_int
                                            != ' ' as ::core::ffi::c_int)
                                    {
                                        break;
                                    }
                                }
                                ccline.cmdspos = cmd_screencol(ccline.cmdpos);
                                if ccline.special_char as ::core::ffi::c_int != NUL {
                                    putcmdline(ccline.special_char, ccline.special_shift);
                                }
                                return command_line_not_changed(s);
                            }
                            -13821 => return command_line_not_changed(s),
                            K_MIDDLEDRAG | K_MIDDLERELEASE => {
                                return command_line_not_changed(s);
                            }
                            K_MIDDLEMOUSE => {
                                cmdline_paste(
                                    if eval_has_provider(
                                        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                                        false_0 != 0,
                                    ) as ::core::ffi::c_int
                                        != 0
                                    {
                                        '*' as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    },
                                    true_0 != 0,
                                    true_0 != 0,
                                );
                                redrawcmd();
                                return command_line_changed(s);
                            }
                            K_LEFTDRAG | -12029 | K_RIGHTDRAG | -13565 => {
                                if (*s).ignore_drag_release {
                                    return command_line_not_changed(s);
                                }
                                break 'c_46090;
                            }
                            K_LEFTMOUSE => {
                                break 'c_46090;
                            }
                            K_RIGHTMOUSE => {
                                break 'c_46093;
                            }
                            K_MOUSEDOWN | K_MOUSEUP | K_MOUSELEFT | K_MOUSERIGHT | K_X1MOUSE
                            | K_X1DRAG | K_X1RELEASE | K_X2MOUSE | K_X2DRAG | K_X2RELEASE
                            | K_MOUSEMOVE => return command_line_not_changed(s),
                            K_SELECT => return command_line_not_changed(s),
                            Ctrl_B | K_HOME | K_KHOME | K_S_HOME | K_C_HOME => {
                                ccline.cmdpos = 0 as ::core::ffi::c_int;
                                ccline.cmdspos = cmd_startcol();
                                return command_line_not_changed(s);
                            }
                            Ctrl_E | K_END | K_KEND | K_S_END | K_C_END => {
                                ccline.cmdpos = ccline.cmdlen;
                                ccline.cmdspos = cmd_screencol(ccline.cmdpos);
                                return command_line_not_changed(s);
                            }
                            Ctrl_A => {
                                if cmdline_pum_active() {
                                    cmdline_pum_cleanup(&raw mut ccline);
                                }
                                if nextwild(
                                    &raw mut (*s).xpc,
                                    WILD_ALL as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                    (*s).firstc != '@' as ::core::ffi::c_int,
                                ) == FAIL
                                {
                                    break 's_680;
                                } else {
                                    (*s).xpc.xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                                    (*s).did_wild_list = false_0 != 0;
                                    return command_line_changed(s);
                                }
                            }
                            Ctrl_L => {
                                if may_add_char_to_search(
                                    (*s).firstc,
                                    &raw mut (*s).c,
                                    &raw mut (*s).is_state,
                                ) == OK
                                {
                                    return command_line_not_changed(s);
                                }
                                if nextwild(
                                    &raw mut (*s).xpc,
                                    WILD_LONGEST as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                    (*s).firstc != '@' as ::core::ffi::c_int,
                                ) == FAIL
                                {
                                    break 's_680;
                                } else {
                                    return command_line_changed(s);
                                }
                            }
                            Ctrl_N | Ctrl_P => {
                                if (*s).xpc.xp_numfiles > 0 as ::core::ffi::c_int {
                                    let wild_type: ::core::ffi::c_int = if (*s).c == Ctrl_P {
                                        WILD_PREV as ::core::ffi::c_int
                                    } else {
                                        WILD_NEXT as ::core::ffi::c_int
                                    };
                                    if nextwild(
                                        &raw mut (*s).xpc,
                                        wild_type,
                                        0 as ::core::ffi::c_int,
                                        (*s).firstc != '@' as ::core::ffi::c_int,
                                    ) == FAIL
                                    {
                                        break 's_680;
                                    } else {
                                        return command_line_changed(s);
                                    }
                                }
                            }
                            K_UP | K_DOWN | -1277 | -1533 | K_PAGEUP | K_KPAGEUP | K_PAGEDOWN
                            | K_KPAGEDOWN => {}
                            Ctrl_G | Ctrl_T => {
                                if may_do_command_line_next_incsearch(
                                    (*s).firstc,
                                    (*s).count,
                                    &raw mut (*s).is_state,
                                    (*s).c == Ctrl_G,
                                ) == FAIL
                                {
                                    return command_line_not_changed(s);
                                }
                                break 's_680;
                            }
                            Ctrl_V | Ctrl_Q => {
                                (*s).ignore_drag_release = true_0 != 0;
                                putcmdline('^' as ::core::ffi::c_char, true_0 != 0);
                                (*s).c = get_literal(mod_mask & MOD_MASK_SHIFT != 0);
                                (*s).do_abbr = false_0 != 0;
                                ccline.special_char = NUL as ::core::ffi::c_char;
                                if utf_iscomposing_first((*s).c) as ::core::ffi::c_int != 0
                                    && !cmd_silent
                                {
                                    if ui_has(kUICmdline) {
                                        unputcmdline();
                                    } else {
                                        draw_cmdline(ccline.cmdpos, ccline.cmdlen - ccline.cmdpos);
                                        msg_putchar(' ' as ::core::ffi::c_int);
                                        cursorcmd();
                                    }
                                }
                                break 's_680;
                            }
                            Ctrl_K => {
                                (*s).ignore_drag_release = true_0 != 0;
                                putcmdline('?' as ::core::ffi::c_char, true_0 != 0);
                                (*s).c = get_digraph(true_0 != 0);
                                ccline.special_char = NUL as ::core::ffi::c_char;
                                if (*s).c != NUL {
                                    break 's_680;
                                } else {
                                    redrawcmd();
                                    return command_line_not_changed(s);
                                }
                            }
                            Ctrl__ => {
                                if p_ari == 0 {
                                    break 's_680;
                                } else {
                                    return command_line_not_changed(s);
                                }
                            }
                            113 => {
                                if !ccline.mouse_used.is_null() {
                                    *ccline.cmdbuff = NUL as ::core::ffi::c_char;
                                    return 0 as ::core::ffi::c_int;
                                }
                                break 'c_46136;
                            }
                            _ => {
                                break 'c_46136;
                            }
                        }
                        if cmdline_pum_active() as ::core::ffi::c_int != 0
                            && ((*s).c == K_PAGEUP
                                || (*s).c == K_PAGEDOWN
                                || (*s).c == K_KPAGEUP
                                || (*s).c == K_KPAGEDOWN)
                        {
                            let wild_type_0: ::core::ffi::c_int =
                                if (*s).c == K_PAGEDOWN || (*s).c == K_KPAGEDOWN {
                                    WILD_PAGEDOWN as ::core::ffi::c_int
                                } else {
                                    WILD_PAGEUP as ::core::ffi::c_int
                                };
                            if nextwild(
                                &raw mut (*s).xpc,
                                wild_type_0,
                                0 as ::core::ffi::c_int,
                                (*s).firstc != '@' as ::core::ffi::c_int,
                            ) == FAIL
                            {
                                break 's_680;
                            } else {
                                return command_line_changed(s);
                            }
                        } else {
                            match command_line_browse_history(s) {
                                2 => {
                                    (*s).did_hist_navigate = true_0 != 0;
                                    return command_line_changed(s);
                                }
                                3 => return 0 as ::core::ffi::c_int,
                                _ => return command_line_not_changed(s),
                            }
                        }
                    }
                    if !ccline.mouse_used.is_null() && mouse_row < cmdline_row {
                        *ccline.mouse_used = true_0 != 0;
                        return 0 as ::core::ffi::c_int;
                    }
                    break 'c_46093;
                }
                if !((*s).c < 0 as ::core::ffi::c_int) {
                    mod_mask = 0 as ::core::ffi::c_int;
                }
                break 's_680;
            }
            command_line_left_right_mouse(s);
            return command_line_not_changed(s);
        }
        if (*s).do_abbr as ::core::ffi::c_int != 0
            && ((*s).c < 0 as ::core::ffi::c_int || !vim_iswordc((*s).c))
            && (ccheck_abbr(if (*s).c >= 0x100 as ::core::ffi::c_int {
                (*s).c + ABBR_OFF
            } else {
                (*s).c
            }) != 0
                || (*s).c == Ctrl_RSB)
        {
            return command_line_changed(s);
        }
    }
    if (*s).c < 0 as ::core::ffi::c_int || mod_mask != 0 as ::core::ffi::c_int {
        put_on_cmdline(
            get_special_key_name((*s).c, mod_mask),
            -1 as ::core::ffi::c_int,
            true_0 != 0,
        );
    } else {
        let mut j_0: ::core::ffi::c_int =
            utf_char2bytes((*s).c, &raw mut IObuff as *mut ::core::ffi::c_char);
        IObuff[j_0 as usize] = NUL as ::core::ffi::c_char;
        put_on_cmdline(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            j_0,
            true_0 != 0,
        );
    }
    return if ccline.one_key as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        command_line_changed(s)
    };
}
unsafe extern "C" fn may_trigger_cursormovedc(mut s: *mut CommandLineState) {
    if ccline.cmdpos != (*s).prev_cmdpos {
        trigger_cmd_autocmd((*s).cmdline_type, EVENT_CURSORMOVEDC);
        ccline.redraw_state = (if ccline.redraw_state as ::core::ffi::c_uint
            > kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ccline.redraw_state as ::core::ffi::c_uint
        } else {
            kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
        }) as CmdRedraw;
    }
}
unsafe extern "C" fn command_line_not_changed(mut s: *mut CommandLineState) -> ::core::ffi::c_int {
    may_trigger_cursormovedc(s);
    (*s).prev_cmdpos = ccline.cmdpos;
    if !(*s).is_state.incsearch_postponed {
        return 1 as ::core::ffi::c_int;
    }
    return command_line_changed(s);
}
unsafe extern "C" fn empty_pattern(
    mut p: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut delim: ::core::ffi::c_int,
) -> bool {
    let mut magic_val: magic_T = MAGIC_ON;
    if len > 0 as size_t {
        skip_regexp_ex(
            p,
            delim,
            magic_isset() as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut magic_val,
        );
    } else {
        return true_0 != 0;
    }
    return empty_pattern_magic(p, len, magic_val);
}
unsafe extern "C" fn empty_pattern_magic(
    mut p: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut magic_val: magic_T,
) -> bool {
    while len >= 2 as size_t
        && *p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
        && !vim_strchr(
            b"mMvVcCZ\0".as_ptr() as *const ::core::ffi::c_char,
            *p.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        len = len.wrapping_sub(2 as size_t);
    }
    return len == 0 as size_t
        || len > 1 as size_t
            && *p.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int
            && (*p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && magic_val as ::core::ffi::c_uint
                    == MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
                || *p.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
                    && magic_val as ::core::ffi::c_uint
                        == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn cmdpreview_get_bufnr() -> handle_T {
    return cmdpreview_bufnr;
}
#[no_mangle]
pub unsafe extern "C" fn cmdpreview_get_ns() -> ::core::ffi::c_int {
    return cmdpreview_ns;
}
unsafe extern "C" fn cmdpreview_open_buf() -> *mut buf_T {
    let mut cmdpreview_buf: *mut buf_T = if cmdpreview_bufnr != 0 {
        buflist_findnr(cmdpreview_bufnr as ::core::ffi::c_int)
    } else {
        ::core::ptr::null_mut::<buf_T>()
    };
    if cmdpreview_buf.is_null() {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut bufnr: handle_T = nvim_create_buf(false_0 != 0, true_0 != 0, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<buf_T>();
        }
        cmdpreview_buf = buflist_findnr(bufnr as ::core::ffi::c_int);
    }
    if cmdpreview_buf == curbuf {
        return ::core::ptr::null_mut::<buf_T>();
    }
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
    aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
    let mut retv: ::core::ffi::c_int = rename_buffer(b"[Preview]\0".as_ptr()
        as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char);
    aucmd_restbuf(&raw mut aco);
    if retv == FAIL {
        return ::core::ptr::null_mut::<buf_T>();
    }
    aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
    buf_clear();
    (*curbuf).b_p_ma = true_0;
    (*curbuf).b_p_ul = -1 as OptInt;
    (*curbuf).b_p_tw = 0 as OptInt;
    aucmd_restbuf(&raw mut aco);
    cmdpreview_bufnr = (*cmdpreview_buf).handle;
    return cmdpreview_buf;
}
unsafe extern "C" fn cmdpreview_open_win(mut cmdpreview_buf: *mut buf_T) -> *mut win_T {
    let mut save_curwin: *mut win_T = curwin;
    if win_split(p_cwh as ::core::ffi::c_int, WSP_BOT as ::core::ffi::c_int) == FAIL {
        return ::core::ptr::null_mut::<win_T>();
    }
    let mut preview_win: *mut win_T = curwin;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: ::core::ffi::c_int = OK;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    result = do_buffer(
        DOBUF_GOTO as ::core::ffi::c_int,
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*cmdpreview_buf).handle as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    try_leave(&raw mut tstate, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int || result == FAIL {
        api_clear_error(&raw mut err);
        return ::core::ptr::null_mut::<win_T>();
    }
    (*curwin).w_onebuf_opt.wo_cul = false_0;
    (*curwin).w_onebuf_opt.wo_cuc = false_0;
    (*curwin).w_onebuf_opt.wo_spell = false_0;
    (*curwin).w_onebuf_opt.wo_fen = false_0;
    win_enter(save_curwin, false_0 != 0);
    return preview_win;
}
unsafe extern "C" fn cmdpreview_close_win() {
    let mut buf: *mut buf_T = if cmdpreview_bufnr != 0 {
        buflist_findnr(cmdpreview_bufnr as ::core::ffi::c_int)
    } else {
        ::core::ptr::null_mut::<buf_T>()
    };
    if !buf.is_null() {
        close_windows(buf, false_0 != 0);
    }
}
unsafe extern "C" fn cmdpreview_save_undo(mut cp_undoinfo: *mut CpUndoInfo, mut buf: *mut buf_T) {
    (*cp_undoinfo).save_b_u_synced = (*buf).b_u_synced;
    (*cp_undoinfo).save_b_u_oldhead = (*buf).b_u_oldhead;
    (*cp_undoinfo).save_b_u_newhead = (*buf).b_u_newhead;
    (*cp_undoinfo).save_b_u_curhead = (*buf).b_u_curhead;
    (*cp_undoinfo).save_b_u_numhead = (*buf).b_u_numhead;
    (*cp_undoinfo).save_b_u_seq_last = (*buf).b_u_seq_last;
    (*cp_undoinfo).save_b_u_save_nr_last = (*buf).b_u_save_nr_last;
    (*cp_undoinfo).save_b_u_seq_cur = (*buf).b_u_seq_cur;
    (*cp_undoinfo).save_b_u_time_cur = (*buf).b_u_time_cur;
    (*cp_undoinfo).save_b_u_save_nr_cur = (*buf).b_u_save_nr_cur;
    (*cp_undoinfo).save_b_u_line_ptr = (*buf).b_u_line_ptr;
    (*cp_undoinfo).save_b_u_line_lnum = (*buf).b_u_line_lnum;
    (*cp_undoinfo).save_b_u_line_colnr = (*buf).b_u_line_colnr;
}
unsafe extern "C" fn cmdpreview_restore_undo(
    mut cp_undoinfo: *const CpUndoInfo,
    mut buf: *mut buf_T,
) {
    (*buf).b_u_oldhead = (*cp_undoinfo).save_b_u_oldhead;
    (*buf).b_u_newhead = (*cp_undoinfo).save_b_u_newhead;
    (*buf).b_u_curhead = (*cp_undoinfo).save_b_u_curhead;
    (*buf).b_u_numhead = (*cp_undoinfo).save_b_u_numhead;
    (*buf).b_u_seq_last = (*cp_undoinfo).save_b_u_seq_last;
    (*buf).b_u_save_nr_last = (*cp_undoinfo).save_b_u_save_nr_last;
    (*buf).b_u_seq_cur = (*cp_undoinfo).save_b_u_seq_cur;
    (*buf).b_u_time_cur = (*cp_undoinfo).save_b_u_time_cur;
    (*buf).b_u_save_nr_cur = (*cp_undoinfo).save_b_u_save_nr_cur;
    (*buf).b_u_line_ptr = (*cp_undoinfo).save_b_u_line_ptr;
    (*buf).b_u_line_lnum = (*cp_undoinfo).save_b_u_line_lnum;
    (*buf).b_u_line_colnr = (*cp_undoinfo).save_b_u_line_colnr;
    if (*buf).b_u_curhead.is_null() {
        (*buf).b_u_synced = (*cp_undoinfo).save_b_u_synced;
    }
}
unsafe extern "C" fn cmdpreview_prepare(mut cpinfo: *mut CpInfo) {
    let mut saved_bufs: Set_ptr_t = SET_INIT;
    (*cpinfo).buf_info.capacity = 0 as size_t;
    (*cpinfo).buf_info.size = (*cpinfo).buf_info.capacity;
    (*cpinfo).buf_info.items = ::core::ptr::null_mut::<CpBufInfo>();
    (*cpinfo).win_info.capacity = 0 as size_t;
    (*cpinfo).win_info.size = (*cpinfo).win_info.capacity;
    (*cpinfo).win_info.items = ::core::ptr::null_mut::<CpWinInfo>();
    let mut win: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !win.is_null() {
        let mut buf: *mut buf_T = (*win).w_buffer;
        if (*buf).handle != cmdpreview_bufnr {
            if !set_has_ptr_t(&raw mut saved_bufs, buf as ptr_t) {
                let mut cp_bufinfo: CpBufInfo = CpBufInfo {
                    buf: ::core::ptr::null_mut::<buf_T>(),
                    save_b_p_ul: 0,
                    save_b_p_ma: 0,
                    save_b_changed: 0,
                    save_b_op_start: pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    },
                    save_b_op_end: pos_T {
                        lnum: 0,
                        col: 0,
                        coladd: 0,
                    },
                    save_changedtick: 0,
                    undo_info: CpUndoInfo {
                        save_b_u_oldhead: ::core::ptr::null_mut::<u_header_T>(),
                        save_b_u_newhead: ::core::ptr::null_mut::<u_header_T>(),
                        save_b_u_curhead: ::core::ptr::null_mut::<u_header_T>(),
                        save_b_u_numhead: 0,
                        save_b_u_synced: false,
                        save_b_u_seq_last: 0,
                        save_b_u_save_nr_last: 0,
                        save_b_u_seq_cur: 0,
                        save_b_u_time_cur: 0,
                        save_b_u_save_nr_cur: 0,
                        save_b_u_line_ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        save_b_u_line_lnum: 0,
                        save_b_u_line_colnr: 0,
                    },
                };
                cp_bufinfo.buf = buf;
                cp_bufinfo.save_b_p_ma = (*buf).b_p_ma;
                cp_bufinfo.save_b_p_ul = (*buf).b_p_ul;
                cp_bufinfo.save_b_changed = (*buf).b_changed;
                cp_bufinfo.save_b_op_start = (*buf).b_op_start;
                cp_bufinfo.save_b_op_end = (*buf).b_op_end;
                cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
                cmdpreview_save_undo(&raw mut cp_bufinfo.undo_info, buf);
                if (*cpinfo).buf_info.size == (*cpinfo).buf_info.capacity {
                    (*cpinfo).buf_info.capacity = if (*cpinfo).buf_info.capacity != 0 {
                        (*cpinfo).buf_info.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*cpinfo).buf_info.items = xrealloc(
                        (*cpinfo).buf_info.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<CpBufInfo>()
                            .wrapping_mul((*cpinfo).buf_info.capacity),
                    ) as *mut CpBufInfo;
                } else {
                };
                let c2rust_fresh15 = (*cpinfo).buf_info.size;
                (*cpinfo).buf_info.size = (*cpinfo).buf_info.size.wrapping_add(1);
                *(*cpinfo).buf_info.items.offset(c2rust_fresh15 as isize) = cp_bufinfo;
                set_put_ptr_t(
                    &raw mut saved_bufs,
                    buf as ptr_t,
                    ::core::ptr::null_mut::<*mut ptr_t>(),
                );
                u_clearall(buf);
                (*buf).b_p_ul = INT_MAX as OptInt;
            }
            let mut cp_wininfo: CpWinInfo = CpWinInfo {
                win: ::core::ptr::null_mut::<win_T>(),
                save_w_cursor: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                save_viewstate: viewstate_T {
                    vs_curswant: 0,
                    vs_leftcol: 0,
                    vs_skipcol: 0,
                    vs_topline: 0,
                    vs_topfill: 0,
                    vs_botline: 0,
                    vs_empty_rows: 0,
                },
                save_w_p_cul: 0,
                save_w_p_cuc: 0,
            };
            cp_wininfo.win = win;
            cp_wininfo.save_w_cursor = (*win).w_cursor;
            save_viewstate(win, &raw mut cp_wininfo.save_viewstate);
            cp_wininfo.save_w_p_cul = (*win).w_onebuf_opt.wo_cul;
            cp_wininfo.save_w_p_cuc = (*win).w_onebuf_opt.wo_cuc;
            if (*cpinfo).win_info.size == (*cpinfo).win_info.capacity {
                (*cpinfo).win_info.capacity = if (*cpinfo).win_info.capacity != 0 {
                    (*cpinfo).win_info.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*cpinfo).win_info.items = xrealloc(
                    (*cpinfo).win_info.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<CpWinInfo>().wrapping_mul((*cpinfo).win_info.capacity),
                ) as *mut CpWinInfo;
            } else {
            };
            let c2rust_fresh16 = (*cpinfo).win_info.size;
            (*cpinfo).win_info.size = (*cpinfo).win_info.size.wrapping_add(1);
            *(*cpinfo).win_info.items.offset(c2rust_fresh16 as isize) = cp_wininfo;
            (*win).w_onebuf_opt.wo_cul = false_0;
            (*win).w_onebuf_opt.wo_cuc = false_0;
        }
        win = (*win).w_next;
    }
    xfree(saved_bufs.keys as *mut ::core::ffi::c_void);
    xfree(saved_bufs.h.hash as *mut ::core::ffi::c_void);
    saved_bufs = SET_INIT;
    (*cpinfo).save_hls = p_hls != 0;
    (*cpinfo).save_cmdmod = cmdmod;
    win_size_save(&raw mut (*cpinfo).save_view);
    save_search_patterns();
    p_hls = false_0;
    cmdmod.cmod_split = 0 as ::core::ffi::c_int;
    cmdmod.cmod_tab = 0 as ::core::ffi::c_int;
    cmdmod.cmod_flags |= CMOD_NOSWAPFILE as ::core::ffi::c_int;
    u_sync(true_0 != 0);
}
unsafe extern "C" fn cmdpreview_restore_state(mut cpinfo: *mut CpInfo) {
    let mut i: size_t = 0 as size_t;
    while i < (*cpinfo).buf_info.size {
        let mut cp_bufinfo: CpBufInfo = *(*cpinfo).buf_info.items.offset(i as isize);
        let mut buf: *mut buf_T = cp_bufinfo.buf;
        (*buf).b_changed = cp_bufinfo.save_b_changed;
        extmark_clear(
            buf,
            cmdpreview_ns as uint32_t,
            0 as ::core::ffi::c_int,
            0 as colnr_T,
            MAXLNUM as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
        );
        if (*buf).b_u_seq_cur != cp_bufinfo.undo_info.save_b_u_seq_cur {
            let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut uhp: *mut u_header_T = if !(*buf).b_u_curhead.is_null() {
                (*buf).b_u_curhead
            } else {
                (*buf).b_u_newhead
            };
            while !uhp.is_null() {
                uhp = (*uhp).uh_next.ptr;
                count += 1;
            }
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
            aucmd_prepbuf(&raw mut aco, buf);
            if (*curbuf).b_u_synced as ::core::ffi::c_int == false_0 {
                u_sync(true_0 != 0);
            }
            if !u_undo_and_forget(count, false_0 != 0) {
                abort();
            }
            aucmd_restbuf(&raw mut aco);
        }
        u_blockfree(buf);
        cmdpreview_restore_undo(&raw mut cp_bufinfo.undo_info, buf);
        (*buf).b_op_start = cp_bufinfo.save_b_op_start;
        (*buf).b_op_end = cp_bufinfo.save_b_op_end;
        if cp_bufinfo.save_changedtick != buf_get_changedtick(buf) {
            buf_set_changedtick(buf, cp_bufinfo.save_changedtick);
        }
        (*buf).b_p_ul = cp_bufinfo.save_b_p_ul;
        (*buf).b_p_ma = cp_bufinfo.save_b_p_ma;
        i = i.wrapping_add(1);
    }
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*cpinfo).win_info.size {
        let mut cp_wininfo: CpWinInfo = *(*cpinfo).win_info.items.offset(i_0 as isize);
        let mut win: *mut win_T = cp_wininfo.win;
        (*win).w_cursor = cp_wininfo.save_w_cursor;
        restore_viewstate(win, &raw mut cp_wininfo.save_viewstate);
        (*win).w_onebuf_opt.wo_cul = cp_wininfo.save_w_p_cul;
        (*win).w_onebuf_opt.wo_cuc = cp_wininfo.save_w_p_cuc;
        update_topline(win);
        i_0 = i_0.wrapping_add(1);
    }
    cmdmod = (*cpinfo).save_cmdmod;
    p_hls = (*cpinfo).save_hls as ::core::ffi::c_int;
    restore_search_patterns();
    win_size_restore(&raw mut (*cpinfo).save_view);
    ga_clear(&raw mut (*cpinfo).save_view);
    xfree((*cpinfo).win_info.items as *mut ::core::ffi::c_void);
    (*cpinfo).win_info.capacity = 0 as size_t;
    (*cpinfo).win_info.size = (*cpinfo).win_info.capacity;
    (*cpinfo).win_info.items = ::core::ptr::null_mut::<CpWinInfo>();
    xfree((*cpinfo).buf_info.items as *mut ::core::ffi::c_void);
    (*cpinfo).buf_info.capacity = 0 as size_t;
    (*cpinfo).buf_info.size = (*cpinfo).buf_info.capacity;
    (*cpinfo).buf_info.items = ::core::ptr::null_mut::<CpBufInfo>();
}
unsafe extern "C" fn cmdpreview_may_show(mut _s: *mut CommandLineState) -> bool {
    let mut cpinfo: CpInfo = CpInfo {
        win_info: C2Rust_Unnamed_50 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<CpWinInfo>(),
        },
        buf_info: C2Rust_Unnamed_49 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<CpBufInfo>(),
        },
        save_hls: false,
        save_cmdmod: cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        },
        save_view: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
    };
    let mut icm_split: bool = false;
    let mut cmdpreview_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut cmdpreview_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut err: Error = Error {
        type_0: kErrorTypeException,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
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
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        },
        magic: C2Rust_Unnamed_21 {
            file: false,
            bar: false,
        },
    };
    let mut cmdpreview_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmdline: *mut ::core::ffi::c_char = xstrdup(ccline.cmdbuff);
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    emsg_off += 1;
    if !parse_cmdline(
        &raw mut cmdline,
        &raw mut ea,
        &raw mut cmdinfo,
        &raw mut errormsg,
    ) {
        emsg_off -= 1;
    } else {
        emsg_off -= 1;
        if ea.argt & EX_PREVIEW as uint32_t == 0 {
            undo_cmdmod(&raw mut cmdinfo.cmdmod);
        } else {
            cursorcmd();
            cmdline_ui_flush();
            if ea.argt & EX_RANGE as uint32_t != 0 && ea.line1 > ea.line2 {
                let mut lnum: linenr_T = ea.line1;
                ea.line1 = ea.line2;
                ea.line2 = lnum;
            }
            cpinfo = CpInfo {
                win_info: C2Rust_Unnamed_50 {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<CpWinInfo>(),
                },
                buf_info: C2Rust_Unnamed_49 {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<CpBufInfo>(),
                },
                save_hls: false,
                save_cmdmod: cmdmod_T {
                    cmod_flags: 0,
                    cmod_split: 0,
                    cmod_tab: 0,
                    cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    cmod_filter_regmatch: regmatch_T {
                        regprog: ::core::ptr::null_mut::<regprog_T>(),
                        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                        rm_matchcol: 0,
                        rm_ic: false,
                    },
                    cmod_filter_force: false,
                    cmod_verbose: 0,
                    cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    cmod_did_sandbox: 0,
                    cmod_verbose_save: 0,
                    cmod_save_msg_silent: 0,
                    cmod_save_msg_scroll: 0,
                    cmod_did_esilent: 0,
                },
                save_view: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
            };
            icm_split = *p_icm as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
            cmdpreview_buf = ::core::ptr::null_mut::<buf_T>();
            cmdpreview_win = ::core::ptr::null_mut::<win_T>();
            emsg_silent += 1;
            msg_silent += 1;
            block_autocmds();
            cmdpreview_prepare(&raw mut cpinfo);
            if icm_split as ::core::ffi::c_int != 0 && {
                cmdpreview_buf = cmdpreview_open_buf();
                cmdpreview_buf.is_null()
            } {
                set_option_direct(
                    kOptInccommand,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: String_0 {
                                data: b"nosplit\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                    .wrapping_sub(1 as size_t),
                            },
                        },
                    },
                    0 as ::core::ffi::c_int,
                    SID_NONE,
                );
                icm_split = false_0 != 0;
            }
            if cmdpreview_ns == 0 {
                cmdpreview_ns = nvim_create_namespace(String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                }) as ::core::ffi::c_int;
            }
            cmdpreview = true_0 != 0;
            err = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            let mut tstate: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate);
            cmdpreview_type = execute_cmd(&raw mut ea, &raw mut cmdinfo, true);
            try_leave(&raw mut tstate, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                api_clear_error(&raw mut err);
                cmdpreview_type = 0 as ::core::ffi::c_int;
            }
            if icm_split as ::core::ffi::c_int != 0
                && cmdpreview_type == 2 as ::core::ffi::c_int
                && {
                    cmdpreview_win = cmdpreview_open_win(cmdpreview_buf);
                    cmdpreview_win.is_null()
                }
            {
                cmdpreview_type = 1 as ::core::ffi::c_int;
            }
            if cmdpreview_type != 0 as ::core::ffi::c_int {
                let mut save_rd: ::core::ffi::c_int = RedrawingDisabled;
                RedrawingDisabled = 0 as ::core::ffi::c_int;
                update_screen();
                RedrawingDisabled = save_rd;
            }
            if icm_split as ::core::ffi::c_int != 0
                && cmdpreview_type == 2 as ::core::ffi::c_int
                && !cmdpreview_win.is_null()
            {
                cmdpreview_close_win();
            }
            cmdpreview_restore_state(&raw mut cpinfo);
            unblock_autocmds();
            msg_silent -= 1;
            emsg_silent -= 1;
            redrawcmdline();
        }
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    return cmdpreview_type != 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn do_autocmd_cmdlinechanged(mut firstc: ::core::ffi::c_int) {
    if has_event(EVENT_CMDLINECHANGED) {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        let mut firstcbuf: [::core::ffi::c_char; 2] = [0; 2];
        firstcbuf[0 as ::core::ffi::c_int as usize] = firstc as ::core::ffi::c_char;
        firstcbuf[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            b"cmdtype\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            &raw mut firstcbuf as *mut ::core::ffi::c_char,
        );
        tv_dict_add_nr(
            dict,
            b"cmdlevel\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            ccline.level as varnumber_T,
        );
        tv_dict_set_keys_readonly(dict);
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        apply_autocmds(
            EVENT_CMDLINECHANGED,
            &raw mut firstcbuf as *mut ::core::ffi::c_char,
            &raw mut firstcbuf as *mut ::core::ffi::c_char,
            false,
            curbuf,
        );
        restore_v_event(dict, &raw mut save_v_event);
        try_leave(&raw mut tstate, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            msg_scroll = true_0;
            msg_puts_hl(err.msg, HLF_E as ::core::ffi::c_int, true_0 != 0);
            api_clear_error(&raw mut err);
            redrawcmd();
        }
    }
}
unsafe extern "C" fn command_line_changed(mut s: *mut CommandLineState) -> ::core::ffi::c_int {
    let prev_cmdpreview: bool = cmdpreview;
    if !((*s).firstc == ':' as ::core::ffi::c_int
        && current_sctx.sc_sid == 0 as ::core::ffi::c_int
        && *p_icm as ::core::ffi::c_int != NUL
        && !exmode_active
        && cmdline_star == 0 as ::core::ffi::c_int
        && vpeekc_any() == 0
        && cmdpreview_may_show(s) as ::core::ffi::c_int != 0)
    {
        cmdpreview = false_0 != 0;
        if prev_cmdpreview {
            update_screen();
        }
        if (*s).xpc.xp_context == EXPAND_NOTHING as ::core::ffi::c_int
            && (KeyTyped as ::core::ffi::c_int != 0 || vpeekc() == NUL)
        {
            may_do_incsearch_highlighting((*s).firstc, (*s).count, &raw mut (*s).is_state);
        }
    }
    if !ccline.cmdbuff_replaced
        && (ccline.cmdpos != (*s).prev_cmdpos
            || !(*s).prev_cmdbuff.is_null()
                && strcmp((*s).prev_cmdbuff, ccline.cmdbuff) != 0 as ::core::ffi::c_int)
    {
        do_autocmd_cmdlinechanged(if (*s).firstc > 0 as ::core::ffi::c_int {
            (*s).firstc
        } else {
            '-' as ::core::ffi::c_int
        });
    }
    may_trigger_cursormovedc(s);
    if p_arshape != 0 && p_tbidi == 0 {
        if !ui_has(kUICmdline) && vpeekc() == NUL {
            redrawcmd();
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn abandon_cmdline() {
    dealloc_cmdbuff();
    if msg_scrolled == 0 as ::core::ffi::c_int {
        compute_cmdrow();
    }
    if !ccline.one_key {
        msg(
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        redraw_cmdline = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn getcmdline(
    mut firstc: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    return command_line_enter(firstc, count, indent, true_0 != 0) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn getcmdline_prompt(
    firstc: ::core::ffi::c_int,
    prompt: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    xp_context: ::core::ffi::c_int,
    xp_arg: *const ::core::ffi::c_char,
    highlight_callback: Callback,
    mut one_key: bool,
    mut mouse_used: *mut bool,
) -> *mut ::core::ffi::c_char {
    let msg_col_save: ::core::ffi::c_int = msg_col;
    let mut save_ccline: CmdlineInfo = CmdlineInfo {
        cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdbufflen: 0,
        cmdlen: 0,
        cmdpos: 0,
        cmdspos: 0,
        cmdfirstc: 0,
        cmdindent: 0,
        cmdprompt: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        hl_id: 0,
        overstrike: 0,
        xpc: ::core::ptr::null_mut::<expand_T>(),
        xp_context: 0,
        xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        input_fn: 0,
        cmdbuff_replaced: false,
        prompt_id: 0,
        highlight_callback: Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        last_colors: ColoredCmdline {
            prompt_id: 0,
            cmdbuff: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            colors: CmdlineColors {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<CmdlineColorChunk>(),
            },
        },
        level: 0,
        prev_ccline: ::core::ptr::null_mut::<CmdlineInfo>(),
        special_char: 0,
        special_shift: false,
        redraw_state: kCmdRedrawNone,
        one_key: false,
        mouse_used: ::core::ptr::null_mut::<bool>(),
    };
    let mut did_save_ccline: bool = false_0 != 0;
    if !ccline.cmdbuff.is_null() {
        save_cmdline(&raw mut save_ccline);
        did_save_ccline = true_0 != 0;
    } else {
        memset(
            &raw mut ccline as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CmdlineInfo>(),
        );
    }
    let c2rust_fresh32 = last_prompt_id;
    last_prompt_id = last_prompt_id.wrapping_add(1);
    ccline.prompt_id = c2rust_fresh32;
    ccline.cmdprompt = prompt as *mut ::core::ffi::c_char;
    ccline.hl_id = hl_id;
    ccline.xp_context = xp_context;
    ccline.xp_arg = xp_arg as *mut ::core::ffi::c_char;
    ccline.input_fn = (firstc == '@' as ::core::ffi::c_int) as ::core::ffi::c_int;
    ccline.highlight_callback = highlight_callback;
    ccline.one_key = one_key;
    ccline.mouse_used = mouse_used;
    let cmd_silent_saved: bool = cmd_silent;
    let mut msg_silent_saved: ::core::ffi::c_int = msg_silent;
    msg_silent = 0 as ::core::ffi::c_int;
    cmd_silent = false_0 != 0;
    let ret: *mut ::core::ffi::c_char = command_line_enter(
        firstc,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        false_0 != 0,
    ) as *mut ::core::ffi::c_char;
    ccline.redraw_state = kCmdRedrawNone;
    if did_save_ccline {
        restore_cmdline(&raw mut save_ccline);
    }
    msg_silent = msg_silent_saved;
    cmd_silent = cmd_silent_saved;
    if !ccline.cmdbuff.is_null() {
        msg_col = msg_col_save;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn check_opt_wim() -> ::core::ffi::c_int {
    let mut new_wim_flags: [uint8_t; 4] = [0; 4];
    let mut i: ::core::ffi::c_int = 0;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        new_wim_flags[i as usize] = 0 as uint8_t;
        i += 1;
    }
    let mut p: *mut ::core::ffi::c_char = p_wim;
    while *p != 0 {
        i = 0 as ::core::ffi::c_int;
        while *p.offset(i as isize) as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p.offset(i as isize) as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p.offset(i as isize) as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p.offset(i as isize) as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        {
            i += 1;
        }
        if *p.offset(i as isize) as ::core::ffi::c_int != NUL
            && *p.offset(i as isize) as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            && *p.offset(i as isize) as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            return FAIL;
        }
        if i == 7 as ::core::ffi::c_int
            && strncmp(
                p,
                b"longest\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                | kOptWimFlagLongest as ::core::ffi::c_int)
                as uint8_t;
        } else if i == 4 as ::core::ffi::c_int
            && strncmp(
                p,
                b"full\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                | kOptWimFlagFull as ::core::ffi::c_int)
                as uint8_t;
        } else if i == 4 as ::core::ffi::c_int
            && strncmp(
                p,
                b"list\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                | kOptWimFlagList as ::core::ffi::c_int)
                as uint8_t;
        } else if i == 8 as ::core::ffi::c_int
            && strncmp(
                p,
                b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                | kOptWimFlagLastused as ::core::ffi::c_int)
                as uint8_t;
        } else if i == 8 as ::core::ffi::c_int
            && strncmp(
                p,
                b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            new_wim_flags[idx as usize] = (new_wim_flags[idx as usize] as ::core::ffi::c_int
                | kOptWimFlagNoselect as ::core::ffi::c_int)
                as uint8_t;
        } else {
            return FAIL;
        }
        p = p.offset(i as isize);
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            if idx == 3 as ::core::ffi::c_int {
                return FAIL;
            }
            idx += 1;
        }
        p = p.offset(1);
    }
    while idx < 3 as ::core::ffi::c_int {
        new_wim_flags[(idx + 1 as ::core::ffi::c_int) as usize] = new_wim_flags[idx as usize];
        idx += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        wim_flags[i as usize] = new_wim_flags[i as usize];
        i += 1;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn text_locked() -> bool {
    if cmdwin_type != 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if expr_map_locked() {
        return true_0 != 0;
    }
    return textlock != 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn text_locked_msg() {
    emsg(gettext(get_text_locked_msg()));
}
#[no_mangle]
pub unsafe extern "C" fn get_text_locked_msg() -> *const ::core::ffi::c_char {
    if cmdwin_type != 0 as ::core::ffi::c_int {
        return &raw const e_cmdwin as *const ::core::ffi::c_char;
    } else {
        return &raw const e_textlock as *const ::core::ffi::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn text_or_buf_locked() -> bool {
    if text_locked() {
        text_locked_msg();
        return true_0 != 0;
    }
    return curbuf_locked();
}
#[no_mangle]
pub unsafe extern "C" fn curbuf_locked() -> bool {
    if (*curbuf).b_ro_locked > 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_cannot_edit_other_buf as *const ::core::ffi::c_char,
        ));
        return true_0 != 0;
    }
    return allbuf_locked();
}
#[no_mangle]
pub unsafe extern "C" fn allbuf_locked() -> bool {
    if allbuf_lock > 0 as ::core::ffi::c_int {
        emsg(gettext(
            b"E811: Not allowed to change buffer information now\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn cmdline_charsize(mut idx: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if cmdline_star > 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    return ptr2cells(ccline.cmdbuff.offset(idx as isize));
}
unsafe extern "C" fn cmd_startcol() -> ::core::ffi::c_int {
    return ccline.cmdindent
        + (if ccline.cmdfirstc != NUL {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}
#[no_mangle]
pub unsafe extern "C" fn cmd_screencol(mut bytepos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut m: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_int = cmd_startcol();
    if KeyTyped {
        m = if !cmdline_win.is_null() {
            (*cmdline_win).w_view_width * (*cmdline_win).w_view_height
        } else {
            Columns * Rows
        };
        if m < 0 as ::core::ffi::c_int {
            m = MAXCOL as ::core::ffi::c_int;
        }
    } else {
        m = MAXCOL as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ccline.cmdlen && i < bytepos {
        let mut c: ::core::ffi::c_int = cmdline_charsize(i);
        correct_screencol(i, c, &raw mut col);
        col += c;
        if col >= m {
            col -= c;
            break;
        } else {
            i += utfc_ptr2len(ccline.cmdbuff.offset(i as isize));
        }
    }
    return col;
}
unsafe extern "C" fn correct_screencol(
    mut idx: ::core::ffi::c_int,
    mut cells: ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    if utfc_ptr2len(ccline.cmdbuff.offset(idx as isize)) > 1 as ::core::ffi::c_int
        && utf_ptr2cells(ccline.cmdbuff.offset(idx as isize)) > 1 as ::core::ffi::c_int
        && *col % Columns + cells > Columns
    {
        *col += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn getexline(
    mut c: ::core::ffi::c_int,
    mut _cookie: *mut ::core::ffi::c_void,
    mut indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    if exec_from_reg as ::core::ffi::c_int != 0 && vpeekc() == ':' as ::core::ffi::c_int {
        vgetc();
    }
    return getcmdline(c, 1 as ::core::ffi::c_int, indent, do_concat);
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_overstrike() -> bool {
    return ccline.overstrike != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_at_end() -> bool {
    return ccline.cmdpos >= ccline.cmdlen;
}
unsafe extern "C" fn dealloc_cmdbuff() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut ccline.cmdbuff as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    ccline.cmdbufflen = 0 as ::core::ffi::c_int;
    ccline.cmdlen = ccline.cmdbufflen;
}
unsafe extern "C" fn alloc_cmdbuff(mut len: ::core::ffi::c_int) {
    if len < 80 as ::core::ffi::c_int {
        len = 100 as ::core::ffi::c_int;
    } else {
        len += 20 as ::core::ffi::c_int;
    }
    ccline.cmdbuff = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
    ccline.cmdbufflen = len;
}
#[no_mangle]
pub unsafe extern "C" fn realloc_cmdbuff(mut len: ::core::ffi::c_int) {
    if len < ccline.cmdbufflen {
        return;
    }
    let mut p: *mut ::core::ffi::c_char = ccline.cmdbuff;
    alloc_cmdbuff(len);
    memmove(
        ccline.cmdbuff as *mut ::core::ffi::c_void,
        p as *const ::core::ffi::c_void,
        ccline.cmdlen as size_t,
    );
    *ccline.cmdbuff.offset(ccline.cmdlen as isize) = NUL as ::core::ffi::c_char;
    if !ccline.xpc.is_null()
        && !(*ccline.xpc).xp_pattern.is_null()
        && (*ccline.xpc).xp_context != EXPAND_NOTHING as ::core::ffi::c_int
        && (*ccline.xpc).xp_context != EXPAND_UNSUCCESSFUL as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int =
            (*ccline.xpc).xp_pattern.offset_from(p) as ::core::ffi::c_int;
        if i >= 0 as ::core::ffi::c_int && i <= ccline.cmdlen {
            (*ccline.xpc).xp_pattern = ccline.cmdbuff.offset(i as isize);
        }
    }
    xfree(p as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn color_expr_cmdline(
    colored_ccline: *const CmdlineInfo,
    ret_ccline_colors: *mut ColoredCmdline,
) {
    let mut parser_lines: [ParserLine; 2] = [
        ParserLine {
            data: (*colored_ccline).cmdbuff,
            size: strlen((*colored_ccline).cmdbuff),
            allocated: false_0 != 0,
        },
        ParserLine {
            data: ::core::ptr::null::<::core::ffi::c_char>(),
            size: 0 as size_t,
            allocated: false_0 != 0,
        },
    ];
    let mut plines_p: *mut ParserLine = &raw mut parser_lines as *mut ParserLine;
    let mut colors: ParserHighlight = ParserHighlight {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<ParserHighlightChunk>(),
        init_array: [ParserHighlightChunk {
            start: ParserPosition { line: 0, col: 0 },
            end_col: 0,
            group: ::core::ptr::null::<::core::ffi::c_char>(),
        }; 16],
    };
    colors.capacity = ::core::mem::size_of::<[ParserHighlightChunk; 16]>()
        .wrapping_div(::core::mem::size_of::<ParserHighlightChunk>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserHighlightChunk; 16]>()
                .wrapping_rem(::core::mem::size_of::<ParserHighlightChunk>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    colors.size = 0 as size_t;
    colors.items = &raw mut colors.init_array as *mut ParserHighlightChunk;
    let mut pstate: ParserState = ParserState {
        reader: ParserInputReader {
            get_line: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            lines: C2Rust_Unnamed_35 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<ParserLine>(),
                init_array: [ParserLine {
                    data: ::core::ptr::null::<::core::ffi::c_char>(),
                    size: 0,
                    allocated: false,
                }; 4],
            },
            conv: vimconv_T {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false,
            },
        },
        pos: ParserPosition { line: 0, col: 0 },
        stack: C2Rust_Unnamed_30 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserStateItem>(),
            init_array: [ParserStateItem {
                type_0: kPTopStateParsingCommand,
                data: C2Rust_Unnamed_31 {
                    expr: C2Rust_Unnamed_32 {
                        type_0: kExprUnknown,
                    },
                },
            }; 16],
        },
        colors: ::core::ptr::null_mut::<ParserHighlight>(),
        can_continuate: false,
    };
    viml_parser_init(
        &raw mut pstate,
        Some(
            parser_simple_get_line
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ParserLine) -> (),
        ),
        &raw mut plines_p as *mut ::core::ffi::c_void,
        &raw mut colors,
    );
    let mut east: ExprAST =
        viml_pexpr_parse(&raw mut pstate, kExprFlagsDisallowEOC as ::core::ffi::c_int);
    viml_pexpr_free_ast(east);
    viml_parser_destroy(&raw mut pstate);
    (*ret_ccline_colors).colors.capacity = colors.size;
    (*ret_ccline_colors).colors.items = xrealloc(
        (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<CmdlineColorChunk>()
            .wrapping_mul((*ret_ccline_colors).colors.capacity),
    ) as *mut CmdlineColorChunk;
    let mut prev_end: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < colors.size {
        let chunk: ParserHighlightChunk = *colors.items.offset(i as isize);
        '_c2rust_label: {
            if chunk.start.col < 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"chunk.start.col < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    3313 as ::core::ffi::c_uint,
                    b"void color_expr_cmdline(const CmdlineInfo *const, ColoredCmdline *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        '_c2rust_label_0: {
            if chunk.end_col < 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"chunk.end_col < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    3314 as ::core::ffi::c_uint,
                    b"void color_expr_cmdline(const CmdlineInfo *const, ColoredCmdline *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if chunk.start.col != prev_end {
            if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
                (*ret_ccline_colors).colors.capacity = if (*ret_ccline_colors).colors.capacity != 0
                {
                    (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*ret_ccline_colors).colors.items = xrealloc(
                    (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<CmdlineColorChunk>()
                        .wrapping_mul((*ret_ccline_colors).colors.capacity),
                ) as *mut CmdlineColorChunk;
            } else {
            };
            let c2rust_fresh12 = (*ret_ccline_colors).colors.size;
            (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
            *(*ret_ccline_colors)
                .colors
                .items
                .offset(c2rust_fresh12 as isize) = CmdlineColorChunk {
                start: prev_end as ::core::ffi::c_int,
                end: chunk.start.col as ::core::ffi::c_int,
                hl_id: 0 as ::core::ffi::c_int,
            };
        }
        if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
            (*ret_ccline_colors).colors.capacity = if (*ret_ccline_colors).colors.capacity != 0 {
                (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*ret_ccline_colors).colors.items = xrealloc(
                (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<CmdlineColorChunk>()
                    .wrapping_mul((*ret_ccline_colors).colors.capacity),
            ) as *mut CmdlineColorChunk;
        } else {
        };
        let c2rust_fresh13 = (*ret_ccline_colors).colors.size;
        (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
        *(*ret_ccline_colors)
            .colors
            .items
            .offset(c2rust_fresh13 as isize) = CmdlineColorChunk {
            start: chunk.start.col as ::core::ffi::c_int,
            end: chunk.end_col as ::core::ffi::c_int,
            hl_id: syn_name2id(chunk.group),
        };
        prev_end = chunk.end_col;
        i = i.wrapping_add(1);
    }
    if prev_end < (*colored_ccline).cmdlen as size_t {
        if (*ret_ccline_colors).colors.size == (*ret_ccline_colors).colors.capacity {
            (*ret_ccline_colors).colors.capacity = if (*ret_ccline_colors).colors.capacity != 0 {
                (*ret_ccline_colors).colors.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*ret_ccline_colors).colors.items = xrealloc(
                (*ret_ccline_colors).colors.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<CmdlineColorChunk>()
                    .wrapping_mul((*ret_ccline_colors).colors.capacity),
            ) as *mut CmdlineColorChunk;
        } else {
        };
        let c2rust_fresh14 = (*ret_ccline_colors).colors.size;
        (*ret_ccline_colors).colors.size = (*ret_ccline_colors).colors.size.wrapping_add(1);
        *(*ret_ccline_colors)
            .colors
            .items
            .offset(c2rust_fresh14 as isize) = CmdlineColorChunk {
            start: prev_end as ::core::ffi::c_int,
            end: (*colored_ccline).cmdlen,
            hl_id: 0 as ::core::ffi::c_int,
        };
    }
    if colors.items != &raw mut colors.init_array as *mut ParserHighlightChunk {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut colors.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
    }
}
unsafe extern "C" fn color_cmdline(mut colored_ccline: *mut CmdlineInfo) -> bool {
    let mut cbcall_ret: bool = false;
    let mut prev_end: varnumber_T = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut printed_errmsg: bool = false_0 != 0;
    let mut ret: bool = true_0 != 0;
    let mut ccline_colors: *mut ColoredCmdline = &raw mut (*colored_ccline).last_colors;
    if (*ccline_colors).prompt_id == (*colored_ccline).prompt_id
        && !(*ccline_colors).cmdbuff.is_null()
        && strcmp((*ccline_colors).cmdbuff, (*colored_ccline).cmdbuff) == 0 as ::core::ffi::c_int
    {
        return ret;
    }
    (*ccline_colors).colors.size = 0 as size_t;
    if (*colored_ccline).cmdbuff.is_null()
        || *(*colored_ccline).cmdbuff as ::core::ffi::c_int == NUL
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*ccline_colors).cmdbuff as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        return ret;
    }
    let mut arg_allocated: bool = false_0 != 0;
    let mut arg: typval_T = typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union {
            v_string: (*colored_ccline).cmdbuff,
        },
    };
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    static mut prev_prompt_errors: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut color_cb: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut can_free_cb: bool = false_0 != 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut err_errmsg: *const ::core::ffi::c_char =
        &raw const e_intern2 as *const ::core::ffi::c_char;
    let mut dgc_ret: bool = true_0 != 0;
    '_color_cmdline_end: {
        if (*colored_ccline).prompt_id != prev_prompt_id {
            prev_prompt_errors = 0 as ::core::ffi::c_int;
            prev_prompt_id = (*colored_ccline).prompt_id;
        } else if prev_prompt_errors >= MAX_CB_ERRORS as ::core::ffi::c_int {
            break '_color_cmdline_end;
        }
        if (*colored_ccline).highlight_callback.type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            '_c2rust_label: {
                if (*colored_ccline).input_fn != 0 {
                } else {
                    __assert_fail(
                        b"colored_ccline->input_fn\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        3408 as ::core::ffi::c_uint,
                        b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            color_cb = (*colored_ccline).highlight_callback;
        } else if (*colored_ccline).cmdfirstc == ':' as ::core::ffi::c_int {
            let mut tstate: TryState = TryState {
                current_exception: ::core::ptr::null_mut::<except_T>(),
                private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                msg_list: ::core::ptr::null::<*const msglist_T>(),
                got_int: 0,
                did_throw: false,
                need_rethrow: 0,
                did_emsg: 0,
            };
            try_enter(&raw mut tstate);
            err_errmsg = b"E5408: Unable to get g:Nvim_color_cmdline callback: %s\0".as_ptr()
                as *const ::core::ffi::c_char;
            dgc_ret = tv_dict_get_callback(
                get_globvar_dict(),
                b"Nvim_color_cmdline\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
                &raw mut color_cb,
            );
            try_leave(&raw mut tstate, &raw mut err);
            can_free_cb = true_0 != 0;
        } else if (*colored_ccline).cmdfirstc == '=' as ::core::ffi::c_int {
            color_expr_cmdline(colored_ccline, ccline_colors);
        }
        '_color_cmdline_error: {
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                || !dgc_ret)
            {
                if color_cb.type_0 as ::core::ffi::c_uint
                    == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    break '_color_cmdline_end;
                } else {
                    if *(*colored_ccline)
                        .cmdbuff
                        .offset((*colored_ccline).cmdlen as isize)
                        as ::core::ffi::c_int
                        != NUL
                    {
                        arg_allocated = true_0 != 0;
                        arg.vval.v_string = xmemdupz(
                            (*colored_ccline).cmdbuff as *const ::core::ffi::c_void,
                            (*colored_ccline).cmdlen as size_t,
                        ) as *mut ::core::ffi::c_char;
                    }
                    getln_interrupted_highlight = false_0 != 0;
                    cbcall_ret = true_0 != 0;
                    let mut tstate_0: TryState = TryState {
                        current_exception: ::core::ptr::null_mut::<except_T>(),
                        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                        msg_list: ::core::ptr::null::<*const msglist_T>(),
                        got_int: 0,
                        did_throw: false,
                        need_rethrow: 0,
                        did_emsg: 0,
                    };
                    try_enter(&raw mut tstate_0);
                    err_errmsg = b"E5407: Callback has thrown an exception: %s\0".as_ptr()
                        as *const ::core::ffi::c_char;
                    let saved_msg_col: ::core::ffi::c_int = msg_col;
                    msg_silent += 1;
                    cbcall_ret = callback_call(
                        &raw mut color_cb,
                        1 as ::core::ffi::c_int,
                        &raw mut arg,
                        &raw mut tv,
                    );
                    msg_silent -= 1;
                    msg_col = saved_msg_col;
                    if got_int {
                        getln_interrupted_highlight = true;
                    }
                    try_leave(&raw mut tstate_0, &raw mut err);
                    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        || !cbcall_ret)
                    {
                        if tv.v_type as ::core::ffi::c_uint
                            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            msg_scroll = true_0;
                            msg_putchar('\n' as ::core::ffi::c_int);
                            smsg(
                                HLF_E as ::core::ffi::c_int,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                gettext(b"E5400: Callback should return list\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                            );
                            printed_errmsg = true_0 != 0;
                        } else if tv.vval.v_list.is_null() {
                            break '_color_cmdline_end;
                        } else {
                            prev_end = 0 as varnumber_T;
                            i = 0 as ::core::ffi::c_int;
                            let l_: *const list_T = tv.vval.v_list;
                            's_561: {
                                if !l_.is_null() {
                                    let mut li: *const listitem_T = (*l_).lv_first;
                                    loop {
                                        if li.is_null() {
                                            break 's_561;
                                        }
                                        if (*li).li_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            msg_scroll = 1 as ::core::ffi::c_int;
                                            msg_putchar('\n' as ::core::ffi::c_int);
                                            smsg(
                                                HLF_E as ::core::ffi::c_int,
                                                gettext(
                                                    b"E5401: List item %i is not a List\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                i,
                                            );
                                            printed_errmsg = true;
                                            break '_color_cmdline_error;
                                        } else {
                                            let l: *const list_T = (*li).li_tv.vval.v_list;
                                            if tv_list_len(l) != 3 as ::core::ffi::c_int {
                                                msg_scroll = 1 as ::core::ffi::c_int;
                                                msg_putchar('\n' as ::core::ffi::c_int);
                                                smsg(
                                                    HLF_E as ::core::ffi::c_int,
                                                    gettext(
                                                        b"E5402: List item %i has incorrect length: %d /= 3\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    ),
                                                    i,
                                                    tv_list_len(l),
                                                );
                                                printed_errmsg = true;
                                                break '_color_cmdline_error;
                                            } else {
                                                let mut error: bool = false;
                                                let start: varnumber_T = tv_get_number_chk(
                                                    &raw mut (*(tv_list_first
                                                        as unsafe extern "C" fn(
                                                            *const list_T,
                                                        ) -> *mut listitem_T)(l))
                                                        .li_tv,
                                                    &raw mut error,
                                                );
                                                if error {
                                                    break '_color_cmdline_error;
                                                }
                                                if !(prev_end <= start
                                                    && start
                                                        < (*colored_ccline).cmdlen as varnumber_T)
                                                {
                                                    msg_scroll = 1 as ::core::ffi::c_int;
                                                    msg_putchar('\n' as ::core::ffi::c_int);
                                                    smsg(
                                                        HLF_E as ::core::ffi::c_int,
                                                        gettext(
                                                            b"E5403: Chunk %i start %ld not in range [%ld, %i)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        i,
                                                        start,
                                                        prev_end,
                                                        (*colored_ccline).cmdlen,
                                                    );
                                                    printed_errmsg = true;
                                                    break '_color_cmdline_error;
                                                } else if utf8len_tab_zero[*(*colored_ccline)
                                                    .cmdbuff
                                                    .offset(start as isize)
                                                    as uint8_t
                                                    as usize]
                                                    as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                                {
                                                    msg_scroll = 1 as ::core::ffi::c_int;
                                                    msg_putchar('\n' as ::core::ffi::c_int);
                                                    smsg(
                                                        HLF_E as ::core::ffi::c_int,
                                                        gettext(
                                                            b"E5405: Chunk %i start %ld splits multibyte character\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        ),
                                                        i,
                                                        start,
                                                    );
                                                    printed_errmsg = true;
                                                    break '_color_cmdline_error;
                                                } else {
                                                    if start != prev_end {
                                                        if (*ccline_colors).colors.size
                                                            == (*ccline_colors).colors.capacity
                                                        {
                                                            (*ccline_colors).colors.capacity =
                                                                if (*ccline_colors).colors.capacity
                                                                    != 0
                                                                {
                                                                    (*ccline_colors).colors.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                } else {
                                                                    8 as size_t
                                                                };
                                                            (*ccline_colors).colors.items = xrealloc(
                                                                (*ccline_colors).colors.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ::core::mem::size_of::<
                                                                    CmdlineColorChunk,
                                                                >(
                                                                )
                                                                .wrapping_mul(
                                                                    (*ccline_colors)
                                                                        .colors
                                                                        .capacity,
                                                                ),
                                                            )
                                                                as *mut CmdlineColorChunk;
                                                        } else {
                                                        };
                                                        let c2rust_fresh9 =
                                                            (*ccline_colors).colors.size;
                                                        (*ccline_colors).colors.size =
                                                            (*ccline_colors)
                                                                .colors
                                                                .size
                                                                .wrapping_add(1);
                                                        *(*ccline_colors)
                                                            .colors
                                                            .items
                                                            .offset(c2rust_fresh9 as isize) =
                                                            CmdlineColorChunk {
                                                                start: prev_end
                                                                    as ::core::ffi::c_int,
                                                                end: start as ::core::ffi::c_int,
                                                                hl_id: 0 as ::core::ffi::c_int,
                                                            };
                                                    }
                                                    let end: varnumber_T = tv_get_number_chk(
                                                        &raw mut (*(*(tv_list_first
                                                            as unsafe extern "C" fn(
                                                                *const list_T,
                                                            ) -> *mut listitem_T)(l))
                                                            .li_next)
                                                            .li_tv,
                                                        &raw mut error,
                                                    );
                                                    if error {
                                                        break '_color_cmdline_error;
                                                    }
                                                    if !(start < end
                                                        && end
                                                            <= (*colored_ccline).cmdlen
                                                                as varnumber_T)
                                                    {
                                                        msg_scroll = 1 as ::core::ffi::c_int;
                                                        msg_putchar('\n' as ::core::ffi::c_int);
                                                        smsg(
                                                            HLF_E as ::core::ffi::c_int,
                                                            gettext(
                                                                b"E5404: Chunk %i end %ld not in range (%ld, %i]\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            i,
                                                            end,
                                                            start,
                                                            (*colored_ccline).cmdlen,
                                                        );
                                                        printed_errmsg = true;
                                                        break '_color_cmdline_error;
                                                    } else if end
                                                        < (*colored_ccline).cmdlen as varnumber_T
                                                        && utf8len_tab_zero[*(*colored_ccline)
                                                            .cmdbuff
                                                            .offset(end as isize)
                                                            as uint8_t
                                                            as usize]
                                                            as ::core::ffi::c_int
                                                            == 0 as ::core::ffi::c_int
                                                    {
                                                        msg_scroll = 1 as ::core::ffi::c_int;
                                                        msg_putchar('\n' as ::core::ffi::c_int);
                                                        smsg(
                                                            HLF_E as ::core::ffi::c_int,
                                                            gettext(
                                                                b"E5406: Chunk %i end %ld splits multibyte character\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            i,
                                                            end,
                                                        );
                                                        printed_errmsg = true;
                                                        break '_color_cmdline_error;
                                                    } else {
                                                        prev_end = end;
                                                        let group: *const ::core::ffi::c_char = tv_get_string_chk(
                                                            &raw mut (*(tv_list_last
                                                                as unsafe extern "C" fn(
                                                                    *const list_T,
                                                                ) -> *mut listitem_T)(l))
                                                                .li_tv,
                                                        );
                                                        if group.is_null() {
                                                            break '_color_cmdline_error;
                                                        }
                                                        if (*ccline_colors).colors.size
                                                            == (*ccline_colors).colors.capacity
                                                        {
                                                            (*ccline_colors).colors.capacity =
                                                                if (*ccline_colors).colors.capacity
                                                                    != 0
                                                                {
                                                                    (*ccline_colors).colors.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                } else {
                                                                    8 as size_t
                                                                };
                                                            (*ccline_colors).colors.items = xrealloc(
                                                                (*ccline_colors).colors.items
                                                                    as *mut ::core::ffi::c_void,
                                                                ::core::mem::size_of::<
                                                                    CmdlineColorChunk,
                                                                >(
                                                                )
                                                                .wrapping_mul(
                                                                    (*ccline_colors)
                                                                        .colors
                                                                        .capacity,
                                                                ),
                                                            )
                                                                as *mut CmdlineColorChunk;
                                                        } else {
                                                        };
                                                        let c2rust_fresh10 =
                                                            (*ccline_colors).colors.size;
                                                        (*ccline_colors).colors.size =
                                                            (*ccline_colors)
                                                                .colors
                                                                .size
                                                                .wrapping_add(1);
                                                        *(*ccline_colors)
                                                            .colors
                                                            .items
                                                            .offset(c2rust_fresh10 as isize) =
                                                            CmdlineColorChunk {
                                                                start: start as ::core::ffi::c_int,
                                                                end: end as ::core::ffi::c_int,
                                                                hl_id: syn_name2id(group),
                                                            };
                                                        i += 1;
                                                        li = (*li).li_next;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if prev_end < (*colored_ccline).cmdlen as varnumber_T {
                                if (*ccline_colors).colors.size == (*ccline_colors).colors.capacity
                                {
                                    (*ccline_colors).colors.capacity =
                                        if (*ccline_colors).colors.capacity != 0 {
                                            (*ccline_colors).colors.capacity
                                                << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                    (*ccline_colors).colors.items = xrealloc(
                                        (*ccline_colors).colors.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<CmdlineColorChunk>()
                                            .wrapping_mul((*ccline_colors).colors.capacity),
                                    )
                                        as *mut CmdlineColorChunk;
                                } else {
                                };
                                let c2rust_fresh11 = (*ccline_colors).colors.size;
                                (*ccline_colors).colors.size =
                                    (*ccline_colors).colors.size.wrapping_add(1);
                                *(*ccline_colors)
                                    .colors
                                    .items
                                    .offset(c2rust_fresh11 as isize) = CmdlineColorChunk {
                                    start: prev_end as ::core::ffi::c_int,
                                    end: (*colored_ccline).cmdlen,
                                    hl_id: 0 as ::core::ffi::c_int,
                                };
                            }
                            prev_prompt_errors = 0 as ::core::ffi::c_int;
                            break '_color_cmdline_end;
                        }
                    }
                }
            }
        }
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            msg_scroll = true_0;
            msg_putchar('\n' as ::core::ffi::c_int);
            smsg(HLF_E as ::core::ffi::c_int, gettext(err_errmsg), err.msg);
            printed_errmsg = true_0 != 0;
            api_clear_error(&raw mut err);
        }
        '_c2rust_label_1: {
            if printed_errmsg {
            } else {
                __assert_fail(
                    b"printed_errmsg\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    3557 as ::core::ffi::c_uint,
                    b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        prev_prompt_errors += 1;
        (*ccline_colors).colors.size = 0 as size_t;
        redrawcmdline();
        ret = false_0 != 0;
    }
    '_c2rust_label_0: {
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        } else {
            __assert_fail(
                b"!ERROR_SET(&err)\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                3538 as ::core::ffi::c_uint,
                b"_Bool color_cmdline(CmdlineInfo *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if can_free_cb {
        callback_free(&raw mut color_cb);
    }
    xfree((*ccline_colors).cmdbuff as *mut ::core::ffi::c_void);
    (*ccline_colors).prompt_id = (*colored_ccline).prompt_id;
    if arg_allocated {
        (*ccline_colors).cmdbuff = arg.vval.v_string;
    } else {
        (*ccline_colors).cmdbuff = xmemdupz(
            (*colored_ccline).cmdbuff as *const ::core::ffi::c_void,
            (*colored_ccline).cmdlen as size_t,
        ) as *mut ::core::ffi::c_char;
    }
    tv_clear(&raw mut tv);
    return ret;
}
unsafe extern "C" fn draw_cmdline(mut start: ::core::ffi::c_int, mut len: ::core::ffi::c_int) {
    if ccline.cmdbuff.is_null() || !color_cmdline(&raw mut ccline) {
        return;
    }
    if ui_has(kUICmdline) {
        ccline.special_char = NUL as ::core::ffi::c_char;
        ccline.redraw_state = kCmdRedrawAll;
        return;
    }
    if cmdline_star > 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len {
            msg_putchar('*' as ::core::ffi::c_int);
            i += utfc_ptr2len(ccline.cmdbuff.offset(start as isize).offset(i as isize))
                - 1 as ::core::ffi::c_int;
            i += 1;
        }
    } else if ccline.last_colors.colors.size != 0 {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < ccline.last_colors.colors.size {
            let mut chunk: CmdlineColorChunk =
                *ccline.last_colors.colors.items.offset(i_0 as isize);
            if chunk.end > start {
                let chunk_start: ::core::ffi::c_int = if chunk.start > start {
                    chunk.start
                } else {
                    start
                };
                msg_outtrans_len(
                    ccline.cmdbuff.offset(chunk_start as isize),
                    chunk.end - chunk_start,
                    chunk.hl_id,
                    false_0 != 0,
                );
            }
            i_0 = i_0.wrapping_add(1);
        }
    } else {
        msg_outtrans_len(
            ccline.cmdbuff.offset(start as isize),
            len,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    };
}
unsafe extern "C" fn ui_ext_cmdline_show(mut line: *mut CmdlineInfo) {
    let mut arena: Arena = ARENA_EMPTY;
    let mut content: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if cmdline_star != 0 {
        content = arena_array(&raw mut arena, 1 as size_t);
        let mut len: size_t = 0 as size_t;
        let mut p: *mut ::core::ffi::c_char = ccline.cmdbuff;
        while *p != 0 {
            len = len.wrapping_add(1);
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        let mut buf: *mut ::core::ffi::c_char =
            arena_alloc(&raw mut arena, len, false_0 != 0) as *mut ::core::ffi::c_char;
        memset(
            buf as *mut ::core::ffi::c_void,
            '*' as ::core::ffi::c_int,
            len,
        );
        let mut item: Array = arena_array(&raw mut arena, 3 as size_t);
        let c2rust_fresh17 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh17 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh18 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh18 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: buf,
                    size: len,
                },
            },
        };
        let c2rust_fresh19 = item.size;
        item.size = item.size.wrapping_add(1);
        *item.items.offset(c2rust_fresh19 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh20 = content.size;
        content.size = content.size.wrapping_add(1);
        *content.items.offset(c2rust_fresh20 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: item },
        };
    } else if (*line).last_colors.colors.size != 0 {
        content = arena_array(&raw mut arena, (*line).last_colors.colors.size);
        let mut i: size_t = 0 as size_t;
        while i < (*line).last_colors.colors.size {
            let mut chunk: CmdlineColorChunk = *(*line).last_colors.colors.items.offset(i as isize);
            let mut item_0: Array = arena_array(&raw mut arena, 3 as size_t);
            let c2rust_fresh21 = item_0.size;
            item_0.size = item_0.size.wrapping_add(1);
            *item_0.items.offset(c2rust_fresh21 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if chunk.hl_id == 0 as ::core::ffi::c_int {
                        0 as ::core::ffi::c_int
                    } else {
                        syn_id2attr(chunk.hl_id)
                    }) as Integer,
                },
            };
            '_c2rust_label: {
                if chunk.end >= chunk.start {
                } else {
                    __assert_fail(
                        b"chunk.end >= chunk.start\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/ex_getln.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        3627 as ::core::ffi::c_uint,
                        b"void ui_ext_cmdline_show(CmdlineInfo *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            let c2rust_fresh22 = item_0.size;
            item_0.size = item_0.size.wrapping_add(1);
            *item_0.items.offset(c2rust_fresh22 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: String_0 {
                        data: (*line).cmdbuff.offset(chunk.start as isize),
                        size: (chunk.end - chunk.start) as size_t,
                    },
                },
            };
            let c2rust_fresh23 = item_0.size;
            item_0.size = item_0.size.wrapping_add(1);
            *item_0.items.offset(c2rust_fresh23 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: chunk.hl_id as Integer,
                },
            };
            let c2rust_fresh24 = content.size;
            content.size = content.size.wrapping_add(1);
            *content.items.offset(c2rust_fresh24 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: item_0 },
            };
            i = i.wrapping_add(1);
        }
    } else {
        let mut item_1: Array = arena_array(&raw mut arena, 3 as size_t);
        let c2rust_fresh25 = item_1.size;
        item_1.size = item_1.size.wrapping_add(1);
        *item_1.items.offset(c2rust_fresh25 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh26 = item_1.size;
        item_1.size = item_1.size.wrapping_add(1);
        *item_1.items.offset(c2rust_fresh26 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string((*line).cmdbuff),
            },
        };
        let c2rust_fresh27 = item_1.size;
        item_1.size = item_1.size.wrapping_add(1);
        *item_1.items.offset(c2rust_fresh27 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        content = arena_array(&raw mut arena, 1 as size_t);
        let c2rust_fresh28 = content.size;
        content.size = content.size.wrapping_add(1);
        *content.items.offset(c2rust_fresh28 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: item_1 },
        };
    }
    let mut charbuf: [::core::ffi::c_char; 2] = [
        (*line).cmdfirstc as ::core::ffi::c_char,
        0 as ::core::ffi::c_char,
    ];
    ui_call_cmdline_show(
        content,
        (*line).cmdpos as Integer,
        cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
        cstr_as_string((*line).cmdprompt),
        (*line).cmdindent as Integer,
        (*line).level as Integer,
        (*line).hl_id as Integer,
    );
    if (*line).special_char != 0 {
        charbuf[0 as ::core::ffi::c_int as usize] = (*line).special_char;
        ui_call_cmdline_special_char(
            cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
            (*line).special_shift as Boolean,
            (*line).level as Integer,
        );
    }
    arena_mem_free(arena_finish(&raw mut arena));
}
#[no_mangle]
pub unsafe extern "C" fn ui_ext_cmdline_block_append(
    mut indent: size_t,
    mut line: *const ::core::ffi::c_char,
) {
    let mut buf: *mut ::core::ffi::c_char =
        xmallocz(indent.wrapping_add(strlen(line))) as *mut ::core::ffi::c_char;
    memset(
        buf as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        indent,
    );
    memcpy(
        buf.offset(indent as isize) as *mut ::core::ffi::c_void,
        line as *const ::core::ffi::c_void,
        strlen(line),
    );
    let mut item: Array = ARRAY_DICT_INIT;
    if item.size == item.capacity {
        item.capacity = if item.capacity != 0 {
            item.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        item.items = xrealloc(
            item.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh2 = item.size;
    item.size = item.size.wrapping_add(1);
    *item.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: 0 as Integer,
        },
    };
    if item.size == item.capacity {
        item.capacity = if item.capacity != 0 {
            item.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        item.items = xrealloc(
            item.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh3 = item.size;
    item.size = item.size.wrapping_add(1);
    *item.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(buf),
        },
    };
    if item.size == item.capacity {
        item.capacity = if item.capacity != 0 {
            item.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        item.items = xrealloc(
            item.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(item.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh4 = item.size;
    item.size = item.size.wrapping_add(1);
    *item.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: 0 as Integer,
        },
    };
    let mut content: Array = ARRAY_DICT_INIT;
    if content.size == content.capacity {
        content.capacity = if content.capacity != 0 {
            content.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        content.items = xrealloc(
            content.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh5 = content.size;
    content.size = content.size.wrapping_add(1);
    *content.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed { array: item },
    };
    if cmdline_block.size == cmdline_block.capacity {
        cmdline_block.capacity = if cmdline_block.capacity != 0 {
            cmdline_block.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        cmdline_block.items = xrealloc(
            cmdline_block.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(cmdline_block.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh6 = cmdline_block.size;
    cmdline_block.size = cmdline_block.size.wrapping_add(1);
    *cmdline_block.items.offset(c2rust_fresh6 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed { array: content },
    };
    if cmdline_block.size > 1 as size_t {
        ui_call_cmdline_block_append(content);
    } else {
        ui_call_cmdline_block_show(cmdline_block);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_ext_cmdline_block_leave() {
    api_free_array(cmdline_block);
    cmdline_block = ARRAY_DICT_INIT;
    ui_call_cmdline_block_hide();
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_screen_cleared() {
    if !ui_has(kUICmdline) {
        return;
    }
    if cmdline_block.size != 0 {
        ui_call_cmdline_block_show(cmdline_block);
    }
    let mut prev_level: ::core::ffi::c_int = ccline.level - 1 as ::core::ffi::c_int;
    let mut line: *mut CmdlineInfo = ccline.prev_ccline;
    while prev_level > 0 as ::core::ffi::c_int && !line.is_null() {
        if (*line).level == prev_level {
            if prev_level != cmdwin_level {
                (*line).redraw_state = kCmdRedrawAll;
            }
            prev_level -= 1;
        }
        line = (*line).prev_ccline;
    }
    redrawcmd();
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_ui_flush() {
    if !ui_has(kUICmdline) {
        return;
    }
    let mut level: ::core::ffi::c_int = ccline.level;
    let mut line: *mut CmdlineInfo = &raw mut ccline;
    while level > 0 as ::core::ffi::c_int && !line.is_null() {
        if (*line).level == level {
            let mut redraw_state: CmdRedraw = (*line).redraw_state;
            (*line).redraw_state = kCmdRedrawNone;
            if redraw_state as ::core::ffi::c_uint
                == kCmdRedrawAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                cmdline_was_last_drawn = true_0 != 0;
                ui_ext_cmdline_show(line);
            } else if redraw_state as ::core::ffi::c_uint
                == kCmdRedrawPos as ::core::ffi::c_int as ::core::ffi::c_uint
                && cmdline_was_last_drawn as ::core::ffi::c_int != 0
            {
                ui_call_cmdline_pos((*line).cmdpos as Integer, (*line).level as Integer);
            }
            level -= 1;
        }
        line = (*line).prev_ccline;
    }
}
#[no_mangle]
pub unsafe extern "C" fn putcmdline(mut c: ::core::ffi::c_char, mut shift: bool) {
    if cmd_silent {
        return;
    }
    if !ui_has(kUICmdline) {
        msg_no_more = true_0 != 0;
        msg_putchar(c as ::core::ffi::c_int);
        if shift {
            draw_cmdline(ccline.cmdpos, ccline.cmdlen - ccline.cmdpos);
        }
        msg_no_more = false_0 != 0;
    } else if ccline.redraw_state as ::core::ffi::c_uint
        != kCmdRedrawAll as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut charbuf: [::core::ffi::c_char; 2] = [c, 0 as ::core::ffi::c_char];
        ui_call_cmdline_special_char(
            cstr_as_string(&raw mut charbuf as *mut ::core::ffi::c_char),
            shift as Boolean,
            ccline.level as Integer,
        );
    }
    cursorcmd();
    ccline.special_char = c;
    ccline.special_shift = shift;
    ui_cursor_shape();
}
#[no_mangle]
pub unsafe extern "C" fn unputcmdline() {
    if cmd_silent {
        return;
    }
    msg_no_more = true_0 != 0;
    if ccline.cmdlen == ccline.cmdpos && !ui_has(kUICmdline) {
        msg_putchar(' ' as ::core::ffi::c_int);
    } else {
        draw_cmdline(
            ccline.cmdpos,
            utfc_ptr2len(ccline.cmdbuff.offset(ccline.cmdpos as isize)),
        );
    }
    msg_no_more = false_0 != 0;
    cursorcmd();
    ccline.special_char = NUL as ::core::ffi::c_char;
    ui_cursor_shape();
}
#[no_mangle]
pub unsafe extern "C" fn put_on_cmdline(
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut redraw: bool,
) {
    if len < 0 as ::core::ffi::c_int {
        len = strlen(str) as ::core::ffi::c_int;
    }
    realloc_cmdbuff(ccline.cmdlen + len + 1 as ::core::ffi::c_int);
    if ccline.overstrike == 0 {
        memmove(
            ccline
                .cmdbuff
                .offset(ccline.cmdpos as isize)
                .offset(len as isize) as *mut ::core::ffi::c_void,
            ccline.cmdbuff.offset(ccline.cmdpos as isize) as *const ::core::ffi::c_void,
            (ccline.cmdlen - ccline.cmdpos) as size_t,
        );
        ccline.cmdlen += len;
    } else {
        let mut m: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < len {
            m += 1;
            i += utfc_ptr2len(str.offset(i as isize));
        }
        i = ccline.cmdpos;
        while i < ccline.cmdlen && m > 0 as ::core::ffi::c_int {
            m -= 1;
            i += utfc_ptr2len(ccline.cmdbuff.offset(i as isize));
        }
        if i < ccline.cmdlen {
            memmove(
                ccline
                    .cmdbuff
                    .offset(ccline.cmdpos as isize)
                    .offset(len as isize) as *mut ::core::ffi::c_void,
                ccline.cmdbuff.offset(i as isize) as *const ::core::ffi::c_void,
                (ccline.cmdlen - i) as size_t,
            );
            ccline.cmdlen += ccline.cmdpos + len - i;
        } else {
            ccline.cmdlen = ccline.cmdpos + len;
        }
    }
    memmove(
        ccline.cmdbuff.offset(ccline.cmdpos as isize) as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len as size_t,
    );
    *ccline.cmdbuff.offset(ccline.cmdlen as isize) = NUL as ::core::ffi::c_char;
    if ccline.cmdpos > 0 as ::core::ffi::c_int
        && *ccline.cmdbuff.offset(ccline.cmdpos as isize) as uint8_t as ::core::ffi::c_int
            >= 0x80 as ::core::ffi::c_int
    {
        let mut i_0: ::core::ffi::c_int = utf_head_off(
            ccline.cmdbuff,
            ccline.cmdbuff.offset(ccline.cmdpos as isize),
        );
        if i_0 != 0 as ::core::ffi::c_int {
            ccline.cmdpos -= i_0;
            len += i_0;
            ccline.cmdspos = cmd_screencol(ccline.cmdpos);
        }
    }
    if redraw as ::core::ffi::c_int != 0 && !cmd_silent {
        msg_no_more = true_0 != 0;
        let mut i_1: ::core::ffi::c_int = cmdline_row;
        cursorcmd();
        draw_cmdline(ccline.cmdpos, ccline.cmdlen - ccline.cmdpos);
        if cmdline_row != i_1 || ccline.overstrike != 0 {
            msg_clr_eos();
        }
        msg_no_more = false_0 != 0;
    }
    let mut m_0: ::core::ffi::c_int = 0;
    if KeyTyped {
        m_0 = Columns * Rows;
        if m_0 < 0 as ::core::ffi::c_int {
            m_0 = MAXCOL as ::core::ffi::c_int;
        }
    } else {
        m_0 = MAXCOL as ::core::ffi::c_int;
    }
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < len {
        let mut c: ::core::ffi::c_int = cmdline_charsize(ccline.cmdpos);
        correct_screencol(ccline.cmdpos, c, &raw mut ccline.cmdspos);
        if ccline.cmdspos + c < m_0 {
            ccline.cmdspos += c;
        }
        c = utfc_ptr2len(ccline.cmdbuff.offset(ccline.cmdpos as isize)) - 1 as ::core::ffi::c_int;
        c = if c < len - i_2 - 1 as ::core::ffi::c_int {
            c
        } else {
            len - i_2 - 1 as ::core::ffi::c_int
        };
        ccline.cmdpos += c;
        i_2 += c;
        ccline.cmdpos += 1;
        i_2 += 1;
    }
    if redraw {
        msg_check();
    }
}
unsafe extern "C" fn save_cmdline(mut ccp: *mut CmdlineInfo) {
    *ccp = ccline;
    memset(
        &raw mut ccline as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CmdlineInfo>(),
    );
    ccline.prev_ccline = ccp;
    ccline.cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn restore_cmdline(mut ccp: *mut CmdlineInfo) {
    ccline = *ccp;
}
unsafe extern "C" fn cmdline_paste(
    mut regname: ::core::ffi::c_int,
    mut literally: bool,
    mut remcr: bool,
) -> bool {
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut allocated: bool = false;
    if regname != Ctrl_F
        && regname != Ctrl_P
        && regname != Ctrl_W
        && regname != Ctrl_A
        && regname != Ctrl_L
        && !valid_yank_reg(regname, false_0 != 0)
    {
        return FAIL != 0;
    }
    line_breakcheck();
    if got_int {
        return FAIL != 0;
    }
    textlock += 1;
    let i: bool = get_spec_reg(regname, &raw mut arg, &raw mut allocated, true_0 != 0);
    textlock -= 1;
    if i {
        if arg.is_null() {
            return FAIL != 0;
        }
        let mut p: *mut ::core::ffi::c_char = arg;
        if p_is != 0 && regname == Ctrl_W {
            let mut w: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut len: ::core::ffi::c_int = 0;
            w = ccline.cmdbuff.offset(ccline.cmdpos as isize);
            while w > ccline.cmdbuff {
                len = utf_head_off(
                    ccline.cmdbuff,
                    w.offset(-(1 as ::core::ffi::c_int as isize)),
                ) + 1 as ::core::ffi::c_int;
                if !vim_iswordc(utf_ptr2char(w.offset(-(len as isize)))) {
                    break;
                }
                w = w.offset(-(len as isize));
            }
            len =
                ccline.cmdbuff.offset(ccline.cmdpos as isize).offset_from(w) as ::core::ffi::c_int;
            if if p_ic != 0 {
                (strncasecmp(w, arg, len as size_t) == 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int
            } else {
                (strncmp(w, arg, len as size_t) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } != 0
            {
                p = p.offset(len as isize);
            }
        }
        cmdline_paste_str(p, literally);
        if allocated {
            xfree(arg as *mut ::core::ffi::c_void);
        }
        return OK != 0;
    }
    return cmdline_paste_reg(regname, literally, remcr);
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_paste_str(mut s: *const ::core::ffi::c_char, mut literally: bool) {
    if literally {
        put_on_cmdline(s, -1 as ::core::ffi::c_int, true_0 != 0);
    } else {
        while *s as ::core::ffi::c_int != NUL {
            let mut cv: ::core::ffi::c_int = *s as uint8_t as ::core::ffi::c_int;
            if cv == Ctrl_V
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            {
                s = s.offset(1);
            }
            let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
            if cv == Ctrl_V
                || c == ESC
                || c == Ctrl_C
                || c == CAR
                || c == NL
                || c == Ctrl_L
                || c == Ctrl_BSL && *s as ::core::ffi::c_int == Ctrl_N
            {
                stuffcharReadbuff(Ctrl_V);
            }
            stuffcharReadbuff(c);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn redrawcmdline() {
    if cmd_silent {
        return;
    }
    need_wait_return = false_0 != 0;
    compute_cmdrow();
    redrawcmd();
    cursorcmd();
    ui_cursor_shape();
}
unsafe extern "C" fn redrawcmdprompt() {
    if cmd_silent {
        return;
    }
    if ui_has(kUICmdline) {
        ccline.redraw_state = kCmdRedrawAll;
        return;
    }
    if ccline.cmdfirstc != NUL {
        msg_putchar(ccline.cmdfirstc);
    }
    if !ccline.cmdprompt.is_null() {
        msg_puts_hl(ccline.cmdprompt, ccline.hl_id, false_0 != 0);
        ccline.cmdindent = msg_col + (msg_row - cmdline_row) * Columns;
        if ccline.cmdfirstc != NUL {
            ccline.cmdindent -= 1;
        }
    } else {
        let mut i: ::core::ffi::c_int = ccline.cmdindent;
        while i > 0 as ::core::ffi::c_int {
            msg_putchar(' ' as ::core::ffi::c_int);
            i -= 1;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn redrawcmd() {
    if cmd_silent {
        return;
    }
    if ui_has(kUICmdline) {
        draw_cmdline(0 as ::core::ffi::c_int, ccline.cmdlen);
        return;
    }
    if ccline.cmdbuff.is_null() {
        msg_cursor_goto(cmdline_row, 0 as ::core::ffi::c_int);
        msg_clr_eos();
        return;
    }
    redrawing_cmdline = true_0 != 0;
    sb_text_restart_cmdline();
    msg_start();
    redrawcmdprompt();
    msg_no_more = true_0 != 0;
    draw_cmdline(0 as ::core::ffi::c_int, ccline.cmdlen);
    msg_clr_eos();
    msg_no_more = false_0 != 0;
    ccline.cmdspos = cmd_screencol(ccline.cmdpos);
    if ccline.special_char as ::core::ffi::c_int != NUL {
        putcmdline(ccline.special_char, ccline.special_shift);
    }
    msg_scroll = false_0;
    skip_redraw = false_0 != 0;
    cmdline_was_last_drawn = true_0 != 0;
    redrawing_cmdline = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn compute_cmdrow() {
    if exmode_active as ::core::ffi::c_int != 0 || msg_scrolled != 0 as ::core::ffi::c_int {
        cmdline_row = Rows - 1 as ::core::ffi::c_int;
    } else {
        let mut wp: *mut win_T = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
        cmdline_row = (*wp).w_winrow
            + (*wp).w_height
            + (*wp).w_hsep_height
            + (*wp).w_status_height
            + global_stl_height();
    }
    if cmdline_row == Rows && p_ch > 0 as OptInt {
        cmdline_row -= 1;
    }
    lines_left = cmdline_row;
}
#[no_mangle]
pub unsafe extern "C" fn cursorcmd() {
    if cmd_silent as ::core::ffi::c_int != 0 || ui_has(kUICmdline) as ::core::ffi::c_int != 0 {
        return;
    }
    msg_row = cmdline_row + ccline.cmdspos / Columns;
    msg_col = ccline.cmdspos % Columns;
    msg_row = if msg_row < Rows - 1 as ::core::ffi::c_int {
        msg_row
    } else {
        Rows - 1 as ::core::ffi::c_int
    };
    msg_cursor_goto(msg_row, msg_col);
}
#[no_mangle]
pub unsafe extern "C" fn gotocmdline(mut clr: bool) {
    if ui_has(kUICmdline) {
        return;
    }
    msg_start();
    msg_col = 0 as ::core::ffi::c_int;
    if clr {
        msg_clr_eos();
    }
    msg_cursor_goto(cmdline_row, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn ccheck_abbr(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut spos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if p_paste != 0 || no_abbr as ::core::ffi::c_int != 0 {
        return false_0;
    }
    while spos < ccline.cmdlen
        && ascii_iswhite(*ccline.cmdbuff.offset(spos as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        spos += 1;
    }
    if ccline.cmdlen - spos > 5 as ::core::ffi::c_int
        && *ccline.cmdbuff.offset(spos as isize) as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
        && *ccline
            .cmdbuff
            .offset((spos + 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == ',' as ::core::ffi::c_int
        && *ccline
            .cmdbuff
            .offset((spos + 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '\'' as ::core::ffi::c_int
    {
        spos += 5 as ::core::ffi::c_int;
    } else {
        spos = 0 as ::core::ffi::c_int;
    }
    return check_abbr(c, ccline.cmdbuff, ccline.cmdpos, spos) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_strsave_fnameescape(
    fname: *const ::core::ffi::c_char,
    what: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped(
        fname,
        if what == VSE_SHELL as ::core::ffi::c_int {
            SHELL_ESC_CHARS.as_ptr()
        } else if what == VSE_BUFFER as ::core::ffi::c_int {
            BUFFER_ESC_CHARS.as_ptr()
        } else {
            PATH_ESC_CHARS.as_ptr()
        },
    );
    if what == VSE_SHELL as ::core::ffi::c_int && csh_like_shell() != 0 {
        let mut s: *mut ::core::ffi::c_char =
            vim_strsave_escaped(p, b"!\0".as_ptr() as *const ::core::ffi::c_char);
        xfree(p as *mut ::core::ffi::c_void);
        p = s;
    }
    if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        escape_fname(&raw mut p);
    }
    return p;
}
pub const PATH_ESC_CHARS: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b" \t\n*?[{`$\\%#'\"|!<\0")
};
pub const SHELL_ESC_CHARS: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b" \t\n*?[{`$\\%#'\"|!<>();&\0")
};
pub const BUFFER_ESC_CHARS: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b" \t\n*?[`$\\%#'\"|!<\0")
};
#[no_mangle]
pub unsafe extern "C" fn escape_fname(mut pp: *mut *mut ::core::ffi::c_char) {
    let mut p: *mut ::core::ffi::c_char =
        xmalloc(strlen(*pp).wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
    *p.offset(0 as ::core::ffi::c_int as isize) = '\\' as ::core::ffi::c_char;
    strcpy(p.offset(1 as ::core::ffi::c_int as isize), *pp);
    xfree(*pp as *mut ::core::ffi::c_void);
    *pp = p;
}
#[no_mangle]
pub unsafe extern "C" fn tilde_replace(
    mut orig_pat: *mut ::core::ffi::c_char,
    mut num_files: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    if *orig_pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '~' as ::core::ffi::c_int
        && vim_ispathsep(*orig_pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_files {
            let mut p: *mut ::core::ffi::c_char =
                home_replace_save(::core::ptr::null_mut::<buf_T>(), *files.offset(i as isize));
            xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
            *files.offset(i as isize) = p;
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_cmdline_info() -> *mut CmdlineInfo {
    return &raw mut ccline;
}
#[no_mangle]
pub unsafe extern "C" fn get_cmdline_last_prompt_id() -> ::core::ffi::c_uint {
    return last_prompt_id;
}
unsafe extern "C" fn get_ccline_ptr() -> *mut CmdlineInfo {
    if State & MODE_CMDLINE as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<CmdlineInfo>();
    } else if !ccline.cmdbuff.is_null() {
        return &raw mut ccline;
    } else if !ccline.prev_ccline.is_null() && !(*ccline.prev_ccline).cmdbuff.is_null() {
        return ccline.prev_ccline;
    } else {
        return ::core::ptr::null_mut::<CmdlineInfo>();
    };
}
unsafe extern "C" fn get_cmdline_type() -> ::core::ffi::c_int {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() {
        return NUL;
    }
    if (*p).cmdfirstc == NUL {
        return if (*p).input_fn != 0 {
            '@' as ::core::ffi::c_int
        } else {
            '-' as ::core::ffi::c_int
        };
    }
    return (*p).cmdfirstc;
}
unsafe extern "C" fn get_cmdline_str() -> *mut ::core::ffi::c_char {
    if cmdline_star > 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrnsave((*p).cmdbuff, (*p).cmdlen as size_t);
}
unsafe extern "C" fn get_cmdline_completion_pattern() -> *mut ::core::ffi::c_char {
    if cmdline_star > 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() || (*p).xpc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut xp_context: ::core::ffi::c_int = (*(*p).xpc).xp_context;
    if xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
        set_expand_context((*p).xpc);
        xp_context = (*(*p).xpc).xp_context;
        (*(*p).xpc).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    if xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut compl_pat: *mut ::core::ffi::c_char = (*(*p).xpc).xp_pattern;
    if compl_pat.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup(compl_pat);
}
unsafe extern "C" fn get_cmdline_completion() -> *mut ::core::ffi::c_char {
    if cmdline_star > 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() || (*p).xpc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut xp_context: ::core::ffi::c_int = (*(*p).xpc).xp_context;
    if xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
        set_expand_context((*p).xpc);
        xp_context = (*(*p).xpc).xp_context;
        (*(*p).xpc).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    if xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return cmdcomplete_type_to_str(xp_context, (*(*p).xpc).xp_arg);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdcomplpat(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = get_cmdline_completion_pattern();
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdcompltype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = get_cmdline_completion();
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdline(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = get_cmdline_str();
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    (*rettv).vval.v_number = (if !p.is_null() {
        (*p).cmdpos + 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdprompt(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = if !p.is_null() && !(*p).cmdprompt.is_null() {
        xstrdup((*p).cmdprompt)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdscreenpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    (*rettv).vval.v_number = (if !p.is_null() {
        (*p).cmdspos + 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdtype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = xmallocz(1 as size_t) as *mut ::core::ffi::c_char;
    *(*rettv)
        .vval
        .v_string
        .offset(0 as ::core::ffi::c_int as isize) = get_cmdline_type() as ::core::ffi::c_char;
}
unsafe extern "C" fn set_cmdline_str(
    mut str: *const ::core::ffi::c_char,
    mut pos: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    let mut len: ::core::ffi::c_int = strlen(str) as ::core::ffi::c_int;
    realloc_cmdbuff(len + 1 as ::core::ffi::c_int);
    (*p).cmdlen = len;
    strcpy((*p).cmdbuff, str as *mut ::core::ffi::c_char);
    (*p).cmdpos = if pos < 0 as ::core::ffi::c_int || pos > (*p).cmdlen {
        (*p).cmdlen
    } else {
        pos
    };
    new_cmdpos = (*p).cmdpos;
    (*p).cmdbuff_replaced = true_0 != 0;
    redrawcmd();
    do_autocmd_cmdlinechanged(get_cmdline_type());
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn set_cmdline_pos(mut pos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut p: *mut CmdlineInfo = get_ccline_ptr();
    if p.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    new_cmdpos = if 0 as ::core::ffi::c_int > pos {
        0 as ::core::ffi::c_int
    } else {
        pos
    };
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_setcmdline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut pos: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut error: bool = false_0 != 0;
        pos = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int;
        if error {
            return;
        }
        if pos < 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_positive as *const ::core::ffi::c_char));
            return;
        }
    }
    (*rettv).vval.v_number = set_cmdline_str(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        pos,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_setcmdpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let pos: ::core::ffi::c_int = tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize))
        as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int;
    if pos >= 0 as ::core::ffi::c_int {
        (*rettv).vval.v_number = set_cmdline_pos(pos) as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_cmdline_firstc() -> ::core::ffi::c_int {
    return ccline.cmdfirstc;
}
#[no_mangle]
pub unsafe extern "C" fn get_list_range(
    mut str: *mut *mut ::core::ffi::c_char,
    mut num1: *mut ::core::ffi::c_int,
    mut num2: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    let mut first: bool = false_0 != 0;
    let mut num: varnumber_T = 0;
    *str = skipwhite(*str);
    if **str as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || ascii_isdigit(**str as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        vim_str2nr(
            *str,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut len,
            0 as ::core::ffi::c_int,
            &raw mut num,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        *str = (*str).offset(len as isize);
        if num > INT_MAX as varnumber_T {
            return FAIL;
        }
        *num1 = num as ::core::ffi::c_int;
        first = true_0 != 0;
    }
    *str = skipwhite(*str);
    if **str as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        *str = skipwhite((*str).offset(1 as ::core::ffi::c_int as isize));
        vim_str2nr(
            *str,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut len,
            0 as ::core::ffi::c_int,
            &raw mut num,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if len > 0 as ::core::ffi::c_int {
            *str = skipwhite((*str).offset(len as isize));
            if num > INT_MAX as varnumber_T {
                return FAIL;
            }
            *num2 = num as ::core::ffi::c_int;
        } else if !first {
            return FAIL;
        }
    } else if first {
        *num2 = *num1;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_init() {
    memset(
        &raw mut ccline as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CmdlineInfo>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_cedit(mut _args: *mut optset_T) -> *const ::core::ffi::c_char {
    if *p_cedit as ::core::ffi::c_int == NUL {
        cedit_key = -1 as ::core::ffi::c_int;
    } else {
        let mut n: ::core::ffi::c_int = string_to_key(p_cedit);
        if n == 0 as ::core::ffi::c_int || vim_isprintc(n) as ::core::ffi::c_int != 0 {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        cedit_key = n;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn open_cmdwin() -> ::core::ffi::c_int {
    let mut old_curbuf: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut old_curwin: *mut win_T = curwin;
    let mut i: ::core::ffi::c_int = 0;
    let mut winsizes: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut save_restart_edit: ::core::ffi::c_int = restart_edit;
    let mut save_State: ::core::ffi::c_int = State;
    let mut save_exmode: bool = exmode_active;
    let mut save_cmdmsg_rl: bool = cmdmsg_rl;
    if text_or_buf_locked() as ::core::ffi::c_int != 0
        || cmdwin_type != 0 as ::core::ffi::c_int
        || cmdline_star > 0 as ::core::ffi::c_int
    {
        beep_flush();
        return -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    }
    set_bufref(&raw mut old_curbuf, curbuf);
    win_size_save(&raw mut winsizes);
    pum_undisplay(true_0 != 0);
    cmdmod.cmod_tab = 0 as ::core::ffi::c_int;
    cmdmod.cmod_flags |= CMOD_NOSWAPFILE as ::core::ffi::c_int;
    if win_split(p_cwh as ::core::ffi::c_int, WSP_BOT as ::core::ffi::c_int) == FAIL {
        beep_flush();
        ga_clear(&raw mut winsizes);
        return -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
    }
    if !win_valid(old_curwin)
        || curwin == old_curwin
        || !bufref_valid(&raw mut old_curbuf)
        || (*old_curwin).w_buffer != old_curbuf.br_buf
    {
        beep_flush();
        ga_clear(&raw mut winsizes);
        return Ctrl_C;
    }
    got_int = false_0 != 0;
    cmdwin_type = get_cmdline_type();
    cmdwin_level = ccline.level;
    cmdwin_win = curwin;
    cmdwin_old_curwin = old_curwin;
    let newbuf_status: ::core::ffi::c_int = buf_open_scratch(
        0 as handle_T,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    let cmdwin_valid: bool = win_valid(cmdwin_win);
    if newbuf_status == FAIL
        || !cmdwin_valid
        || curwin != cmdwin_win
        || !win_valid(old_curwin)
        || !bufref_valid(&raw mut old_curbuf)
        || (*old_curwin).w_buffer != old_curbuf.br_buf
    {
        if newbuf_status == OK {
            set_bufref(&raw mut bufref, curbuf);
        }
        if cmdwin_valid as ::core::ffi::c_int != 0 && !last_window(cmdwin_win) {
            win_close(cmdwin_win, true_0 != 0, false_0 != 0);
        }
        if newbuf_status == OK
            && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
            && bufref.br_buf != curbuf
        {
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                bufref.br_buf,
                DOBUF_WIPE as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
        }
        cmdwin_type = 0 as ::core::ffi::c_int;
        cmdwin_level = 0 as ::core::ffi::c_int;
        cmdwin_win = ::core::ptr::null_mut::<win_T>();
        cmdwin_old_curwin = ::core::ptr::null_mut::<win_T>();
        beep_flush();
        ga_clear(&raw mut winsizes);
        return Ctrl_C;
    }
    cmdwin_buf = curbuf;
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
    (*curbuf).b_p_ma = true_0;
    (*curwin).w_onebuf_opt.wo_fen = false_0;
    (*curwin).w_onebuf_opt.wo_rl = cmdmsg_rl as ::core::ffi::c_int;
    cmdmsg_rl = false_0 != 0;
    (*curbuf).b_ro_locked += 1;
    need_wait_return = false_0 != 0;
    let histtype: ::core::ffi::c_int = hist_char2type(cmdwin_type) as ::core::ffi::c_int;
    if histtype == HIST_CMD as ::core::ffi::c_int || histtype == HIST_DEBUG as ::core::ffi::c_int {
        if p_wc == TAB as OptInt {
            add_map(
                b"<Tab>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"<C-X><C-V>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                MODE_INSERT as ::core::ffi::c_int,
                true_0 != 0,
            );
            add_map(
                b"<Tab>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"a<C-X><C-V>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                MODE_NORMAL as ::core::ffi::c_int,
                true_0 != 0,
            );
        }
        set_option_value_give_err(
            kOptFiletype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"vim\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
    }
    (*curbuf).b_ro_locked -= 1;
    (*curbuf).b_p_tw = 0 as OptInt;
    init_history();
    if get_hislen() > 0 as ::core::ffi::c_int && histtype != HIST_INVALID as ::core::ffi::c_int {
        i = *get_hisidx(histtype);
        if i >= 0 as ::core::ffi::c_int {
            let mut lnum: linenr_T = 0 as linenr_T;
            loop {
                i += 1;
                if i == get_hislen() {
                    i = 0 as ::core::ffi::c_int;
                }
                if !(*get_histentry(histtype).offset(i as isize))
                    .hisstr
                    .is_null()
                {
                    let c2rust_fresh31 = lnum;
                    lnum = lnum + 1;
                    ml_append(
                        c2rust_fresh31,
                        (*get_histentry(histtype).offset(i as isize)).hisstr,
                        0 as colnr_T,
                        false_0 != 0,
                    );
                }
                if i == *get_hisidx(histtype) {
                    break;
                }
            }
        }
    }
    ml_replace((*curbuf).b_ml.ml_line_count, ccline.cmdbuff, true_0 != 0);
    (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
    (*curwin).w_cursor.col = ccline.cmdpos as colnr_T;
    changed_line_abv_curs();
    invalidate_botline_win(curwin);
    ui_ext_cmdline_hide(false_0 != 0);
    redraw_later(curwin, UPD_SOME_VALID as ::core::ffi::c_int);
    exmode_active = false_0 != 0;
    State = MODE_NORMAL as ::core::ffi::c_int;
    setmouse();
    clear_showcmd();
    cmdwin_result = 0 as ::core::ffi::c_int;
    trigger_cmd_autocmd(cmdwin_type, EVENT_CMDWINENTER);
    if restart_edit != 0 as ::core::ffi::c_int {
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    }
    i = RedrawingDisabled;
    RedrawingDisabled = 0 as ::core::ffi::c_int;
    let mut save_count: ::core::ffi::c_int = save_batch_count();
    normal_enter(true_0 != 0, false_0 != 0);
    RedrawingDisabled = i;
    restore_batch_count(save_count);
    let save_KeyTyped: bool = KeyTyped;
    trigger_cmd_autocmd(cmdwin_type, EVENT_CMDWINLEAVE);
    KeyTyped = save_KeyTyped;
    cmdwin_type = 0 as ::core::ffi::c_int;
    cmdwin_level = 0 as ::core::ffi::c_int;
    cmdwin_buf = ::core::ptr::null_mut::<buf_T>();
    cmdwin_win = ::core::ptr::null_mut::<win_T>();
    cmdwin_old_curwin = ::core::ptr::null_mut::<win_T>();
    exmode_active = save_exmode;
    if !win_valid(old_curwin)
        || !bufref_valid(&raw mut old_curbuf)
        || (*old_curwin).w_buffer != old_curbuf.br_buf
    {
        cmdwin_result = Ctrl_C;
        emsg(gettext(
            &raw const e_active_window_or_buffer_changed_or_deleted as *const ::core::ffi::c_char,
        ));
    } else {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if aborting() as ::core::ffi::c_int != 0
            && cmdwin_result
                != -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            cmdwin_result = Ctrl_C;
        }
        dealloc_cmdbuff();
        if cmdwin_result
            == -(253 as ::core::ffi::c_int
                + ((KE_XF1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || cmdwin_result
                == -(253 as ::core::ffi::c_int
                    + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            let mut p: *const ::core::ffi::c_char = if cmdwin_result
                == -(253 as ::core::ffi::c_int
                    + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                b"qa\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"qa!\0".as_ptr() as *const ::core::ffi::c_char
            };
            let mut plen: size_t = (if cmdwin_result
                == -(253 as ::core::ffi::c_int
                    + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                2 as ::core::ffi::c_int
            } else {
                3 as ::core::ffi::c_int
            }) as size_t;
            if histtype == HIST_CMD as ::core::ffi::c_int {
                ccline.cmdbuff =
                    xmemdupz(p as *const ::core::ffi::c_void, plen) as *mut ::core::ffi::c_char;
                ccline.cmdlen = plen as ::core::ffi::c_int;
                ccline.cmdbufflen = plen as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                cmdwin_result = CAR;
            } else {
                stuffcharReadbuff(':' as ::core::ffi::c_int);
                stuffReadbuff(p);
                stuffcharReadbuff(CAR);
            }
        } else if cmdwin_result == Ctrl_C {
            ccline.cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            ccline.cmdlen = get_cursor_line_len() as ::core::ffi::c_int;
            ccline.cmdbufflen = ccline.cmdlen + 1 as ::core::ffi::c_int;
            ccline.cmdbuff = xstrnsave(get_cursor_line_ptr(), ccline.cmdlen as size_t);
        }
        if ccline.cmdbuff.is_null() {
            ccline.cmdbuff = xmemdupz(
                b"\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                0 as size_t,
            ) as *mut ::core::ffi::c_char;
            ccline.cmdlen = 0 as ::core::ffi::c_int;
            ccline.cmdbufflen = 1 as ::core::ffi::c_int;
            ccline.cmdpos = 0 as ::core::ffi::c_int;
            cmdwin_result = Ctrl_C;
        } else {
            ccline.cmdpos = (*curwin).w_cursor.col as ::core::ffi::c_int;
            if ccline.cmdpos == ccline.cmdlen - 1 as ::core::ffi::c_int
                || ccline.cmdpos > ccline.cmdlen
            {
                ccline.cmdpos = ccline.cmdlen;
            }
            if cmdwin_result
                == -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                ccline.cmdspos = cmd_screencol(ccline.cmdpos);
                redrawcmd();
            }
        }
        (*curwin).w_onebuf_opt.wo_cole = 0 as OptInt;
        wp = curwin;
        set_bufref(&raw mut bufref, curbuf);
        skip_win_fix_cursor = true_0 != 0;
        win_goto(old_curwin);
        if win_valid(wp) as ::core::ffi::c_int != 0 && wp != curwin {
            win_close(wp, true_0 != 0, false_0 != 0);
        }
        if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0 && bufref.br_buf != curbuf {
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                bufref.br_buf,
                DOBUF_WIPE as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
        }
        win_size_restore(&raw mut winsizes);
        skip_win_fix_cursor = false_0 != 0;
    }
    ga_clear(&raw mut winsizes);
    restart_edit = save_restart_edit;
    cmdmsg_rl = save_cmdmsg_rl;
    State = save_State;
    may_trigger_modechanged();
    setmouse();
    setcursor();
    return cmdwin_result;
}
#[no_mangle]
pub unsafe extern "C" fn is_in_cmdwin() -> bool {
    return cmdwin_type != 0 as ::core::ffi::c_int && get_cmdline_type() == NUL;
}
#[no_mangle]
pub unsafe extern "C" fn script_get(
    eap: *mut exarg_T,
    lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut cmd: *mut ::core::ffi::c_char = (*eap).arg;
    if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '<' as ::core::ffi::c_int
        || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '<' as ::core::ffi::c_int
        || (*eap).ea_getline.is_none()
    {
        *lenp = strlen((*eap).arg);
        return (if (*eap).skip != 0 {
            NULL_0
        } else {
            xmemdupz((*eap).arg as *const ::core::ffi::c_void, *lenp)
        }) as *mut ::core::ffi::c_char;
    }
    cmd = cmd.offset(2 as ::core::ffi::c_int as isize);
    let mut ga: garray_T = garray_T {
        ga_len: 0 as ::core::ffi::c_int,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: NULL_0,
    };
    let l: *mut list_T = heredoc_get(eap, cmd, true_0 != 0);
    if l.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*eap).skip == 0 {
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            0x400 as ::core::ffi::c_int,
        );
    }
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*eap).skip == 0 {
                ga_concat(&raw mut ga, tv_get_string(&raw const (*li).li_tv));
                ga_append(&raw mut ga, '\n' as uint8_t);
            }
            li = (*li).li_next;
        }
    }
    *lenp = ga.ga_len as size_t;
    if (*eap).skip == 0 {
        ga_append(&raw mut ga, NUL as uint8_t);
    }
    tv_list_free(l);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn get_user_input(
    argvars: *const typval_T,
    rettv: *mut typval_T,
    inputdialog: bool,
    secret: bool,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cmdpreview {
        return;
    }
    let mut prompt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut defstr: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut cancelreturn: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    let mut cancelreturn_strarg2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut xp_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut input_callback: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut prompt_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut defstr_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut cancelreturn_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut xp_name_buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut def: [::core::ffi::c_char; 1] = [0 as ::core::ffi::c_char];
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                b"E5050: {opts} must be the only argument\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            return;
        }
        let dict: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        prompt = tv_dict_get_string_buf_chk(
            dict,
            b"prompt\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut prompt_buf as *mut ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if prompt.is_null() {
            return;
        }
        defstr = tv_dict_get_string_buf_chk(
            dict,
            b"default\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut defstr_buf as *mut ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if defstr.is_null() {
            return;
        }
        let mut cancelreturn_di: *mut dictitem_T = tv_dict_find(
            dict,
            b"cancelreturn\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !cancelreturn_di.is_null() {
            cancelreturn = &raw mut (*cancelreturn_di).di_tv;
        }
        xp_name = tv_dict_get_string_buf_chk(
            dict,
            b"completion\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut xp_name_buf as *mut ::core::ffi::c_char,
            &raw mut def as *mut ::core::ffi::c_char,
        );
        if xp_name.is_null() {
            return;
        }
        if xp_name == &raw mut def as *mut ::core::ffi::c_char as *const ::core::ffi::c_char {
            xp_name = ::core::ptr::null::<::core::ffi::c_char>();
        }
        if !tv_dict_get_callback(
            dict,
            b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
            &raw mut input_callback,
        ) {
            return;
        }
    } else {
        prompt = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut prompt_buf as *mut ::core::ffi::c_char,
        );
        if prompt.is_null() {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            defstr = tv_get_string_buf_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut defstr_buf as *mut ::core::ffi::c_char,
            );
            if defstr.is_null() {
                return;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let strarg2: *const ::core::ffi::c_char = tv_get_string_buf_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut cancelreturn_buf as *mut ::core::ffi::c_char,
                );
                if strarg2.is_null() {
                    return;
                }
                if inputdialog {
                    cancelreturn_strarg2.v_type = VAR_STRING;
                    cancelreturn_strarg2.vval.v_string = strarg2 as *mut ::core::ffi::c_char;
                    cancelreturn = &raw mut cancelreturn_strarg2;
                } else {
                    xp_name = strarg2;
                }
            }
        }
    }
    let mut xp_type: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut xp_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !xp_name.is_null() {
        let xp_namelen: ::core::ffi::c_int = strlen(xp_name) as ::core::ffi::c_int;
        let mut argt: uint32_t = 0 as uint32_t;
        if parse_compl_arg(
            xp_name,
            xp_namelen,
            &raw mut xp_type,
            &raw mut argt,
            &raw mut xp_arg,
        ) == FAIL
        {
            return;
        }
    }
    let mut p: *const ::core::ffi::c_char = prompt;
    if !ui_has(kUICmdline) {
        let mut lastnl: *const ::core::ffi::c_char = strrchr(prompt, '\n' as ::core::ffi::c_int);
        if !lastnl.is_null() {
            p = lastnl.offset(1 as ::core::ffi::c_int as isize);
            msg_start();
            msg_clr_eos();
            msg_puts_len(
                prompt,
                p.offset_from(prompt),
                get_echo_hl_id(),
                false_0 != 0,
            );
            msg_didout = false_0 != 0;
            msg_starthere();
        }
    }
    cmdline_row = msg_row;
    stuffReadbuffSpec(defstr);
    let save_ex_normal_busy: ::core::ffi::c_int = ex_normal_busy;
    ex_normal_busy = 0 as ::core::ffi::c_int;
    (*rettv).vval.v_string = getcmdline_prompt(
        if secret as ::core::ffi::c_int != 0 {
            NUL
        } else {
            '@' as ::core::ffi::c_int
        },
        p,
        get_echo_hl_id(),
        xp_type,
        xp_arg,
        input_callback,
        false_0 != 0,
        ::core::ptr::null_mut::<bool>(),
    );
    ex_normal_busy = save_ex_normal_busy;
    callback_free(&raw mut input_callback);
    if (*rettv).vval.v_string.is_null() && !cancelreturn.is_null() {
        tv_copy(cancelreturn, rettv);
    }
    xfree(xp_arg as *mut ::core::ffi::c_void);
    need_wait_return = false_0 != 0;
    msg_didout = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn f_wildtrigger(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if State & MODE_CMDLINE as ::core::ffi::c_int == 0
        || char_avail() as ::core::ffi::c_int != 0
        || wild_menu_showing != 0
        || cmdline_pum_active() as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut cmd_type: ::core::ffi::c_int = get_cmdline_type();
    if cmd_type == ':' as ::core::ffi::c_int
        || cmd_type == '/' as ::core::ffi::c_int
        || cmd_type == '?' as ::core::ffi::c_int
    {
        let mut key_string: [uint8_t; 4] = [0; 4];
        key_string[0 as ::core::ffi::c_int as usize] = K_SPECIAL as uint8_t;
        key_string[1 as ::core::ffi::c_int as usize] = KS_EXTRA as uint8_t;
        key_string[2 as ::core::ffi::c_int as usize] = KE_WILD as ::core::ffi::c_int as uint8_t;
        key_string[3 as ::core::ffi::c_int as usize] = NUL as uint8_t;
        ins_typebuf(
            &raw mut key_string as *mut uint8_t as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
        );
    }
}
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_LEFT: ::core::ffi::c_int = -27755;
pub const K_RIGHT: ::core::ffi::c_int = -29291;
pub const K_S_LEFT: ::core::ffi::c_int =
    -('#' as ::core::ffi::c_int + (('4' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_RIGHT: ::core::ffi::c_int =
    -('%' as ::core::ffi::c_int + (('i' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_S_HOME: ::core::ffi::c_int = -12835;
pub const K_C_HOME: ::core::ffi::c_int = -22525;
pub const K_S_END: ::core::ffi::c_int = -14122;
pub const K_C_END: ::core::ffi::c_int = -22781;
pub const K_S_TAB: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('B' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_INS: ::core::ffi::c_int = -18795;
pub const K_KINS: ::core::ffi::c_int = -20477;
pub const K_DEL: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('D' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_HOME: ::core::ffi::c_int = -26731;
pub const K_KHOME: ::core::ffi::c_int = -12619;
pub const K_END: ::core::ffi::c_int = -14144;
pub const K_KEND: ::core::ffi::c_int = -13387;
pub const K_PAGEUP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('P' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_PAGEDOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEUP: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('3' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KPAGEDOWN: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('5' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_SELECT: ::core::ffi::c_int = -22773;
pub const K_LEFTMOUSE: ::core::ffi::c_int = -11517;
pub const K_LEFTDRAG: ::core::ffi::c_int = -11773;
pub const K_MOUSEMOVE: ::core::ffi::c_int = -25853;
pub const K_MIDDLEMOUSE: ::core::ffi::c_int = -12285;
pub const K_MIDDLEDRAG: ::core::ffi::c_int = -12541;
pub const K_MIDDLERELEASE: ::core::ffi::c_int = -12797;
pub const K_RIGHTMOUSE: ::core::ffi::c_int = -13053;
pub const K_RIGHTDRAG: ::core::ffi::c_int = -13309;
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
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
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
#[inline(always)]
unsafe extern "C" fn viml_parser_init(
    ret_pstate: *mut ParserState,
    get_line: ParserLineGetter,
    cookie: *mut ::core::ffi::c_void,
    colors: *mut ParserHighlight,
) {
    *ret_pstate = ParserState {
        reader: ParserInputReader {
            get_line: get_line,
            cookie: cookie,
            lines: C2Rust_Unnamed_35 {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<ParserLine>(),
                init_array: [ParserLine {
                    data: ::core::ptr::null::<::core::ffi::c_char>(),
                    size: 0,
                    allocated: false,
                }; 4],
            },
            conv: vimconv_T {
                vc_type: CONV_NONE as ::core::ffi::c_int,
                vc_factor: 1 as ::core::ffi::c_int,
                vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                vc_fail: false_0 != 0,
            },
        },
        pos: ParserPosition {
            line: 0 as size_t,
            col: 0 as size_t,
        },
        stack: C2Rust_Unnamed_30 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserStateItem>(),
            init_array: [ParserStateItem {
                type_0: kPTopStateParsingCommand,
                data: C2Rust_Unnamed_31 {
                    expr: C2Rust_Unnamed_32 {
                        type_0: kExprUnknown,
                    },
                },
            }; 16],
        },
        colors: colors,
        can_continuate: false_0 != 0,
    };
    (*ret_pstate).reader.lines.capacity = ::core::mem::size_of::<[ParserLine; 4]>()
        .wrapping_div(::core::mem::size_of::<ParserLine>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserLine; 4]>()
                .wrapping_rem(::core::mem::size_of::<ParserLine>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    (*ret_pstate).reader.lines.size = 0 as size_t;
    (*ret_pstate).reader.lines.items =
        &raw mut (*ret_pstate).reader.lines.init_array as *mut ParserLine;
    (*ret_pstate).stack.capacity = ::core::mem::size_of::<[ParserStateItem; 16]>()
        .wrapping_div(::core::mem::size_of::<ParserStateItem>())
        .wrapping_div(
            (::core::mem::size_of::<[ParserStateItem; 16]>()
                .wrapping_rem(::core::mem::size_of::<ParserStateItem>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    (*ret_pstate).stack.size = 0 as size_t;
    (*ret_pstate).stack.items = &raw mut (*ret_pstate).stack.init_array as *mut ParserStateItem;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (INT_MAX as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn c2rust_run_static_initializers() {
    prev_prompt_id = UINT_MAX;
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
