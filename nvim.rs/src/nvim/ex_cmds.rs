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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn log10(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
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
    fn strcoll(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn time(__timer: *mut time_t) -> time_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(
        data: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn check_arg_idx(win: *mut win_T);
    fn do_argfile(eap: *mut exarg_T, argn: ::core::ffi::c_int);
    static mut autocmd_busy: bool;
    static mut au_new_curbuf: bufref_T;
    fn augroup_exists(name: *const ::core::ffi::c_char) -> bool;
    fn do_doautocmd(
        arg_start: *mut ::core::ffi::c_char,
        do_msg: bool,
        did_something: *mut bool,
    ) -> ::core::ffi::c_int;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn apply_autocmds_retval(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
        retval: *mut ::core::ffi::c_int,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    fn buf_ensure_loaded(buf: *mut buf_T) -> bool;
    fn open_buffer(
        read_stdin: bool,
        eap: *mut exarg_T,
        flags_arg: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
    fn buf_clear_file(buf: *mut buf_T);
    fn buf_freeall(buf: *mut buf_T, flags: ::core::ffi::c_int);
    fn handle_swap_exists(old_curbuf: *mut bufref_T);
    fn do_autochdir();
    fn no_write_message_buf(buf: *mut buf_T);
    fn no_write_message();
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn buflist_findname(ffname: *mut ::core::ffi::c_char) -> *mut buf_T;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn get_winopts(buf: *mut buf_T);
    fn buflist_findfmark(buf: *mut buf_T) -> *mut fmark_T;
    fn setfname(
        buf: *mut buf_T,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        message: bool,
    ) -> ::core::ffi::c_int;
    fn buf_name_changed(buf: *mut buf_T);
    fn setaltfname(
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut buf_T;
    fn buflist_altfpos(win: *mut win_T);
    fn otherfile(ffname: *mut ::core::ffi::c_char) -> bool;
    fn fileinfo(
        fullname: ::core::ffi::c_int,
        shorthelp: ::core::ffi::c_int,
        dont_truncate: bool,
    );
    fn maketitle();
    fn fname_expand(
        buf: *mut buf_T,
        ffname: *mut *mut ::core::ffi::c_char,
        sfname: *mut *mut ::core::ffi::c_char,
    );
    fn do_modelines(flags: ::core::ffi::c_int);
    fn bt_nofilename(buf: *const buf_T) -> bool;
    fn bt_dontwrite(buf: *const buf_T) -> bool;
    fn bt_dontwrite_msg(buf: *const buf_T) -> bool;
    fn buf_hide(buf: *const buf_T) -> bool;
    fn set_buflisted(on: ::core::ffi::c_int);
    fn buf_updates_send_changes(
        buf: *mut buf_T,
        firstline: linenr_T,
        num_added: int64_t,
        num_removed: int64_t,
    );
    fn buf_write(
        buf: *mut buf_T,
        fname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        start: linenr_T,
        end: linenr_T,
        eap: *mut exarg_T,
        append: bool,
        forceit: bool,
        reset_changed: bool,
        filtering: bool,
    ) -> ::core::ffi::c_int;
    fn changed_bytes(lnum: linenr_T, col: colnr_T);
    fn appended_lines(lnum: linenr_T, count: linenr_T);
    fn appended_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn deleted_lines(lnum: linenr_T, count: linenr_T);
    fn deleted_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn del_lines(nlines: linenr_T, undo: bool);
    fn channel_job_running(id: uint64_t) -> bool;
    static mut p_awa: ::core::ffi::c_int;
    static mut p_cwh: OptInt;
    static mut p_ch: OptInt;
    static mut p_confirm: ::core::ffi::c_int;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_dir: *mut ::core::ffi::c_char;
    static mut p_gd: ::core::ffi::c_int;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_icm: *mut ::core::ffi::c_char;
    static mut p_lz: ::core::ffi::c_int;
    static mut p_rdt: OptInt;
    static mut p_report: OptInt;
    static mut p_so: OptInt;
    static mut p_sh: *mut ::core::ffi::c_char;
    static mut p_shq: *mut ::core::ffi::c_char;
    static mut p_srr: *mut ::core::ffi::c_char;
    static mut p_stmp: ::core::ffi::c_int;
    static mut p_shm: *mut ::core::ffi::c_char;
    static mut p_sol: ::core::ffi::c_int;
    static mut p_ur: OptInt;
    static mut p_verbose: OptInt;
    static mut p_warn: ::core::ffi::c_int;
    static mut p_window: OptInt;
    static mut p_write: ::core::ffi::c_int;
    static mut p_wa: ::core::ffi::c_int;
    fn xstrnsave(
        string: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf_add(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn transchar_nonprint(
        buf: *const buf_T,
        charbuf: *mut ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    );
    fn vim_isIDc(c: ::core::ffi::c_int) -> bool;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptodigit(q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptobin(q: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn skiptohex(q: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn check_pos(buf: *mut buf_T, pos: *mut pos_T);
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor_col(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    fn get_cursor_pos_ptr() -> *mut ::core::ffi::c_char;
    fn bufhl_add_hl_pos_offset(
        buf: *mut buf_T,
        src_id: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        pos_start: lpos_T,
        pos_end: lpos_T,
        offset: colnr_T,
    );
    fn diff_buf_add(buf: *mut buf_T);
    fn diff_invalidate(buf: *mut buf_T);
    fn get_digraph_for_char(val_arg: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn keymap_init() -> *mut ::core::ffi::c_char;
    fn add_to_history(
        histype: ::core::ffi::c_int,
        new_entry: *const ::core::ffi::c_char,
        new_entrylen: size_t,
        in_map: bool,
        sep: ::core::ffi::c_int,
    );
    fn update_screen() -> ::core::ffi::c_int;
    fn show_cursor_info_later(force: bool);
    fn number_width(wp: *mut win_T) -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn beginline(flags: ::core::ffi::c_int);
    static e_argreq: [::core::ffi::c_char; 0];
    static e_backslash: [::core::ffi::c_char; 0];
    static e_curdir: [::core::ffi::c_char; 0];
    static e_exists: [::core::ffi::c_char; 0];
    static e_interr: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invcmd: [::core::ffi::c_char; 0];
    static e_isadir2: [::core::ffi::c_char; 0];
    static e_modifiable: [::core::ffi::c_char; 0];
    static e_nopresub: [::core::ffi::c_char; 0];
    static e_noprev: [::core::ffi::c_char; 0];
    static e_noprevre: [::core::ffi::c_char; 0];
    static e_notmp: [::core::ffi::c_char; 0];
    static e_cant_read_file_str: [::core::ffi::c_char; 0];
    static e_patnotf2: [::core::ffi::c_char; 0];
    static e_readonly: [::core::ffi::c_char; 0];
    static e_sandbox: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_zerocount: [::core::ffi::c_char; 0];
    static e_bufloaded: [::core::ffi::c_char; 0];
    static e_val_too_large_len: [::core::ffi::c_char; 0];
    static e_cannot_switch_to_a_closing_buffer: [::core::ffi::c_char; 0];
    static mut msg_listdo_overwrite: ::core::ffi::c_int;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn msg_multiline(
        str: String_0,
        hl_id: ::core::ffi::c_int,
        check_int: bool,
        hist: bool,
        need_clear: *mut bool,
    );
    fn smsg(
        hl_id: ::core::ffi::c_int,
        s: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn wait_return(redraw: ::core::ffi::c_int);
    fn set_keep_msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int);
    fn messaging() -> bool;
    fn msgmore(n: ::core::ffi::c_int);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_starthere();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outnum(n: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_prt_line(s: *const ::core::ffi::c_char, list: bool);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn message_filtered(msg_0: *const ::core::ffi::c_char) -> bool;
    fn msg_sb_eol();
    fn msg_clr_eos();
    fn msg_end() -> bool;
    fn vim_dialog_yesno(
        type_0: ::core::ffi::c_int,
        title: *mut ::core::ffi::c_char,
        message: *mut ::core::ffi::c_char,
        dflt: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn tv_list_find_str(
        l: *mut list_T,
        n: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(
        idx: VimVarIndex,
        val: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    );
    fn autowrite(buf: *mut buf_T, forceit: bool) -> ::core::ffi::c_int;
    fn autowrite_all();
    fn check_changed(buf: *mut buf_T, flags: ::core::ffi::c_int) -> bool;
    fn dialog_changed(buf: *mut buf_T, checkall: bool);
    fn check_fname() -> ::core::ffi::c_int;
    fn buf_write_all(buf: *mut buf_T, forceit: bool) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn check_nextcmd(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn not_exiting(save_exiting: bool);
    fn before_quit_all(eap: *mut exarg_T) -> ::core::ffi::c_int;
    fn do_exedit(eap: *mut exarg_T, old_curwin: *mut win_T);
    fn ex_may_print(eap: *mut exarg_T);
    fn dialog_msg(
        buff: *mut ::core::ffi::c_char,
        format: *mut ::core::ffi::c_char,
        fname: *mut ::core::ffi::c_char,
    );
    fn aborting() -> bool;
    fn should_abort(retcode: ::core::ffi::c_int) -> bool;
    fn getcmdline_prompt(
        firstc: ::core::ffi::c_int,
        prompt: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        xp_context: ::core::ffi::c_int,
        xp_arg: *const ::core::ffi::c_char,
        highlight_callback: Callback,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> *mut ::core::ffi::c_char;
    fn text_locked() -> bool;
    fn curbuf_locked() -> bool;
    fn gotocmdline(clr: bool);
    fn readfile(
        fname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        from: linenr_T,
        lines_to_skip: linenr_T,
        lines_to_read: linenr_T,
        eap: *mut exarg_T,
        flags: ::core::ffi::c_int,
        silent: bool,
    ) -> ::core::ffi::c_int;
    fn set_file_options(set_options: bool, eap: *mut exarg_T);
    fn set_forced_fenc(eap: *mut exarg_T);
    fn buf_check_timestamp(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn write_lnum_adjust(offset: linenr_T);
    fn vim_tempname() -> *mut ::core::ffi::c_char;
    fn extmark_splice(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        old_row: ::core::ffi::c_int,
        old_col: colnr_T,
        old_byte: bcount_t,
        new_row: ::core::ffi::c_int,
        new_col: colnr_T,
        new_byte: bcount_t,
        undo: ExtmarkOp,
    );
    fn extmark_move_region(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        start_byte: bcount_t,
        extent_row: ::core::ffi::c_int,
        extent_col: colnr_T,
        extent_byte: bcount_t,
        new_row: ::core::ffi::c_int,
        new_col: colnr_T,
        new_byte: bcount_t,
        undo: ExtmarkOp,
    );
    static mut disable_fold_update: ::core::ffi::c_int;
    fn hasAnyFolding(win: *mut win_T) -> ::core::ffi::c_int;
    fn foldUpdate(wp: *mut win_T, top: linenr_T, bot: linenr_T);
    fn foldUpdateAll(win: *mut win_T);
    fn foldMoveRange(
        wp: *mut win_T,
        gap: *mut garray_T,
        line1: linenr_T,
        line2: linenr_T,
        dest: linenr_T,
    );
    fn AppendToRedobuff(s: *const ::core::ffi::c_char);
    fn AppendToRedobuffLit(str: *const ::core::ffi::c_char, len: ::core::ffi::c_int);
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_scrolled_ign: bool;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didout: bool;
    static mut info_message: bool;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut quit_more: bool;
    static mut lines_left: ::core::ffi::c_int;
    static mut highlight_match: bool;
    static mut search_match_lines: linenr_T;
    static mut search_match_endcol: colnr_T;
    static mut need_check_timestamps: bool;
    static mut did_check_timestamps: bool;
    static mut firstwin: *mut win_T;
    static mut lastwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut redraw_tabline: bool;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut exiting: bool;
    static mut secure: ::core::ffi::c_int;
    static mut textlock: ::core::ffi::c_int;
    static mut sandbox: ::core::ffi::c_int;
    static mut silent_mode: bool;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut State: ::core::ffi::c_int;
    static mut exmode_active: bool;
    static mut ex_no_reprint: bool;
    static mut no_u_sync: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut swap_exists_action: ::core::ffi::c_int;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut msg_buf: [::core::ffi::c_char; 480];
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut ex_normal_busy: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut skip_redraw: bool;
    static mut got_int: bool;
    static mut bangredo: bool;
    static mut global_busy: ::core::ffi::c_int;
    static mut g_do_tagpreview: ::core::ffi::c_int;
    static mut keep_help_flag: bool;
    static mut cmdwin_type: ::core::ffi::c_int;
    static mut cmdwin_buf: *mut buf_T;
    static mut cmdwin_win: *mut win_T;
    static mut cmdwin_old_curwin: *mut win_T;
    static mut sub_nsubs: ::core::ffi::c_int;
    static mut sub_nlines: linenr_T;
    fn prepare_help_buffer();
    fn syn_check_group(
        name: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn get_indent() -> ::core::ffi::c_int;
    fn get_indent_lnum(lnum: linenr_T) -> ::core::ffi::c_int;
    fn set_indent(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> bool;
    fn prompt_for_input(
        prompt: *mut ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> ::core::ffi::c_int;
    fn getout(exitval: ::core::ffi::c_int) -> !;
    fn setpcmark();
    fn mark_adjust(
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
        op: ExtmarkOp,
    );
    fn mark_adjust_nofold(
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
        op: ExtmarkOp,
    );
    fn set_last_cursor(win: *mut win_T);
    fn os_time() -> Timestamp;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_iscomposing_first(c: ::core::ffi::c_int) -> bool;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(
        c: ::core::ffi::c_int,
        buf: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_len(lnum: linenr_T) -> colnr_T;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_append_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
        noalloc: bool,
    ) -> ::core::ffi::c_int;
    fn ml_delete(lnum: linenr_T) -> ::core::ffi::c_int;
    fn ml_delete_flags(lnum: linenr_T, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ml_setmarked(lnum: linenr_T);
    fn ml_firstmarked() -> linenr_T;
    fn ml_clearmarked();
    fn makeswapname(
        fname: *mut ::core::ffi::c_char,
        ffname: *mut ::core::ffi::c_char,
        buf: *mut buf_T,
        dir_name: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn ml_find_line_or_offset(
        buf: *mut buf_T,
        lnum: linenr_T,
        offp: *mut ::core::ffi::c_int,
        no_ff: bool,
    ) -> ::core::ffi::c_int;
    fn setmouse();
    fn update_topline(wp: *mut win_T);
    fn changed_window_setting(wp: *mut win_T);
    fn changed_line_abv_curs();
    fn invalidate_botline_win(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn scrolldown_clamp();
    fn scrollup_clamp();
    fn do_check_cursorbind();
    fn reset_VIsual();
    fn do_join(
        count: size_t,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark: bool,
    ) -> ::core::ffi::c_int;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn buf_copy_options(buf: *mut buf_T, flags: ::core::ffi::c_int);
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn magic_isset() -> bool;
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_nodetype(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_file_is_writable(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_file_mkdir(
        fname: *mut ::core::ffi::c_char,
        mode: int32_t,
    ) -> ::core::ffi::c_int;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_breakcheck();
    fn line_breakcheck();
    fn fast_breakcheck();
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn call_shell(
        cmd: *mut ::core::ffi::c_char,
        opts: ::core::ffi::c_int,
        extra_shell_arg: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn invocation_path_tail(
        invocation: *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn fix_fname(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn linetabsize_col(
        startvcol: ::core::ffi::c_int,
        s: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn plines_m_win_fill(
        wp: *mut win_T,
        first: linenr_T,
        last: linenr_T,
    ) -> ::core::ffi::c_int;
    fn profile_setlimit(msec: int64_t) -> proftime_T;
    fn profile_passed_limit(tm: proftime_T) -> bool;
    fn profile_zero() -> proftime_T;
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn skip_regexp(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn skip_regexp_err(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn skip_regexp_ex(
        startp: *mut ::core::ffi::c_char,
        dirc: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
        newp: *mut *mut ::core::ffi::c_char,
        dropped: *mut ::core::ffi::c_int,
        magic_val: *mut magic_T,
    ) -> *mut ::core::ffi::c_char;
    fn regtilde(
        source: *mut ::core::ffi::c_char,
        magic: ::core::ffi::c_int,
        preview: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regsub_multi(
        rmp: *mut regmmatch_T,
        lnum: linenr_T,
        source: *mut ::core::ffi::c_char,
        dest: *mut ::core::ffi::c_char,
        destlen: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(
        rmp: *mut regmatch_T,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn vim_regexec_multi(
        rmp: *mut regmmatch_T,
        win: *mut win_T,
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        tm: *mut proftime_T,
        timed_out: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn search_regcomp(
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        used_pat: *mut *mut ::core::ffi::c_char,
        pat_save: ::core::ffi::c_int,
        pat_use: ::core::ffi::c_int,
        options: ::core::ffi::c_int,
        regmatch: *mut regmmatch_T,
    ) -> ::core::ffi::c_int;
    fn get_search_pat() -> *mut ::core::ffi::c_char;
    fn save_re_pat(
        idx: ::core::ffi::c_int,
        pat: *mut ::core::ffi::c_char,
        patlen: size_t,
        magic: ::core::ffi::c_int,
    );
    fn last_search_pat() -> *mut ::core::ffi::c_char;
    fn parse_spelllang(wp: *mut win_T) -> *mut ::core::ffi::c_char;
    fn terminal_check_size(term: *mut Terminal);
    fn terminal_running(term: *const Terminal) -> bool;
    fn ui_cursor_goto(new_row: ::core::ffi::c_int, new_col: ::core::ffi::c_int);
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn u_save_cursor() -> ::core::ffi::c_int;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn u_savesub(lnum: linenr_T) -> ::core::ffi::c_int;
    fn u_inssub(lnum: linenr_T) -> ::core::ffi::c_int;
    fn u_savedel(lnum: linenr_T, nlines: linenr_T) -> ::core::ffi::c_int;
    fn u_savecommon(
        buf: *mut buf_T,
        top: linenr_T,
        bot: linenr_T,
        newbot: linenr_T,
        reload: bool,
    ) -> ::core::ffi::c_int;
    fn u_sync(force: bool);
    fn u_unchanged(buf: *mut buf_T);
    fn bufIsChanged(buf: *mut buf_T) -> bool;
    fn curbufIsChanged() -> bool;
    fn check_can_set_curbuf_forceit(forceit: ::core::ffi::c_int) -> bool;
    fn win_split(
        size: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_valid_any_tab(win: *mut win_T) -> bool;
    fn curwin_init();
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn check_lnums(do_curwin: bool);
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
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type uvarnumber_T = uint64_t;
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_17 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_17 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_17 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_17 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_17 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_17 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_17 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_17 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_17 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_17 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_17 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_17 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_17 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_17 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_17 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_17 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_17 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_17 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_17 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_17 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_17 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_17 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_17 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_17 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_17 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_17 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_17 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_17 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_17 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_17 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_17 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_17 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_17 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_17 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_17 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_17 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_17 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_17 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_17 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_17 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_17 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_17 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_17 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_17 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_17 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_17 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_17 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_17 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_17 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_17 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_17 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_17 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_17 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_17 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_17 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_17 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_17 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_17 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_17 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_17 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_17 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_17 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_17 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_17 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_17 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_17 = -2;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const REGSUB_BACKSLASH: C2Rust_Unnamed_18 = 4;
pub const REGSUB_MAGIC: C2Rust_Unnamed_18 = 2;
pub const REGSUB_COPY: C2Rust_Unnamed_18 = 1;
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
pub type ExtmarkOp = ::core::ffi::c_uint;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
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
pub struct SubReplacementString {
    pub sub: *mut ::core::ffi::c_char,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
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
pub type getf_retvalues = ::core::ffi::c_int;
pub const GETFILE_UNUSED: getf_retvalues = 8;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub const GETFILE_NOT_WRITTEN: getf_retvalues = 2;
pub const GETFILE_ERROR: getf_retvalues = 1;
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub type dobuf_action_values = ::core::ffi::c_uint;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_SPLIT: dobuf_action_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub type bfa_values = ::core::ffi::c_uint;
pub const BFA_IGNORE_ABORT: bfa_values = 8;
pub const BFA_KEEP_UNDO: bfa_values = 4;
pub const BFA_WIPE: bfa_values = 2;
pub const BFA_DEL: bfa_values = 1;
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
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_21 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_21 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_21 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_21 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_21 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_21 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_21 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_21 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_21 = 79;
pub const SHM_OVER: C2Rust_Unnamed_21 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_21 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_21 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_21 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_21 = 97;
pub const SHM_WRI: C2Rust_Unnamed_21 = 119;
pub const SHM_LINES: C2Rust_Unnamed_21 = 108;
pub const SHM_MOD: C2Rust_Unnamed_21 = 109;
pub const SHM_RO: C2Rust_Unnamed_21 = 114;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_22 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_22 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_22 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_22 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_22 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_22 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_22 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_22 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_22 = 0;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_int;
pub const HIST_DEBUG: C2Rust_Unnamed_23 = 4;
pub const HIST_INPUT: C2Rust_Unnamed_23 = 3;
pub const HIST_EXPR: C2Rust_Unnamed_23 = 2;
pub const HIST_SEARCH: C2Rust_Unnamed_23 = 1;
pub const HIST_CMD: C2Rust_Unnamed_23 = 0;
pub const HIST_INVALID: C2Rust_Unnamed_23 = -1;
pub const HIST_DEFAULT: C2Rust_Unnamed_23 = -2;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_24 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_24 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_24 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_24 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_24 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_24 = 20;
pub const UPD_VALID: C2Rust_Unnamed_24 = 10;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_25 = 4;
pub const BL_SOL: C2Rust_Unnamed_25 = 2;
pub const BL_WHITE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const VIM_LAST_TYPE: C2Rust_Unnamed_26 = 4;
pub const VIM_QUESTION: C2Rust_Unnamed_26 = 4;
pub const VIM_INFO: C2Rust_Unnamed_26 = 3;
pub const VIM_WARNING: C2Rust_Unnamed_26 = 2;
pub const VIM_ERROR: C2Rust_Unnamed_26 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_26 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_27 = 6;
pub const VIM_ALL: C2Rust_Unnamed_27 = 5;
pub const VIM_CANCEL: C2Rust_Unnamed_27 = 4;
pub const VIM_NO: C2Rust_Unnamed_27 = 3;
pub const VIM_YES: C2Rust_Unnamed_27 = 2;
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
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_28 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_28 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_28 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_28 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_28 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_28 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_29 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_29 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_29 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sorti_T {
    pub lnum: linenr_T,
    pub st_u: C2Rust_Unnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_30 {
    pub line: C2Rust_Unnamed_32,
    pub num: C2Rust_Unnamed_31,
    pub value_flt: float_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub value: varnumber_T,
    pub is_number: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_32 {
    pub start_col_nr: varnumber_T,
    pub end_col_nr: varnumber_T,
}
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_39 = 1;
pub const kShellOptRead: C2Rust_Unnamed_42 = 16;
pub const READ_FILTER: C2Rust_Unnamed_37 = 2;
pub const kShellOptFilter: C2Rust_Unnamed_42 = 1;
pub const kShellOptWrite: C2Rust_Unnamed_42 = 32;
pub const kShellOptDoOut: C2Rust_Unnamed_42 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_36 = 1;
pub const OPT_WINONLY: C2Rust_Unnamed_41 = 8;
pub const READ_NOWINENTER: C2Rust_Unnamed_37 = 128;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_37 = 32;
pub const BCO_ENTER: C2Rust_Unnamed_40 = 1;
pub const CCGD_EXCMD: C2Rust_Unnamed_35 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_35 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_35 = 2;
pub const CCGD_AW: C2Rust_Unnamed_35 = 1;
pub const MODE_NORMAL: C2Rust_Unnamed_38 = 1;
pub const MODE_CMDLINE: C2Rust_Unnamed_38 = 8;
pub const MODE_LANGMAP: C2Rust_Unnamed_38 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_38 = 16;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_36 = 2;
pub const SEARCH_HIS: C2Rust_Unnamed_44 = 32;
pub const RE_LAST: C2Rust_Unnamed_45 = 2;
pub const RE_BOTH: C2Rust_Unnamed_45 = 2;
pub const RE_SEARCH: C2Rust_Unnamed_45 = 0;
pub const RE_SUBST: C2Rust_Unnamed_45 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SubResult {
    pub start: lpos_T,
    pub end: lpos_T,
    pub pre_match: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_33 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SubResult,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PreviewLines {
    pub subresults: C2Rust_Unnamed_33,
    pub lines_needed: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct subflags_T {
    pub do_all: bool,
    pub do_ask: bool,
    pub do_count: bool,
    pub do_error: bool,
    pub do_print: bool,
    pub do_list: bool,
    pub do_number: bool,
    pub do_ic: SubIgnoreType,
}
pub type SubIgnoreType = ::core::ffi::c_uint;
pub const kSubMatchCase: SubIgnoreType = 2;
pub const kSubIgnoreCase: SubIgnoreType = 1;
pub const kSubHonorOptions: SubIgnoreType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineData {
    pub start_col: ::core::ffi::c_int,
    pub start: lpos_T,
    pub end: lpos_T,
    pub matchcols: ::core::ffi::c_int,
    pub matchbytes: bcount_t,
    pub subcols: ::core::ffi::c_int,
    pub subbytes: bcount_t,
    pub lnum_before: linenr_T,
    pub lnum_after: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_34 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut LineData,
}
pub const VGR_FUZZY: C2Rust_Unnamed_43 = 4;
pub const VGR_NOJUMP: C2Rust_Unnamed_43 = 2;
pub const VGR_GLOBAL: C2Rust_Unnamed_43 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const CCGD_ALLBUF: C2Rust_Unnamed_35 = 8;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_36 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_36 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_36 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_36 = 4;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_37 = 256;
pub const READ_FIFO: C2Rust_Unnamed_37 = 64;
pub const READ_DUMMY: C2Rust_Unnamed_37 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_37 = 8;
pub const READ_STDIN: C2Rust_Unnamed_37 = 4;
pub const READ_NEW: C2Rust_Unnamed_37 = 1;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_38 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_38 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_38 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_38 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_38 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_38 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_38 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_38 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_38 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_38 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_38 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_38 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_38 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_38 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_38 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_38 = 2;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_40 = 4;
pub const BCO_ALWAYS: C2Rust_Unnamed_40 = 2;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_41 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_41 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_41 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_41 = 16;
pub const OPT_MODELINE: C2Rust_Unnamed_41 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_41 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_41 = 1;
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_42 = 64;
pub const kShellOptSilent: C2Rust_Unnamed_42 = 8;
pub const kShellOptExpand: C2Rust_Unnamed_42 = 2;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_44 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_44 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_44 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_44 = 512;
pub const SEARCH_START: C2Rust_Unnamed_44 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_44 = 128;
pub const SEARCH_END: C2Rust_Unnamed_44 = 64;
pub const SEARCH_OPT: C2Rust_Unnamed_44 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_44 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_44 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_44 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NEVERLOADED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EXFLAG_LIST: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EXFLAG_NR: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EXFLAG_PRINT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_E: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const Ctrl_Y: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_ALTWRITE: ::core::ffi::c_int = 'A' as ::core::ffi::c_int;
pub const CPO_OVERNEW: ::core::ffi::c_int = 'O' as ::core::ffi::c_int;
pub const CPO_REMMARK: ::core::ffi::c_int = 'R' as ::core::ffi::c_int;
pub const CPO_UNDO: ::core::ffi::c_int = 'u' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
static mut e_non_numeric_argument_to_z: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<
        [u8; 33],
        [::core::ffi::c_char; 33],
    >(*b"E144: Non-numeric argument to :z\0")
};
#[no_mangle]
pub unsafe extern "C" fn do_ascii(mut eap: *mut exarg_T) {
    let mut data: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
    let mut len: size_t = utfc_ptr2len(data) as size_t;
    if len == 0 as size_t {
        msg(b"NUL\0".as_ptr() as *const ::core::ffi::c_char, 0 as ::core::ffi::c_int);
        return;
    }
    let mut need_clear: bool = true_0 != 0;
    msg_sb_eol();
    msg_start();
    let mut c: ::core::ffi::c_int = utf_ptr2char(data);
    let mut off: size_t = 0 as size_t;
    if c < 0x80 as ::core::ffi::c_int {
        if c == NL {
            c = NUL;
        }
        let cval: ::core::ffi::c_int = if c == CAR && get_fileformat(curbuf) == EOL_MAC {
            NL
        } else {
            c
        };
        let mut buf1: [::core::ffi::c_char; 20] = [0; 20];
        if vim_isprintc(c) as ::core::ffi::c_int != 0
            && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
        {
            let mut buf3: [::core::ffi::c_char; 7] = [0; 7];
            transchar_nonprint(curbuf, &raw mut buf3 as *mut ::core::ffi::c_char, c);
            vim_snprintf(
                &raw mut buf1 as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"  <%s>\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut buf3 as *mut ::core::ffi::c_char,
            );
        } else {
            buf1[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        let mut buf2: [::core::ffi::c_char; 20] = [0; 20];
        buf2[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        let mut dig: *mut ::core::ffi::c_char = get_digraph_for_char(cval);
        if !dig.is_null() {
            vim_snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                gettext(
                    b"<%s>%s%s  %d,  Hex %02x,  Oct %03o, Digr %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                transchar(c),
                &raw mut buf1 as *mut ::core::ffi::c_char,
                &raw mut buf2 as *mut ::core::ffi::c_char,
                cval,
                cval,
                cval,
                dig,
            );
        } else {
            vim_snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                gettext(
                    b"<%s>%s%s  %d,  Hex %02x,  Octal %03o\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                transchar(c),
                &raw mut buf1 as *mut ::core::ffi::c_char,
                &raw mut buf2 as *mut ::core::ffi::c_char,
                cval,
                cval,
                cval,
            );
        }
        msg_multiline(
            cstr_as_string(&raw mut IObuff as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
        off = off.wrapping_add(utf_ptr2len(data) as size_t);
    }
    while off < len {
        c = utf_ptr2char(data.offset(off as isize));
        let mut iobuff_len: size_t = 0 as size_t;
        if off > 0 as size_t {
            let c2rust_fresh0 = iobuff_len;
            iobuff_len = iobuff_len.wrapping_add(1);
            IObuff[c2rust_fresh0 as usize] = ' ' as ::core::ffi::c_char;
        }
        let c2rust_fresh1 = iobuff_len;
        iobuff_len = iobuff_len.wrapping_add(1);
        IObuff[c2rust_fresh1 as usize] = '<' as ::core::ffi::c_char;
        if utf_iscomposing_first(c) {
            let c2rust_fresh2 = iobuff_len;
            iobuff_len = iobuff_len.wrapping_add(1);
            IObuff[c2rust_fresh2 as usize] = ' ' as ::core::ffi::c_char;
        }
        iobuff_len = iobuff_len
            .wrapping_add(
                utf_char2bytes(
                    c,
                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                        .offset(iobuff_len as isize),
                ) as size_t,
            );
        let mut dig_0: *mut ::core::ffi::c_char = get_digraph_for_char(c);
        if !dig_0.is_null() {
            vim_snprintf(
                (&raw mut IObuff as *mut ::core::ffi::c_char)
                    .offset(iobuff_len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>()
                    .wrapping_sub(iobuff_len),
                if c < 0x10000 as ::core::ffi::c_int {
                    gettext(
                        b"> %d, Hex %04x, Oct %o, Digr %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    )
                } else {
                    gettext(
                        b"> %d, Hex %08x, Oct %o, Digr %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    )
                },
                c,
                c,
                c,
                dig_0,
            );
        } else {
            vim_snprintf(
                (&raw mut IObuff as *mut ::core::ffi::c_char)
                    .offset(iobuff_len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 1025]>()
                    .wrapping_sub(iobuff_len),
                if c < 0x10000 as ::core::ffi::c_int {
                    gettext(
                        b"> %d, Hex %04x, Octal %o\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    )
                } else {
                    gettext(
                        b"> %d, Hex %08x, Octal %o\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    )
                },
                c,
                c,
                c,
            );
        }
        msg_multiline(
            cstr_as_string(&raw mut IObuff as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
            true_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
        off = off.wrapping_add(utf_ptr2len(data.offset(off as isize)) as size_t);
    }
    if need_clear {
        msg_clr_eos();
    }
    msg_end();
}
#[no_mangle]
pub unsafe extern "C" fn ex_align(mut eap: *mut exarg_T) {
    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut new_indent: ::core::ffi::c_int = 0;
    if (*curwin).w_onebuf_opt.wo_rl != 0 {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_right as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_left;
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_right;
        }
    }
    let mut width: ::core::ffi::c_int = atoi((*eap).arg);
    let mut save_curpos: pos_T = (*curwin).w_cursor;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
        if width >= 0 as ::core::ffi::c_int {
            indent = width;
        }
    } else {
        if width <= 0 as ::core::ffi::c_int {
            width = (*curbuf).b_p_tw as ::core::ffi::c_int;
        }
        if width == 0 as ::core::ffi::c_int && (*curbuf).b_p_wm > 0 as OptInt {
            width = (*curwin).w_view_width - (*curbuf).b_p_wm as ::core::ffi::c_int;
        }
        if width <= 0 as ::core::ffi::c_int {
            width = 80 as ::core::ffi::c_int;
        }
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    (*curwin).w_cursor.lnum = (*eap).line1;
    while (*curwin).w_cursor.lnum <= (*eap).line2 {
        's_118: {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_left as ::core::ffi::c_int {
                new_indent = indent;
            } else {
                let mut has_tab: ::core::ffi::c_int = false_0;
                let mut len: ::core::ffi::c_int = linelen(
                    (if (*eap).cmdidx as ::core::ffi::c_int
                        == CMD_right as ::core::ffi::c_int
                    {
                        &raw mut has_tab
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_int>()
                    }),
                ) - get_indent();
                if len <= 0 as ::core::ffi::c_int {
                    break 's_118;
                } else if (*eap).cmdidx as ::core::ffi::c_int
                    == CMD_center as ::core::ffi::c_int
                {
                    new_indent = (width - len) / 2 as ::core::ffi::c_int;
                } else {
                    new_indent = width - len;
                    if has_tab != 0 {
                        while new_indent > 0 as ::core::ffi::c_int {
                            set_indent(new_indent, 0 as ::core::ffi::c_int);
                            if linelen(::core::ptr::null_mut::<::core::ffi::c_int>())
                                <= width
                            {
                                loop {
                                    new_indent += 1;
                                    set_indent(new_indent, 0 as ::core::ffi::c_int);
                                    if linelen(::core::ptr::null_mut::<::core::ffi::c_int>())
                                        > width
                                    {
                                        break;
                                    }
                                }
                                new_indent -= 1;
                                break;
                            } else {
                                new_indent -= 1;
                            }
                        }
                    }
                }
            }
            new_indent = if new_indent > 0 as ::core::ffi::c_int {
                new_indent
            } else {
                0 as ::core::ffi::c_int
            };
            set_indent(new_indent, 0 as ::core::ffi::c_int);
        }
        (*curwin).w_cursor.lnum += 1;
    }
    changed_lines(
        curbuf,
        (*eap).line1,
        0 as colnr_T,
        (*eap).line2 + 1 as linenr_T,
        0 as linenr_T,
        true_0 != 0,
    );
    (*curwin).w_cursor = save_curpos;
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
}
unsafe extern "C" fn linelen(
    mut has_tab: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    if *line as ::core::ffi::c_int == NUL {
        return 0 as ::core::ffi::c_int;
    }
    let mut first: *mut ::core::ffi::c_char = skipwhite(line);
    last = first.offset(strlen(first) as isize);
    while last > first
        && ascii_iswhite(
            *last.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        ) as ::core::ffi::c_int != 0
    {
        last = last.offset(-1);
    }
    let mut save: ::core::ffi::c_char = *last;
    *last = NUL as ::core::ffi::c_char;
    let mut len: ::core::ffi::c_int = linetabsize_str(line);
    if !has_tab.is_null() {
        *has_tab = !vim_strchr(first, TAB).is_null() as ::core::ffi::c_int;
    }
    *last = save;
    return len;
}
static mut sortbuf1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
    ::core::ffi::c_char,
>();
static mut sortbuf2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
    ::core::ffi::c_char,
>();
static mut sort_lc: bool = false;
static mut sort_ic: bool = false;
static mut sort_nr: bool = false;
static mut sort_rx: bool = false;
static mut sort_flt: bool = false;
static mut sort_abort: bool = false;
unsafe extern "C" fn string_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if sort_lc {
        return strcoll(
            s1 as *const ::core::ffi::c_char,
            s2 as *const ::core::ffi::c_char,
        );
    }
    return if sort_ic as ::core::ffi::c_int != 0 {
        strcasecmp(s1 as *mut ::core::ffi::c_char, s2 as *mut ::core::ffi::c_char)
    } else {
        strcmp(s1 as *const ::core::ffi::c_char, s2 as *const ::core::ffi::c_char)
    };
}
unsafe extern "C" fn sort_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut l1: sorti_T = *(s1 as *mut sorti_T);
    let mut l2: sorti_T = *(s2 as *mut sorti_T);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if sort_abort {
        return 0 as ::core::ffi::c_int;
    }
    fast_breakcheck();
    if got_int {
        sort_abort = true_0 != 0;
    }
    if sort_nr {
        if l1.st_u.num.is_number as ::core::ffi::c_int
            != l2.st_u.num.is_number as ::core::ffi::c_int
        {
            result = if l1.st_u.num.is_number as ::core::ffi::c_int
                > l2.st_u.num.is_number as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else {
            result = if l1.st_u.num.value == l2.st_u.num.value {
                0 as ::core::ffi::c_int
            } else if l1.st_u.num.value > l2.st_u.num.value {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
    } else if sort_flt {
        result = if l1.st_u.value_flt == l2.st_u.value_flt {
            0 as ::core::ffi::c_int
        } else if l1.st_u.value_flt > l2.st_u.value_flt {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        memcpy(
            sortbuf1 as *mut ::core::ffi::c_void,
            ml_get(l1.lnum).offset(l1.st_u.line.start_col_nr as isize)
                as *const ::core::ffi::c_void,
            (l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr + 1 as varnumber_T)
                as size_t,
        );
        *sortbuf1
            .offset((l1.st_u.line.end_col_nr - l1.st_u.line.start_col_nr) as isize) = NUL
            as ::core::ffi::c_char;
        memcpy(
            sortbuf2 as *mut ::core::ffi::c_void,
            ml_get(l2.lnum).offset(l2.st_u.line.start_col_nr as isize)
                as *const ::core::ffi::c_void,
            (l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr + 1 as varnumber_T)
                as size_t,
        );
        *sortbuf2
            .offset((l2.st_u.line.end_col_nr - l2.st_u.line.start_col_nr) as isize) = NUL
            as ::core::ffi::c_char;
        result = string_compare(
            sortbuf1 as *const ::core::ffi::c_void,
            sortbuf2 as *const ::core::ffi::c_void,
        );
    }
    if result == 0 as ::core::ffi::c_int {
        return l1.lnum as ::core::ffi::c_int - l2.lnum as ::core::ffi::c_int;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn ex_sort(mut eap: *mut exarg_T) {
    let mut old_count: bcount_t = 0;
    let mut new_count: bcount_t = 0;
    let mut lnum_0: linenr_T = 0;
    let mut deleted: linenr_T = 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: size_t = (((*eap).line2 - (*eap).line1) as size_t)
        .wrapping_add(1 as size_t);
    let mut i: size_t = 0;
    let mut unique: bool = false_0 != 0;
    let mut sort_what: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if count <= 1 as size_t {
        return;
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    sortbuf1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    sortbuf2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut nrs: *mut sorti_T = xmalloc(
        count.wrapping_mul(::core::mem::size_of::<sorti_T>()),
    ) as *mut sorti_T;
    sort_flt = false_0 != 0;
    sort_nr = sort_flt;
    sort_rx = sort_nr;
    sort_lc = sort_rx;
    sort_ic = sort_lc;
    sort_abort = sort_ic;
    let mut format_found: size_t = 0 as size_t;
    let mut change_occurred: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_sortend: {
        while *p as ::core::ffi::c_int != NUL {
            if !ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                    sort_ic = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    sort_lc = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                    sort_rx = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                    sort_nr = true_0 != 0;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'f' as ::core::ffi::c_int {
                    sort_flt = true_0 != 0;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'b' as ::core::ffi::c_int {
                    sort_what = STR2NR_BIN as ::core::ffi::c_int
                        + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
                    sort_what = STR2NR_OCT as ::core::ffi::c_int
                        + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'x' as ::core::ffi::c_int {
                    sort_what = STR2NR_HEX as ::core::ffi::c_int
                        + STR2NR_FORCE as ::core::ffi::c_int;
                    format_found = format_found.wrapping_add(1);
                } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                    unique = true_0 != 0;
                } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    break;
                } else if !check_nextcmd(p).is_null() {
                    (*eap).nextcmd = check_nextcmd(p);
                    break;
                } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                    && regmatch.regprog.is_null()
                {
                    let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        *p as ::core::ffi::c_int,
                        true_0,
                    );
                    if s.is_null() {
                        break '_sortend;
                    }
                    *s = NUL as ::core::ffi::c_char;
                    if s == p.offset(1 as ::core::ffi::c_int as isize) {
                        if last_search_pat().is_null() {
                            emsg(
                                gettext(&raw const e_noprevre as *const ::core::ffi::c_char),
                            );
                            break '_sortend;
                        } else {
                            regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                        }
                    } else {
                        regmatch.regprog = vim_regcomp(
                            p.offset(1 as ::core::ffi::c_int as isize),
                            RE_MAGIC,
                        );
                    }
                    if regmatch.regprog.is_null() {
                        break '_sortend;
                    }
                    p = s;
                    regmatch.rm_ic = p_ic != 0;
                } else {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                    break '_sortend;
                }
            }
            p = p.offset(1);
        }
        if format_found > 1 as size_t {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            sort_nr = sort_nr as ::core::ffi::c_int | sort_what != 0;
            let mut lnum: linenr_T = (*eap).line1;
            while lnum <= (*eap).line2 {
                let mut s_0: *mut ::core::ffi::c_char = ml_get(lnum);
                let mut len: ::core::ffi::c_int = ml_get_len(lnum);
                maxlen = if maxlen > len { maxlen } else { len };
                let mut start_col: colnr_T = 0 as colnr_T;
                let mut end_col: colnr_T = len as colnr_T;
                if !regmatch.regprog.is_null()
                    && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T)
                        as ::core::ffi::c_int != 0
                {
                    if sort_rx {
                        start_col = regmatch
                            .startp[0 as ::core::ffi::c_int as usize]
                            .offset_from(s_0) as colnr_T;
                        end_col = regmatch
                            .endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(s_0) as colnr_T;
                    } else {
                        start_col = regmatch
                            .endp[0 as ::core::ffi::c_int as usize]
                            .offset_from(s_0) as colnr_T;
                    }
                } else if !regmatch.regprog.is_null() {
                    end_col = 0 as ::core::ffi::c_int as colnr_T;
                }
                if sort_nr as ::core::ffi::c_int != 0
                    || sort_flt as ::core::ffi::c_int != 0
                {
                    let mut s2: *mut ::core::ffi::c_char = s_0.offset(end_col as isize);
                    let mut c: ::core::ffi::c_char = *s2;
                    *s2 = NUL as ::core::ffi::c_char;
                    let mut p_0: *mut ::core::ffi::c_char = s_0
                        .offset(start_col as isize);
                    if sort_nr {
                        if sort_what & STR2NR_HEX as ::core::ffi::c_int != 0 {
                            s_0 = skiptohex(p_0);
                        } else if sort_what & STR2NR_BIN as ::core::ffi::c_int != 0 {
                            s_0 = skiptobin(p_0) as *mut ::core::ffi::c_char;
                        } else {
                            s_0 = skiptodigit(p_0);
                        }
                        if s_0 > p_0
                            && *s_0.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                        {
                            s_0 = s_0.offset(-1);
                        }
                        if *s_0 as ::core::ffi::c_int == NUL {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .num
                                .is_number = false_0 != 0;
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .num
                                .value = 0 as varnumber_T;
                        } else {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .num
                                .is_number = true_0 != 0;
                            vim_str2nr(
                                s_0,
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                sort_what,
                                &raw mut (*nrs.offset((lnum - (*eap).line1) as isize))
                                    .st_u
                                    .num
                                    .value,
                                ::core::ptr::null_mut::<uvarnumber_T>(),
                                0 as ::core::ffi::c_int,
                                false_0 != 0,
                                ::core::ptr::null_mut::<bool>(),
                            );
                        }
                    } else {
                        s_0 = skipwhite(p_0);
                        if *s_0 as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                            s_0 = skipwhite(
                                s_0.offset(1 as ::core::ffi::c_int as isize),
                            );
                        }
                        if *s_0 as ::core::ffi::c_int == NUL {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .value_flt = -DBL_MAX as float_T;
                        } else {
                            (*nrs.offset((lnum - (*eap).line1) as isize))
                                .st_u
                                .value_flt = strtod(
                                s_0,
                                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            ) as float_T;
                        }
                    }
                    *s2 = c;
                } else {
                    (*nrs.offset((lnum - (*eap).line1) as isize))
                        .st_u
                        .line
                        .start_col_nr = start_col as varnumber_T;
                    (*nrs.offset((lnum - (*eap).line1) as isize)).st_u.line.end_col_nr = end_col
                        as varnumber_T;
                }
                (*nrs.offset((lnum - (*eap).line1) as isize)).lnum = lnum;
                if !regmatch.regprog.is_null() {
                    fast_breakcheck();
                }
                if got_int {
                    break '_sortend;
                }
                lnum += 1;
            }
            sortbuf1 = xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                as *mut ::core::ffi::c_char;
            sortbuf2 = xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
                as *mut ::core::ffi::c_char;
            qsort(
                nrs as *mut ::core::ffi::c_void,
                count,
                ::core::mem::size_of::<sorti_T>(),
                Some(
                    sort_compare
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            if !sort_abort {
                old_count = 0 as bcount_t;
                new_count = 0 as bcount_t;
                lnum_0 = (*eap).line2;
                i = 0 as size_t;
                while i < count {
                    let get_lnum: linenr_T = (*nrs
                        .offset(
                            (if (*eap).forceit != 0 {
                                count.wrapping_sub(i).wrapping_sub(1 as size_t)
                            } else {
                                i
                            }) as isize,
                        ))
                        .lnum;
                    if get_lnum + (count as linenr_T - 1 as linenr_T) != lnum_0 {
                        change_occurred = true_0 != 0;
                    }
                    let mut s_1: *mut ::core::ffi::c_char = ml_get(get_lnum);
                    let mut bytelen: colnr_T = ml_get_len(get_lnum) + 1 as colnr_T;
                    old_count += bytelen as bcount_t;
                    if !unique || i == 0 as size_t
                        || string_compare(
                            s_1 as *const ::core::ffi::c_void,
                            sortbuf1 as *const ::core::ffi::c_void,
                        ) != 0 as ::core::ffi::c_int
                    {
                        strcpy(sortbuf1, s_1);
                        let c2rust_fresh3 = lnum_0;
                        lnum_0 = lnum_0 + 1;
                        if ml_append(c2rust_fresh3, sortbuf1, 0 as colnr_T, false_0 != 0)
                            == FAIL
                        {
                            break;
                        }
                        new_count += bytelen as bcount_t;
                    }
                    fast_breakcheck();
                    if got_int {
                        break '_sortend;
                    }
                    i = i.wrapping_add(1);
                }
                if i == count {
                    i = 0 as size_t;
                    while i < count {
                        ml_delete((*eap).line1);
                        i = i.wrapping_add(1);
                    }
                } else {
                    count = 0 as size_t;
                }
                deleted = count as linenr_T - (lnum_0 - (*eap).line2);
                if deleted > 0 as linenr_T {
                    mark_adjust(
                        (*eap).line2 - deleted,
                        (*eap).line2,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        -deleted,
                        kExtmarkNOOP,
                    );
                    msgmore(-(deleted as ::core::ffi::c_int));
                } else if deleted < 0 as linenr_T {
                    mark_adjust(
                        (*eap).line2,
                        MAXLNUM as ::core::ffi::c_int as linenr_T,
                        -deleted,
                        0 as linenr_T,
                        kExtmarkNOOP,
                    );
                }
                if change_occurred as ::core::ffi::c_int != 0 || deleted != 0 as linenr_T
                {
                    extmark_splice(
                        curbuf,
                        (*eap).line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        0 as colnr_T,
                        count as ::core::ffi::c_int,
                        0 as colnr_T,
                        old_count,
                        lnum_0 as ::core::ffi::c_int
                            - (*eap).line2 as ::core::ffi::c_int,
                        0 as colnr_T,
                        new_count,
                        kExtmarkUndo,
                    );
                    changed_lines(
                        curbuf,
                        (*eap).line1,
                        0 as colnr_T,
                        (*eap).line2 + 1 as linenr_T,
                        -deleted,
                        true_0 != 0,
                    );
                }
                (*curwin).w_cursor.lnum = (*eap).line1;
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            }
        }
    }
    xfree(nrs as *mut ::core::ffi::c_void);
    xfree(sortbuf1 as *mut ::core::ffi::c_void);
    xfree(sortbuf2 as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    if got_int {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_uniq(mut eap: *mut exarg_T) {
    let mut match_continue: bool = false;
    let mut next_is_unmatch: bool = false;
    let mut done_lnum: linenr_T = 0;
    let mut delete_lnum: linenr_T = 0;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
    let mut keep_only_unique: bool = false_0 != 0;
    let mut keep_only_not_unique: bool = (*eap).forceit != 0;
    let mut deleted: linenr_T = 0 as linenr_T;
    if count <= 1 as linenr_T {
        return;
    }
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    sortbuf1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    sort_flt = false_0 != 0;
    sort_nr = sort_flt;
    sort_rx = sort_nr;
    sort_lc = sort_rx;
    sort_ic = sort_lc;
    sort_abort = sort_ic;
    let mut change_occurred: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_uniqend: {
        while *p as ::core::ffi::c_int != NUL {
            if !ascii_iswhite(*p as ::core::ffi::c_int) {
                if *p as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
                    sort_ic = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    sort_lc = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
                    sort_rx = true_0 != 0;
                } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                    if !keep_only_not_unique {
                        keep_only_unique = true_0 != 0;
                    }
                } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                    break;
                } else if (*eap).nextcmd.is_null() && !check_nextcmd(p).is_null() {
                    (*eap).nextcmd = check_nextcmd(p);
                    break;
                } else if !(*p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                    || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                        && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
                    && regmatch.regprog.is_null()
                {
                    let mut s: *mut ::core::ffi::c_char = skip_regexp_err(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        *p as ::core::ffi::c_int,
                        true_0,
                    );
                    if s.is_null() {
                        break '_uniqend;
                    }
                    *s = NUL as ::core::ffi::c_char;
                    if s == p.offset(1 as ::core::ffi::c_int as isize) {
                        if last_search_pat().is_null() {
                            emsg(
                                gettext(&raw const e_noprevre as *const ::core::ffi::c_char),
                            );
                            break '_uniqend;
                        } else {
                            regmatch.regprog = vim_regcomp(last_search_pat(), RE_MAGIC);
                        }
                    } else {
                        regmatch.regprog = vim_regcomp(
                            p.offset(1 as ::core::ffi::c_int as isize),
                            RE_MAGIC,
                        );
                    }
                    if regmatch.regprog.is_null() {
                        break '_uniqend;
                    }
                    p = s;
                    regmatch.rm_ic = p_ic != 0;
                } else {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                    break '_uniqend;
                }
            }
            p = p.offset(1);
        }
        let mut lnum: linenr_T = (*eap).line1;
        while lnum <= (*eap).line2 {
            let mut len: ::core::ffi::c_int = ml_get_len(lnum);
            if maxlen < len {
                maxlen = len;
            }
            if got_int {
                break '_uniqend;
            }
            lnum += 1;
        }
        sortbuf1 = xmalloc((maxlen as size_t).wrapping_add(1 as size_t))
            as *mut ::core::ffi::c_char;
        match_continue = false_0 != 0;
        next_is_unmatch = false_0 != 0;
        done_lnum = (*eap).line1 - 1 as linenr_T;
        delete_lnum = 0 as linenr_T;
        let mut i: linenr_T = 0 as linenr_T;
        while i < count {
            let mut get_lnum: linenr_T = (*eap).line1 + i;
            let mut s_0: *mut ::core::ffi::c_char = ml_get(get_lnum);
            let mut len_0: ::core::ffi::c_int = ml_get_len(get_lnum);
            let mut start_col: colnr_T = 0 as colnr_T;
            let mut end_col: colnr_T = len_0 as colnr_T;
            if !regmatch.regprog.is_null()
                && vim_regexec(&raw mut regmatch, s_0, 0 as colnr_T)
                    as ::core::ffi::c_int != 0
            {
                if sort_rx {
                    start_col = regmatch
                        .startp[0 as ::core::ffi::c_int as usize]
                        .offset_from(s_0) as colnr_T;
                    end_col = regmatch
                        .endp[0 as ::core::ffi::c_int as usize]
                        .offset_from(s_0) as colnr_T;
                } else {
                    start_col = regmatch
                        .endp[0 as ::core::ffi::c_int as usize]
                        .offset_from(s_0) as colnr_T;
                }
            } else if !regmatch.regprog.is_null() {
                end_col = 0 as ::core::ffi::c_int as colnr_T;
            }
            let mut save_c: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
            if end_col > 0 as ::core::ffi::c_int {
                save_c = *s_0.offset(end_col as isize);
                *s_0.offset(end_col as isize) = NUL as ::core::ffi::c_char;
            }
            let mut is_match: bool = if i > 0 as linenr_T {
                (string_compare(
                    s_0.offset(start_col as isize) as *const ::core::ffi::c_void,
                    sortbuf1 as *const ::core::ffi::c_void,
                ) == 0) as ::core::ffi::c_int
            } else {
                false_0
            } != 0;
            delete_lnum = 0 as ::core::ffi::c_int as linenr_T;
            if next_is_unmatch {
                is_match = false_0 != 0;
                next_is_unmatch = false_0 != 0;
            }
            if !keep_only_unique && !keep_only_not_unique {
                if is_match {
                    delete_lnum = get_lnum;
                } else {
                    strcpy(sortbuf1, s_0.offset(start_col as isize));
                }
            } else if keep_only_not_unique {
                if is_match {
                    done_lnum = get_lnum - 1 as linenr_T;
                    delete_lnum = get_lnum;
                    match_continue = true_0 != 0;
                } else {
                    if i > 0 as linenr_T && !match_continue
                        && get_lnum - 1 as linenr_T > done_lnum
                    {
                        delete_lnum = get_lnum - 1 as linenr_T;
                        next_is_unmatch = true_0 != 0;
                    } else if i >= count - 1 as linenr_T {
                        delete_lnum = get_lnum;
                    }
                    match_continue = false_0 != 0;
                    strcpy(sortbuf1, s_0.offset(start_col as isize));
                }
            } else if is_match {
                if !match_continue {
                    delete_lnum = get_lnum - 1 as linenr_T;
                } else {
                    delete_lnum = get_lnum;
                }
                match_continue = true_0 != 0;
            } else {
                if i == 0 as linenr_T && match_continue as ::core::ffi::c_int != 0 {
                    delete_lnum = get_lnum;
                }
                match_continue = false_0 != 0;
                strcpy(sortbuf1, s_0.offset(start_col as isize));
            }
            if end_col > 0 as ::core::ffi::c_int {
                *s_0.offset(end_col as isize) = save_c;
            }
            if delete_lnum > 0 as linenr_T {
                ml_delete(delete_lnum);
                i = (i as ::core::ffi::c_int
                    - (get_lnum - delete_lnum + 1 as linenr_T) as ::core::ffi::c_int)
                    as linenr_T;
                count -= 1;
                deleted += 1;
                change_occurred = true_0 != 0;
            }
            fast_breakcheck();
            if got_int {
                break '_uniqend;
            }
            i += 1;
        }
        mark_adjust(
            (*eap).line2 - deleted,
            (*eap).line2,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -deleted,
            (if change_occurred as ::core::ffi::c_int != 0 {
                kExtmarkUndo as ::core::ffi::c_int
            } else {
                kExtmarkNOOP as ::core::ffi::c_int
            }) as ExtmarkOp,
        );
        msgmore(-(deleted as ::core::ffi::c_int));
        if change_occurred {
            changed_lines(
                curbuf,
                (*eap).line1,
                0 as colnr_T,
                (*eap).line2 + 1 as linenr_T,
                -deleted,
                true_0 != 0,
            );
        }
        (*curwin).w_cursor.lnum = (*eap).line1;
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
    xfree(sortbuf1 as *mut ::core::ffi::c_void);
    vim_regfree(regmatch.regprog);
    if got_int {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_move(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut dest: linenr_T,
) -> ::core::ffi::c_int {
    if dest >= line1 && dest < line2 {
        emsg(
            gettext(
                b"E134: Cannot move a range of lines into itself\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    if dest == line1 - 1 as linenr_T || dest == line2 {
        (*curwin).w_cursor.lnum = if dest >= line1 {
            dest
        } else {
            dest + (line2 - line1) + 1 as linenr_T
        };
        return OK;
    }
    let mut start_byte: bcount_t = ml_find_line_or_offset(
        curbuf,
        line1,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut end_byte: bcount_t = ml_find_line_or_offset(
        curbuf,
        line2 + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut extent_byte: bcount_t = end_byte - start_byte;
    let mut dest_byte: bcount_t = ml_find_line_or_offset(
        curbuf,
        dest + 1 as linenr_T,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        true_0 != 0,
    ) as bcount_t;
    let mut num_lines: linenr_T = line2 - line1 + 1 as linenr_T;
    if u_save(dest, dest + 1 as linenr_T) == FAIL {
        return FAIL;
    }
    let mut l: linenr_T = 0;
    let mut extra: linenr_T = 0;
    extra = 0 as ::core::ffi::c_int as linenr_T;
    l = line1;
    while l <= line2 {
        let mut str: *mut ::core::ffi::c_char = xstrnsave(
            ml_get(l + extra),
            ml_get_len(l + extra) as size_t,
        );
        ml_append(dest + l - line1, str, 0 as colnr_T, false_0 != 0);
        xfree(str as *mut ::core::ffi::c_void);
        if dest < line1 {
            extra += 1;
        }
        l += 1;
    }
    let mut last_line: linenr_T = (*curbuf).b_ml.ml_line_count;
    mark_adjust_nofold(line1, line2, last_line - line2, 0 as linenr_T, kExtmarkNOOP);
    disable_fold_update += 1;
    changed_lines(
        curbuf,
        last_line - num_lines + 1 as linenr_T,
        0 as colnr_T,
        last_line + 1 as linenr_T,
        num_lines,
        false_0 != 0,
    );
    disable_fold_update -= 1;
    let mut line_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut byte_off: bcount_t = 0 as bcount_t;
    if dest >= line2 {
        mark_adjust_nofold(
            line2 + 1 as linenr_T,
            dest,
            -num_lines,
            0 as linenr_T,
            kExtmarkNOOP,
        );
        let mut tab: *mut tabpage_T = first_tabpage as *mut tabpage_T;
        while !tab.is_null() {
            let mut win: *mut win_T = if tab == curtab {
                firstwin
            } else {
                (*tab).tp_firstwin
            };
            while !win.is_null() {
                if (*win).w_buffer == curbuf {
                    foldMoveRange(win, &raw mut (*win).w_folds, line1, line2, dest);
                }
                win = (*win).w_next;
            }
            tab = (*tab).tp_next as *mut tabpage_T;
        }
        if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf).b_op_start.lnum = dest - num_lines + 1 as linenr_T;
            (*curbuf).b_op_end.lnum = dest;
        }
        line_off = -num_lines as ::core::ffi::c_int;
        byte_off = -extent_byte;
    } else {
        mark_adjust_nofold(
            dest + 1 as linenr_T,
            line1 - 1 as linenr_T,
            num_lines,
            0 as linenr_T,
            kExtmarkNOOP,
        );
        let mut tab_0: *mut tabpage_T = first_tabpage as *mut tabpage_T;
        while !tab_0.is_null() {
            let mut win_0: *mut win_T = if tab_0 == curtab {
                firstwin
            } else {
                (*tab_0).tp_firstwin
            };
            while !win_0.is_null() {
                if (*win_0).w_buffer == curbuf {
                    foldMoveRange(
                        win_0,
                        &raw mut (*win_0).w_folds,
                        dest + 1 as linenr_T,
                        line1 - 1 as linenr_T,
                        line2,
                    );
                }
                win_0 = (*win_0).w_next;
            }
            tab_0 = (*tab_0).tp_next as *mut tabpage_T;
        }
        if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf).b_op_start.lnum = dest + 1 as linenr_T;
            (*curbuf).b_op_end.lnum = dest + num_lines;
        }
    }
    if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*curbuf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf).b_op_start.col = (*curbuf).b_op_end.col;
    }
    mark_adjust_nofold(
        last_line - num_lines + 1 as linenr_T,
        last_line,
        -(last_line - dest - extra),
        0 as linenr_T,
        kExtmarkNOOP,
    );
    disable_fold_update += 1;
    changed_lines(
        curbuf,
        last_line - num_lines + 1 as linenr_T,
        0 as colnr_T,
        last_line + 1 as linenr_T,
        -extra,
        false_0 != 0,
    );
    disable_fold_update -= 1;
    buf_updates_send_changes(
        curbuf,
        dest + 1 as linenr_T,
        num_lines as int64_t,
        0 as int64_t,
    );
    if u_save(line1 + extra - 1 as linenr_T, line2 + extra + 1 as linenr_T) == FAIL {
        return FAIL;
    }
    l = line1;
    while l <= line2 {
        ml_delete_flags(line1 + extra, ML_DEL_MESSAGE as ::core::ffi::c_int);
        l += 1;
    }
    if global_busy == 0 && num_lines as OptInt > p_report {
        smsg(
            0 as ::core::ffi::c_int,
            ngettext(
                b"%ld line moved\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld lines moved\0".as_ptr() as *const ::core::ffi::c_char,
                num_lines as ::core::ffi::c_ulong,
            ),
            num_lines as int64_t,
        );
    }
    extmark_move_region(
        curbuf,
        line1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        start_byte,
        line2 as ::core::ffi::c_int - line1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int,
        0 as colnr_T,
        extent_byte,
        dest as ::core::ffi::c_int + line_off,
        0 as colnr_T,
        dest_byte + byte_off,
        kExtmarkUndo,
    );
    if dest >= line1 {
        (*curwin).w_cursor.lnum = dest;
    } else {
        (*curwin).w_cursor.lnum = dest + (line2 - line1) + 1 as linenr_T;
    }
    if line1 < dest {
        dest = (dest as ::core::ffi::c_int
            + (num_lines + 1 as linenr_T) as ::core::ffi::c_int) as linenr_T;
        last_line = (*curbuf).b_ml.ml_line_count;
        dest = if dest < last_line + 1 as linenr_T {
            dest
        } else {
            last_line + 1 as linenr_T
        };
        changed_lines(curbuf, line1, 0 as colnr_T, dest, 0 as linenr_T, false_0 != 0);
    } else {
        changed_lines(
            curbuf,
            dest + 1 as linenr_T,
            0 as colnr_T,
            line1 + num_lines,
            0 as linenr_T,
            false_0 != 0,
        );
    }
    buf_updates_send_changes(curbuf, line1 + extra, 0 as int64_t, num_lines as int64_t);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_copy(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut n: linenr_T,
) {
    let mut count: linenr_T = line2 - line1 + 1 as linenr_T;
    if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*curbuf).b_op_start.lnum = n + 1 as linenr_T;
        (*curbuf).b_op_end.lnum = n + count;
        (*curbuf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf).b_op_start.col = (*curbuf).b_op_end.col;
    }
    if u_save(n, n + 1 as linenr_T) == FAIL {
        return;
    }
    (*curwin).w_cursor.lnum = n;
    while line1 <= line2 {
        let mut p: *mut ::core::ffi::c_char = xstrnsave(
            ml_get(line1),
            ml_get_len(line1) as size_t,
        );
        ml_append((*curwin).w_cursor.lnum, p, 0 as colnr_T, false_0 != 0);
        xfree(p as *mut ::core::ffi::c_void);
        if line1 == n {
            line1 = (*curwin).w_cursor.lnum;
        }
        line1 += 1;
        if (*curwin).w_cursor.lnum < line1 {
            line1 += 1;
        }
        if (*curwin).w_cursor.lnum < line2 {
            line2 += 1;
        }
        (*curwin).w_cursor.lnum += 1;
    }
    appended_lines_mark(n, count as ::core::ffi::c_int);
    if VIsual_active {
        check_pos(curbuf, &raw mut VIsual);
    }
    msgmore(count as ::core::ffi::c_int);
}
static mut prevcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
    ::core::ffi::c_char,
>();
unsafe extern "C" fn prevcmd_is_set() -> ::core::ffi::c_int {
    if prevcmd.is_null() {
        emsg(gettext(&raw const e_noprev as *const ::core::ffi::c_char));
        return false_0;
    }
    return true_0;
}
#[no_mangle]
pub unsafe extern "C" fn do_bang(
    mut addr_count: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
    mut forceit: bool,
    mut do_in: bool,
    mut do_out: bool,
) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut line1: linenr_T = (*eap).line1;
    let mut line2: linenr_T = (*eap).line2;
    let mut newcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut free_newcmd: bool = false_0 != 0;
    let mut scroll_save: ::core::ffi::c_int = msg_scroll;
    if check_secure() {
        return;
    }
    if addr_count == 0 as ::core::ffi::c_int {
        msg_scroll = false_0;
        autowrite_all();
        msg_scroll = scroll_save;
    }
    let mut ins_prevcmd: bool = forceit;
    let mut trailarg: *mut ::core::ffi::c_char = skipwhite(arg);
    loop {
        let mut len: size_t = strlen(trailarg).wrapping_add(1 as size_t);
        if !newcmd.is_null() {
            len = len.wrapping_add(strlen(newcmd));
        }
        if ins_prevcmd {
            if prevcmd_is_set() == 0 {
                xfree(newcmd as *mut ::core::ffi::c_void);
                return;
            }
            len = len.wrapping_add(strlen(prevcmd));
        }
        let mut t: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        *t = NUL as ::core::ffi::c_char;
        if !newcmd.is_null() {
            strcat(t, newcmd);
        }
        if ins_prevcmd {
            strcat(t, prevcmd);
        }
        let mut p: *mut ::core::ffi::c_char = t.offset(strlen(t) as isize);
        strcat(t, trailarg);
        xfree(newcmd as *mut ::core::ffi::c_void);
        newcmd = t;
        trailarg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while *p != 0 {
            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                if p > newcmd
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                {
                    memmove(
                        p.offset(-(1 as ::core::ffi::c_int as isize))
                            as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        strlen(p).wrapping_add(1 as size_t),
                    );
                } else {
                    trailarg = p;
                    let c2rust_fresh4 = trailarg;
                    trailarg = trailarg.offset(1);
                    *c2rust_fresh4 = NUL as ::core::ffi::c_char;
                    ins_prevcmd = true_0 != 0;
                    break;
                }
            }
            p = p.offset(1);
        }
        if trailarg.is_null() {
            break;
        }
    }
    if strlen(newcmd) > 0 as size_t {
        xfree(prevcmd as *mut ::core::ffi::c_void);
        prevcmd = newcmd;
    } else {
        free_newcmd = true_0 != 0;
    }
    '_theend: {
        if bangredo {
            if prevcmd_is_set() == 0 {
                break '_theend;
            } else {
                let mut cmd: *mut ::core::ffi::c_char = vim_strsave_escaped(
                    prevcmd,
                    b"%#\0".as_ptr() as *const ::core::ffi::c_char,
                );
                AppendToRedobuffLit(cmd, -1 as ::core::ffi::c_int);
                xfree(cmd as *mut ::core::ffi::c_void);
                AppendToRedobuff(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                bangredo = false_0 != 0;
            }
        }
        if *p_shq as ::core::ffi::c_int != NUL {
            if free_newcmd {
                xfree(newcmd as *mut ::core::ffi::c_void);
            }
            newcmd = xmalloc(
                strlen(prevcmd)
                    .wrapping_add((2 as size_t).wrapping_mul(strlen(p_shq)))
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            strcpy(newcmd, p_shq);
            strcat(newcmd, prevcmd);
            strcat(newcmd, p_shq);
            free_newcmd = true_0 != 0;
        }
        if addr_count == 0 as ::core::ffi::c_int {
            msg_start();
            msg_ext_set_kind(b"shell_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            msg_putchar(':' as ::core::ffi::c_int);
            msg_putchar('!' as ::core::ffi::c_int);
            msg_outtrans(newcmd, 0 as ::core::ffi::c_int, false_0 != 0);
            msg_clr_eos();
            ui_cursor_goto(msg_row, msg_col);
            do_shell(newcmd, 0 as ::core::ffi::c_int);
        } else {
            do_filter(line1, line2, eap, newcmd, do_in, do_out);
            apply_autocmds(
                EVENT_SHELLFILTERPOST,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf,
            );
        }
    }
    if free_newcmd {
        xfree(newcmd as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn do_filter(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut eap: *mut exarg_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut do_in: bool,
    mut do_out: bool,
) {
    let mut read_linecount: linenr_T = 0;
    let mut cmd_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut itmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut otmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut old_curbuf: *mut buf_T = curbuf;
    let mut shell_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let orig_start: pos_T = (*curbuf).b_op_start;
    let orig_end: pos_T = (*curbuf).b_op_end;
    let stmp: ::core::ffi::c_int = p_stmp;
    if *cmd as ::core::ffi::c_int == NUL {
        return;
    }
    let save_cmod_flags: ::core::ffi::c_int = cmdmod.cmod_flags;
    cmdmod.cmod_flags &= !(CMOD_LOCKMARKS as ::core::ffi::c_int);
    let mut cursor_save: pos_T = (*curwin).w_cursor;
    let mut linecount: linenr_T = line2 - line1 + 1 as linenr_T;
    (*curwin).w_cursor.lnum = line1;
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    changed_line_abv_curs();
    invalidate_botline_win(curwin);
    if do_out {
        shell_flags |= kShellOptDoOut as ::core::ffi::c_int;
    }
    '_filterend: {
        if !do_in && do_out as ::core::ffi::c_int != 0 && stmp == 0 {
            shell_flags |= kShellOptRead as ::core::ffi::c_int;
            (*curwin).w_cursor.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0 && !do_out && stmp == 0 {
            shell_flags |= kShellOptWrite as ::core::ffi::c_int;
            (*curbuf).b_op_start.lnum = line1;
            (*curbuf).b_op_end.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0 && do_out as ::core::ffi::c_int != 0
            && stmp == 0
        {
            shell_flags
                |= kShellOptRead as ::core::ffi::c_int
                    | kShellOptWrite as ::core::ffi::c_int;
            (*curbuf).b_op_start.lnum = line1;
            (*curbuf).b_op_end.lnum = line2;
            (*curwin).w_cursor.lnum = line2;
        } else if do_in as ::core::ffi::c_int != 0
            && {
                itmp = vim_tempname();
                itmp.is_null()
            }
            || do_out as ::core::ffi::c_int != 0
                && {
                    otmp = vim_tempname();
                    otmp.is_null()
                }
        {
            emsg(gettext(&raw const e_notmp as *const ::core::ffi::c_char));
            break '_filterend;
        }
        no_wait_return += 1;
        if !itmp.is_null()
            && buf_write(
                curbuf,
                itmp,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                line1,
                line2,
                eap,
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
                true_0 != 0,
            ) == FAIL
        {
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            no_wait_return -= 1;
            if !aborting() {
                semsg(
                    gettext(
                        b"E482: Can't create file %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    itmp,
                );
            }
        } else if curbuf == old_curbuf {
            if !do_out {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            cmd_buf = make_filter_cmd(cmd, itmp, otmp, do_in);
            ui_cursor_goto(Rows - 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            '_error: {
                if do_out {
                    if u_save(line2, line2 + 1 as linenr_T) == FAIL {
                        xfree(cmd_buf as *mut ::core::ffi::c_void);
                        break '_error;
                    } else {
                        redraw_curbuf_later(UPD_VALID as ::core::ffi::c_int);
                    }
                }
                read_linecount = (*curbuf).b_ml.ml_line_count;
                call_shell(
                    cmd_buf,
                    kShellOptFilter as ::core::ffi::c_int | shell_flags,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                xfree(cmd_buf as *mut ::core::ffi::c_void);
                did_check_timestamps = false_0 != 0;
                need_check_timestamps = true_0 != 0;
                os_breakcheck();
                got_int = false_0 != 0;
                if do_out {
                    if !otmp.is_null() {
                        if readfile(
                            otmp,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            line2,
                            0 as linenr_T,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            eap,
                            READ_FILTER as ::core::ffi::c_int,
                            false_0 != 0,
                        ) != OK
                        {
                            if !aborting() {
                                msg_putchar('\n' as ::core::ffi::c_int);
                                semsg(
                                    gettext(
                                        &raw const e_cant_read_file_str
                                            as *const ::core::ffi::c_char,
                                    ),
                                    otmp,
                                );
                            }
                            break '_error;
                        } else if curbuf != old_curbuf {
                            break '_filterend;
                        }
                    }
                    read_linecount = (*curbuf).b_ml.ml_line_count - read_linecount;
                    if shell_flags & kShellOptRead as ::core::ffi::c_int != 0 {
                        (*curbuf).b_op_start.lnum = line2 + 1 as linenr_T;
                        (*curbuf).b_op_end.lnum = (*curwin).w_cursor.lnum;
                        appended_lines_mark(line2, read_linecount as ::core::ffi::c_int);
                    }
                    if do_in {
                        if cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0
                            || vim_strchr(p_cpo, CPO_REMMARK).is_null()
                        {
                            if read_linecount >= linecount {
                                mark_adjust(
                                    line1,
                                    line2,
                                    linecount,
                                    0 as linenr_T,
                                    kExtmarkNOOP,
                                );
                            } else {
                                mark_adjust(
                                    line1,
                                    line1 + read_linecount - 1 as linenr_T,
                                    linecount,
                                    0 as linenr_T,
                                    kExtmarkNOOP,
                                );
                                mark_adjust(
                                    line1 + read_linecount,
                                    line2,
                                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                                    0 as linenr_T,
                                    kExtmarkNOOP,
                                );
                            }
                        }
                        (*curwin).w_cursor.lnum = line1;
                        del_lines(linecount, true_0 != 0);
                        (*curbuf).b_op_start.lnum -= linecount;
                        (*curbuf).b_op_end.lnum -= linecount;
                        write_lnum_adjust(-linecount);
                        foldUpdate(
                            curwin,
                            (*curbuf).b_op_start.lnum,
                            (*curbuf).b_op_end.lnum,
                        );
                    } else {
                        linecount = (*curbuf).b_op_end.lnum - (*curbuf).b_op_start.lnum
                            + 1 as linenr_T;
                        (*curwin).w_cursor.lnum = (*curbuf).b_op_end.lnum;
                    }
                    beginline(
                        BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                    );
                    no_wait_return -= 1;
                    if linecount as OptInt > p_report {
                        if do_in {
                            vim_snprintf(
                                &raw mut msg_buf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
                                gettext(
                                    b"%ld lines filtered\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                linecount as int64_t,
                            );
                            if msg(
                                &raw mut msg_buf as *mut ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int != 0 && msg_scroll == 0
                            {
                                set_keep_msg(
                                    &raw mut msg_buf as *mut ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                );
                            }
                        } else {
                            msgmore(linecount as ::core::ffi::c_int);
                        }
                    }
                    break '_filterend;
                }
            }
            (*curwin).w_cursor = cursor_save;
            no_wait_return -= 1;
            wait_return(false_0);
        }
    }
    cmdmod.cmod_flags = save_cmod_flags;
    if curbuf != old_curbuf {
        no_wait_return -= 1;
        emsg(
            gettext(
                b"E135: *Filter* Autocommands must not change current buffer\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
    } else if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*curbuf).b_op_start = orig_start;
        (*curbuf).b_op_end = orig_end;
    }
    if !itmp.is_null() {
        os_remove(itmp);
    }
    if !otmp.is_null() {
        os_remove(otmp);
    }
    xfree(itmp as *mut ::core::ffi::c_void);
    xfree(otmp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn do_shell(
    mut cmd: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    if check_secure() {
        msg_end();
        return;
    }
    msg_putchar('\r' as ::core::ffi::c_int);
    msg_putchar('\n' as ::core::ffi::c_int);
    if p_warn != 0 && !autocmd_busy && msg_silent == 0 as ::core::ffi::c_int {
        let mut buf: *mut buf_T = firstbuf;
        while !buf.is_null() {
            if bufIsChanged(buf) {
                msg_puts(
                    gettext(
                        b"[No write since last change]\n\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                );
                break;
            } else {
                buf = (*buf).b_next;
            }
        }
    }
    ui_cursor_goto(msg_row, msg_col);
    call_shell(cmd, flags, ::core::ptr::null_mut::<::core::ffi::c_char>());
    if msg_silent == 0 as ::core::ffi::c_int {
        msg_didout = true_0 != 0;
    }
    did_check_timestamps = false_0 != 0;
    need_check_timestamps = true_0 != 0;
    msg_row = Rows - 1 as ::core::ffi::c_int;
    msg_col = 0 as ::core::ffi::c_int;
    apply_autocmds(
        EVENT_SHELLCMDPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
}
#[no_mangle]
pub unsafe extern "C" fn make_filter_cmd(
    mut cmd: *mut ::core::ffi::c_char,
    mut itmp: *mut ::core::ffi::c_char,
    mut otmp: *mut ::core::ffi::c_char,
    mut do_in: bool,
) -> *mut ::core::ffi::c_char {
    let mut is_fish_shell: bool = strncmp(
        invocation_path_tail(p_sh, ::core::ptr::null_mut::<size_t>()),
        b"fish\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int;
    let mut is_pwsh: bool = strncmp(
        invocation_path_tail(p_sh, ::core::ptr::null_mut::<size_t>()),
        b"pwsh\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            invocation_path_tail(p_sh, ::core::ptr::null_mut::<size_t>()),
            b"powershell\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int;
    let mut len: size_t = strlen(cmd).wrapping_add(1 as size_t);
    len = (len as ::core::ffi::c_ulong)
        .wrapping_add(
            (if is_fish_shell as ::core::ffi::c_int != 0 {
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>()
                    .wrapping_sub(1 as usize)
            } else if !is_pwsh {
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                    .wrapping_sub(1 as usize)
            } else {
                0 as usize
            }) as ::core::ffi::c_ulong,
        ) as size_t;
    if !itmp.is_null() {
        len = (len as ::core::ffi::c_ulong)
            .wrapping_add(
                (if is_pwsh as ::core::ffi::c_int != 0 {
                    strlen(itmp)
                        .wrapping_add(
                            ::core::mem::size_of::<[::core::ffi::c_char; 24]>(),
                        )
                        .wrapping_sub(1 as size_t)
                        .wrapping_add(6 as size_t)
                } else {
                    strlen(itmp)
                        .wrapping_add(
                            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
                        )
                        .wrapping_sub(1 as size_t)
                }) as ::core::ffi::c_ulong,
            ) as size_t;
    }
    if do_in as ::core::ffi::c_int != 0 && is_pwsh as ::core::ffi::c_int != 0 {
        len = (len as ::core::ffi::c_ulong)
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>()
                    as ::core::ffi::c_ulong,
            ) as size_t;
    }
    if !otmp.is_null() {
        len = len
            .wrapping_add(
                strlen(otmp).wrapping_add(strlen(p_srr)).wrapping_add(2 as size_t),
            );
    }
    let buf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if is_pwsh {
        if !itmp.is_null() {
            xstrlcpy(
                buf,
                b"& { Get-Content \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
            xstrlcat(
                buf,
                b" | & \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, cmd, len.wrapping_sub(1 as size_t));
            xstrlcat(
                buf,
                b" }\0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
        } else if do_in {
            xstrlcpy(
                buf,
                b" $input | \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, cmd, len);
        } else {
            xstrlcpy(buf, cmd, len);
        }
    } else {
        if !itmp.is_null() || !otmp.is_null() {
            let mut fmt: *mut ::core::ffi::c_char = (if is_fish_shell
                as ::core::ffi::c_int != 0
            {
                b"begin; %s; end\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"(%s)\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            vim_snprintf(buf, len, fmt, cmd);
        } else {
            xstrlcpy(buf, cmd, len);
        }
        if !itmp.is_null() {
            xstrlcat(
                buf,
                b" < \0".as_ptr() as *const ::core::ffi::c_char,
                len.wrapping_sub(1 as size_t),
            );
            xstrlcat(buf, itmp, len.wrapping_sub(1 as size_t));
        }
    }
    if !otmp.is_null() {
        append_redir(buf, len, p_srr, otmp);
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn append_redir(
    buf: *mut ::core::ffi::c_char,
    buflen: size_t,
    opt: *const ::core::ffi::c_char,
    fname: *const ::core::ffi::c_char,
) {
    let end: *mut ::core::ffi::c_char = buf.offset(strlen(buf) as isize);
    let mut p: *const ::core::ffi::c_char = opt;
    loop {
        p = strchr(p, '%' as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 's' as ::core::ffi::c_int
        {
            break;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '%' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    if !p.is_null() {
        *end = ' ' as ::core::ffi::c_char;
        vim_snprintf(
            end.offset(1 as ::core::ffi::c_int as isize),
            (buflen as ptrdiff_t
                - end.offset(1 as ::core::ffi::c_int as isize).offset_from(buf))
                as size_t,
            opt,
            fname,
        );
    } else {
        vim_snprintf(
            end,
            (buflen as ptrdiff_t - end.offset_from(buf)) as size_t,
            b" %s %s\0".as_ptr() as *const ::core::ffi::c_char,
            opt,
            fname,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn print_line_no_prefix(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
) {
    let mut numbuf: [::core::ffi::c_char; 30] = [0; 30];
    if (*curwin).w_onebuf_opt.wo_nu != 0 || use_number as ::core::ffi::c_int != 0 {
        vim_snprintf(
            &raw mut numbuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"%*d \0".as_ptr() as *const ::core::ffi::c_char,
            number_width(curwin),
            lnum,
        );
        msg_puts_hl(
            &raw mut numbuf as *mut ::core::ffi::c_char,
            HLF_N as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    msg_prt_line(ml_get(lnum), list);
}
static mut global_need_msg_kind: bool = false_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn print_line(
    mut lnum: linenr_T,
    mut use_number: bool,
    mut list: bool,
    mut first: bool,
) {
    let mut save_silent: bool = silent_mode;
    if message_filtered(ml_get(lnum)) {
        return;
    }
    silent_mode = false_0 != 0;
    info_message = true_0 != 0;
    if (global_busy == 0 || global_need_msg_kind as ::core::ffi::c_int != 0)
        && first as ::core::ffi::c_int != 0
    {
        msg_start();
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        global_need_msg_kind = false_0 != 0;
    } else if !save_silent {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    print_line_no_prefix(lnum, use_number, list);
    if save_silent {
        msg_putchar('\n' as ::core::ffi::c_int);
        silent_mode = save_silent;
    }
    info_message = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn rename_buffer(
    mut new_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = curbuf;
    apply_autocmds(
        EVENT_BUFFILEPRE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if buf != curbuf {
        return FAIL;
    }
    if aborting() {
        return FAIL;
    }
    let mut fname: *mut ::core::ffi::c_char = (*curbuf).b_ffname;
    let mut sfname: *mut ::core::ffi::c_char = (*curbuf).b_sfname;
    let mut xfname: *mut ::core::ffi::c_char = (*curbuf).b_fname;
    (*curbuf).b_ffname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*curbuf).b_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if setfname(
        curbuf,
        new_fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
    ) == FAIL
    {
        (*curbuf).b_ffname = fname;
        (*curbuf).b_sfname = sfname;
        return FAIL;
    }
    (*curbuf).b_flags |= BF_NOTEDITED;
    if !xfname.is_null() && *xfname as ::core::ffi::c_int != NUL {
        buf = buflist_new(
            fname,
            xfname,
            (*curwin).w_cursor.lnum,
            0 as ::core::ffi::c_int,
        );
        if !buf.is_null()
            && cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        {
            (*curwin).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(sfname as *mut ::core::ffi::c_void);
    apply_autocmds(
        EVENT_BUFFILEPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    do_autochdir();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_file(mut eap: *mut exarg_T) {
    if (*eap).addr_count > 0 as ::core::ffi::c_int
        && (*(*eap).arg as ::core::ffi::c_int != NUL || (*eap).line2 > 0 as linenr_T
            || (*eap).addr_count > 1 as ::core::ffi::c_int)
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL
        || (*eap).addr_count == 1 as ::core::ffi::c_int
    {
        if rename_buffer((*eap).arg) == FAIL {
            return;
        }
        redraw_tabline = true_0 != 0;
    }
    if *(*eap).arg as ::core::ffi::c_int == NUL
        || !shortmess(SHM_FILEINFO as ::core::ffi::c_int)
    {
        fileinfo(false_0, false_0, (*eap).forceit != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_update(mut eap: *mut exarg_T) {
    if curbufIsChanged() as ::core::ffi::c_int != 0
        || !bt_nofilename(curbuf) && !(*curbuf).b_ffname.is_null()
            && !os_path_exists((*curbuf).b_ffname)
    {
        do_write(eap);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_write(mut eap: *mut exarg_T) {
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
        (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).line2 = (*curbuf).b_ml.ml_line_count;
    }
    if (*eap).usefilter != 0 {
        do_bang(1 as ::core::ffi::c_int, eap, false_0 != 0, true_0 != 0, false_0 != 0);
    } else {
        do_write(eap);
    };
}
unsafe extern "C" fn check_writable(
    mut fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if os_nodetype(fname) == NODE_OTHER {
        semsg(
            gettext(
                b"E503: \"%s\" is not a file or writable device\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            fname,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn handle_mkdir_p_arg(
    mut eap: *mut exarg_T,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*eap).mkdir_p != 0
        && os_file_mkdir(fname, 0o755 as int32_t) < 0 as ::core::ffi::c_int
    {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn do_write(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    let mut other: bool = false;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut free_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut alt_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if not_writing() {
        return FAIL;
    }
    let mut ffname: *mut ::core::ffi::c_char = (*eap).arg;
    '_theend: {
        if *ffname as ::core::ffi::c_int == NUL {
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_saveas as ::core::ffi::c_int {
                emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
                break '_theend;
            } else {
                other = false_0 != 0;
            }
        } else {
            fname = ffname;
            free_fname = fix_fname(ffname);
            if !free_fname.is_null() {
                ffname = free_fname;
            }
            other = otherfile(ffname);
        }
        if other {
            if !vim_strchr(p_cpo, CPO_ALTWRITE).is_null()
                || (*eap).cmdidx as ::core::ffi::c_int
                    == CMD_saveas as ::core::ffi::c_int
            {
                alt_buf = setaltfname(ffname, fname, 1 as linenr_T);
            } else {
                alt_buf = buflist_findname(ffname);
            }
            if !alt_buf.is_null() && !(*alt_buf).b_ml.ml_mfp.is_null() {
                emsg(gettext(&raw const e_bufloaded as *const ::core::ffi::c_char));
                break '_theend;
            }
        }
        if !(!other
            && (bt_dontwrite_msg(curbuf) as ::core::ffi::c_int != 0
                || check_fname() == FAIL || check_writable((*curbuf).b_ffname) == FAIL
                || check_readonly(&raw mut (*eap).forceit, curbuf) != 0))
        {
            if !other {
                ffname = (*curbuf).b_ffname;
                fname = (*curbuf).b_fname;
                if ((*eap).line1 != 1 as linenr_T
                    || (*eap).line2 != (*curbuf).b_ml.ml_line_count)
                    && (*eap).forceit == 0 && (*eap).append == 0 && p_wa == 0
                {
                    if p_confirm != 0
                        || cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                    {
                        if vim_dialog_yesno(
                            VIM_QUESTION as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            gettext(
                                b"Write partial file?\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            2 as ::core::ffi::c_int,
                        ) != VIM_YES as ::core::ffi::c_int
                        {
                            break '_theend;
                        } else {
                            (*eap).forceit = true_0;
                        }
                    } else {
                        emsg(
                            gettext(
                                b"E140: Use ! to write partial buffer\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                        );
                        break '_theend;
                    }
                }
            }
            if check_overwrite(eap, curbuf, fname, ffname, other) == OK {
                if (*eap).cmdidx as ::core::ffi::c_int
                    == CMD_saveas as ::core::ffi::c_int && !alt_buf.is_null()
                {
                    let mut was_curbuf: *mut buf_T = curbuf;
                    apply_autocmds(
                        EVENT_BUFFILEPRE,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        curbuf,
                    );
                    apply_autocmds(
                        EVENT_BUFFILEPRE,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                        alt_buf,
                    );
                    if curbuf != was_curbuf || aborting() as ::core::ffi::c_int != 0 {
                        retval = FAIL;
                        break '_theend;
                    } else {
                        fname = (*alt_buf).b_fname;
                        (*alt_buf).b_fname = (*curbuf).b_fname;
                        (*curbuf).b_fname = fname;
                        fname = (*alt_buf).b_ffname;
                        (*alt_buf).b_ffname = (*curbuf).b_ffname;
                        (*curbuf).b_ffname = fname;
                        fname = (*alt_buf).b_sfname;
                        (*alt_buf).b_sfname = (*curbuf).b_sfname;
                        (*curbuf).b_sfname = fname;
                        buf_name_changed(curbuf);
                        apply_autocmds(
                            EVENT_BUFFILEPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            curbuf,
                        );
                        apply_autocmds(
                            EVENT_BUFFILEPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            alt_buf,
                        );
                        if (*alt_buf).b_p_bl == 0 {
                            (*alt_buf).b_p_bl = true_0;
                            apply_autocmds(
                                EVENT_BUFADD,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                alt_buf,
                            );
                        }
                        if curbuf != was_curbuf || aborting() as ::core::ffi::c_int != 0
                        {
                            retval = FAIL;
                            break '_theend;
                        } else {
                            if *(*curbuf).b_p_ft as ::core::ffi::c_int == NUL {
                                if augroup_exists(
                                    b"filetypedetect\0".as_ptr() as *const ::core::ffi::c_char,
                                ) {
                                    do_doautocmd(
                                        b"filetypedetect BufRead\0".as_ptr()
                                            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                }
                                do_modelines(0 as ::core::ffi::c_int);
                            }
                            fname = (*curbuf).b_sfname;
                        }
                    }
                }
                if handle_mkdir_p_arg(eap, fname) == FAIL {
                    retval = FAIL;
                } else {
                    let mut name_was_missing: ::core::ffi::c_int = (*curbuf)
                        .b_ffname
                        .is_null() as ::core::ffi::c_int;
                    retval = buf_write(
                        curbuf,
                        ffname,
                        fname,
                        (*eap).line1,
                        (*eap).line2,
                        eap,
                        (*eap).append != 0,
                        (*eap).forceit != 0,
                        true_0 != 0,
                        false_0 != 0,
                    );
                    if (*eap).cmdidx as ::core::ffi::c_int
                        == CMD_saveas as ::core::ffi::c_int
                    {
                        if retval == OK {
                            (*curbuf).b_p_ro = false_0;
                            redraw_tabline = true_0 != 0;
                        }
                    }
                    if (*eap).cmdidx as ::core::ffi::c_int
                        == CMD_saveas as ::core::ffi::c_int || name_was_missing != 0
                    {
                        do_autochdir();
                    }
                }
            }
        }
    }
    xfree(free_fname as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn check_overwrite(
    mut eap: *mut exarg_T,
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut ffname: *mut ::core::ffi::c_char,
    mut other: bool,
) -> ::core::ffi::c_int {
    if (other as ::core::ffi::c_int != 0
        || !bt_nofilename(buf)
            && ((*buf).b_flags & BF_NOTEDITED != 0
                || (*buf).b_flags & BF_NEW != 0
                    && vim_strchr(p_cpo, CPO_OVERNEW).is_null()
                || (*buf).b_flags & BF_READERR != 0)) && p_wa == 0
        && os_path_exists(ffname) as ::core::ffi::c_int != 0
    {
        if (*eap).forceit == 0 && (*eap).append == 0 {
            if os_isdir(ffname) {
                semsg(
                    gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                    ffname,
                );
                return FAIL;
            }
            if p_confirm != 0
                || cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
            {
                let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"Overwrite existing file \"%s\"?\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    fname,
                );
                if vim_dialog_yesno(
                    VIM_QUESTION as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut buff as *mut ::core::ffi::c_char,
                    2 as ::core::ffi::c_int,
                ) != VIM_YES as ::core::ffi::c_int
                {
                    return FAIL;
                }
                (*eap).forceit = true_0;
            } else {
                emsg(gettext(&raw const e_exists as *const ::core::ffi::c_char));
                return FAIL;
            }
        }
        if other as ::core::ffi::c_int != 0 && emsg_silent == 0 {
            let mut dir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            if *p_dir as ::core::ffi::c_int == NUL {
                dir = xmalloc(5 as size_t) as *mut ::core::ffi::c_char;
                strcpy(
                    dir,
                    b".\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
            } else {
                dir = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                let mut p: *mut ::core::ffi::c_char = p_dir;
                copy_option_part(
                    &raw mut p,
                    dir,
                    MAXPATHL as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
            }
            let mut swapname: *mut ::core::ffi::c_char = makeswapname(
                fname,
                ffname,
                curbuf,
                dir,
            );
            xfree(dir as *mut ::core::ffi::c_void);
            if os_path_exists(swapname) {
                if p_confirm != 0
                    || cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0
                {
                    let mut buff_0: [::core::ffi::c_char; 1000] = [0; 1000];
                    dialog_msg(
                        &raw mut buff_0 as *mut ::core::ffi::c_char,
                        gettext(
                            b"Swap file \"%s\" exists, overwrite anyway?\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        swapname,
                    );
                    if vim_dialog_yesno(
                        VIM_QUESTION as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut buff_0 as *mut ::core::ffi::c_char,
                        2 as ::core::ffi::c_int,
                    ) != VIM_YES as ::core::ffi::c_int
                    {
                        xfree(swapname as *mut ::core::ffi::c_void);
                        return FAIL;
                    }
                    (*eap).forceit = true_0;
                } else {
                    semsg(
                        gettext(
                            b"E768: Swap file exists: %s (:silent! overrides)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        swapname,
                    );
                    xfree(swapname as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            xfree(swapname as *mut ::core::ffi::c_void);
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_wnext(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0;
    if *(*eap).cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'n' as ::core::ffi::c_int
    {
        i = (*curwin).w_arg_idx + (*eap).line2 as ::core::ffi::c_int;
    } else {
        i = (*curwin).w_arg_idx - (*eap).line2 as ::core::ffi::c_int;
    }
    (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
    (*eap).line2 = (*curbuf).b_ml.ml_line_count;
    if do_write(eap) != FAIL {
        do_argfile(eap, i);
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_wqall(mut eap: *mut exarg_T) {
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_forceit: ::core::ffi::c_int = (*eap).forceit;
    let mut save_exiting: bool = exiting;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_xall as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_wqall as ::core::ffi::c_int
    {
        if before_quit_all(eap) == FAIL {
            return;
        }
        exiting = true_0 != 0;
    }
    let mut buf: *mut buf_T = firstbuf;
    's_136: while !buf.is_null() {
        's_32: {
            if exiting as ::core::ffi::c_int != 0 && (*eap).forceit == 0
                && !(*buf).terminal.is_null()
                && channel_job_running((*buf).b_p_channel as uint64_t)
                    as ::core::ffi::c_int != 0
            {
                no_write_message_buf(buf);
                error += 1;
            } else if !bufIsChanged(buf) || bt_dontwrite(buf) as ::core::ffi::c_int != 0
            {
                break 's_32;
            }
            if not_writing() {
                error += 1;
                break 's_136;
            } else {
                if (*buf).b_ffname.is_null() {
                    semsg(
                        gettext(
                            b"E141: No file name for buffer %ld\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        (*buf).handle as int64_t,
                    );
                    error += 1;
                } else if check_readonly(&raw mut (*eap).forceit, buf) != 0
                    || check_overwrite(
                        eap,
                        buf,
                        (*buf).b_fname,
                        (*buf).b_ffname,
                        false_0 != 0,
                    ) == FAIL
                {
                    error += 1;
                } else {
                    let mut bufref: bufref_T = bufref_T {
                        br_buf: ::core::ptr::null_mut::<buf_T>(),
                        br_fnum: 0,
                        br_buf_free_count: 0,
                    };
                    set_bufref(&raw mut bufref, buf);
                    if handle_mkdir_p_arg(eap, (*buf).b_fname) == FAIL
                        || buf_write_all(buf, (*eap).forceit != 0) == FAIL
                    {
                        error += 1;
                    }
                    if !bufref_valid(&raw mut bufref) {
                        buf = firstbuf;
                    }
                }
                (*eap).forceit = save_forceit;
            }
        }
        buf = (*buf).b_next;
    }
    if exiting {
        if error == 0 {
            getout(0 as ::core::ffi::c_int);
        }
        not_exiting(save_exiting);
    }
}
unsafe extern "C" fn not_writing() -> bool {
    if p_write != 0 {
        return false_0 != 0;
    }
    emsg(
        gettext(
            b"E142: File not written: Writing is disabled by 'write' option\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
    );
    return true_0 != 0;
}
unsafe extern "C" fn check_readonly(
    mut forceit: *mut ::core::ffi::c_int,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    if *forceit == 0
        && ((*buf).b_p_ro != 0
            || os_path_exists((*buf).b_ffname) as ::core::ffi::c_int != 0
                && os_file_is_writable((*buf).b_ffname) == 0)
    {
        if (p_confirm != 0
            || cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && !(*buf).b_fname.is_null()
        {
            let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
            if (*buf).b_p_ro != 0 {
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"'readonly' option is set for \"%s\".\nDo you wish to write anyway?\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    (*buf).b_fname,
                );
            } else {
                dialog_msg(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    gettext(
                        b"File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                    (*buf).b_fname,
                );
            }
            if vim_dialog_yesno(
                VIM_QUESTION as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                &raw mut buff as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
            ) == VIM_YES as ::core::ffi::c_int
            {
                *forceit = true_0;
                return false_0;
            }
            return true_0;
        } else if (*buf).b_p_ro != 0 {
            emsg(gettext(&raw const e_readonly as *const ::core::ffi::c_char));
        } else {
            semsg(
                gettext(
                    b"E505: \"%s\" is read-only (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                (*buf).b_fname,
            );
        }
        return true_0;
    }
    return false_0;
}
#[no_mangle]
pub unsafe extern "C" fn getfile(
    mut fnum: ::core::ffi::c_int,
    mut ffname_arg: *mut ::core::ffi::c_char,
    mut sfname_arg: *mut ::core::ffi::c_char,
    mut setpm: bool,
    mut lnum: linenr_T,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    if !check_can_set_curbuf_forceit(forceit as ::core::ffi::c_int) {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    let mut ffname: *mut ::core::ffi::c_char = ffname_arg;
    let mut sfname: *mut ::core::ffi::c_char = sfname_arg;
    let mut other: bool = false;
    let mut retval: ::core::ffi::c_int = 0;
    let mut free_me: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if text_locked() {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    if curbuf_locked() {
        return GETFILE_ERROR as ::core::ffi::c_int;
    }
    if fnum == 0 as ::core::ffi::c_int {
        fname_expand(curbuf, &raw mut ffname, &raw mut sfname);
        other = otherfile(ffname);
        free_me = ffname;
    } else {
        other = fnum != (*curbuf).handle;
    }
    if other {
        no_wait_return += 1;
    }
    '_theend: {
        if other as ::core::ffi::c_int != 0 && !forceit
            && (*curbuf).b_nwindows == 1 as ::core::ffi::c_int && !buf_hide(curbuf)
            && curbufIsChanged() as ::core::ffi::c_int != 0
            && autowrite(curbuf, forceit) == FAIL
        {
            if p_confirm != 0 && p_write != 0 {
                dialog_changed(curbuf, false_0 != 0);
            }
            if curbufIsChanged() {
                no_wait_return -= 1;
                no_write_message();
                retval = GETFILE_NOT_WRITTEN as ::core::ffi::c_int;
                break '_theend;
            }
        }
        if other {
            no_wait_return -= 1;
        }
        if setpm {
            setpcmark();
        }
        if !other {
            if lnum != 0 as linenr_T {
                (*curwin).w_cursor.lnum = lnum;
            }
            check_cursor_lnum(curwin);
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            retval = GETFILE_SAME_FILE as ::core::ffi::c_int;
        } else if do_ecmd(
            fnum,
            ffname,
            sfname,
            ::core::ptr::null_mut::<exarg_T>(),
            lnum,
            (if buf_hide(curbuf) as ::core::ffi::c_int != 0 {
                ECMD_HIDE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
                + (if forceit as ::core::ffi::c_int != 0 {
                    ECMD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
            curwin,
        ) == OK
        {
            retval = GETFILE_OPEN_OTHER as ::core::ffi::c_int;
        } else {
            retval = GETFILE_ERROR as ::core::ffi::c_int;
        }
    }
    xfree(free_me as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn set_swapcommand(
    mut command: *mut ::core::ffi::c_char,
    mut newlnum: linenr_T,
) -> bool {
    if command.is_null() && newlnum <= 0 as linenr_T
        || *get_vim_var_str(VV_SWAPCOMMAND) as ::core::ffi::c_int != NUL
    {
        return false_0 != 0;
    }
    let valsize: size_t = if !command.is_null() {
        strlen(command).wrapping_add(3 as size_t)
    } else {
        30 as size_t
    };
    let mut val: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    val.data = xmalloc(valsize) as *mut ::core::ffi::c_char;
    val.size = if !command.is_null() {
        vim_snprintf_safelen(
            val.data,
            valsize,
            b":%s\r\0".as_ptr() as *const ::core::ffi::c_char,
            command,
        )
    } else {
        vim_snprintf_safelen(
            val.data,
            valsize,
            b"%ldG\0".as_ptr() as *const ::core::ffi::c_char,
            newlnum as int64_t,
        )
    };
    set_vim_var_string(VV_SWAPCOMMAND, val.data, val.size as ptrdiff_t);
    xfree(val.data as *mut ::core::ffi::c_void);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_ecmd(
    mut fnum: ::core::ffi::c_int,
    mut ffname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut newlnum: linenr_T,
    mut flags: ::core::ffi::c_int,
    mut oldwin: *mut win_T,
) -> ::core::ffi::c_int {
    let mut other_file: bool = false;
    let mut oldbuf: ::core::ffi::c_int = 0;
    let mut auto_buf: bool = false_0 != 0;
    let mut new_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut did_set_swapcommand: bool = false_0 != 0;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut old_curbuf: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut free_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut topline: linenr_T = 0 as linenr_T;
    let mut newcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut solcol: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut did_get_winopts: bool = false_0 != 0;
    let mut readfile_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_inc_redrawing_disabled: bool = false_0 != 0;
    let mut so_ptr: *mut OptInt = if (*curwin).w_onebuf_opt.wo_so >= 0 as OptInt {
        &raw mut (*curwin).w_onebuf_opt.wo_so
    } else {
        &raw mut p_so
    };
    if !eap.is_null() {
        command = (*eap).do_ecmd_cmd;
    }
    set_bufref(&raw mut old_curbuf, curbuf);
    '_theend: {
        if fnum != 0 as ::core::ffi::c_int {
            if fnum == (*curbuf).handle {
                return OK;
            }
            other_file = true_0 != 0;
        } else {
            if sfname.is_null() {
                sfname = ffname;
            }
            if flags
                & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                != 0 && (ffname.is_null() || *ffname as ::core::ffi::c_int == NUL)
            {
                break '_theend;
            } else if ffname.is_null() {
                other_file = true_0 != 0;
            } else if *ffname as ::core::ffi::c_int == NUL
                && (*curbuf).b_ffname.is_null()
            {
                other_file = false_0 != 0;
            } else {
                if *ffname as ::core::ffi::c_int == NUL {
                    ffname = (*curbuf).b_ffname;
                    sfname = (*curbuf).b_fname;
                }
                free_fname = fix_fname(ffname);
                if !free_fname.is_null() {
                    ffname = free_fname;
                }
                other_file = otherfile(ffname);
            }
        }
        if !other_file && !(*curbuf).terminal.is_null() {
            check_arg_idx(curwin);
            maketitle();
            retval = OK;
        } else if (!other_file && flags & ECMD_OLDBUF as ::core::ffi::c_int == 0
            || (*curbuf).b_nwindows == 1 as ::core::ffi::c_int
                && flags
                    & (ECMD_HIDE as ::core::ffi::c_int
                        | ECMD_ADDBUF as ::core::ffi::c_int
                        | ECMD_ALTBUF as ::core::ffi::c_int) == 0)
            && check_changed(
                curbuf,
                (if p_awa != 0 {
                    CCGD_AW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                    | (if other_file as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_MULTWIN as ::core::ffi::c_int
                    })
                    | (if flags & ECMD_FORCEIT as ::core::ffi::c_int != 0 {
                        CCGD_FORCEIT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    | (if eap.is_null() {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_EXCMD as ::core::ffi::c_int
                    }),
            ) as ::core::ffi::c_int != 0
        {
            if fnum == 0 as ::core::ffi::c_int && other_file as ::core::ffi::c_int != 0
                && !ffname.is_null()
            {
                setaltfname(
                    ffname,
                    sfname,
                    if newlnum < 0 as linenr_T { 0 as linenr_T } else { newlnum },
                );
            }
        } else {
            reset_VIsual();
            if !oldwin.is_null() && !win_valid(oldwin) {
                oldwin = ::core::ptr::null_mut::<win_T>();
            }
            did_set_swapcommand = set_swapcommand(command, newlnum);
            if other_file {
                let prev_alt_fnum: ::core::ffi::c_int = (*curwin).w_alt_fnum;
                if flags
                    & (ECMD_ADDBUF as ::core::ffi::c_int
                        | ECMD_ALTBUF as ::core::ffi::c_int) == 0
                {
                    if cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        (*curwin).w_alt_fnum = (*curbuf).handle as ::core::ffi::c_int;
                    }
                    if !oldwin.is_null() {
                        buflist_altfpos(oldwin);
                    }
                }
                if fnum != 0 {
                    buf = buflist_findnr(fnum);
                } else if flags
                    & (ECMD_ADDBUF as ::core::ffi::c_int
                        | ECMD_ALTBUF as ::core::ffi::c_int) != 0
                {
                    let mut tlnum: linenr_T = 0 as linenr_T;
                    if !command.is_null() {
                        tlnum = atol(command) as linenr_T;
                        if tlnum <= 0 as linenr_T {
                            tlnum = 1 as ::core::ffi::c_int as linenr_T;
                        }
                    }
                    let newbuf: *const buf_T = buflist_new(
                        ffname,
                        sfname,
                        tlnum,
                        BLN_LISTED as ::core::ffi::c_int
                            | BLN_NOCURWIN as ::core::ffi::c_int,
                    );
                    if !newbuf.is_null()
                        && flags & ECMD_ALTBUF as ::core::ffi::c_int != 0
                    {
                        (*curwin).w_alt_fnum = (*newbuf).handle as ::core::ffi::c_int;
                    }
                    break '_theend;
                } else {
                    buf = buflist_new(
                        ffname,
                        sfname,
                        0 as linenr_T,
                        BLN_CURBUF as ::core::ffi::c_int
                            | (if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                BLN_LISTED as ::core::ffi::c_int
                            }),
                    );
                    if !oldwin.is_null() {
                        oldwin = curwin;
                    }
                    set_bufref(&raw mut old_curbuf, curbuf);
                }
                if buf.is_null() {
                    break '_theend;
                } else if (*buf).b_locked_split != 0 {
                    if oldwin.is_null() && !(*curwin).w_buffer.is_null()
                        && (*(*curwin).w_buffer).b_nwindows > 1 as ::core::ffi::c_int
                    {
                        (*(*curwin).w_buffer).b_nwindows -= 1;
                    }
                    emsg(
                        gettext(
                            &raw const e_cannot_switch_to_a_closing_buffer
                                as *const ::core::ffi::c_char,
                        ),
                    );
                    break '_theend;
                } else {
                    if (*curwin).w_alt_fnum == (*buf).handle
                        && prev_alt_fnum != 0 as ::core::ffi::c_int
                    {
                        (*curwin).w_alt_fnum = prev_alt_fnum;
                    }
                    if (*buf).b_ml.ml_mfp.is_null() {
                        oldbuf = false_0;
                    } else {
                        oldbuf = true_0;
                        set_bufref(&raw mut bufref, buf);
                        buf_check_timestamp(buf);
                        if !bufref_valid(&raw mut bufref) || curbuf != old_curbuf.br_buf
                        {
                            break '_theend;
                        } else if aborting() {
                            break '_theend;
                        }
                    }
                    if oldbuf != 0
                        && newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                        || newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                    {
                        let mut pos: *mut pos_T = &raw mut (*(buflist_findfmark
                            as unsafe extern "C" fn(*mut buf_T) -> *mut fmark_T)(buf))
                            .mark;
                        newlnum = (*pos).lnum;
                        solcol = (*pos).col as ::core::ffi::c_int;
                    }
                    if buf != curbuf {
                        '_c2rust_label: {
                            if cmdwin_buf.is_null() {} else {
                                __assert_fail(
                                    b"cmdwin_buf == NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_cmds.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2549 as ::core::ffi::c_uint,
                                    b"int do_ecmd(int, char *, char *, exarg_T *, linenr_T, int, win_T *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        let save_cmdwin_type: ::core::ffi::c_int = cmdwin_type;
                        let save_cmdwin_win: *mut win_T = cmdwin_win;
                        let save_cmdwin_old_curwin: *mut win_T = cmdwin_old_curwin;
                        cmdwin_type = 0 as ::core::ffi::c_int;
                        cmdwin_win = ::core::ptr::null_mut::<win_T>();
                        cmdwin_old_curwin = ::core::ptr::null_mut::<win_T>();
                        if !(*buf).b_fname.is_null() {
                            new_name = xstrdup((*buf).b_fname);
                        }
                        let save_au_new_curbuf: bufref_T = au_new_curbuf;
                        set_bufref(&raw mut au_new_curbuf, buf);
                        apply_autocmds(
                            EVENT_BUFLEAVE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            false_0 != 0,
                            curbuf,
                        );
                        cmdwin_type = save_cmdwin_type;
                        cmdwin_win = save_cmdwin_win;
                        cmdwin_old_curwin = save_cmdwin_old_curwin;
                        if !bufref_valid(&raw mut au_new_curbuf) {
                            delbuf_msg(new_name);
                            au_new_curbuf = save_au_new_curbuf;
                            break '_theend;
                        } else if aborting() {
                            xfree(new_name as *mut ::core::ffi::c_void);
                            au_new_curbuf = save_au_new_curbuf;
                            break '_theend;
                        } else {
                            if buf == curbuf {
                                auto_buf = true_0 != 0;
                            } else {
                                let mut the_curwin: *mut win_T = curwin;
                                let mut was_curbuf: *mut buf_T = curbuf;
                                (*the_curwin).w_locked = true_0 != 0;
                                (*buf).b_locked += 1;
                                if curbuf == old_curbuf.br_buf {
                                    buf_copy_options(buf, BCO_ENTER as ::core::ffi::c_int);
                                }
                                u_sync(false_0 != 0);
                                let did_decrement: bool = close_buffer(
                                    oldwin,
                                    curbuf,
                                    if flags & ECMD_HIDE as ::core::ffi::c_int != 0
                                        || !(*curbuf).terminal.is_null()
                                            && terminal_running((*curbuf).terminal)
                                                as ::core::ffi::c_int != 0
                                    {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        DOBUF_UNLOAD as ::core::ffi::c_int
                                    },
                                    false_0 != 0,
                                    false_0 != 0,
                                );
                                if win_valid(the_curwin) {
                                    (*the_curwin).w_locked = false_0 != 0;
                                }
                                (*buf).b_locked -= 1;
                                if aborting() as ::core::ffi::c_int != 0
                                    && !(*curwin).w_buffer.is_null()
                                {
                                    xfree(new_name as *mut ::core::ffi::c_void);
                                    au_new_curbuf = save_au_new_curbuf;
                                    break '_theend;
                                } else if !bufref_valid(&raw mut au_new_curbuf) {
                                    delbuf_msg(new_name);
                                    au_new_curbuf = save_au_new_curbuf;
                                    break '_theend;
                                } else {
                                    if buf == curbuf {
                                        if did_decrement as ::core::ffi::c_int != 0
                                            && buf_valid(was_curbuf) as ::core::ffi::c_int != 0
                                        {
                                            (*was_curbuf).b_nwindows += 1;
                                        }
                                        if win_valid_any_tab(oldwin) as ::core::ffi::c_int != 0
                                            && (*oldwin).w_buffer.is_null()
                                        {
                                            (*oldwin).w_buffer = was_curbuf;
                                        }
                                        auto_buf = true_0 != 0;
                                    } else {
                                        if (*curwin).w_buffer.is_null()
                                            || (*curwin).w_s == &raw mut (*(*curwin).w_buffer).b_s
                                        {
                                            (*curwin).w_s = &raw mut (*buf).b_s;
                                        }
                                        (*curwin).w_buffer = buf;
                                        curbuf = buf;
                                        (*curbuf).b_nwindows += 1;
                                        if oldbuf == 0 && !eap.is_null() {
                                            set_file_options(true_0 != 0, eap);
                                            set_forced_fenc(eap);
                                        }
                                    }
                                    get_winopts(curbuf);
                                    did_get_winopts = true_0 != 0;
                                }
                            }
                            xfree(new_name as *mut ::core::ffi::c_void);
                            au_new_curbuf = save_au_new_curbuf;
                        }
                    }
                    (*curwin).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
                    (*curwin).w_pcmark.col = 0 as ::core::ffi::c_int as colnr_T;
                }
            } else if flags
                & (ECMD_ADDBUF as ::core::ffi::c_int | ECMD_ALTBUF as ::core::ffi::c_int)
                != 0 || check_fname() == FAIL
            {
                break '_theend;
            } else {
                oldbuf = flags & ECMD_OLDBUF as ::core::ffi::c_int;
            }
            RedrawingDisabled += 1;
            did_inc_redrawing_disabled = true_0 != 0;
            buf = curbuf;
            if flags & ECMD_SET_HELP as ::core::ffi::c_int != 0
                || keep_help_flag as ::core::ffi::c_int != 0
            {
                prepare_help_buffer();
            } else if !(*curbuf).b_help {
                set_buflisted(true_0);
            }
            if buf == curbuf {
                if !aborting() {
                    (*curbuf).b_did_filetype = false_0 != 0;
                    if !other_file && oldbuf == 0 {
                        set_last_cursor(curwin);
                        if newlnum == ECMD_LAST as ::core::ffi::c_int as linenr_T
                            || newlnum == ECMD_LASTL as ::core::ffi::c_int as linenr_T
                        {
                            newlnum = (*curwin).w_cursor.lnum;
                            solcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
                        }
                        buf = curbuf;
                        if !(*buf).b_fname.is_null() {
                            new_name = xstrdup((*buf).b_fname);
                        } else {
                            new_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        set_bufref(&raw mut bufref, buf);
                        if (*curbuf).b_flags & BF_NEVERLOADED == 0
                            && (p_ur < 0 as OptInt
                                || (*curbuf).b_ml.ml_line_count as OptInt <= p_ur)
                        {
                            u_sync(false_0 != 0);
                            if u_savecommon(
                                curbuf,
                                0 as linenr_T,
                                (*curbuf).b_ml.ml_line_count + 1 as linenr_T,
                                0 as linenr_T,
                                true_0 != 0,
                            ) == FAIL
                            {
                                xfree(new_name as *mut ::core::ffi::c_void);
                                break '_theend;
                            } else {
                                u_unchanged(curbuf);
                                buf_freeall(curbuf, BFA_KEEP_UNDO as ::core::ffi::c_int);
                                readfile_flags = READ_KEEP_UNDO as ::core::ffi::c_int;
                            }
                        } else {
                            buf_freeall(curbuf, 0 as ::core::ffi::c_int);
                        }
                        if !bufref_valid(&raw mut bufref) {
                            delbuf_msg(new_name);
                            break '_theend;
                        } else {
                            xfree(new_name as *mut ::core::ffi::c_void);
                            if buf != curbuf {
                                break '_theend;
                            } else if aborting() {
                                break '_theend;
                            } else {
                                buf_clear_file(curbuf);
                                (*curbuf).b_op_start.lnum = 0 as ::core::ffi::c_int
                                    as linenr_T;
                                (*curbuf).b_op_end.lnum = 0 as ::core::ffi::c_int
                                    as linenr_T;
                            }
                        }
                    }
                    retval = OK;
                    if !other_file {
                        (*curbuf).b_flags &= !BF_NOTEDITED;
                    }
                    check_arg_idx(curwin);
                    if !auto_buf {
                        curwin_init();
                        let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
                        while !tp.is_null() {
                            let mut win: *mut win_T = if tp == curtab {
                                firstwin
                            } else {
                                (*tp).tp_firstwin
                            };
                            while !win.is_null() {
                                if (*win).w_buffer == curbuf {
                                    foldUpdateAll(win);
                                }
                                win = (*win).w_next;
                            }
                            tp = (*tp).tp_next as *mut tabpage_T;
                        }
                        do_autochdir();
                        let mut orig_pos: pos_T = (*curwin).w_cursor;
                        topline = (*curwin).w_topline;
                        if oldbuf == 0 {
                            swap_exists_action = SEA_DIALOG;
                            (*curbuf).b_flags |= BF_CHECK_RO;
                            if flags & ECMD_NOWINENTER as ::core::ffi::c_int != 0 {
                                readfile_flags |= READ_NOWINENTER as ::core::ffi::c_int;
                            }
                            if should_abort(
                                open_buffer(false_0 != 0, eap, readfile_flags),
                            ) {
                                retval = FAIL;
                            }
                            if swap_exists_action == SEA_QUIT {
                                retval = FAIL;
                            }
                            handle_swap_exists(&raw mut old_curbuf);
                        } else {
                            do_modelines(OPT_WINONLY as ::core::ffi::c_int);
                            apply_autocmds_retval(
                                EVENT_BUFENTER,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                false_0 != 0,
                                curbuf,
                                &raw mut retval,
                            );
                            if flags & ECMD_NOWINENTER as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                apply_autocmds_retval(
                                    EVENT_BUFWINENTER,
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                    false_0 != 0,
                                    curbuf,
                                    &raw mut retval,
                                );
                            }
                        }
                        check_arg_idx(curwin);
                        if !equalpos((*curwin).w_cursor, orig_pos) {
                            let mut text: *const ::core::ffi::c_char = get_cursor_line_ptr();
                            if (*curwin).w_cursor.lnum != orig_pos.lnum
                                || (*curwin).w_cursor.col
                                    != skipwhite(text).offset_from(text) as ::core::ffi::c_int
                            {
                                newlnum = (*curwin).w_cursor.lnum;
                                newcol = (*curwin).w_cursor.col as ::core::ffi::c_int;
                            }
                        }
                        if (*curwin).w_topline == topline {
                            topline = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        changed_line_abv_curs();
                        maketitle();
                    }
                    if (*curwin).w_onebuf_opt.wo_diff != 0 {
                        diff_buf_add(curbuf);
                        diff_invalidate(curbuf);
                    }
                    if did_get_winopts as ::core::ffi::c_int != 0
                        && (*curwin).w_onebuf_opt.wo_spell != 0
                        && *(*(*curwin).w_s).b_p_spl as ::core::ffi::c_int != NUL
                    {
                        parse_spelllang(curwin);
                    }
                    if command.is_null() {
                        if newcol >= 0 as ::core::ffi::c_int {
                            (*curwin).w_cursor.lnum = newlnum;
                            (*curwin).w_cursor.col = newcol as colnr_T;
                            check_cursor(curwin);
                        } else if newlnum > 0 as linenr_T {
                            (*curwin).w_cursor.lnum = newlnum;
                            check_cursor_lnum(curwin);
                            if solcol >= 0 as ::core::ffi::c_int && p_sol == 0 {
                                (*curwin).w_cursor.col = solcol as colnr_T;
                                check_cursor_col(curwin);
                                (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int
                                    as colnr_T;
                                (*curwin).w_set_curswant = true_0;
                            } else {
                                beginline(
                                    BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                                );
                            }
                        } else {
                            if exmode_active {
                                (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
                            }
                            beginline(
                                BL_WHITE as ::core::ffi::c_int
                                    | BL_FIX as ::core::ffi::c_int,
                            );
                        }
                    }
                    check_lnums(false_0 != 0);
                    if oldbuf != 0 && !auto_buf {
                        let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll;
                        if shortmess(SHM_OVERALL as ::core::ffi::c_int)
                            as ::core::ffi::c_int != 0 && msg_listdo_overwrite == 0
                            && !exiting && p_verbose == 0 as OptInt
                        {
                            msg_scroll = false_0;
                        }
                        if msg_scroll == 0 {
                            msg_check_for_delay(false_0 != 0);
                        }
                        msg_start();
                        msg_scroll = msg_scroll_save;
                        msg_scrolled_ign = true_0 != 0;
                        if !shortmess(SHM_FILEINFO as ::core::ffi::c_int) {
                            fileinfo(false_0, true_0, false_0 != 0);
                        }
                        msg_scrolled_ign = false_0 != 0;
                    }
                    (*curbuf).b_last_used = time(::core::ptr::null_mut::<time_t>());
                    if !command.is_null() {
                        do_cmdline(
                            command,
                            None,
                            NULL_0,
                            DOCMD_VERBOSE as ::core::ffi::c_int,
                        );
                    }
                    if (*curbuf).b_kmap_state as ::core::ffi::c_int & KEYMAP_INIT != 0 {
                        keymap_init();
                    }
                    RedrawingDisabled -= 1;
                    did_inc_redrawing_disabled = false_0 != 0;
                    if !skip_redraw {
                        let mut n: OptInt = *so_ptr;
                        if topline == 0 as linenr_T && command.is_null() {
                            *so_ptr = 999 as OptInt;
                        }
                        update_topline(curwin);
                        (*curwin).w_scbind_pos = plines_m_win_fill(
                            curwin,
                            1 as linenr_T,
                            (*curwin).w_topline,
                        );
                        *so_ptr = n;
                        redraw_curbuf_later(UPD_NOT_VALID as ::core::ffi::c_int);
                    }
                    do_autochdir();
                }
            }
        }
    }
    if bufref_valid(&raw mut old_curbuf) as ::core::ffi::c_int != 0
        && !(*old_curbuf.br_buf).terminal.is_null()
    {
        terminal_check_size((*old_curbuf.br_buf).terminal);
    }
    if (!bufref_valid(&raw mut old_curbuf) || curbuf != old_curbuf.br_buf)
        && !(*curbuf).terminal.is_null()
    {
        terminal_check_size((*curbuf).terminal);
    }
    if did_inc_redrawing_disabled {
        RedrawingDisabled -= 1;
    }
    if did_set_swapcommand {
        set_vim_var_string(
            VV_SWAPCOMMAND,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
    }
    xfree(free_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn delbuf_msg(mut name: *mut ::core::ffi::c_char) {
    semsg(
        gettext(
            b"E143: Autocommands unexpectedly deleted new buffer %s\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
        if name.is_null() {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            name as *const ::core::ffi::c_char
        },
    );
    xfree(name as *mut ::core::ffi::c_void);
    au_new_curbuf.br_buf = ::core::ptr::null_mut::<buf_T>();
    au_new_curbuf.br_buf_free_count = 0 as ::core::ffi::c_int;
}
static mut append_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn ex_append(mut eap: *mut exarg_T) {
    let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut did_undo: bool = false_0 != 0;
    let mut lnum: linenr_T = (*eap).line2;
    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut empty: bool = (*curbuf).b_ml.ml_flags & ML_EMPTY != 0;
    if (*eap).forceit != 0 {
        (*curbuf).b_p_ai = ((*curbuf).b_p_ai == 0) as ::core::ffi::c_int;
    }
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_change as ::core::ffi::c_int
        && (*curbuf).b_p_ai != 0 && lnum > 0 as linenr_T
    {
        append_indent = get_indent_lnum(lnum);
    }
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
        lnum -= 1;
    }
    if empty as ::core::ffi::c_int != 0 && lnum == 1 as linenr_T {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    State = MODE_INSERT as ::core::ffi::c_int;
    if (*curbuf).b_p_iminsert == B_IMODE_LMAP as OptInt {
        State |= MODE_LANGMAP as ::core::ffi::c_int;
    }
    loop {
        msg_scroll = true_0;
        need_wait_return = false_0 != 0;
        if (*curbuf).b_p_ai != 0 {
            if append_indent >= 0 as ::core::ffi::c_int {
                indent = append_indent;
                append_indent = -1 as ::core::ffi::c_int;
            } else if lnum > 0 as linenr_T {
                indent = get_indent_lnum(lnum);
            }
        }
        if *(*eap).arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
            theline = xstrdup((*eap).arg.offset(1 as ::core::ffi::c_int as isize));
            *(*eap).arg = NUL as ::core::ffi::c_char;
        } else if (*eap).ea_getline.is_none() {
            if (*eap).nextcmd.is_null() {
                break;
            }
            p = vim_strchr((*eap).nextcmd, NL);
            if p.is_null() {
                p = (*eap).nextcmd.offset(strlen((*eap).nextcmd) as isize);
            }
            theline = xmemdupz(
                (*eap).nextcmd as *const ::core::ffi::c_void,
                p.offset_from((*eap).nextcmd) as size_t,
            ) as *mut ::core::ffi::c_char;
            if *p as ::core::ffi::c_int != NUL {
                p = p.offset(1);
            } else {
                p = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*eap).nextcmd = p;
        } else {
            let mut save_State: ::core::ffi::c_int = State;
            State = MODE_CMDLINE as ::core::ffi::c_int;
            theline = (*eap)
                .ea_getline
                .expect(
                    "non-null function pointer",
                )(
                if (*(*eap).cstack).cs_looplevel > 0 as ::core::ffi::c_int {
                    -1 as ::core::ffi::c_int
                } else {
                    NUL
                },
                (*eap).cookie,
                indent,
                true_0 != 0,
            );
            State = save_State;
        }
        lines_left = Rows - 1 as ::core::ffi::c_int;
        if theline.is_null() {
            break;
        }
        let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        p = theline;
        while indent > vcol {
            if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                vcol += 1;
            } else {
                if *p as ::core::ffi::c_int != TAB {
                    break;
                }
                vcol += 8 as ::core::ffi::c_int - vcol % 8 as ::core::ffi::c_int;
            }
            p = p.offset(1);
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || !did_undo
                && u_save(
                    lnum,
                    lnum + 1 as linenr_T
                        + (if empty as ::core::ffi::c_int != 0 {
                            1 as linenr_T
                        } else {
                            0 as linenr_T
                        }),
                ) == FAIL
        {
            xfree(theline as *mut ::core::ffi::c_void);
            break;
        } else {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                *theline.offset(0 as ::core::ffi::c_int as isize) = NUL
                    as ::core::ffi::c_char;
            }
            did_undo = true_0 != 0;
            ml_append(lnum, theline, 0 as colnr_T, false_0 != 0);
            if empty {
                appended_lines(lnum, 1 as linenr_T);
            } else {
                appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
            }
            xfree(theline as *mut ::core::ffi::c_void);
            lnum += 1;
            if empty {
                ml_delete(2 as linenr_T);
                empty = false_0 != 0;
            }
        }
    }
    State = MODE_NORMAL as ::core::ffi::c_int;
    ui_cursor_shape();
    if (*eap).forceit != 0 {
        (*curbuf).b_p_ai = ((*curbuf).b_p_ai == 0) as ::core::ffi::c_int;
    }
    if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*curbuf).b_op_start.lnum = if (*eap).line2 < (*curbuf).b_ml.ml_line_count {
            (*eap).line2 + 1 as linenr_T
        } else {
            (*curbuf).b_ml.ml_line_count
        };
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
            (*curbuf).b_op_start.lnum -= 1;
        }
        (*curbuf).b_op_end.lnum = if (*eap).line2 < lnum {
            lnum
        } else {
            (*curbuf).b_op_start.lnum
        };
        (*curbuf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curbuf).b_op_start.col = (*curbuf).b_op_end.col;
    }
    (*curwin).w_cursor.lnum = lnum;
    check_cursor_lnum(curwin);
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    need_wait_return = false_0 != 0;
    ex_no_reprint = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ex_change(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = 0;
    if (*eap).line2 >= (*eap).line1
        && u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL
    {
        return;
    }
    if if (*eap).forceit != 0 {
        ((*curbuf).b_p_ai == 0) as ::core::ffi::c_int
    } else {
        (*curbuf).b_p_ai
    } != 0
    {
        append_indent = get_indent_lnum((*eap).line1);
    }
    lnum = (*eap).line2;
    while lnum >= (*eap).line1 {
        if (*curbuf).b_ml.ml_flags & ML_EMPTY != 0 {
            break;
        }
        ml_delete((*eap).line1);
        lnum -= 1;
    }
    check_cursor_lnum(curwin);
    deleted_lines_mark(
        (*eap).line1,
        (*eap).line2 as ::core::ffi::c_int - lnum as ::core::ffi::c_int,
    );
    (*eap).line2 = (*eap).line1;
    ex_append(eap);
}
#[no_mangle]
pub unsafe extern "C" fn ex_z(mut eap: *mut exarg_T) {
    let mut bigness: int64_t = 0;
    let mut minus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start: linenr_T = 0;
    let mut end: linenr_T = 0;
    let mut curs: linenr_T = 0;
    let mut lnum: linenr_T = (*eap).line2;
    if (*eap).forceit != 0 {
        bigness = (Rows - 1 as ::core::ffi::c_int) as int64_t;
    } else if firstwin == lastwin {
        bigness = ((*curwin).w_onebuf_opt.wo_scr * 2 as OptInt) as int64_t;
    } else {
        bigness = ((*curwin).w_view_height - 3 as ::core::ffi::c_int) as int64_t;
    }
    bigness = if bigness > 1 as int64_t { bigness } else { 1 as int64_t };
    let mut x: *mut ::core::ffi::c_char = (*eap).arg;
    let mut kind: *mut ::core::ffi::c_char = x;
    if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '^' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        x = x.offset(1);
    }
    while *x as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *x as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        x = x.offset(1);
    }
    if *x as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if !ascii_isdigit(*x as ::core::ffi::c_int) {
            emsg(
                gettext(
                    &raw const e_non_numeric_argument_to_z as *const ::core::ffi::c_char,
                ),
            );
            return;
        }
        bigness = atol(x) as int64_t;
        if bigness > (2 as linenr_T * (*curbuf).b_ml.ml_line_count) as int64_t
            || bigness < 0 as int64_t
        {
            bigness = (2 as linenr_T * (*curbuf).b_ml.ml_line_count) as int64_t;
        }
        p_window = bigness as ::core::ffi::c_int as OptInt;
        if *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
            bigness += 2 as int64_t;
        }
    }
    if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
    {
        x = kind.offset(1 as ::core::ffi::c_int as isize);
        while *x as ::core::ffi::c_int == *kind as ::core::ffi::c_int {
            x = x.offset(1);
        }
    }
    match *kind as ::core::ffi::c_int {
        45 => {
            start = lnum - bigness as linenr_T * x.offset_from(kind) as linenr_T
                + 1 as linenr_T;
            end = start + bigness as linenr_T - 1 as linenr_T;
            curs = end;
        }
        61 => {
            start = lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T
                + 1 as linenr_T;
            end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T
                - 1 as linenr_T;
            curs = lnum;
            minus = 1 as ::core::ffi::c_int;
        }
        94 => {
            start = lnum - bigness as linenr_T * 2 as linenr_T;
            end = lnum - bigness as linenr_T;
            curs = lnum - bigness as linenr_T;
        }
        46 => {
            start = lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T
                + 1 as linenr_T;
            end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T
                - 1 as linenr_T;
            curs = end;
        }
        _ => {
            start = lnum;
            if *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                start = (start as ::core::ffi::c_int
                    + (bigness as linenr_T
                        * (x.offset_from(kind) - 1 as isize) as linenr_T + 1 as linenr_T)
                        as ::core::ffi::c_int) as linenr_T;
            } else if (*eap).addr_count == 0 as ::core::ffi::c_int {
                start += 1;
            }
            end = start + bigness as linenr_T - 1 as linenr_T;
            curs = end;
        }
    }
    start = if start > 1 as linenr_T { start } else { 1 as linenr_T };
    end = if end < (*curbuf).b_ml.ml_line_count {
        end
    } else {
        (*curbuf).b_ml.ml_line_count
    };
    curs = if (if curs > 1 as linenr_T { curs } else { 1 as linenr_T })
        < (*curbuf).b_ml.ml_line_count
    {
        if curs > 1 as linenr_T { curs } else { 1 as linenr_T }
    } else {
        (*curbuf).b_ml.ml_line_count
    };
    let mut i: linenr_T = start;
    while i <= end {
        if minus != 0 && i == lnum {
            msg_putchar('\n' as ::core::ffi::c_int);
            let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while j < Columns {
                msg_putchar('-' as ::core::ffi::c_int);
                j += 1;
            }
        }
        print_line(
            i,
            (*eap).flags & EXFLAG_NR != 0,
            (*eap).flags & EXFLAG_LIST != 0,
            i == start,
        );
        if minus != 0 && i == lnum {
            msg_putchar('\n' as ::core::ffi::c_int);
            let mut j_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while j_0 < Columns {
                msg_putchar('-' as ::core::ffi::c_int);
                j_0 += 1;
            }
        }
        i += 1;
    }
    if (*curwin).w_cursor.lnum != curs {
        (*curwin).w_cursor.lnum = curs;
        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    ex_no_reprint = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn check_secure() -> bool {
    if secure != 0 {
        secure = 2 as ::core::ffi::c_int;
        emsg(gettext(&raw const e_curdir as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    if sandbox != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    return false_0 != 0;
}
static mut old_sub: SubReplacementString = SubReplacementString {
    sub: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    timestamp: 0 as Timestamp,
    additional_data: ::core::ptr::null_mut::<AdditionalData>(),
};
static mut global_need_beginline: ::core::ffi::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn sub_get_replacement(ret_sub: *mut SubReplacementString) {
    *ret_sub = old_sub;
}
#[no_mangle]
pub unsafe extern "C" fn sub_set_replacement(mut sub: SubReplacementString) {
    xfree(old_sub.sub as *mut ::core::ffi::c_void);
    if sub.additional_data != old_sub.additional_data {
        xfree(old_sub.additional_data as *mut ::core::ffi::c_void);
    }
    old_sub = sub;
}
unsafe extern "C" fn sub_joining_lines(
    mut eap: *mut exarg_T,
    mut pat: *mut ::core::ffi::c_char,
    mut patlen: size_t,
    mut sub: *const ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut save: bool,
    mut keeppatterns: bool,
) -> bool {
    if !pat.is_null()
        && strcmp(pat, b"\\n\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int && *sub as ::core::ffi::c_int == NUL
        && (*cmd as ::core::ffi::c_int == NUL
            || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                && (*cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int
                    || *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int))
    {
        if (*eap).skip != 0 {
            return true_0 != 0;
        }
        (*curwin).w_cursor.lnum = (*eap).line1;
        if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_LIST;
        } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_NR;
        } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
            (*eap).flags = EXFLAG_PRINT;
        }
        let mut joined_lines_count: linenr_T = (*eap).line2 - (*eap).line1
            + 1 as linenr_T
            + (if (*eap).line2 < (*curbuf).b_ml.ml_line_count {
                1 as linenr_T
            } else {
                0 as linenr_T
            });
        if joined_lines_count > 1 as linenr_T {
            do_join(
                joined_lines_count as size_t,
                false_0 != 0,
                true_0 != 0,
                false_0 != 0,
                true_0 != 0,
            );
            sub_nsubs = (joined_lines_count - 1 as linenr_T) as ::core::ffi::c_int;
            sub_nlines = 1 as ::core::ffi::c_int as linenr_T;
            do_sub_msg(false_0 != 0);
            ex_may_print(eap);
        }
        if save {
            if !keeppatterns {
                save_re_pat(
                    RE_SUBST as ::core::ffi::c_int,
                    pat,
                    patlen,
                    magic_isset() as ::core::ffi::c_int,
                );
            }
            add_to_history(
                HIST_SEARCH as ::core::ffi::c_int,
                pat,
                patlen,
                true_0 != 0,
                NUL,
            );
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn sub_grow_buf(
    mut new_start: *mut *mut ::core::ffi::c_char,
    mut new_start_len: *mut ::core::ffi::c_int,
    mut needed_len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut new_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if (*new_start).is_null() {
        *new_start_len = needed_len + 50 as ::core::ffi::c_int;
        *new_start = xcalloc(1 as size_t, *new_start_len as size_t)
            as *mut ::core::ffi::c_char;
        **new_start = NUL as ::core::ffi::c_char;
        new_end = *new_start;
    } else {
        let mut len: size_t = strlen(*new_start);
        needed_len += len as ::core::ffi::c_int;
        if needed_len > *new_start_len {
            let mut prev_new_start_len: size_t = *new_start_len as size_t;
            *new_start_len = needed_len + 50 as ::core::ffi::c_int;
            let mut added_len: size_t = (*new_start_len as size_t)
                .wrapping_sub(prev_new_start_len);
            *new_start = xrealloc(
                *new_start as *mut ::core::ffi::c_void,
                *new_start_len as size_t,
            ) as *mut ::core::ffi::c_char;
            memset(
                (*new_start).offset(prev_new_start_len as isize)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                added_len,
            );
        }
        new_end = (*new_start).offset(len as isize);
    }
    return new_end;
}
unsafe extern "C" fn sub_parse_flags(
    mut cmd: *mut ::core::ffi::c_char,
    mut subflags: *mut subflags_T,
    mut which_pat: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        cmd = cmd.offset(1);
    } else {
        (*subflags).do_all = p_gd != 0;
        (*subflags).do_ask = false_0 != 0;
        (*subflags).do_error = true_0 != 0;
        (*subflags).do_print = false_0 != 0;
        (*subflags).do_list = false_0 != 0;
        (*subflags).do_count = false_0 != 0;
        (*subflags).do_number = false_0 != 0;
        (*subflags).do_ic = kSubHonorOptions;
    }
    while *cmd != 0 {
        if *cmd as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
            (*subflags).do_all = !(*subflags).do_all;
        } else if *cmd as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
            (*subflags).do_ask = !(*subflags).do_ask;
        } else if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            (*subflags).do_count = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            (*subflags).do_error = !(*subflags).do_error;
        } else if *cmd as ::core::ffi::c_int == 'r' as ::core::ffi::c_int {
            *which_pat = RE_LAST as ::core::ffi::c_int;
        } else if *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
            (*subflags).do_number = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*subflags).do_print = true_0 != 0;
            (*subflags).do_list = true_0 != 0;
        } else if *cmd as ::core::ffi::c_int == 'i' as ::core::ffi::c_int {
            (*subflags).do_ic = kSubIgnoreCase;
        } else {
            if *cmd as ::core::ffi::c_int != 'I' as ::core::ffi::c_int {
                break;
            }
            (*subflags).do_ic = kSubMatchCase;
        }
        cmd = cmd.offset(1);
    }
    if (*subflags).do_count {
        (*subflags).do_ask = false_0 != 0;
    }
    return cmd;
}
unsafe extern "C" fn skip_substitute(
    mut start: *mut ::core::ffi::c_char,
    mut delimiter: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = start;
    while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == delimiter
        {
            let c2rust_fresh12 = p;
            p = p.offset(1);
            *c2rust_fresh12 = NUL as ::core::ffi::c_char;
            break;
        } else {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
    }
    return p;
}
unsafe extern "C" fn check_regexp_delim(
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if *(*__ctype_b_loc()).offset(c as isize) as ::core::ffi::c_int
        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        emsg(
            gettext(
                b"E146: Regular expressions can't be delimited by letters\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn do_sub(
    mut eap: *mut exarg_T,
    timeout: proftime_T,
    cmdpreview_ns: ::core::ffi::c_int,
    cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    static mut subflags: subflags_T = subflags_T {
        do_all: false_0 != 0,
        do_ask: false_0 != 0,
        do_count: false_0 != 0,
        do_error: true_0 != 0,
        do_print: false_0 != 0,
        do_list: false_0 != 0,
        do_number: false_0 != 0,
        do_ic: kSubHonorOptions,
    };
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut sub: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut patlen: size_t = 0 as size_t;
    let mut delimiter: ::core::ffi::c_int = 0;
    let mut has_second_delim: bool = false_0 != 0;
    let mut sublen: ::core::ffi::c_int = 0;
    let mut got_quit: bool = false_0 != 0;
    let mut got_match: bool = false_0 != 0;
    let mut which_pat: ::core::ffi::c_int = 0;
    let mut cmd: *mut ::core::ffi::c_char = (*eap).arg;
    let mut first_line: linenr_T = 0 as linenr_T;
    let mut last_line: linenr_T = 0 as linenr_T;
    let mut old_line_count: linenr_T = (*curbuf).b_ml.ml_line_count;
    let mut sub_firstline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut endcolumn: bool = false_0 != 0;
    let keeppatterns: bool = cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int
        != 0;
    let mut preview_lines: PreviewLines = PreviewLines {
        subresults: C2Rust_Unnamed_33 {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<SubResult>(),
        },
        lines_needed: 0 as linenr_T,
    };
    static mut pre_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut old_cursor: pos_T = (*curwin).w_cursor;
    let mut start_nsubs: ::core::ffi::c_int = 0;
    let mut did_save: bool = false_0 != 0;
    if global_busy == 0 {
        sub_nsubs = 0 as ::core::ffi::c_int;
        sub_nlines = 0 as ::core::ffi::c_int as linenr_T;
    }
    start_nsubs = sub_nsubs;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_tilde as ::core::ffi::c_int {
        which_pat = RE_LAST as ::core::ffi::c_int;
    } else {
        which_pat = RE_SUBST as ::core::ffi::c_int;
    }
    if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int && *cmd as ::core::ffi::c_int != NUL
        && !ascii_iswhite(*cmd as ::core::ffi::c_int)
        && vim_strchr(
                b"0123456789cegriIp|\"\0".as_ptr() as *const ::core::ffi::c_char,
                *cmd as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
    {
        if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
            return 0 as ::core::ffi::c_int;
        }
        if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            cmd = cmd.offset(1);
            if vim_strchr(
                    b"/?&\0".as_ptr() as *const ::core::ffi::c_char,
                    *cmd as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
            {
                emsg(gettext(&raw const e_backslash as *const ::core::ffi::c_char));
                return 0 as ::core::ffi::c_int;
            }
            if *cmd as ::core::ffi::c_int != '&' as ::core::ffi::c_int {
                which_pat = RE_SEARCH as ::core::ffi::c_int;
            }
            pat = b"\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
            patlen = 0 as size_t;
            let c2rust_fresh6 = cmd;
            cmd = cmd.offset(1);
            delimiter = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
            has_second_delim = true_0 != 0;
        } else {
            which_pat = RE_LAST as ::core::ffi::c_int;
            let c2rust_fresh7 = cmd;
            cmd = cmd.offset(1);
            delimiter = *c2rust_fresh7 as uint8_t as ::core::ffi::c_int;
            pat = cmd;
            cmd = skip_regexp_ex(
                cmd,
                delimiter,
                magic_isset() as ::core::ffi::c_int,
                &raw mut (*eap).arg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<magic_T>(),
            );
            if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == delimiter
            {
                let c2rust_fresh8 = cmd;
                cmd = cmd.offset(1);
                *c2rust_fresh8 = NUL as ::core::ffi::c_char;
                has_second_delim = true_0 != 0;
            }
            patlen = strlen(pat);
        }
        let mut p: *mut ::core::ffi::c_char = cmd;
        cmd = skip_substitute(cmd, delimiter);
        sub = xstrdup(p);
        if (*eap).skip == 0 && !keeppatterns && cmdpreview_ns <= 0 as ::core::ffi::c_int
        {
            sub_set_replacement(SubReplacementString {
                sub: xstrdup(sub),
                timestamp: os_time(),
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            });
        }
    } else if (*eap).skip == 0 {
        if old_sub.sub.is_null() {
            emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
            return 0 as ::core::ffi::c_int;
        }
        pat = ::core::ptr::null_mut::<::core::ffi::c_char>();
        patlen = 0 as size_t;
        sub = xstrdup(old_sub.sub);
        endcolumn = (*curwin).w_curswant == MAXCOL as ::core::ffi::c_int;
    }
    if !sub.is_null()
        && sub_joining_lines(
            eap,
            pat,
            patlen,
            sub,
            cmd,
            cmdpreview_ns <= 0 as ::core::ffi::c_int,
            keeppatterns,
        ) as ::core::ffi::c_int != 0
    {
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    cmd = sub_parse_flags(cmd, &raw mut subflags, &raw mut which_pat);
    let mut save_do_all: bool = subflags.do_all;
    let mut save_do_ask: bool = subflags.do_ask;
    cmd = skipwhite(cmd);
    if ascii_isdigit(*cmd as ::core::ffi::c_int) {
        let count_arg: *const ::core::ffi::c_char = cmd;
        i = getdigits_int(&raw mut cmd, false_0 != 0, INT_MAX);
        if i <= 0 as ::core::ffi::c_int && (*eap).skip == 0
            && subflags.do_error as ::core::ffi::c_int != 0
        {
            emsg(gettext(&raw const e_zerocount as *const ::core::ffi::c_char));
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        } else if i >= INT_MAX {
            semsg(
                gettext(&raw const e_val_too_large_len as *const ::core::ffi::c_char),
                cmd.offset_from(count_arg) as ::core::ffi::c_int,
                count_arg,
            );
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        (*eap).line1 = (*eap).line2;
        (*eap).line2 = ((*eap).line2 as ::core::ffi::c_int
            + (i as linenr_T - 1 as linenr_T) as ::core::ffi::c_int) as linenr_T;
        (*eap).line2 = if (*eap).line2 < (*curbuf).b_ml.ml_line_count {
            (*eap).line2
        } else {
            (*curbuf).b_ml.ml_line_count
        };
    }
    cmd = skipwhite(cmd);
    if *cmd as ::core::ffi::c_int != 0
        && *cmd as ::core::ffi::c_int != '"' as ::core::ffi::c_int
    {
        (*eap).nextcmd = check_nextcmd(cmd);
        if (*eap).nextcmd.is_null() {
            semsg(gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char), cmd);
            xfree(sub as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*eap).skip != 0 {
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !subflags.do_count && (*curbuf).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const ::core::ffi::c_char));
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if search_regcomp(
        pat,
        patlen,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        RE_SUBST as ::core::ffi::c_int,
        which_pat,
        (if cmdpreview_ns > 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            SEARCH_HIS as ::core::ffi::c_int
        }),
        &raw mut regmatch,
    ) == FAIL
    {
        if subflags.do_error {
            emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
        }
        xfree(sub as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if subflags.do_ic as ::core::ffi::c_uint
        == kSubIgnoreCase as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        regmatch.rmm_ic = true_0;
    } else if subflags.do_ic as ::core::ffi::c_uint
        == kSubMatchCase as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        regmatch.rmm_ic = false_0;
    }
    sub_firstline = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !sub.is_null() {} else {
            __assert_fail(
                b"sub != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/ex_cmds.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                3738 as ::core::ffi::c_uint,
                b"int do_sub(exarg_T *, const proftime_T, const int, const handle_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if *sub.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\\' as ::core::ffi::c_int
        && *sub.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        let mut p_0: *mut ::core::ffi::c_char = xstrdup(sub);
        xfree(sub as *mut ::core::ffi::c_void);
        sub = p_0;
    } else {
        let mut p_1: *mut ::core::ffi::c_char = regtilde(
            sub,
            magic_isset() as ::core::ffi::c_int,
            cmdpreview_ns > 0 as ::core::ffi::c_int,
        );
        if p_1 != sub {
            xfree(sub as *mut ::core::ffi::c_void);
            sub = p_1;
        }
    }
    let mut line2: linenr_T = (*eap).line2;
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= line2 && !got_quit && !aborting()
        && (cmdpreview_ns <= 0 as ::core::ffi::c_int
            || preview_lines.lines_needed <= p_cwh as linenr_T
            || lnum <= (*curwin).w_botline)
    {
        let mut nmatch: ::core::ffi::c_int = vim_regexec_multi(
            &raw mut regmatch,
            curwin,
            curbuf,
            lnum,
            0 as colnr_T,
            ::core::ptr::null_mut::<proftime_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if nmatch != 0 {
            let mut copycol: colnr_T = 0;
            let mut matchcol: colnr_T = 0;
            let mut prev_matchcol: colnr_T = MAXCOL as ::core::ffi::c_int;
            let mut new_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            let mut new_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            let mut new_start_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut p1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            let mut did_sub: bool = false_0 != 0;
            let mut lastone: ::core::ffi::c_int = 0;
            let mut nmatch_tl: linenr_T = 0 as linenr_T;
            let mut do_again: ::core::ffi::c_int = 0;
            let mut skip_match: bool = false_0 != 0;
            let mut sub_firstlnum: linenr_T = 0;
            let mut lnum_start: linenr_T = 0 as linenr_T;
            let mut line_matches: C2Rust_Unnamed_34 = C2Rust_Unnamed_34 {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<LineData>(),
            };
            sub_firstlnum = lnum;
            copycol = 0 as ::core::ffi::c_int as colnr_T;
            matchcol = 0 as ::core::ffi::c_int as colnr_T;
            if !got_match {
                setpcmark();
                got_match = true_0 != 0;
            }
            loop {
                let mut current_match: SubResult = SubResult {
                    start: lpos_T {
                        lnum: 0 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    end: lpos_T {
                        lnum: 0 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    pre_match: 0 as linenr_T,
                };
                if regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                    > 0 as linenr_T
                {
                    current_match.pre_match = lnum;
                    lnum += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    sub_firstlnum
                        += regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    nmatch
                        -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                            as ::core::ffi::c_int;
                    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut sub_firstline
                        as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    *ptr_;
                }
                current_match.start.lnum = sub_firstlnum;
                if lnum > (*curbuf).b_ml.ml_line_count {
                    break;
                }
                if sub_firstline.is_null() {
                    sub_firstline = xstrnsave(
                        ml_get(sub_firstlnum),
                        ml_get_len(sub_firstlnum) as size_t,
                    );
                }
                (*curwin).w_cursor.lnum = lnum;
                do_again = false_0;
                '_skip: {
                    if matchcol == prev_matchcol
                        && regmatch.endpos[0 as ::core::ffi::c_int as usize].lnum
                            == 0 as linenr_T
                        && matchcol
                            == regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                    {
                        if *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int
                            == NUL
                        {
                            skip_match = true_0 != 0;
                        } else {
                            matchcol
                                += utfc_ptr2len(sub_firstline.offset(matchcol as isize));
                        }
                        current_match.start.col = matchcol;
                        current_match.end.lnum = sub_firstlnum;
                        current_match.end.col = matchcol;
                    } else {
                        matchcol = regmatch.endpos[0 as ::core::ffi::c_int as usize].col;
                        prev_matchcol = matchcol;
                        if subflags.do_count {
                            if nmatch > 1 as ::core::ffi::c_int {
                                matchcol = strlen(sub_firstline) as colnr_T;
                                nmatch = 1 as ::core::ffi::c_int;
                                skip_match = true_0 != 0;
                            }
                            sub_nsubs += 1;
                            did_sub = true_0 != 0;
                            if !(*sub.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                && *sub.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int == '=' as ::core::ffi::c_int)
                            {
                                break '_skip;
                            }
                        }
                        if subflags.do_ask as ::core::ffi::c_int != 0
                            && cmdpreview_ns <= 0 as ::core::ffi::c_int
                        {
                            let mut typed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            let mut save_State: ::core::ffi::c_int = State;
                            (*curwin).w_cursor.col = regmatch
                                .startpos[0 as ::core::ffi::c_int as usize]
                                .col;
                            if (*curwin).w_onebuf_opt.wo_crb != 0 {
                                do_check_cursorbind();
                            }
                            if !vim_strchr(p_cpo, CPO_UNDO).is_null() {
                                no_u_sync += 1;
                            }
                            while subflags.do_ask {
                                if exmode_active {
                                    print_line_no_prefix(
                                        lnum,
                                        subflags.do_number,
                                        subflags.do_list,
                                    );
                                    let mut sc: colnr_T = 0;
                                    let mut ec: colnr_T = 0;
                                    getvcol(
                                        curwin,
                                        &raw mut (*curwin).w_cursor,
                                        &raw mut sc,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                    );
                                    (*curwin).w_cursor.col = (if regmatch
                                        .endpos[0 as ::core::ffi::c_int as usize]
                                        .col as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                        > 0 as ::core::ffi::c_int
                                    {
                                        regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    }) as colnr_T;
                                    getvcol(
                                        curwin,
                                        &raw mut (*curwin).w_cursor,
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        ::core::ptr::null_mut::<colnr_T>(),
                                        &raw mut ec,
                                    );
                                    (*curwin).w_cursor.col = regmatch
                                        .startpos[0 as ::core::ffi::c_int as usize]
                                        .col;
                                    if subflags.do_number as ::core::ffi::c_int != 0
                                        || (*curwin).w_onebuf_opt.wo_nu != 0
                                    {
                                        let mut numw: ::core::ffi::c_int = number_width(curwin)
                                            + 1 as ::core::ffi::c_int;
                                        sc += numw;
                                        ec += numw;
                                    }
                                    let mut prompt: *mut ::core::ffi::c_char = xmallocz(
                                        (ec as size_t).wrapping_add(1 as size_t),
                                    ) as *mut ::core::ffi::c_char;
                                    memset(
                                        prompt as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        sc as size_t,
                                    );
                                    memset(
                                        prompt.offset(sc as isize) as *mut ::core::ffi::c_void,
                                        '^' as ::core::ffi::c_int,
                                        ((ec - sc) as size_t).wrapping_add(1 as size_t),
                                    );
                                    let mut resp: *mut ::core::ffi::c_char = getcmdline_prompt(
                                        -1 as ::core::ffi::c_int,
                                        prompt,
                                        0 as ::core::ffi::c_int,
                                        EXPAND_NOTHING as ::core::ffi::c_int,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        Callback {
                                            data: C2Rust_Unnamed_5 {
                                                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            },
                                            type_0: kCallbackNone,
                                        },
                                        false_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                    if !ui_has(kUIMessages) {
                                        msg_putchar('\n' as ::core::ffi::c_int);
                                    }
                                    xfree(prompt as *mut ::core::ffi::c_void);
                                    if !resp.is_null() {
                                        typed = *resp as uint8_t as ::core::ffi::c_int;
                                        xfree(resp as *mut ::core::ffi::c_void);
                                    } else {
                                        typed = NUL;
                                    }
                                    if ex_normal_busy != 0 && typed == NUL {
                                        typed = 'q' as ::core::ffi::c_int;
                                    }
                                } else {
                                    let mut orig_line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                                        ::core::ffi::c_char,
                                    >();
                                    let mut len_change: ::core::ffi::c_int = 0
                                        as ::core::ffi::c_int;
                                    let save_p_lz: bool = p_lz != 0;
                                    let mut save_p_fen: ::core::ffi::c_int = (*curwin)
                                        .w_onebuf_opt
                                        .wo_fen;
                                    (*curwin).w_onebuf_opt.wo_fen = false_0;
                                    let mut temp: ::core::ffi::c_int = RedrawingDisabled;
                                    RedrawingDisabled = 0 as ::core::ffi::c_int;
                                    p_lz = false_0;
                                    if !new_start.is_null() {
                                        orig_line = xstrnsave(
                                            ml_get(lnum),
                                            ml_get_len(lnum) as size_t,
                                        );
                                        let mut new_line: *mut ::core::ffi::c_char = concat_str(
                                            new_start,
                                            sub_firstline.offset(copycol as isize),
                                        );
                                        len_change = strlen(new_line) as ::core::ffi::c_int
                                            - strlen(orig_line) as ::core::ffi::c_int;
                                        (*curwin).w_cursor.col += len_change;
                                        ml_replace(lnum, new_line, false_0 != 0);
                                    }
                                    search_match_lines = regmatch
                                        .endpos[0 as ::core::ffi::c_int as usize]
                                        .lnum
                                        - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                                    search_match_endcol = (regmatch
                                        .endpos[0 as ::core::ffi::c_int as usize]
                                        .col as ::core::ffi::c_int + len_change) as colnr_T;
                                    if search_match_lines == 0 as linenr_T
                                        && search_match_endcol == 0 as ::core::ffi::c_int
                                    {
                                        search_match_endcol = 1 as ::core::ffi::c_int as colnr_T;
                                    }
                                    highlight_match = true_0 != 0;
                                    update_topline(curwin);
                                    validate_cursor(curwin);
                                    redraw_later(curwin, UPD_SOME_VALID as ::core::ffi::c_int);
                                    show_cursor_info_later(true_0 != 0);
                                    update_screen();
                                    redraw_later(curwin, UPD_SOME_VALID as ::core::ffi::c_int);
                                    (*curwin).w_onebuf_opt.wo_fen = save_p_fen;
                                    let mut p_2: *mut ::core::ffi::c_char = gettext(
                                        b"replace with %s? (y)es/(n)o/(a)ll/(q)uit/(l)ast/scroll up(^E)/down(^Y)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    snprintf(
                                        &raw mut IObuff as *mut ::core::ffi::c_char,
                                        IOSIZE as size_t,
                                        p_2,
                                        sub,
                                    );
                                    p_2 = xstrdup(&raw mut IObuff as *mut ::core::ffi::c_char);
                                    typed = prompt_for_input(
                                        p_2,
                                        HLF_R as ::core::ffi::c_int,
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                    highlight_match = false_0 != 0;
                                    xfree(p_2 as *mut ::core::ffi::c_void);
                                    msg_didout = false_0 != 0;
                                    gotocmdline(true_0 != 0);
                                    p_lz = save_p_lz as ::core::ffi::c_int;
                                    RedrawingDisabled = temp;
                                    if !orig_line.is_null() {
                                        ml_replace(lnum, orig_line, false_0 != 0);
                                    }
                                }
                                need_wait_return = false_0 != 0;
                                if typed == 'q' as ::core::ffi::c_int || typed == ESC
                                    || typed == Ctrl_C
                                {
                                    got_quit = true_0 != 0;
                                    break;
                                } else {
                                    if typed == 'n' as ::core::ffi::c_int {
                                        break;
                                    }
                                    if typed == 'y' as ::core::ffi::c_int {
                                        break;
                                    }
                                    if typed == 'l' as ::core::ffi::c_int {
                                        subflags.do_all = false_0 != 0;
                                        line2 = lnum;
                                        break;
                                    } else if typed == 'a' as ::core::ffi::c_int {
                                        subflags.do_ask = false_0 != 0;
                                        break;
                                    } else if typed == Ctrl_E {
                                        scrollup_clamp();
                                    } else if typed == Ctrl_Y {
                                        scrolldown_clamp();
                                    }
                                }
                            }
                            State = save_State;
                            setmouse();
                            if !vim_strchr(p_cpo, CPO_UNDO).is_null() {
                                no_u_sync -= 1;
                            }
                            if typed == 'n' as ::core::ffi::c_int {
                                if nmatch > 1 as ::core::ffi::c_int {
                                    matchcol = strlen(sub_firstline) as colnr_T;
                                    skip_match = true_0 != 0;
                                }
                                break '_skip;
                            } else if got_quit {
                                break '_skip;
                            }
                        }
                        (*curwin).w_cursor.col = regmatch
                            .startpos[0 as ::core::ffi::c_int as usize]
                            .col;
                        if nmatch as linenr_T
                            > (*curbuf).b_ml.ml_line_count - sub_firstlnum
                                + 1 as linenr_T
                        {
                            nmatch = ((*curbuf).b_ml.ml_line_count - sub_firstlnum
                                + 1 as linenr_T) as ::core::ffi::c_int;
                            current_match.end.lnum = sub_firstlnum + nmatch as linenr_T;
                            skip_match = true_0 != 0;
                            if nmatch < 0 as ::core::ffi::c_int {
                                break '_skip;
                            }
                        }
                        if cmdpreview_ns > 0 as ::core::ffi::c_int && !has_second_delim {
                            current_match.start.col = regmatch
                                .startpos[0 as ::core::ffi::c_int as usize]
                                .col;
                            if current_match.end.lnum == 0 as linenr_T {
                                current_match.end.lnum = sub_firstlnum + nmatch as linenr_T
                                    - 1 as linenr_T;
                            }
                            current_match.end.col = regmatch
                                .endpos[0 as ::core::ffi::c_int as usize]
                                .col;
                            if nmatch > 1 as ::core::ffi::c_int {
                                sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                    + (nmatch as linenr_T - 1 as linenr_T)
                                        as ::core::ffi::c_int) as linenr_T;
                                xfree(sub_firstline as *mut ::core::ffi::c_void);
                                sub_firstline = xstrnsave(
                                    ml_get(sub_firstlnum),
                                    ml_get_len(sub_firstlnum) as size_t,
                                );
                                if sub_firstlnum <= line2 {
                                    do_again = true_0;
                                } else {
                                    subflags.do_all = false_0 != 0;
                                }
                            }
                            if skip_match {
                                xfree(sub_firstline as *mut ::core::ffi::c_void);
                                sub_firstline = xstrdup(
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                copycol = 0 as ::core::ffi::c_int as colnr_T;
                            }
                            lnum = (lnum as ::core::ffi::c_int
                                + (nmatch as linenr_T - 1 as linenr_T)
                                    as ::core::ffi::c_int) as linenr_T;
                        } else if cmdpreview_ns <= 0 as ::core::ffi::c_int
                            || has_second_delim as ::core::ffi::c_int != 0
                        {
                            lnum_start = lnum;
                            let mut save_ma: ::core::ffi::c_int = (*curbuf).b_p_ma;
                            let mut save_sandbox: ::core::ffi::c_int = sandbox;
                            if subflags.do_count {
                                (*curbuf).b_p_ma = false_0;
                                sandbox += 1;
                            }
                            let mut subflags_save: subflags_T = subflags;
                            textlock += 1;
                            sublen = vim_regsub_multi(
                                &raw mut regmatch,
                                sub_firstlnum
                                    - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum,
                                sub,
                                sub_firstline,
                                0 as ::core::ffi::c_int,
                                REGSUB_BACKSLASH as ::core::ffi::c_int
                                    | (if magic_isset() as ::core::ffi::c_int != 0 {
                                        REGSUB_MAGIC as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    }),
                            );
                            textlock -= 1;
                            subflags = subflags_save;
                            if sublen == 0 as ::core::ffi::c_int
                                || aborting() as ::core::ffi::c_int != 0
                                || subflags.do_count as ::core::ffi::c_int != 0
                            {
                                (*curbuf).b_p_ma = save_ma;
                                sandbox = save_sandbox;
                            } else {
                                if nmatch == 1 as ::core::ffi::c_int {
                                    p1 = sub_firstline;
                                } else {
                                    let mut lastlnum: linenr_T = sub_firstlnum
                                        + nmatch as linenr_T - 1 as linenr_T;
                                    p1 = ml_get(lastlnum);
                                    nmatch_tl = (nmatch_tl as ::core::ffi::c_int
                                        + (nmatch - 1 as ::core::ffi::c_int)) as linenr_T;
                                }
                                let mut copy_len: ::core::ffi::c_int = regmatch
                                    .startpos[0 as ::core::ffi::c_int as usize]
                                    .col as ::core::ffi::c_int - copycol as ::core::ffi::c_int;
                                new_end = sub_grow_buf(
                                    &raw mut new_start,
                                    &raw mut new_start_len,
                                    strlen(p1) as ::core::ffi::c_int
                                        - regmatch.endpos[0 as ::core::ffi::c_int as usize].col
                                            as ::core::ffi::c_int + copy_len + sublen
                                        + 1 as ::core::ffi::c_int,
                                );
                                memmove(
                                    new_end as *mut ::core::ffi::c_void,
                                    sub_firstline.offset(copycol as isize)
                                        as *const ::core::ffi::c_void,
                                    copy_len as size_t,
                                );
                                new_end = new_end.offset(copy_len as isize);
                                if new_start_len - copy_len < sublen {
                                    sublen = new_start_len - copy_len - 1 as ::core::ffi::c_int;
                                }
                                let mut start_col: ::core::ffi::c_int = new_end
                                    .offset_from(new_start) as ::core::ffi::c_int;
                                current_match.start.col = start_col as colnr_T;
                                textlock += 1;
                                vim_regsub_multi(
                                    &raw mut regmatch,
                                    sub_firstlnum
                                        - regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum,
                                    sub,
                                    new_end,
                                    sublen,
                                    REGSUB_COPY as ::core::ffi::c_int
                                        | REGSUB_BACKSLASH as ::core::ffi::c_int
                                        | (if magic_isset() as ::core::ffi::c_int != 0 {
                                            REGSUB_MAGIC as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        }),
                                );
                                textlock -= 1;
                                sub_nsubs += 1;
                                did_sub = true_0 != 0;
                                (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                                copycol = regmatch
                                    .endpos[0 as ::core::ffi::c_int as usize]
                                    .col;
                                if nmatch > 1 as ::core::ffi::c_int {
                                    sub_firstlnum = (sub_firstlnum as ::core::ffi::c_int
                                        + (nmatch as linenr_T - 1 as linenr_T)
                                            as ::core::ffi::c_int) as linenr_T;
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline = xstrnsave(
                                        ml_get(sub_firstlnum),
                                        ml_get_len(sub_firstlnum) as size_t,
                                    );
                                    if sub_firstlnum <= line2 {
                                        do_again = true_0;
                                    } else {
                                        subflags.do_all = false_0 != 0;
                                    }
                                }
                                if skip_match {
                                    xfree(sub_firstline as *mut ::core::ffi::c_void);
                                    sub_firstline = xstrdup(
                                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    copycol = 0 as ::core::ffi::c_int as colnr_T;
                                }
                                let mut replaced_bytes: bcount_t = 0 as bcount_t;
                                let mut start: lpos_T = regmatch
                                    .startpos[0 as ::core::ffi::c_int as usize];
                                let mut end: lpos_T = regmatch
                                    .endpos[0 as ::core::ffi::c_int as usize];
                                i = 0 as ::core::ffi::c_int;
                                while i < nmatch - 1 as ::core::ffi::c_int {
                                    replaced_bytes
                                        += strlen(ml_get(lnum_start + i as linenr_T)) as bcount_t
                                            + 1 as bcount_t;
                                    i += 1;
                                }
                                replaced_bytes += (end.col - start.col) as bcount_t;
                                let mut lnum_before_newlines: linenr_T = lnum;
                                p1 = new_end;
                                while *p1 != 0 {
                                    if *p1.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                        && *p1.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int != NUL
                                    {
                                        sublen -= 1;
                                        memmove(
                                            p1 as *mut ::core::ffi::c_void,
                                            p1.offset(1 as ::core::ffi::c_int as isize)
                                                as *const ::core::ffi::c_void,
                                            strlen(p1.offset(1 as ::core::ffi::c_int as isize))
                                                .wrapping_add(1 as size_t),
                                        );
                                    } else if *p1 as ::core::ffi::c_int == CAR {
                                        if u_inssub(lnum) == OK {
                                            *p1 = NUL as ::core::ffi::c_char;
                                            ml_append(
                                                lnum - 1 as linenr_T,
                                                new_start,
                                                (p1.offset_from(new_start) + 1 as isize) as colnr_T,
                                                false_0 != 0,
                                            );
                                            mark_adjust(
                                                lnum + 1 as linenr_T,
                                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                                1 as linenr_T,
                                                0 as linenr_T,
                                                kExtmarkNOOP,
                                            );
                                            if subflags.do_ask {
                                                appended_lines(lnum - 1 as linenr_T, 1 as linenr_T);
                                            } else {
                                                if first_line == 0 as linenr_T {
                                                    first_line = lnum;
                                                }
                                                last_line = lnum + 1 as linenr_T;
                                            }
                                            sub_firstlnum += 1;
                                            lnum += 1;
                                            line2 += 1;
                                            (*curwin).w_cursor.lnum += 1;
                                            memmove(
                                                new_start as *mut ::core::ffi::c_void,
                                                p1.offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(p1.offset(1 as ::core::ffi::c_int as isize))
                                                    .wrapping_add(1 as size_t),
                                            );
                                            p1 = new_start.offset(-(1 as ::core::ffi::c_int as isize));
                                        }
                                    } else {
                                        p1 = p1
                                            .offset(
                                                (utfc_ptr2len(p1) - 1 as ::core::ffi::c_int) as isize,
                                            );
                                    }
                                    p1 = p1.offset(1);
                                }
                                let mut new_endcol: colnr_T = strlen(new_start) as colnr_T;
                                current_match.end.col = new_endcol;
                                current_match.end.lnum = lnum;
                                let mut matchcols: ::core::ffi::c_int = end.col
                                    as ::core::ffi::c_int
                                    - (if end.lnum == start.lnum {
                                        start.col as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    });
                                let mut subcols: ::core::ffi::c_int = new_endcol
                                    as ::core::ffi::c_int
                                    - (if lnum == lnum_start {
                                        start_col
                                    } else {
                                        0 as ::core::ffi::c_int
                                    });
                                if !did_save {
                                    u_save_cursor();
                                    did_save = true_0 != 0;
                                }
                                if line_matches.size == line_matches.capacity {
                                    line_matches.capacity = (if line_matches.capacity != 0 {
                                        line_matches.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    });
                                    line_matches.items = xrealloc(
                                        line_matches.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<LineData>()
                                            .wrapping_mul(line_matches.capacity),
                                    ) as *mut LineData;
                                } else {};
                                let c2rust_fresh9 = line_matches.size;
                                line_matches.size = line_matches.size.wrapping_add(1);
                                let mut data: *mut LineData = line_matches
                                    .items
                                    .offset(c2rust_fresh9 as isize);
                                (*data).start_col = start_col;
                                (*data).start = start;
                                (*data).end = end;
                                (*data).matchcols = matchcols;
                                (*data).matchbytes = replaced_bytes;
                                (*data).subcols = subcols;
                                (*data).subbytes = (sublen - 1 as ::core::ffi::c_int)
                                    as bcount_t;
                                (*data).lnum_before = lnum_before_newlines;
                                (*data).lnum_after = lnum;
                            }
                        }
                    }
                }
                lastone = (skip_match as ::core::ffi::c_int != 0
                    || got_int as ::core::ffi::c_int != 0
                    || got_quit as ::core::ffi::c_int != 0 || lnum > line2
                    || !(subflags.do_all as ::core::ffi::c_int != 0 || do_again != 0)
                    || *sub_firstline.offset(matchcol as isize) as ::core::ffi::c_int
                        == NUL && nmatch <= 1 as ::core::ffi::c_int
                        && re_multiline(regmatch.regprog) == 0) as ::core::ffi::c_int;
                nmatch = -1 as ::core::ffi::c_int;
                if lastone != 0 || nmatch_tl > 0 as linenr_T
                    || {
                        nmatch = vim_regexec_multi(
                            &raw mut regmatch,
                            curwin,
                            curbuf,
                            sub_firstlnum,
                            matchcol,
                            ::core::ptr::null_mut::<proftime_T>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                        nmatch == 0 as ::core::ffi::c_int
                    }
                    || regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum
                        > 0 as linenr_T
                {
                    if !new_start.is_null() {
                        strcat(new_start, sub_firstline.offset(copycol as isize));
                        matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                        prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                        if u_savesub(lnum) != OK {
                            break;
                        }
                        ml_replace(lnum, new_start, true_0 != 0);
                        let mut match_idx: size_t = 0 as size_t;
                        while match_idx < line_matches.size {
                            let mut match_0: *mut LineData = line_matches
                                .items
                                .offset(match_idx as isize);
                            extmark_splice(
                                curbuf,
                                (*match_0).lnum_before as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                                (*match_0).start_col as colnr_T,
                                (*match_0).end.lnum as ::core::ffi::c_int
                                    - (*match_0).start.lnum as ::core::ffi::c_int,
                                (*match_0).matchcols as colnr_T,
                                (*match_0).matchbytes,
                                (*match_0).lnum_after as ::core::ffi::c_int
                                    - (*match_0).lnum_before as ::core::ffi::c_int,
                                (*match_0).subcols as colnr_T,
                                (*match_0).subbytes,
                                kExtmarkUndo,
                            );
                            match_idx = match_idx.wrapping_add(1);
                        }
                        line_matches.size = 0 as size_t;
                        if nmatch_tl > 0 as linenr_T {
                            lnum += 1;
                            if u_savedel(lnum, nmatch_tl) != OK {
                                break;
                            }
                            i = 0 as ::core::ffi::c_int;
                            while (i as linenr_T) < nmatch_tl {
                                ml_delete(lnum);
                                i += 1;
                            }
                            mark_adjust(
                                lnum,
                                lnum + nmatch_tl - 1 as linenr_T,
                                MAXLNUM as ::core::ffi::c_int as linenr_T,
                                -nmatch_tl,
                                kExtmarkNOOP,
                            );
                            if subflags.do_ask {
                                deleted_lines(lnum, nmatch_tl);
                            }
                            lnum -= 1;
                            line2 -= nmatch_tl;
                            nmatch_tl = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if subflags.do_ask {
                            changed_bytes(lnum, 0 as colnr_T);
                        } else {
                            if first_line == 0 as linenr_T {
                                first_line = lnum;
                            }
                            last_line = lnum + 1 as linenr_T;
                        }
                        sub_firstlnum = lnum;
                        xfree(sub_firstline as *mut ::core::ffi::c_void);
                        sub_firstline = new_start;
                        new_start = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        matchcol = strlen(sub_firstline) as colnr_T - matchcol;
                        prev_matchcol = strlen(sub_firstline) as colnr_T - prev_matchcol;
                        copycol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    if nmatch == -1 as ::core::ffi::c_int && lastone == 0 {
                        nmatch = vim_regexec_multi(
                            &raw mut regmatch,
                            curwin,
                            curbuf,
                            sub_firstlnum,
                            matchcol,
                            ::core::ptr::null_mut::<proftime_T>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                    }
                    if nmatch <= 0 as ::core::ffi::c_int {
                        if nmatch == -1 as ::core::ffi::c_int {
                            lnum
                                -= regmatch.startpos[0 as ::core::ffi::c_int as usize].lnum;
                        }
                        if cmdpreview_ns > 0 as ::core::ffi::c_int {
                            let mut match_lines: linenr_T = current_match.end.lnum
                                - current_match.start.lnum + 1 as linenr_T;
                            if preview_lines.subresults.size > 0 as size_t {
                                let mut last: linenr_T = (*preview_lines
                                    .subresults
                                    .items
                                    .offset(
                                        preview_lines
                                            .subresults
                                            .size
                                            .wrapping_sub(0 as size_t)
                                            .wrapping_sub(1 as size_t) as isize,
                                    ))
                                    .end
                                    .lnum;
                                if last == current_match.start.lnum {
                                    preview_lines.lines_needed = (preview_lines.lines_needed
                                        as ::core::ffi::c_int
                                        + (match_lines - 1 as linenr_T) as ::core::ffi::c_int)
                                        as linenr_T;
                                } else {
                                    preview_lines.lines_needed += match_lines;
                                }
                            } else {
                                preview_lines.lines_needed += match_lines;
                            }
                            if preview_lines.subresults.size
                                == preview_lines.subresults.capacity
                            {
                                preview_lines.subresults.capacity = (if preview_lines
                                    .subresults
                                    .capacity != 0
                                {
                                    preview_lines.subresults.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    8 as size_t
                                });
                                preview_lines.subresults.items = xrealloc(
                                    preview_lines.subresults.items as *mut ::core::ffi::c_void,
                                    ::core::mem::size_of::<SubResult>()
                                        .wrapping_mul(preview_lines.subresults.capacity),
                                ) as *mut SubResult;
                            } else {};
                            let c2rust_fresh10 = preview_lines.subresults.size;
                            preview_lines.subresults.size = preview_lines
                                .subresults
                                .size
                                .wrapping_add(1);
                            *preview_lines
                                .subresults
                                .items
                                .offset(c2rust_fresh10 as isize) = current_match;
                        }
                        break;
                    }
                }
                if cmdpreview_ns > 0 as ::core::ffi::c_int {
                    let mut match_lines_0: linenr_T = current_match.end.lnum
                        - current_match.start.lnum + 1 as linenr_T;
                    if preview_lines.subresults.size > 0 as size_t {
                        let mut last_0: linenr_T = (*preview_lines
                            .subresults
                            .items
                            .offset(
                                preview_lines
                                    .subresults
                                    .size
                                    .wrapping_sub(0 as size_t)
                                    .wrapping_sub(1 as size_t) as isize,
                            ))
                            .end
                            .lnum;
                        if last_0 == current_match.start.lnum {
                            preview_lines.lines_needed = (preview_lines.lines_needed
                                as ::core::ffi::c_int
                                + (match_lines_0 - 1 as linenr_T) as ::core::ffi::c_int)
                                as linenr_T;
                        } else {
                            preview_lines.lines_needed += match_lines_0;
                        }
                    } else {
                        preview_lines.lines_needed += match_lines_0;
                    }
                    if preview_lines.subresults.size == preview_lines.subresults.capacity
                    {
                        preview_lines.subresults.capacity = (if preview_lines
                            .subresults
                            .capacity != 0
                        {
                            preview_lines.subresults.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        });
                        preview_lines.subresults.items = xrealloc(
                            preview_lines.subresults.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<SubResult>()
                                .wrapping_mul(preview_lines.subresults.capacity),
                        ) as *mut SubResult;
                    } else {};
                    let c2rust_fresh11 = preview_lines.subresults.size;
                    preview_lines.subresults.size = preview_lines
                        .subresults
                        .size
                        .wrapping_add(1);
                    *preview_lines.subresults.items.offset(c2rust_fresh11 as isize) = current_match;
                }
                line_breakcheck();
            }
            if did_sub {
                sub_nlines += 1;
            }
            xfree(new_start as *mut ::core::ffi::c_void);
            let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut sub_firstline
                as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            *ptr__0;
            xfree(line_matches.items as *mut ::core::ffi::c_void);
            line_matches.capacity = 0 as size_t;
            line_matches.size = line_matches.capacity;
            line_matches.items = ::core::ptr::null_mut::<LineData>();
        }
        line_breakcheck();
        if profile_passed_limit(timeout) {
            got_quit = true_0 != 0;
        }
        lnum += 1;
    }
    (*curbuf).deleted_bytes2 = 0 as size_t;
    if first_line != 0 as linenr_T {
        i = ((*curbuf).b_ml.ml_line_count - old_line_count) as ::core::ffi::c_int;
        changed_lines(
            curbuf,
            first_line,
            0 as colnr_T,
            last_line - i as linenr_T,
            i as linenr_T,
            false_0 != 0,
        );
        let mut num_added: int64_t = (last_line - first_line) as int64_t;
        let mut num_removed: int64_t = num_added - i as int64_t;
        buf_updates_send_changes(curbuf, first_line, num_added, num_removed);
    }
    xfree(sub_firstline as *mut ::core::ffi::c_void);
    if subflags.do_count {
        (*curwin).w_cursor = old_cursor;
    }
    if sub_nsubs > start_nsubs {
        if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf).b_op_start.lnum = (*eap).line1;
            (*curbuf).b_op_end.lnum = line2;
            (*curbuf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf).b_op_start.col = (*curbuf).b_op_end.col;
        }
        if global_busy == 0 {
            if !subflags.do_ask {
                if endcolumn {
                    coladvance(curwin, MAXCOL as ::core::ffi::c_int);
                } else {
                    beginline(
                        BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                    );
                }
            }
            if cmdpreview_ns <= 0 as ::core::ffi::c_int && !do_sub_msg(subflags.do_count)
                && subflags.do_ask as ::core::ffi::c_int != 0 && p_ch > 0 as OptInt
            {
                msg(
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else {
            global_need_beginline = true_0;
        }
        if subflags.do_print {
            print_line(
                (*curwin).w_cursor.lnum,
                subflags.do_number,
                subflags.do_list,
                true_0 != 0,
            );
        }
    } else if global_busy == 0 {
        if got_int {
            emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        } else if got_match {
            if p_ch > 0 as OptInt && !ui_has(kUIMessages) {
                msg(
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if subflags.do_error {
            semsg(
                gettext(&raw const e_patnotf2 as *const ::core::ffi::c_char),
                get_search_pat(),
            );
        }
    }
    if subflags.do_ask as ::core::ffi::c_int != 0 && hasAnyFolding(curwin) != 0 {
        changed_window_setting(curwin);
    }
    vim_regfree(regmatch.regprog);
    xfree(sub as *mut ::core::ffi::c_void);
    subflags.do_all = save_do_all;
    subflags.do_ask = save_do_ask;
    let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cmdpreview_ns > 0 as ::core::ffi::c_int && !aborting() {
        if got_quit as ::core::ffi::c_int != 0
            || profile_passed_limit(timeout) as ::core::ffi::c_int != 0
        {
            set_option_direct(
                kOptInccommand,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                0 as ::core::ffi::c_int,
                SID_NONE,
            );
        } else if *p_icm as ::core::ffi::c_int != NUL && !pat.is_null() {
            if pre_hl_id == 0 as ::core::ffi::c_int {
                pre_hl_id = syn_check_group(
                    b"Substitute\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 11]>()
                        .wrapping_sub(1 as size_t),
                );
            }
            retv = show_sub(
                eap,
                old_cursor,
                &raw mut preview_lines,
                pre_hl_id,
                cmdpreview_ns,
                cmdpreview_bufnr,
            );
        }
    }
    xfree(preview_lines.subresults.items as *mut ::core::ffi::c_void);
    preview_lines.subresults.capacity = 0 as size_t;
    preview_lines.subresults.size = preview_lines.subresults.capacity;
    preview_lines.subresults.items = ::core::ptr::null_mut::<SubResult>();
    return retv;
}
#[no_mangle]
pub unsafe extern "C" fn do_sub_msg(mut count_only: bool) -> bool {
    if (sub_nsubs as OptInt > p_report
        && (KeyTyped as ::core::ffi::c_int != 0 || sub_nlines > 1 as linenr_T
            || p_report < 1 as OptInt) || count_only as ::core::ffi::c_int != 0)
        && messaging() as ::core::ffi::c_int != 0
    {
        if got_int {
            strcpy(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                gettext(b"(Interrupted) \0".as_ptr() as *const ::core::ffi::c_char),
            );
        } else {
            *(&raw mut msg_buf as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
        }
        let mut msg_single: *mut ::core::ffi::c_char = if count_only
            as ::core::ffi::c_int != 0
        {
            ngettext(
                b"%ld match on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld matches on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs as ::core::ffi::c_ulong,
            )
        } else {
            ngettext(
                b"%ld substitution on %ld line\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld substitutions on %ld line\0".as_ptr()
                    as *const ::core::ffi::c_char,
                sub_nsubs as ::core::ffi::c_ulong,
            )
        };
        let mut msg_plural: *mut ::core::ffi::c_char = if count_only
            as ::core::ffi::c_int != 0
        {
            ngettext(
                b"%ld match on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld matches on %ld lines\0".as_ptr() as *const ::core::ffi::c_char,
                sub_nsubs as ::core::ffi::c_ulong,
            )
        } else {
            ngettext(
                b"%ld substitution on %ld lines\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"%ld substitutions on %ld lines\0".as_ptr()
                    as *const ::core::ffi::c_char,
                sub_nsubs as ::core::ffi::c_ulong,
            )
        };
        vim_snprintf_add(
            &raw mut msg_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 480]>(),
            ngettext(msg_single, msg_plural, sub_nlines as ::core::ffi::c_ulong),
            sub_nsubs as int64_t,
            sub_nlines as int64_t,
        );
        if msg(&raw mut msg_buf as *mut ::core::ffi::c_char, 0 as ::core::ffi::c_int) {
            set_keep_msg(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
        return true_0 != 0;
    }
    if got_int {
        emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn global_exe_one(cmd: *mut ::core::ffi::c_char, lnum: linenr_T) {
    (*curwin).w_cursor.lnum = lnum;
    (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    if *cmd as ::core::ffi::c_int == NUL
        || *cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
    {
        do_cmdline(
            b"p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            None,
            NULL_0,
            DOCMD_NOWAIT as ::core::ffi::c_int,
        );
    } else {
        do_cmdline(cmd, None, NULL_0, DOCMD_NOWAIT as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_global(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = 0;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut delim: ::core::ffi::c_char = 0;
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut patlen: size_t = 0;
    let mut regmatch: regmmatch_T = regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    };
    if global_busy != 0
        && ((*eap).line1 != 1 as linenr_T
            || (*eap).line2 != (*curbuf).b_ml.ml_line_count)
    {
        emsg(
            gettext(
                b"E147: Cannot do :global recursive with a range\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return;
    }
    if (*eap).forceit != 0 {
        type_0 = 'v' as ::core::ffi::c_int;
    } else {
        type_0 = *(*eap).cmd as uint8_t as ::core::ffi::c_int;
    }
    cmd = (*eap).arg;
    let mut which_pat: ::core::ffi::c_int = RE_LAST as ::core::ffi::c_int;
    if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
        cmd = cmd.offset(1);
        if vim_strchr(
                b"/?&\0".as_ptr() as *const ::core::ffi::c_char,
                *cmd as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            emsg(gettext(&raw const e_backslash as *const ::core::ffi::c_char));
            return;
        }
        if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            which_pat = RE_SUBST as ::core::ffi::c_int;
        } else {
            which_pat = RE_SEARCH as ::core::ffi::c_int;
        }
        cmd = cmd.offset(1);
        pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        patlen = 0 as size_t;
    } else if *cmd as ::core::ffi::c_int == NUL {
        emsg(
            gettext(
                b"E148: Regular expression missing from global\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return;
    } else if check_regexp_delim(*cmd as ::core::ffi::c_int) == FAIL {
        return
    } else {
        delim = *cmd;
        cmd = cmd.offset(1);
        pat = cmd;
        cmd = skip_regexp_ex(
            cmd,
            delim as ::core::ffi::c_int,
            magic_isset() as ::core::ffi::c_int,
            &raw mut (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<magic_T>(),
        );
        if *cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == delim as ::core::ffi::c_int
        {
            let c2rust_fresh5 = cmd;
            cmd = cmd.offset(1);
            *c2rust_fresh5 = NUL as ::core::ffi::c_char;
        }
        patlen = strlen(pat);
    }
    let mut used_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if search_regcomp(
        pat,
        patlen,
        &raw mut used_pat,
        RE_BOTH as ::core::ffi::c_int,
        which_pat,
        SEARCH_HIS as ::core::ffi::c_int,
        &raw mut regmatch,
    ) == FAIL
    {
        emsg(gettext(&raw const e_invcmd as *const ::core::ffi::c_char));
        return;
    }
    if global_busy != 0 {
        lnum = (*curwin).w_cursor.lnum;
        let mut match_0: ::core::ffi::c_int = vim_regexec_multi(
            &raw mut regmatch,
            curwin,
            curbuf,
            lnum,
            0 as colnr_T,
            ::core::ptr::null_mut::<proftime_T>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if type_0 == 'g' as ::core::ffi::c_int && match_0 != 0
            || type_0 == 'v' as ::core::ffi::c_int && match_0 == 0
        {
            global_exe_one(cmd, lnum);
        }
    } else {
        let mut ndone: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        lnum = (*eap).line1;
        while lnum <= (*eap).line2 && !got_int {
            let mut match_1: ::core::ffi::c_int = vim_regexec_multi(
                &raw mut regmatch,
                curwin,
                curbuf,
                lnum,
                0 as colnr_T,
                ::core::ptr::null_mut::<proftime_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if regmatch.regprog.is_null() {
                break;
            }
            if type_0 == 'g' as ::core::ffi::c_int && match_1 != 0
                || type_0 == 'v' as ::core::ffi::c_int && match_1 == 0
            {
                ml_setmarked(lnum);
                ndone += 1;
            }
            line_breakcheck();
            lnum += 1;
        }
        if got_int {
            msg(
                gettext(&raw const e_interr as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else if ndone == 0 as ::core::ffi::c_int {
            if type_0 == 'v' as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(
                        b"Pattern found in every line: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    used_pat,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(
                        b"Pattern not found: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    ),
                    used_pat,
                );
            }
        } else {
            global_exe(cmd);
        }
        ml_clearmarked();
    }
    vim_regfree(regmatch.regprog);
}
#[no_mangle]
pub unsafe extern "C" fn global_exe(mut cmd: *mut ::core::ffi::c_char) {
    let mut old_lcount: linenr_T = 0;
    let mut old_buf: *mut buf_T = curbuf;
    let mut lnum: linenr_T = 0;
    setpcmark();
    msg_didout = true_0 != 0;
    sub_nsubs = 0 as ::core::ffi::c_int;
    sub_nlines = 0 as ::core::ffi::c_int as linenr_T;
    global_need_msg_kind = true_0 != 0;
    global_need_beginline = false_0;
    global_busy = 1 as ::core::ffi::c_int;
    old_lcount = (*curbuf).b_ml.ml_line_count;
    while !got_int
        && {
            lnum = ml_firstmarked();
            lnum != 0 as linenr_T
        } && global_busy == 1 as ::core::ffi::c_int
    {
        global_exe_one(cmd, lnum);
        os_breakcheck();
    }
    global_busy = 0 as ::core::ffi::c_int;
    if global_need_beginline != 0 {
        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else {
        check_cursor(curwin);
    }
    changed_line_abv_curs();
    if msg_col == 0 as ::core::ffi::c_int && msg_scrolled == 0 as ::core::ffi::c_int {
        msg_didout = false_0 != 0;
    }
    if !do_sub_msg(false_0 != 0) && curbuf == old_buf {
        msgmore(
            (*curbuf).b_ml.ml_line_count as ::core::ffi::c_int
                - old_lcount as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn prepare_tagpreview(mut undo_sync: bool) -> bool {
    if (*curwin).w_onebuf_opt.wo_pvw != 0 {
        return false_0 != 0;
    }
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_onebuf_opt.wo_pvw != 0 {
            win_enter(wp, undo_sync);
            return false_0 != 0;
        }
        wp = (*wp).w_next;
    }
    if win_split(
        (if g_do_tagpreview > 0 as ::core::ffi::c_int {
            g_do_tagpreview
        } else {
            0 as ::core::ffi::c_int
        }),
        0 as ::core::ffi::c_int,
    ) == FAIL
    {
        return false_0 != 0;
    }
    (*curwin).w_onebuf_opt.wo_pvw = true_0;
    (*curwin).w_onebuf_opt.wo_wfh = true_0;
    (*curwin).w_onebuf_opt.wo_scb = false_0;
    (*curwin).w_onebuf_opt.wo_crb = false_0;
    (*curwin).w_onebuf_opt.wo_diff = false_0;
    set_option_direct(
        kOptFoldcolumn,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"0\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    return true_0 != 0;
}
unsafe extern "C" fn show_sub(
    mut eap: *mut exarg_T,
    mut old_cusr: pos_T,
    mut preview_lines: *mut PreviewLines,
    mut hl_id: ::core::ffi::c_int,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    let mut save_shm_p: *mut ::core::ffi::c_char = xstrdup(p_shm);
    let mut lines: PreviewLines = *preview_lines;
    let mut orig_buf: *mut buf_T = curbuf;
    let mut cmdpreview_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    set_option_direct(
        kOptShortmess,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"F\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    let mut i: size_t = 0 as size_t;
    while i < lines.subresults.size {
        let mut curres: SubResult = *lines.subresults.items.offset(i as isize);
        if curres.start.lnum >= old_cusr.lnum {
            (*curwin).w_cursor.lnum = curres.start.lnum;
            (*curwin).w_cursor.col = curres.start.col;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    update_topline(curwin);
    let mut col_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut preview: bool = *p_icm as ::core::ffi::c_int == 's' as ::core::ffi::c_int
        && ((*eap).line1 != old_cusr.lnum || (*eap).line2 != old_cusr.lnum);
    if preview {
        cmdpreview_buf = buflist_findnr(cmdpreview_bufnr as ::core::ffi::c_int);
        '_c2rust_label: {
            if !cmdpreview_buf.is_null() {} else {
                __assert_fail(
                    b"cmdpreview_buf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/ex_cmds.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    4886 as ::core::ffi::c_uint,
                    b"int show_sub(exarg_T *, pos_T, PreviewLines *, int, int, handle_T)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if lines.subresults.size > 0 as size_t {
            let mut last_match: SubResult = *lines
                .subresults
                .items
                .offset(
                    lines
                        .subresults
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t) as isize,
                );
            let mut highest_lnum: linenr_T = if last_match.start.lnum
                > last_match.end.lnum
            {
                last_match.start.lnum
            } else {
                last_match.end.lnum
            };
            '_c2rust_label_0: {
                if highest_lnum > 0 as linenr_T {} else {
                    __assert_fail(
                        b"highest_lnum > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/ex_cmds.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        4892 as ::core::ffi::c_uint,
                        b"int show_sub(exarg_T *, pos_T, PreviewLines *, int, int, handle_T)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            col_width = log10(highest_lnum as ::core::ffi::c_double)
                as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                + 3 as ::core::ffi::c_int;
        }
    }
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut old_line_size: colnr_T = 0 as colnr_T;
    let mut line_size: colnr_T = 0 as colnr_T;
    let mut linenr_preview: linenr_T = 0 as linenr_T;
    let mut linenr_origbuf: linenr_T = 0 as linenr_T;
    let mut next_linenr: linenr_T = 0 as linenr_T;
    let mut matchidx: size_t = 0 as size_t;
    while matchidx < lines.subresults.size {
        let mut match_0: SubResult = *lines.subresults.items.offset(matchidx as isize);
        if !cmdpreview_buf.is_null() {
            let mut p_start: lpos_T = lpos_T {
                lnum: 0 as linenr_T,
                col: match_0.start.col,
            };
            let mut p_end: lpos_T = lpos_T {
                lnum: 0 as linenr_T,
                col: match_0.end.col,
            };
            buf_ensure_loaded(cmdpreview_buf);
            if match_0.pre_match == 0 as linenr_T {
                next_linenr = match_0.start.lnum;
            } else {
                next_linenr = match_0.pre_match;
            }
            if next_linenr == linenr_origbuf {
                next_linenr += 1;
                p_start.lnum = linenr_preview;
                p_end.lnum = linenr_preview;
            }
            while next_linenr <= match_0.end.lnum {
                if next_linenr == match_0.start.lnum {
                    p_start.lnum = linenr_preview + 1 as linenr_T;
                }
                if next_linenr == match_0.end.lnum {
                    p_end.lnum = linenr_preview + 1 as linenr_T;
                }
                let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                    ::core::ffi::c_char,
                >();
                if next_linenr == (*orig_buf).b_ml.ml_line_count + 1 as linenr_T {
                    line = b"\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                } else {
                    line = ml_get_buf(orig_buf, next_linenr);
                    line_size = (ml_get_buf_len(orig_buf, next_linenr) + col_width
                        + 1 as ::core::ffi::c_int) as colnr_T;
                    if line_size > old_line_size {
                        str = xrealloc(
                            str as *mut ::core::ffi::c_void,
                            (line_size as size_t)
                                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                        ) as *mut ::core::ffi::c_char;
                        old_line_size = line_size;
                    }
                }
                snprintf(
                    str,
                    line_size as size_t,
                    b"|%*d| %s\0".as_ptr() as *const ::core::ffi::c_char,
                    col_width - 3 as ::core::ffi::c_int,
                    next_linenr,
                    line,
                );
                if linenr_preview == 0 as linenr_T {
                    ml_replace_buf(
                        cmdpreview_buf,
                        1 as linenr_T,
                        str,
                        true_0 != 0,
                        false_0 != 0,
                    );
                } else {
                    ml_append_buf(
                        cmdpreview_buf,
                        linenr_preview,
                        str,
                        line_size,
                        false_0 != 0,
                    );
                }
                linenr_preview = (linenr_preview as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as linenr_T;
                next_linenr += 1;
            }
            linenr_origbuf = match_0.end.lnum;
            bufhl_add_hl_pos_offset(
                cmdpreview_buf,
                cmdpreview_ns,
                hl_id,
                p_start,
                p_end,
                col_width as colnr_T,
            );
        }
        bufhl_add_hl_pos_offset(
            orig_buf,
            cmdpreview_ns,
            hl_id,
            match_0.start,
            match_0.end,
            0 as colnr_T,
        );
        matchidx = matchidx.wrapping_add(1);
    }
    xfree(str as *mut ::core::ffi::c_void);
    set_option_direct(
        kOptShortmess,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(save_shm_p),
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    xfree(save_shm_p as *mut ::core::ffi::c_void);
    return if preview as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_substitute(mut eap: *mut exarg_T) {
    do_sub(eap, profile_zero(), 0 as ::core::ffi::c_int, 0 as handle_T);
}
#[no_mangle]
pub unsafe extern "C" fn ex_substitute_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    if *(*eap).arg as ::core::ffi::c_int != 0
        && !(*(*eap).arg as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *(*eap).arg as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *(*eap).arg as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *(*eap).arg as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0)
    {
        let mut save_eap: *mut ::core::ffi::c_char = (*eap).arg;
        let mut retv: ::core::ffi::c_int = do_sub(
            eap,
            profile_setlimit(p_rdt as int64_t),
            cmdpreview_ns,
            cmdpreview_bufnr,
        );
        (*eap).arg = save_eap;
        return retv;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn skip_vimgrep_pat(
    mut p: *mut ::core::ffi::c_char,
    mut s: *mut *mut ::core::ffi::c_char,
    mut flags: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if vim_isIDc(*p as uint8_t as ::core::ffi::c_int) {
        if !s.is_null() {
            *s = p;
        }
        p = skiptowhite(p);
        if !s.is_null() && *p as ::core::ffi::c_int != NUL {
            let c2rust_fresh13 = p;
            p = p.offset(1);
            *c2rust_fresh13 = NUL as ::core::ffi::c_char;
        }
    } else {
        if !s.is_null() {
            *s = p.offset(1 as ::core::ffi::c_int as isize);
        }
        let mut c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
        p = skip_regexp(p.offset(1 as ::core::ffi::c_int as isize), c, true_0);
        if *p as ::core::ffi::c_int != c {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !s.is_null() {
            *p = NUL as ::core::ffi::c_char;
        }
        p = p.offset(1);
        while *p as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'j' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
        {
            if !flags.is_null() {
                if *p as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
                    *flags |= VGR_GLOBAL as ::core::ffi::c_int;
                } else if *p as ::core::ffi::c_int == 'j' as ::core::ffi::c_int {
                    *flags |= VGR_NOJUMP as ::core::ffi::c_int;
                } else {
                    *flags |= VGR_FUZZY as ::core::ffi::c_int;
                }
            }
            p = p.offset(1);
        }
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn ex_oldfiles(mut eap: *mut exarg_T) {
    let mut l: *mut list_T = get_vim_var_list(VV_OLDFILES);
    let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if l.is_null() {
        msg(
            gettext(b"No old files\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    msg_start();
    msg_scroll = true_0;
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if got_int {
                break;
            }
            nr += 1;
            let mut fname: *const ::core::ffi::c_char = tv_get_string(
                &raw mut (*li).li_tv,
            );
            if !message_filtered(fname) {
                msg_outnum(nr);
                msg_puts(b": \0".as_ptr() as *const ::core::ffi::c_char);
                msg_outtrans(
                    tv_get_string(&raw mut (*li).li_tv),
                    0 as ::core::ffi::c_int,
                    false,
                );
                msg_clr_eos();
                msg_putchar('\n' as ::core::ffi::c_int);
                os_breakcheck();
            }
            li = (*li).li_next;
        }
    }
    got_int = false_0 != 0;
    if cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
        quit_more = false_0 != 0;
        nr = prompt_for_input(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        msg_starthere();
        if nr > 0 as ::core::ffi::c_int && nr <= tv_list_len(l) {
            let p: *const ::core::ffi::c_char = tv_list_find_str(
                l,
                nr - 1 as ::core::ffi::c_int,
            );
            if p.is_null() {
                return;
            }
            let s: *mut ::core::ffi::c_char = expand_env_save(
                p as *mut ::core::ffi::c_char,
            );
            (*eap).arg = s;
            (*eap).cmdidx = CMD_edit;
            cmdmod.cmod_flags &= !(CMOD_BROWSE as ::core::ffi::c_int);
            do_exedit(eap, ::core::ptr::null_mut::<win_T>());
            xfree(s as *mut ::core::ffi::c_void);
        }
    }
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int
    + 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn linetabsize_str(
    mut s: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return linetabsize_col(0 as ::core::ffi::c_int, s);
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const DBL_MAX: ::core::ffi::c_double = __DBL_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const __DBL_MAX__: ::core::ffi::c_double = 1.7976931348623157e+308f64;
