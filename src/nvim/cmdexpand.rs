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
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
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
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstpcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_arglist_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn expand_get_augroup_name(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn set_context_in_autocmd(
        xp: *mut expand_T,
        arg: *mut ::core::ffi::c_char,
        doautocmd: bool,
    ) -> *mut ::core::ffi::c_char;
    fn expand_get_event_name(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ExpandBufnames(
        pat: *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static mut p_fic: ::core::ffi::c_int;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_ls: OptInt;
    static mut p_scs: ::core::ffi::c_int;
    static mut wop_flags: ::core::ffi::c_uint;
    static mut p_wc: OptInt;
    static mut p_wic: ::core::ffi::c_int;
    static mut p_wmnu: ::core::ffi::c_int;
    static mut p_wmh: OptInt;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcase_save(orig: *const ::core::ffi::c_char, upper: bool) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn sort_strings(files: *mut *mut ::core::ffi::c_char, count: ::core::ffi::c_int);
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn transchar_byte(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn vim_isfilec_or_wc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn rem_backslash(str: *const ::core::ffi::c_char) -> bool;
    fn backslash_halve_save(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_free_object(value: Object);
    fn api_clear_error(value: *mut Error);
    fn get_history_arg(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn update_screen() -> ::core::ffi::c_int;
    fn redraw_statuslines();
    fn win_redraw_last_status(frp: *const frame_T);
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_nomatch2: [::core::ffi::c_char; 0];
    static e_toomany: [::core::ffi::c_char; 0];
    fn call_func_retstr(
        func: *const ::core::ffi::c_char,
        argc: ::core::ffi::c_int,
        argv: *mut typval_T,
    ) -> *mut ::core::ffi::c_void;
    fn call_func_retlist(
        func: *const ::core::ffi::c_char,
        argc: ::core::ffi::c_int,
        argv: *mut typval_T,
    ) -> *mut ::core::ffi::c_void;
    fn set_context_for_expression(
        xp: *mut expand_T,
        arg: *mut ::core::ffi::c_char,
        cmdidx: cmdidx_T,
    );
    fn get_function_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_expr_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    static mut hash_removed: ::core::ffi::c_char;
    fn hash_init(ht: *mut hashtab_T);
    fn hash_clear(ht: *mut hashtab_T);
    fn hash_lookup(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        hash: hash_T,
    ) -> *mut hashitem_T;
    fn hash_add_item(
        ht: *mut hashtab_T,
        hi: *mut hashitem_T,
        key: *mut ::core::ffi::c_char,
        hash: hash_T,
    );
    fn hash_hash(key: *const ::core::ffi::c_char) -> hash_T;
    static mut msg_grid_adj: GridView;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_outtrans_long(longstr: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_scroll_up(may_throttle: bool, zerocmd: bool);
    fn msg_clr_eos();
    fn msg_advance(col: ::core::ffi::c_int);
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_unref(l: *mut list_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_str(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_string_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn get_user_func_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_user_var_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn skip_vimgrep_pat(
        p: *mut ::core::ffi::c_char,
        s: *mut *mut ::core::ffi::c_char,
        flags: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn excmd_get_cmdidx(cmd: *const ::core::ffi::c_char, len: size_t) -> cmdidx_T;
    fn excmd_get_argt(idx: cmdidx_T) -> uint32_t;
    fn skip_range(
        cmd: *const ::core::ffi::c_char,
        ctx: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn skip_cmd_arg(p: *mut ::core::ffi::c_char, rembs: bool) -> *mut ::core::ffi::c_char;
    fn expand_argopt(
        pat: *mut ::core::ffi::c_char,
        xp: *mut expand_T,
        rmp: *mut regmatch_T,
        matches: *mut *mut *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn find_nextcmd(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_command_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn expand_findfunc(
        pat: *mut ::core::ffi::c_char,
        files: *mut *mut *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn set_no_hlsearch(flag: bool);
    fn parse_pattern_and_range(
        incsearch_start: *mut pos_T,
        search_delim: *mut ::core::ffi::c_int,
        skiplen: *mut ::core::ffi::c_int,
        patlen: *mut ::core::ffi::c_int,
    ) -> bool;
    fn cmd_screencol(bytepos: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn realloc_cmdbuff(len: ::core::ffi::c_int);
    fn put_on_cmdline(str: *const ::core::ffi::c_char, len: ::core::ffi::c_int, redraw: bool);
    fn redrawcmd();
    fn cursorcmd();
    fn vim_strsave_fnameescape(
        fname: *const ::core::ffi::c_char,
        what: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn escape_fname(pp: *mut *mut ::core::ffi::c_char);
    fn tilde_replace(
        orig_pat: *mut ::core::ffi::c_char,
        num_files: ::core::ffi::c_int,
        files: *mut *mut ::core::ffi::c_char,
    );
    fn get_cmdline_info() -> *mut CmdlineInfo;
    fn get_cmdline_last_prompt_id() -> ::core::ffi::c_uint;
    fn fuzzy_match_str(
        str: *mut ::core::ffi::c_char,
        pat: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn fuzzymatches_to_strmatches(
        fuzmatch: *mut fuzmatch_str_T,
        matches: *mut *mut *mut ::core::ffi::c_char,
        count: ::core::ffi::c_int,
        funcsort: bool,
    );
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn beep_flush();
    fn vpeekc() -> ::core::ffi::c_int;
    fn char_avail() -> bool;
    static mut default_gridview: GridView;
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_didany: bool;
    static mut emsg_off: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut search_first_line: linenr_T;
    static mut search_last_line: linenr_T;
    static mut lastwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut topframe: *mut frame_T;
    static mut curbuf: *mut buf_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut cmd_silent: bool;
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut got_int: bool;
    static mut save_p_ls: ::core::ffi::c_int;
    static mut save_p_wmh: ::core::ffi::c_int;
    static mut wild_menu_showing: ::core::ffi::c_int;
    static mut cmdline_win: *mut win_T;
    fn find_help_tags(
        arg: *const ::core::ffi::c_char,
        num_matches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
        keep_lang: bool,
    ) -> ::core::ffi::c_int;
    fn cleanup_help_tags(num_file: ::core::ffi::c_int, file: *mut *mut ::core::ffi::c_char);
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
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
    static mut ns_hl_fast: NS;
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn set_context_in_highlight_cmd(xp: *mut expand_T, arg: *const ::core::ffi::c_char);
    fn get_highlight_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn find_word_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_ucmd(
        eap: *mut exarg_T,
        p: *mut ::core::ffi::c_char,
        full: *mut ::core::ffi::c_int,
        xp: *mut expand_T,
        complp: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn set_context_in_user_cmd(
        xp: *mut expand_T,
        arg_in: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn set_context_in_user_cmdarg(
        cmd: *const ::core::ffi::c_char,
        arg: *const ::core::ffi::c_char,
        argt: uint32_t,
        context: ::core::ffi::c_int,
        xp: *mut expand_T,
        forceit: bool,
    ) -> *const ::core::ffi::c_char;
    fn get_user_commands(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_user_cmd_addr_type(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn get_user_cmd_flags(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_user_cmd_nargs(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_user_cmd_complete(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn cmdcomplete_type_to_str(
        expand: ::core::ffi::c_int,
        compl_arg: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmdcomplete_str_to_type(complete_str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn nlua_call_user_expand_func(xp: *mut expand_T, ret_tv: *mut typval_T);
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_expand_pat(xp: *mut expand_T);
    fn nlua_expand_get_matches(
        num_results: *mut ::core::ffi::c_int,
        results: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn set_context_in_map_cmd(
        xp: *mut expand_T,
        cmd: *mut ::core::ffi::c_char,
        arg: *mut ::core::ffi::c_char,
        forceit: bool,
        isabbrev: bool,
        isunmap: bool,
        cmdidx: cmdidx_T,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandMappings(
        pat: *mut ::core::ffi::c_char,
        regmatch: *mut regmatch_T,
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn set_context_in_menu_cmd(
        xp: *mut expand_T,
        cmd: *const ::core::ffi::c_char,
        arg: *mut ::core::ffi::c_char,
        forceit: bool,
    ) -> *mut ::core::ffi::c_char;
    fn get_menu_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_menu_names(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn menu_is_separator(name: *mut ::core::ffi::c_char) -> bool;
    fn get_findfunc() -> *mut ::core::ffi::c_char;
    fn set_context_in_set_cmd(
        xp: *mut expand_T,
        arg: *mut ::core::ffi::c_char,
        opt_flags: ::core::ffi::c_int,
    );
    fn ExpandSettings(
        xp: *mut expand_T,
        regmatch: *mut regmatch_T,
        fuzzystr: *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
        can_fuzzy: bool,
    ) -> ::core::ffi::c_int;
    fn ExpandOldSetting(
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ExpandStringSetting(
        xp: *mut expand_T,
        regmatch: *mut regmatch_T,
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ExpandSettingSubtract(
        xp: *mut expand_T,
        regmatch: *mut regmatch_T,
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn magic_isset() -> bool;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn csh_like_shell() -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn get_lang_arg(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_locales(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn expand_env_save_opt(src: *mut ::core::ffi::c_char, one: bool) -> *mut ::core::ffi::c_char;
    fn vim_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn get_env_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_users(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn match_user(name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    static mut pum_want: C2Rust_Unnamed_22;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn expand_wildcards_eval(
        pat: *mut *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_files: *mut ::core::ffi::c_int,
        files: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn match_suffix(fname: *mut ::core::ffi::c_char) -> bool;
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn pum_display(
        array: *mut pumitem_T,
        size: ::core::ffi::c_int,
        selected: ::core::ffi::c_int,
        array_changed: bool,
        cmd_startcol: ::core::ffi::c_int,
    );
    fn pum_undisplay(immediate: bool);
    fn pum_clear();
    fn pum_visible() -> bool;
    fn pum_get_height() -> ::core::ffi::c_int;
    fn get_profile_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn set_context_in_profile_cmd(xp: *mut expand_T, arg: *const ::core::ffi::c_char);
    fn skip_regexp(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
        -> bool;
    fn set_context_in_runtime_cmd(xp: *mut expand_T, arg: *const ::core::ffi::c_char);
    fn ExpandRTDir(
        pat: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        dirnames: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn expand_runtime_cmd(
        pat: *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
        matches: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ExpandPackAddDir(
        pat: *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static mut script_items: garray_T;
    fn ignorecase(pat: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn get_sign_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn set_context_in_sign_cmd(xp: *mut expand_T, arg: *mut ::core::ffi::c_char);
    fn fillchar_status(group: *mut hlf_T, wp: *mut win_T) -> schar_T;
    fn expand_tags(
        tagnames: bool,
        pat: *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn reset_expand_highlight();
    fn set_context_in_echohl_cmd(xp: *mut expand_T, arg: *const ::core::ffi::c_char);
    fn set_context_in_syntax_cmd(xp: *mut expand_T, arg: *const ::core::ffi::c_char);
    fn get_syntax_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_syntime_arg(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn last_status(morewin: bool);
    fn global_stl_height() -> ::core::ffi::c_int;
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
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type ptrdiff_t = isize;
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
pub type NS = handle_T;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const XP_BS_COMMA: C2Rust_Unnamed_13 = 4;
pub const XP_BS_THREE: C2Rust_Unnamed_13 = 2;
pub const XP_BS_ONE: C2Rust_Unnamed_13 = 1;
pub const XP_BS_NONE: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_14 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_14 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_14 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_14 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_14 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_14 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_14 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_14 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_14 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_14 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_14 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_14 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_14 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_14 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_14 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_14 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_14 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_14 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_14 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_14 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_14 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_14 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_14 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_14 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_14 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_14 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_14 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_14 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_14 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_14 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_14 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_14 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_14 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_14 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_14 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_14 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_14 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_14 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_14 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_14 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_14 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_14 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_14 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_14 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_14 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_14 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_14 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_14 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_14 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub type CompleteListItemGetter =
    Option<unsafe extern "C" fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char>;
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
    pub cs_pend: C2Rust_Unnamed_15,
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
pub union C2Rust_Unnamed_15 {
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_16 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_16 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_16 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_16 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_16 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_16 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_16 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_16 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_16 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_16 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_16 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_16 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_16 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_16 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_16 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_16 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_16 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_16 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_16 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptWopFlagExacttext: C2Rust_Unnamed_17 = 8;
pub const kOptWopFlagPum: C2Rust_Unnamed_17 = 4;
pub const kOptWopFlagTagfile: C2Rust_Unnamed_17 = 2;
pub const kOptWopFlagFuzzy: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_18 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_18 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_18 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_18 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_18 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_18 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_18 = 7;
pub const WILD_ALL: C2Rust_Unnamed_18 = 6;
pub const WILD_PREV: C2Rust_Unnamed_18 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_18 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_18 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_18 = 2;
pub const WILD_FREE: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_19 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_19 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_19 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_19 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_19 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_19 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_19 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_19 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_19 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_19 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_19 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_19 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_19 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_19 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_19 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_19 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_19 = 1;
pub const VSE_NONE: C2Rust_Unnamed_24 = 0;
pub const VSE_BUFFER: C2Rust_Unnamed_24 = 2;
pub const VSE_SHELL: C2Rust_Unnamed_24 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expgen {
    pub context: ::core::ffi::c_int,
    pub func: ExpandFunc,
    pub ic: ::core::ffi::c_int,
    pub escaped: ::core::ffi::c_int,
}
pub type ExpandFunc = CompleteListItemGetter;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct scriptitem_T {
    pub sn_vars: *mut scriptvar_T,
    pub sn_name: *mut ::core::ffi::c_char,
    pub sn_lua: bool,
    pub sn_prof_on: bool,
    pub sn_pr_force: bool,
    pub sn_pr_child: proftime_T,
    pub sn_pr_nest: ::core::ffi::c_int,
    pub sn_pr_count: ::core::ffi::c_int,
    pub sn_pr_total: proftime_T,
    pub sn_pr_self: proftime_T,
    pub sn_pr_start: proftime_T,
    pub sn_pr_children: proftime_T,
    pub sn_prl_ga: garray_T,
    pub sn_prl_start: proftime_T,
    pub sn_prl_children: proftime_T,
    pub sn_prl_wait: proftime_T,
    pub sn_prl_idx: linenr_T,
    pub sn_prl_execed: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct scriptvar_T {
    pub sv_var: ScopeDictDictItem,
    pub sv_dict: dict_T,
}
pub const EXP_BREAKPT_DEL: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EXP_PROFDEL: C2Rust_Unnamed_20 = 2;
pub const EXP_BREAKPT_ADD: C2Rust_Unnamed_20 = 0;
pub const EXP_FILETYPECMD_ONOFF: C2Rust_Unnamed_21 = 3;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const EXP_FILETYPECMD_INDENT: C2Rust_Unnamed_21 = 2;
pub const EXP_FILETYPECMD_PLUGIN: C2Rust_Unnamed_21 = 1;
pub const EXP_FILETYPECMD_ALL: C2Rust_Unnamed_21 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzmatch_str_T {
    pub idx: ::core::ffi::c_int,
    pub str: *mut ::core::ffi::c_char,
    pub score: ::core::ffi::c_int,
}
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_25 = -2147483648;
pub type user_expand_func_T = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
        *mut typval_T,
    ) -> *mut ::core::ffi::c_void,
>;
pub const TAG_MANY: C2Rust_Unnamed_32 = 300;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchit_arg_T {
    pub sa_stop_lnum: linenr_T,
    pub sa_tm: *mut proftime_T,
    pub sa_timed_out: ::core::ffi::c_int,
    pub sa_wrapped: ::core::ffi::c_int,
}
pub const RE_LAST: C2Rust_Unnamed_31 = 2;
pub const SEARCH_START: C2Rust_Unnamed_30 = 256;
pub const SEARCH_NFMSG: C2Rust_Unnamed_30 = 8;
pub const SEARCH_PEEK: C2Rust_Unnamed_30 = 2048;
pub const SEARCH_NOOF: C2Rust_Unnamed_30 = 128;
pub const SEARCH_OPT: C2Rust_Unnamed_30 = 16;
pub const DIP_OPT: C2Rust_Unnamed_29 = 16;
pub const DIP_START: C2Rust_Unnamed_29 = 8;
pub const EW_DIR: C2Rust_Unnamed_28 = 1;
pub const EW_ALLLINKS: C2Rust_Unnamed_28 = 4096;
pub const EW_NOERROR: C2Rust_Unnamed_28 = 512;
pub const EW_SILENT: C2Rust_Unnamed_28 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_28 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_28 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_28 = 4;
pub const EW_SHELLCMD: C2Rust_Unnamed_28 = 8192;
pub const EW_EXEC: C2Rust_Unnamed_28 = 64;
pub const EW_FILE: C2Rust_Unnamed_28 = 2;
pub const EW_ICASE: C2Rust_Unnamed_28 = 256;
pub const EW_CDPATH: C2Rust_Unnamed_28 = 131072;
pub const EW_PATH: C2Rust_Unnamed_28 = 128;
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
pub const MB_MAXBYTES: C2Rust_Unnamed_23 = 21;
pub const WM_SCROLLED: C2Rust_Unnamed_26 = 2;
pub const WM_SHOWN: C2Rust_Unnamed_26 = 1;
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
pub struct C2Rust_Unnamed_22 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const EXPAND_FILETYPECMD_INDENT: C2Rust_Unnamed_33 = 2;
pub const EXPAND_FILETYPECMD_PLUGIN: C2Rust_Unnamed_33 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_27 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_23 = 6;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_27 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_27 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_27 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_27 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_27 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_27 = 4;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_28 = 262144;
pub const EW_NOTENV: C2Rust_Unnamed_28 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_28 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_28 = 16384;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_28 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_28 = 1024;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_29 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_29 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_29 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_29 = 32;
pub const DIP_ERR: C2Rust_Unnamed_29 = 4;
pub const DIP_DIR: C2Rust_Unnamed_29 = 2;
pub const DIP_ALL: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_30 = 4096;
pub const SEARCH_KEEP: C2Rust_Unnamed_30 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_30 = 512;
pub const SEARCH_END: C2Rust_Unnamed_30 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_30 = 32;
pub const SEARCH_MSG: C2Rust_Unnamed_30 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_30 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_30 = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const RE_BOTH: C2Rust_Unnamed_31 = 2;
pub const RE_SUBST: C2Rust_Unnamed_31 = 1;
pub const RE_SEARCH: C2Rust_Unnamed_31 = 0;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_32 = 256;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_32 = 128;
pub const TAG_INS_COMP: C2Rust_Unnamed_32 = 64;
pub const TAG_VERBOSE: C2Rust_Unnamed_32 = 32;
pub const TAG_NOIC: C2Rust_Unnamed_32 = 8;
pub const TAG_REGEXP: C2Rust_Unnamed_32 = 4;
pub const TAG_NAMES: C2Rust_Unnamed_32 = 2;
pub const TAG_HELP: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const EXPAND_FILETYPECMD_ONOFF: C2Rust_Unnamed_33 = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
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
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_NOTRLCOM: ::core::ffi::c_uint = 0x800 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const Ctrl_N: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
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
static mut cmd_showtail: bool = false;
static mut may_expand_pattern: bool = false_0 != 0;
static mut pre_incsearch_pos: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
static mut compl_match_array: *mut pumitem_T = ::core::ptr::null_mut::<pumitem_T>();
static mut compl_match_arraysize: ::core::ffi::c_int = 0;
static mut compl_startcol: ::core::ffi::c_int = 0;
static mut compl_selected: ::core::ffi::c_int = 0;
static mut cmdline_orig: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
unsafe extern "C" fn cmdline_fuzzy_completion_supported(xp: *const expand_T) -> bool {
    match (*xp).xp_context {
        5 | 28 | 29 | 3 | 56 | 2 | 37 | 36 | 59 | 58 | 8 | 55 | 63 | 7 | 52 | 53 | 38 | 44 | 51
        | 33 | 57 | 6 | 17 | 31 | 32 => return false_0 != 0,
        _ => {}
    }
    return wop_flags & kOptWopFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_fuzzy_complete(fuzzystr: *const ::core::ffi::c_char) -> bool {
    return wop_flags & kOptWopFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && *fuzzystr as ::core::ffi::c_int != NUL;
}
unsafe extern "C" fn sort_func_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut ::core::ffi::c_char = *(s1 as *mut *mut ::core::ffi::c_char);
    let mut p2: *mut ::core::ffi::c_char = *(s2 as *mut *mut ::core::ffi::c_char);
    if *p1 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        && *p2 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    if *p1 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        && *p2 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    return strcmp(p1, p2);
}
unsafe extern "C" fn wildescape(
    mut xp: *mut expand_T,
    mut str: *const ::core::ffi::c_char,
    mut numfiles: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let vse_what: ::core::ffi::c_int = if (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int {
        VSE_BUFFER as ::core::ffi::c_int
    } else {
        VSE_NONE as ::core::ffi::c_int
    };
    if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_FILES_IN_PATH as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < numfiles {
            if (*xp).xp_backslash & XP_BS_THREE as ::core::ffi::c_int != 0 {
                let mut pat: *mut ::core::ffi::c_char =
                    (if (*xp).xp_backslash & XP_BS_COMMA as ::core::ffi::c_int != 0 {
                        b" ,\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b" \0".as_ptr() as *const ::core::ffi::c_char
                    }) as *mut ::core::ffi::c_char;
                p = vim_strsave_escaped(*files.offset(i as isize), pat);
                xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                *files.offset(i as isize) = p;
            } else if (*xp).xp_backslash & XP_BS_COMMA as ::core::ffi::c_int != 0 {
                if !vim_strchr(*files.offset(i as isize), ',' as ::core::ffi::c_int).is_null() {
                    p = vim_strsave_escaped(
                        *files.offset(i as isize),
                        b",\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                    *files.offset(i as isize) = p;
                }
            }
            p = vim_strsave_fnameescape(
                *files.offset(i as isize),
                if (*xp).xp_shell as ::core::ffi::c_int != 0 {
                    VSE_SHELL as ::core::ffi::c_int
                } else {
                    vse_what
                },
            );
            xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
            *files.offset(i as isize) = p;
            if *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '~' as ::core::ffi::c_int
                && *(*files.offset(i as isize)).offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == '~' as ::core::ffi::c_int
            {
                escape_fname(files.offset(i as isize));
            }
            i += 1;
        }
        (*xp).xp_backslash = XP_BS_NONE as ::core::ffi::c_int;
        if **files.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '+' as ::core::ffi::c_int
        {
            escape_fname(files.offset(0 as ::core::ffi::c_int as isize));
        }
    } else if (*xp).xp_context == EXPAND_TAGS as ::core::ffi::c_int {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < numfiles {
            p = vim_strsave_escaped(
                *files.offset(i_0 as isize),
                b"\\|\"\0".as_ptr() as *const ::core::ffi::c_char,
            );
            xfree(*files.offset(i_0 as isize) as *mut ::core::ffi::c_void);
            *files.offset(i_0 as isize) = p;
            i_0 += 1;
        }
    }
}
unsafe extern "C" fn ExpandEscape(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut numfiles: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) {
    if options & WILD_HOME_REPLACE as ::core::ffi::c_int != 0 {
        tilde_replace(str, numfiles, files);
    }
    if options & WILD_ESCAPE as ::core::ffi::c_int != 0 {
        wildescape(xp, str, numfiles, files);
    }
}
#[no_mangle]
pub unsafe extern "C" fn nextwild(
    mut xp: *mut expand_T,
    mut type_0: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut escape: bool,
) -> ::core::ffi::c_int {
    let ccline: *mut CmdlineInfo = get_cmdline_info();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut from_wildtrigger_func: bool = options & WILD_FUNC_TRIGGER as ::core::ffi::c_int != 0;
    let mut wild_navigate: bool = type_0 == WILD_NEXT as ::core::ffi::c_int
        || type_0 == WILD_PREV as ::core::ffi::c_int
        || type_0 == WILD_PAGEUP as ::core::ffi::c_int
        || type_0 == WILD_PAGEDOWN as ::core::ffi::c_int
        || type_0 == WILD_PUM_WANT as ::core::ffi::c_int;
    if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
        pre_incsearch_pos = (*xp).xp_pre_incsearch_pos;
        if (*ccline).input_fn != 0 && (*ccline).xp_context == EXPAND_COMMANDS as ::core::ffi::c_int
        {
            set_cmd_context(
                xp,
                (*ccline).cmdbuff,
                (*ccline).cmdlen,
                (*ccline).cmdpos,
                false_0,
            );
        } else {
            may_expand_pattern = options & WILD_MAY_EXPAND_PATTERN as ::core::ffi::c_int != 0;
            set_expand_context(xp);
            may_expand_pattern = false_0 != 0;
        }
        if (*xp).xp_context == EXPAND_LUA as ::core::ffi::c_int {
            nlua_expand_pat(xp);
        }
        cmd_showtail = expand_showtail(xp);
    }
    if (*xp).xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
        beep_flush();
        return OK;
    }
    if (*xp).xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
        return FAIL;
    }
    let mut i: ::core::ffi::c_int =
        (*xp).xp_pattern.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int;
    '_c2rust_label: {
        if (*ccline).cmdpos >= i {
        } else {
            __assert_fail(
                b"ccline->cmdpos >= i\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                288 as ::core::ffi::c_uint,
                b"int nextwild(expand_T *, int, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*xp).xp_pattern_len = ((*ccline).cmdpos as size_t).wrapping_sub(i as size_t);
    if from_wildtrigger_func as ::core::ffi::c_int != 0
        && (*xp).xp_context == EXPAND_COMMANDS as ::core::ffi::c_int
        && (*xp).xp_pattern_len == 0 as size_t
    {
        return FAIL;
    }
    if !cmd_silent
        && !from_wildtrigger_func
        && !wild_navigate
        && !(ui_has(kUICmdline) as ::core::ffi::c_int != 0
            || ui_has(kUIWildmenu) as ::core::ffi::c_int != 0)
    {
        msg_puts(b"...\0".as_ptr() as *const ::core::ffi::c_char);
        ui_flush();
    }
    if wild_navigate {
        p = ExpandOne(
            xp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            type_0,
        );
    } else {
        let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0
            || (*xp).xp_context == EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int
        {
            tmp = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
        } else {
            tmp = addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context);
        }
        let use_options: ::core::ffi::c_int = options
            | WILD_HOME_REPLACE as ::core::ffi::c_int
            | WILD_ADD_SLASH as ::core::ffi::c_int
            | WILD_SILENT as ::core::ffi::c_int
            | (if escape as ::core::ffi::c_int != 0 {
                WILD_ESCAPE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | (if p_wic != 0 {
                WILD_ICASE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
        p = ExpandOne(
            xp,
            tmp,
            xstrnsave((*ccline).cmdbuff.offset(i as isize), (*xp).xp_pattern_len),
            use_options,
            type_0,
        );
        xfree(tmp as *mut ::core::ffi::c_void);
        if !p.is_null() && type_0 == WILD_LONGEST as ::core::ffi::c_int {
            let mut j: ::core::ffi::c_int = 0;
            j = 0 as ::core::ffi::c_int;
            while (j as size_t) < (*xp).xp_pattern_len {
                let mut c: ::core::ffi::c_char = *(*ccline).cmdbuff.offset((i + j) as isize);
                if c as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                    || c as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                {
                    break;
                }
                j += 1;
            }
            if (strlen(p) as ::core::ffi::c_int) < j {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut p as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
            }
        }
    }
    if !wild_navigate && !(*ccline).cmdbuff.is_null() {
        xfree(cmdline_orig as *mut ::core::ffi::c_void);
        cmdline_orig = xstrnsave((*ccline).cmdbuff, (*ccline).cmdlen as size_t);
    }
    if !p.is_null() && !got_int && options & WILD_NOSELECT as ::core::ffi::c_int == 0 {
        let mut plen: size_t = strlen(p);
        let mut difflen: ::core::ffi::c_int =
            plen as ::core::ffi::c_int - (*xp).xp_pattern_len as ::core::ffi::c_int;
        if (*ccline).cmdlen + difflen + 4 as ::core::ffi::c_int > (*ccline).cmdbufflen {
            realloc_cmdbuff((*ccline).cmdlen + difflen + 4 as ::core::ffi::c_int);
            (*xp).xp_pattern = (*ccline).cmdbuff.offset(i as isize);
        }
        '_c2rust_label_0: {
            if (*ccline).cmdpos <= (*ccline).cmdlen {
            } else {
                __assert_fail(
                    b"ccline->cmdpos <= ccline->cmdlen\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    355 as ::core::ffi::c_uint,
                    b"int nextwild(expand_T *, int, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            (*ccline)
                .cmdbuff
                .offset(((*ccline).cmdpos + difflen) as isize)
                as *mut ::core::ffi::c_void,
            (*ccline).cmdbuff.offset((*ccline).cmdpos as isize) as *const ::core::ffi::c_void,
            ((*ccline).cmdlen as size_t)
                .wrapping_sub((*ccline).cmdpos as size_t)
                .wrapping_add(1 as size_t),
        );
        memmove(
            (*ccline).cmdbuff.offset(i as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            plen,
        );
        (*ccline).cmdlen += difflen;
        (*ccline).cmdpos += difflen;
    }
    redrawcmd();
    cursorcmd();
    if (*xp).xp_context == EXPAND_MAPPINGS as ::core::ffi::c_int && p.is_null() {
        return FAIL;
    }
    if (*xp).xp_numfiles <= 0 as ::core::ffi::c_int && p.is_null() {
        beep_flush();
    } else if (*xp).xp_numfiles == 1 as ::core::ffi::c_int
        && options & WILD_NOSELECT as ::core::ffi::c_int == 0
        && !wild_navigate
    {
        ExpandOne(
            xp,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            WILD_FREE as ::core::ffi::c_int,
        );
    }
    xfree(p as *mut ::core::ffi::c_void);
    return OK;
}
unsafe extern "C" fn cmdline_pum_create(
    mut ccline: *mut CmdlineInfo,
    mut xp: *mut expand_T,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut numMatches: ::core::ffi::c_int,
    mut showtail: bool,
    mut noselect: bool,
) {
    '_c2rust_label: {
        if numMatches >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"numMatches >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                389 as ::core::ffi::c_uint,
                b"void cmdline_pum_create(CmdlineInfo *, expand_T *, char **, int, _Bool, _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    compl_match_array =
        xmalloc(::core::mem::size_of::<pumitem_T>().wrapping_mul(numMatches as size_t))
            as *mut pumitem_T;
    compl_match_arraysize = numMatches;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < numMatches {
        *compl_match_array.offset(i as isize) = pumitem_T {
            pum_text: if showtail as ::core::ffi::c_int != 0 {
                showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
            } else {
                *matches.offset(i as isize)
            },
            pum_kind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pum_extra: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pum_info: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            pum_cpt_source_idx: 0,
            pum_user_abbr_hlattr: -1 as ::core::ffi::c_int,
            pum_user_kind_hlattr: -1 as ::core::ffi::c_int,
        };
        i += 1;
    }
    let mut endpos: *mut ::core::ffi::c_char = if showtail as ::core::ffi::c_int != 0 {
        showmatches_gettail((*xp).xp_pattern, noselect)
    } else {
        (*xp).xp_pattern
    };
    if ui_has(kUICmdline) as ::core::ffi::c_int != 0 && cmdline_win.is_null() {
        compl_startcol = endpos.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int;
    } else {
        compl_startcol = cmd_screencol(endpos.offset_from((*ccline).cmdbuff) as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_pum_display(mut changed_array: bool) {
    pum_display(
        compl_match_array,
        compl_match_arraysize,
        compl_selected,
        changed_array,
        compl_startcol,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_pum_active() -> bool {
    return pum_visible() as ::core::ffi::c_int != 0 && !compl_match_array.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_pum_remove(mut defer_redraw: bool) {
    pum_undisplay(!defer_redraw);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut compl_match_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    compl_match_arraysize = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_pum_cleanup(mut cclp: *mut CmdlineInfo) {
    cmdline_pum_remove(false_0 != 0);
    wildmenu_cleanup(cclp);
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_compl_pattern() -> *mut ::core::ffi::c_char {
    let mut xp: *mut expand_T = (*get_cmdline_info()).xpc;
    return if xp.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        (*xp).xp_orig
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmdline_compl_is_fuzzy() -> bool {
    let mut xp: *mut expand_T = (*get_cmdline_info()).xpc;
    return !xp.is_null() && cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn cmdline_compl_use_pum(mut need_wildmenu: bool) -> bool {
    return need_wildmenu as ::core::ffi::c_int != 0
        && wop_flags & kOptWopFlagPum as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && !(ui_has(kUICmdline) as ::core::ffi::c_int != 0 && cmdline_win.is_null())
        || ui_has(kUIWildmenu) as ::core::ffi::c_int != 0
        || ui_has(kUICmdline) as ::core::ffi::c_int != 0
            && ui_has(kUIPopupmenu) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn skip_wildmenu_char(
    mut xp: *mut expand_T,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if rem_backslash(s) as ::core::ffi::c_int != 0
        && (*xp).xp_context != EXPAND_HELP as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int
        || ((*xp).xp_context == EXPAND_MENUS as ::core::ffi::c_int
            || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
            && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\t' as ::core::ffi::c_int
                || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL)
    {
        if (*xp).xp_shell as ::core::ffi::c_int != 0
            && csh_like_shell() != 0
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
            && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '!' as ::core::ffi::c_int
        {
            return 2 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn wildmenu_match_len(
    mut xp: *mut expand_T,
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut emenu: ::core::ffi::c_int = ((*xp).xp_context == EXPAND_MENUS as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if emenu != 0 && menu_is_separator(s) as ::core::ffi::c_int != 0 {
        return 1 as ::core::ffi::c_int;
    }
    while *s as ::core::ffi::c_int != NUL {
        s = s.offset(skip_wildmenu_char(xp, s) as isize);
        len += ptr2cells(s);
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    return len;
}
unsafe extern "C" fn redraw_wildmenu(
    mut xp: *mut expand_T,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut match_0: ::core::ffi::c_int,
    mut showtail: bool,
) {
    let mut highlight: bool = true_0 != 0;
    let mut selstart: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut selstart_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut selend: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut first_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut add_left: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut l: ::core::ffi::c_int = 0;
    if matches.is_null() {
        return;
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(
        (Columns as size_t)
            .wrapping_mul(MB_MAXBYTES as ::core::ffi::c_int as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    if match_0 == -1 as ::core::ffi::c_int {
        match_0 = 0 as ::core::ffi::c_int;
        highlight = false_0 != 0;
    }
    let mut clen: ::core::ffi::c_int = wildmenu_match_len(
        xp,
        if showtail as ::core::ffi::c_int != 0 {
            showmatches_gettail(*matches.offset(match_0 as isize), false_0 != 0)
        } else {
            *matches.offset(match_0 as isize)
        },
    ) + 3 as ::core::ffi::c_int;
    if match_0 == 0 as ::core::ffi::c_int {
        first_match = 0 as ::core::ffi::c_int;
    } else if match_0 < first_match {
        first_match = match_0;
        add_left = true_0 != 0;
    } else {
        i = first_match;
        while i < match_0 {
            clen += wildmenu_match_len(
                xp,
                if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                } else {
                    *matches.offset(i as isize)
                },
            ) + 2 as ::core::ffi::c_int;
            i += 1;
        }
        if first_match > 0 as ::core::ffi::c_int {
            clen += 2 as ::core::ffi::c_int;
        }
        if clen > Columns {
            first_match = match_0;
            clen = 2 as ::core::ffi::c_int;
            i = match_0;
            while i < num_matches {
                clen += wildmenu_match_len(
                    xp,
                    if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                    } else {
                        *matches.offset(i as isize)
                    },
                ) + 2 as ::core::ffi::c_int;
                if clen >= Columns {
                    break;
                }
                i += 1;
            }
            if i == num_matches {
                add_left = true_0 != 0;
            }
        }
    }
    if add_left {
        while first_match > 0 as ::core::ffi::c_int {
            clen += wildmenu_match_len(
                xp,
                if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(
                        *matches.offset((first_match - 1 as ::core::ffi::c_int) as isize),
                        false_0 != 0,
                    )
                } else {
                    *matches.offset((first_match - 1 as ::core::ffi::c_int) as isize)
                },
            ) + 2 as ::core::ffi::c_int;
            if clen >= Columns {
                break;
            }
            first_match -= 1;
        }
    }
    let mut len: ::core::ffi::c_int = 0;
    let mut group: hlf_T = HLF_NONE;
    let mut fillchar: schar_T = fillchar_status(&raw mut group, curwin);
    let mut attr: ::core::ffi::c_int = win_hl_attr(curwin, group as ::core::ffi::c_int);
    if first_match == 0 as ::core::ffi::c_int {
        *buf = NUL as ::core::ffi::c_char;
        len = 0 as ::core::ffi::c_int;
    } else {
        strcpy(
            buf,
            b"< \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        len = 2 as ::core::ffi::c_int;
    }
    clen = len;
    i = first_match;
    while (clen
        + wildmenu_match_len(
            xp,
            if showtail as ::core::ffi::c_int != 0 {
                showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
            } else {
                *matches.offset(i as isize)
            },
        )
        + 2 as ::core::ffi::c_int)
        < Columns
    {
        if i == match_0 {
            selstart = buf.offset(len as isize);
            selstart_col = clen;
        }
        let mut s: *mut ::core::ffi::c_char = if showtail as ::core::ffi::c_int != 0 {
            showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
        } else {
            *matches.offset(i as isize)
        };
        let mut emenu: ::core::ffi::c_int = ((*xp).xp_context == EXPAND_MENUS as ::core::ffi::c_int
            || (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if emenu != 0 && menu_is_separator(s) as ::core::ffi::c_int != 0 {
            strcpy(
                buf.offset(len as isize),
                transchar('|' as ::core::ffi::c_int),
            );
            l = strlen(buf.offset(len as isize)) as ::core::ffi::c_int;
            len += l;
            clen += l;
        } else {
            while *s as ::core::ffi::c_int != NUL {
                s = s.offset(skip_wildmenu_char(xp, s) as isize);
                clen += ptr2cells(s);
                l = utfc_ptr2len(s);
                if l > 1 as ::core::ffi::c_int {
                    strncpy(buf.offset(len as isize), s, l as size_t);
                    s = s.offset((l - 1 as ::core::ffi::c_int) as isize);
                    len += l;
                } else {
                    strcpy(
                        buf.offset(len as isize),
                        transchar_byte(*s as uint8_t as ::core::ffi::c_int),
                    );
                    len += strlen(buf.offset(len as isize)) as ::core::ffi::c_int;
                }
                s = s.offset(1);
            }
        }
        if i == match_0 {
            selend = buf.offset(len as isize);
        }
        let c2rust_fresh3 = len;
        len = len + 1;
        *buf.offset(c2rust_fresh3 as isize) = ' ' as ::core::ffi::c_char;
        let c2rust_fresh4 = len;
        len = len + 1;
        *buf.offset(c2rust_fresh4 as isize) = ' ' as ::core::ffi::c_char;
        clen += 2 as ::core::ffi::c_int;
        i += 1;
        if i == num_matches {
            break;
        }
    }
    if i != num_matches {
        let c2rust_fresh5 = len;
        len = len + 1;
        *buf.offset(c2rust_fresh5 as isize) = '>' as ::core::ffi::c_char;
        clen += 1;
    }
    *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
    let mut row: ::core::ffi::c_int = cmdline_row - 1 as ::core::ffi::c_int;
    if row >= 0 as ::core::ffi::c_int {
        if wild_menu_showing == 0 as ::core::ffi::c_int {
            if msg_scrolled > 0 as ::core::ffi::c_int {
                if cmdline_row == Rows - 1 as ::core::ffi::c_int {
                    msg_scroll_up(false_0 != 0, false_0 != 0);
                    msg_scrolled += 1;
                } else {
                    cmdline_row += 1;
                    row += 1;
                }
                wild_menu_showing = WM_SCROLLED as ::core::ffi::c_int;
            } else {
                if (*lastwin).w_status_height == 0 as ::core::ffi::c_int
                    && global_stl_height() == 0 as ::core::ffi::c_int
                {
                    save_p_ls = p_ls as ::core::ffi::c_int;
                    save_p_wmh = p_wmh as ::core::ffi::c_int;
                    p_ls = 2 as OptInt;
                    p_wmh = 0 as OptInt;
                    last_status(false_0 != 0);
                }
                wild_menu_showing = WM_SHOWN as ::core::ffi::c_int;
            }
        }
        grid_line_start(
            if wild_menu_showing == WM_SCROLLED as ::core::ffi::c_int {
                &raw mut msg_grid_adj
            } else {
                &raw mut default_gridview
            },
            row,
        );
        grid_line_puts(0 as ::core::ffi::c_int, buf, -1 as ::core::ffi::c_int, attr);
        if !selstart.is_null() && highlight as ::core::ffi::c_int != 0 {
            *selend = NUL as ::core::ffi::c_char;
            grid_line_puts(
                selstart_col,
                selstart,
                -1 as ::core::ffi::c_int,
                *hl_attr_active.offset(HLF_WM as ::core::ffi::c_int as isize),
            );
        }
        grid_line_fill(clen, Columns, fillchar, attr);
        grid_line_flush();
    }
    win_redraw_last_status(topframe);
    xfree(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_next_or_prev_match(
    mut mode: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> *mut ::core::ffi::c_char {
    if (*xp).xp_numfiles <= 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut findex: ::core::ffi::c_int = (*xp).xp_selected;
    if mode == WILD_PREV as ::core::ffi::c_int {
        if findex == -1 as ::core::ffi::c_int {
            findex = (*xp).xp_numfiles;
        }
        findex -= 1;
    } else if mode == WILD_NEXT as ::core::ffi::c_int {
        findex += 1;
    } else if mode == WILD_PAGEUP as ::core::ffi::c_int
        || mode == WILD_PAGEDOWN as ::core::ffi::c_int
    {
        let mut ht: ::core::ffi::c_int = pum_get_height();
        if ht > 3 as ::core::ffi::c_int {
            ht -= 2 as ::core::ffi::c_int;
        }
        if mode == WILD_PAGEUP as ::core::ffi::c_int {
            if findex == 0 as ::core::ffi::c_int {
                findex = -1 as ::core::ffi::c_int;
            } else if findex < 0 as ::core::ffi::c_int {
                findex = (*xp).xp_numfiles - 1 as ::core::ffi::c_int;
            } else {
                findex = if findex - ht > 0 as ::core::ffi::c_int {
                    findex - ht
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        } else if findex == (*xp).xp_numfiles - 1 as ::core::ffi::c_int {
            findex = -1 as ::core::ffi::c_int;
        } else if findex < 0 as ::core::ffi::c_int {
            findex = 0 as ::core::ffi::c_int;
        } else {
            findex = if findex + ht < (*xp).xp_numfiles - 1 as ::core::ffi::c_int {
                findex + ht
            } else {
                (*xp).xp_numfiles - 1 as ::core::ffi::c_int
            };
        }
    } else {
        '_c2rust_label: {
            if pum_want.active {
            } else {
                __assert_fail(
                    b"pum_want.active\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    739 as ::core::ffi::c_uint,
                    b"char *get_next_or_prev_match(int, expand_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        findex = pum_want.item;
    }
    if findex < 0 as ::core::ffi::c_int || findex >= (*xp).xp_numfiles {
        if !(*xp).xp_orig.is_null() {
            findex = -1 as ::core::ffi::c_int;
        } else {
            findex = if findex < 0 as ::core::ffi::c_int {
                (*xp).xp_numfiles - 1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        }
    }
    if p_wmnu != 0 {
        if !compl_match_array.is_null() {
            compl_selected = findex;
            cmdline_pum_display(false_0 != 0);
        } else if cmdline_compl_use_pum(true_0 != 0) {
            cmdline_pum_create(
                get_cmdline_info(),
                xp,
                (*xp).xp_files,
                (*xp).xp_numfiles,
                cmd_showtail,
                false_0 != 0,
            );
            compl_selected = findex;
            pum_clear();
            cmdline_pum_display(true_0 != 0);
        } else {
            redraw_wildmenu(xp, (*xp).xp_numfiles, (*xp).xp_files, findex, cmd_showtail);
        }
    }
    (*xp).xp_selected = findex;
    return xstrdup(if findex == -1 as ::core::ffi::c_int {
        (*xp).xp_orig
    } else {
        *(*xp).xp_files.offset(findex as isize)
    });
}
unsafe extern "C" fn ExpandOne_start(
    mut mode: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut non_suf_match: ::core::ffi::c_int = 0;
    let mut ss: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if ExpandFromContext(
        xp,
        str,
        &raw mut (*xp).xp_files,
        &raw mut (*xp).xp_numfiles,
        options,
    ) != FAIL
    {
        if (*xp).xp_numfiles == 0 as ::core::ffi::c_int {
            if options & WILD_SILENT as ::core::ffi::c_int == 0 {
                semsg(
                    gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                    str,
                );
            }
        } else {
            ExpandEscape(xp, str, (*xp).xp_numfiles, (*xp).xp_files, options);
            if mode != WILD_ALL as ::core::ffi::c_int
                && mode != WILD_ALL_KEEP as ::core::ffi::c_int
                && mode != WILD_LONGEST as ::core::ffi::c_int
            {
                if (*xp).xp_numfiles != 0 {
                    non_suf_match = (*xp).xp_numfiles;
                } else {
                    non_suf_match = 1 as ::core::ffi::c_int;
                }
                if ((*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int)
                    && (*xp).xp_numfiles > 1 as ::core::ffi::c_int
                {
                    non_suf_match = 0 as ::core::ffi::c_int;
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < 2 as ::core::ffi::c_int {
                        if match_suffix(*(*xp).xp_files.offset(i as isize)) {
                            non_suf_match += 1;
                        }
                        i += 1;
                    }
                }
                if non_suf_match != 1 as ::core::ffi::c_int {
                    if options & WILD_SILENT as ::core::ffi::c_int == 0 {
                        emsg(gettext(&raw const e_toomany as *const ::core::ffi::c_char));
                    } else if options & WILD_NO_BEEP as ::core::ffi::c_int == 0 {
                        beep_flush();
                    }
                }
                if !(non_suf_match != 1 as ::core::ffi::c_int
                    && mode == WILD_EXPAND_FREE as ::core::ffi::c_int)
                {
                    ss = xstrdup(*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize));
                }
            }
        }
    }
    return ss;
}
unsafe extern "C" fn find_longest_match(
    mut xp: *mut expand_T,
    mut options: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0 as size_t;
    let mut mb_len: size_t = 0;
    while *(*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize) != 0 {
        mb_len = utfc_ptr2len(
            (*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize),
        ) as size_t;
        let mut c0: ::core::ffi::c_int = utf_ptr2char(
            (*(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize)).offset(len as isize),
        );
        let mut i: ::core::ffi::c_int = 0;
        i = 1 as ::core::ffi::c_int;
        while i < (*xp).xp_numfiles {
            let mut ci: ::core::ffi::c_int =
                utf_ptr2char((*(*xp).xp_files.offset(i as isize)).offset(len as isize));
            if p_fic != 0
                && ((*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int)
            {
                if mb_tolower(c0) != mb_tolower(ci) {
                    break;
                }
            } else if c0 != ci {
                break;
            }
            i += 1;
        }
        if i < (*xp).xp_numfiles {
            if options & WILD_NO_BEEP as ::core::ffi::c_int == 0 {
                vim_beep(kOptBoFlagWildmode as ::core::ffi::c_int as ::core::ffi::c_uint);
            }
            break;
        } else {
            len = len.wrapping_add(mb_len);
        }
    }
    return xmemdupz(
        *(*xp).xp_files.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        len,
    ) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn ExpandOne(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut orig: *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut ss: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut orig_saved: bool = false_0 != 0;
    if mode == WILD_NEXT as ::core::ffi::c_int
        || mode == WILD_PREV as ::core::ffi::c_int
        || mode == WILD_PAGEUP as ::core::ffi::c_int
        || mode == WILD_PAGEDOWN as ::core::ffi::c_int
        || mode == WILD_PUM_WANT as ::core::ffi::c_int
    {
        return get_next_or_prev_match(mode, xp);
    }
    if mode == WILD_CANCEL as ::core::ffi::c_int {
        ss = xstrdup(if !(*xp).xp_orig.is_null() {
            (*xp).xp_orig as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        });
    } else if mode == WILD_APPLY as ::core::ffi::c_int {
        ss = xstrdup(if (*xp).xp_selected == -1 as ::core::ffi::c_int {
            if !(*xp).xp_orig.is_null() {
                (*xp).xp_orig as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else {
            *(*xp).xp_files.offset((*xp).xp_selected as isize) as *const ::core::ffi::c_char
        });
    }
    if (*xp).xp_numfiles != -1 as ::core::ffi::c_int
        && mode != WILD_ALL as ::core::ffi::c_int
        && mode != WILD_LONGEST as ::core::ffi::c_int
    {
        FreeWild((*xp).xp_numfiles, (*xp).xp_files);
        (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*xp).xp_orig as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        if !compl_match_array.is_null() {
            cmdline_pum_remove(false_0 != 0);
        }
    }
    (*xp).xp_selected = if options & WILD_NOSELECT as ::core::ffi::c_int != 0 {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if mode == WILD_FREE as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*xp).xp_numfiles == -1 as ::core::ffi::c_int
        && mode != WILD_APPLY as ::core::ffi::c_int
        && mode != WILD_CANCEL as ::core::ffi::c_int
    {
        xfree((*xp).xp_orig as *mut ::core::ffi::c_void);
        (*xp).xp_orig = orig;
        orig_saved = true_0 != 0;
        ss = ExpandOne_start(mode, xp, str, options);
    }
    if mode == WILD_LONGEST as ::core::ffi::c_int && (*xp).xp_numfiles > 0 as ::core::ffi::c_int {
        ss = find_longest_match(xp, options);
        (*xp).xp_selected = -1 as ::core::ffi::c_int;
    }
    if mode == WILD_ALL as ::core::ffi::c_int
        && (*xp).xp_numfiles > 0 as ::core::ffi::c_int
        && !got_int
    {
        let mut ss_size: size_t = 0 as size_t;
        let mut prefix: *mut ::core::ffi::c_char =
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut suffix: *mut ::core::ffi::c_char =
            (if options & WILD_USE_NL as ::core::ffi::c_int != 0 {
                b"\n\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b" \0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
        let n: ::core::ffi::c_int = (*xp).xp_numfiles - 1 as ::core::ffi::c_int;
        if (*xp).xp_prefix as ::core::ffi::c_uint
            == XP_PREFIX_NO as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prefix = b"no\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            ss_size = ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                .wrapping_sub(1 as usize)
                .wrapping_mul(n as usize) as size_t;
        } else if (*xp).xp_prefix as ::core::ffi::c_uint
            == XP_PREFIX_INV as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prefix = b"inv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            ss_size = ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                .wrapping_sub(1 as usize)
                .wrapping_mul(n as usize) as size_t;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*xp).xp_numfiles {
            ss_size = ss_size
                .wrapping_add(strlen(*(*xp).xp_files.offset(i as isize)).wrapping_add(1 as size_t));
            i += 1;
        }
        ss_size = ss_size.wrapping_add(1);
        ss = xmalloc(ss_size) as *mut ::core::ffi::c_char;
        *ss = NUL as ::core::ffi::c_char;
        let mut ssp: *mut ::core::ffi::c_char = ss;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*xp).xp_numfiles {
            if i_0 > 0 as ::core::ffi::c_int {
                ssp = xstpcpy(ssp, prefix);
            }
            ssp = xstpcpy(ssp, *(*xp).xp_files.offset(i_0 as isize));
            if i_0 < n {
                ssp = xstpcpy(ssp, suffix);
            }
            '_c2rust_label: {
                if ssp < ss.offset(ss_size as isize) {
                } else {
                    __assert_fail(
                        b"ssp < ss + ss_size\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        995 as ::core::ffi::c_uint,
                        b"char *ExpandOne(expand_T *, char *, char *, int, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            i_0 += 1;
        }
    }
    if mode == WILD_EXPAND_FREE as ::core::ffi::c_int || mode == WILD_ALL as ::core::ffi::c_int {
        ExpandCleanup(xp);
    }
    if !orig_saved {
        xfree(orig as *mut ::core::ffi::c_void);
    }
    return ss;
}
#[no_mangle]
pub unsafe extern "C" fn ExpandInit(mut xp: *mut expand_T) {
    memset(
        xp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<expand_T>(),
    );
    (*xp).xp_backslash = XP_BS_NONE as ::core::ffi::c_int;
    (*xp).xp_prefix = XP_PREFIX_NONE;
    (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ExpandCleanup(mut xp: *mut expand_T) {
    if (*xp).xp_numfiles >= 0 as ::core::ffi::c_int {
        FreeWild((*xp).xp_numfiles, (*xp).xp_files);
        (*xp).xp_numfiles = -1 as ::core::ffi::c_int;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*xp).xp_orig as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
}
#[no_mangle]
pub unsafe extern "C" fn clear_cmdline_orig() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut cmdline_orig as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
}
unsafe extern "C" fn showmatches_oneline(
    mut xp: *mut expand_T,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut numMatches: ::core::ffi::c_int,
    mut lines: ::core::ffi::c_int,
    mut linenr: ::core::ffi::c_int,
    mut maxlen: ::core::ffi::c_int,
    mut showtail: bool,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lastlen: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = linenr;
    while j < numMatches {
        if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
            msg_outtrans(
                *matches.offset(j as isize),
                HLF_D as ::core::ffi::c_int,
                false_0 != 0,
            );
            p = (*matches.offset(j as isize))
                .offset(strlen(*matches.offset(j as isize)) as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            msg_advance(maxlen + 1 as ::core::ffi::c_int);
            msg_puts(p);
            msg_advance(maxlen + 3 as ::core::ffi::c_int);
            msg_outtrans_long(
                p.offset(2 as ::core::ffi::c_int as isize),
                HLF_D as ::core::ffi::c_int,
            );
            break;
        } else {
            let mut i: ::core::ffi::c_int = maxlen - lastlen;
            loop {
                i -= 1;
                if i < 0 as ::core::ffi::c_int {
                    break;
                }
                msg_putchar(' ' as ::core::ffi::c_int);
            }
            let mut isdir: bool = false;
            if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int
            {
                if (*xp).xp_numfiles != -1 as ::core::ffi::c_int {
                    let mut exp_path: *mut ::core::ffi::c_char =
                        expand_env_save_opt(*matches.offset(j as isize), true_0 != 0);
                    let mut path: *mut ::core::ffi::c_char = if !exp_path.is_null() {
                        exp_path
                    } else {
                        *matches.offset(j as isize)
                    };
                    let mut halved_slash: *mut ::core::ffi::c_char = backslash_halve_save(path);
                    isdir = os_isdir(halved_slash);
                    xfree(exp_path as *mut ::core::ffi::c_void);
                    if halved_slash != path {
                        xfree(halved_slash as *mut ::core::ffi::c_void);
                    }
                } else {
                    isdir = os_isdir(*matches.offset(j as isize));
                }
                if showtail {
                    p = if showtail as ::core::ffi::c_int != 0 {
                        showmatches_gettail(*matches.offset(j as isize), false_0 != 0)
                    } else {
                        *matches.offset(j as isize)
                    };
                } else {
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        *matches.offset(j as isize),
                        &raw mut NameBuff as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        true_0 != 0,
                    );
                    p = &raw mut NameBuff as *mut ::core::ffi::c_char;
                }
            } else {
                isdir = false_0 != 0;
                p = if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(*matches.offset(j as isize), false_0 != 0)
                } else {
                    *matches.offset(j as isize)
                };
            }
            lastlen = msg_outtrans(
                p,
                if isdir as ::core::ffi::c_int != 0 {
                    HLF_D as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
                false_0 != 0,
            );
            j += lines;
        }
    }
    if msg_col > 0 as ::core::ffi::c_int {
        msg_clr_eos();
        msg_putchar('\n' as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn showmatches(
    mut xp: *mut expand_T,
    mut display_wildmenu: bool,
    mut display_list: bool,
    mut noselect: bool,
) -> ::core::ffi::c_int {
    let ccline: *mut CmdlineInfo = get_cmdline_info();
    let mut numMatches: ::core::ffi::c_int = 0;
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut maxlen: ::core::ffi::c_int = 0;
    let mut lines: ::core::ffi::c_int = 0;
    let mut columns: ::core::ffi::c_int = 0;
    let mut showtail: bool = false;
    if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
        set_expand_context(xp);
        if (*xp).xp_context == EXPAND_LUA as ::core::ffi::c_int {
            nlua_expand_pat(xp);
        }
        let mut retval: ::core::ffi::c_int = expand_cmdline(
            xp,
            (*ccline).cmdbuff,
            (*ccline).cmdpos,
            &raw mut numMatches,
            &raw mut matches,
        );
        if retval != EXPAND_OK as ::core::ffi::c_int {
            return retval;
        }
        showtail = expand_showtail(xp);
    } else {
        numMatches = (*xp).xp_numfiles;
        matches = (*xp).xp_files;
        showtail = cmd_showtail;
    }
    if cmdline_compl_use_pum(display_wildmenu as ::core::ffi::c_int != 0 && !display_list) {
        cmdline_pum_create(ccline, xp, matches, numMatches, showtail, noselect);
        compl_selected = if noselect as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        pum_clear();
        cmdline_pum_display(true_0 != 0);
        return EXPAND_OK as ::core::ffi::c_int;
    }
    if display_list {
        msg_didany = false_0 != 0;
        msg_start();
        if !ui_has(kUIMessages) {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        ui_flush();
        cmdline_row = msg_row;
        msg_didany = false_0 != 0;
        msg_ext_set_kind(b"wildlist\0".as_ptr() as *const ::core::ffi::c_char);
        msg_start();
    }
    if got_int {
        got_int = false_0 != 0;
    } else if display_wildmenu as ::core::ffi::c_int != 0 && !display_list {
        redraw_wildmenu(
            xp,
            numMatches,
            matches,
            if noselect as ::core::ffi::c_int != 0 {
                -1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            showtail,
        );
    } else if display_list {
        maxlen = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < numMatches {
            let mut len: ::core::ffi::c_int = 0;
            if !showtail
                && ((*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
                    || (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int)
            {
                home_replace(
                    ::core::ptr::null::<buf_T>(),
                    *matches.offset(i as isize),
                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    true_0 != 0,
                );
                len = vim_strsize(&raw mut NameBuff as *mut ::core::ffi::c_char);
            } else {
                len = vim_strsize(if showtail as ::core::ffi::c_int != 0 {
                    showmatches_gettail(*matches.offset(i as isize), false_0 != 0)
                } else {
                    *matches.offset(i as isize)
                });
            }
            maxlen = if maxlen > len { maxlen } else { len };
            i += 1;
        }
        if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
            lines = numMatches;
        } else {
            maxlen += 2 as ::core::ffi::c_int;
            columns = (Columns + 2 as ::core::ffi::c_int) / maxlen;
            if columns < 1 as ::core::ffi::c_int {
                columns = 1 as ::core::ffi::c_int;
            }
            lines = (numMatches + columns - 1 as ::core::ffi::c_int) / columns;
        }
        if (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int {
            msg_puts_hl(
                gettext(b"tagname\0".as_ptr() as *const ::core::ffi::c_char),
                HLF_T as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_clr_eos();
            msg_advance(maxlen - 3 as ::core::ffi::c_int);
            msg_puts_hl(
                gettext(b" kind file\n\0".as_ptr() as *const ::core::ffi::c_char),
                HLF_T as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < lines {
            showmatches_oneline(xp, matches, numMatches, lines, i_0, maxlen, showtail);
            if got_int {
                got_int = false_0 != 0;
                break;
            } else {
                i_0 += 1;
            }
        }
        cmdline_row = msg_row;
    }
    if (*xp).xp_numfiles == -1 as ::core::ffi::c_int {
        FreeWild(numMatches, matches);
    }
    return EXPAND_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn showmatches_gettail(
    mut s: *mut ::core::ffi::c_char,
    mut eager: bool,
) -> *mut ::core::ffi::c_char {
    let mut t: *mut ::core::ffi::c_char = s;
    let mut had_sep: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = s;
    while *p as ::core::ffi::c_int != NUL {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            if eager {
                t = p.offset(1 as ::core::ffi::c_int as isize);
            } else {
                had_sep = true_0 != 0;
            }
        } else if had_sep {
            t = p;
            had_sep = false_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return t;
}
unsafe extern "C" fn expand_showtail(mut xp: *mut expand_T) -> bool {
    if (*xp).xp_context != EXPAND_FILES as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_SHELLCMD as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_DIRECTORIES as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    let mut end: *mut ::core::ffi::c_char = path_tail((*xp).xp_pattern);
    if end == (*xp).xp_pattern {
        return false_0 != 0;
    }
    let mut s: *mut ::core::ffi::c_char = (*xp).xp_pattern;
    while s < end {
        if rem_backslash(s) {
            s = s.offset(1);
        } else if !vim_strchr(
            b"*?[\0".as_ptr() as *const ::core::ffi::c_char,
            *s as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            return false_0 != 0;
        }
        s = s.offset(1);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn addstar(
    mut fname: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut context: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if context != EXPAND_FILES as ::core::ffi::c_int
        && context != EXPAND_FILES_IN_PATH as ::core::ffi::c_int
        && context != EXPAND_SHELLCMD as ::core::ffi::c_int
        && context != EXPAND_DIRECTORIES as ::core::ffi::c_int
        && context != EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int
    {
        if context == EXPAND_FINDFUNC as ::core::ffi::c_int
            || context == EXPAND_HELP as ::core::ffi::c_int
            || context == EXPAND_COLORS as ::core::ffi::c_int
            || context == EXPAND_COMPILER as ::core::ffi::c_int
            || context == EXPAND_OWNSYNTAX as ::core::ffi::c_int
            || context == EXPAND_FILETYPE as ::core::ffi::c_int
            || context == EXPAND_KEYMAP as ::core::ffi::c_int
            || context == EXPAND_PACKADD as ::core::ffi::c_int
            || context == EXPAND_RUNTIME as ::core::ffi::c_int
            || (context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int
                || context == EXPAND_TAGS as ::core::ffi::c_int)
                && *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
            || context == EXPAND_CHECKHEALTH as ::core::ffi::c_int
            || context == EXPAND_LSP as ::core::ffi::c_int
            || context == EXPAND_LUA as ::core::ffi::c_int
        {
            retval = xstrnsave(fname, len);
        } else {
            let mut new_len: size_t = len.wrapping_add(2 as size_t);
            let mut i: size_t = 0 as size_t;
            while i < len {
                if *fname.offset(i as isize) as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                    || *fname.offset(i as isize) as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                {
                    new_len = new_len.wrapping_add(1);
                }
                if context == EXPAND_BUFFERS as ::core::ffi::c_int
                    && *fname.offset(i as isize) as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                {
                    new_len = new_len.wrapping_add(1);
                }
                if (context == EXPAND_USER_DEFINED as ::core::ffi::c_int
                    || context == EXPAND_USER_LIST as ::core::ffi::c_int)
                    && *fname.offset(i as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                {
                    new_len = new_len.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
            retval = xmalloc(new_len) as *mut ::core::ffi::c_char;
            *retval.offset(0 as ::core::ffi::c_int as isize) = '^' as ::core::ffi::c_char;
            let mut j: size_t = 1 as size_t;
            let mut i_0: size_t = 0 as size_t;
            while i_0 < len {
                if context != EXPAND_USER_DEFINED as ::core::ffi::c_int
                    && context != EXPAND_USER_LIST as ::core::ffi::c_int
                    && *fname.offset(i_0 as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && {
                        i_0 = i_0.wrapping_add(1);
                        i_0 == len
                    }
                {
                    break;
                }
                's_82: {
                    match *fname.offset(i_0 as isize) as ::core::ffi::c_int {
                        42 => {
                            let c2rust_fresh6 = j;
                            j = j.wrapping_add(1);
                            *retval.offset(c2rust_fresh6 as isize) = '.' as ::core::ffi::c_char;
                        }
                        126 => {
                            let c2rust_fresh7 = j;
                            j = j.wrapping_add(1);
                            *retval.offset(c2rust_fresh7 as isize) = '\\' as ::core::ffi::c_char;
                        }
                        63 => {
                            *retval.offset(j as isize) = '.' as ::core::ffi::c_char;
                            break 's_82;
                        }
                        46 => {
                            if context == EXPAND_BUFFERS as ::core::ffi::c_int {
                                let c2rust_fresh8 = j;
                                j = j.wrapping_add(1);
                                *retval.offset(c2rust_fresh8 as isize) =
                                    '\\' as ::core::ffi::c_char;
                            }
                        }
                        92 => {
                            if context == EXPAND_USER_DEFINED as ::core::ffi::c_int
                                || context == EXPAND_USER_LIST as ::core::ffi::c_int
                            {
                                let c2rust_fresh9 = j;
                                j = j.wrapping_add(1);
                                *retval.offset(c2rust_fresh9 as isize) =
                                    '\\' as ::core::ffi::c_char;
                            }
                        }
                        _ => {}
                    }
                    *retval.offset(j as isize) = *fname.offset(i_0 as isize);
                }
                i_0 = i_0.wrapping_add(1);
                j = j.wrapping_add(1);
            }
            *retval.offset(j as isize) = NUL as ::core::ffi::c_char;
        }
    } else {
        retval = xmalloc(len.wrapping_add(4 as size_t)) as *mut ::core::ffi::c_char;
        xmemcpyz(
            retval as *mut ::core::ffi::c_void,
            fname as *const ::core::ffi::c_void,
            len,
        );
        let mut tail: *mut ::core::ffi::c_char = path_tail(retval);
        let mut ends_in_star: ::core::ffi::c_int = (len > 0 as size_t
            && *retval.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        let mut k: ssize_t = len as ssize_t - 2 as ssize_t;
        while k >= 0 as ssize_t {
            if *retval.offset(k as isize) as ::core::ffi::c_int != '\\' as ::core::ffi::c_int {
                break;
            }
            ends_in_star = (ends_in_star == 0) as ::core::ffi::c_int;
            k -= 1;
        }
        if (*retval as ::core::ffi::c_int != '~' as ::core::ffi::c_int || tail != retval)
            && ends_in_star == 0
            && vim_strchr(tail, '$' as ::core::ffi::c_int).is_null()
            && vim_strchr(retval, '`' as ::core::ffi::c_int).is_null()
        {
            let c2rust_fresh10 = len;
            len = len.wrapping_add(1);
            *retval.offset(c2rust_fresh10 as isize) = '*' as ::core::ffi::c_char;
        } else if len > 0 as size_t
            && *retval.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '$' as ::core::ffi::c_int
        {
            len = len.wrapping_sub(1);
        }
        *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn set_expand_context(mut xp: *mut expand_T) {
    let ccline: *mut CmdlineInfo = get_cmdline_info();
    if ((*ccline).cmdfirstc == '/' as ::core::ffi::c_int
        || (*ccline).cmdfirstc == '?' as ::core::ffi::c_int)
        && may_expand_pattern as ::core::ffi::c_int != 0
    {
        (*xp).xp_context = EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int;
        (*xp).xp_search_dir = (if (*ccline).cmdfirstc == '/' as ::core::ffi::c_int {
            FORWARD as ::core::ffi::c_int
        } else {
            BACKWARD as ::core::ffi::c_int
        }) as Direction;
        (*xp).xp_pattern = (*ccline).cmdbuff;
        (*xp).xp_pattern_len = (*ccline).cmdpos as size_t;
        search_first_line = 0 as ::core::ffi::c_int as linenr_T;
        return;
    }
    if (*ccline).cmdfirstc != ':' as ::core::ffi::c_int
        && (*ccline).cmdfirstc != '>' as ::core::ffi::c_int
        && (*ccline).cmdfirstc != '=' as ::core::ffi::c_int
        && (*ccline).input_fn == 0
    {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        return;
    }
    set_cmd_context(
        xp,
        (*ccline).cmdbuff,
        (*ccline).cmdlen,
        (*ccline).cmdpos,
        true_0,
    );
}
unsafe extern "C" fn set_cmd_index(
    mut cmd: *const ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let fuzzy: bool = cmdline_fuzzy_complete(cmd);
    if !fuzzy
        && p_ic == 0
        && (*cmd as ::core::ffi::c_int == 'k' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 'e' as ::core::ffi::c_int)
    {
        (*eap).cmdidx = CMD_k;
        p = cmd.offset(1 as ::core::ffi::c_int as isize);
    } else {
        p = cmd;
        while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
        {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'p' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'y' as ::core::ffi::c_int
            && p == cmd.offset(2 as ::core::ffi::c_int as isize)
            && *p as ::core::ffi::c_int == '3' as ::core::ffi::c_int
        {
            p = p.offset(1);
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
        if p == cmd
            && !vim_strchr(
                b"@*!=><&~#\0".as_ptr() as *const ::core::ffi::c_char,
                *p as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        let mut len: size_t = p.offset_from(cmd) as size_t;
        if len == 0 as size_t {
            (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        (*eap).cmdidx = excmd_get_cmdidx(cmd, len);
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            >= 'A' as ::core::ffi::c_int
            && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                <= 'Z' as ::core::ffi::c_int
            || fuzzy as ::core::ffi::c_int != 0
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_bang as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
        {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                || *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
        }
    }
    if *p as ::core::ffi::c_int == NUL
        && (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0)
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
        if *cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && !vim_strchr(
                b"cgriI\0".as_ptr() as *const ::core::ffi::c_char,
                *cmd.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            (*eap).cmdidx = CMD_substitute;
            p = cmd.offset(1 as ::core::ffi::c_int as isize);
        } else if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            >= 'A' as ::core::ffi::c_int
            && *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                <= 'Z' as ::core::ffi::c_int
        {
            (*eap).cmd = cmd as *mut ::core::ffi::c_char;
            p = find_ucmd(
                eap,
                p as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                xp,
                complp,
            );
            if p.is_null() {
                (*eap).cmdidx = CMD_SIZE;
            }
        }
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
        (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return p;
}
unsafe extern "C" fn set_context_for_wildcard_arg(
    mut eap: *mut exarg_T,
    mut arg: *const ::core::ffi::c_char,
    mut usefilter: bool,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) {
    let mut in_quote: bool = false_0 != 0;
    let mut bow: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    (*xp).xp_pattern = skipwhite(arg);
    let mut p: *const ::core::ffi::c_char = (*xp).xp_pattern;
    while *p as ::core::ffi::c_int != NUL {
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if c == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else if c == '`' as ::core::ffi::c_int {
            if !in_quote {
                (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
                bow = p.offset(1 as ::core::ffi::c_int as isize);
            }
            in_quote = !in_quote;
        } else if c == '|' as ::core::ffi::c_int
            || c == '\n' as ::core::ffi::c_int
            || c == '"' as ::core::ffi::c_int
            || ascii_iswhite(c) as ::core::ffi::c_int != 0
        {
            len = 0 as size_t;
            while *p as ::core::ffi::c_int != NUL {
                c = utf_ptr2char(p);
                if c == '`' as ::core::ffi::c_int || vim_isfilec_or_wc(c) as ::core::ffi::c_int != 0
                {
                    break;
                }
                len = utfc_ptr2len(p) as size_t;
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
            if in_quote {
                bow = p;
            } else {
                (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
            }
            p = p.offset(-(len as isize));
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    if !bow.is_null() && in_quote as ::core::ffi::c_int != 0 {
        (*xp).xp_pattern = bow as *mut ::core::ffi::c_char;
    }
    (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
    if usefilter as ::core::ffi::c_int != 0
        || !eap.is_null()
            && ((*eap).cmdidx as ::core::ffi::c_int == CMD_bang as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_terminal as ::core::ffi::c_int)
        || *complp == EXPAND_SHELLCMDLINE as ::core::ffi::c_int
    {
        (*xp).xp_shell = true_0 != 0;
        if (*xp).xp_pattern == skipwhite(arg) {
            (*xp).xp_context = EXPAND_SHELLCMD as ::core::ffi::c_int;
        }
    }
    if *(*xp).xp_pattern as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
        p = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_int != NUL {
            if !vim_isIDc(*p as uint8_t as ::core::ffi::c_int) {
                break;
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            (*xp).xp_context = EXPAND_ENV_VARS as ::core::ffi::c_int;
            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
            if *complp != EXPAND_USER_DEFINED as ::core::ffi::c_int
                && *complp != EXPAND_USER_LIST as ::core::ffi::c_int
            {
                *complp = EXPAND_ENV_VARS as ::core::ffi::c_int;
            }
        }
    }
    if *(*xp).xp_pattern as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
        p = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int != '/' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL
            && p > (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_char
            && match_user((*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize))
                >= 1 as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_USER as ::core::ffi::c_int;
            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
        }
    }
}
unsafe extern "C" fn set_context_in_argopt(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = vim_strchr(arg, '=' as ::core::ffi::c_int);
    if p.is_null() {
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    } else {
        (*xp).xp_pattern = p.offset(1 as ::core::ffi::c_int as isize);
    }
    (*xp).xp_context = EXPAND_ARGOPT as ::core::ffi::c_int;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_in_filter_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if *arg as ::core::ffi::c_int != NUL {
        arg = skip_vimgrep_pat(
            arg as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
    }
    if arg.is_null() || *arg as ::core::ffi::c_int == NUL {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return skipwhite(arg);
}
unsafe extern "C" fn set_context_in_match_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if *arg as ::core::ffi::c_int == NUL || ends_excmd(*arg as ::core::ffi::c_int) == 0 {
        set_context_in_echohl_cmd(xp, arg);
        arg = skipwhite(skiptowhite(arg));
        if *arg as ::core::ffi::c_int != NUL {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            arg = skip_regexp(
                (arg as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
                *arg as uint8_t as ::core::ffi::c_int,
                magic_isset() as ::core::ffi::c_int,
            );
        }
    }
    return find_nextcmd(arg);
}
unsafe extern "C" fn find_cmd_after_global_cmd(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let delim: ::core::ffi::c_int = *arg as uint8_t as ::core::ffi::c_int;
    if delim != 0 {
        arg = arg.offset(1);
    }
    while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *arg.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int != delim
    {
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            arg = arg.offset(1);
        }
        arg = arg.offset(1);
    }
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        return arg.offset(1 as ::core::ffi::c_int as isize);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn find_cmd_after_substitute_cmd(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let delim: ::core::ffi::c_int = *arg as uint8_t as ::core::ffi::c_int;
    if delim != 0 {
        arg = arg.offset(1);
        arg = skip_regexp(
            arg as *mut ::core::ffi::c_char,
            delim,
            magic_isset() as ::core::ffi::c_int,
        );
        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delim
        {
            arg = arg.offset(1);
            while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *arg.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    != delim
            {
                if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    arg = arg.offset(1);
                }
                arg = arg.offset(1);
            }
            if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                arg = arg.offset(1);
            }
        }
    }
    while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        && strchr(
            b"|\"#\0".as_ptr() as *const ::core::ffi::c_char,
            *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        )
        .is_null()
    {
        arg = arg.offset(1);
    }
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        return arg;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn find_cmd_after_isearch_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    arg = skipwhite(skipdigits(arg));
    if *arg as ::core::ffi::c_int != '/' as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    arg = arg.offset(1);
    while *arg as ::core::ffi::c_int != 0 && *arg as ::core::ffi::c_int != '/' as ::core::ffi::c_int
    {
        if *arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            arg = arg.offset(1);
        }
        arg = arg.offset(1);
    }
    if *arg != 0 {
        arg = skipwhite(arg.offset(1 as ::core::ffi::c_int as isize));
        if *arg as ::core::ffi::c_int == NUL
            || strchr(
                b"|\"\n\0".as_ptr() as *const ::core::ffi::c_char,
                *arg as ::core::ffi::c_int,
            )
            .is_null()
        {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        } else {
            return arg;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_in_unlet_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    loop {
        (*xp).xp_pattern = strchr(arg, ' ' as ::core::ffi::c_int);
        if (*xp).xp_pattern.is_null() {
            break;
        }
        arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
    }
    (*xp).xp_context = EXPAND_USER_VARS as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    if *(*xp).xp_pattern as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
        (*xp).xp_context = EXPAND_ENV_VARS as ::core::ffi::c_int;
        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_in_lang_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        (*xp).xp_context = EXPAND_LANGUAGE as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    } else if strncmp(
        arg,
        b"messages\0".as_ptr() as *const ::core::ffi::c_char,
        p.offset_from(arg) as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            arg,
            b"ctype\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            arg,
            b"time\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            arg,
            b"collate\0".as_ptr() as *const ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_LOCALES as ::core::ffi::c_int;
        (*xp).xp_pattern = skipwhite(p);
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
static mut filetype_expand_what: C2Rust_Unnamed_21 = EXP_FILETYPECMD_ALL;
static mut breakpt_expand_what: C2Rust_Unnamed_20 = EXP_BREAKPT_ADD;
unsafe extern "C" fn set_context_in_breakadd_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
    mut cmdidx: cmdidx_T,
) -> *const ::core::ffi::c_char {
    (*xp).xp_context = EXPAND_BREAKPOINT as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    if cmdidx as ::core::ffi::c_int == CMD_breakadd as ::core::ffi::c_int {
        breakpt_expand_what = EXP_BREAKPT_ADD;
    } else if cmdidx as ::core::ffi::c_int == CMD_breakdel as ::core::ffi::c_int {
        breakpt_expand_what = EXP_BREAKPT_DEL;
    } else {
        breakpt_expand_what = EXP_PROFDEL;
    }
    let mut p: *const ::core::ffi::c_char = skipwhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut subcmd_start: *const ::core::ffi::c_char = p;
    if strncmp(
        b"file \0".as_ptr() as *const ::core::ffi::c_char,
        p,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            b"func \0".as_ptr() as *const ::core::ffi::c_char,
            p,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        p = p.offset(4 as ::core::ffi::c_int as isize);
        p = skipwhite(p);
        if ascii_isdigit(*p as ::core::ffi::c_int) {
            p = skipdigits(p);
            if *p as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
                (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            p = skipwhite(p);
        }
        if strncmp(
            b"file\0".as_ptr() as *const ::core::ffi::c_char,
            subcmd_start,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
        } else {
            (*xp).xp_context = EXPAND_USER_FUNC as ::core::ffi::c_int;
        }
        (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
    } else if strncmp(
        b"expr \0".as_ptr() as *const ::core::ffi::c_char,
        p,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_EXPRESSION as ::core::ffi::c_int;
        (*xp).xp_pattern = skipwhite(p.offset(5 as ::core::ffi::c_int as isize));
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_in_scriptnames_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    (*xp).xp_pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = skipwhite(arg);
    if ascii_isdigit(*p as ::core::ffi::c_int) {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    (*xp).xp_context = EXPAND_SCRIPTNAMES as ::core::ffi::c_int;
    (*xp).xp_pattern = p;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_in_filetype_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    (*xp).xp_context = EXPAND_FILETYPECMD as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    filetype_expand_what = EXP_FILETYPECMD_ALL;
    let mut p: *mut ::core::ffi::c_char = skipwhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut val: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if strncmp(
            p,
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            val |= EXPAND_FILETYPECMD_PLUGIN as ::core::ffi::c_int;
            p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
        } else {
            if strncmp(
                p,
                b"indent\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            val |= EXPAND_FILETYPECMD_INDENT as ::core::ffi::c_int;
            p = skipwhite(p.offset(6 as ::core::ffi::c_int as isize));
        }
    }
    if val & EXPAND_FILETYPECMD_PLUGIN as ::core::ffi::c_int != 0
        && val & EXPAND_FILETYPECMD_INDENT as ::core::ffi::c_int != 0
    {
        filetype_expand_what = EXP_FILETYPECMD_ONOFF;
    } else if val & EXPAND_FILETYPECMD_PLUGIN as ::core::ffi::c_int != 0 {
        filetype_expand_what = EXP_FILETYPECMD_INDENT;
    } else if val & EXPAND_FILETYPECMD_INDENT as ::core::ffi::c_int != 0 {
        filetype_expand_what = EXP_FILETYPECMD_PLUGIN;
    }
    (*xp).xp_pattern = p;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_context_with_pattern(mut xp: *mut expand_T) {
    let mut ccline: *mut CmdlineInfo = get_cmdline_info();
    emsg_off += 1;
    let mut skiplen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dummy: ::core::ffi::c_int = 0;
    let mut patlen: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = parse_pattern_and_range(
        &raw mut pre_incsearch_pos,
        &raw mut dummy,
        &raw mut skiplen,
        &raw mut patlen,
    ) as ::core::ffi::c_int;
    emsg_off -= 1;
    if retval == 0 || (*ccline).cmdpos <= skiplen || (*ccline).cmdpos > skiplen + patlen {
        return;
    }
    (*xp).xp_pattern = (*ccline).cmdbuff.offset(skiplen as isize);
    (*xp).xp_pattern_len = ((*ccline).cmdpos - skiplen) as size_t;
    (*xp).xp_context = EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int;
    (*xp).xp_search_dir = FORWARD;
}
unsafe extern "C" fn set_context_by_cmdname(
    mut cmd: *const ::core::ffi::c_char,
    mut cmdidx: cmdidx_T,
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut context: ::core::ffi::c_int,
    mut forceit: bool,
) -> *const ::core::ffi::c_char {
    let mut nextcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    's_685: {
        match cmdidx as ::core::ffi::c_int {
            158 | 403 | 457 => {
                if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
                    (*xp).xp_context = if *get_findfunc() as ::core::ffi::c_int != NUL {
                        EXPAND_FINDFUNC as ::core::ffi::c_int
                    } else {
                        EXPAND_FILES_IN_PATH as ::core::ffi::c_int
                    };
                }
                break 's_685;
            }
            61 | 71 | 225 | 226 | 448 | 449 => {
                if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
                    (*xp).xp_context = EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int;
                }
                break 's_685;
            }
            176 => {
                (*xp).xp_context = EXPAND_HELP as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            3 | 10 | 26 | 31 | 38 | 40 | 62 | 66 | 97 | 111 | 165 | 164 | 181 | 183 | 209 | 207
            | 206 | 208 | 228 | 230 | 234 | 255 | 298 | 302 | 369 | 374 | 386 | 407 | 453 | 455
            | 484 | 502 | 506 | 507 | 528 => return arg,
            157 => return set_context_in_filter_cmd(xp, arg),
            278 => return set_context_in_match_cmd(xp, arg),
            93 => return set_context_in_user_cmd(xp, arg),
            114 => {
                (*xp).xp_context = EXPAND_USER_COMMANDS as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            170 | 504 => {
                nextcmd = find_cmd_after_global_cmd(arg);
                if nextcmd.is_null() && may_expand_pattern as ::core::ffi::c_int != 0 {
                    set_context_with_pattern(xp);
                }
                return nextcmd;
            }
            550 | 382 => {
                nextcmd = find_cmd_after_substitute_cmd(arg);
                if nextcmd.is_null() && may_expand_pattern as ::core::ffi::c_int != 0 {
                    set_context_with_pattern(xp);
                }
                return nextcmd;
            }
            198 | 131 | 189 | 127 | 188 | 334 | 126 | 199 | 132 => {
                return find_cmd_after_isearch_cmd(xp, arg);
            }
            17 => {
                return set_context_in_autocmd(xp, arg as *mut ::core::ffi::c_char, false_0 != 0);
            }
            128 | 129 => {
                return set_context_in_autocmd(xp, arg as *mut ::core::ffi::c_char, true_0 != 0);
            }
            399 => {
                set_context_in_set_cmd(
                    xp,
                    arg as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
                break 's_685;
            }
            401 => {
                set_context_in_set_cmd(
                    xp,
                    arg as *mut ::core::ffi::c_char,
                    OPT_GLOBAL as ::core::ffi::c_int,
                );
                break 's_685;
            }
            402 => {
                set_context_in_set_cmd(
                    xp,
                    arg as *mut ::core::ffi::c_char,
                    OPT_LOCAL as ::core::ffi::c_int,
                );
                break 's_685;
            }
            451 | 431 | 335 | 262 | 489 | 437 | 343 | 474 | 436 | 338 => {
                if wop_flags & kOptWopFlagTagfile as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                {
                    (*xp).xp_context = EXPAND_TAGS_LISTFILES as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_TAGS as ::core::ffi::c_int;
                }
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            18 => {
                (*xp).xp_context = EXPAND_AUGROUP as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            444 => {
                set_context_in_syntax_cmd(xp, arg);
                break 's_685;
            }
            99 | 231 | 187 | 141 | 525 | 167 | 135 | 139 | 151 | 138 | 136 | 53 | 371 | 64 | 50
            | 70 | 232 | 216 | 238 => {
                set_context_for_expression(xp, arg as *mut ::core::ffi::c_char, cmdidx);
                break 's_685;
            }
            498 => return set_context_in_unlet_cmd(xp, arg),
            168 | 115 => {
                (*xp).xp_context = EXPAND_USER_FUNC as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            137 => {
                set_context_in_echohl_cmd(xp, arg);
                break 's_685;
            }
            180 => {
                set_context_in_highlight_cmd(xp, arg);
                break 's_685;
            }
            406 => {
                set_context_in_sign_cmd(xp, arg as *mut ::core::ffi::c_char);
                break 's_685;
            }
            25 | 42 | 41 => loop {
                (*xp).xp_pattern = strchr(arg, ' ' as ::core::ffi::c_int);
                if (*xp).xp_pattern.is_null() {
                    break;
                }
                arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
            },
            20 | 388 | 321 | 75 => {}
            119 | 122 => {
                (*xp).xp_context = EXPAND_DIFF_BUFFERS as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            -1 | -2 => {
                return set_context_in_user_cmdarg(cmd, arg, argt, context, xp, forceit);
            }
            275 | 297 | 292 | 295 | 513 | 516 | 308 | 312 | 190 | 193 | 81 | 87 | 246 | 249
            | 411 | 416 | 539 | 542 => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut ::core::ffi::c_char,
                    arg as *mut ::core::ffi::c_char,
                    forceit,
                    false_0 != 0,
                    false_0 != 0,
                    cmdidx,
                );
            }
            500 | 305 | 520 | 315 | 200 | 105 | 263 | 439 | 544 => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut ::core::ffi::c_char,
                    arg as *mut ::core::ffi::c_char,
                    forceit,
                    false_0 != 0,
                    true_0 != 0,
                    cmdidx,
                );
            }
            276 | 293 | 514 | 309 | 191 | 82 | 247 | 412 | 540 => {
                (*xp).xp_context = EXPAND_MAPCLEAR as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            1 | 300 | 46 | 88 | 185 | 194 => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut ::core::ffi::c_char,
                    arg as *mut ::core::ffi::c_char,
                    forceit,
                    true_0 != 0,
                    false_0 != 0,
                    cmdidx,
                );
            }
            495 | 106 | 201 => {
                return set_context_in_map_cmd(
                    xp,
                    cmd as *mut ::core::ffi::c_char,
                    arg as *mut ::core::ffi::c_char,
                    forceit,
                    true_0 != 0,
                    true_0 != 0,
                    cmdidx,
                );
            }
            279 | 301 | 501 | 5 | 6 | 19 | 294 | 296 | 306 | 515 | 518 | 521 | 310 | 313 | 316
            | 192 | 195 | 202 | 83 | 89 | 107 | 476 | 477 | 478 | 479 | 490 | 328 | 142 => {
                return set_context_in_menu_cmd(xp, cmd, arg as *mut ::core::ffi::c_char, forceit);
            }
            92 => {
                (*xp).xp_context = EXPAND_COLORS as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            95 => {
                (*xp).xp_context = EXPAND_COMPILER as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            317 => {
                (*xp).xp_context = EXPAND_OWNSYNTAX as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            400 => {
                (*xp).xp_context = EXPAND_FILETYPE as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            319 => {
                (*xp).xp_context = EXPAND_PACKADD as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            376 => {
                set_context_in_runtime_cmd(xp, arg);
                break 's_685;
            }
            215 => return set_context_in_lang_cmd(xp, arg),
            332 => {
                set_context_in_profile_cmd(xp, arg);
                break 's_685;
            }
            73 => {
                (*xp).xp_context = EXPAND_CHECKHEALTH as ::core::ffi::c_int;
                break 's_685;
            }
            271 => {
                (*xp).xp_context = EXPAND_LSP as ::core::ffi::c_int;
                break 's_685;
            }
            370 => {
                (*xp).xp_context = EXPAND_RETAB as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            281 => {
                (*xp).xp_context = EXPAND_MESSAGES as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            182 => {
                (*xp).xp_context = EXPAND_HISTORY as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            445 => {
                (*xp).xp_context = EXPAND_SYNTIME as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            9 => {
                loop {
                    (*xp).xp_pattern = vim_strchr(arg, ' ' as ::core::ffi::c_int);
                    if (*xp).xp_pattern.is_null() {
                        break;
                    }
                    arg = (*xp).xp_pattern.offset(1 as ::core::ffi::c_int as isize);
                }
                (*xp).xp_context = EXPAND_ARGLIST as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                break 's_685;
            }
            35 | 333 | 36 => return set_context_in_breakadd_cmd(xp, arg, cmdidx),
            397 => return set_context_in_scriptnames_cmd(xp, arg),
            156 => return set_context_in_filetype_cmd(xp, arg),
            264 | 552 => {
                (*xp).xp_context = EXPAND_LUA as ::core::ffi::c_int;
                break 's_685;
            }
            _ => {
                break 's_685;
            }
        }
        (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn set_one_cmd_context(
    mut xp: *mut expand_T,
    mut buff: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut len: size_t = 0 as size_t;
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
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut forceit: bool = false_0 != 0;
    let mut usefilter: bool = false_0 != 0;
    ExpandInit(xp);
    (*xp).xp_pattern = buff as *mut ::core::ffi::c_char;
    (*xp).xp_line = buff as *mut ::core::ffi::c_char;
    (*xp).xp_context = EXPAND_COMMANDS as ::core::ffi::c_int;
    ea.argt = 0 as uint32_t;
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    cmd = buff;
    while !vim_strchr(
        b" \t:|\0".as_ptr() as *const ::core::ffi::c_char,
        *cmd as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        cmd = cmd.offset(1);
    }
    (*xp).xp_pattern = cmd as *mut ::core::ffi::c_char;
    if *cmd as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if *cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    cmd = skip_range(cmd, &raw mut (*xp).xp_context);
    (*xp).xp_pattern = cmd as *mut ::core::ffi::c_char;
    if *cmd as ::core::ffi::c_int == NUL {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if *cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if *cmd as ::core::ffi::c_int == '|' as ::core::ffi::c_int
        || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
    {
        return cmd.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut p: *const ::core::ffi::c_char = set_cmd_index(cmd, &raw mut ea, xp, &raw mut context);
    if p.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
        forceit = true_0 != 0;
        p = p.offset(1);
    }
    if !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
        ea.argt = excmd_get_argt(ea.cmdidx);
    }
    let mut arg: *const ::core::ffi::c_char = skipwhite(p);
    if ea.argt & EX_ARGOPT as uint32_t != 0 {
        while *arg as ::core::ffi::c_int != NUL
            && strncmp(
                arg,
                b"++\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            p = arg.offset(2 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != 0 && !ascii_isspace(*p as ::core::ffi::c_int) {
                p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
            }
            if *p as ::core::ffi::c_int == NUL {
                if ea.argt & EX_ARGOPT as uint32_t != 0 {
                    return set_context_in_argopt(xp, arg.offset(2 as ::core::ffi::c_int as isize));
                }
            }
            arg = skipwhite(p);
        }
    }
    if ea.cmdidx as ::core::ffi::c_int == CMD_write as ::core::ffi::c_int
        || ea.cmdidx as ::core::ffi::c_int == CMD_update as ::core::ffi::c_int
    {
        if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            arg = arg.offset(1);
            if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                arg = arg.offset(1);
            }
            arg = skipwhite(arg);
        } else if *arg as ::core::ffi::c_int == '!' as ::core::ffi::c_int
            && ea.cmdidx as ::core::ffi::c_int == CMD_write as ::core::ffi::c_int
        {
            arg = arg.offset(1);
            usefilter = true_0 != 0;
        }
    }
    if ea.cmdidx as ::core::ffi::c_int == CMD_read as ::core::ffi::c_int {
        usefilter = forceit;
        if *arg as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
            arg = arg.offset(1);
            usefilter = true_0 != 0;
        }
    }
    if ea.cmdidx as ::core::ffi::c_int == CMD_lshift as ::core::ffi::c_int
        || ea.cmdidx as ::core::ffi::c_int == CMD_rshift as ::core::ffi::c_int
    {
        while *arg as ::core::ffi::c_int == *cmd as ::core::ffi::c_int {
            arg = arg.offset(1);
        }
        arg = skipwhite(arg);
    }
    if ea.argt & EX_CMDARG as uint32_t != 0
        && !usefilter
        && *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        p = arg.offset(1 as ::core::ffi::c_int as isize);
        arg = skip_cmd_arg(arg as *mut ::core::ffi::c_char, false_0 != 0);
        if *arg as ::core::ffi::c_int == NUL {
            return p;
        }
        arg = skipwhite(arg);
    }
    if ea.argt & EX_TRLBAR as uint32_t != 0 && !usefilter {
        p = arg;
        if ea.cmdidx as ::core::ffi::c_int == CMD_redir as ::core::ffi::c_int
            && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '@' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '"' as ::core::ffi::c_int
        {
            p = p.offset(2 as ::core::ffi::c_int as isize);
        }
        while *p != 0 {
            if *p as ::core::ffi::c_int == Ctrl_V {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    p = p.offset(1);
                }
            } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                && ea.argt & EX_NOTRLCOM as uint32_t == 0
                || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                if *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                    {
                        return p.offset(1 as ::core::ffi::c_int as isize);
                    }
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
    }
    if ea.argt & EX_EXTRA as uint32_t == 0
        && *arg as ::core::ffi::c_int != NUL
        && strchr(
            b"|\"\0".as_ptr() as *const ::core::ffi::c_char,
            *arg as ::core::ffi::c_int,
        )
        .is_null()
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    p = buff;
    (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
    len = strlen(buff);
    while *p as ::core::ffi::c_int != 0 && p < buff.offset(len as isize) {
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int || *p as ::core::ffi::c_int == TAB
        {
            p = p.offset(1);
            (*xp).xp_pattern = p as *mut ::core::ffi::c_char;
        } else {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
    }
    if ea.argt & EX_XFILE as uint32_t != 0 {
        set_context_for_wildcard_arg(&raw mut ea, arg, usefilter, xp, &raw mut context);
    }
    return set_context_by_cmdname(cmd, ea.cmdidx, xp, arg, ea.argt, context, forceit);
}
#[no_mangle]
pub unsafe extern "C" fn set_cmd_context(
    mut xp: *mut expand_T,
    mut str: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut use_ccline: ::core::ffi::c_int,
) {
    let ccline: *mut CmdlineInfo = get_cmdline_info();
    let mut old_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    if col < len {
        old_char = *str.offset(col as isize);
    }
    *str.offset(col as isize) = NUL as ::core::ffi::c_char;
    let mut nextcomm: *const ::core::ffi::c_char = str;
    if use_ccline != 0 && (*ccline).cmdfirstc == '=' as ::core::ffi::c_int {
        set_context_for_expression(xp, str, CMD_SIZE);
    } else if use_ccline != 0 && (*ccline).input_fn != 0 {
        (*xp).xp_context = (*ccline).xp_context;
        (*xp).xp_pattern = (*ccline).cmdbuff;
        (*xp).xp_arg = (*ccline).xp_arg;
        if (*xp).xp_context == EXPAND_SHELLCMDLINE as ::core::ffi::c_int {
            let mut context: ::core::ffi::c_int = (*xp).xp_context;
            set_context_for_wildcard_arg(
                ::core::ptr::null_mut::<exarg_T>(),
                (*xp).xp_pattern,
                false_0 != 0,
                xp,
                &raw mut context,
            );
        }
    } else {
        while !nextcomm.is_null() {
            nextcomm = set_one_cmd_context(xp, nextcomm);
        }
    }
    (*xp).xp_line = str;
    (*xp).xp_col = col;
    *str.offset(col as isize) = old_char;
}
#[no_mangle]
pub unsafe extern "C" fn expand_cmdline(
    mut xp: *mut expand_T,
    mut str: *const ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut matchcount: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut file_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut options: ::core::ffi::c_int =
        WILD_ADD_SLASH as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int;
    if (*xp).xp_context == EXPAND_UNSUCCESSFUL as ::core::ffi::c_int {
        beep_flush();
        return EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
    }
    if (*xp).xp_context == EXPAND_NOTHING as ::core::ffi::c_int {
        return EXPAND_NOTHING as ::core::ffi::c_int;
    }
    '_c2rust_label: {
        if str.offset(col as isize).offset_from((*xp).xp_pattern) >= 0 as isize {
        } else {
            __assert_fail(
                b"(str + col) - xp->xp_pattern >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2632 as ::core::ffi::c_uint,
                b"int expand_cmdline(expand_T *, const char *, int, int *, char ***)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*xp).xp_pattern_len = str.offset(col as isize).offset_from((*xp).xp_pattern) as size_t;
    if cmdline_fuzzy_completion_supported(xp) {
        file_str = xstrdup((*xp).xp_pattern);
    } else {
        file_str = addstar((*xp).xp_pattern, (*xp).xp_pattern_len, (*xp).xp_context);
    }
    if p_wic != 0 {
        options += WILD_ICASE as ::core::ffi::c_int;
    }
    if ExpandFromContext(xp, file_str, matches, matchcount, options) == FAIL {
        *matchcount = 0 as ::core::ffi::c_int;
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    xfree(file_str as *mut ::core::ffi::c_void);
    return EXPAND_OK as ::core::ffi::c_int;
}
unsafe extern "C" fn expand_files_and_dirs(
    mut xp: *mut expand_T,
    mut pat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut free_pat: bool = false_0 != 0;
    if (*xp).xp_backslash != XP_BS_NONE as ::core::ffi::c_int {
        free_pat = true_0 != 0;
        let mut pat_len: size_t = strlen(pat);
        pat = xstrnsave(pat, pat_len);
        let mut pat_end: *mut ::core::ffi::c_char = pat.offset(pat_len as isize);
        let mut p: *mut ::core::ffi::c_char = pat;
        while *p as ::core::ffi::c_int != NUL {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                if (*xp).xp_backslash & XP_BS_THREE as ::core::ffi::c_int != 0
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                {
                    let mut from: *mut ::core::ffi::c_char =
                        p.offset(3 as ::core::ffi::c_int as isize);
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        from as *const ::core::ffi::c_void,
                        (pat_end.offset_from(from) as size_t).wrapping_add(1 as size_t),
                    );
                    pat_end = pat_end.offset(-(3 as ::core::ffi::c_int as isize));
                } else if (*xp).xp_backslash & XP_BS_ONE as ::core::ffi::c_int != 0
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                {
                    let mut from_0: *mut ::core::ffi::c_char =
                        p.offset(1 as ::core::ffi::c_int as isize);
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        from_0 as *const ::core::ffi::c_void,
                        (pat_end.offset_from(from_0) as size_t).wrapping_add(1 as size_t),
                    );
                    pat_end = pat_end.offset(-1);
                } else if (*xp).xp_backslash & XP_BS_COMMA as ::core::ffi::c_int != 0 {
                    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int
                    {
                        let mut from_1: *mut ::core::ffi::c_char =
                            p.offset(2 as ::core::ffi::c_int as isize);
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            from_1 as *const ::core::ffi::c_void,
                            (pat_end.offset_from(from_1) as size_t).wrapping_add(1 as size_t),
                        );
                        pat_end = pat_end.offset(-(2 as ::core::ffi::c_int as isize));
                    }
                }
            }
            p = p.offset(1);
        }
    }
    let mut ret: ::core::ffi::c_int = FAIL;
    if (*xp).xp_context == EXPAND_FINDFUNC as ::core::ffi::c_int {
        ret = expand_findfunc(pat, matches, numMatches);
    } else {
        if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int {
            flags |= EW_FILE as ::core::ffi::c_int;
        } else if (*xp).xp_context == EXPAND_FILES_IN_PATH as ::core::ffi::c_int {
            flags |= EW_FILE as ::core::ffi::c_int | EW_PATH as ::core::ffi::c_int;
        } else if (*xp).xp_context == EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int {
            flags = (flags | EW_DIR as ::core::ffi::c_int | EW_CDPATH as ::core::ffi::c_int)
                & !(EW_FILE as ::core::ffi::c_int);
        } else {
            flags = (flags | EW_DIR as ::core::ffi::c_int) & !(EW_FILE as ::core::ffi::c_int);
        }
        if options & WILD_ICASE as ::core::ffi::c_int != 0 {
            flags |= EW_ICASE as ::core::ffi::c_int;
        }
        ret = expand_wildcards_eval(&raw mut pat, numMatches, matches, flags);
    }
    if free_pat {
        xfree(pat as *mut ::core::ffi::c_void);
    }
    return ret;
}
unsafe extern "C" fn get_filetypecmd_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if filetype_expand_what as ::core::ffi::c_uint
        == EXP_FILETYPECMD_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 4 as ::core::ffi::c_int
    {
        let mut opts_all: [*mut ::core::ffi::c_char; 4] = [
            b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_all[idx as usize];
    }
    if filetype_expand_what as ::core::ffi::c_uint
        == EXP_FILETYPECMD_PLUGIN as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 3 as ::core::ffi::c_int
    {
        let mut opts_plugin: [*mut ::core::ffi::c_char; 3] = [
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_plugin[idx as usize];
    }
    if filetype_expand_what as ::core::ffi::c_uint
        == EXP_FILETYPECMD_INDENT as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 3 as ::core::ffi::c_int
    {
        let mut opts_indent: [*mut ::core::ffi::c_char; 3] = [
            b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_indent[idx as usize];
    }
    if filetype_expand_what as ::core::ffi::c_uint
        == EXP_FILETYPECMD_ONOFF as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < 2 as ::core::ffi::c_int
    {
        let mut opts_onoff: [*mut ::core::ffi::c_char; 2] = [
            b"on\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"off\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        return opts_onoff[idx as usize];
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_breakadd_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx >= 0 as ::core::ffi::c_int && idx <= 3 as ::core::ffi::c_int {
        let mut opts: [*mut ::core::ffi::c_char; 4] = [
            b"expr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"func\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"here\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        if breakpt_expand_what as ::core::ffi::c_uint
            == EXP_BREAKPT_ADD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return opts[idx as usize];
        } else if breakpt_expand_what as ::core::ffi::c_uint
            == EXP_BREAKPT_DEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if idx <= 2 as ::core::ffi::c_int {
                return opts[(idx + 1 as ::core::ffi::c_int) as usize];
            }
        } else if idx <= 1 as ::core::ffi::c_int {
            return opts[(idx + 1 as ::core::ffi::c_int) as usize];
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_scriptnames_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if !(idx + 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int
        && idx + 1 as ::core::ffi::c_int <= script_items.ga_len)
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut si: *mut scriptitem_T = *(script_items.ga_data as *mut *mut scriptitem_T)
        .offset((idx + 1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize);
    home_replace(
        ::core::ptr::null::<buf_T>(),
        (*si).sn_name,
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
        true_0 != 0,
    );
    return &raw mut NameBuff as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn get_retab_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"-indentonly\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_messages_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"clear\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_mapclear_arg(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx == 0 as ::core::ffi::c_int {
        return b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_healthcheck_names(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static mut names: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    static mut last_gen: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    if last_gen != get_cmdline_last_prompt_id() || last_gen == 0 as ::core::ffi::c_uint {
        let mut a: Array = ARRAY_DICT_INIT;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut res: Object = nlua_exec(
            String_0 {
                data: b"return vim.health._complete()\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 30]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            a,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        api_free_object(names);
        names = res;
        last_gen = get_cmdline_last_prompt_id();
    }
    if names.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < names.data.array.size as ::core::ffi::c_int
        && (*names.data.array.items.offset(idx as isize)).type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*names.data.array.items.offset(idx as isize))
            .data
            .string
            .data;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_lsp_arg(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static mut names: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    static mut last_xp_line: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut last_gen: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    if last_xp_line.is_null()
        || strcmp(last_xp_line, (*xp).xp_line) != 0 as ::core::ffi::c_int
        || last_gen != get_cmdline_last_prompt_id()
    {
        xfree(last_xp_line as *mut ::core::ffi::c_void);
        last_xp_line = xstrdup((*xp).xp_line);
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let c2rust_fresh0 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string((*xp).xp_line),
            },
        };
        let mut res: Object = nlua_exec(
            String_0 {
                data: b"return require'vim._core.ex_cmd'.lsp_complete(...)\0".as_ptr()
                    as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 51]>().wrapping_sub(1 as size_t),
            },
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        api_free_object(names);
        names = res;
        last_gen = get_cmdline_last_prompt_id();
    }
    if names.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && idx < names.data.array.size as ::core::ffi::c_int
        && (*names.data.array.items.offset(idx as isize)).type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*names.data.array.items.offset(idx as isize))
            .data
            .string
            .data;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn ExpandOther(
    mut pat: *mut ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut rmp: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static mut tab: [expgen; 33] = [
        expgen {
            context: EXPAND_COMMANDS as ::core::ffi::c_int,
            func: Some(
                get_command_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_FILETYPECMD as ::core::ffi::c_int,
            func: Some(
                get_filetypecmd_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_MAPCLEAR as ::core::ffi::c_int,
            func: Some(
                get_mapclear_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_MESSAGES as ::core::ffi::c_int,
            func: Some(
                get_messages_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_HISTORY as ::core::ffi::c_int,
            func: Some(
                get_history_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_COMMANDS as ::core::ffi::c_int,
            func: Some(
                get_user_commands
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_ADDR_TYPE as ::core::ffi::c_int,
            func: Some(
                get_user_cmd_addr_type
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_CMD_FLAGS as ::core::ffi::c_int,
            func: Some(
                get_user_cmd_flags
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_NARGS as ::core::ffi::c_int,
            func: Some(
                get_user_cmd_nargs
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_COMPLETE as ::core::ffi::c_int,
            func: Some(
                get_user_cmd_complete
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_VARS as ::core::ffi::c_int,
            func: Some(
                get_user_var_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_FUNCTIONS as ::core::ffi::c_int,
            func: Some(
                get_function_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER_FUNC as ::core::ffi::c_int,
            func: Some(
                get_user_func_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_EXPRESSION as ::core::ffi::c_int,
            func: Some(
                get_expr_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_MENUS as ::core::ffi::c_int,
            func: Some(
                get_menu_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_MENUNAMES as ::core::ffi::c_int,
            func: Some(
                get_menu_names
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: false_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_SYNTAX as ::core::ffi::c_int,
            func: Some(
                get_syntax_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_SYNTIME as ::core::ffi::c_int,
            func: Some(
                get_syntime_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_HIGHLIGHT as ::core::ffi::c_int,
            func: Some(
                get_highlight_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_EVENTS as ::core::ffi::c_int,
            func: Some(
                expand_get_event_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_AUGROUP as ::core::ffi::c_int,
            func: Some(
                expand_get_augroup_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_SIGN as ::core::ffi::c_int,
            func: Some(
                get_sign_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_PROFILE as ::core::ffi::c_int,
            func: Some(
                get_profile_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_LANGUAGE as ::core::ffi::c_int,
            func: Some(
                get_lang_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_LOCALES as ::core::ffi::c_int,
            func: Some(
                get_locales
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_ENV_VARS as ::core::ffi::c_int,
            func: Some(
                get_env_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_USER as ::core::ffi::c_int,
            func: Some(
                get_users
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_ARGLIST as ::core::ffi::c_int,
            func: Some(
                get_arglist_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_BREAKPOINT as ::core::ffi::c_int,
            func: Some(
                get_breakadd_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_SCRIPTNAMES as ::core::ffi::c_int,
            func: Some(
                get_scriptnames_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_RETAB as ::core::ffi::c_int,
            func: Some(
                get_retab_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: true_0,
        },
        expgen {
            context: EXPAND_CHECKHEALTH as ::core::ffi::c_int,
            func: Some(
                get_healthcheck_names
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
        expgen {
            context: EXPAND_LSP as ::core::ffi::c_int,
            func: Some(
                get_lsp_arg
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ),
            ic: true_0,
            escaped: false_0,
        },
    ];
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ::core::mem::size_of::<[expgen; 33]>()
        .wrapping_div(::core::mem::size_of::<expgen>())
        .wrapping_div(
            (::core::mem::size_of::<[expgen; 33]>().wrapping_rem(::core::mem::size_of::<expgen>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int
    {
        if (*xp).xp_context == tab[i as usize].context {
            if tab[i as usize].ic != 0 {
                (*rmp).rm_ic = true_0 != 0;
            }
            ExpandGeneric(
                pat,
                xp,
                rmp,
                matches,
                numMatches,
                tab[i as usize].func as CompleteListItemGetter,
                tab[i as usize].escaped != 0,
            );
            ret = OK;
            break;
        } else {
            i += 1;
        }
    }
    return ret;
}
unsafe extern "C" fn map_wildopts_to_ewflags(
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = EW_DIR as ::core::ffi::c_int;
    if options & WILD_LIST_NOTFOUND as ::core::ffi::c_int != 0 {
        flags |= EW_NOTFOUND as ::core::ffi::c_int;
    }
    if options & WILD_ADD_SLASH as ::core::ffi::c_int != 0 {
        flags |= EW_ADDSLASH as ::core::ffi::c_int;
    }
    if options & WILD_KEEP_ALL as ::core::ffi::c_int != 0 {
        flags |= EW_KEEPALL as ::core::ffi::c_int;
    }
    if options & WILD_SILENT as ::core::ffi::c_int != 0 {
        flags |= EW_SILENT as ::core::ffi::c_int;
    }
    if options & WILD_NOERROR as ::core::ffi::c_int != 0 {
        flags |= EW_NOERROR as ::core::ffi::c_int;
    }
    if options & WILD_ALLLINKS as ::core::ffi::c_int != 0 {
        flags |= EW_ALLLINKS as ::core::ffi::c_int;
    }
    return flags;
}
unsafe extern "C" fn ExpandFromContext(
    mut xp: *mut expand_T,
    mut pat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false_0 != 0,
    };
    let mut ret: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = map_wildopts_to_ewflags(options);
    let fuzzy: bool = cmdline_fuzzy_complete(pat) as ::core::ffi::c_int != 0
        && cmdline_fuzzy_completion_supported(xp) as ::core::ffi::c_int != 0;
    if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_FILES_IN_PATH as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_FINDFUNC as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_DIRS_IN_CDPATH as ::core::ffi::c_int
    {
        return expand_files_and_dirs(xp, pat, matches, numMatches, flags, options);
    }
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *numMatches = 0 as ::core::ffi::c_int;
    if (*xp).xp_context == EXPAND_HELP as ::core::ffi::c_int {
        if find_help_tags(
            if *pat as ::core::ffi::c_int == NUL {
                b"help\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                pat as *const ::core::ffi::c_char
            },
            numMatches,
            matches,
            false_0 != 0,
        ) == OK
        {
            cleanup_help_tags(*numMatches, *matches);
            return OK;
        }
        return FAIL;
    }
    if (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int {
        expand_shellcmd(pat, matches, numMatches, flags);
        return OK;
    }
    if (*xp).xp_context == EXPAND_OLD_SETTING as ::core::ffi::c_int {
        return ExpandOldSetting(numMatches, matches);
    }
    if (*xp).xp_context == EXPAND_BUFFERS as ::core::ffi::c_int {
        return ExpandBufnames(pat, numMatches, matches, options);
    }
    if (*xp).xp_context == EXPAND_DIFF_BUFFERS as ::core::ffi::c_int {
        return ExpandBufnames(
            pat,
            numMatches,
            matches,
            options | BUF_DIFF_FILTER as ::core::ffi::c_int,
        );
    }
    if (*xp).xp_context == EXPAND_TAGS as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_TAGS_LISTFILES as ::core::ffi::c_int
    {
        return expand_tags(
            (*xp).xp_context == EXPAND_TAGS as ::core::ffi::c_int,
            pat,
            numMatches,
            matches,
        );
    }
    if (*xp).xp_context == EXPAND_COLORS as ::core::ffi::c_int {
        let mut directories: [*mut ::core::ffi::c_char; 2] = [
            b"colors\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        return ExpandRTDir(
            pat,
            DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int,
            numMatches,
            matches,
            &raw mut directories as *mut *mut ::core::ffi::c_char,
        );
    }
    if (*xp).xp_context == EXPAND_COMPILER as ::core::ffi::c_int {
        let mut directories_0: [*mut ::core::ffi::c_char; 2] = [
            b"compiler\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        return ExpandRTDir(
            pat,
            0 as ::core::ffi::c_int,
            numMatches,
            matches,
            &raw mut directories_0 as *mut *mut ::core::ffi::c_char,
        );
    }
    if (*xp).xp_context == EXPAND_OWNSYNTAX as ::core::ffi::c_int {
        let mut directories_1: [*mut ::core::ffi::c_char; 2] = [
            b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        return ExpandRTDir(
            pat,
            0 as ::core::ffi::c_int,
            numMatches,
            matches,
            &raw mut directories_1 as *mut *mut ::core::ffi::c_char,
        );
    }
    if (*xp).xp_context == EXPAND_FILETYPE as ::core::ffi::c_int {
        let mut directories_2: [*mut ::core::ffi::c_char; 4] = [
            b"syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"indent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"ftplugin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        return ExpandRTDir(
            pat,
            0 as ::core::ffi::c_int,
            numMatches,
            matches,
            &raw mut directories_2 as *mut *mut ::core::ffi::c_char,
        );
    }
    if (*xp).xp_context == EXPAND_KEYMAP as ::core::ffi::c_int {
        let mut directories_3: [*mut ::core::ffi::c_char; 2] = [
            b"keymap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        return ExpandRTDir(
            pat,
            0 as ::core::ffi::c_int,
            numMatches,
            matches,
            &raw mut directories_3 as *mut *mut ::core::ffi::c_char,
        );
    }
    if (*xp).xp_context == EXPAND_USER_LIST as ::core::ffi::c_int {
        return ExpandUserList(xp, matches, numMatches);
    }
    if (*xp).xp_context == EXPAND_USER_LUA as ::core::ffi::c_int {
        return ExpandUserLua(xp, numMatches, matches);
    }
    if (*xp).xp_context == EXPAND_PACKADD as ::core::ffi::c_int {
        return ExpandPackAddDir(pat, numMatches, matches);
    }
    if (*xp).xp_context == EXPAND_RUNTIME as ::core::ffi::c_int {
        return expand_runtime_cmd(pat, numMatches, matches);
    }
    if (*xp).xp_context == EXPAND_PATTERN_IN_BUF as ::core::ffi::c_int {
        return expand_pattern_in_buf(pat, (*xp).xp_search_dir, matches, numMatches);
    }
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*xp).xp_context == EXPAND_USER_FUNC as ::core::ffi::c_int
        && strncmp(
            pat,
            b"^s:\0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        let len: size_t = strlen(pat).wrapping_add(20 as size_t);
        tofree = xmalloc(len) as *mut ::core::ffi::c_char;
        snprintf(
            tofree,
            len,
            b"^<SNR>\\d\\+_%s\0".as_ptr() as *const ::core::ffi::c_char,
            pat.offset(3 as ::core::ffi::c_int as isize),
        );
        pat = tofree;
    }
    if (*xp).xp_context == EXPAND_LUA as ::core::ffi::c_int {
        return nlua_expand_get_matches(numMatches, matches);
    }
    if !fuzzy {
        regmatch.regprog = vim_regcomp(
            pat,
            if magic_isset() as ::core::ffi::c_int != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if regmatch.regprog.is_null() {
            xfree(tofree as *mut ::core::ffi::c_void);
            return FAIL;
        }
        regmatch.rm_ic = ignorecase(pat) != 0;
    }
    if (*xp).xp_context == EXPAND_SETTINGS as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_BOOL_SETTINGS as ::core::ffi::c_int
    {
        ret = ExpandSettings(xp, &raw mut regmatch, pat, numMatches, matches, fuzzy);
    } else if (*xp).xp_context == EXPAND_STRING_SETTING as ::core::ffi::c_int {
        ret = ExpandStringSetting(xp, &raw mut regmatch, numMatches, matches);
    } else if (*xp).xp_context == EXPAND_SETTING_SUBTRACT as ::core::ffi::c_int {
        ret = ExpandSettingSubtract(xp, &raw mut regmatch, numMatches, matches);
    } else if (*xp).xp_context == EXPAND_MAPPINGS as ::core::ffi::c_int {
        ret = ExpandMappings(pat, &raw mut regmatch, numMatches, matches);
    } else if (*xp).xp_context == EXPAND_ARGOPT as ::core::ffi::c_int {
        ret = expand_argopt(pat, xp, &raw mut regmatch, matches, numMatches);
    } else if (*xp).xp_context == EXPAND_USER_DEFINED as ::core::ffi::c_int {
        ret = ExpandUserDefined(pat, xp, &raw mut regmatch, matches, numMatches);
    } else {
        ret = ExpandOther(pat, xp, &raw mut regmatch, matches, numMatches);
    }
    if !fuzzy {
        vim_regfree(regmatch.regprog);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ExpandGeneric(
    pat: *const ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut func: CompleteListItemGetter,
    mut escaped: bool,
) {
    let fuzzy: bool = cmdline_fuzzy_complete(pat);
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *numMatches = 0 as ::core::ffi::c_int;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if !fuzzy {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            30 as ::core::ffi::c_int,
        );
    } else {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
            30 as ::core::ffi::c_int,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let mut str: *mut ::core::ffi::c_char = Some(func.expect("non-null function pointer"))
            .expect("non-null function pointer")(
            xp, i
        );
        if str.is_null() {
            break;
        }
        if *str as ::core::ffi::c_int != NUL {
            let mut match_0: bool = false;
            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if *(*xp).xp_pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != NUL
            {
                if !fuzzy {
                    match_0 = vim_regexec(regmatch, str, 0 as colnr_T);
                } else {
                    score = fuzzy_match_str(str, pat);
                    match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                }
            } else {
                match_0 = true_0 != 0;
            }
            if match_0 {
                if escaped {
                    str = vim_strsave_escaped(
                        str,
                        b" \t\\.\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    str = xstrdup(str);
                }
                if fuzzy {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                        fuzmatch_str_T {
                            idx: ga.ga_len,
                            str: str,
                            score: score,
                        };
                    ga.ga_len += 1;
                } else {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = str;
                    ga.ga_len += 1;
                }
                if func
                    == Some(
                        get_menu_names
                            as unsafe extern "C" fn(
                                *mut expand_T,
                                ::core::ffi::c_int,
                            )
                                -> *mut ::core::ffi::c_char,
                    )
                {
                    str = str.offset(strlen(str).wrapping_sub(1 as size_t) as isize);
                    if *str as ::core::ffi::c_int == '\u{1}' as ::core::ffi::c_int {
                        *str = '.' as ::core::ffi::c_char;
                    }
                }
            }
        }
        i += 1;
    }
    if ga.ga_len == 0 as ::core::ffi::c_int {
        return;
    }
    let sort_matches: bool = !fuzzy
        && (*xp).xp_context != EXPAND_MENUNAMES as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_STRING_SETTING as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_MENUS as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_SCRIPTNAMES as ::core::ffi::c_int
        && (*xp).xp_context != EXPAND_ARGOPT as ::core::ffi::c_int;
    let funcsort: bool = (*xp).xp_context == EXPAND_EXPRESSION as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_FUNCTIONS as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_USER_FUNC as ::core::ffi::c_int;
    if sort_matches {
        if funcsort {
            qsort(
                ga.ga_data,
                ga.ga_len as size_t,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
                Some(
                    sort_func_compare
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
        } else {
            sort_strings(ga.ga_data as *mut *mut ::core::ffi::c_char, ga.ga_len);
        }
    }
    if !fuzzy {
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
    } else {
        fuzzymatches_to_strmatches(
            ga.ga_data as *mut fuzmatch_str_T,
            matches,
            ga.ga_len,
            funcsort,
        );
        *numMatches = ga.ga_len;
    }
    reset_expand_highlight();
}
unsafe extern "C" fn expand_shellcmd_onedir(
    mut pathed_pattern: *mut ::core::ffi::c_char,
    mut pathlen: size_t,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut ht: *mut hashtab_T,
    mut gap: *mut garray_T,
) {
    if expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut pathed_pattern,
        numMatches,
        matches,
        flags,
    ) != OK
    {
        return;
    }
    ga_grow(gap, *numMatches);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < *numMatches {
        let mut name: *mut ::core::ffi::c_char = *(*matches).offset(i as isize);
        let mut namelen: size_t = strlen(name);
        if namelen > pathlen {
            let mut hash: hash_T = hash_hash(name.offset(pathlen as isize));
            let mut hi: *mut hashitem_T = hash_lookup(
                ht,
                name.offset(pathlen as isize),
                namelen.wrapping_sub(pathlen),
                hash,
            );
            if (*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed {
                memmove(
                    name as *mut ::core::ffi::c_void,
                    name.offset(pathlen as isize) as *const ::core::ffi::c_void,
                    namelen.wrapping_sub(pathlen).wrapping_add(1 as size_t),
                );
                let c2rust_fresh2 = (*gap).ga_len;
                (*gap).ga_len = (*gap).ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh2 as isize);
                *c2rust_lvalue_ptr = name;
                hash_add_item(ht, hi, name, hash);
                name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        xfree(name as *mut ::core::ffi::c_void);
        i += 1;
    }
    xfree(*matches as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn expand_shellcmd(
    mut filepat: *mut ::core::ffi::c_char,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut flagsarg: ::core::ffi::c_int,
) {
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut flags: ::core::ffi::c_int = flagsarg;
    let mut did_curdir: bool = false_0 != 0;
    let mut patlen: size_t = strlen(filepat);
    let mut pat: *mut ::core::ffi::c_char =
        xmemdupz(filepat as *const ::core::ffi::c_void, patlen) as *mut ::core::ffi::c_char;
    let mut e: *mut ::core::ffi::c_char = pat.offset(patlen as isize);
    let mut s: *mut ::core::ffi::c_char = pat;
    while *s as ::core::ffi::c_int != NUL {
        if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char = s.offset(1 as ::core::ffi::c_int as isize);
            if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                memmove(
                    s as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    (e.offset_from(p) as size_t).wrapping_add(1 as size_t),
                );
                e = e.offset(-1);
            }
        }
        s = s.offset(1);
    }
    patlen = e.offset_from(pat) as size_t;
    flags |= EW_FILE as ::core::ffi::c_int
        | EW_EXEC as ::core::ffi::c_int
        | EW_SHELLCMD as ::core::ffi::c_int;
    let mut mustfree: bool = false_0 != 0;
    if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && (vim_ispathsep(*pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && vim_ispathsep(*pat.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
    {
        path = b".\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        if !path_is_absolute(pat) {
            path = vim_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if path.is_null() {
            path = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            mustfree = true_0 != 0;
        }
    }
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut found_ht: hashtab_T = hashtab_T {
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
    };
    hash_init(&raw mut found_ht);
    let mut s_0: *mut ::core::ffi::c_char = path;
    loop {
        let mut pathlen: size_t = 0;
        let mut seplen: size_t = 0;
        if *s_0 as ::core::ffi::c_int == NUL {
            if did_curdir {
                break;
            }
            did_curdir = true_0 != 0;
            flags |= EW_DIR as ::core::ffi::c_int;
            e = s_0;
            pathlen = 0 as size_t;
            seplen = 0 as size_t;
        } else {
            e = vim_strchr(s_0, ENV_SEPCHAR);
            if e.is_null() {
                e = s_0.offset(strlen(s_0) as isize);
            }
            pathlen = e.offset_from(s_0) as size_t;
            if strncmp(s_0, b".\0".as_ptr() as *const ::core::ffi::c_char, pathlen)
                == 0 as ::core::ffi::c_int
            {
                did_curdir = true_0 != 0;
                flags |= EW_DIR as ::core::ffi::c_int;
            } else {
                flags &= !(EW_DIR as ::core::ffi::c_int);
            }
            seplen = (if after_pathsep(s_0, e) == 0 {
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
            } else {
                0 as usize
            }) as size_t;
        }
        if pathlen
            .wrapping_add(seplen)
            .wrapping_add(patlen)
            .wrapping_add(1 as size_t)
            <= MAXPATHL as size_t
        {
            if pathlen > 0 as size_t {
                xmemcpyz(
                    buf as *mut ::core::ffi::c_void,
                    s_0 as *const ::core::ffi::c_void,
                    pathlen,
                );
                if seplen > 0 as size_t {
                    xmemcpyz(
                        buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                        b"/\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                            .wrapping_sub(1 as size_t),
                    );
                    pathlen = pathlen.wrapping_add(seplen);
                }
            }
            xmemcpyz(
                buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                pat as *const ::core::ffi::c_void,
                patlen,
            );
            expand_shellcmd_onedir(
                buf,
                pathlen,
                matches,
                numMatches,
                flags,
                &raw mut found_ht,
                &raw mut ga,
            );
        }
        if *e as ::core::ffi::c_int != NUL {
            e = e.offset(1);
        }
        s_0 = e;
    }
    *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
    *numMatches = ga.ga_len;
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(pat as *mut ::core::ffi::c_void);
    if mustfree {
        xfree(path as *mut ::core::ffi::c_void);
    }
    hash_clear(&raw mut found_ht);
}
unsafe extern "C" fn call_user_expand_func(
    mut user_expand_func: user_expand_func_T,
    mut xp: *mut expand_T,
) -> *mut ::core::ffi::c_void {
    let ccline: *mut CmdlineInfo = get_cmdline_info();
    let mut keep: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut args: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    let save_current_sctx: sctx_T = current_sctx;
    if (*xp).xp_arg.is_null()
        || *(*xp).xp_arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || (*xp).xp_line.is_null()
    {
        return NULL;
    }
    if !(*ccline).cmdbuff.is_null() {
        keep = *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize);
        *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = 0 as ::core::ffi::c_char;
    }
    let mut pat: *mut ::core::ffi::c_char = xstrnsave((*xp).xp_pattern, (*xp).xp_pattern_len);
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
    args[3 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
    args[1 as ::core::ffi::c_int as usize].vval.v_string = (*xp).xp_line;
    args[2 as ::core::ffi::c_int as usize].vval.v_number = (*xp).xp_col as varnumber_T;
    current_sctx = (*xp).xp_script_ctx;
    let ret: *mut ::core::ffi::c_void = user_expand_func.expect("non-null function pointer")(
        (*xp).xp_arg,
        3 as ::core::ffi::c_int,
        &raw mut args as *mut typval_T,
    );
    current_sctx = save_current_sctx;
    if !(*ccline).cmdbuff.is_null() {
        *(*ccline).cmdbuff.offset((*ccline).cmdlen as isize) = keep;
    }
    xfree(pat as *mut ::core::ffi::c_void);
    return ret;
}
unsafe extern "C" fn ExpandUserDefined(
    pat: *const ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut regmatch: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let fuzzy: bool = cmdline_fuzzy_complete(pat);
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *numMatches = 0 as ::core::ffi::c_int;
    let retstr: *mut ::core::ffi::c_char = call_user_expand_func(
        Some(
            call_func_retstr
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    *mut typval_T,
                ) -> *mut ::core::ffi::c_void,
        ),
        xp,
    ) as *mut ::core::ffi::c_char;
    if retstr.is_null() {
        return FAIL;
    }
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if !fuzzy {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    } else {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    let mut s: *mut ::core::ffi::c_char = retstr;
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while *s as ::core::ffi::c_int != NUL {
        e = vim_strchr(s, '\n' as ::core::ffi::c_int);
        if e.is_null() {
            e = s.offset(strlen(s) as isize);
        }
        let keep: ::core::ffi::c_char = *e;
        *e = NUL as ::core::ffi::c_char;
        let mut match_0: bool = false;
        let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if *(*xp).xp_pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
            if !fuzzy {
                match_0 = vim_regexec(regmatch, s, 0 as colnr_T);
            } else {
                score = fuzzy_match_str(s, pat);
                match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
            }
        } else {
            match_0 = true_0 != 0;
        }
        *e = keep;
        if match_0 {
            let mut p: *mut ::core::ffi::c_char =
                xmemdupz(s as *const ::core::ffi::c_void, e.offset_from(s) as size_t)
                    as *mut ::core::ffi::c_char;
            if !fuzzy {
                ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = p;
                ga.ga_len += 1;
            } else {
                ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) = fuzmatch_str_T {
                    idx: ga.ga_len,
                    str: p,
                    score: score,
                };
                ga.ga_len += 1;
            }
        }
        if *e as ::core::ffi::c_int != NUL {
            e = e.offset(1);
        }
        s = e;
    }
    xfree(retstr as *mut ::core::ffi::c_void);
    if ga.ga_len == 0 as ::core::ffi::c_int {
        return OK;
    }
    if !fuzzy {
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
    } else {
        fuzzymatches_to_strmatches(
            ga.ga_data as *mut fuzmatch_str_T,
            matches,
            ga.ga_len,
            false_0 != 0,
        );
        *numMatches = ga.ga_len;
    }
    return OK;
}
unsafe extern "C" fn process_user_list(
    mut retlist: *mut list_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        3 as ::core::ffi::c_int,
    );
    let l_: *const list_T = retlist;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if !((*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*li).li_tv.vval.v_string.is_null())
            {
                let mut p: *mut ::core::ffi::c_char = xstrdup((*li).li_tv.vval.v_string);
                ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) = p;
                ga.ga_len += 1;
            }
            li = (*li).li_next;
        }
    }
    tv_list_unref(retlist);
    *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
    *numMatches = ga.ga_len;
}
unsafe extern "C" fn ExpandUserList(
    mut xp: *mut expand_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *numMatches = 0 as ::core::ffi::c_int;
    let retlist: *mut list_T = call_user_expand_func(
        Some(
            call_func_retlist
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    *mut typval_T,
                ) -> *mut ::core::ffi::c_void,
        ),
        xp,
    ) as *mut list_T;
    if retlist.is_null() {
        return FAIL;
    }
    process_user_list(retlist, matches, numMatches);
    return OK;
}
unsafe extern "C" fn ExpandUserLua(
    mut xp: *mut expand_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    nlua_call_user_expand_func(xp, &raw mut rettv);
    if rettv.v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(&raw mut rettv);
        return FAIL;
    }
    let retlist: *mut list_T = rettv.vval.v_list;
    process_user_list(retlist, matches, numMatches);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn globpath(
    mut path: *mut ::core::ffi::c_char,
    mut file: *mut ::core::ffi::c_char,
    mut ga: *mut garray_T,
    mut expand_options: ::core::ffi::c_int,
    mut dirs: bool,
) {
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut xpc: expand_T = expand_T {
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
    };
    ExpandInit(&raw mut xpc);
    xpc.xp_context = if dirs as ::core::ffi::c_int != 0 {
        EXPAND_DIRECTORIES as ::core::ffi::c_int
    } else {
        EXPAND_FILES as ::core::ffi::c_int
    };
    let mut filelen: size_t = strlen(file);
    while *path as ::core::ffi::c_int != NUL {
        let mut pathlen: size_t = copy_option_part(
            &raw mut path,
            buf,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut seplen: size_t = if *buf as ::core::ffi::c_int != NUL
            && after_pathsep(buf, buf.offset(pathlen as isize)) == 0
        {
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t)
        } else {
            0 as size_t
        };
        if pathlen
            .wrapping_add(seplen)
            .wrapping_add(filelen)
            .wrapping_add(1 as size_t)
            <= MAXPATHL as size_t
        {
            if seplen > 0 as size_t {
                xmemcpyz(
                    buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                    b"/\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
                );
                pathlen = pathlen.wrapping_add(seplen);
            }
            xmemcpyz(
                buf.offset(pathlen as isize) as *mut ::core::ffi::c_void,
                file as *const ::core::ffi::c_void,
                filelen,
            );
            let mut p: *mut *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
            let mut num_p: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            ExpandFromContext(
                &raw mut xpc,
                buf,
                &raw mut p,
                &raw mut num_p,
                WILD_SILENT as ::core::ffi::c_int | expand_options,
            );
            if num_p > 0 as ::core::ffi::c_int {
                ExpandEscape(
                    &raw mut xpc,
                    buf,
                    num_p,
                    p,
                    WILD_SILENT as ::core::ffi::c_int | expand_options,
                );
                ga_grow(ga, num_p);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < num_p {
                    *((*ga).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset((*ga).ga_len as isize) = *p.offset(i as isize);
                    (*ga).ga_len += 1;
                    i += 1;
                }
                xfree(p as *mut ::core::ffi::c_void);
            }
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wildmenu_translate_key(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut did_wild_list: bool,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = key;
    if cmdline_pum_active() as ::core::ffi::c_int != 0
        || did_wild_list as ::core::ffi::c_int != 0
        || wild_menu_showing != 0
    {
        if c == K_LEFT {
            c = Ctrl_P;
        } else if c == K_RIGHT {
            c = Ctrl_N;
        }
    }
    if (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int
        && (*cclp).cmdpos > 1 as ::core::ffi::c_int
        && *(*cclp)
            .cmdbuff
            .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
        && *(*cclp)
            .cmdbuff
            .offset(((*cclp).cmdpos - 2 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
        && (c == '\n' as ::core::ffi::c_int || c == '\r' as ::core::ffi::c_int || c == K_KENTER)
    {
        c = K_DOWN;
    }
    return c;
}
unsafe extern "C" fn cmdline_del(mut cclp: *mut CmdlineInfo, mut from: ::core::ffi::c_int) {
    '_c2rust_label: {
        if (*cclp).cmdpos <= (*cclp).cmdlen {
        } else {
            __assert_fail(
                b"cclp->cmdpos <= cclp->cmdlen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/cmdexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3650 as ::core::ffi::c_uint,
                b"void cmdline_del(CmdlineInfo *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    memmove(
        (*cclp).cmdbuff.offset(from as isize) as *mut ::core::ffi::c_void,
        (*cclp).cmdbuff.offset((*cclp).cmdpos as isize) as *const ::core::ffi::c_void,
        ((*cclp).cmdlen as size_t)
            .wrapping_sub((*cclp).cmdpos as size_t)
            .wrapping_add(1 as size_t),
    );
    (*cclp).cmdlen -= (*cclp).cmdpos - from;
    (*cclp).cmdpos = from;
}
unsafe extern "C" fn wildmenu_process_key_menunames(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    if key == K_DOWN
        && (*cclp).cmdpos > 0 as ::core::ffi::c_int
        && *(*cclp)
            .cmdbuff
            .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
    {
        key = p_wc as ::core::ffi::c_int;
        KeyTyped = true_0 != 0;
    } else if key == K_UP {
        let mut found: bool = false_0 != 0;
        let mut j: ::core::ffi::c_int =
            (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            j -= 1;
            if j <= 0 as ::core::ffi::c_int {
                break;
            }
            if *(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
                && *(*cclp)
                    .cmdbuff
                    .offset((j - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
            {
                i = j + 1 as ::core::ffi::c_int;
                break;
            } else {
                if !(*(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && *(*cclp)
                        .cmdbuff
                        .offset((j - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int)
                {
                    continue;
                }
                if found {
                    i = j + 1 as ::core::ffi::c_int;
                    break;
                } else {
                    found = true_0 != 0;
                }
            }
        }
        if i > 0 as ::core::ffi::c_int {
            cmdline_del(cclp, i);
        }
        key = p_wc as ::core::ffi::c_int;
        KeyTyped = true_0 != 0;
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return key;
}
unsafe extern "C" fn wildmenu_process_key_filenames(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    let mut upseg: [::core::ffi::c_char; 5] = [0; 5];
    upseg[0 as ::core::ffi::c_int as usize] = PATHSEP as ::core::ffi::c_char;
    upseg[1 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
    upseg[2 as ::core::ffi::c_int as usize] = '.' as ::core::ffi::c_char;
    upseg[3 as ::core::ffi::c_int as usize] = PATHSEP as ::core::ffi::c_char;
    upseg[4 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if key == K_DOWN
        && (*cclp).cmdpos > 0 as ::core::ffi::c_int
        && *(*cclp)
            .cmdbuff
            .offset(((*cclp).cmdpos - 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            == PATHSEP
        && ((*cclp).cmdpos < 3 as ::core::ffi::c_int
            || *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != '.' as ::core::ffi::c_int
            || *(*cclp)
                .cmdbuff
                .offset(((*cclp).cmdpos - 3 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != '.' as ::core::ffi::c_int)
    {
        key = p_wc as ::core::ffi::c_int;
        KeyTyped = true_0 != 0;
    } else if strncmp(
        (*xp).xp_pattern,
        (&raw mut upseg as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
        && key == K_DOWN
    {
        let mut found: bool = false_0 != 0;
        let mut j: ::core::ffi::c_int = (*cclp).cmdpos;
        let mut i: ::core::ffi::c_int =
            (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
        loop {
            j -= 1;
            if j <= i {
                break;
            }
            j -= utf_head_off((*cclp).cmdbuff, (*cclp).cmdbuff.offset(j as isize));
            if !vim_ispathsep(*(*cclp).cmdbuff.offset(j as isize) as ::core::ffi::c_int) {
                continue;
            }
            found = true_0 != 0;
            break;
        }
        if found as ::core::ffi::c_int != 0
            && *(*cclp)
                .cmdbuff
                .offset((j - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && *(*cclp)
                .cmdbuff
                .offset((j - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && (vim_ispathsep(
                *(*cclp)
                    .cmdbuff
                    .offset((j - 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
                || j == i + 2 as ::core::ffi::c_int)
        {
            cmdline_del(cclp, j - 2 as ::core::ffi::c_int);
            key = p_wc as ::core::ffi::c_int;
            KeyTyped = true_0 != 0;
        }
    } else if key == K_UP {
        let mut found_0: bool = false_0 != 0;
        let mut j_0: ::core::ffi::c_int = (*cclp).cmdpos - 1 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int =
            (*xp).xp_pattern.offset_from((*cclp).cmdbuff) as ::core::ffi::c_int;
        loop {
            j_0 -= 1;
            if j_0 <= i_0 {
                break;
            }
            j_0 -= utf_head_off((*cclp).cmdbuff, (*cclp).cmdbuff.offset(j_0 as isize));
            if !vim_ispathsep(*(*cclp).cmdbuff.offset(j_0 as isize) as ::core::ffi::c_int) {
                continue;
            }
            if found_0 {
                i_0 = j_0 + 1 as ::core::ffi::c_int;
                break;
            } else {
                found_0 = true_0 != 0;
            }
        }
        if !found_0 {
            j_0 = i_0;
        } else if strncmp(
            (*cclp).cmdbuff.offset(j_0 as isize),
            &raw mut upseg as *mut ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            j_0 += 4 as ::core::ffi::c_int;
        } else if strncmp(
            (*cclp).cmdbuff.offset(j_0 as isize),
            (&raw mut upseg as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
            && j_0 == i_0
        {
            j_0 += 3 as ::core::ffi::c_int;
        } else {
            j_0 = 0 as ::core::ffi::c_int;
        }
        if j_0 > 0 as ::core::ffi::c_int {
            cmdline_del(cclp, j_0);
            put_on_cmdline(
                (&raw mut upseg as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize),
                3 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else if (*cclp).cmdpos > i_0 {
            cmdline_del(cclp, i_0);
        }
        key = p_wc as ::core::ffi::c_int;
        KeyTyped = true_0 != 0;
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn wildmenu_process_key(
    mut cclp: *mut CmdlineInfo,
    mut key: ::core::ffi::c_int,
    mut xp: *mut expand_T,
) -> ::core::ffi::c_int {
    if (*xp).xp_context == EXPAND_MENUNAMES as ::core::ffi::c_int {
        return wildmenu_process_key_menunames(cclp, key, xp);
    }
    if (*xp).xp_context == EXPAND_FILES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_DIRECTORIES as ::core::ffi::c_int
        || (*xp).xp_context == EXPAND_SHELLCMD as ::core::ffi::c_int
    {
        return wildmenu_process_key_filenames(cclp, key, xp);
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn wildmenu_cleanup(mut cclp: *mut CmdlineInfo) {
    if p_wmnu == 0 || wild_menu_showing == 0 as ::core::ffi::c_int {
        return;
    }
    let skt: bool = KeyTyped;
    let old_RedrawingDisabled: ::core::ffi::c_int = RedrawingDisabled;
    if (*cclp).input_fn != 0 {
        RedrawingDisabled = 0 as ::core::ffi::c_int;
    }
    set_no_hlsearch(true_0 != 0);
    if wild_menu_showing == WM_SCROLLED as ::core::ffi::c_int {
        cmdline_row -= 1;
        redrawcmd();
        wild_menu_showing = 0 as ::core::ffi::c_int;
    } else if save_p_ls != -1 as ::core::ffi::c_int {
        p_ls = save_p_ls as OptInt;
        p_wmh = save_p_wmh as OptInt;
        last_status(false_0 != 0);
        update_screen();
        redrawcmd();
        save_p_ls = -1 as ::core::ffi::c_int;
        wild_menu_showing = 0 as ::core::ffi::c_int;
    } else {
        win_redraw_last_status(topframe);
        wild_menu_showing = 0 as ::core::ffi::c_int;
        redraw_statuslines();
    }
    KeyTyped = skt;
    if (*cclp).input_fn != 0 {
        RedrawingDisabled = old_RedrawingDisabled;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getcompletion(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut xpc: expand_T = expand_T {
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
    };
    let mut filtered: bool = false_0 != 0;
    let mut options: ::core::ffi::c_int = WILD_SILENT as ::core::ffi::c_int
        | WILD_USE_NL as ::core::ffi::c_int
        | WILD_ADD_SLASH as ::core::ffi::c_int
        | WILD_NO_BEEP as ::core::ffi::c_int
        | WILD_HOME_REPLACE as ::core::ffi::c_int;
    if tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let type_0: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        filtered = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) != 0;
    }
    if p_wic != 0 {
        options |= WILD_ICASE as ::core::ffi::c_int;
    }
    if !filtered {
        options |= WILD_KEEP_ALL as ::core::ffi::c_int;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let pattern: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut pattern_start: *const ::core::ffi::c_char = pattern;
    if strcmp(type_0, b"cmdline\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        let cmdline_len: ::core::ffi::c_int = strlen(pattern) as ::core::ffi::c_int;
        set_cmd_context(
            &raw mut xpc,
            pattern as *mut ::core::ffi::c_char,
            cmdline_len,
            cmdline_len,
            false_0,
        );
        pattern_start = xpc.xp_pattern;
        xpc.xp_pattern_len = strlen(xpc.xp_pattern);
        xpc.xp_col = cmdline_len;
    } else {
        ExpandInit(&raw mut xpc);
        xpc.xp_pattern = pattern as *mut ::core::ffi::c_char;
        xpc.xp_pattern_len = strlen(xpc.xp_pattern);
        xpc.xp_line = pattern as *mut ::core::ffi::c_char;
        xpc.xp_context = cmdcomplete_str_to_type(type_0);
        match xpc.xp_context {
            0 => {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    type_0,
                );
                return;
            }
            30 => {
                if strncmp(
                    type_0,
                    b"custom,\0".as_ptr() as *const ::core::ffi::c_char,
                    7 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        type_0,
                    );
                    return;
                }
                xpc.xp_arg =
                    type_0.offset(7 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
            }
            31 => {
                if strncmp(
                    type_0,
                    b"customlist,\0".as_ptr() as *const ::core::ffi::c_char,
                    11 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        type_0,
                    );
                    return;
                }
                xpc.xp_arg =
                    type_0.offset(11 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
            }
            11 => {
                set_context_in_menu_cmd(
                    &raw mut xpc,
                    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
                    xpc.xp_pattern,
                    false_0 != 0,
                );
                xpc.xp_pattern_len = xpc
                    .xp_pattern_len
                    .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
            }
            34 => {
                set_context_in_sign_cmd(&raw mut xpc, xpc.xp_pattern);
                xpc.xp_pattern_len = xpc
                    .xp_pattern_len
                    .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
            }
            51 => {
                set_context_in_runtime_cmd(&raw mut xpc, xpc.xp_pattern);
                xpc.xp_pattern_len = xpc
                    .xp_pattern_len
                    .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
            }
            57 => {
                let mut context: ::core::ffi::c_int = EXPAND_SHELLCMDLINE as ::core::ffi::c_int;
                set_context_for_wildcard_arg(
                    ::core::ptr::null_mut::<exarg_T>(),
                    xpc.xp_pattern,
                    false_0 != 0,
                    &raw mut xpc,
                    &raw mut context,
                );
                xpc.xp_pattern_len = xpc
                    .xp_pattern_len
                    .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
            }
            59 => {
                filetype_expand_what = EXP_FILETYPECMD_ALL;
            }
            _ => {}
        }
    }
    if xpc.xp_context == EXPAND_LUA as ::core::ffi::c_int {
        xpc.xp_col = strlen(xpc.xp_line) as ::core::ffi::c_int;
        nlua_expand_pat(&raw mut xpc);
        xpc.xp_pattern_len = xpc
            .xp_pattern_len
            .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
    }
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cmdline_fuzzy_completion_supported(&raw mut xpc) {
        pat = xmemdupz(
            xpc.xp_pattern as *const ::core::ffi::c_void,
            xpc.xp_pattern_len,
        ) as *mut ::core::ffi::c_char;
    } else {
        pat = addstar(xpc.xp_pattern, xpc.xp_pattern_len, xpc.xp_context);
    }
    ExpandOne(
        &raw mut xpc,
        pat,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        options,
        WILD_ALL_KEEP as ::core::ffi::c_int,
    );
    tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < xpc.xp_numfiles {
        tv_list_append_string(
            (*rettv).vval.v_list,
            *xpc.xp_files.offset(i as isize),
            -1 as ssize_t,
        );
        i += 1;
    }
    xfree(pat as *mut ::core::ffi::c_void);
    ExpandCleanup(&raw mut xpc);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcompletiontype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut pat: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut xpc: expand_T = expand_T {
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
    };
    ExpandInit(&raw mut xpc);
    let mut cmdline_len: ::core::ffi::c_int = strlen(pat) as ::core::ffi::c_int;
    set_cmd_context(
        &raw mut xpc,
        pat as *mut ::core::ffi::c_char,
        cmdline_len,
        cmdline_len,
        false_0,
    );
    (*rettv).vval.v_string = cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg);
    ExpandCleanup(&raw mut xpc);
}
#[no_mangle]
pub unsafe extern "C" fn f_cmdcomplete_info(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ccline: *mut CmdlineInfo = get_cmdline_info();
    tv_dict_alloc_ret(rettv);
    if ccline.is_null() || (*ccline).xpc.is_null() || (*(*ccline).xpc).xp_files.is_null() {
        return;
    }
    let mut retdict: *mut dict_T = (*rettv).vval.v_dict;
    let mut ret: ::core::ffi::c_int = tv_dict_add_str(
        retdict,
        b"cmdline_orig\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
        cmdline_orig,
    );
    if ret == OK {
        ret = tv_dict_add_nr(
            retdict,
            b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            pum_visible() as varnumber_T,
        );
    }
    if ret == OK {
        ret = tv_dict_add_nr(
            retdict,
            b"selected\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*(*ccline).xpc).xp_selected as varnumber_T,
        );
    }
    if ret == OK {
        let mut li: *mut list_T = tv_list_alloc((*(*ccline).xpc).xp_numfiles as ptrdiff_t);
        ret = tv_dict_add_list(
            retdict,
            b"matches\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            li,
        );
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while ret == OK && idx < (*(*ccline).xpc).xp_numfiles {
            tv_list_append_string(
                li,
                *(*(*ccline).xpc).xp_files.offset(idx as isize),
                -1 as ssize_t,
            );
            idx += 1;
        }
    }
}
unsafe extern "C" fn copy_substring_from_pos(
    mut start: *mut pos_T,
    mut end: *mut pos_T,
    mut match_0: *mut *mut ::core::ffi::c_char,
    mut match_end: *mut pos_T,
) -> ::core::ffi::c_int {
    let mut exacttext: bool =
        wop_flags & kOptWopFlagExacttext as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
    if (*start).lnum > (*end).lnum || (*start).lnum == (*end).lnum && (*start).col >= (*end).col {
        return FAIL;
    }
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        128 as ::core::ffi::c_int,
    );
    let mut start_line: *mut ::core::ffi::c_char = ml_get((*start).lnum);
    let mut start_ptr: *mut ::core::ffi::c_char = start_line.offset((*start).col as isize);
    let mut is_single_line: bool = (*start).lnum == (*end).lnum;
    let mut segment_len: ::core::ffi::c_int = if is_single_line as ::core::ffi::c_int != 0 {
        (*end).col - (*start).col
    } else {
        ml_get_len((*start).lnum) - (*start).col
    };
    ga_grow(&raw mut ga, segment_len + 2 as ::core::ffi::c_int);
    ga_concat_len(&raw mut ga, start_ptr, segment_len as size_t);
    if !is_single_line {
        if exacttext {
            ga_concat_len(
                &raw mut ga,
                b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        } else {
            ga_append(&raw mut ga, '\n' as uint8_t);
        }
    }
    if !is_single_line {
        let mut lnum: linenr_T = (*start).lnum + 1 as linenr_T;
        while lnum < (*end).lnum {
            let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
            let mut linelen: ::core::ffi::c_int = ml_get_len(lnum);
            ga_grow(&raw mut ga, linelen + 2 as ::core::ffi::c_int);
            ga_concat_len(&raw mut ga, line, linelen as size_t);
            if exacttext {
                ga_concat_len(
                    &raw mut ga,
                    b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                );
            } else {
                ga_append(&raw mut ga, '\n' as uint8_t);
            }
            lnum += 1;
        }
    }
    let mut end_line: *mut ::core::ffi::c_char = ml_get((*end).lnum);
    let mut word_end: *mut ::core::ffi::c_char =
        find_word_end(end_line.offset((*end).col as isize));
    segment_len = word_end.offset_from(end_line) as ::core::ffi::c_int;
    ga_grow(&raw mut ga, segment_len);
    ga_concat_len(
        &raw mut ga,
        end_line.offset(
            (if is_single_line as ::core::ffi::c_int != 0 {
                (*end).col as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as isize,
        ),
        (segment_len
            - (if is_single_line as ::core::ffi::c_int != 0 {
                (*end).col as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })) as size_t,
    );
    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
    ga_append(&raw mut ga, NUL as uint8_t);
    *match_0 = ga.ga_data as *mut ::core::ffi::c_char;
    (*match_end).lnum = (*end).lnum;
    (*match_end).col = segment_len as colnr_T;
    return OK;
}
unsafe extern "C" fn is_regex_match(
    mut pat: *mut ::core::ffi::c_char,
    mut str: *mut ::core::ffi::c_char,
) -> bool {
    if strcmp(pat, str) == 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    emsg_off += 1;
    msg_silent += 1;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    emsg_off -= 1;
    msg_silent -= 1;
    if regmatch.regprog.is_null() {
        return false_0 != 0;
    }
    regmatch.rm_ic = p_ic != 0;
    if p_ic != 0 && p_scs != 0 {
        regmatch.rm_ic = !pat_has_uppercase(pat);
    }
    emsg_off += 1;
    msg_silent += 1;
    let mut result: bool = vim_regexec_nl(&raw mut regmatch, str, 0 as ::core::ffi::c_int);
    emsg_off -= 1;
    msg_silent -= 1;
    vim_regfree(regmatch.regprog);
    return result;
}
unsafe extern "C" fn concat_pattern_with_buffer_match(
    mut pat: *mut ::core::ffi::c_char,
    mut pat_len: ::core::ffi::c_int,
    mut end_match_pos: *mut pos_T,
    mut lowercase: bool,
) -> *mut ::core::ffi::c_char {
    let mut line: *mut ::core::ffi::c_char = ml_get((*end_match_pos).lnum);
    let mut word_end: *mut ::core::ffi::c_char =
        find_word_end(line.offset((*end_match_pos).col as isize));
    let mut match_len: ::core::ffi::c_int =
        word_end.offset_from(line.offset((*end_match_pos).col as isize)) as ::core::ffi::c_int;
    let mut match_0: *mut ::core::ffi::c_char = xmalloc(
        (match_len as size_t)
            .wrapping_add(pat_len as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    memmove(
        match_0 as *mut ::core::ffi::c_void,
        pat as *const ::core::ffi::c_void,
        pat_len as size_t,
    );
    if match_len > 0 as ::core::ffi::c_int {
        if lowercase {
            let mut mword: *mut ::core::ffi::c_char = xstrnsave(
                line.offset((*end_match_pos).col as isize),
                match_len as size_t,
            );
            let mut lower: *mut ::core::ffi::c_char = strcase_save(mword, false_0 != 0);
            xfree(mword as *mut ::core::ffi::c_void);
            memmove(
                match_0.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                lower as *const ::core::ffi::c_void,
                match_len as size_t,
            );
            xfree(lower as *mut ::core::ffi::c_void);
        } else {
            memmove(
                match_0.offset(pat_len as isize) as *mut ::core::ffi::c_void,
                line.offset((*end_match_pos).col as isize) as *const ::core::ffi::c_void,
                match_len as size_t,
            );
        }
    }
    *match_0.offset((pat_len + match_len) as isize) = NUL as ::core::ffi::c_char;
    return match_0;
}
unsafe extern "C" fn expand_pattern_in_buf(
    mut pat: *mut ::core::ffi::c_char,
    mut dir: Direction,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut exacttext: bool =
        wop_flags & kOptWopFlagExacttext as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
    let mut has_range: bool = search_first_line != 0 as linenr_T;
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    *numMatches = 0 as ::core::ffi::c_int;
    if pat.is_null() || *pat as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    let mut pat_len: ::core::ffi::c_int = strlen(pat) as ::core::ffi::c_int;
    let mut cur_match_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    let mut prev_match_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    if has_range {
        cur_match_pos.lnum = search_first_line;
    } else {
        cur_match_pos = pre_incsearch_pos;
    }
    let mut search_flags: ::core::ffi::c_int = SEARCH_OPT as ::core::ffi::c_int
        | SEARCH_NOOF as ::core::ffi::c_int
        | SEARCH_PEEK as ::core::ffi::c_int
        | SEARCH_NFMSG as ::core::ffi::c_int
        | (if has_range as ::core::ffi::c_int != 0 {
            SEARCH_START as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut end_match_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut word_end_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut looped_around: bool = false_0 != 0;
    let mut compl_started: bool = false_0 != 0;
    let mut match_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut full_match: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_cleanup: {
        loop {
            emsg_off += 1;
            msg_silent += 1;
            let mut found_new_match: ::core::ffi::c_int = searchit(
                ::core::ptr::null_mut::<win_T>(),
                curbuf,
                &raw mut cur_match_pos,
                &raw mut end_match_pos,
                dir,
                pat,
                pat_len as size_t,
                1 as ::core::ffi::c_int,
                search_flags,
                RE_LAST as ::core::ffi::c_int,
                ::core::ptr::null_mut::<searchit_arg_T>(),
            );
            msg_silent -= 1;
            emsg_off -= 1;
            if found_new_match == FAIL {
                break;
            }
            if has_range as ::core::ffi::c_int != 0
                && (cur_match_pos.lnum < search_first_line || cur_match_pos.lnum > search_last_line)
            {
                break;
            }
            if compl_started {
                if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                    && ltoreq(cur_match_pos, prev_match_pos) as ::core::ffi::c_int != 0
                    || dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                        && ltoreq(prev_match_pos, cur_match_pos) as ::core::ffi::c_int != 0
                {
                    if looped_around {
                        break;
                    }
                    looped_around = true_0 != 0;
                }
            }
            compl_started = true_0 != 0;
            prev_match_pos = cur_match_pos;
            if char_avail() as ::core::ffi::c_int != 0 || got_int as ::core::ffi::c_int != 0 {
                if got_int {
                    vpeekc();
                    got_int = false_0 != 0;
                }
                break '_cleanup;
            } else if end_match_pos.lnum > (*curbuf).b_ml.ml_line_count {
                cur_match_pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                cur_match_pos.col = 0 as ::core::ffi::c_int as colnr_T;
                cur_match_pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                if copy_substring_from_pos(
                    &raw mut cur_match_pos,
                    &raw mut end_match_pos,
                    &raw mut full_match,
                    &raw mut word_end_pos,
                ) == 0
                {
                    break;
                }
                if exacttext {
                    match_0 = full_match;
                } else {
                    match_0 = concat_pattern_with_buffer_match(
                        pat,
                        pat_len,
                        &raw mut end_match_pos,
                        false_0 != 0,
                    );
                    if !is_regex_match(match_0, full_match) {
                        xfree(match_0 as *mut ::core::ffi::c_void);
                        match_0 = concat_pattern_with_buffer_match(
                            pat,
                            pat_len,
                            &raw mut end_match_pos,
                            true_0 != 0,
                        );
                        if !is_regex_match(match_0, full_match) {
                            xfree(match_0 as *mut ::core::ffi::c_void);
                            xfree(full_match as *mut ::core::ffi::c_void);
                            continue;
                        }
                    }
                    xfree(full_match as *mut ::core::ffi::c_void);
                }
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < ga.ga_len {
                    if strcmp(
                        match_0,
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                    ) == 0 as ::core::ffi::c_int
                    {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut match_0 as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        *ptr_;
                        break;
                    } else {
                        i += 1;
                    }
                }
                if !match_0.is_null() {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    let c2rust_fresh1 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *(ga.ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh1 as isize);
                    *c2rust_lvalue_ptr = match_0;
                    if ga.ga_len > TAG_MANY as ::core::ffi::c_int {
                        break;
                    }
                }
                if has_range {
                    cur_match_pos = word_end_pos;
                }
            }
        }
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
        return OK;
    }
    ga_clear_strings(&raw mut ga);
    return FAIL;
}
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
pub const K_UP: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('u' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_LEFT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('l' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_RIGHT: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('r' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_KENTER: ::core::ffi::c_int =
    -('K' as ::core::ffi::c_int + (('A' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
