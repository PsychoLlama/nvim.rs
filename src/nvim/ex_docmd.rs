use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    pub type Unpacker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn atol(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_long;
    fn abort() -> !;
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
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
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
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_mem_free(mem: ArenaMem);
    fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_clear_error(value: *mut Error);
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name_0: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn remote_ui_disconnect(channel_id: uint64_t, err: *mut Error, send_error_exit: bool);
    fn remote_ui_connect(
        channel_id: uint64_t,
        server_addr: *mut ::core::ffi::c_char,
        err: *mut Error,
    );
    fn nvim__chan_set_detach(channel_id: uint64_t, detach: Boolean, err: *mut Error);
    fn nvim_command(cmd: String_0, err: *mut Error);
    fn check_arg_idx(win: *mut win_T);
    fn ex_args(eap: *mut exarg_T);
    fn ex_previous(eap: *mut exarg_T);
    fn ex_rewind(eap: *mut exarg_T);
    fn ex_last(eap: *mut exarg_T);
    fn ex_argument(eap: *mut exarg_T);
    fn ex_next(eap: *mut exarg_T);
    fn ex_argdedupe(eap: *mut exarg_T);
    fn ex_argedit(eap: *mut exarg_T);
    fn ex_argadd(eap: *mut exarg_T);
    fn ex_argdelete(eap: *mut exarg_T);
    fn ex_all(eap: *mut exarg_T);
    fn arg_all() -> *mut ::core::ffi::c_char;
    fn do_augroup(arg: *mut ::core::ffi::c_char, del_group: bool);
    fn is_aucmd_win(win: *mut win_T) -> bool;
    fn do_autocmd(eap: *mut exarg_T, arg_in: *mut ::core::ffi::c_char, forceit: ::core::ffi::c_int);
    fn do_doautocmd(
        arg_start: *mut ::core::ffi::c_char,
        do_msg: bool,
        did_something: *mut bool,
    ) -> ::core::ffi::c_int;
    fn ex_doautoall(eap: *mut exarg_T);
    fn check_nomodeline(argp: *mut *mut ::core::ffi::c_char) -> bool;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn getnextac(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn may_trigger_vim_suspend_resume(suspend: bool);
    static autocmd_fname: GlobalCell<*mut ::core::ffi::c_char>;
    static autocmd_fname_full: GlobalCell<bool>;
    static autocmd_bufnr: GlobalCell<::core::ffi::c_int>;
    static autocmd_match: GlobalCell<*mut ::core::ffi::c_char>;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    fn get_highest_fnum() -> ::core::ffi::c_int;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn goto_buffer(
        eap: *mut exarg_T,
        start: ::core::ffi::c_int,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
    );
    fn do_bufdel(
        command: ::core::ffi::c_int,
        arg: *mut ::core::ffi::c_char,
        addr_count: ::core::ffi::c_int,
        start_bnr: ::core::ffi::c_int,
        end_bnr: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn no_write_message();
    fn buflist_findpat(
        pattern: *const ::core::ffi::c_char,
        pattern_end: *const ::core::ffi::c_char,
        unlisted: bool,
        diffmode: bool,
        curtab_only: bool,
    ) -> ::core::ffi::c_int;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buflist_list(eap: *mut exarg_T);
    fn setfname(
        buf: *mut buf_T,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        message: bool,
    ) -> ::core::ffi::c_int;
    fn setaltfname(
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut buf_T;
    fn otherfile(ffname: *mut ::core::ffi::c_char) -> bool;
    fn maketitle();
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn ex_buffer_all(eap: *mut exarg_T);
    fn do_modelines(flags: ::core::ffi::c_int);
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn buf_hide(buf: *const buf_T) -> bool;
    fn buf_spname(buf: *mut buf_T) -> *mut ::core::ffi::c_char;
    fn deleted_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    static channels: GlobalCell<Map_uint64_t_ptr_t>;
    fn channel_close(
        id: uint64_t,
        part: ChannelPart,
        error: *mut *const ::core::ffi::c_char,
    ) -> bool;
    fn channel_job_start(
        argv: *mut *mut ::core::ffi::c_char,
        exepath: *const ::core::ffi::c_char,
        on_stdout: CallbackReader,
        on_stderr: CallbackReader,
        on_exit: Callback,
        pty: bool,
        rpc: bool,
        overlapped: bool,
        detach: bool,
        stdin_mode: ChannelStdinMode,
        cwd: *const ::core::ffi::c_char,
        pty_width: uint16_t,
        pty_height: uint16_t,
        env: *mut dict_T,
        status_out: *mut varnumber_T,
    ) -> *mut Channel;
    static p_awa: GlobalCell<::core::ffi::c_int>;
    static p_confirm: GlobalCell<::core::ffi::c_int>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_gp: GlobalCell<*mut ::core::ffi::c_char>;
    static p_ei: GlobalCell<*mut ::core::ffi::c_char>;
    static p_ffu: GlobalCell<*mut ::core::ffi::c_char>;
    static p_hls: GlobalCell<::core::ffi::c_int>;
    static p_lz: GlobalCell<::core::ffi::c_int>;
    static p_mp: GlobalCell<*mut ::core::ffi::c_char>;
    static p_mfd: GlobalCell<OptInt>;
    static p_mmd: GlobalCell<OptInt>;
    static p_pvh: GlobalCell<OptInt>;
    static p_rtp: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sh: GlobalCell<*mut ::core::ffi::c_char>;
    static p_shada: GlobalCell<*mut ::core::ffi::c_char>;
    static p_verbose: GlobalCell<OptInt>;
    static p_wic: GlobalCell<::core::ffi::c_int>;
    static p_write: GlobalCell<::core::ffi::c_int>;
    static p_cdh: GlobalCell<::core::ffi::c_int>;
    fn vim_strsave_escaped(
        string: *const ::core::ffi::c_char,
        esc_chars: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn del_trailing_spaces(ptr: *mut ::core::ffi::c_char);
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strrep(
        src: *const ::core::ffi::c_char,
        what: *const ::core::ffi::c_char,
        rep: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite_esc(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: intmax_t) -> intmax_t;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getdigits_int32(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: int32_t) -> int32_t;
    fn backslash_halve(p: *mut ::core::ffi::c_char);
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    fn ExpandGeneric(
        pat: *const ::core::ffi::c_char,
        xp: *mut expand_T,
        regmatch: *mut regmatch_T,
        matches: *mut *mut *mut ::core::ffi::c_char,
        numMatches: *mut ::core::ffi::c_int,
        func: CompleteListItemGetter,
        escaped: bool,
    );
    fn check_cursor_col(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn do_debug(cmd: *mut ::core::ffi::c_char);
    fn ex_debug(eap: *mut exarg_T);
    fn dbg_check_breakpoint(eap: *mut exarg_T);
    fn ex_breakadd(eap: *mut exarg_T);
    fn ex_debuggreedy(eap: *mut exarg_T);
    fn ex_breakdel(eap: *mut exarg_T);
    fn ex_breaklist(eap: *mut exarg_T);
    fn dbg_find_breakpoint(
        file: bool,
        fname: *mut ::core::ffi::c_char,
        after: linenr_T,
    ) -> linenr_T;
    fn dbg_breakpoint(name: *mut ::core::ffi::c_char, lnum: linenr_T);
    fn putdigraph(str: *mut ::core::ffi::c_char);
    fn listdigraphs(use_headers: bool);
    fn ex_loadkeymap(eap: *mut exarg_T);
    fn screen_resize(width: ::core::ffi::c_int, height: ::core::ffi::c_int);
    fn update_screen() -> ::core::ffi::c_int;
    fn setcursor_mayforce(wp: *mut win_T, force: bool);
    fn showmode() -> ::core::ffi::c_int;
    fn clearmode();
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn status_redraw_all();
    fn status_redraw_curbuf();
    fn redraw_statuslines();
    static e_argreq: [::core::ffi::c_char; 0];
    static e_backslash: [::core::ffi::c_char; 0];
    static e_cmdwin: [::core::ffi::c_char; 0];
    static e_curdir: [::core::ffi::c_char; 0];
    static e_command_too_recursive: [::core::ffi::c_char; 0];
    static e_endif: [::core::ffi::c_char; 0];
    static e_endtry: [::core::ffi::c_char; 0];
    static e_endwhile: [::core::ffi::c_char; 0];
    static e_endfor: [::core::ffi::c_char; 0];
    static e_failed: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargval: [::core::ffi::c_char; 0];
    static e_invrange: [::core::ffi::c_char; 0];
    static e_invcmd: [::core::ffi::c_char; 0];
    static e_isadir2: [::core::ffi::c_char; 0];
    static e_invchan: [::core::ffi::c_char; 0];
    static e_mkdir: [::core::ffi::c_char; 0];
    static e_modifiable: [::core::ffi::c_char; 0];
    static e_nobang: [::core::ffi::c_char; 0];
    static e_norange: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    static e_no_errors: [::core::ffi::c_char; 0];
    static e_sandbox: [::core::ffi::c_char; 0];
    static e_secure: [::core::ffi::c_char; 0];
    static e_screenmode: [::core::ffi::c_char; 0];
    static e_shellempty: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_zerocount: [::core::ffi::c_char; 0];
    static e_usingsid: [::core::ffi::c_char; 0];
    static e_empty_buffer: [::core::ffi::c_char; 0];
    static e_autocmd_close: [::core::ffi::c_char; 0];
    static e_cant_find_file_str_in_path: [::core::ffi::c_char; 0];
    static e_no_more_file_str_found_in_path: [::core::ffi::c_char; 0];
    static e_line_number_out_of_range: [::core::ffi::c_char; 0];
    static e_undobang_cannot_redo_or_move_branch: [::core::ffi::c_char; 0];
    static e_invalid_return_type_from_findfunc: [::core::ffi::c_char; 0];
    fn modify_fname(
        src: *mut ::core::ffi::c_char,
        tilde_file: bool,
        usedlen: *mut size_t,
        fnamep: *mut *mut ::core::ffi::c_char,
        bufp: *mut *mut ::core::ffi::c_char,
        fnamelen: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn beginline(flags: ::core::ffi::c_int);
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg_multiline(
        s: *const ::core::ffi::c_char,
        kind: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        multiline: bool,
    ) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn semsg_multiline(
        kind: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn ex_messages(eap: *mut exarg_T);
    fn wait_return(redraw: ::core::ffi::c_int);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_make(arg: *const ::core::ffi::c_char);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_scroll_flush();
    fn msg_clr_eos();
    fn redirecting() -> ::core::ffi::c_int;
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn vim_dialog_yesno(
        type_0: ::core::ffi::c_int,
        title: *mut ::core::ffi::c_char,
        message: *mut ::core::ffi::c_char,
        dflt: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_list_free(l: *mut list_T);
    fn tv_list_copy(
        conv: *const vimconv_T,
        orig: *mut list_T,
        deep: bool,
        copyID: ::core::ffi::c_int,
    ) -> *mut list_T;
    fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T;
    fn tv_list_find_str(l: *mut list_T, n: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn callback_free(callback: *mut Callback);
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn get_scriptlocal_funcname(funcname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ex_function(eap: *mut exarg_T);
    fn ex_delfunction(eap: *mut exarg_T);
    fn ex_return(eap: *mut exarg_T);
    fn ex_call(eap: *mut exarg_T);
    fn do_return(
        eap: *mut exarg_T,
        reanimate: bool,
        is_cmd: bool,
        rettv: *mut ::core::ffi::c_void,
    ) -> bool;
    fn get_func_line(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn func_has_ended(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn func_has_abort(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn func_name(cookie: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_char;
    fn func_breakpoint(cookie: *mut ::core::ffi::c_void) -> *mut linenr_T;
    fn func_dbg_tick(cookie: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_int;
    fn func_level(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn current_func_returned() -> ::core::ffi::c_int;
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    fn ex_let(eap: *mut exarg_T);
    fn ex_unlet(eap: *mut exarg_T);
    fn ex_lockvar(eap: *mut exarg_T);
    fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn v_exception(oldval: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn v_throwpoint(oldval: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn var_redir_start(name: *mut ::core::ffi::c_char, append: bool) -> ::core::ffi::c_int;
    fn var_redir_stop();
    fn os_hrtime() -> uint64_t;
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn proc_wait(
        proc: *mut Proc,
        ms: ::core::ffi::c_int,
        events: *mut MultiQueue,
    ) -> ::core::ffi::c_int;
    fn proc_stop(proc: *mut Proc);
    fn do_ascii(eap: *mut exarg_T);
    fn ex_align(eap: *mut exarg_T);
    fn ex_sort(eap: *mut exarg_T);
    fn ex_uniq(eap: *mut exarg_T);
    fn do_move(line1: linenr_T, line2: linenr_T, dest: linenr_T) -> ::core::ffi::c_int;
    fn ex_copy(line1: linenr_T, line2: linenr_T, n: linenr_T);
    fn do_bang(
        addr_count: ::core::ffi::c_int,
        eap: *mut exarg_T,
        forceit: bool,
        do_in: bool,
        do_out: bool,
    );
    fn print_line_no_prefix(lnum: linenr_T, use_number: bool, list: bool);
    fn print_line(lnum: linenr_T, use_number: bool, list: bool, first: bool);
    fn ex_file(eap: *mut exarg_T);
    fn ex_update(eap: *mut exarg_T);
    fn ex_write(eap: *mut exarg_T);
    fn do_write(eap: *mut exarg_T) -> ::core::ffi::c_int;
    fn ex_wnext(eap: *mut exarg_T);
    fn do_wqall(eap: *mut exarg_T);
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn ex_append(eap: *mut exarg_T);
    fn ex_change(eap: *mut exarg_T);
    fn ex_z(eap: *mut exarg_T);
    fn ex_global(eap: *mut exarg_T);
    fn global_exe(cmd: *mut ::core::ffi::c_char);
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn ex_substitute(eap: *mut exarg_T);
    fn ex_substitute_preview(
        eap: *mut exarg_T,
        cmdpreview_ns: ::core::ffi::c_int,
        cmdpreview_bufnr: handle_T,
    ) -> ::core::ffi::c_int;
    fn skip_vimgrep_pat(
        p: *mut ::core::ffi::c_char,
        s: *mut *mut ::core::ffi::c_char,
        flags: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ex_oldfiles(eap: *mut exarg_T);
    fn ex_ruby(eap: *mut exarg_T);
    fn ex_rubyfile(eap: *mut exarg_T);
    fn ex_rubydo(eap: *mut exarg_T);
    fn ex_python3(eap: *mut exarg_T);
    fn ex_py3file(eap: *mut exarg_T);
    fn ex_pydo3(eap: *mut exarg_T);
    fn ex_perl(eap: *mut exarg_T);
    fn ex_perlfile(eap: *mut exarg_T);
    fn ex_perldo(eap: *mut exarg_T);
    fn autowrite_all();
    fn check_changed(buf: *mut buf_T, flags: ::core::ffi::c_int) -> bool;
    fn dialog_changed(buf: *mut buf_T, checkall: bool);
    fn check_changed_any(hidden: bool, unload: bool) -> bool;
    fn check_fname() -> ::core::ffi::c_int;
    fn ex_listdo(eap: *mut exarg_T);
    fn ex_compiler(eap: *mut exarg_T);
    fn ex_checktime(eap: *mut exarg_T);
    fn ex_drop(eap: *mut exarg_T);
    fn aborting() -> bool;
    fn do_errthrow(cstack: *mut cstack_T, cmdname: *mut ::core::ffi::c_char);
    fn do_intthrow(cstack: *mut cstack_T) -> bool;
    fn discard_current_exception();
    fn report_make_pending(pending: ::core::ffi::c_int, value: *mut ::core::ffi::c_void);
    fn ex_eval(eap: *mut exarg_T);
    fn ex_if(eap: *mut exarg_T);
    fn ex_endif(eap: *mut exarg_T);
    fn ex_else(eap: *mut exarg_T);
    fn ex_while(eap: *mut exarg_T);
    fn ex_continue(eap: *mut exarg_T);
    fn ex_break(eap: *mut exarg_T);
    fn ex_endwhile(eap: *mut exarg_T);
    fn ex_throw(eap: *mut exarg_T);
    fn do_throw(cstack: *mut cstack_T);
    fn ex_try(eap: *mut exarg_T);
    fn ex_catch(eap: *mut exarg_T);
    fn ex_finally(eap: *mut exarg_T);
    fn ex_endtry(eap: *mut exarg_T);
    fn enter_cleanup(csp: *mut cleanup_T);
    fn leave_cleanup(csp: *mut cleanup_T);
    fn cleanup_conditionals(
        cstack: *mut cstack_T,
        searched_cond: ::core::ffi::c_int,
        inclusive: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn rewind_conditionals(
        cstack: *mut cstack_T,
        idx: ::core::ffi::c_int,
        cond_type: ::core::ffi::c_int,
        cond_level: *mut ::core::ffi::c_int,
    );
    fn ex_endfunction(eap: *mut exarg_T);
    fn has_loop_cmd(p: *mut ::core::ffi::c_char) -> bool;
    fn cmdpreview_get_bufnr() -> handle_T;
    fn cmdpreview_get_ns() -> ::core::ffi::c_int;
    fn getcmdline(
        firstc: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn text_locked() -> bool;
    fn text_locked_msg();
    fn get_text_locked_msg() -> *const ::core::ffi::c_char;
    fn text_or_buf_locked() -> bool;
    fn curbuf_locked() -> bool;
    fn allbuf_locked() -> bool;
    fn getexline(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn ui_ext_cmdline_block_append(indent: size_t, line: *const ::core::ffi::c_char);
    fn ui_ext_cmdline_block_leave();
    fn script_get(eap: *mut exarg_T, lenp: *mut size_t) -> *mut ::core::ffi::c_char;
    fn vim_findfile_cleanup(ctx: *mut ::core::ffi::c_void);
    fn find_file_in_path(
        ptr: *mut ::core::ffi::c_char,
        len: size_t,
        options: ::core::ffi::c_int,
        first: ::core::ffi::c_int,
        rel_fname: *mut ::core::ffi::c_char,
        file_to_find: *mut *mut ::core::ffi::c_char,
        search_ctx: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn file_name_at_cursor(
        options: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        file_lnum: *mut linenr_T,
    ) -> *mut ::core::ffi::c_char;
    fn do_autocmd_dirchanged(
        new_dir: *mut ::core::ffi::c_char,
        scope: CdScope,
        cause: CdCause,
        pre: bool,
    );
    fn vim_chdir(new_dir: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn shorten_fnames(force: ::core::ffi::c_int);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn opFoldRange(
        firstpos: pos_T,
        lastpos: pos_T,
        opening: ::core::ffi::c_int,
        recurse: ::core::ffi::c_int,
        had_visual: bool,
    );
    fn foldManualAllowed(create: bool) -> ::core::ffi::c_int;
    fn foldCreate(wp: *mut win_T, start: pos_T, end: pos_T);
    fn stuff_empty() -> bool;
    fn beep_flush();
    fn stuffReadbuff(s: *const ::core::ffi::c_char);
    fn ins_typebuf(
        str: *mut ::core::ffi::c_char,
        noremap: ::core::ffi::c_int,
        offset: ::core::ffi::c_int,
        nottyped: bool,
        silent: bool,
    ) -> ::core::ffi::c_int;
    fn typebuf_typed() -> ::core::ffi::c_int;
    fn save_typeahead(tp: *mut tasave_T);
    fn restore_typeahead(tp: *mut tasave_T);
    fn vpeekc() -> ::core::ffi::c_int;
    static Rows: GlobalCell<::core::ffi::c_int>;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static redraw_cmdline: GlobalCell<bool>;
    static exec_from_reg: GlobalCell<bool>;
    static msg_col: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scroll: GlobalCell<::core::ffi::c_int>;
    static msg_didout: GlobalCell<bool>;
    static msg_didany: GlobalCell<bool>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static emsg_skip: GlobalCell<::core::ffi::c_int>;
    static did_endif: GlobalCell<bool>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static did_emsg_syntax: GlobalCell<bool>;
    static no_wait_return: GlobalCell<::core::ffi::c_int>;
    static need_wait_return: GlobalCell<bool>;
    static need_maketitle: GlobalCell<bool>;
    static lines_left: GlobalCell<::core::ffi::c_int>;
    static ex_nesting_level: GlobalCell<::core::ffi::c_int>;
    static debug_break_level: GlobalCell<::core::ffi::c_int>;
    static debug_tick: GlobalCell<::core::ffi::c_int>;
    static do_profiling: GlobalCell<::core::ffi::c_int>;
    static current_exception: GlobalCell<*mut except_T>;
    static did_throw: GlobalCell<bool>;
    static need_rethrow: GlobalCell<bool>;
    static check_cstack: GlobalCell<bool>;
    static trylevel: GlobalCell<::core::ffi::c_int>;
    static force_abort: GlobalCell<bool>;
    static msg_list: GlobalCell<*mut *mut msglist_T>;
    static suppress_errthrow: GlobalCell<bool>;
    static caught_stack: GlobalCell<*mut except_T>;
    static current_sctx: GlobalCell<sctx_T>;
    static current_ui: GlobalCell<uint64_t>;
    static firstwin: GlobalCell<*mut win_T>;
    static lastwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static topframe: GlobalCell<*mut frame_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static lastused_tabpage: GlobalCell<*mut tabpage_T>;
    static firstbuf: GlobalCell<*mut buf_T>;
    static lastbuf: GlobalCell<*mut buf_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static arg_had_last: GlobalCell<bool>;
    static exiting: GlobalCell<bool>;
    static secure: GlobalCell<::core::ffi::c_int>;
    static textlock: GlobalCell<::core::ffi::c_int>;
    static sandbox: GlobalCell<::core::ffi::c_int>;
    static VIsual_active: GlobalCell<bool>;
    static did_syncbind: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    static finish_op: GlobalCell<bool>;
    static opcount: GlobalCell<::core::ffi::c_int>;
    static exmode_active: GlobalCell<bool>;
    static pending_exmode_active: GlobalCell<bool>;
    static ex_no_reprint: GlobalCell<bool>;
    static cmdpreview: GlobalCell<bool>;
    static reg_executing: GlobalCell<::core::ffi::c_int>;
    static pending_end_reg_executing: GlobalCell<bool>;
    static force_restart_edit: GlobalCell<bool>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static cmdmod: GlobalCell<cmdmod_T>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static emsg_silent: GlobalCell<::core::ffi::c_int>;
    static IObuff: GlobalCell<[::core::ffi::c_char; 1025]>;
    static NameBuff: GlobalCell<[::core::ffi::c_char; 4096]>;
    static RedrawingDisabled: GlobalCell<::core::ffi::c_int>;
    static readonlymode: GlobalCell<bool>;
    static recoverymode: GlobalCell<bool>;
    static typebuf: GlobalCell<typebuf_T>;
    static ex_normal_busy: GlobalCell<::core::ffi::c_int>;
    static expr_map_lock: GlobalCell<::core::ffi::c_int>;
    static stop_insert_mode: GlobalCell<bool>;
    static KeyTyped: GlobalCell<bool>;
    static must_redraw: GlobalCell<::core::ffi::c_int>;
    static got_int: GlobalCell<bool>;
    static searchcmdlen: GlobalCell<::core::ffi::c_int>;
    static global_busy: GlobalCell<::core::ffi::c_int>;
    static last_cmdline: GlobalCell<*mut ::core::ffi::c_char>;
    static repeat_cmdline: GlobalCell<*mut ::core::ffi::c_char>;
    static new_last_cmdline: GlobalCell<*mut ::core::ffi::c_char>;
    static postponed_split: GlobalCell<::core::ffi::c_int>;
    static postponed_split_flags: GlobalCell<::core::ffi::c_int>;
    static postponed_split_tab: GlobalCell<::core::ffi::c_int>;
    static g_do_tagpreview: GlobalCell<::core::ffi::c_int>;
    static escape_chars: GlobalCell<*mut ::core::ffi::c_char>;
    static redir_off: GlobalCell<bool>;
    static redir_fd: GlobalCell<*mut FILE>;
    static redir_reg: GlobalCell<::core::ffi::c_int>;
    static redir_vname: GlobalCell<bool>;
    static globaldir: GlobalCell<*mut ::core::ffi::c_char>;
    static last_chdir_reason: GlobalCell<*mut ::core::ffi::c_char>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static cmdwin_result: GlobalCell<::core::ffi::c_int>;
    static no_hlsearch: GlobalCell<bool>;
    static virtual_op: GlobalCell<TriState>;
    static magic_overruled: GlobalCell<optmagic_T>;
    fn ask_yesno(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn find_ucmd(
        eap: *mut exarg_T,
        p: *mut ::core::ffi::c_char,
        full: *mut ::core::ffi::c_int,
        xp: *mut expand_T,
        complp: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn expand_user_command_name(idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn get_user_command_name(
        idx: ::core::ffi::c_int,
        cmdidx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ex_command(eap: *mut exarg_T);
    fn ex_comclear(eap: *mut exarg_T);
    fn ex_delcommand(eap: *mut exarg_T);
    fn add_win_cmd_modifiers(
        buf: *mut ::core::ffi::c_char,
        cmod: *const cmdmod_T,
        multi_mods: *mut bool,
    ) -> size_t;
    fn do_ucmd(eap: *mut exarg_T, preview: bool) -> ::core::ffi::c_int;
    static main_loop: SharedCell<Loop>;
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn ex_lua(eap: *mut exarg_T);
    fn ex_luado(eap: *mut exarg_T);
    fn ex_luafile(eap: *mut exarg_T);
    fn getout(exitval: ::core::ffi::c_int) -> !;
    fn setmark(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn setpcmark();
    fn checkpcmark();
    fn mark_get(
        buf: *mut buf_T,
        win: *mut win_T,
        fmp: *mut fmark_T,
        flag: MarkGet,
        name: ::core::ffi::c_int,
    ) -> *mut fmark_T;
    fn mark_get_visual(buf: *mut buf_T, name: ::core::ffi::c_int) -> *mut fmark_T;
    fn mark_move_to(fm: *mut fmark_T, flags: MarkMove) -> MarkMoveRes;
    fn mark_check(fm: *mut fmark_T, errormsg: *mut *const ::core::ffi::c_char) -> bool;
    fn ex_marks(eap: *mut exarg_T);
    fn ex_delmarks(eap: *mut exarg_T);
    fn ex_jumps(eap: *mut exarg_T);
    fn ex_clearjumps(eap: *mut exarg_T);
    fn ex_changes(eap: *mut exarg_T);
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_copy_char(fp: *mut *const ::core::ffi::c_char, tp: *mut *mut ::core::ffi::c_char);
    fn get_encoding_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    static utf8len_tab: [uint8_t; 256];
    fn ml_recover(checkext: bool);
    fn ml_preserve(buf: *mut buf_T, message: bool, do_fsync: bool);
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_delete(lnum: linenr_T) -> ::core::ffi::c_int;
    fn ml_setmarked(lnum: linenr_T);
    fn ml_clearmarked();
    fn goto_byte(cnt: ::core::ffi::c_int);
    fn setmouse();
    fn update_topline(wp: *mut win_T);
    fn update_curswant();
    fn check_cursor_moved(wp: *mut win_T);
    fn cursor_valid(wp: *mut win_T) -> ::core::ffi::c_int;
    fn validate_cursor(wp: *mut win_T);
    fn scrolldown(wp: *mut win_T, line_count: linenr_T, byfold: ::core::ffi::c_int) -> bool;
    fn scrollup(wp: *mut win_T, line_count: linenr_T, byfold: bool) -> bool;
    fn cursor_correct(wp: *mut win_T);
    fn server_start(addr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn server_stop(endpoint: *const ::core::ffi::c_char, keep_vservername: bool) -> bool;
    fn rpc_send_call(
        id: uint64_t,
        method_name: *const ::core::ffi::c_char,
        args: Array,
        result_mem: *mut ArenaMem,
        err: *mut Error,
    ) -> Object;
    fn normal_enter(cmdwin: bool, noexmode: bool);
    fn end_visual_mode();
    fn find_ident_under_cursor(
        text: *mut *mut ::core::ffi::c_char,
        find_type: ::core::ffi::c_int,
        offset: *mut ::core::ffi::c_int,
    ) -> size_t;
    fn get_vtopline(wp: *mut win_T) -> ::core::ffi::c_int;
    fn do_check_scrollbind(check: bool);
    fn set_cursor_for_append_to_line();
    fn normal_cmd(oap: *mut oparg_T, toplevel: bool);
    fn ex_set(eap: *mut exarg_T);
    fn get_option_sctx(opt_idx: OptIndex) -> *mut sctx_T;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn get_findfunc() -> *mut ::core::ffi::c_char;
    fn magic_isset() -> bool;
    fn option_set_callback_func(
        optval: *mut ::core::ffi::c_char,
        optcb: *mut Callback,
    ) -> ::core::ffi::c_int;
    fn get_scrolloff_value(wp: *mut win_T) -> int64_t;
    fn free_string_option(p: *mut ::core::ffi::c_char);
    fn get_fileformat_name(xp: *mut expand_T, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn check_ff_value(p: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_mkdir(path: *const ::core::ffi::c_char, mode: int32_t) -> ::core::ffi::c_int;
    fn os_breakcheck();
    fn line_breakcheck();
    fn os_getenv_noalloc(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn expand_env_esc(
        srcp: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
        esc: bool,
        one: bool,
        prefix: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn shell_build_argv(
        cmd: *const ::core::ffi::c_char,
        extra_args: *const ::core::ffi::c_char,
    ) -> *mut *mut ::core::ffi::c_char;
    fn shell_free_argv(argv: *mut *mut ::core::ffi::c_char);
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn path_has_wildcard(p: *const ::core::ffi::c_char) -> bool;
    fn pathcmp(
        p: *const ::core::ffi::c_char,
        q: *const ::core::ffi::c_char,
        maxlen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn path_try_shorten_fname(full_path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn plines_m_win_fill(wp: *mut win_T, first: linenr_T, last: linenr_T) -> ::core::ffi::c_int;
    fn ex_profile(eap: *mut exarg_T);
    fn func_line_start(cookie: *mut ::core::ffi::c_void);
    fn func_line_exec(cookie: *mut ::core::ffi::c_void);
    fn func_line_end(cookie: *mut ::core::ffi::c_void);
    fn script_line_start();
    fn script_line_exec();
    fn script_line_end();
    fn pum_make_popup(path_name: *const ::core::ffi::c_char, use_mouse_pos: ::core::ffi::c_int);
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
    fn qf_list(eap: *mut exarg_T);
    fn qf_age(eap: *mut exarg_T);
    fn qf_history(eap: *mut exarg_T);
    fn ex_cwindow(eap: *mut exarg_T);
    fn ex_cclose(eap: *mut exarg_T);
    fn ex_copen(eap: *mut exarg_T);
    fn ex_cbottom(eap: *mut exarg_T);
    fn grep_internal(cmdidx: cmdidx_T) -> ::core::ffi::c_int;
    fn ex_make(eap: *mut exarg_T);
    fn qf_get_size(eap: *mut exarg_T) -> size_t;
    fn qf_get_valid_size(eap: *mut exarg_T) -> size_t;
    fn qf_get_cur_idx(eap: *mut exarg_T) -> size_t;
    fn qf_get_cur_valid_idx(eap: *mut exarg_T) -> ::core::ffi::c_int;
    fn ex_cc(eap: *mut exarg_T);
    fn ex_cnext(eap: *mut exarg_T);
    fn ex_cbelow(eap: *mut exarg_T);
    fn ex_cfile(eap: *mut exarg_T);
    fn ex_vimgrep(eap: *mut exarg_T);
    fn ex_cbuffer(eap: *mut exarg_T);
    fn ex_cexpr(eap: *mut exarg_T);
    fn ex_helpgrep(eap: *mut exarg_T);
    static exestack: GlobalCell<garray_T>;
    fn save_last_search_pattern();
    fn restore_last_search_pattern();
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
    fn find_pattern_in_path(
        ptr: *mut ::core::ffi::c_char,
        dir: Direction,
        len: size_t,
        whole: bool,
        skip_comments: bool,
        type_0: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        action: ::core::ffi::c_int,
        start_lnum: linenr_T,
        end_lnum: linenr_T,
        forceit: bool,
        silent: bool,
    );
    fn shada_write_file(file: *const ::core::ffi::c_char, nomerge: bool) -> ::core::ffi::c_int;
    fn shada_read_everything(
        fname: *const ::core::ffi::c_char,
        forceit: bool,
        missing_ok: bool,
    ) -> ::core::ffi::c_int;
    fn estack_push(
        type_0: etype_T,
        name: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut estack_T;
    fn estack_pop();
    fn estack_sfile(which: estack_arg_T) -> *mut ::core::ffi::c_char;
    fn ex_runtime(eap: *mut exarg_T);
    fn source_runtime(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ex_packloadall(eap: *mut exarg_T);
    fn ex_packadd(eap: *mut exarg_T);
    fn ex_source(eap: *mut exarg_T);
    fn ex_options(eap: *mut exarg_T);
    fn source_breakpoint(cookie: *mut ::core::ffi::c_void) -> *mut linenr_T;
    fn source_dbg_tick(cookie: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_int;
    fn source_level(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn ex_scriptnames(eap: *mut exarg_T);
    fn getsourceline(
        c: ::core::ffi::c_int,
        cookie: *mut ::core::ffi::c_void,
        indent: ::core::ffi::c_int,
        do_concat: bool,
    ) -> *mut ::core::ffi::c_char;
    fn ex_scriptencoding(eap: *mut exarg_T);
    fn ex_finish(eap: *mut exarg_T);
    fn do_finish(eap: *mut exarg_T, reanimate: bool);
    fn source_finished(fgetline: LineGetter, cookie: *mut ::core::ffi::c_void) -> bool;
    fn may_trigger_modechanged();
    fn draw_tabline();
    fn ui_active() -> size_t;
    fn ui_busy_start();
    fn ui_busy_stop();
    fn ui_flush();
    fn ui_cursor_shape();
    fn ui_has(ext: UIExtension) -> bool;
    fn ui_call_restart(listen_addr: String_0);
    fn ui_call_suspend();
    fn ui_call_error_exit(status: Integer);
    fn do_tag(
        tag: *mut ::core::ffi::c_char,
        type_0: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
        verbose: bool,
    );
    fn do_tags(eap: *mut exarg_T);
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn u_savedel(lnum: linenr_T, nlines: linenr_T) -> ::core::ffi::c_int;
    fn u_compute_hash(buf: *mut buf_T, hash: *mut uint8_t);
    fn u_write_undo(
        name: *const ::core::ffi::c_char,
        forceit: bool,
        buf: *mut buf_T,
        hash: *mut uint8_t,
    );
    fn u_read_undo(
        name: *mut ::core::ffi::c_char,
        hash: *const uint8_t,
        orig_name: *const ::core::ffi::c_char,
    );
    fn u_undo(count: ::core::ffi::c_int);
    fn u_redo(count: ::core::ffi::c_int);
    fn u_undo_and_forget(count: ::core::ffi::c_int, do_buf_event: bool) -> bool;
    fn undo_time(step: ::core::ffi::c_int, sec: bool, file: bool, absolute: bool);
    fn ex_undolist(eap: *mut exarg_T);
    fn ex_undojoin(eap: *mut exarg_T);
    fn u_clearline(buf: *mut buf_T);
    fn bufIsChanged(buf: *mut buf_T) -> bool;
    fn curbufIsChanged() -> bool;
    fn window_layout_locked(cmd: cmdidx_T) -> bool;
    fn check_can_set_curbuf_forceit(forceit: ::core::ffi::c_int) -> bool;
    fn do_window(nchar: ::core::ffi::c_int, Prenum: ::core::ffi::c_int, xchar: ::core::ffi::c_int);
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn trigger_tabclosedpre(tp: *mut tabpage_T);
    fn win_close_othertab(
        win: *mut win_T,
        free_buf: ::core::ffi::c_int,
        tp: *mut tabpage_T,
        force: bool,
    ) -> bool;
    fn close_others(message: ::core::ffi::c_int, forceit: ::core::ffi::c_int);
    fn win_new_tabpage(
        after: ::core::ffi::c_int,
        filename: *mut ::core::ffi::c_char,
        enter: bool,
        first: *mut *mut win_T,
    ) -> *mut tabpage_T;
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn find_tabpage(n: ::core::ffi::c_int) -> *mut tabpage_T;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn goto_tabpage(n: ::core::ffi::c_int);
    fn tabpage_move(nr: ::core::ffi::c_int);
    fn win_goto(wp: *mut win_T);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_setheight_win(height: ::core::ffi::c_int, win: *mut win_T);
    fn win_setwidth_win(width: ::core::ffi::c_int, wp: *mut win_T);
    fn only_one_window() -> bool;
    fn win_float_remove(bang: bool, count: ::core::ffi::c_int);
    fn ex_history(eap: *mut exarg_T);
    fn ex_diffupdate(eap: *mut exarg_T);
    fn ex_diffpatch(eap: *mut exarg_T);
    fn ex_diffsplit(eap: *mut exarg_T);
    fn ex_diffthis(eap: *mut exarg_T);
    fn ex_diffoff(eap: *mut exarg_T);
    fn ex_diffgetput(eap: *mut exarg_T);
    fn skip_expr(pp: *mut *mut ::core::ffi::c_char, evalarg: *mut evalarg_T) -> ::core::ffi::c_int;
    fn eval_to_string(
        arg: *mut ::core::ffi::c_char,
        join_list: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    fn get_copyID() -> ::core::ffi::c_int;
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn set_ref_in_callback(
        callback: *mut Callback,
        copyID: ::core::ffi::c_int,
        ht_stack: *mut *mut ht_stack_T,
        list_stack: *mut *mut list_stack_T,
    ) -> bool;
    fn ex_echo(eap: *mut exarg_T);
    fn ex_echohl(eap: *mut exarg_T);
    fn ex_execute(eap: *mut exarg_T);
    fn ex_loadview(eap: *mut exarg_T);
    fn ex_mkrc(eap: *mut exarg_T);
    fn ex_help(eap: *mut exarg_T);
    fn ex_helpclose(eap: *mut exarg_T);
    fn ex_exusage(eap: *mut exarg_T);
    fn ex_viusage(eap: *mut exarg_T);
    fn ex_helptags(eap: *mut exarg_T);
    fn ex_retab(eap: *mut exarg_T);
    fn ex_trust(eap: *mut exarg_T);
    fn ex_abbreviate(eap: *mut exarg_T);
    fn ex_map(eap: *mut exarg_T);
    fn ex_unmap(eap: *mut exarg_T);
    fn ex_mapclear(eap: *mut exarg_T);
    fn ex_abclear(eap: *mut exarg_T);
    fn ex_match(eap: *mut exarg_T);
    fn ex_menu(eap: *mut exarg_T);
    fn ex_emenu(eap: *mut exarg_T);
    fn ex_menutranslate(eap: *mut exarg_T);
    fn op_shift(oap: *mut oparg_T, curs_top: bool, amount: ::core::ffi::c_int);
    fn op_delete(oap: *mut oparg_T) -> ::core::ffi::c_int;
    fn do_join(
        count: size_t,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark_0: bool,
    ) -> ::core::ffi::c_int;
    fn clear_oparg(oap: *mut oparg_T);
    fn set_expr_line(new_line: *mut ::core::ffi::c_char);
    fn valid_yank_reg(regname: ::core::ffi::c_int, writing: bool) -> bool;
    fn do_execreg(
        regname: ::core::ffi::c_int,
        colon: ::core::ffi::c_int,
        addcr: ::core::ffi::c_int,
        silent: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn op_yank(oap: *mut oparg_T, message: bool) -> bool;
    fn do_put(
        regname: ::core::ffi::c_int,
        reg: *mut yankreg_T,
        dir: ::core::ffi::c_int,
        count: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    );
    fn ex_display(eap: *mut exarg_T);
    fn write_reg_contents(
        name: ::core::ffi::c_int,
        str: *const ::core::ffi::c_char,
        len: ssize_t,
        must_append: ::core::ffi::c_int,
    );
    fn ex_language(eap: *mut exarg_T);
    fn ex_sign(eap: *mut exarg_T);
    fn ex_spellrepall(eap: *mut exarg_T);
    fn ex_spellinfo(eap: *mut exarg_T);
    fn ex_spelldump(eap: *mut exarg_T);
    fn ex_mkspell(eap: *mut exarg_T);
    fn ex_spell(eap: *mut exarg_T);
    fn ex_syntax(eap: *mut exarg_T);
    fn ex_ownsyntax(eap: *mut exarg_T);
    fn ex_syntime(eap: *mut exarg_T);
    fn ex_version(eap: *mut exarg_T);
    fn ex_intro(eap: *mut exarg_T);
    fn load_colors(name: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_highlight(line: *const ::core::ffi::c_char, forceit: bool, init: bool);
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
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
pub type intmax_t = ::libc::intmax_t;
pub type uid_t = __uid_t;
pub type time_t = __time_t;
pub type size_t = usize;
pub type ssize_t = isize;
pub type gid_t = __gid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
pub type ptrdiff_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_5,
    pub internal_fields: *mut ::core::ffi::c_void,
    pub stop_flag: ::core::ffi::c_uint,
    pub flags: ::core::ffi::c_ulong,
    pub backend_fd: ::core::ffi::c_int,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub watchers: *mut *mut uv__io_t,
    pub nwatchers: ::core::ffi::c_uint,
    pub nfds: ::core::ffi::c_uint,
    pub wq: uv__queue,
    pub wq_mutex: uv_mutex_t,
    pub wq_async: uv_async_t,
    pub cloexec_lock: uv_rwlock_t,
    pub closing_handles: *mut uv_handle_t,
    pub process_handles: uv__queue,
    pub prepare_handles: uv__queue,
    pub check_handles: uv__queue,
    pub idle_handles: uv__queue,
    pub async_handles: uv__queue,
    pub async_unused: Option<unsafe extern "C" fn() -> ()>,
    pub async_io_watcher: uv__io_t,
    pub async_wfd: ::core::ffi::c_int,
    pub timer_heap: C2Rust_Unnamed_3,
    pub timer_counter: uint64_t,
    pub time: uint64_t,
    pub signal_pipefd: [::core::ffi::c_int; 2],
    pub signal_io_watcher: uv__io_t,
    pub child_watcher: uv_signal_t,
    pub emfile_fd: ::core::ffi::c_int,
    pub inotify_read_watcher: uv__io_t,
    pub inotify_watchers: *mut ::core::ffi::c_void,
    pub inotify_fd: ::core::ffi::c_int,
}
pub type uv__io_t = uv__io_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__io_s {
    pub cb: uv__io_cb,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub pevents: ::core::ffi::c_uint,
    pub events: ::core::ffi::c_uint,
    pub fd: ::core::ffi::c_int,
}
pub type uv__io_cb =
    Option<unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> ()>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_2,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_0,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_1,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_close_cb = Option<unsafe extern "C" fn(*mut uv_handle_t) -> ()>;
pub type uv_handle_type = ::core::ffi::c_uint;
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub type uv_loop_t = uv_loop_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_4,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_mutex_t = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type uv_file = ::core::ffi::c_int;
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
pub type uv_req_type = ::core::ffi::c_uint;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_6,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
pub type uv_connection_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> ()>;
pub type uv_stream_t = uv_stream_s;
pub type uv_shutdown_t = uv_shutdown_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_shutdown_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub handle: *mut uv_stream_t,
    pub cb: uv_shutdown_cb,
}
pub type uv_shutdown_cb =
    Option<unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> ()>;
pub type uv_connect_t = uv_connect_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_connect_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_connect_cb,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
}
pub type uv_connect_cb = Option<unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()>;
pub type uv_read_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()>;
pub type uv_alloc_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_7,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_8,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_10,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_9,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_11,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_12,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub exit_cb: uv_exit_cb,
    pub pid: ::core::ffi::c_int,
    pub queue: uv__queue,
    pub status: ::core::ffi::c_int,
}
pub type uv_exit_cb =
    Option<unsafe extern "C" fn(*mut uv_process_t, int64_t, ::core::ffi::c_int) -> ()>;
pub type uv_process_t = uv_process_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_stdio_flags = ::core::ffi::c_uint;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stdio_container_s {
    pub flags: uv_stdio_flags,
    pub data: C2Rust_Unnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
    pub stream: *mut uv_stream_t,
    pub fd: ::core::ffi::c_int,
}
pub type uv_stdio_container_t = uv_stdio_container_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_options_s {
    pub exit_cb: uv_exit_cb,
    pub file: *const ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub env: *mut *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_uint,
    pub stdio_count: ::core::ffi::c_int,
    pub stdio: *mut uv_stdio_container_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
}
pub type uv_process_options_t = uv_process_options_s;
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
    pub data: C2Rust_Unnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
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
    pub b_wininfo: C2Rust_Unnamed_26,
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
    pub b_signcols: C2Rust_Unnamed_18,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_16,
    pub update_callbacks: C2Rust_Unnamed_15,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
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
pub struct C2Rust_Unnamed_16 {
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
    pub data: C2Rust_Unnamed_17,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
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
pub struct C2Rust_Unnamed_18 {
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
    pub sst_union: C2Rust_Unnamed_19,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
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
    pub data: C2Rust_Unnamed_20,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
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
    pub fc_fixvar: [C2Rust_Unnamed_21; 12],
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
pub struct C2Rust_Unnamed_21 {
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
    pub uh_next: C2Rust_Unnamed_25,
    pub uh_prev: C2Rust_Unnamed_24,
    pub uh_alt_next: C2Rust_Unnamed_23,
    pub uh_alt_prev: C2Rust_Unnamed_22,
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
pub union C2Rust_Unnamed_22 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
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
pub struct C2Rust_Unnamed_26 {
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
    pub type_0: C2Rust_Unnamed_27,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_27 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_27 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_27 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_27 = 0;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_28,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
pub type Proc = proc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut dict_T,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: *mut MultiQueue,
}
pub type MultiQueue = multiqueue;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_29,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub events: *mut MultiQueue,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_29 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_30 = 2147483647;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_31 = 2147483647;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ht_stack_S {
    pub ht: *mut hashtab_T,
    pub prev: *mut ht_stack_S,
}
pub type ht_stack_T = ht_stack_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct list_stack_S {
    pub list: *mut list_T,
    pub prev: *mut list_stack_S,
}
pub type list_stack_T = list_stack_S;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_32 = 76;
pub const HLF_PRE: C2Rust_Unnamed_32 = 75;
pub const HLF_OK: C2Rust_Unnamed_32 = 74;
pub const HLF_SO: C2Rust_Unnamed_32 = 73;
pub const HLF_SE: C2Rust_Unnamed_32 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_32 = 71;
pub const HLF_TS: C2Rust_Unnamed_32 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_32 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_32 = 68;
pub const HLF_CU: C2Rust_Unnamed_32 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_32 = 66;
pub const HLF_WBR: C2Rust_Unnamed_32 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_32 = 64;
pub const HLF_MSG: C2Rust_Unnamed_32 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_32 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_32 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_32 = 60;
pub const HLF_0: C2Rust_Unnamed_32 = 59;
pub const HLF_QFL: C2Rust_Unnamed_32 = 58;
pub const HLF_MC: C2Rust_Unnamed_32 = 57;
pub const HLF_CUL: C2Rust_Unnamed_32 = 56;
pub const HLF_CUC: C2Rust_Unnamed_32 = 55;
pub const HLF_TPF: C2Rust_Unnamed_32 = 54;
pub const HLF_TPS: C2Rust_Unnamed_32 = 53;
pub const HLF_TP: C2Rust_Unnamed_32 = 52;
pub const HLF_PBR: C2Rust_Unnamed_32 = 51;
pub const HLF_PST: C2Rust_Unnamed_32 = 50;
pub const HLF_PSB: C2Rust_Unnamed_32 = 49;
pub const HLF_PSX: C2Rust_Unnamed_32 = 48;
pub const HLF_PNX: C2Rust_Unnamed_32 = 47;
pub const HLF_PSK: C2Rust_Unnamed_32 = 46;
pub const HLF_PNK: C2Rust_Unnamed_32 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_32 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_32 = 43;
pub const HLF_PSI: C2Rust_Unnamed_32 = 42;
pub const HLF_PNI: C2Rust_Unnamed_32 = 41;
pub const HLF_SPL: C2Rust_Unnamed_32 = 40;
pub const HLF_SPR: C2Rust_Unnamed_32 = 39;
pub const HLF_SPC: C2Rust_Unnamed_32 = 38;
pub const HLF_SPB: C2Rust_Unnamed_32 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_32 = 36;
pub const HLF_SC: C2Rust_Unnamed_32 = 35;
pub const HLF_TXA: C2Rust_Unnamed_32 = 34;
pub const HLF_TXD: C2Rust_Unnamed_32 = 33;
pub const HLF_DED: C2Rust_Unnamed_32 = 32;
pub const HLF_CHD: C2Rust_Unnamed_32 = 31;
pub const HLF_ADD: C2Rust_Unnamed_32 = 30;
pub const HLF_FC: C2Rust_Unnamed_32 = 29;
pub const HLF_FL: C2Rust_Unnamed_32 = 28;
pub const HLF_WM: C2Rust_Unnamed_32 = 27;
pub const HLF_W: C2Rust_Unnamed_32 = 26;
pub const HLF_VNC: C2Rust_Unnamed_32 = 25;
pub const HLF_V: C2Rust_Unnamed_32 = 24;
pub const HLF_T: C2Rust_Unnamed_32 = 23;
pub const HLF_VSP: C2Rust_Unnamed_32 = 22;
pub const HLF_C: C2Rust_Unnamed_32 = 21;
pub const HLF_SNC: C2Rust_Unnamed_32 = 20;
pub const HLF_S: C2Rust_Unnamed_32 = 19;
pub const HLF_R: C2Rust_Unnamed_32 = 18;
pub const HLF_CLF: C2Rust_Unnamed_32 = 17;
pub const HLF_CLS: C2Rust_Unnamed_32 = 16;
pub const HLF_CLN: C2Rust_Unnamed_32 = 15;
pub const HLF_LNB: C2Rust_Unnamed_32 = 14;
pub const HLF_LNA: C2Rust_Unnamed_32 = 13;
pub const HLF_N: C2Rust_Unnamed_32 = 12;
pub const HLF_CM: C2Rust_Unnamed_32 = 11;
pub const HLF_M: C2Rust_Unnamed_32 = 10;
pub const HLF_LC: C2Rust_Unnamed_32 = 9;
pub const HLF_L: C2Rust_Unnamed_32 = 8;
pub const HLF_I: C2Rust_Unnamed_32 = 7;
pub const HLF_E: C2Rust_Unnamed_32 = 6;
pub const HLF_D: C2Rust_Unnamed_32 = 5;
pub const HLF_AT: C2Rust_Unnamed_32 = 4;
pub const HLF_TERM: C2Rust_Unnamed_32 = 3;
pub const HLF_EOB: C2Rust_Unnamed_32 = 2;
pub const HLF_8: C2Rust_Unnamed_32 = 1;
pub const HLF_NONE: C2Rust_Unnamed_32 = 0;
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type CdScope = ::core::ffi::c_int;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub type CdCause = ::core::ffi::c_int;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub const kCdCauseOther: CdCause = -1;
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
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_33 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_33 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_33 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_33 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_33 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_33 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_33 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_33 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_33 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_33 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_33 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_33 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_33 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_33 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_33 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_33 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_33 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_33 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_33 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_33 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_33 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_33 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_33 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_33 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_33 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_33 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_33 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_33 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_33 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_33 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_33 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_33 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_33 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_33 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_33 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_33 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_33 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_33 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_33 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_33 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_33 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_33 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_33 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_33 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_33 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_33 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_33 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_33 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_33 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_33 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_33 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_33 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_33 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_33 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_33 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_33 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_33 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_33 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_33 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_33 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_33 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_33 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_33 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_33 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_33 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_33 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_33 = -2;
pub type CompleteListItemGetter =
    Option<unsafe extern "C" fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char>;
pub type optmagic_T = ::core::ffi::c_uint;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
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
pub type MarkMoveRes = ::core::ffi::c_uint;
pub const kMarkChangedView: MarkMoveRes = 64;
pub const kMarkChangedCursor: MarkMoveRes = 32;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkChangedCol: MarkMoveRes = 8;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub type MarkMove = ::core::ffi::c_uint;
pub const kMarkJumpList: MarkMove = 16;
pub const kMarkSetView: MarkMove = 8;
pub const KMarkNoContext: MarkMove = 4;
pub const kMarkContext: MarkMove = 2;
pub const kMarkBeginLine: MarkMove = 1;
pub type MarkGet = ::core::ffi::c_uint;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
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
    pub cs_pend: C2Rust_Unnamed_34,
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
pub union C2Rust_Unnamed_34 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const CSF_SILENT: C2Rust_Unnamed_35 = 16384;
pub const CSF_FINISHED: C2Rust_Unnamed_35 = 8192;
pub const CSF_CAUGHT: C2Rust_Unnamed_35 = 4096;
pub const CSF_THROWN: C2Rust_Unnamed_35 = 2048;
pub const CSF_FINALLY: C2Rust_Unnamed_35 = 512;
pub const CSF_TRY: C2Rust_Unnamed_35 = 256;
pub const CSF_FOR: C2Rust_Unnamed_35 = 16;
pub const CSF_WHILE: C2Rust_Unnamed_35 = 8;
pub const CSF_ELSE: C2Rust_Unnamed_35 = 4;
pub const CSF_ACTIVE: C2Rust_Unnamed_35 = 2;
pub const CSF_TRUE: C2Rust_Unnamed_35 = 1;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const CSTP_FINISH: C2Rust_Unnamed_36 = 32;
pub const CSTP_RETURN: C2Rust_Unnamed_36 = 24;
pub const CSTP_CONTINUE: C2Rust_Unnamed_36 = 16;
pub const CSTP_BREAK: C2Rust_Unnamed_36 = 8;
pub const CSTP_THROW: C2Rust_Unnamed_36 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_36 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_36 = 1;
pub const CSTP_NONE: C2Rust_Unnamed_36 = 0;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const CSL_HAD_FINA: C2Rust_Unnamed_37 = 8;
pub const CSL_HAD_CONT: C2Rust_Unnamed_37 = 4;
pub const CSL_HAD_ENDLOOP: C2Rust_Unnamed_37 = 2;
pub const CSL_HAD_LOOP: C2Rust_Unnamed_37 = 1;
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
pub struct cleanup_stuff {
    pub pending: ::core::ffi::c_int,
    pub exception: *mut except_T,
}
pub type cleanup_T = cleanup_stuff;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct packer_buffer_t {
    pub startptr: *mut ::core::ffi::c_char,
    pub ptr: *mut ::core::ffi::c_char,
    pub endptr: *mut ::core::ffi::c_char,
    pub anydata: *mut ::core::ffi::c_void,
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer) -> ()>;
pub type PackerBuffer = packer_buffer_t;
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
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_background: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
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
pub type ex_func_T = Option<unsafe extern "C" fn(*mut exarg_T) -> ()>;
pub type ex_preview_func_T =
    Option<unsafe extern "C" fn(*mut exarg_T, ::core::ffi::c_int, handle_T) -> ::core::ffi::c_int>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandDefinition {
    pub cmd_name: *mut ::core::ffi::c_char,
    pub cmd_func: ex_func_T,
    pub cmd_preview_func: ex_preview_func_T,
    pub cmd_argt: uint32_t,
    pub cmd_addr_type: cmd_addr_T,
}
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_38 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_38 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_38 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_38 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_38 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_38 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_38 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_38 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_38 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_38 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_38 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_38 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_38 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_38 = 1;
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
    pub magic: C2Rust_Unnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_39 {
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
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub type AutoPatCmd = AutoPatCmd_S;
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
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub type ChannelPart = ::core::ffi::c_uint;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub type ChannelStdinMode = ::core::ffi::c_uint;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallbackReader {
    pub cb: Callback,
    pub self_0: *mut dict_T,
    pub buffer: garray_T,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    pub type_0: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_41,
    pub is_rpc: bool,
    pub detach: bool,
    pub rpc: RpcState,
    pub term: *mut Terminal,
    pub on_data: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub exit_status: ::core::ffi::c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_40,
    pub info: Dict,
    pub client_type: ClientType,
}
pub type ClientType = ::core::ffi::c_int;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_40 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ChannelCallFrame,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
    pub result_mem: ArenaMem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_41 {
    pub proc: Proc,
    pub uv: LibuvProc,
    pub pty: PtyProc,
    pub socket: RStream,
    pub stdio: StdioPair,
    pub err: StderrState,
    pub internal: InternalState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PtyProc {
    pub proc: Proc,
    pub width: uint16_t,
    pub height: uint16_t,
    pub winsize: winsize,
    pub tty_fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
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
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_42 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_42 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_42 = 16;
pub const PUT_LINE: C2Rust_Unnamed_42 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_42 = 4;
pub const PUT_CURSEND: C2Rust_Unnamed_42 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_42 = 1;
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
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_43 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_43 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_43 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_43 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_43 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_43 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_43 = 7;
pub const WILD_ALL: C2Rust_Unnamed_43 = 6;
pub const WILD_PREV: C2Rust_Unnamed_43 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_43 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_43 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_43 = 2;
pub const WILD_FREE: C2Rust_Unnamed_43 = 1;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_44 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_44 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_44 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_44 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_44 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_44 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_44 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_44 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_44 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_44 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_44 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_44 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_44 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_44 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_44 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_44 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_45 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_45 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_45 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_45 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_45 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_45 = 20;
pub const UPD_VALID: C2Rust_Unnamed_45 = 10;
pub type C2Rust_Unnamed_46 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_46 = 4;
pub const BL_SOL: C2Rust_Unnamed_46 = 2;
pub const BL_WHITE: C2Rust_Unnamed_46 = 1;
pub type iconv_t = *mut ::core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
pub type C2Rust_Unnamed_47 = ::core::ffi::c_uint;
pub const VIM_LAST_TYPE: C2Rust_Unnamed_47 = 4;
pub const VIM_QUESTION: C2Rust_Unnamed_47 = 4;
pub const VIM_INFO: C2Rust_Unnamed_47 = 3;
pub const VIM_WARNING: C2Rust_Unnamed_47 = 2;
pub const VIM_ERROR: C2Rust_Unnamed_47 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_47 = 0;
pub type C2Rust_Unnamed_48 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_48 = 6;
pub const VIM_ALL: C2Rust_Unnamed_48 = 5;
pub const VIM_CANCEL: C2Rust_Unnamed_48 = 4;
pub const VIM_NO: C2Rust_Unnamed_48 = 3;
pub const VIM_YES: C2Rust_Unnamed_48 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
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
pub type C2Rust_Unnamed_49 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_49 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_49 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_49 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_49 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_49 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_49 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_49 = 1;
pub type C2Rust_Unnamed_50 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_50 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_50 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_50 = 0;
pub type C2Rust_Unnamed_51 = ::core::ffi::c_uint;
pub const CCGD_EXCMD: C2Rust_Unnamed_51 = 16;
pub const CCGD_ALLBUF: C2Rust_Unnamed_51 = 8;
pub const CCGD_FORCEIT: C2Rust_Unnamed_51 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_51 = 2;
pub const CCGD_AW: C2Rust_Unnamed_51 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffblock {
    pub b_next: *mut buffblock,
    pub b_strlen: size_t,
    pub b_str: [::core::ffi::c_char; 1],
}
pub type buffblock_T = buffblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffheader_T {
    pub bh_first: buffblock_T,
    pub bh_curr: *mut buffblock_T,
    pub bh_index: size_t,
    pub bh_space: size_t,
    pub bh_create_newblock: bool,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tasave_T {
    pub save_typebuf: typebuf_T,
    pub typebuf_valid: bool,
    pub old_char: ::core::ffi::c_int,
    pub old_mod_mask: ::core::ffi::c_int,
    pub save_readbuf1: buffheader_T,
    pub save_readbuf2: buffheader_T,
    pub save_inputbuf: String_0,
}
pub type RemapValues = ::core::ffi::c_int;
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_52 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_52 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_52 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_52 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_52 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_52 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_52 = 1;
pub type C2Rust_Unnamed_53 = ::core::ffi::c_uint;
pub const VALID_HEAD: C2Rust_Unnamed_53 = 2;
pub const VALID_PATH: C2Rust_Unnamed_53 = 1;
pub type C2Rust_Unnamed_54 = ::core::ffi::c_uint;
pub const DIALOG_MSG_SIZE: C2Rust_Unnamed_54 = 1000;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_state_T {
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_restart_edit: ::core::ffi::c_int,
    pub save_msg_didout: bool,
    pub save_State: ::core::ffi::c_int,
    pub save_finish_op: bool,
    pub save_opcount: ::core::ffi::c_int,
    pub save_reg_executing: ::core::ffi::c_int,
    pub save_pending_end_reg_executing: bool,
    pub tabuf: tasave_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dbg_stuff {
    pub trylevel: ::core::ffi::c_int,
    pub force_abort: ::core::ffi::c_int,
    pub caught_stack: *mut except_T,
    pub vv_exception: *mut ::core::ffi::c_char,
    pub vv_throwpoint: *mut ::core::ffi::c_char,
    pub did_emsg: ::core::ffi::c_int,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub check_cstack: ::core::ffi::c_int,
    pub current_exception: *mut except_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_cookie {
    pub lines_gap: *mut garray_T,
    pub current_line: ::core::ffi::c_int,
    pub repeating: ::core::ffi::c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wcmd_T {
    pub line: *mut ::core::ffi::c_char,
    pub lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_55,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_55 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type etype_T = ::core::ffi::c_uint;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub const OP_LSHIFT: C2Rust_Unnamed_67 = 4;
pub const OP_RSHIFT: C2Rust_Unnamed_67 = 5;
pub const OP_YANK: C2Rust_Unnamed_67 = 2;
pub const OP_DELETE: C2Rust_Unnamed_67 = 1;
pub const WSP_VERT: C2Rust_Unnamed_66 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_56 = 1;
pub const DT_LTAG: C2Rust_Unnamed_65 = 11;
pub const DT_TAG: C2Rust_Unnamed_65 = 1;
pub const DT_LAST: C2Rust_Unnamed_65 = 6;
pub const DT_FIRST: C2Rust_Unnamed_65 = 5;
pub const DT_POP: C2Rust_Unnamed_65 = 2;
pub const DT_NEXT: C2Rust_Unnamed_65 = 3;
pub const DT_PREV: C2Rust_Unnamed_65 = 4;
pub const DT_SELECT: C2Rust_Unnamed_65 = 7;
pub const DT_JUMP: C2Rust_Unnamed_65 = 9;
pub const KE_IGNORE: key_extra = 53;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchit_arg_T {
    pub sa_stop_lnum: linenr_T,
    pub sa_tm: *mut proftime_T,
    pub sa_timed_out: ::core::ffi::c_int,
    pub sa_wrapped: ::core::ffi::c_int,
}
pub const SEARCH_MSG: C2Rust_Unnamed_63 = 12;
pub const RE_SEARCH: C2Rust_Unnamed_64 = 0;
pub const RE_SUBST: C2Rust_Unnamed_64 = 1;
pub const SEARCH_HIS: C2Rust_Unnamed_63 = 32;
pub const SEARCH_KEEP: C2Rust_Unnamed_63 = 1024;
pub const MODE_INSERT: C2Rust_Unnamed_57 = 16;
pub const OPT_LOCAL: C2Rust_Unnamed_59 = 2;
pub const MODE_CMDLINE: C2Rust_Unnamed_57 = 8;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const FIND_ANY: C2Rust_Unnamed_61 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_61 = 2;
pub const ACTION_SPLIT: C2Rust_Unnamed_62 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_62 = 2;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_62 = 4;
pub const ACTION_SHOW: C2Rust_Unnamed_62 = 1;
pub const MODE_TERMINAL: C2Rust_Unnamed_57 = 128;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const DIP_ALL: C2Rust_Unnamed_60 = 1;
pub const CHECK_PATH: C2Rust_Unnamed_61 = 3;
pub const SPEC_SID: C2Rust_Unnamed_68 = 14;
pub const SPEC_SFLNUM: C2Rust_Unnamed_68 = 13;
pub const SPEC_SLNUM: C2Rust_Unnamed_68 = 7;
pub type estack_arg_T = ::core::ffi::c_uint;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
pub const SPEC_SCRIPT: C2Rust_Unnamed_68 = 9;
pub const SPEC_STACK: C2Rust_Unnamed_68 = 8;
pub const SPEC_SFILE: C2Rust_Unnamed_68 = 6;
pub const SPEC_AMATCH: C2Rust_Unnamed_68 = 12;
pub const SPEC_ABUF: C2Rust_Unnamed_68 = 11;
pub const SPEC_AFILE: C2Rust_Unnamed_68 = 10;
pub const FNAME_HYP: C2Rust_Unnamed_56 = 4;
pub const SPEC_CFILE: C2Rust_Unnamed_68 = 5;
pub const SPEC_HASH: C2Rust_Unnamed_68 = 1;
pub const SPEC_PERC: C2Rust_Unnamed_68 = 0;
pub const FIND_STRING: C2Rust_Unnamed_58 = 2;
pub const FIND_EVAL: C2Rust_Unnamed_58 = 4;
pub const FIND_IDENT: C2Rust_Unnamed_58 = 1;
pub const SPEC_CEXPR: C2Rust_Unnamed_68 = 4;
pub const SPEC_CWORD: C2Rust_Unnamed_68 = 2;
pub const SPEC_CCWORD: C2Rust_Unnamed_68 = 3;
pub const WSP_TOP: C2Rust_Unnamed_66 = 8;
pub const WSP_BELOW: C2Rust_Unnamed_66 = 64;
pub const WSP_ABOVE: C2Rust_Unnamed_66 = 128;
pub const WSP_HOR: C2Rust_Unnamed_66 = 4;
pub const WSP_BOT: C2Rust_Unnamed_66 = 16;
pub const MODE_NORMAL: C2Rust_Unnamed_57 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod {
    pub name: *mut ::core::ffi::c_char,
    pub minlen: ::core::ffi::c_int,
    pub has_count: ::core::ffi::c_int,
}
pub const OPT_GLOBAL: C2Rust_Unnamed_59 = 1;
pub type C2Rust_Unnamed_56 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_56 = 32;
pub const FNAME_REL: C2Rust_Unnamed_56 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_56 = 8;
pub const FNAME_EXP: C2Rust_Unnamed_56 = 2;
pub type C2Rust_Unnamed_57 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_57 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_57 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_57 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_57 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_57 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_57 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_57 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_57 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_57 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_57 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_57 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_57 = 255;
pub const MODE_SELECT: C2Rust_Unnamed_57 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_57 = 32;
pub const MODE_OP_PENDING: C2Rust_Unnamed_57 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_57 = 2;
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
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
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
pub type C2Rust_Unnamed_58 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_59 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_59 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_59 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_59 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_59 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_59 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_59 = 4;
pub type C2Rust_Unnamed_60 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_60 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_60 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_60 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_60 = 32;
pub const DIP_OPT: C2Rust_Unnamed_60 = 16;
pub const DIP_START: C2Rust_Unnamed_60 = 8;
pub const DIP_ERR: C2Rust_Unnamed_60 = 4;
pub const DIP_DIR: C2Rust_Unnamed_60 = 2;
pub type C2Rust_Unnamed_61 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_62 = ::core::ffi::c_uint;
pub const ACTION_EXPAND: C2Rust_Unnamed_62 = 5;
pub type C2Rust_Unnamed_63 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_63 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_63 = 2048;
pub const SEARCH_MARK: C2Rust_Unnamed_63 = 512;
pub const SEARCH_START: C2Rust_Unnamed_63 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_63 = 128;
pub const SEARCH_END: C2Rust_Unnamed_63 = 64;
pub const SEARCH_OPT: C2Rust_Unnamed_63 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_63 = 8;
pub const SEARCH_ECHO: C2Rust_Unnamed_63 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_63 = 1;
pub type C2Rust_Unnamed_64 = ::core::ffi::c_uint;
pub const RE_LAST: C2Rust_Unnamed_64 = 2;
pub const RE_BOTH: C2Rust_Unnamed_64 = 2;
pub type C2Rust_Unnamed_65 = ::core::ffi::c_uint;
pub const DT_FREE: C2Rust_Unnamed_65 = 99;
pub const DT_HELP: C2Rust_Unnamed_65 = 8;
pub type C2Rust_Unnamed_66 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_66 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_66 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_66 = 256;
pub const WSP_HELP: C2Rust_Unnamed_66 = 32;
pub const WSP_ROOM: C2Rust_Unnamed_66 = 1;
pub type C2Rust_Unnamed_67 = ::core::ffi::c_uint;
pub const OP_NR_SUB: C2Rust_Unnamed_67 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_67 = 28;
pub const OP_FUNCTION: C2Rust_Unnamed_67 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_67 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_67 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_67 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_67 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_67 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_67 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_67 = 20;
pub const OP_FOLD: C2Rust_Unnamed_67 = 19;
pub const OP_APPEND: C2Rust_Unnamed_67 = 18;
pub const OP_INSERT: C2Rust_Unnamed_67 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_67 = 16;
pub const OP_ROT13: C2Rust_Unnamed_67 = 15;
pub const OP_JOIN_NS: C2Rust_Unnamed_67 = 14;
pub const OP_JOIN: C2Rust_Unnamed_67 = 13;
pub const OP_LOWER: C2Rust_Unnamed_67 = 12;
pub const OP_UPPER: C2Rust_Unnamed_67 = 11;
pub const OP_COLON: C2Rust_Unnamed_67 = 10;
pub const OP_FORMAT: C2Rust_Unnamed_67 = 9;
pub const OP_INDENT: C2Rust_Unnamed_67 = 8;
pub const OP_TILDE: C2Rust_Unnamed_67 = 7;
pub const OP_FILTER: C2Rust_Unnamed_67 = 6;
pub const OP_CHANGE: C2Rust_Unnamed_67 = 3;
pub const OP_NOP: C2Rust_Unnamed_67 = 0;
pub type C2Rust_Unnamed_68 = ::core::ffi::c_uint;
pub const INT32_MAX: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_1: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EXIT_FAILURE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const BF_DUMMY: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_1,
};
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_WHOLEFOLD: ::core::ffi::c_uint = 0x40 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_NOTRLCOM: ::core::ffi::c_uint = 0x800 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_CTRLV: ::core::ffi::c_uint = 0x2000 as ::core::ffi::c_uint;
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_BUFNAME: ::core::ffi::c_uint = 0x8000 as ::core::ffi::c_uint;
pub const EX_BUFUNL: ::core::ffi::c_uint = 0x10000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const EX_SBOXOK: ::core::ffi::c_uint = 0x40000 as ::core::ffi::c_uint;
pub const EX_CMDWIN: ::core::ffi::c_uint = 0x80000 as ::core::ffi::c_uint;
pub const EX_MODIFY: ::core::ffi::c_uint = 0x100000 as ::core::ffi::c_uint;
pub const EX_FLAGS: ::core::ffi::c_uint = 0x200000 as ::core::ffi::c_uint;
pub const EX_LOCK_OK: ::core::ffi::c_uint = 0x1000000 as ::core::ffi::c_uint;
pub const BAD_KEEP: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const BAD_DROP: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FORCE_NOBIN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EXFLAG_LIST: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EXFLAG_NR: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EXFLAG_PRINT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = 13;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_D: ::core::ffi::c_int = 4;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8;
pub const Ctrl_I: ::core::ffi::c_int = 9;
pub const Ctrl_J: ::core::ffi::c_int = 10;
pub const Ctrl_K: ::core::ffi::c_int = 11;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_N: ::core::ffi::c_int = 14;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16;
pub const Ctrl_Q: ::core::ffi::c_int = 17;
pub const Ctrl_R: ::core::ffi::c_int = 18;
pub const Ctrl_S: ::core::ffi::c_int = 19;
pub const Ctrl_T: ::core::ffi::c_int = 20;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23;
pub const Ctrl_X: ::core::ffi::c_int = 24;
pub const Ctrl_Z: ::core::ffi::c_int = 26;
pub const Ctrl_RSB: ::core::ffi::c_int = 29;
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
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
pub const CPO_ALTREAD: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const CPO_BAR: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const CPO_EXECBUF: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const CPO_NOSYMLINKS: ::core::ffi::c_int = '~' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
static e_ambiguous_use_of_user_defined_command: GlobalCell<[::core::ffi::c_char; 44]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"E464: Ambiguous use of user-defined command\0",
        )
    });
static e_no_call_stack_to_substitute_for_stack: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E489: No call stack to substitute for \"<stack>\"\0",
        )
    });
static e_not_an_editor_command: GlobalCell<[::core::ffi::c_char; 28]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E492: Not an editor command\0")
});
static e_no_autocommand_file_name_to_substitute_for_afile: GlobalCell<[::core::ffi::c_char; 59]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 59], [::core::ffi::c_char; 59]>(
            *b"E495: No autocommand file name to substitute for \"<afile>\"\0",
        )
    });
static e_no_autocommand_buffer_number_to_substitute_for_abuf: GlobalCell<
    [::core::ffi::c_char; 62],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
        *b"E496: No autocommand buffer number to substitute for \"<abuf>\"\0",
    )
});
static e_no_autocommand_match_name_to_substitute_for_amatch: GlobalCell<[::core::ffi::c_char; 61]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 61], [::core::ffi::c_char; 61]>(
            *b"E497: No autocommand match name to substitute for \"<amatch>\"\0",
        )
    });
static e_no_source_file_name_to_substitute_for_sfile: GlobalCell<[::core::ffi::c_char; 55]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
            *b"E498: No :source file name to substitute for \"<sfile>\"\0",
        )
    });
static e_no_line_number_to_use_for_slnum: GlobalCell<[::core::ffi::c_char; 42]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
            *b"E842: No line number to use for \"<slnum>\"\0",
        )
    });
static e_no_line_number_to_use_for_sflnum: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E961: No line number to use for \"<sflnum>\"\0",
        )
    });
static e_no_script_file_name_to_substitute_for_script: GlobalCell<[::core::ffi::c_char; 56]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
            *b"E1274: No script file name to substitute for \"<script>\"\0",
        )
    });
static quitmore: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static ex_pressedreturn: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static dollar_command: GlobalCell<[::core::ffi::c_char; 2]> =
    GlobalCell::new(['$' as ::core::ffi::c_char, 0 as ::core::ffi::c_char]);
unsafe extern "C" fn save_dbg_stuff(mut dsp: *mut dbg_stuff) {
    (*dsp).trylevel = trylevel.get();
    trylevel.set(0 as ::core::ffi::c_int);
    (*dsp).force_abort = force_abort.get() as ::core::ffi::c_int;
    force_abort.set(false_0 != 0);
    (*dsp).caught_stack = caught_stack.get();
    caught_stack.set(::core::ptr::null_mut::<except_T>());
    (*dsp).vv_exception = v_exception(::core::ptr::null_mut::<::core::ffi::c_char>());
    (*dsp).vv_throwpoint = v_throwpoint(::core::ptr::null_mut::<::core::ffi::c_char>());
    (*dsp).did_emsg = did_emsg.get();
    did_emsg.set(false_0);
    (*dsp).got_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false_0 != 0);
    (*dsp).did_throw = did_throw.get();
    did_throw.set(false_0 != 0);
    (*dsp).need_rethrow = need_rethrow.get() as ::core::ffi::c_int;
    need_rethrow.set(false_0 != 0);
    (*dsp).check_cstack = check_cstack.get() as ::core::ffi::c_int;
    check_cstack.set(false_0 != 0);
    (*dsp).current_exception = current_exception.get();
    current_exception.set(::core::ptr::null_mut::<except_T>());
}
unsafe extern "C" fn restore_dbg_stuff(mut dsp: *mut dbg_stuff) {
    suppress_errthrow.set(false_0 != 0);
    trylevel.set((*dsp).trylevel);
    force_abort.set((*dsp).force_abort != 0);
    caught_stack.set((*dsp).caught_stack);
    v_exception((*dsp).vv_exception);
    v_throwpoint((*dsp).vv_throwpoint);
    did_emsg.set((*dsp).did_emsg);
    got_int.set((*dsp).got_int != 0);
    did_throw.set((*dsp).did_throw);
    need_rethrow.set((*dsp).need_rethrow != 0);
    check_cstack.set((*dsp).check_cstack != 0);
    current_exception.set((*dsp).current_exception);
}
unsafe extern "C" fn is_other_file(
    mut fnum: ::core::ffi::c_int,
    mut ffname: *mut ::core::ffi::c_char,
) -> bool {
    if fnum != 0 as ::core::ffi::c_int {
        if fnum == (*curbuf.get()).handle {
            return false_0 != 0;
        }
        return true_0 != 0;
    }
    if ffname.is_null() {
        return true_0 != 0;
    }
    if *ffname as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if !(*curbuf.get()).file_id_valid
        && !(*curbuf.get()).b_sfname.is_null()
        && *(*curbuf.get()).b_sfname as ::core::ffi::c_int != NUL
    {
        return path_fnamecmp(ffname, (*curbuf.get()).b_sfname) != 0 as ::core::ffi::c_int;
    }
    return otherfile(ffname);
}
#[no_mangle]
pub unsafe extern "C" fn do_exmode() {
    exmode_active.set(true_0 != 0);
    State.set(MODE_NORMAL as ::core::ffi::c_int);
    may_trigger_modechanged();
    if global_busy.get() != 0 {
        return;
    }
    let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
    (*RedrawingDisabled.ptr()) += 1;
    (*no_wait_return.ptr()) += 1;
    msg(
        gettext(
            b"Entering Ex mode.  Type \"visual\" to go to Normal mode.\0".as_ptr()
                as *const ::core::ffi::c_char,
        ),
        0 as ::core::ffi::c_int,
    );
    while exmode_active.get() {
        if ex_normal_busy.get() > 0 as ::core::ffi::c_int
            && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        {
            exmode_active.set(false_0 != 0);
            break;
        } else {
            msg_scroll.set(true_0);
            need_wait_return.set(false_0 != 0);
            ex_pressedreturn.set(false_0 != 0);
            ex_no_reprint.set(false_0 != 0);
            let mut changedtick: varnumber_T = buf_get_changedtick(curbuf.get());
            let mut prev_msg_row: ::core::ffi::c_int = msg_row.get();
            let mut prev_line: linenr_T = (*curwin.get()).w_cursor.lnum;
            cmdline_row.set(msg_row.get());
            do_cmdline(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                Some(
                    getexline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
                NULL_1,
                0 as ::core::ffi::c_int,
            );
            lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
            if (prev_line != (*curwin.get()).w_cursor.lnum
                || changedtick != buf_get_changedtick(curbuf.get()))
                && !ex_no_reprint.get()
            {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(
                        &raw const e_empty_buffer as *const ::core::ffi::c_char,
                    ));
                } else {
                    if ex_pressedreturn.get() {
                        msg_scroll_flush();
                        msg_row.set(prev_msg_row);
                        if prev_msg_row == Rows.get() - 1 as ::core::ffi::c_int {
                            (*msg_row.ptr()) -= 1;
                        }
                    }
                    msg_col.set(0 as ::core::ffi::c_int);
                    print_line_no_prefix((*curwin.get()).w_cursor.lnum, false_0 != 0, false_0 != 0);
                    msg_clr_eos();
                }
            } else if ex_pressedreturn.get() as ::core::ffi::c_int != 0 && !ex_no_reprint.get() {
                if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                    emsg(gettext(
                        &raw const e_empty_buffer as *const ::core::ffi::c_char,
                    ));
                } else {
                    emsg(gettext(
                        b"E501: At end-of-file\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
            }
        }
    }
    (*RedrawingDisabled.ptr()) -= 1;
    (*no_wait_return.ptr()) -= 1;
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    update_screen();
    need_wait_return.set(false_0 != 0);
    msg_scroll.set(save_msg_scroll);
}
unsafe extern "C" fn msg_verbose_cmd(mut lnum: linenr_T, mut cmd: *mut ::core::ffi::c_char) {
    (*no_wait_return.ptr()) += 1;
    verbose_enter_scroll();
    if lnum == 0 as linenr_T {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Executing: %s\0".as_ptr() as *const ::core::ffi::c_char),
            cmd,
        );
    } else {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"line %d: %s\0".as_ptr() as *const ::core::ffi::c_char),
            lnum,
            cmd,
        );
    }
    if msg_silent.get() == 0 as ::core::ffi::c_int {
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    }
    verbose_leave_scroll();
    (*no_wait_return.ptr()) -= 1;
}
static cmdline_call_depth: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
unsafe extern "C" fn do_cmdline_start() -> ::core::ffi::c_int {
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                364 as ::core::ffi::c_uint,
                b"int do_cmdline_start(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if cmdline_call_depth.get() >= 200 as ::core::ffi::c_int
        && cmdline_call_depth.get() as OptInt >= p_mfd.get()
    {
        return FAIL;
    }
    (*cmdline_call_depth.ptr()) += 1;
    crate::src::nvim::clipboard::start_batch_changes();
    return OK;
}
unsafe extern "C" fn do_cmdline_end() {
    (*cmdline_call_depth.ptr()) -= 1;
    '_c2rust_label: {
        if cmdline_call_depth.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"cmdline_call_depth >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                380 as ::core::ffi::c_uint,
                b"void do_cmdline_end(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    crate::src::nvim::clipboard::end_batch_changes();
}
#[no_mangle]
pub unsafe extern "C" fn do_cmdline_cmd(mut cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return do_cmdline(
        cmd as *mut ::core::ffi::c_char,
        None,
        NULL_1,
        DOCMD_VERBOSE as ::core::ffi::c_int
            | DOCMD_NOWAIT as ::core::ffi::c_int
            | DOCMD_KEYTYPED as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn do_cmdline(
    mut cmdline: *mut ::core::ffi::c_char,
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut next_cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmdline_copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut used_getline: bool = false_0 != 0;
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut msg_didout_before_start: bool = false_0 != 0;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_inc: bool = false_0 != 0;
    let mut did_block: bool = false_0 != 0;
    let mut retval: ::core::ffi::c_int = OK;
    let mut cstack: cstack_T = cstack_T {
        cs_flags: [0; 50],
        cs_pending: [0; 50],
        cs_pend: C2Rust_Unnamed_34 {
            csp_rv: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
        },
        cs_forinfo: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
        cs_line: [0; 50],
        cs_idx: -1 as ::core::ffi::c_int,
        cs_looplevel: 0,
        cs_trylevel: 0,
        cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
        cs_lflags: 0,
    };
    let mut lines_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut current_line: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut breakpoint: *mut linenr_T = ::core::ptr::null_mut::<linenr_T>();
    let mut dbg_tick: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut debug_saved: dbg_stuff = dbg_stuff {
        trylevel: 0,
        force_abort: 0,
        caught_stack: ::core::ptr::null_mut::<except_T>(),
        vv_exception: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        vv_throwpoint: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        did_emsg: 0,
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        check_cstack: 0,
        current_exception: ::core::ptr::null_mut::<except_T>(),
    };
    let mut private_msg_list: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    let mut cmd_getline: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
            ::core::ffi::c_int,
            bool,
        ) -> *mut ::core::ffi::c_char,
    > = None;
    let mut cmd_cookie: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut cmd_loop_cookie: loop_cookie = loop_cookie {
        lines_gap: ::core::ptr::null_mut::<garray_T>(),
        current_line: 0,
        repeating: 0,
        lc_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut saved_msg_list: *mut *mut msglist_T = msg_list.get();
    msg_list.set(&raw mut private_msg_list);
    private_msg_list = ::core::ptr::null_mut::<msglist_T>();
    if do_cmdline_start() == FAIL {
        emsg(gettext(
            &raw const e_command_too_recursive as *const ::core::ffi::c_char,
        ));
        do_errthrow(
            NULL_1 as *mut cstack_T,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        msg_list.set(saved_msg_list);
        return FAIL;
    }
    ga_init(
        &raw mut lines_ga,
        ::core::mem::size_of::<wcmd_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut real_cookie: *mut ::core::ffi::c_void = getline_cookie(fgetline, cookie);
    let mut getline_is_func: bool = getline_equal(
        fgetline,
        cookie,
        Some(
            get_func_line
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ),
    );
    if getline_is_func as ::core::ffi::c_int != 0
        && ex_nesting_level.get() == func_level(real_cookie)
    {
        (*ex_nesting_level.ptr()) += 1;
    }
    if getline_is_func {
        fname = func_name(real_cookie);
        breakpoint = func_breakpoint(real_cookie);
        dbg_tick = func_dbg_tick(real_cookie);
    } else if getline_equal(
        fgetline,
        cookie,
        Some(
            getsourceline
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ),
    ) {
        fname = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name;
        breakpoint = source_breakpoint(real_cookie);
        dbg_tick = source_dbg_tick(real_cookie);
    }
    if recursive.get() == 0 {
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
    }
    if flags & DOCMD_EXCRESET as ::core::ffi::c_int != 0 {
        save_dbg_stuff(&raw mut debug_saved);
    } else {
        memset(
            &raw mut debug_saved as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<dbg_stuff>(),
        );
    }
    let mut initial_trylevel: ::core::ffi::c_int = trylevel.get();
    did_throw.set(false_0 != 0);
    did_emsg.set(false_0);
    if flags & DOCMD_KEYTYPED as ::core::ffi::c_int == 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(
                getexline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        )
    {
        KeyTyped.set(false_0 != 0);
    }
    next_cmdline = cmdline;
    loop {
        getline_is_func = getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        );
        if next_cmdline.is_null()
            && !force_abort.get()
            && cstack.cs_idx < 0 as ::core::ffi::c_int
            && !(getline_is_func as ::core::ffi::c_int != 0 && func_has_abort(real_cookie) != 0)
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as ::core::ffi::c_int && current_line < lines_ga.ga_len {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut cmdline_copy as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_1;
            let _ = *ptr_;
            if getline_is_func {
                if do_profiling.get() == PROF_YES {
                    func_line_end(real_cookie);
                }
                if func_has_ended(real_cookie) != 0 {
                    retval = FAIL;
                    break;
                }
            } else if do_profiling.get() == PROF_YES
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                ) as ::core::ffi::c_int
                    != 0
            {
                script_line_end();
            }
            if source_finished(fgetline, cookie) {
                retval = FAIL;
                break;
            } else {
                if !breakpoint.is_null() && !dbg_tick.is_null() && *dbg_tick != debug_tick.get() {
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        ::core::ffi::c_int,
                                        *mut ::core::ffi::c_void,
                                        ::core::ffi::c_int,
                                        bool,
                                    )
                                        -> *mut ::core::ffi::c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                next_cmdline =
                    (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).line;
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum;
                if !breakpoint.is_null()
                    && *breakpoint != 0 as linenr_T
                    && *breakpoint
                        <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum
                {
                    dbg_breakpoint(
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum,
                    );
                    *breakpoint = dbg_find_breakpoint(
                        getline_equal(
                            fgetline,
                            cookie,
                            Some(
                                getsourceline
                                    as unsafe extern "C" fn(
                                        ::core::ffi::c_int,
                                        *mut ::core::ffi::c_void,
                                        ::core::ffi::c_int,
                                        bool,
                                    )
                                        -> *mut ::core::ffi::c_char,
                            ),
                        ),
                        fname,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum,
                    );
                    *dbg_tick = debug_tick.get();
                }
                if do_profiling.get() == PROF_YES {
                    if getline_is_func {
                        func_line_start(real_cookie);
                    } else if getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getsourceline
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    *mut ::core::ffi::c_void,
                                    ::core::ffi::c_int,
                                    bool,
                                )
                                    -> *mut ::core::ffi::c_char,
                        ),
                    ) {
                        script_line_start();
                    }
                }
            }
        }
        if next_cmdline.is_null() {
            let mut indent: ::core::ffi::c_int = if cstack.cs_idx < 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                (cstack.cs_idx + 1 as ::core::ffi::c_int) * 2 as ::core::ffi::c_int
            };
            if count == 1 as ::core::ffi::c_int
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                ) as ::core::ffi::c_int
                    != 0
            {
                if ui_has(kUICmdline) {
                    ui_ext_cmdline_block_append(0 as size_t, last_cmdline.get());
                    did_block = true_0 != 0;
                }
                msg_didout.set(true_0 != 0);
            }
            if fgetline.is_none() || {
                next_cmdline = fgetline.expect("non-null function pointer")(
                    ':' as ::core::ffi::c_int,
                    cookie,
                    indent,
                    true_0 != 0,
                );
                next_cmdline.is_null()
            } {
                if KeyTyped.get() as ::core::ffi::c_int != 0
                    && flags & DOCMD_REPEAT as ::core::ffi::c_int == 0
                {
                    need_wait_return.set(false_0 != 0);
                }
                retval = FAIL;
                break;
            } else {
                used_getline = true_0 != 0;
                if ui_has(kUICmdline) as ::core::ffi::c_int != 0
                    && count > 0 as ::core::ffi::c_int
                    && getline_equal(
                        fgetline,
                        cookie,
                        Some(
                            getexline
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    *mut ::core::ffi::c_void,
                                    ::core::ffi::c_int,
                                    bool,
                                )
                                    -> *mut ::core::ffi::c_char,
                        ),
                    ) as ::core::ffi::c_int
                        != 0
                {
                    ui_ext_cmdline_block_append(indent as size_t, next_cmdline);
                }
                if flags & DOCMD_KEEPLINE as ::core::ffi::c_int != 0 {
                    xfree(repeat_cmdline.get() as *mut ::core::ffi::c_void);
                    if count == 0 as ::core::ffi::c_int {
                        repeat_cmdline.set(xstrdup(next_cmdline));
                    } else {
                        repeat_cmdline.set(::core::ptr::null_mut::<::core::ffi::c_char>());
                    }
                }
            }
        } else if cmdline_copy.is_null() {
            next_cmdline = xstrdup(next_cmdline);
        }
        cmdline_copy = next_cmdline;
        let mut current_line_before: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if cstack.cs_looplevel > 0 as ::core::ffi::c_int
            || has_loop_cmd(next_cmdline) as ::core::ffi::c_int != 0
        {
            cmd_getline = Some(
                get_loop_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            )
                as Option<
                    unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
                >;
            cmd_cookie = &raw mut cmd_loop_cookie as *mut ::core::ffi::c_void;
            cmd_loop_cookie.lines_gap = &raw mut lines_ga;
            cmd_loop_cookie.current_line = current_line;
            cmd_loop_cookie.lc_getline = fgetline;
            cmd_loop_cookie.cookie = cookie;
            cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len) as ::core::ffi::c_int;
            if current_line == lines_ga.ga_len {
                store_loop_line(&raw mut lines_ga, next_cmdline);
            }
            current_line_before = current_line;
        } else {
            cmd_getline = fgetline
                as Option<
                    unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
                >;
            cmd_cookie = cookie;
        }
        did_endif.set(false_0 != 0);
        let c2rust_fresh0 = count;
        count = count + 1;
        if c2rust_fresh0 == 0 as ::core::ffi::c_int {
            if flags & DOCMD_NOWAIT as ::core::ffi::c_int == 0 && recursive.get() == 0 {
                msg_didout_before_start = msg_didout.get();
                msg_didany.set(false_0 != 0);
                msg_start();
                msg_scroll.set(true_0);
                (*no_wait_return.ptr()) += 1;
                (*RedrawingDisabled.ptr()) += 1;
                did_inc = true_0 != 0;
            }
        }
        if p_verbose.get() >= 15 as OptInt
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
            || p_verbose.get() >= 16 as OptInt
        {
            msg_verbose_cmd(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
                cmdline_copy,
            );
        }
        (*recursive.ptr()) += 1;
        next_cmdline = do_one_cmd(
            &raw mut cmdline_copy,
            flags,
            &raw mut cstack,
            cmd_getline as LineGetter,
            cmd_cookie,
        );
        (*recursive.ptr()) -= 1;
        if cmd_cookie == &raw mut cmd_loop_cookie as *mut ::core::ffi::c_void {
            current_line = cmd_loop_cookie.current_line;
        }
        if next_cmdline.is_null() {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut cmdline_copy as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_1;
            let _ = *ptr__0;
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) as ::core::ffi::c_int
                != 0
                && !(*new_last_cmdline.ptr()).is_null()
            {
                xfree(last_cmdline.get() as *mut ::core::ffi::c_void);
                last_cmdline.set(new_last_cmdline.get());
                new_last_cmdline.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            }
        } else {
            memmove(
                cmdline_copy as *mut ::core::ffi::c_void,
                next_cmdline as *const ::core::ffi::c_void,
                strlen(next_cmdline).wrapping_add(1 as size_t),
            );
            next_cmdline = cmdline_copy;
        }
        if did_emsg.get() != 0
            && !force_abort.get()
            && getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) as ::core::ffi::c_int
                != 0
            && func_has_abort(real_cookie) == 0
        {
            did_emsg.set(false_0);
        }
        if cstack.cs_looplevel > 0 as ::core::ffi::c_int {
            current_line += 1;
            if cstack.cs_lflags
                & (CSL_HAD_CONT as ::core::ffi::c_int | CSL_HAD_ENDLOOP as ::core::ffi::c_int)
                != 0
            {
                cstack.cs_lflags &=
                    !(CSL_HAD_CONT as ::core::ffi::c_int | CSL_HAD_ENDLOOP as ::core::ffi::c_int);
                if did_emsg.get() == 0
                    && !got_int.get()
                    && !did_throw.get()
                    && cstack.cs_idx >= 0 as ::core::ffi::c_int
                    && cstack.cs_flags[cstack.cs_idx as usize]
                        & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int)
                        != 0
                    && cstack.cs_line[cstack.cs_idx as usize] >= 0 as ::core::ffi::c_int
                    && cstack.cs_flags[cstack.cs_idx as usize] & CSF_ACTIVE as ::core::ffi::c_int
                        != 0
                {
                    current_line = cstack.cs_line[cstack.cs_idx as usize];
                    cstack.cs_lflags |= CSL_HAD_LOOP as ::core::ffi::c_int;
                    line_breakcheck();
                    if !breakpoint.is_null() && lines_ga.ga_len > current_line {
                        *breakpoint = dbg_find_breakpoint(
                            getline_equal(
                                fgetline,
                                cookie,
                                Some(
                                    getsourceline
                                        as unsafe extern "C" fn(
                                            ::core::ffi::c_int,
                                            *mut ::core::ffi::c_void,
                                            ::core::ffi::c_int,
                                            bool,
                                        )
                                            -> *mut ::core::ffi::c_char,
                                ),
                            ),
                            fname,
                            (*(lines_ga.ga_data as *mut wcmd_T).offset(current_line as isize)).lnum
                                - 1 as linenr_T,
                        );
                        *dbg_tick = debug_tick.get();
                    }
                } else if cstack.cs_idx >= 0 as ::core::ffi::c_int {
                    rewind_conditionals(
                        &raw mut cstack,
                        cstack.cs_idx - 1 as ::core::ffi::c_int,
                        CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                        &raw mut cstack.cs_looplevel,
                    );
                }
            } else if cstack.cs_lflags & CSL_HAD_LOOP as ::core::ffi::c_int != 0 {
                cstack.cs_lflags &= !(CSL_HAD_LOOP as ::core::ffi::c_int);
                cstack.cs_line[cstack.cs_idx as usize] = current_line_before;
            }
        }
        if cstack.cs_looplevel == 0 as ::core::ffi::c_int {
            if !(lines_ga.ga_len <= 0 as ::core::ffi::c_int) {
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum = (*(lines_ga.ga_data as *mut wcmd_T)
                    .offset((lines_ga.ga_len - 1 as ::core::ffi::c_int) as isize))
                .lnum;
                let mut _gap: *mut garray_T = &raw mut lines_ga;
                if !(*_gap).ga_data.is_null() {
                    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i < (*_gap).ga_len {
                        let mut _item: *mut wcmd_T =
                            ((*_gap).ga_data as *mut wcmd_T).offset(i as isize);
                        xfree((*_item).line as *mut ::core::ffi::c_void);
                        i += 1;
                    }
                }
                ga_clear(_gap);
            }
            current_line = 0 as ::core::ffi::c_int;
        }
        if cstack.cs_lflags & CSL_HAD_FINA as ::core::ffi::c_int != 0 {
            cstack.cs_lflags &= !(CSL_HAD_FINA as ::core::ffi::c_int);
            report_make_pending(
                cstack.cs_pending[cstack.cs_idx as usize] as ::core::ffi::c_int
                    & (CSTP_ERROR as ::core::ffi::c_int
                        | CSTP_INTERRUPT as ::core::ffi::c_int
                        | CSTP_THROW as ::core::ffi::c_int),
                (if did_throw.get() as ::core::ffi::c_int != 0 {
                    current_exception.get()
                } else {
                    ::core::ptr::null_mut::<except_T>()
                }) as *mut ::core::ffi::c_void,
            );
            did_throw.set(false_0 != 0);
            got_int.set(did_throw.get());
            did_emsg.set(got_int.get() as ::core::ffi::c_int);
            cstack.cs_flags[cstack.cs_idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_FINALLY as ::core::ffi::c_int;
        }
        trylevel.set(initial_trylevel + cstack.cs_trylevel);
        if trylevel.get() == 0 as ::core::ffi::c_int
            && did_emsg.get() == 0
            && !got_int.get()
            && !did_throw.get()
        {
            force_abort.set(false_0 != 0);
        }
        do_intthrow(&raw mut cstack);
        if !(!((got_int.get() as ::core::ffi::c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0)
            && cstack.cs_trylevel == 0 as ::core::ffi::c_int)
            && !(did_emsg.get() != 0
                && (cstack.cs_trylevel == 0 as ::core::ffi::c_int
                    || did_emsg_syntax.get() as ::core::ffi::c_int != 0)
                && used_getline as ::core::ffi::c_int != 0
                && getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getexline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                ) as ::core::ffi::c_int
                    != 0)
            && (!next_cmdline.is_null()
                || cstack.cs_idx >= 0 as ::core::ffi::c_int
                || flags & DOCMD_REPEAT as ::core::ffi::c_int != 0))
        {
            break;
        }
    }
    xfree(cmdline_copy as *mut ::core::ffi::c_void);
    did_emsg_syntax.set(false_0 != 0);
    let mut _gap_0: *mut garray_T = &raw mut lines_ga;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut wcmd_T = ((*_gap_0).ga_data as *mut wcmd_T).offset(i_0 as isize);
            xfree((*_item_0).line as *mut ::core::ffi::c_void);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    if cstack.cs_idx >= 0 as ::core::ffi::c_int {
        if !got_int.get()
            && !did_throw.get()
            && !aborting()
            && (getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) as ::core::ffi::c_int
                != 0
                && !source_finished(fgetline, cookie)
                || getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        get_func_line
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                ) as ::core::ffi::c_int
                    != 0
                    && func_has_ended(real_cookie) == 0)
        {
            if cstack.cs_flags[cstack.cs_idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
                emsg(gettext(&raw const e_endtry as *const ::core::ffi::c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_WHILE as ::core::ffi::c_int != 0
            {
                emsg(gettext(&raw const e_endwhile as *const ::core::ffi::c_char));
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_FOR as ::core::ffi::c_int != 0 {
                emsg(gettext(&raw const e_endfor as *const ::core::ffi::c_char));
            } else {
                emsg(gettext(&raw const e_endif as *const ::core::ffi::c_char));
            }
        }
        loop {
            let mut idx: ::core::ffi::c_int =
                cleanup_conditionals(&raw mut cstack, 0 as ::core::ffi::c_int, true_0);
            if idx >= 0 as ::core::ffi::c_int {
                idx -= 1;
            }
            rewind_conditionals(
                &raw mut cstack,
                idx,
                CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                &raw mut cstack.cs_looplevel,
            );
            if cstack.cs_idx < 0 as ::core::ffi::c_int {
                break;
            }
        }
        trylevel.set(initial_trylevel);
    }
    do_errthrow(
        &raw mut cstack,
        (if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) as ::core::ffi::c_int
            != 0
        {
            b"endfunction\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        }) as *mut ::core::ffi::c_char,
    );
    if trylevel.get() == 0 as ::core::ffi::c_int {
        if did_throw.get() {
            handle_did_throw();
        } else if got_int.get() as ::core::ffi::c_int != 0
            || did_emsg.get() != 0 && force_abort.get() as ::core::ffi::c_int != 0
        {
            suppress_errthrow.set(true_0 != 0);
        }
    }
    if did_throw.get() {
        need_rethrow.set(true_0 != 0);
    }
    if getline_equal(
        fgetline,
        cookie,
        Some(
            getsourceline
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ),
    ) as ::core::ffi::c_int
        != 0
        && ex_nesting_level.get() > source_level(real_cookie)
        || getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) as ::core::ffi::c_int
            != 0
            && ex_nesting_level.get() > func_level(real_cookie) + 1 as ::core::ffi::c_int
    {
        if !did_throw.get() {
            check_cstack.set(true_0 != 0);
        }
    } else {
        if getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) {
            (*ex_nesting_level.ptr()) -= 1;
        }
        if (getline_equal(
            fgetline,
            cookie,
            Some(
                getsourceline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        ) as ::core::ffi::c_int
            != 0
            || getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) as ::core::ffi::c_int
                != 0)
            && ex_nesting_level.get() + 1 as ::core::ffi::c_int <= debug_break_level.get()
        {
            do_debug(
                if getline_equal(
                    fgetline,
                    cookie,
                    Some(
                        getsourceline
                            as unsafe extern "C" fn(
                                ::core::ffi::c_int,
                                *mut ::core::ffi::c_void,
                                ::core::ffi::c_int,
                                bool,
                            )
                                -> *mut ::core::ffi::c_char,
                    ),
                ) as ::core::ffi::c_int
                    != 0
                {
                    gettext(b"End of sourced file\0".as_ptr() as *const ::core::ffi::c_char)
                } else {
                    gettext(b"End of function\0".as_ptr() as *const ::core::ffi::c_char)
                },
            );
        }
    }
    if flags & DOCMD_EXCRESET as ::core::ffi::c_int != 0 {
        restore_dbg_stuff(&raw mut debug_saved);
    }
    msg_list.set(saved_msg_list);
    if !cstack.cs_emsg_silent_list.is_null() {
        let mut temp: *mut eslist_T = ::core::ptr::null_mut::<eslist_T>();
        let mut elem: *mut eslist_T = cstack.cs_emsg_silent_list;
        while !elem.is_null() {
            temp = (*elem).next;
            xfree(elem as *mut ::core::ffi::c_void);
            elem = temp;
        }
    }
    if did_inc {
        (*RedrawingDisabled.ptr()) -= 1;
        (*no_wait_return.ptr()) -= 1;
        msg_scroll.set(false_0);
        if retval == FAIL
            || did_endif.get() as ::core::ffi::c_int != 0
                && KeyTyped.get() as ::core::ffi::c_int != 0
                && did_emsg.get() == 0
        {
            need_wait_return.set(false_0 != 0);
            msg_didany.set(false_0 != 0);
        } else if need_wait_return.get() {
            msg_didout.set(
                msg_didout.get() as ::core::ffi::c_int
                    | msg_didout_before_start as ::core::ffi::c_int
                    != 0,
            );
            wait_return(false_0);
        }
    }
    if did_block {
        ui_ext_cmdline_block_leave();
    }
    did_endif.set(false_0 != 0);
    do_cmdline_end();
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn handle_did_throw() {
    '_c2rust_label: {
        if !(*current_exception.ptr()).is_null() {
        } else {
            __assert_fail(
                b"current_exception != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                974 as ::core::ffi::c_uint,
                b"void handle_did_throw(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut messages: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    match (*current_exception.get()).type_0 as ::core::ffi::c_uint {
        0 => {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"E605: Exception not caught: %s\0".as_ptr() as *const ::core::ffi::c_char),
                (*current_exception.get()).value,
            );
            p = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        }
        1 => {
            messages = (*current_exception.get()).messages;
            (*current_exception.get()).messages = ::core::ptr::null_mut::<msglist_T>();
        }
        2 | _ => {}
    }
    estack_push(
        ETYPE_EXCEPT,
        (*current_exception.get()).throw_name,
        (*current_exception.get()).throw_lnum,
    );
    (*current_exception.get()).throw_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    discard_current_exception();
    if emsg_silent.get() == 0 as ::core::ffi::c_int {
        suppress_errthrow.set(true_0 != 0);
        force_abort.set(true_0 != 0);
    }
    if !messages.is_null() {
        loop {
            let mut next: *mut msglist_T = (*messages).next;
            emsg_multiline(
                (*messages).msg,
                b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
                HLF_E as ::core::ffi::c_int,
                (*messages).multiline,
            );
            xfree((*messages).msg as *mut ::core::ffi::c_void);
            xfree((*messages).sfile as *mut ::core::ffi::c_void);
            xfree(messages as *mut ::core::ffi::c_void);
            messages = next;
            if messages.is_null() {
                break;
            }
        }
    } else if !p.is_null() {
        emsg(p);
        xfree(p as *mut ::core::ffi::c_void);
    }
    xfree(
        (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name as *mut ::core::ffi::c_void,
    );
    estack_pop();
}
unsafe extern "C" fn get_loop_line(
    mut c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    if (*cp).current_line + 1 as ::core::ffi::c_int >= (*(*cp).lines_gap).ga_len {
        if (*cp).repeating != 0 {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*cp).lc_getline.is_none() {
            line = getcmdline(c, 0 as ::core::ffi::c_int, indent, do_concat);
        } else {
            line = (*cp).lc_getline.expect("non-null function pointer")(
                c,
                (*cp).cookie,
                indent,
                do_concat,
            );
        }
        if !line.is_null() {
            store_loop_line((*cp).lines_gap, line);
            (*cp).current_line += 1;
        }
        return line;
    }
    KeyTyped.set(false_0 != 0);
    (*cp).current_line += 1;
    let mut wp: *mut wcmd_T =
        ((*(*cp).lines_gap).ga_data as *mut wcmd_T).offset((*cp).current_line as isize);
    (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum = (*wp).lnum;
    return xstrdup((*wp).line);
}
unsafe extern "C" fn store_loop_line(mut gap: *mut garray_T, mut line: *mut ::core::ffi::c_char) {
    let mut p: *mut wcmd_T =
        ga_append_via_ptr(gap, ::core::mem::size_of::<wcmd_T>()) as *mut wcmd_T;
    (*p).line = xstrdup(line);
    (*p).lnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
}
#[no_mangle]
pub unsafe extern "C" fn getline_equal(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
    mut func: LineGetter,
) -> bool {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while gp
        == Some(
            get_loop_line
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        )
    {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return gp == func;
}
#[no_mangle]
pub unsafe extern "C" fn getline_cookie(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    let mut gp: LineGetter = fgetline;
    let mut cp: *mut loop_cookie = cookie as *mut loop_cookie;
    while gp
        == Some(
            get_loop_line
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        )
    {
        gp = (*cp).lc_getline;
        cp = (*cp).cookie as *mut loop_cookie;
    }
    return cp as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn compute_buffer_local_count(
    mut addr_type: cmd_addr_T,
    mut lnum: linenr_T,
    mut offset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = offset;
    let mut buf: *mut buf_T = firstbuf.get();
    while !(*buf).b_next.is_null() && ((*buf).handle as linenr_T) < lnum {
        buf = (*buf).b_next;
    }
    while count != 0 as ::core::ffi::c_int {
        count += if count < 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        let mut nextbuf: *mut buf_T = if offset < 0 as ::core::ffi::c_int {
            (*buf).b_prev
        } else {
            (*buf).b_next
        };
        if nextbuf.is_null() {
            break;
        }
        buf = nextbuf;
        if addr_type as ::core::ffi::c_uint
            == ADDR_LOADED_BUFFERS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            while (*buf).b_ml.ml_mfp.is_null() {
                nextbuf = if offset < 0 as ::core::ffi::c_int {
                    (*buf).b_prev
                } else {
                    (*buf).b_next
                };
                if nextbuf.is_null() {
                    break;
                }
                buf = nextbuf;
            }
        }
    }
    if addr_type as ::core::ffi::c_uint
        == ADDR_LOADED_BUFFERS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        while (*buf).b_ml.ml_mfp.is_null() {
            let mut nextbuf_0: *mut buf_T = if offset >= 0 as ::core::ffi::c_int {
                (*buf).b_prev
            } else {
                (*buf).b_next
            };
            if nextbuf_0.is_null() {
                break;
            }
            buf = nextbuf_0;
        }
    }
    return (*buf).handle as ::core::ffi::c_int;
}
unsafe extern "C" fn current_win_nr(mut win: *const win_T) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        nr += 1;
        if wp == win as *mut win_T {
            break;
        }
        wp = (*wp).w_next;
    }
    return nr;
}
unsafe extern "C" fn current_tab_nr(mut tab: *mut tabpage_T) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        nr += 1;
        if tp == tab {
            break;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return nr;
}
unsafe extern "C" fn get_wincmd_addr_type(
    mut arg: *const ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) {
    match *arg as ::core::ffi::c_int {
        83 | Ctrl_S | 115 | Ctrl_N | 110 | 106 | Ctrl_J | 107 | Ctrl_K | 84 | Ctrl_R | 114 | 82
        | 75 | 74 | 43 | 45 | Ctrl__ | 95 | 124 | 93 | Ctrl_RSB | 103 | Ctrl_G | Ctrl_V | 118
        | 104 | Ctrl_H | 108 | Ctrl_L | 72 | 76 | 62 | 60 | 125 | 102 | 70 | Ctrl_F | 105
        | Ctrl_I | 100 | Ctrl_D => {
            (*eap).addr_type = ADDR_OTHER;
        }
        Ctrl_HAT | 94 => {
            (*eap).addr_type = ADDR_BUFFERS;
        }
        Ctrl_Q | 113 | Ctrl_C | 99 | Ctrl_O | 111 | Ctrl_W | 119 | 87 | 120 | Ctrl_X => {
            (*eap).addr_type = ADDR_WINDOWS;
        }
        Ctrl_Z | 122 | 80 | 116 | Ctrl_T | 98 | Ctrl_B | 112 | Ctrl_P | 61 | CAR => {
            (*eap).addr_type = ADDR_NONE;
        }
        _ => {}
    };
}
unsafe extern "C" fn skip_colon_white(
    mut p: *const ::core::ffi::c_char,
    mut skipleadingwhite: bool,
) -> *mut ::core::ffi::c_char {
    if skipleadingwhite {
        p = skipwhite(p);
    }
    while *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    }
    return p as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn set_cmd_addr_type(mut eap: *mut exarg_T, mut p: *mut ::core::ffi::c_char) {
    if ((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int {
        (*eap).addr_type =
            (*cmdnames.ptr())[(*eap).cmdidx as ::core::ffi::c_int as usize].cmd_addr_type;
    } else {
        (*eap).addr_type = ADDR_LINES;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_wincmd as ::core::ffi::c_int && !p.is_null() {
        get_wincmd_addr_type(skipwhite(p), eap);
    }
    if ((*eap).cmdidx as ::core::ffi::c_int == CMD_cc as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_ll as ::core::ffi::c_int)
        && bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
    {
        (*eap).addr_type = ADDR_OTHER;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_cmd_default_range(mut eap: *mut exarg_T) -> linenr_T {
    match (*eap).addr_type as ::core::ffi::c_uint {
        0 | 10 => {
            return if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
        }
        1 => return current_win_nr(curwin.get()) as linenr_T,
        2 => {
            return if ((*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int)
                < (*(*curwin.get()).w_alist).al_ga.ga_len
            {
                (*curwin.get()).w_arg_idx as linenr_T + 1 as linenr_T
            } else {
                (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
            };
        }
        3 | 4 => return (*curbuf.get()).handle as linenr_T,
        5 => return current_tab_nr(curtab.get()) as linenr_T,
        6 | 9 => return 1 as linenr_T,
        8 => return qf_get_cur_idx(eap) as linenr_T,
        7 => return qf_get_cur_valid_idx(eap) as linenr_T,
        _ => return 0 as linenr_T,
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_cmd_dflall_range(mut eap: *mut exarg_T) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
    match (*eap).addr_type as ::core::ffi::c_uint {
        0 | 10 => {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
        3 => {
            buf = firstbuf.get();
            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_next;
            }
            (*eap).line1 = (*buf).handle as linenr_T;
            buf = lastbuf.get();
            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                buf = (*buf).b_prev;
            }
            (*eap).line2 = (*buf).handle as linenr_T;
        }
        4 => {
            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
        }
        1 => {
            (*eap).line2 = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
        }
        5 => {
            (*eap).line2 = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
        }
        6 => {
            (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
        }
        2 => {
            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
                (*eap).line2 = 0 as ::core::ffi::c_int as linenr_T;
                (*eap).line1 = (*eap).line2;
            } else {
                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
            }
        }
        7 => {
            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
            if (*eap).line2 == 0 as linenr_T {
                (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
            }
        }
        11 | 9 | 8 => {
            iemsg(gettext(
                b"INTERNAL: Cannot use EX_DFLALL with ADDR_NONE, ADDR_UNSIGNED or ADDR_QUICKFIX\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ));
        }
        _ => {}
    };
}
unsafe extern "C" fn parse_register(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_REGSTR as uint32_t != 0
        && *(*eap).arg as ::core::ffi::c_int != NUL
        && (!(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
            || *(*eap).arg as ::core::ffi::c_int != '=' as ::core::ffi::c_int)
        && !((*eap).argt & EX_COUNT as uint32_t != 0
            && ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        if valid_yank_reg(
            *(*eap).arg as ::core::ffi::c_int,
            !(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_put as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_iput as ::core::ffi::c_int,
        ) {
            let c2rust_fresh25 = (*eap).arg;
            (*eap).arg = (*eap).arg.offset(1);
            (*eap).regname = *c2rust_fresh25 as uint8_t as ::core::ffi::c_int;
            if *(*eap).arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
                && *(*eap).arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                if (*eap).skip == 0 {
                    set_expr_line(xstrdup((*eap).arg));
                }
                (*eap).arg = (*eap).arg.offset(strlen((*eap).arg) as isize);
            }
            (*eap).arg = skipwhite((*eap).arg);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_cmd_count(
    mut eap: *mut exarg_T,
    mut count: linenr_T,
    mut validate: bool,
) {
    if (*eap).addr_type as ::core::ffi::c_uint
        != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*eap).line2 = count;
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            (*eap).addr_count = 1 as ::core::ffi::c_int;
        }
    } else {
        (*eap).line1 = (*eap).line2;
        if (*eap).line2 >= INT32_MAX as linenr_T - (count - 1 as linenr_T) {
            (*eap).line2 = INT32_MAX as linenr_T;
        } else {
            (*eap).line2 = ((*eap).line2 as ::core::ffi::c_int
                + (count - 1 as linenr_T) as ::core::ffi::c_int)
                as linenr_T;
        }
        (*eap).addr_count += 1;
        if validate as ::core::ffi::c_int != 0 && (*eap).line2 > (*curbuf.get()).b_ml.ml_line_count
        {
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
    };
}
unsafe extern "C" fn parse_count(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
    mut validate: bool,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*eap).argt & EX_COUNT as uint32_t != 0
        && ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && ((*eap).argt & EX_BUFNAME as uint32_t == 0
            || {
                p = skipdigits((*eap).arg.offset(1 as ::core::ffi::c_int as isize));
                *p as ::core::ffi::c_int == NUL
            }
            || ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
    {
        let mut n: linenr_T =
            getdigits_int32(&raw mut (*eap).arg, false_0 != 0, INT32_MAX as int32_t);
        (*eap).arg = skipwhite((*eap).arg);
        if !(*eap).args.is_null() {
            '_c2rust_label: {
                if (*eap).argc > 0 as size_t
                    && (*eap).arg >= *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
                {
                } else {
                    __assert_fail(
                        b"eap->argc > 0 && eap->arg >= eap->args[0]\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1467 as ::core::ffi::c_uint,
                        b"int parse_count(exarg_T *, const char **, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if (*eap).arg
                < (*(*eap).args.offset(0 as ::core::ffi::c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as ::core::ffi::c_int as isize) as isize)
            {
                *(*eap).arglens.offset(0 as ::core::ffi::c_int as isize) =
                    (*(*eap).arglens.offset(0 as ::core::ffi::c_int as isize)).wrapping_sub(
                        (*eap)
                            .arg
                            .offset_from(*(*eap).args.offset(0 as ::core::ffi::c_int as isize))
                            as size_t,
                    );
                *(*eap).args.offset(0 as ::core::ffi::c_int as isize) = (*eap).arg;
            } else {
                shift_cmd_args(eap);
            }
        }
        if n <= 0 as linenr_T && (*eap).argt & EX_ZEROR as uint32_t == 0 as uint32_t {
            if !errormsg.is_null() {
                *errormsg = gettext(&raw const e_zerocount as *const ::core::ffi::c_char);
            }
            return FAIL;
        }
        set_cmd_count(eap, n, validate);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn is_cmd_ni(mut cmdidx: cmdidx_T) -> bool {
    return !((cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
        && ((*cmdnames.ptr())[cmdidx as usize].cmd_func
            == Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())
            || (*cmdnames.ptr())[cmdidx as usize].cmd_func
                == Some(ex_script_ni as unsafe extern "C" fn(*mut exarg_T) -> ()));
}
unsafe extern "C" fn find_excmd_after_range(mut eap: *mut exarg_T) -> *mut ::core::ffi::c_char {
    let mut cmd: *mut ::core::ffi::c_char = (*eap).cmd;
    (*eap).cmd = skip_range((*eap).cmd, ::core::ptr::null_mut::<::core::ffi::c_int>());
    let mut p: *mut ::core::ffi::c_char =
        find_ex_command(eap, ::core::ptr::null_mut::<::core::ffi::c_int>());
    (*eap).cmd = cmd;
    return p;
}
unsafe extern "C" fn parse_bang(
    mut eap: *const exarg_T,
    mut p: *mut *mut ::core::ffi::c_char,
) -> bool {
    if **p as ::core::ffi::c_int == '!' as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_substitute as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_smagic as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_snomagic as ::core::ffi::c_int
    {
        *p = (*p).offset(1);
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_has_expr_args(mut cmdidx: cmdidx_T) -> bool {
    return cmdidx as ::core::ffi::c_int == CMD_execute as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_echo as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_echon as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_echomsg as ::core::ffi::c_int
        || cmdidx as ::core::ffi::c_int == CMD_echoerr as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn parse_cmdline(
    mut cmdline: *mut *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut errormsg: *mut *const ::core::ffi::c_char,
) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut after_modifier: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: bool = false_0 != 0;
    let save_ex_pressedreturn: bool = ex_pressedreturn.get();
    let save_cursor: pos_T = (*curwin.get()).w_cursor;
    save_last_search_pattern();
    memset(
        cmdinfo as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CmdParseInfo>(),
    );
    *eap = exarg {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: *cmdline,
        cmdlinep: cmdline,
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
        cookie: NULL_1,
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut orig_cmd: *mut ::core::ffi::c_char = (*eap).cmd;
    let mut result: ::core::ffi::c_int =
        parse_command_modifiers(eap, errormsg, &raw mut (*cmdinfo).cmdmod, false_0 != 0);
    after_modifier = (*eap).cmd;
    if !(result == FAIL && after_modifier == orig_cmd) {
        p = find_excmd_after_range(eap);
        if p.is_null() {
            *errormsg = gettext(
                (e_ambiguous_use_of_user_defined_command.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            );
        } else {
            set_cmd_addr_type(eap, p);
            if parse_cmd_address(eap, errormsg, true_0 != 0) != FAIL {
                (*eap).cmd = skip_colon_white((*eap).cmd, true_0 != 0);
                if *(*eap).cmd as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
                    if !(*(*eap).cmd as ::core::ffi::c_int == NUL
                        && (*eap).addr_count == 0 as ::core::ffi::c_int
                        && after_modifier == *cmdline)
                    {
                        if *(*eap).cmd as ::core::ffi::c_int == NUL
                            && (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                        {
                            (*eap).arg = (*eap).cmd;
                            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                                (*eap).argt = EX_RANGE as uint32_t;
                            } else {
                                (*eap).argt = 0 as uint32_t;
                                (*eap).addr_type = ADDR_NONE;
                            }
                            retval = true_0 != 0;
                        } else if (*eap).cmdidx as ::core::ffi::c_int
                            == CMD_SIZE as ::core::ffi::c_int
                        {
                            xstrlcpy(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                gettext(
                                    (e_not_an_editor_command.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                IOSIZE as size_t,
                            );
                            let mut cmdname: *mut ::core::ffi::c_char = if !after_modifier.is_null()
                            {
                                after_modifier
                            } else {
                                *cmdline
                            };
                            append_command(cmdname);
                            *errormsg = IObuff.ptr() as *mut ::core::ffi::c_char;
                        } else {
                            (*eap).forceit = parse_bang(eap, &raw mut p) as ::core::ffi::c_int;
                            if !(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
                                (*eap).argt = (*cmdnames.ptr())
                                    [(*eap).cmdidx as ::core::ffi::c_int as usize]
                                    .cmd_argt;
                            }
                            (*eap).arg = if (*eap).cmdidx as ::core::ffi::c_int
                                == CMD_bang as ::core::ffi::c_int
                            {
                                p
                            } else {
                                skipwhite(p)
                            };
                            if (*eap).cmdidx as ::core::ffi::c_int == CMD_read as ::core::ffi::c_int
                                && (*eap).forceit != 0
                            {
                                (*eap).forceit = false_0;
                            }
                            if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                separate_nextcmd(eap);
                            } else if cmd_has_expr_args((*eap).cmdidx) {
                                let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
                                while *arg as ::core::ffi::c_int != NUL
                                    && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
                                    && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                                {
                                    let mut start: *mut ::core::ffi::c_char = arg;
                                    (*emsg_skip.ptr()) += 1;
                                    skip_expr(&raw mut arg, ::core::ptr::null_mut::<evalarg_T>());
                                    (*emsg_skip.ptr()) -= 1;
                                    if arg == start {
                                        arg = arg.offset(1);
                                    }
                                }
                                if *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                                    || *arg as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                                {
                                    (*eap).nextcmd = check_nextcmd(arg);
                                    *arg = NUL as ::core::ffi::c_char;
                                }
                            }
                            if (*eap).argt & EX_BANG as uint32_t == 0 && (*eap).forceit != 0 {
                                *errormsg =
                                    gettext(&raw const e_nobang as *const ::core::ffi::c_char);
                            } else if (*eap).argt & EX_RANGE as uint32_t == 0
                                && (*eap).addr_count > 0 as ::core::ffi::c_int
                            {
                                *errormsg =
                                    gettext(&raw const e_norange as *const ::core::ffi::c_char);
                            } else {
                                if (*eap).argt & EX_DFLALL as uint32_t != 0
                                    && (*eap).addr_count == 0 as ::core::ffi::c_int
                                {
                                    set_cmd_dflall_range(eap);
                                }
                                parse_register(eap);
                                if parse_count(eap, errormsg, false_0 != 0) != FAIL {
                                    if !(*eap).nextcmd.is_null() {
                                        (*eap).nextcmd =
                                            skip_colon_white((*eap).nextcmd, true_0 != 0);
                                    }
                                    if (*eap).argt & EX_XFILE as uint32_t != 0 {
                                        (*cmdinfo).magic.file = true_0 != 0;
                                    }
                                    if (*eap).argt & EX_TRLBAR as uint32_t != 0 {
                                        (*cmdinfo).magic.bar = true_0 != 0;
                                    }
                                    retval = true_0 != 0;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !retval {
        undo_cmdmod(&raw mut (*cmdinfo).cmdmod);
    }
    ex_pressedreturn.set(save_ex_pressedreturn);
    (*curwin.get()).w_cursor = save_cursor;
    restore_last_search_pattern();
    return retval;
}
unsafe extern "C" fn shift_cmd_args(mut eap: *mut exarg_T) {
    '_c2rust_label: {
        if !(*eap).args.is_null() && (*eap).argc > 0 as size_t {
        } else {
            __assert_fail(
                b"eap->args != NULL && eap->argc > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1708 as ::core::ffi::c_uint,
                b"void shift_cmd_args(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut oldargs: *mut *mut ::core::ffi::c_char = (*eap).args;
    let mut oldarglens: *mut size_t = (*eap).arglens;
    (*eap).argc = (*eap).argc.wrapping_sub(1);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc(
            (*eap).argc,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
        )
    } else {
        NULL_1
    }) as *mut *mut ::core::ffi::c_char;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc((*eap).argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL_1
    }) as *mut size_t;
    let mut i: size_t = 0 as size_t;
    while i < (*eap).argc {
        *(*eap).args.offset(i as isize) = *oldargs.offset(i.wrapping_add(1 as size_t) as isize);
        *(*eap).arglens.offset(i as isize) =
            *oldarglens.offset(i.wrapping_add(1 as size_t) as isize);
        i = i.wrapping_add(1);
    }
    (*eap).arg = if (*eap).argc > 0 as size_t {
        *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
    } else {
        (*oldargs.offset(0 as ::core::ffi::c_int as isize))
            .offset(*oldarglens.offset(0 as ::core::ffi::c_int as isize) as isize)
    };
    xfree(oldargs as *mut ::core::ffi::c_void);
    xfree(oldarglens as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn execute_cmd0(
    mut retv: *mut ::core::ffi::c_int,
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
    mut preview: bool,
) -> ::core::ffi::c_int {
    if (*eap).argt & EX_XFILE as uint32_t != 0 {
        if expand_filename(eap, (*eap).cmdlinep, errormsg) == FAIL {
            return FAIL;
        }
    }
    if (*eap).argt & EX_BUFNAME as uint32_t != 0
        && *(*eap).arg as ::core::ffi::c_int != NUL
        && (*eap).addr_count == 0 as ::core::ffi::c_int
        && !(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
    {
        if (*eap).args.is_null() {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_bdelete as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_bwipeout as ::core::ffi::c_int
                || (*eap).cmdidx as ::core::ffi::c_int == CMD_bunload as ::core::ffi::c_int
            {
                p = skiptowhite_esc((*eap).arg);
            } else {
                p = (*eap).arg.offset(strlen((*eap).arg) as isize);
                while p > (*eap).arg
                    && ascii_iswhite(
                        *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                {
                    p = p.offset(-1);
                }
            }
            (*eap).line2 = buflist_findpat(
                (*eap).arg,
                p,
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as ::core::ffi::c_int;
            (*eap).arg = skipwhite(p);
        } else {
            (*eap).line2 = buflist_findpat(
                *(*eap).args.offset(0 as ::core::ffi::c_int as isize),
                (*(*eap).args.offset(0 as ::core::ffi::c_int as isize))
                    .offset(*(*eap).arglens.offset(0 as ::core::ffi::c_int as isize) as isize),
                (*eap).argt & EX_BUFUNL as uint32_t != 0 as uint32_t,
                false_0 != 0,
                false_0 != 0,
            ) as linenr_T;
            (*eap).addr_count = 1 as ::core::ffi::c_int;
            shift_cmd_args(eap);
        }
        if (*eap).line2 < 0 as linenr_T {
            return FAIL;
        }
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_try as ::core::ffi::c_int
        && (*cmdmod.ptr()).cmod_did_esilent > 0 as ::core::ffi::c_int
    {
        (*emsg_silent.ptr()) -= (*cmdmod.ptr()).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as ::core::ffi::c_int {
            emsg_silent.get()
        } else {
            0 as ::core::ffi::c_int
        });
        (*cmdmod.ptr()).cmod_did_esilent = 0 as ::core::ffi::c_int;
    }
    if ((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        *retv = do_ucmd(eap, preview);
    } else {
        (*eap).errmsg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if preview {
            *retv = (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_preview_func
                .expect("non-null function pointer")(
                eap,
                cmdpreview_get_ns(),
                cmdpreview_get_bufnr(),
            );
        } else {
            (*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_func
                .expect("non-null function pointer")(eap);
        }
        if !(*eap).errmsg.is_null() {
            *errormsg = (*eap).errmsg;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn execute_cmd(
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut preview: bool,
) -> ::core::ffi::c_int {
    let mut cstack: cstack_T = cstack_T {
        cs_flags: [0; 50],
        cs_pending: [0; 50],
        cs_pend: C2Rust_Unnamed_34 {
            csp_rv: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
        },
        cs_forinfo: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
        cs_line: [0; 50],
        cs_idx: 0,
        cs_looplevel: 0,
        cs_trylevel: 0,
        cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
        cs_lflags: 0,
    };
    let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if do_cmdline_start() == FAIL {
        emsg(gettext(
            &raw const e_command_too_recursive as *const ::core::ffi::c_char,
        ));
        return retv;
    }
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    cmdmod.set((*cmdinfo).cmdmod);
    apply_cmdmod(cmdmod.ptr());
    '_end: {
        if (*curbuf.get()).b_p_ma == 0
            && (*eap).argt & EX_MODIFY as uint32_t != 0
            && !(!(*curbuf.get()).terminal.is_null()
                && ((*eap).cmdidx as ::core::ffi::c_int == CMD_put as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_iput as ::core::ffi::c_int))
        {
            errormsg = gettext(&raw const e_modifiable as *const ::core::ffi::c_char);
        } else {
            if !(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
                if cmdwin_type.get() != 0 as ::core::ffi::c_int
                    && (*eap).argt & EX_CMDWIN as uint32_t == 0
                {
                    errormsg = gettext(&raw const e_cmdwin as *const ::core::ffi::c_char);
                    break '_end;
                } else if text_locked() as ::core::ffi::c_int != 0
                    && (*eap).argt & EX_LOCK_OK as uint32_t == 0
                {
                    errormsg = gettext(get_text_locked_msg());
                    break '_end;
                }
            }
            if !((*eap).argt & EX_CMDWIN as uint32_t == 0
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_checktime as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_edit as ::core::ffi::c_int
                && !((*eap).cmdidx as ::core::ffi::c_int == CMD_file as ::core::ffi::c_int
                    && *(*eap).arg as ::core::ffi::c_int == NUL)
                && !(((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
                && curbuf_locked() as ::core::ffi::c_int != 0)
            {
                correct_range(eap);
                if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                    && (*eap).addr_count > 0 as ::core::ffi::c_int
                {
                    errormsg = ex_range_without_command(eap);
                } else {
                    if ((*eap).argt & EX_WHOLEFOLD as uint32_t != 0
                        || (*eap).addr_count >= 2 as ::core::ffi::c_int)
                        && global_busy.get() == 0
                        && (*eap).addr_type as ::core::ffi::c_uint
                            == ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        hasFolding(
                            curwin.get(),
                            (*eap).line1,
                            &raw mut (*eap).line1,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        hasFolding(
                            curwin.get(),
                            (*eap).line2,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut (*eap).line2,
                        );
                    }
                    if parse_count(eap, &raw mut errormsg, true_0 != 0) != FAIL {
                        cstack = cstack_T {
                            cs_flags: [0; 50],
                            cs_pending: [0; 50],
                            cs_pend: C2Rust_Unnamed_34 {
                                csp_rv: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
                            },
                            cs_forinfo: [::core::ptr::null_mut::<::core::ffi::c_void>(); 50],
                            cs_line: [0; 50],
                            cs_idx: -1 as ::core::ffi::c_int,
                            cs_looplevel: 0,
                            cs_trylevel: 0,
                            cs_emsg_silent_list: ::core::ptr::null_mut::<eslist_T>(),
                            cs_lflags: 0,
                        };
                        (*eap).cstack = &raw mut cstack;
                        execute_cmd0(&raw mut retv, eap, &raw mut errormsg, preview);
                    }
                }
            }
        }
    }
    if !errormsg.is_null() && *errormsg as ::core::ffi::c_int != NUL {
        emsg(errormsg);
    }
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    do_cmdline_end();
    return retv;
}
unsafe extern "C" fn profile_cmd(
    mut eap: *const exarg_T,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) {
    if do_profiling.get() == PROF_YES
        && ((*eap).skip == 0
            || (*cstack).cs_idx == 0 as ::core::ffi::c_int
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    != 0)
    {
        let mut skip: bool = did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_catch as ::core::ffi::c_int {
            skip = !skip
                && !((*cstack).cs_idx >= 0 as ::core::ffi::c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                        & CSF_THROWN as ::core::ffi::c_int
                        != 0
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                        & CSF_CAUGHT as ::core::ffi::c_int
                        == 0);
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_else as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_elseif as ::core::ffi::c_int
        {
            skip = skip as ::core::ffi::c_int != 0
                || !((*cstack).cs_idx >= 0 as ::core::ffi::c_int
                    && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                        & (CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int)
                        == 0);
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_finally as ::core::ffi::c_int {
            skip = false_0 != 0;
        } else if (*eap).cmdidx as ::core::ffi::c_int != CMD_endif as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_endfor as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_endtry as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_endwhile as ::core::ffi::c_int
        {
            skip = (*eap).skip != 0;
        }
        if !skip {
            if getline_equal(
                fgetline,
                cookie,
                Some(
                    get_func_line
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) {
                func_line_exec(getline_cookie(fgetline, cookie));
            } else if getline_equal(
                fgetline,
                cookie,
                Some(
                    getsourceline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) {
                script_line_exec();
            }
        }
    }
}
unsafe extern "C" fn skip_cmd(mut eap: *const exarg_T) -> bool {
    if (*eap).skip != 0 {
        match (*eap).cmdidx as ::core::ffi::c_int {
            525 | 147 | 167 | 145 | 187 | 141 | 140 | 143 | 488 | 54 | 159 | 146 | 168 | 3
            | 550 | 26 | 31 | 38 | 53 | 97 | 99 | 115 | 126 | 127 | 131 | 132 | 135 | 136 | 138
            | 139 | 149 | 151 | 157 | 176 | 181 | 183 | 188 | 189 | 198 | 199 | 209 | 207 | 206
            | 208 | 230 | 231 | 255 | 256 | 264 | 278 | 288 | 298 | 302 | 323 | 334 | 346 | 349
            | 351 | 355 | 353 | 371 | 374 | 378 | 407 | 410 | 415 | 382 | 444 | 453 | 468 | 473
            | 555 | 484 | 498 | 499 | 506 | 507 | 527 => {}
            _ => return true_0 != 0,
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn do_one_cmd(
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut cstack: *mut cstack_T,
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut after_modifier: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ni: ::core::ffi::c_int = 0;
    let mut retv: ::core::ffi::c_int = 0;
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let save_reg_executing: ::core::ffi::c_int = reg_executing.get();
    let save_pending_end_reg_executing: bool = pending_end_reg_executing.get();
    let mut ea: exarg_T = exarg {
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
    (*ex_nesting_level.ptr()) += 1;
    if quitmore.get() != 0
        && !getline_equal(
            fgetline,
            cookie,
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        )
        && !getline_equal(
            fgetline,
            cookie,
            Some(
                getnextac
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
        )
    {
        (*quitmore.ptr()) -= 1;
    }
    let mut save_cmdmod: cmdmod_T = cmdmod.get();
    '_doend: {
        if !(*(*cmdlinep).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '#' as ::core::ffi::c_int
            && *(*cmdlinep).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '!' as ::core::ffi::c_int)
        {
            ea.cmd = *cmdlinep;
            ea.cmdlinep = cmdlinep;
            ea.ea_getline = fgetline;
            ea.cookie = cookie;
            ea.cstack = cstack;
            if parse_command_modifiers(&raw mut ea, &raw mut errormsg, cmdmod.ptr(), false_0 != 0)
                != FAIL
            {
                apply_cmdmod(cmdmod.ptr());
                after_modifier = ea.cmd;
                ea.skip = (did_emsg.get() != 0
                    || got_int.get() as ::core::ffi::c_int != 0
                    || did_throw.get() as ::core::ffi::c_int != 0
                    || (*cstack).cs_idx >= 0 as ::core::ffi::c_int
                        && (*cstack).cs_flags[(*cstack).cs_idx as usize]
                            & CSF_ACTIVE as ::core::ffi::c_int
                            == 0) as ::core::ffi::c_int;
                p = find_excmd_after_range(&raw mut ea);
                profile_cmd(&raw mut ea, cstack, fgetline, cookie);
                if !exiting.get() {
                    dbg_check_breakpoint(&raw mut ea);
                }
                if ea.skip == 0 && got_int.get() as ::core::ffi::c_int != 0 {
                    ea.skip = true_0;
                    do_intthrow(cstack);
                }
                set_cmd_addr_type(&raw mut ea, p);
                if parse_cmd_address(&raw mut ea, &raw mut errormsg, false_0 != 0) != FAIL {
                    ea.cmd = skip_colon_white(ea.cmd, true_0 != 0);
                    if *ea.cmd as ::core::ffi::c_int == NUL
                        || *ea.cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                        || {
                            ea.nextcmd = check_nextcmd(ea.cmd);
                            !ea.nextcmd.is_null()
                        }
                    {
                        if ea.skip == 0 {
                            '_c2rust_label: {
                                if errormsg.is_null() {
                                } else {
                                    __assert_fail(
                                        b"errormsg == NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/ex_docmd.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2156 as ::core::ffi::c_uint,
                                        b"char *do_one_cmd(char **, int, cstack_T *, LineGetter, void *)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            errormsg = ex_range_without_command(&raw mut ea);
                        }
                    } else {
                        if !p.is_null()
                            && ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                            && ea.skip == 0
                            && (*ea.cmd as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                && *ea.cmd as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
                            && has_event(EVENT_CMDUNDEFINED) as ::core::ffi::c_int != 0
                        {
                            let mut cmdname: *mut ::core::ffi::c_char = ea.cmd;
                            while *cmdname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                && *cmdname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                || *cmdname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                                    && *cmdname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                                || ascii_isdigit(*cmdname as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                cmdname = cmdname.offset(1);
                            }
                            cmdname = xmemdupz(
                                ea.cmd as *const ::core::ffi::c_void,
                                cmdname.offset_from(ea.cmd) as size_t,
                            ) as *mut ::core::ffi::c_char;
                            let mut ret: ::core::ffi::c_int = apply_autocmds(
                                EVENT_CMDUNDEFINED,
                                cmdname,
                                cmdname,
                                true_0 != 0,
                                ::core::ptr::null_mut::<buf_T>(),
                            )
                                as ::core::ffi::c_int;
                            xfree(cmdname as *mut ::core::ffi::c_void);
                            p = if ret != 0 && !aborting() {
                                find_ex_command(
                                    &raw mut ea,
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                )
                            } else {
                                ea.cmd
                            };
                        }
                        if p.is_null() {
                            if ea.skip == 0 {
                                errormsg = gettext(
                                    (e_ambiguous_use_of_user_defined_command.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        } else if ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                        {
                            if ea.skip == 0 {
                                xstrlcpy(
                                    IObuff.ptr() as *mut ::core::ffi::c_char,
                                    gettext(
                                        (e_not_an_editor_command.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    IOSIZE as size_t,
                                );
                                let mut cmdname_0: *mut ::core::ffi::c_char =
                                    if !after_modifier.is_null() {
                                        after_modifier
                                    } else {
                                        *cmdlinep
                                    };
                                if flags & DOCMD_VERBOSE as ::core::ffi::c_int == 0 {
                                    append_command(cmdname_0);
                                }
                                errormsg = IObuff.ptr() as *mut ::core::ffi::c_char;
                                did_emsg_syntax.set(true_0 != 0);
                                verify_command(cmdname_0);
                            }
                        } else {
                            ni = is_cmd_ni(ea.cmdidx) as ::core::ffi::c_int;
                            ea.forceit = parse_bang(&raw mut ea, &raw mut p) as ::core::ffi::c_int;
                            if !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int) {
                                ea.argt = (*cmdnames.ptr())
                                    [ea.cmdidx as ::core::ffi::c_int as usize]
                                    .cmd_argt;
                            }
                            if ea.skip == 0 {
                                if sandbox.get() != 0 as ::core::ffi::c_int
                                    && ea.argt & EX_SBOXOK as uint32_t == 0
                                {
                                    errormsg =
                                        gettext(&raw const e_sandbox as *const ::core::ffi::c_char);
                                    break '_doend;
                                } else if (*curbuf.get()).b_p_ma == 0
                                    && ea.argt & EX_MODIFY as uint32_t != 0
                                    && !(!(*curbuf.get()).terminal.is_null()
                                        && (ea.cmdidx as ::core::ffi::c_int
                                            == CMD_put as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_iput as ::core::ffi::c_int))
                                {
                                    errormsg = gettext(
                                        &raw const e_modifiable as *const ::core::ffi::c_char,
                                    );
                                    break '_doend;
                                } else {
                                    if !((ea.cmdidx as ::core::ffi::c_int)
                                        < 0 as ::core::ffi::c_int)
                                    {
                                        if cmdwin_type.get() != 0 as ::core::ffi::c_int
                                            && ea.argt & EX_CMDWIN as uint32_t == 0
                                        {
                                            errormsg = gettext(
                                                &raw const e_cmdwin as *const ::core::ffi::c_char,
                                            );
                                            break '_doend;
                                        } else if text_locked() as ::core::ffi::c_int != 0
                                            && ea.argt & EX_LOCK_OK as uint32_t == 0
                                        {
                                            errormsg = gettext(get_text_locked_msg());
                                            break '_doend;
                                        }
                                    }
                                    if ea.argt & EX_CMDWIN as uint32_t == 0
                                        && ea.cmdidx as ::core::ffi::c_int
                                            != CMD_checktime as ::core::ffi::c_int
                                        && ea.cmdidx as ::core::ffi::c_int
                                            != CMD_edit as ::core::ffi::c_int
                                        && ea.cmdidx as ::core::ffi::c_int
                                            != CMD_file as ::core::ffi::c_int
                                        && !((ea.cmdidx as ::core::ffi::c_int)
                                            < 0 as ::core::ffi::c_int)
                                        && curbuf_locked() as ::core::ffi::c_int != 0
                                    {
                                        break '_doend;
                                    } else if ni == 0
                                        && ea.argt & EX_RANGE as uint32_t == 0
                                        && ea.addr_count > 0 as ::core::ffi::c_int
                                    {
                                        errormsg = gettext(
                                            &raw const e_norange as *const ::core::ffi::c_char,
                                        );
                                        break '_doend;
                                    }
                                }
                            }
                            if ni == 0 && ea.argt & EX_BANG as uint32_t == 0 && ea.forceit != 0 {
                                errormsg =
                                    gettext(&raw const e_nobang as *const ::core::ffi::c_char);
                            } else {
                                if ea.skip == 0 && ni == 0 && ea.argt & EX_RANGE as uint32_t != 0 {
                                    if global_busy.get() == 0 && ea.line1 > ea.line2 {
                                        if msg_silent.get() == 0 as ::core::ffi::c_int {
                                            if flags & DOCMD_VERBOSE as ::core::ffi::c_int != 0
                                                || exmode_active.get() as ::core::ffi::c_int != 0
                                            {
                                                errormsg = gettext(
                                                    b"E493: Backwards range given\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                                break '_doend;
                                            } else if ask_yesno(gettext(
                                                b"Backwards range given, OK to swap\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            )) != 'y' as ::core::ffi::c_int
                                            {
                                                break '_doend;
                                            }
                                        }
                                        let mut lnum: linenr_T = ea.line1;
                                        ea.line1 = ea.line2;
                                        ea.line2 = lnum;
                                    }
                                    errormsg = invalid_range(&raw mut ea);
                                    if !errormsg.is_null() {
                                        break '_doend;
                                    }
                                }
                                if ea.addr_type as ::core::ffi::c_uint
                                    == ADDR_OTHER as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && ea.addr_count == 0 as ::core::ffi::c_int
                                {
                                    ea.line2 = 1 as ::core::ffi::c_int as linenr_T;
                                }
                                correct_range(&raw mut ea);
                                if (ea.argt & EX_WHOLEFOLD as uint32_t != 0
                                    || ea.addr_count >= 2 as ::core::ffi::c_int)
                                    && global_busy.get() == 0
                                    && ea.addr_type as ::core::ffi::c_uint
                                        == ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    hasFolding(
                                        curwin.get(),
                                        ea.line1,
                                        &raw mut ea.line1,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                    );
                                    hasFolding(
                                        curwin.get(),
                                        ea.line2,
                                        ::core::ptr::null_mut::<linenr_T>(),
                                        &raw mut ea.line2,
                                    );
                                }
                                p = replace_makeprg(&raw mut ea, p, cmdlinep);
                                if !p.is_null() {
                                    ea.arg = if ea.cmdidx as ::core::ffi::c_int
                                        == CMD_bang as ::core::ffi::c_int
                                    {
                                        p
                                    } else {
                                        skipwhite(p)
                                    };
                                    if !(ea.cmdidx as ::core::ffi::c_int
                                        == CMD_file as ::core::ffi::c_int
                                        && *ea.arg as ::core::ffi::c_int != NUL
                                        && curbuf_locked() as ::core::ffi::c_int != 0)
                                    {
                                        's_449: {
                                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                                loop {
                                                    if !(*ea
                                                        .arg
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '+' as ::core::ffi::c_int
                                                        && *ea.arg.offset(
                                                            1 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == '+' as ::core::ffi::c_int)
                                                    {
                                                        break 's_449;
                                                    }
                                                    if !(getargopt(&raw mut ea) == FAIL && ni == 0)
                                                    {
                                                        continue;
                                                    }
                                                    errormsg = gettext(
                                                        &raw const e_invarg
                                                            as *const ::core::ffi::c_char,
                                                    );
                                                    break '_doend;
                                                }
                                            }
                                        }
                                        if ea.cmdidx as ::core::ffi::c_int
                                            == CMD_write as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_update as ::core::ffi::c_int
                                        {
                                            if *ea.arg as ::core::ffi::c_int
                                                == '>' as ::core::ffi::c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                if *ea.arg as ::core::ffi::c_int
                                                    != '>' as ::core::ffi::c_int
                                                {
                                                    errormsg =
                                                        gettext(b"E494: Use w or w>>\0".as_ptr()
                                                            as *const ::core::ffi::c_char);
                                                    break '_doend;
                                                } else {
                                                    ea.arg =
                                                        skipwhite(ea.arg.offset(
                                                            1 as ::core::ffi::c_int as isize,
                                                        ));
                                                    ea.append = true_0;
                                                }
                                            } else if *ea.arg as ::core::ffi::c_int
                                                == '!' as ::core::ffi::c_int
                                                && ea.cmdidx as ::core::ffi::c_int
                                                    == CMD_write as ::core::ffi::c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as ::core::ffi::c_int
                                            == CMD_read as ::core::ffi::c_int
                                        {
                                            if ea.forceit != 0 {
                                                ea.usefilter = true_0;
                                                ea.forceit = false_0;
                                            } else if *ea.arg as ::core::ffi::c_int
                                                == '!' as ::core::ffi::c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                ea.usefilter = true_0;
                                            }
                                        } else if ea.cmdidx as ::core::ffi::c_int
                                            == CMD_lshift as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_rshift as ::core::ffi::c_int
                                        {
                                            ea.amount = 1 as ::core::ffi::c_int;
                                            while *ea.arg as ::core::ffi::c_int
                                                == *ea.cmd as ::core::ffi::c_int
                                            {
                                                ea.arg = ea.arg.offset(1);
                                                ea.amount += 1;
                                            }
                                            ea.arg = skipwhite(ea.arg);
                                        }
                                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                                        }
                                        if ea.argt & EX_TRLBAR as uint32_t != 0 && ea.usefilter == 0
                                        {
                                            separate_nextcmd(&raw mut ea);
                                        } else if ea.cmdidx as ::core::ffi::c_int
                                            == CMD_bang as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_terminal as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_global as ::core::ffi::c_int
                                            || ea.cmdidx as ::core::ffi::c_int
                                                == CMD_vglobal as ::core::ffi::c_int
                                            || ea.usefilter != 0
                                        {
                                            let mut s: *mut ::core::ffi::c_char = ea.arg;
                                            while *s != 0 {
                                                if *s as ::core::ffi::c_int
                                                    == '\\' as ::core::ffi::c_int
                                                    && *s.offset(1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == '\n' as ::core::ffi::c_int
                                                {
                                                    memmove(
                                                        s as *mut ::core::ffi::c_void,
                                                        s.offset(1 as ::core::ffi::c_int as isize)
                                                            as *const ::core::ffi::c_void,
                                                        strlen(s.offset(
                                                            1 as ::core::ffi::c_int as isize,
                                                        ))
                                                        .wrapping_add(1 as size_t),
                                                    );
                                                } else if *s as ::core::ffi::c_int
                                                    == '\n' as ::core::ffi::c_int
                                                {
                                                    ea.nextcmd =
                                                        s.offset(1 as ::core::ffi::c_int as isize);
                                                    *s = NUL as ::core::ffi::c_char;
                                                    break;
                                                }
                                                s = s.offset(1);
                                            }
                                        }
                                        if ea.argt & EX_DFLALL as uint32_t != 0
                                            && ea.addr_count == 0 as ::core::ffi::c_int
                                        {
                                            set_cmd_dflall_range(&raw mut ea);
                                        }
                                        parse_register(&raw mut ea);
                                        if parse_count(&raw mut ea, &raw mut errormsg, true_0 != 0)
                                            != FAIL
                                        {
                                            if ea.argt & EX_FLAGS as uint32_t != 0 {
                                                get_flags(&raw mut ea);
                                            }
                                            if ni == 0
                                                && ea.argt & EX_EXTRA as uint32_t == 0
                                                && *ea.arg as ::core::ffi::c_int != NUL
                                                && *ea.arg as ::core::ffi::c_int
                                                    != '"' as ::core::ffi::c_int
                                                && (*ea.arg as ::core::ffi::c_int
                                                    != '|' as ::core::ffi::c_int
                                                    || ea.argt & EX_TRLBAR as uint32_t
                                                        == 0 as uint32_t)
                                            {
                                                errormsg = ex_errmsg(
                                                    &raw const e_trailing_arg
                                                        as *const ::core::ffi::c_char,
                                                    ea.arg,
                                                );
                                            } else if ni == 0
                                                && ea.argt & EX_NEEDARG as uint32_t != 0
                                                && *ea.arg as ::core::ffi::c_int == NUL
                                            {
                                                errormsg = gettext(
                                                    &raw const e_argreq
                                                        as *const ::core::ffi::c_char,
                                                );
                                            } else if !skip_cmd(&raw mut ea) {
                                                retv = 0 as ::core::ffi::c_int;
                                                if execute_cmd0(
                                                    &raw mut retv,
                                                    &raw mut ea,
                                                    &raw mut errormsg,
                                                    false_0 != 0,
                                                ) != FAIL
                                                {
                                                    if need_rethrow.get() {
                                                        do_throw(cstack);
                                                    } else if check_cstack.get() {
                                                        if source_finished(fgetline, cookie) {
                                                            do_finish(&raw mut ea, true_0 != 0);
                                                        } else if getline_equal(
                                                            fgetline,
                                                            cookie,
                                                            Some(
                                                                get_func_line
                                                                    as unsafe extern "C" fn(
                                                                        ::core::ffi::c_int,
                                                                        *mut ::core::ffi::c_void,
                                                                        ::core::ffi::c_int,
                                                                        bool,
                                                                    ) -> *mut ::core::ffi::c_char,
                                                            ),
                                                        ) as ::core::ffi::c_int != 0 && current_func_returned() != 0
                                                        {
                                                            do_return(&raw mut ea, true_0 != 0, false_0 != 0, NULL_1);
                                                        }
                                                    }
                                                    check_cstack.set(false_0 != 0);
                                                    need_rethrow.set(check_cstack.get());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if (*curwin.get()).w_cursor.lnum == 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if !errormsg.is_null() && *errormsg as ::core::ffi::c_int != NUL && did_emsg.get() == 0 {
        if flags & DOCMD_VERBOSE as ::core::ffi::c_int != 0 {
            if errormsg != IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_char {
                xstrlcpy(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    errormsg,
                    IOSIZE as size_t,
                );
                errormsg = IObuff.ptr() as *mut ::core::ffi::c_char;
            }
            append_command(*ea.cmdlinep);
        }
        emsg(errormsg);
    }
    do_errthrow(
        cstack,
        if ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
            && !((ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int)
        {
            (*cmdnames.ptr())[ea.cmdidx as ::core::ffi::c_int as usize].cmd_name
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        },
    );
    undo_cmdmod(cmdmod.ptr());
    cmdmod.set(save_cmdmod);
    reg_executing.set(save_reg_executing);
    pending_end_reg_executing.set(save_pending_end_reg_executing);
    if !ea.nextcmd.is_null() && *ea.nextcmd as ::core::ffi::c_int == NUL {
        ea.nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ex_nesting_level.ptr()) -= 1;
    xfree(ea.cmdline_tofree as *mut ::core::ffi::c_void);
    return ea.nextcmd;
}
static ex_error_buf: GlobalCell<[::core::ffi::c_char; 480]> = GlobalCell::new([0; 480]);
#[no_mangle]
pub unsafe extern "C" fn ex_errmsg(
    msg_0: *const ::core::ffi::c_char,
    arg: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    vim_snprintf(
        ex_error_buf.ptr() as *mut ::core::ffi::c_char,
        MSG_BUF_LEN as size_t,
        gettext(msg_0),
        arg,
    );
    return ex_error_buf.ptr() as *mut ::core::ffi::c_char;
}
static exmode_plus: GlobalCell<[::core::ffi::c_char; 2]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"+\0")
});
unsafe extern "C" fn ex_range_without_command(mut eap: *mut exarg_T) -> *mut ::core::ffi::c_char {
    let mut errormsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *(*eap).cmd as ::core::ffi::c_int == '|' as ::core::ffi::c_int
        || exmode_active.get() as ::core::ffi::c_int != 0
            && (*eap).cmd
                != (exmode_plus.ptr() as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize)
    {
        (*eap).cmdidx = CMD_print;
        (*eap).argt = (EX_RANGE | EX_COUNT | EX_TRLBAR) as uint32_t;
        errormsg = invalid_range(eap);
        if errormsg.is_null() {
            correct_range(eap);
            ex_print(eap);
        }
    } else if (*eap).addr_count != 0 as ::core::ffi::c_int {
        (*eap).line2 = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
            (*eap).line2
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        if (*eap).line2 < 0 as linenr_T {
            errormsg = gettext(&raw const e_invrange as *const ::core::ffi::c_char);
        } else {
            if (*eap).line2 == 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
            } else {
                (*curwin.get()).w_cursor.lnum = (*eap).line2;
            }
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
    }
    return errormsg;
}
#[no_mangle]
pub unsafe extern "C" fn parse_command_modifiers(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
    mut cmod: *mut cmdmod_T,
    mut skip_only: bool,
) -> ::core::ffi::c_int {
    let mut orig_cmd: *mut ::core::ffi::c_char = (*eap).cmd;
    let mut cmd_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_plus_cmd: bool = false_0 != 0;
    let mut has_visual_range: bool = false_0 != 0;
    memset(
        cmod as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdmod_T>(),
    );
    if strncmp(
        (*eap).cmd,
        b"'<,'>\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        let mut p: *const ::core::ffi::c_char =
            skipwhite((*eap).cmd.offset(5 as ::core::ffi::c_int as isize));
        if *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        {
            (*eap).cmd = (*eap).cmd.offset(5 as ::core::ffi::c_int as isize);
            cmd_start = (*eap).cmd;
            has_visual_range = true_0 != 0;
        }
    }
    loop {
        while *(*eap).cmd as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            || *(*eap).cmd as ::core::ffi::c_int == '\t' as ::core::ffi::c_int
            || *(*eap).cmd as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        {
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if *(*eap).cmd as ::core::ffi::c_int == NUL
            && exmode_active.get() as ::core::ffi::c_int != 0
            && getline_equal(
                (*eap).ea_getline,
                (*eap).cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                            bool,
                        )
                            -> *mut ::core::ffi::c_char,
                ),
            ) as ::core::ffi::c_int
                != 0
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
        {
            (*eap).cmd = exmode_plus.ptr() as *mut ::core::ffi::c_char;
            use_plus_cmd = true_0 != 0;
            if !skip_only {
                ex_pressedreturn.set(true_0 != 0);
            }
            break;
        } else {
            if *(*eap).cmd as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
                (*eap).nextcmd = vim_strchr((*eap).cmd, '\n' as ::core::ffi::c_int);
                if !(*eap).nextcmd.is_null() {
                    (*eap).nextcmd = (*eap).nextcmd.offset(1);
                }
                return FAIL;
            }
            if *(*eap).cmd as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                (*eap).nextcmd = (*eap).cmd.offset(1 as ::core::ffi::c_int as isize);
                return FAIL;
            }
            if *(*eap).cmd as ::core::ffi::c_int == NUL {
                if !skip_only {
                    ex_pressedreturn.set(true_0 != 0);
                }
                return FAIL;
            }
            let mut p_0: *mut ::core::ffi::c_char =
                skip_range((*eap).cmd, ::core::ptr::null_mut::<::core::ffi::c_int>());
            match *p_0 as ::core::ffi::c_int {
                97 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_ABOVE as ::core::ffi::c_int;
                }
                98 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_BELOW as ::core::ffi::c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"browse\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_BROWSE as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"botright\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_BOT as ::core::ffi::c_int;
                    }
                }
                99 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"confirm\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as ::core::ffi::c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_CONFIRM as ::core::ffi::c_int;
                }
                107 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPMARKS as ::core::ffi::c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepalt\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPALT as ::core::ffi::c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_KEEPJUMPS as ::core::ffi::c_int;
                    }
                }
                102 => {
                    let mut reg_pat: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if !checkforcmd(
                        &raw mut p_0,
                        b"filter\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as ::core::ffi::c_int,
                    ) || *p_0 as ::core::ffi::c_int == NUL
                        || ends_excmd(*p_0 as ::core::ffi::c_int) != 0
                    {
                        break;
                    }
                    if *p_0 as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                        (*cmod).cmod_filter_force = true_0 != 0;
                        p_0 = skipwhite(p_0.offset(1 as ::core::ffi::c_int as isize));
                        if *p_0 as ::core::ffi::c_int == NUL
                            || ends_excmd(*p_0 as ::core::ffi::c_int) != 0
                        {
                            break;
                        }
                    }
                    if skip_only {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                    } else {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            &raw mut reg_pat,
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        );
                    }
                    if p_0.is_null() || *p_0 as ::core::ffi::c_int == NUL {
                        break;
                    }
                    if !skip_only {
                        (*cmod).cmod_filter_pat = xstrdup(reg_pat);
                        (*cmod).cmod_filter_regmatch.regprog = vim_regcomp(reg_pat, RE_MAGIC);
                        if (*cmod).cmod_filter_regmatch.regprog.is_null() {
                            break;
                        }
                    }
                    (*eap).cmd = p_0;
                }
                104 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_HOR as ::core::ffi::c_int;
                    } else {
                        if p_0 != (*eap).cmd
                            || !checkforcmd(
                                &raw mut p_0,
                                b"hide\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as ::core::ffi::c_int,
                            )
                            || *p_0 as ::core::ffi::c_int == NUL
                            || ends_excmd(*p_0 as ::core::ffi::c_int) != 0
                        {
                            break;
                        }
                        (*eap).cmd = p_0;
                        (*cmod).cmod_flags |= CMOD_HIDE as ::core::ffi::c_int;
                    }
                }
                108 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"leftabove\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_ABOVE as ::core::ffi::c_int;
                    }
                }
                110 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_NOAUTOCMD as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char,
                            3 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_NOSWAPFILE as ::core::ffi::c_int;
                    }
                }
                114 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as ::core::ffi::c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_BELOW as ::core::ffi::c_int;
                }
                115 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"sandbox\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_SANDBOX as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"silent\0".as_ptr() as *const ::core::ffi::c_char,
                            3 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_SILENT as ::core::ffi::c_int;
                        if *(*eap).cmd as ::core::ffi::c_int == '!' as ::core::ffi::c_int
                            && !ascii_iswhite(*(*eap).cmd.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                        {
                            (*eap).cmd =
                                skipwhite((*eap).cmd.offset(1 as ::core::ffi::c_int as isize));
                            (*cmod).cmod_flags |= CMOD_ERRSILENT as ::core::ffi::c_int;
                        }
                    }
                }
                116 => {
                    if checkforcmd(
                        &raw mut p_0,
                        b"tab\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        if !skip_only {
                            let mut tabnr: ::core::ffi::c_int = get_address(
                                eap,
                                &raw mut (*eap).cmd,
                                ADDR_TABS,
                                (*eap).skip != 0,
                                skip_only,
                                false_0,
                                1 as ::core::ffi::c_int,
                                errormsg,
                            )
                                as ::core::ffi::c_int;
                            if (*eap).cmd.is_null() {
                                return false_0;
                            }
                            if tabnr == MAXLNUM as ::core::ffi::c_int {
                                (*cmod).cmod_tab =
                                    tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                            } else {
                                if tabnr < 0 as ::core::ffi::c_int
                                    || tabnr > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                {
                                    *errormsg = gettext(
                                        &raw const e_invrange as *const ::core::ffi::c_char,
                                    );
                                    return false_0;
                                }
                                (*cmod).cmod_tab = tabnr + 1 as ::core::ffi::c_int;
                            }
                        }
                        (*eap).cmd = p_0;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"topleft\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_TOP as ::core::ffi::c_int;
                    }
                }
                117 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"unsilent\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as ::core::ffi::c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_UNSILENT as ::core::ffi::c_int;
                }
                118 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as ::core::ffi::c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_VERT as ::core::ffi::c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut p_0,
                            b"verbose\0".as_ptr() as *const ::core::ffi::c_char,
                            4 as ::core::ffi::c_int,
                        ) {
                            break;
                        }
                        if ascii_isdigit(*(*eap).cmd as ::core::ffi::c_int) {
                            (*cmod).cmod_verbose = atoi((*eap).cmd) + 1 as ::core::ffi::c_int;
                        } else {
                            (*cmod).cmod_verbose = 2 as ::core::ffi::c_int;
                        }
                        (*eap).cmd = p_0;
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }
    if has_visual_range {
        if (*eap).cmd > cmd_start {
            if use_plus_cmd {
                let mut len: size_t = strlen(cmd_start);
                memmove(
                    orig_cmd as *mut ::core::ffi::c_void,
                    cmd_start as *const ::core::ffi::c_void,
                    len,
                );
                xmemcpyz(
                    orig_cmd.offset(len as isize) as *mut ::core::ffi::c_void,
                    b" *+\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                );
            } else {
                memmove(
                    cmd_start.offset(-(5 as ::core::ffi::c_int as isize))
                        as *mut ::core::ffi::c_void,
                    cmd_start as *const ::core::ffi::c_void,
                    (*eap).cmd.offset_from(cmd_start) as size_t,
                );
                (*eap).cmd = (*eap).cmd.offset(-(5 as ::core::ffi::c_int as isize));
                memmove(
                    (*eap).cmd.offset(-(1 as ::core::ffi::c_int as isize))
                        as *mut ::core::ffi::c_void,
                    b":'<,'>\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    6 as size_t,
                );
            }
        } else if use_plus_cmd {
            (*eap).cmd =
                b"'<,'>+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            (*eap).cmd = orig_cmd;
        }
    } else if use_plus_cmd {
        (*eap).cmd = exmode_plus.ptr() as *mut ::core::ffi::c_char;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn apply_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0 && (*cmod).cmod_did_sandbox == 0
    {
        (*sandbox.ptr()) += 1;
        (*cmod).cmod_did_sandbox = true_0;
    }
    if (*cmod).cmod_verbose > 0 as ::core::ffi::c_int {
        if (*cmod).cmod_verbose_save == 0 as OptInt {
            (*cmod).cmod_verbose_save = p_verbose.get() + 1 as OptInt;
        }
        p_verbose.set(((*cmod).cmod_verbose - 1 as ::core::ffi::c_int) as OptInt);
    }
    if (*cmod).cmod_flags
        & (CMOD_SILENT as ::core::ffi::c_int | CMOD_UNSILENT as ::core::ffi::c_int)
        != 0
        && (*cmod).cmod_save_msg_silent == 0 as ::core::ffi::c_int
    {
        (*cmod).cmod_save_msg_silent = msg_silent.get() + 1 as ::core::ffi::c_int;
        (*cmod).cmod_save_msg_scroll = msg_scroll.get();
    }
    if (*cmod).cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        (*msg_silent.ptr()) += 1;
    }
    if (*cmod).cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0 {
        msg_silent.set(0 as ::core::ffi::c_int);
    }
    if (*cmod).cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        (*emsg_silent.ptr()) += 1;
        (*cmod).cmod_did_esilent += 1;
    }
    if (*cmod).cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0
        && (*cmod).cmod_save_ei.is_null()
    {
        (*cmod).cmod_save_ei = xstrdup(p_ei.get());
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"all\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn undo_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_verbose_save > 0 as OptInt {
        p_verbose.set((*cmod).cmod_verbose_save - 1 as OptInt);
        (*cmod).cmod_verbose_save = 0 as OptInt;
    }
    if (*cmod).cmod_did_sandbox != 0 {
        (*sandbox.ptr()) -= 1;
        (*cmod).cmod_did_sandbox = false_0;
    }
    if !(*cmod).cmod_save_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string((*cmod).cmod_save_ei),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        free_string_option((*cmod).cmod_save_ei);
        (*cmod).cmod_save_ei = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    xfree((*cmod).cmod_filter_pat as *mut ::core::ffi::c_void);
    vim_regfree((*cmod).cmod_filter_regmatch.regprog);
    if (*cmod).cmod_save_msg_silent > 0 as ::core::ffi::c_int {
        if did_emsg.get() == 0
            || msg_silent.get() > (*cmod).cmod_save_msg_silent - 1 as ::core::ffi::c_int
        {
            msg_silent.set((*cmod).cmod_save_msg_silent - 1 as ::core::ffi::c_int);
        }
        (*emsg_silent.ptr()) -= (*cmod).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as ::core::ffi::c_int {
            emsg_silent.get()
        } else {
            0 as ::core::ffi::c_int
        });
        msg_scroll.set((*cmod).cmod_save_msg_scroll);
        if redirecting() != 0 {
            msg_col.set(0 as ::core::ffi::c_int);
        }
        (*cmod).cmod_save_msg_silent = 0 as ::core::ffi::c_int;
        (*cmod).cmod_did_esilent = 0 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn parse_cmd_address(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut address_count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut lnum: linenr_T = 0;
    let mut need_check_cursor: bool = false_0 != 0;
    let mut ret: ::core::ffi::c_int = FAIL;
    '_theend: {
        loop {
            (*eap).line1 = (*eap).line2;
            (*eap).line2 = get_cmd_default_range(eap);
            (*eap).cmd = skipwhite((*eap).cmd);
            let c2rust_fresh29 = address_count;
            address_count = address_count + 1;
            lnum = get_address(
                eap,
                &raw mut (*eap).cmd,
                (*eap).addr_type,
                (*eap).skip != 0,
                silent,
                ((*eap).addr_count == 0 as ::core::ffi::c_int) as ::core::ffi::c_int,
                c2rust_fresh29,
                errormsg,
            );
            if (*eap).cmd.is_null() {
                break '_theend;
            }
            if lnum == MAXLNUM as ::core::ffi::c_int as linenr_T {
                if *(*eap).cmd as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                    (*eap).cmd = (*eap).cmd.offset(1);
                    match (*eap).addr_type as ::core::ffi::c_uint {
                        0 | 10 => {
                            (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
                            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
                        }
                        3 => {
                            let mut buf: *mut buf_T = firstbuf.get();
                            while !(*buf).b_next.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_next;
                            }
                            (*eap).line1 = (*buf).handle as linenr_T;
                            buf = lastbuf.get();
                            while !(*buf).b_prev.is_null() && (*buf).b_ml.ml_mfp.is_null() {
                                buf = (*buf).b_prev;
                            }
                            (*eap).line2 = (*buf).handle as linenr_T;
                        }
                        4 => {
                            (*eap).line1 = (*firstbuf.get()).handle as linenr_T;
                            (*eap).line2 = (*lastbuf.get()).handle as linenr_T;
                        }
                        1 | 5 => {
                            if ((*eap).cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                                (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
                                (*eap).line2 = (if (*eap).addr_type as ::core::ffi::c_uint
                                    == ADDR_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    current_win_nr(::core::ptr::null::<win_T>())
                                } else {
                                    current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                }) as linenr_T;
                            } else {
                                *errormsg =
                                    gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                                break '_theend;
                            }
                        }
                        6 | 9 | 8 => {
                            *errormsg =
                                gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                            break '_theend;
                        }
                        2 => {
                            if (*(*curwin.get()).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
                                (*eap).line2 = 0 as ::core::ffi::c_int as linenr_T;
                                (*eap).line1 = (*eap).line2;
                            } else {
                                (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
                                (*eap).line2 = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                            }
                        }
                        7 => {
                            (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
                            (*eap).line2 = qf_get_valid_size(eap) as linenr_T;
                            if (*eap).line2 == 0 as linenr_T {
                                (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                        11 | _ => {}
                    }
                    (*eap).addr_count += 1;
                } else if *(*eap).cmd as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    if (*eap).addr_type as ::core::ffi::c_uint
                        != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        *errormsg = gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                        break '_theend;
                    } else {
                        (*eap).cmd = (*eap).cmd.offset(1);
                        if (*eap).skip == 0 {
                            let mut fm: *mut fmark_T =
                                mark_get_visual(curbuf.get(), '<' as ::core::ffi::c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        3027 as ::core::ffi::c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            (*eap).line1 = (*fm).mark.lnum;
                            fm = mark_get_visual(curbuf.get(), '>' as ::core::ffi::c_int);
                            if !mark_check(fm, errormsg) {
                                break '_theend;
                            }
                            '_c2rust_label_0: {
                                if !fm.is_null() {
                                } else {
                                    __assert_fail(
                                        b"fm != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/ex_docmd.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        3033 as ::core::ffi::c_uint,
                                        b"int parse_cmd_address(exarg_T *, const char **, _Bool)\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            (*eap).line2 = (*fm).mark.lnum;
                            (*eap).addr_count += 1;
                        }
                    }
                }
            } else {
                (*eap).line2 = lnum;
            }
            (*eap).addr_count += 1;
            if *(*eap).cmd as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                if (*eap).skip == 0 {
                    (*curwin.get()).w_cursor.lnum = (*eap).line2;
                    if (*eap).line2 > 0 as linenr_T {
                        check_cursor(curwin.get());
                    } else {
                        check_cursor_col(curwin.get());
                    }
                    need_check_cursor = true_0 != 0;
                }
            } else if *(*eap).cmd as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                break;
            }
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if (*eap).addr_count == 1 as ::core::ffi::c_int {
            (*eap).line1 = (*eap).line2;
            if lnum == MAXLNUM as ::core::ffi::c_int as linenr_T {
                (*eap).addr_count = 0 as ::core::ffi::c_int;
            }
        }
        ret = OK;
    }
    if need_check_cursor {
        check_cursor(curwin.get());
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn checkforcmd(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while *cmd.offset(i as isize) as ::core::ffi::c_int != NUL {
        if *cmd.offset(i as isize) as ::core::ffi::c_int
            != *(*pp).offset(i as isize) as ::core::ffi::c_int
        {
            break;
        }
        i += 1;
    }
    if i >= len
        && !(*(*pp).offset(i as isize) as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *(*pp).offset(i as isize) as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *(*pp).offset(i as isize) as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *(*pp).offset(i as isize) as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
    {
        *pp = skipwhite((*pp).offset(i as isize));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn append_command(mut cmd: *const ::core::ffi::c_char) {
    let mut len: size_t = strlen(IObuff.ptr() as *mut ::core::ffi::c_char);
    let mut s: *const ::core::ffi::c_char = cmd;
    let mut d: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len > (IOSIZE - 100 as ::core::ffi::c_int) as size_t {
        d = (IObuff.ptr() as *mut ::core::ffi::c_char)
            .offset(IOSIZE as isize)
            .offset(-(100 as ::core::ffi::c_int as isize));
        d = d.offset(-(utf_head_off(IObuff.ptr() as *mut ::core::ffi::c_char, d) as isize));
        strcpy(
            d,
            b"...\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
    xstrlcat(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        b": \0".as_ptr() as *const ::core::ffi::c_char,
        IOSIZE as size_t,
    );
    d = (IObuff.ptr() as *mut ::core::ffi::c_char)
        .offset(strlen(IObuff.ptr() as *mut ::core::ffi::c_char) as isize);
    while *s as ::core::ffi::c_int != NUL
        && (d.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char) + 5 as isize) < IOSIZE as isize
    {
        if *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == 0xc2 as ::core::ffi::c_int
            && *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == 0xa0 as ::core::ffi::c_int
        {
            s = s.offset(2 as ::core::ffi::c_int as isize);
            strcpy(
                d,
                b"<a0>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            d = d.offset(4 as ::core::ffi::c_int as isize);
        } else {
            if d.offset_from(IObuff.ptr() as *mut ::core::ffi::c_char)
                + utfc_ptr2len(s) as isize
                + 1 as isize
                >= IOSIZE as isize
            {
                break;
            }
            mb_copy_char(&raw mut s, &raw mut d);
        }
    }
    *d = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn one_letter_cmd(
    mut p: *const ::core::ffi::c_char,
    mut idx: *mut cmdidx_T,
) -> ::core::ffi::c_int {
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'k' as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 'e' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'e' as ::core::ffi::c_int)
    {
        *idx = CMD_k;
        return true_0;
    }
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int
        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'c' as ::core::ffi::c_int
            && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 's' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != 'r' as ::core::ffi::c_int
                    && (*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != 'i' as ::core::ffi::c_int
                            && *p.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != 'p' as ::core::ffi::c_int))
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'g' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'i' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'm' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'l' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'g' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'I' as ::core::ffi::c_int
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'r' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != 'e' as ::core::ffi::c_int)
    {
        *idx = CMD_substitute;
        return true_0;
    }
    return false_0;
}
#[no_mangle]
pub unsafe extern "C" fn find_ex_command(
    mut eap: *mut exarg_T,
    mut full: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = (*eap).cmd;
    if one_letter_cmd(p, &raw mut (*eap).cmdidx) != 0 {
        p = p.offset(1);
        if !full.is_null() {
            *full = true_0;
        }
    } else {
        while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        {
            p = p.offset(1);
        }
        if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'p' as ::core::ffi::c_int
            && *(*eap).cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'y' as ::core::ffi::c_int
        {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                p = p.offset(1);
            }
        }
        if p == (*eap).cmd
            && !vim_strchr(
                b"@!=><&~#\0".as_ptr() as *const ::core::ffi::c_char,
                *p as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            p = p.offset(1);
        }
        let mut len: ::core::ffi::c_int = p.offset_from((*eap).cmd) as ::core::ffi::c_int;
        if *(*eap).cmd as ::core::ffi::c_int == 'd' as ::core::ffi::c_int
            && (*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'l' as ::core::ffi::c_int
                || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int)
        {
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            while i < len {
                if *(*eap).cmd.offset(i as isize) as ::core::ffi::c_int
                    != ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"delete\0")
                        [i as usize] as ::core::ffi::c_int
                {
                    break;
                }
                i += 1;
            }
            if i == len - 1 as ::core::ffi::c_int {
                len -= 1;
                if *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'l' as ::core::ffi::c_int
                {
                    (*eap).flags |= EXFLAG_LIST;
                } else {
                    (*eap).flags |= EXFLAG_PRINT;
                }
            }
        }
        if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint
        {
            let c1: ::core::ffi::c_int = *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize)
                as uint8_t as ::core::ffi::c_int;
            let c2: ::core::ffi::c_int = if len == 1 as ::core::ffi::c_int {
                NUL
            } else {
                *(*eap).cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            };
            if command_count.get() != CMD_SIZE as ::core::ffi::c_int {
                iemsg(gettext(
                    b"E943: Command table needs to be updated, run 'make'\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                getout(1 as ::core::ffi::c_int);
            }
            (*eap).cmdidx = (*cmdidxs1.ptr())
                [(c1 as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize]
                as cmdidx_T;
            if c2 as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && c2 as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            {
                (*eap).cmdidx = ((*eap).cmdidx as ::core::ffi::c_int
                    + (*cmdidxs2.ptr())
                        [(c1 as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize]
                        [(c2 as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize]
                        as ::core::ffi::c_int) as cmdidx_T;
            }
        } else if *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *(*eap).cmd.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
        {
            (*eap).cmdidx = CMD_Next;
        } else {
            (*eap).cmdidx = CMD_bang;
        }
        '_c2rust_label: {
            if (*eap).cmdidx as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"eap->cmdidx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3236 as ::core::ffi::c_uint,
                    b"char *find_ex_command(exarg_T *, int *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if len == 3 as ::core::ffi::c_int
            && strncmp(
                b"def\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).cmd,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*eap).cmdidx = CMD_SIZE;
        }
        while ((*eap).cmdidx as ::core::ffi::c_int) < CMD_SIZE as ::core::ffi::c_int {
            if strncmp(
                (*cmdnames.ptr())[(*eap).cmdidx as ::core::ffi::c_int as usize].cmd_name,
                (*eap).cmd,
                len as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                if !full.is_null()
                    && *(*cmdnames.ptr())[(*eap).cmdidx as ::core::ffi::c_int as usize]
                        .cmd_name
                        .offset(len as isize) as ::core::ffi::c_int
                        == NUL
                {
                    *full = true_0;
                }
                break;
            } else {
                (*eap).cmdidx =
                    ((*eap).cmdidx as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as cmdidx_T;
            }
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
            && *(*eap).cmd as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
            && *(*eap).cmd as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
        {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                p = p.offset(1);
            }
            p = find_ucmd(
                eap,
                p,
                full,
                ::core::ptr::null_mut::<expand_T>(),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
        }
        if p == (*eap).cmd {
            (*eap).cmdidx = CMD_SIZE;
        }
    }
    return p;
}
static cmdmods: GlobalCell<[cmdmod; 24]> = GlobalCell::new([
    cmdmod {
        name: b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"belowright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"botright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 2 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"browse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 4 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"filter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 4 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"horizontal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 5 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 5 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 5 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"leftabove\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 5 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 6 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"sandbox\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"silent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: true_0,
    },
    cmdmod {
        name: b"topleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 2 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"unsilent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 3 as ::core::ffi::c_int,
        has_count: false_0,
    },
    cmdmod {
        name: b"verbose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 4 as ::core::ffi::c_int,
        has_count: true_0,
    },
    cmdmod {
        name: b"vertical\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        minlen: 4 as ::core::ffi::c_int,
        has_count: false_0,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn modifier_len(mut cmd: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = cmd;
    if ascii_isdigit(*cmd as ::core::ffi::c_int) {
        p = skipwhite(skipdigits(cmd.offset(1 as ::core::ffi::c_int as isize)));
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ::core::mem::size_of::<[cmdmod; 24]>()
        .wrapping_div(::core::mem::size_of::<cmdmod>())
        .wrapping_div(
            (::core::mem::size_of::<[cmdmod; 24]>().wrapping_rem(::core::mem::size_of::<cmdmod>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int
    {
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while *p.offset(j as isize) as ::core::ffi::c_int != NUL {
            if *p.offset(j as isize) as ::core::ffi::c_int
                != *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as ::core::ffi::c_int
            {
                break;
            }
            j += 1;
        }
        if j >= (*cmdmods.ptr())[i as usize].minlen
            && !(*p.offset(j as isize) as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p.offset(j as isize) as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p.offset(j as isize) as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p.offset(j as isize) as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint)
            && (p == cmd || (*cmdmods.ptr())[i as usize].has_count != 0)
        {
            return j + p.offset_from(cmd) as ::core::ffi::c_int;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_exists(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ::core::mem::size_of::<[cmdmod; 24]>()
        .wrapping_div(::core::mem::size_of::<cmdmod>())
        .wrapping_div(
            (::core::mem::size_of::<[cmdmod; 24]>().wrapping_rem(::core::mem::size_of::<cmdmod>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int
    {
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while *name.offset(j as isize) as ::core::ffi::c_int != NUL {
            if *name.offset(j as isize) as ::core::ffi::c_int
                != *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as ::core::ffi::c_int
            {
                break;
            }
            j += 1;
        }
        if *name.offset(j as isize) as ::core::ffi::c_int == NUL
            && j >= (*cmdmods.ptr())[i as usize].minlen
        {
            return if *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as ::core::ffi::c_int
                == NUL
            {
                2 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
        }
        i += 1;
    }
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
    ea.cmd = (if *name as ::core::ffi::c_int == '2' as ::core::ffi::c_int
        || *name as ::core::ffi::c_int == '3' as ::core::ffi::c_int
    {
        name.offset(1 as ::core::ffi::c_int as isize)
    } else {
        name
    }) as *mut ::core::ffi::c_char;
    ea.cmdidx = CMD_append;
    ea.flags = 0 as ::core::ffi::c_int;
    let mut full: ::core::ffi::c_int = false_0;
    let mut p: *mut ::core::ffi::c_char = find_ex_command(&raw mut ea, &raw mut full);
    if p.is_null() {
        return 3 as ::core::ffi::c_int;
    }
    if ascii_isdigit(*name as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && ea.cmdidx as ::core::ffi::c_int != CMD_match as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if *skipwhite(p) as ::core::ffi::c_int != NUL {
        return 0 as ::core::ffi::c_int;
    }
    return if ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else if full != 0 {
        2 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_fullcommand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut name: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while *name as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
        name = name.offset(1);
    }
    name = skip_range(name, ::core::ptr::null_mut::<::core::ffi::c_int>());
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
    ea.cmd = if *name as ::core::ffi::c_int == '2' as ::core::ffi::c_int
        || *name as ::core::ffi::c_int == '3' as ::core::ffi::c_int
    {
        name.offset(1 as ::core::ffi::c_int as isize)
    } else {
        name
    };
    ea.cmdidx = CMD_append;
    ea.flags = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char =
        find_ex_command(&raw mut ea, ::core::ptr::null_mut::<::core::ffi::c_int>());
    if p.is_null() || ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
        return;
    }
    (*rettv).vval.v_string = xstrdup(
        if (ea.cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            get_user_command_name(ea.useridx, ea.cmdidx as ::core::ffi::c_int)
        } else {
            (*cmdnames.ptr())[ea.cmdidx as usize].cmd_name
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn excmd_get_cmdidx(
    mut cmd: *const ::core::ffi::c_char,
    mut len: size_t,
) -> cmdidx_T {
    if len == 3 as size_t
        && strncmp(
            b"def\0".as_ptr() as *const ::core::ffi::c_char,
            cmd,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        return CMD_SIZE;
    }
    let mut idx: cmdidx_T = CMD_append;
    if one_letter_cmd(cmd, &raw mut idx) == 0 {
        idx = CMD_append;
        while (idx as ::core::ffi::c_int) < CMD_SIZE as ::core::ffi::c_int {
            if strncmp(
                (*cmdnames.ptr())[idx as ::core::ffi::c_int as usize].cmd_name,
                cmd,
                len,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            idx = (idx as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as cmdidx_T;
        }
    }
    return idx;
}
#[no_mangle]
pub unsafe extern "C" fn excmd_get_argt(mut idx: cmdidx_T) -> uint32_t {
    return (*cmdnames.ptr())[idx as ::core::ffi::c_int as usize].cmd_argt;
}
#[no_mangle]
pub unsafe extern "C" fn skip_range(
    mut cmd: *const ::core::ffi::c_char,
    mut ctx: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    while !vim_strchr(
        b" \t0123456789.$%'/?-+,;\\\0".as_ptr() as *const ::core::ffi::c_char,
        *cmd as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        if *cmd as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            if !(*cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '?' as ::core::ffi::c_int
                || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                || *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '&' as ::core::ffi::c_int)
            {
                break;
            }
            cmd = cmd.offset(1);
        } else if *cmd as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
            cmd = cmd.offset(1);
            if *cmd as ::core::ffi::c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        } else if *cmd as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *cmd as ::core::ffi::c_int == '?' as ::core::ffi::c_int
        {
            let c2rust_fresh27 = cmd;
            cmd = cmd.offset(1);
            let mut delim: ::core::ffi::c_uint = *c2rust_fresh27 as ::core::ffi::c_uint;
            while *cmd as ::core::ffi::c_int != NUL
                && *cmd as ::core::ffi::c_int != delim as ::core::ffi::c_char as ::core::ffi::c_int
            {
                let c2rust_fresh28 = cmd;
                cmd = cmd.offset(1);
                if *c2rust_fresh28 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && *cmd as ::core::ffi::c_int != NUL
                {
                    cmd = cmd.offset(1);
                }
            }
            if *cmd as ::core::ffi::c_int == NUL && !ctx.is_null() {
                *ctx = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
        if *cmd as ::core::ffi::c_int != NUL {
            cmd = cmd.offset(1);
        }
    }
    cmd = skip_colon_white(cmd, false_0 != 0);
    if *cmd as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        cmd = skipwhite(cmd.offset(1 as ::core::ffi::c_int as isize));
    }
    return cmd as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn addr_error(mut addr_type: cmd_addr_T) -> *const ::core::ffi::c_char {
    if addr_type as ::core::ffi::c_uint == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint {
        return gettext(&raw const e_norange as *const ::core::ffi::c_char);
    } else {
        return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
    };
}
#[no_mangle]
pub unsafe extern "C" fn get_address(
    mut eap: *mut exarg_T,
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut addr_type: cmd_addr_T,
    mut skip: bool,
    mut silent: bool,
    mut to_other_file: ::core::ffi::c_int,
    mut address_count: ::core::ffi::c_int,
    mut errormsg: *mut *const ::core::ffi::c_char,
) -> linenr_T {
    let mut c: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut n: linenr_T = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut cmd: *mut ::core::ffi::c_char = skipwhite(*ptr);
    let mut lnum: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
    '_error: loop {
        match *cmd as ::core::ffi::c_int {
            46 => {
                cmd = cmd.offset(1);
                match addr_type as ::core::ffi::c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    _ => {}
                }
            }
            36 => {
                cmd = cmd.offset(1);
                match addr_type as ::core::ffi::c_uint {
                    0 | 10 => {
                        lnum = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    1 => {
                        lnum = current_win_nr(::core::ptr::null::<win_T>()) as linenr_T;
                    }
                    2 => {
                        lnum = (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T;
                    }
                    3 => {
                        buf = lastbuf.get();
                        while (*buf).b_ml.ml_mfp.is_null() {
                            if (*buf).b_prev.is_null() {
                                break;
                            }
                            buf = (*buf).b_prev;
                        }
                        lnum = (*buf).handle as linenr_T;
                    }
                    4 => {
                        lnum = (*lastbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T;
                    }
                    11 | 6 | 9 => {
                        *errormsg = addr_error(addr_type);
                        cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        break;
                    }
                    8 => {
                        lnum = qf_get_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as ::core::ffi::c_int as linenr_T;
                        }
                    }
                    7 => {
                        lnum = qf_get_valid_size(eap) as linenr_T;
                        if lnum == 0 as linenr_T {
                            lnum = 1 as ::core::ffi::c_int as linenr_T;
                        }
                    }
                    _ => {}
                }
            }
            39 => {
                cmd = cmd.offset(1);
                if *cmd as ::core::ffi::c_int == NUL {
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break;
                } else if addr_type as ::core::ffi::c_uint
                    != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break;
                } else if skip {
                    cmd = cmd.offset(1);
                } else {
                    let mut flag: MarkGet = (if to_other_file != 0
                        && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                    {
                        kMarkAll as ::core::ffi::c_int
                    } else {
                        kMarkBufLocal as ::core::ffi::c_int
                    }) as MarkGet;
                    let mut fm: *mut fmark_T = mark_get(
                        curbuf.get(),
                        curwin.get(),
                        ::core::ptr::null_mut::<fmark_T>(),
                        flag,
                        *cmd as ::core::ffi::c_int,
                    );
                    cmd = cmd.offset(1);
                    if !fm.is_null() && (*fm).fnum != (*curbuf.get()).handle {
                        mark_move_to(fm, 0 as MarkMove);
                        lnum = (*curwin.get()).w_cursor.lnum;
                    } else if !mark_check(fm, errormsg) {
                        cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        break;
                    } else {
                        '_c2rust_label: {
                            if !fm.is_null() {
                            } else {
                                __assert_fail(
                                    b"fm != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/ex_docmd.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    3618 as ::core::ffi::c_uint,
                                    b"linenr_T get_address(exarg_T *, char **, cmd_addr_T, _Bool, _Bool, int, int, const char **)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        lnum = (*fm).mark.lnum;
                    }
                }
            }
            47 | 63 => {
                let c2rust_fresh2 = cmd;
                cmd = cmd.offset(1);
                c = *c2rust_fresh2 as uint8_t as ::core::ffi::c_int;
                if addr_type as ::core::ffi::c_uint
                    != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break;
                } else if skip {
                    cmd = skip_regexp(cmd, c, magic_isset() as ::core::ffi::c_int);
                    if *cmd as ::core::ffi::c_int == c {
                        cmd = cmd.offset(1);
                    }
                } else {
                    let mut flags: ::core::ffi::c_int = 0;
                    pos = (*curwin.get()).w_cursor;
                    if lnum > 0 as linenr_T && lnum != MAXLNUM as ::core::ffi::c_int as linenr_T {
                        (*curwin.get()).w_cursor.lnum = if lnum > (*curbuf.get()).b_ml.ml_line_count
                        {
                            (*curbuf.get()).b_ml.ml_line_count
                        } else {
                            lnum
                        };
                    }
                    (*curwin.get()).w_cursor.col = (if c == '/' as ::core::ffi::c_int
                        && (*curwin.get()).w_cursor.lnum > 0 as linenr_T
                    {
                        MAXCOL as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as colnr_T;
                    searchcmdlen.set(0 as ::core::ffi::c_int);
                    flags = if silent as ::core::ffi::c_int != 0 {
                        SEARCH_KEEP as ::core::ffi::c_int
                    } else {
                        SEARCH_HIS as ::core::ffi::c_int | SEARCH_MSG as ::core::ffi::c_int
                    };
                    if do_search(
                        ::core::ptr::null_mut::<oparg_T>(),
                        c,
                        c,
                        cmd,
                        strlen(cmd),
                        1 as ::core::ffi::c_int,
                        flags,
                        ::core::ptr::null_mut::<searchit_arg_T>(),
                    ) == 0
                    {
                        (*curwin.get()).w_cursor = pos;
                        cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        break;
                    } else {
                        lnum = (*curwin.get()).w_cursor.lnum;
                        (*curwin.get()).w_cursor = pos;
                        cmd = cmd.offset(searchcmdlen.get() as isize);
                    }
                }
            }
            92 => {
                cmd = cmd.offset(1);
                if addr_type as ::core::ffi::c_uint
                    != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    *errormsg = addr_error(addr_type);
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break;
                } else {
                    if *cmd as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
                        i = RE_SUBST as ::core::ffi::c_int;
                    } else if *cmd as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                        || *cmd as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    {
                        i = RE_SEARCH as ::core::ffi::c_int;
                    } else {
                        *errormsg = gettext(&raw const e_backslash as *const ::core::ffi::c_char);
                        cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        break;
                    }
                    if !skip {
                        pos.lnum = if lnum != MAXLNUM as ::core::ffi::c_int as linenr_T {
                            lnum
                        } else {
                            (*curwin.get()).w_cursor.lnum
                        };
                        pos.col = (if *cmd as ::core::ffi::c_int != '?' as ::core::ffi::c_int {
                            MAXCOL as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as colnr_T;
                        pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        if searchit(
                            curwin.get(),
                            curbuf.get(),
                            &raw mut pos,
                            ::core::ptr::null_mut::<pos_T>(),
                            (if *cmd as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                BACKWARD as ::core::ffi::c_int
                            } else {
                                FORWARD as ::core::ffi::c_int
                            }) as Direction,
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            0 as size_t,
                            1 as ::core::ffi::c_int,
                            SEARCH_MSG as ::core::ffi::c_int,
                            i,
                            ::core::ptr::null_mut::<searchit_arg_T>(),
                        ) != FAIL
                        {
                            lnum = pos.lnum;
                        } else {
                            cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            break;
                        }
                    }
                    cmd = cmd.offset(1);
                }
            }
            _ => {
                if ascii_isdigit(*cmd as ::core::ffi::c_int) {
                    lnum = getdigits(&raw mut cmd, false_0 != 0, 0 as intmax_t) as linenr_T;
                }
            }
        }
        loop {
            cmd = skipwhite(cmd);
            if *cmd as ::core::ffi::c_int != '-' as ::core::ffi::c_int
                && *cmd as ::core::ffi::c_int != '+' as ::core::ffi::c_int
                && !ascii_isdigit(*cmd as ::core::ffi::c_int)
            {
                break;
            }
            if lnum == MAXLNUM as ::core::ffi::c_int as linenr_T {
                match addr_type as ::core::ffi::c_uint {
                    0 | 10 => {
                        lnum = (*curwin.get()).w_cursor.lnum;
                    }
                    1 => {
                        lnum = current_win_nr(curwin.get()) as linenr_T;
                    }
                    2 => {
                        lnum = ((*curwin.get()).w_arg_idx + 1 as ::core::ffi::c_int) as linenr_T;
                    }
                    3 | 4 => {
                        lnum = (*curbuf.get()).handle as linenr_T;
                    }
                    5 => {
                        lnum = current_tab_nr(curtab.get()) as linenr_T;
                    }
                    6 => {
                        lnum = 1 as ::core::ffi::c_int as linenr_T;
                    }
                    8 => {
                        lnum = qf_get_cur_idx(eap) as linenr_T;
                    }
                    7 => {
                        lnum = qf_get_cur_valid_idx(eap) as linenr_T;
                    }
                    11 | 9 => {
                        lnum = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    _ => {}
                }
            }
            if ascii_isdigit(*cmd as ::core::ffi::c_int) {
                i = '+' as ::core::ffi::c_int;
            } else {
                let c2rust_fresh3 = cmd;
                cmd = cmd.offset(1);
                i = *c2rust_fresh3 as uint8_t as ::core::ffi::c_int;
            }
            if !ascii_isdigit(*cmd as ::core::ffi::c_int) {
                n = 1 as ::core::ffi::c_int as linenr_T;
            } else {
                n = getdigits_int32(
                    &raw mut cmd,
                    false_0 != 0,
                    MAXLNUM as ::core::ffi::c_int as int32_t,
                ) as linenr_T;
                if n == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *errormsg = gettext(
                        &raw const e_line_number_out_of_range as *const ::core::ffi::c_char,
                    );
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break '_error;
                }
            }
            if addr_type as ::core::ffi::c_uint
                == ADDR_TABS_RELATIVE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *errormsg = gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                break '_error;
            } else if addr_type as ::core::ffi::c_uint
                == ADDR_LOADED_BUFFERS as ::core::ffi::c_int as ::core::ffi::c_uint
                || addr_type as ::core::ffi::c_uint
                    == ADDR_BUFFERS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lnum = compute_buffer_local_count(
                    addr_type,
                    lnum,
                    if i == '-' as ::core::ffi::c_int {
                        -1 as ::core::ffi::c_int * n as ::core::ffi::c_int
                    } else {
                        n as ::core::ffi::c_int
                    },
                ) as linenr_T;
            } else {
                if addr_type as ::core::ffi::c_uint
                    == ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (i == '-' as ::core::ffi::c_int || i == '+' as ::core::ffi::c_int)
                    && address_count >= 2 as ::core::ffi::c_int
                {
                    hasFolding(
                        curwin.get(),
                        lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut lnum,
                    );
                }
                if i == '-' as ::core::ffi::c_int {
                    lnum -= n;
                } else if lnum >= 0 as linenr_T && n >= INT32_MAX as linenr_T - lnum {
                    *errormsg = gettext(
                        &raw const e_line_number_out_of_range as *const ::core::ffi::c_char,
                    );
                    cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    break '_error;
                } else {
                    lnum += n;
                }
            }
        }
        if !(*cmd as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *cmd as ::core::ffi::c_int == '?' as ::core::ffi::c_int)
        {
            break;
        }
    }
    *ptr = cmd;
    return lnum;
}
unsafe extern "C" fn get_flags(mut eap: *mut exarg_T) {
    while !vim_strchr(
        b"lp#\0".as_ptr() as *const ::core::ffi::c_char,
        *(*eap).arg as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        if *(*eap).arg as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*eap).flags |= EXFLAG_LIST;
        } else if *(*eap).arg as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
            (*eap).flags |= EXFLAG_PRINT;
        } else {
            (*eap).flags |= EXFLAG_NR;
        }
        (*eap).arg = skipwhite((*eap).arg.offset(1 as ::core::ffi::c_int as isize));
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        (*eap).errmsg = gettext(
            b"E319: The command is not available in this version\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
}
unsafe extern "C" fn ex_script_ni(mut eap: *mut exarg_T) {
    if (*eap).skip == 0 {
        ex_ni(eap);
    } else {
        let mut len: size_t = 0;
        xfree(script_get(eap, &raw mut len) as *mut ::core::ffi::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn invalid_range(mut eap: *mut exarg_T) -> *mut ::core::ffi::c_char {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*eap).line1 < 0 as linenr_T || (*eap).line2 < 0 as linenr_T || (*eap).line1 > (*eap).line2 {
        return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        match (*eap).addr_type as ::core::ffi::c_uint {
            0 => {
                if (*eap).line2
                    > (*curbuf.get()).b_ml.ml_line_count
                        + ((*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int
                            || (*eap).cmdidx as ::core::ffi::c_int
                                == CMD_diffput as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            2 => {
                if (*eap).line2
                    > (*(*curwin.get()).w_alist).al_ga.ga_len as linenr_T
                        + ((*(*curwin.get()).w_alist).al_ga.ga_len == 0) as ::core::ffi::c_int
                {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            4 => {
                if (*eap).line1 < 1 as linenr_T || (*eap).line2 > get_highest_fnum() as linenr_T {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            3 => {
                buf = firstbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_next.is_null() {
                        return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                    }
                    buf = (*buf).b_next;
                }
                if (*eap).line1 < (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
                buf = lastbuf.get();
                while (*buf).b_ml.ml_mfp.is_null() {
                    if (*buf).b_prev.is_null() {
                        return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                    }
                    buf = (*buf).b_prev;
                }
                if (*eap).line2 > (*buf).handle as linenr_T {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            1 => {
                if (*eap).line2 > current_win_nr(::core::ptr::null::<win_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            5 => {
                if (*eap).line2 > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) as linenr_T {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            8 => {
                '_c2rust_label: {
                    if (*eap).line2 >= 0 as linenr_T {
                    } else {
                        __assert_fail(
                            b"eap->line2 >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3906 as ::core::ffi::c_uint,
                            b"char *invalid_range(exarg_T *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if (*eap).line2 <= 0 as linenr_T {
                    if (*eap).addr_count == 0 as ::core::ffi::c_int {
                        return gettext(&raw const e_no_errors as *const ::core::ffi::c_char);
                    }
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            7 => {
                if (*eap).line2 != 1 as linenr_T && (*eap).line2 as size_t > qf_get_valid_size(eap)
                    || (*eap).line2 < 0 as linenr_T
                {
                    return gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                }
            }
            6 | 10 | 9 | 11 | _ => {}
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn correct_range(mut eap: *mut exarg_T) {
    if (*eap).argt & EX_ZEROR as uint32_t == 0 {
        if (*eap).line1 == 0 as linenr_T {
            (*eap).line1 = 1 as ::core::ffi::c_int as linenr_T;
        }
        if (*eap).line2 == 0 as linenr_T {
            (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
        }
    }
}
unsafe extern "C" fn skip_grep_pat(mut eap: *mut exarg_T) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    if *p as ::core::ffi::c_int != NUL
        && ((*eap).cmdidx as ::core::ffi::c_int == CMD_vimgrep as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lvimgrep as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_vimgrepadd as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_lvimgrepadd as ::core::ffi::c_int
            || grep_internal((*eap).cmdidx) != 0)
    {
        p = skip_vimgrep_pat(
            p,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if p.is_null() {
            p = (*eap).arg;
        }
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn replace_makeprg(
    mut eap: *mut exarg_T,
    mut arg: *mut ::core::ffi::c_char,
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut isgrep: bool = (*eap).cmdidx as ::core::ffi::c_int == CMD_grep as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lgrep as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_grepadd as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lgrepadd as ::core::ffi::c_int;
    if ((*eap).cmdidx as ::core::ffi::c_int == CMD_make as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_lmake as ::core::ffi::c_int
        || isgrep as ::core::ffi::c_int != 0)
        && grep_internal((*eap).cmdidx) == 0
    {
        let mut program: *const ::core::ffi::c_char = if isgrep as ::core::ffi::c_int != 0 {
            if *(*curbuf.get()).b_p_gp as ::core::ffi::c_int == NUL {
                p_gp.get()
            } else {
                (*curbuf.get()).b_p_gp
            }
        } else if *(*curbuf.get()).b_p_mp as ::core::ffi::c_int == NUL {
            p_mp.get()
        } else {
            (*curbuf.get()).b_p_mp
        };
        arg = skipwhite(arg);
        let mut new_cmdline: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        new_cmdline = strrep(program, b"$*\0".as_ptr() as *const ::core::ffi::c_char, arg);
        if new_cmdline.is_null() {
            new_cmdline = xmalloc(
                strlen(program)
                    .wrapping_add(strlen(arg))
                    .wrapping_add(2 as size_t),
            ) as *mut ::core::ffi::c_char;
            strcpy(new_cmdline, program as *mut ::core::ffi::c_char);
            strcat(new_cmdline, b" \0".as_ptr() as *const ::core::ffi::c_char);
            strcat(new_cmdline, arg);
        }
        msg_make(arg);
        xfree(*cmdlinep as *mut ::core::ffi::c_void);
        *cmdlinep = new_cmdline;
        arg = new_cmdline;
    }
    return arg;
}
#[no_mangle]
pub unsafe extern "C" fn expand_filename(
    mut eap: *mut exarg_T,
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut errormsgp: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = skip_grep_pat(eap);
    let mut has_wildcards: bool = path_has_wildcard(p);
    while *p as ::core::ffi::c_int != NUL {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            p = p.offset(2 as ::core::ffi::c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        } else if vim_strchr(
            b"%#<\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut escaped: ::core::ffi::c_int = 0;
            let mut repl: *mut ::core::ffi::c_char = eval_vars(
                p,
                (*eap).arg,
                &raw mut srclen,
                &raw mut (*eap).do_ecmd_lnum,
                errormsgp,
                &raw mut escaped,
                true_0 != 0,
            );
            if !(*errormsgp).is_null() {
                return FAIL;
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                if !vim_strchr(repl, '$' as ::core::ffi::c_int).is_null()
                    || !vim_strchr(repl, '~' as ::core::ffi::c_int).is_null()
                {
                    let mut l: *mut ::core::ffi::c_char = repl;
                    repl = expand_env_save(repl);
                    xfree(l as *mut ::core::ffi::c_void);
                }
                if (*eap).usefilter == 0
                    && escaped == 0
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_bang as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_grep as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_grepadd as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrep as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_lgrepadd as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_lmake as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_make as ::core::ffi::c_int
                    && (*eap).cmdidx as ::core::ffi::c_int != CMD_terminal as ::core::ffi::c_int
                    && (*eap).argt & EX_NOSPC as uint32_t == 0
                {
                    let mut l_0: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    l_0 = repl;
                    while *l_0 != 0 {
                        if !vim_strchr(escape_chars.get(), *l_0 as uint8_t as ::core::ffi::c_int)
                            .is_null()
                        {
                            l_0 = vim_strsave_escaped(repl, escape_chars.get());
                            xfree(repl as *mut ::core::ffi::c_void);
                            repl = l_0;
                            break;
                        } else {
                            l_0 = l_0.offset(1);
                        }
                    }
                }
                if ((*eap).usefilter != 0
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_bang as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_terminal as ::core::ffi::c_int)
                    && !strpbrk(repl, b"!\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                {
                    let mut l_1: *mut ::core::ffi::c_char =
                        vim_strsave_escaped(repl, b"!\0".as_ptr() as *const ::core::ffi::c_char);
                    xfree(repl as *mut ::core::ffi::c_void);
                    repl = l_1;
                }
                p = repl_cmdline(eap, p, srclen, repl, cmdlinep);
                xfree(repl as *mut ::core::ffi::c_void);
            }
        }
    }
    if (*eap).argt & EX_NOSPC as uint32_t != 0 && (*eap).usefilter == 0 {
        if has_wildcards {
            if !vim_strchr((*eap).arg, '$' as ::core::ffi::c_int).is_null()
                || !vim_strchr((*eap).arg, '~' as ::core::ffi::c_int).is_null()
            {
                expand_env_esc(
                    (*eap).arg,
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    MAXPATHL,
                    true_0 != 0,
                    true_0 != 0,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                has_wildcards = path_has_wildcard(NameBuff.ptr() as *mut ::core::ffi::c_char);
                p = NameBuff.ptr() as *mut ::core::ffi::c_char;
            } else {
                p = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if !p.is_null() {
                repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            }
        }
        if !has_wildcards {
            backslash_halve((*eap).arg);
        }
        if has_wildcards {
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
            let mut options: ::core::ffi::c_int = WILD_LIST_NOTFOUND as ::core::ffi::c_int
                | WILD_NOERROR as ::core::ffi::c_int
                | WILD_ADD_SLASH as ::core::ffi::c_int;
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
            if p_wic.get() != 0 {
                options += WILD_ICASE as ::core::ffi::c_int;
            }
            p = ExpandOne(
                &raw mut xpc,
                (*eap).arg,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                options,
                WILD_EXPAND_FREE as ::core::ffi::c_int,
            );
            if p.is_null() {
                return FAIL;
            }
            repl_cmdline(eap, (*eap).arg, strlen((*eap).arg), p, cmdlinep);
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
    return OK;
}
unsafe extern "C" fn repl_cmdline(
    mut eap: *mut exarg_T,
    mut src: *mut ::core::ffi::c_char,
    mut srclen: size_t,
    mut repl: *mut ::core::ffi::c_char,
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(repl);
    let mut i: size_t = (src.offset_from(*cmdlinep) as size_t)
        .wrapping_add(strlen(src.offset(srclen as isize)))
        .wrapping_add(len)
        .wrapping_add(3 as size_t);
    if !(*eap).nextcmd.is_null() {
        i = i.wrapping_add(strlen((*eap).nextcmd));
    }
    let mut new_cmdline: *mut ::core::ffi::c_char = xmalloc(i) as *mut ::core::ffi::c_char;
    let mut offset: size_t = src.offset_from(*cmdlinep) as size_t;
    i = offset;
    memmove(
        new_cmdline as *mut ::core::ffi::c_void,
        *cmdlinep as *const ::core::ffi::c_void,
        i,
    );
    memmove(
        new_cmdline.offset(i as isize) as *mut ::core::ffi::c_void,
        repl as *const ::core::ffi::c_void,
        len,
    );
    i = i.wrapping_add(len);
    strcpy(new_cmdline.offset(i as isize), src.offset(srclen as isize));
    src = new_cmdline.offset(i as isize);
    if !(*eap).nextcmd.is_null() {
        i = strlen(new_cmdline).wrapping_add(1 as size_t);
        strcpy(new_cmdline.offset(i as isize), (*eap).nextcmd);
        (*eap).nextcmd = new_cmdline.offset(i as isize);
    }
    (*eap).cmd = new_cmdline.offset((*eap).cmd.offset_from(*cmdlinep) as isize);
    (*eap).arg = new_cmdline.offset((*eap).arg.offset_from(*cmdlinep) as isize);
    let mut j: size_t = 0 as size_t;
    while j < (*eap).argc {
        if offset >= (*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as size_t {
            *(*eap).args.offset(j as isize) = new_cmdline
                .offset((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep) as isize);
        } else {
            *(*eap).args.offset(j as isize) = new_cmdline.offset(
                ((*(*eap).args.offset(j as isize)).offset_from(*cmdlinep)
                    + len.wrapping_sub(srclen) as isize) as isize,
            );
        }
        j = j.wrapping_add(1);
    }
    if !(*eap).do_ecmd_cmd.is_null()
        && (*eap).do_ecmd_cmd != dollar_command.ptr() as *mut ::core::ffi::c_char
    {
        (*eap).do_ecmd_cmd = new_cmdline.offset((*eap).do_ecmd_cmd.offset_from(*cmdlinep) as isize);
    }
    xfree(*cmdlinep as *mut ::core::ffi::c_void);
    *cmdlinep = new_cmdline;
    return src;
}
#[no_mangle]
pub unsafe extern "C" fn separate_nextcmd(mut eap: *mut exarg_T) {
    let mut p: *mut ::core::ffi::c_char = skip_grep_pat(eap);
    while *p != 0 {
        if *p as ::core::ffi::c_int == Ctrl_V {
            if (*eap).argt & (EX_CTRLV as uint32_t | EX_XFILE as uint32_t) != 0 {
                p = p.offset(1);
            } else {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
            && (*eap).argt & EX_XFILE as uint32_t != 0
        {
            p = p.offset(2 as ::core::ffi::c_int as isize);
            skip_expr(&raw mut p, ::core::ptr::null_mut::<evalarg_T>());
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            && (*eap).argt & EX_NOTRLCOM as uint32_t == 0
            && ((*eap).cmdidx as ::core::ffi::c_int != CMD_at as ::core::ffi::c_int
                || p != (*eap).arg)
            && ((*eap).cmdidx as ::core::ffi::c_int != CMD_redir as ::core::ffi::c_int
                || p != (*eap).arg.offset(1 as ::core::ffi::c_int as isize)
                || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '@' as ::core::ffi::c_int)
            || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_change as ::core::ffi::c_int
                && (*eap).cmdidx as ::core::ffi::c_int != CMD_insert as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            if (vim_strchr(p_cpo.get(), CPO_BAR).is_null()
                || (*eap).argt & EX_CTRLV as uint32_t == 0)
                && *p.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                memmove(
                    p.offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    strlen(p).wrapping_add(1 as size_t),
                );
                p = p.offset(-1);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
                *p = NUL as ::core::ffi::c_char;
                break;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if (*eap).argt & EX_NOTRLCOM as uint32_t == 0 {
        del_trailing_spaces((*eap).arg);
    }
}
#[no_mangle]
pub unsafe extern "C" fn getargcmd(
    mut argp: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut arg: *mut ::core::ffi::c_char = *argp;
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
        arg = arg.offset(1);
        if ascii_isspace(*arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || *arg as ::core::ffi::c_int == NUL
        {
            command = dollar_command.ptr() as *mut ::core::ffi::c_char;
        } else {
            command = arg;
            arg = skip_cmd_arg(command, true_0 != 0);
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh26 = arg;
                arg = arg.offset(1);
                *c2rust_fresh26 = NUL as ::core::ffi::c_char;
            }
        }
        arg = skipwhite(arg);
        *argp = arg;
    }
    return command;
}
#[no_mangle]
pub unsafe extern "C" fn skip_cmd_arg(
    mut p: *mut ::core::ffi::c_char,
    mut rembs: bool,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != 0 && !ascii_isspace(*p as ::core::ffi::c_int) {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            if rembs {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            } else {
                p = p.offset(1);
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn get_bad_opt(
    mut p: *const ::core::ffi::c_char,
    mut eap: *mut exarg_T,
) -> ::core::ffi::c_int {
    if strcasecmp(
        p as *mut ::core::ffi::c_char,
        b"keep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*eap).bad_char = BAD_KEEP;
    } else if strcasecmp(
        p as *mut ::core::ffi::c_char,
        b"drop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*eap).bad_char = BAD_DROP;
    } else if utf8len_tab[*p as uint8_t as usize] as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        (*eap).bad_char = *p as uint8_t as ::core::ffi::c_int;
    } else {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn get_bad_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static p_bad_values: GlobalCell<[*mut ::core::ffi::c_char; 3]> = GlobalCell::new([
        b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"keep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"drop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 3]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 3]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return (*p_bad_values.ptr())[idx as usize] as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn getargopt(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg.offset(2 as ::core::ffi::c_int as isize);
    let mut pp: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut bad_char_idx: ::core::ffi::c_int = 0;
    if strncmp(
        arg,
        b"bin\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
        || strncmp(
            arg,
            b"nobin\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        if *arg as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            arg = arg.offset(2 as ::core::ffi::c_int as isize);
            (*eap).force_bin = FORCE_NOBIN;
        } else {
            (*eap).force_bin = FORCE_BIN;
        }
        if !checkforcmd(
            &raw mut arg,
            b"binary\0".as_ptr() as *const ::core::ffi::c_char,
            3 as ::core::ffi::c_int,
        ) {
            return FAIL;
        }
        (*eap).arg = skipwhite(arg);
        return OK;
    }
    if strncmp(
        arg,
        b"edit\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
        && !(*arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        (*eap).read_edit = true_0;
        (*eap).arg = skipwhite(arg.offset(4 as ::core::ffi::c_int as isize));
        return OK;
    }
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'p' as ::core::ffi::c_int
        && !(*arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint
            || *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        (*eap).mkdir_p = true_0;
        (*eap).arg = skipwhite(arg.offset(1 as ::core::ffi::c_int as isize));
        return OK;
    }
    if strncmp(
        arg,
        b"ff\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(2 as ::core::ffi::c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(
        arg,
        b"fileformat\0".as_ptr() as *const ::core::ffi::c_char,
        10 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(10 as ::core::ffi::c_int as isize);
        pp = &raw mut (*eap).force_ff;
    } else if strncmp(
        arg,
        b"enc\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        if strncmp(
            arg,
            b"encoding\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(8 as ::core::ffi::c_int as isize);
        } else {
            arg = arg.offset(3 as ::core::ffi::c_int as isize);
        }
        pp = &raw mut (*eap).force_enc;
    } else if strncmp(
        arg,
        b"bad\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(3 as ::core::ffi::c_int as isize);
        pp = &raw mut bad_char_idx;
    }
    if pp.is_null() || *arg as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
        return FAIL;
    }
    arg = arg.offset(1);
    *pp = arg.offset_from((*eap).cmd) as ::core::ffi::c_int;
    arg = skip_cmd_arg(arg, false_0 != 0);
    (*eap).arg = skipwhite(arg);
    *arg = NUL as ::core::ffi::c_char;
    if pp == &raw mut (*eap).force_ff {
        if check_ff_value((*eap).cmd.offset((*eap).force_ff as isize)) == FAIL {
            return FAIL;
        }
        (*eap).force_ff =
            *(*eap).cmd.offset((*eap).force_ff as isize) as uint8_t as ::core::ffi::c_int;
    } else if pp == &raw mut (*eap).force_enc {
        let mut p: *mut ::core::ffi::c_char = (*eap).cmd.offset((*eap).force_enc as isize);
        while *p as ::core::ffi::c_int != NUL {
            *p = (if (*p as ::core::ffi::c_int) < 'A' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int > 'Z' as ::core::ffi::c_int
            {
                *p as ::core::ffi::c_int
            } else {
                *p as ::core::ffi::c_int + ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) as ::core::ffi::c_char;
            p = p.offset(1);
        }
    } else if get_bad_opt((*eap).cmd.offset(bad_char_idx as isize), eap) == FAIL {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn get_argopt_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static p_opt_values: GlobalCell<[*mut ::core::ffi::c_char; 7]> = GlobalCell::new([
        b"fileformat=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"encoding=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"binary\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"nobinary\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"bad=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"edit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    if idx
        < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 7]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 7]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return (*p_opt_values.ptr())[idx as usize] as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_argopt(
    mut pat: *mut ::core::ffi::c_char,
    mut xp: *mut expand_T,
    mut rmp: *mut regmatch_T,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*xp).xp_pattern > (*xp).xp_line
        && *(*xp).xp_pattern.offset(-(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        let mut cb: CompleteListItemGetter = None;
        let mut name_end: *mut ::core::ffi::c_char =
            (*xp).xp_pattern.offset(-(1 as ::core::ffi::c_int as isize));
        if name_end.offset_from((*xp).xp_line) >= 2 as isize
            && strncmp(
                name_end.offset(-(2 as ::core::ffi::c_int as isize)),
                b"ff\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            cb = Some(
                get_fileformat_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 10 as isize
            && strncmp(
                name_end.offset(-(10 as ::core::ffi::c_int as isize)),
                b"fileformat\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            cb = Some(
                get_fileformat_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as ::core::ffi::c_int as isize)),
                b"enc\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            cb = Some(
                get_encoding_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 8 as isize
            && strncmp(
                name_end.offset(-(8 as ::core::ffi::c_int as isize)),
                b"encoding\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            cb = Some(
                get_encoding_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ) as CompleteListItemGetter;
        } else if name_end.offset_from((*xp).xp_line) >= 3 as isize
            && strncmp(
                name_end.offset(-(3 as ::core::ffi::c_int as isize)),
                b"bad\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            cb = Some(
                get_bad_name
                    as unsafe extern "C" fn(
                        *mut expand_T,
                        ::core::ffi::c_int,
                    ) -> *mut ::core::ffi::c_char,
            ) as CompleteListItemGetter;
        }
        if cb.is_some() {
            ExpandGeneric(pat, xp, rmp, matches, numMatches, cb, false_0 != 0);
            return OK;
        }
        return FAIL;
    }
    if (*xp).xp_pattern_len == 2 as size_t
        && strncmp(
            (*xp).xp_pattern,
            b"ff\0".as_ptr() as *const ::core::ffi::c_char,
            (*xp).xp_pattern_len,
        ) == 0 as ::core::ffi::c_int
    {
        *matches = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            as *mut *mut ::core::ffi::c_char;
        *numMatches = 1 as ::core::ffi::c_int;
        *(*matches).offset(0 as ::core::ffi::c_int as isize) =
            xstrdup(b"fileformat=\0".as_ptr() as *const ::core::ffi::c_char);
        return OK;
    }
    ExpandGeneric(
        pat,
        xp,
        rmp,
        matches,
        numMatches,
        Some(
            get_argopt_name
                as unsafe extern "C" fn(
                    *mut expand_T,
                    ::core::ffi::c_int,
                ) -> *mut ::core::ffi::c_char,
        ),
        false_0 != 0,
    );
    return OK;
}
unsafe extern "C" fn get_tabpage_arg(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    let mut tab_number: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut unaccept_arg0: ::core::ffi::c_int =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_tabmove as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    '_theend: {
        if !(*eap).arg.is_null() && *(*eap).arg as ::core::ffi::c_int != NUL {
            let mut p: *mut ::core::ffi::c_char = (*eap).arg;
            let mut relative: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                relative = -1 as ::core::ffi::c_int;
                p = p.offset(1);
            } else if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                relative = 1 as ::core::ffi::c_int;
                p = p.offset(1);
            }
            let mut p_save: *mut ::core::ffi::c_char = p;
            tab_number =
                getdigits(&raw mut p, false_0 != 0, tab_number as intmax_t) as ::core::ffi::c_int;
            if relative == 0 as ::core::ffi::c_int {
                if strcmp(p, b"$\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                } else if strcmp(p, b"#\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    if valid_tabpage(lastused_tabpage.get()) {
                        tab_number = tabpage_index(lastused_tabpage.get());
                    } else {
                        (*eap).errmsg = ex_errmsg(
                            &raw const e_invargval as *const ::core::ffi::c_char,
                            (*eap).arg,
                        );
                        tab_number = 0 as ::core::ffi::c_int;
                        break '_theend;
                    }
                } else if p == p_save
                    || *p_save as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int != NUL
                    || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                {
                    (*eap).errmsg = ex_errmsg(
                        &raw const e_invarg2 as *const ::core::ffi::c_char,
                        (*eap).arg,
                    );
                    break '_theend;
                }
            } else {
                if *p_save as ::core::ffi::c_int == NUL {
                    tab_number = 1 as ::core::ffi::c_int;
                } else if p == p_save
                    || *p_save as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int != NUL
                    || tab_number == 0 as ::core::ffi::c_int
                {
                    (*eap).errmsg = ex_errmsg(
                        &raw const e_invarg2 as *const ::core::ffi::c_char,
                        (*eap).arg,
                    );
                    break '_theend;
                }
                tab_number = tab_number * relative + tabpage_index(curtab.get());
                if unaccept_arg0 == 0 && relative == -1 as ::core::ffi::c_int {
                    tab_number -= 1;
                }
            }
            if tab_number < unaccept_arg0
                || tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
            {
                (*eap).errmsg = ex_errmsg(
                    &raw const e_invarg2 as *const ::core::ffi::c_char,
                    (*eap).arg,
                );
            }
        } else if (*eap).addr_count > 0 as ::core::ffi::c_int {
            if unaccept_arg0 != 0 && (*eap).line2 == 0 as linenr_T {
                (*eap).errmsg = gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                tab_number = 0 as ::core::ffi::c_int;
            } else {
                tab_number = (*eap).line2 as ::core::ffi::c_int;
                if unaccept_arg0 == 0 {
                    let mut cmdp: *mut ::core::ffi::c_char = (*eap).cmd;
                    loop {
                        cmdp = cmdp.offset(-1);
                        if !(cmdp > *(*eap).cmdlinep
                            && (ascii_iswhite(*cmdp as ::core::ffi::c_int) as ::core::ffi::c_int
                                != 0
                                || ascii_isdigit(*cmdp as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0))
                        {
                            break;
                        }
                    }
                    if *cmdp as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                        tab_number -= 1;
                        if tab_number < unaccept_arg0 {
                            (*eap).errmsg =
                                gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                        }
                    }
                }
            }
        } else {
            match (*eap).cmdidx as ::core::ffi::c_int {
                461 => {
                    tab_number = tabpage_index(curtab.get()) + 1 as ::core::ffi::c_int;
                    if tab_number > current_tab_nr(::core::ptr::null_mut::<tabpage_T>()) {
                        tab_number = 1 as ::core::ffi::c_int;
                    }
                }
                459 => {
                    tab_number = current_tab_nr(::core::ptr::null_mut::<tabpage_T>());
                }
                _ => {
                    tab_number = tabpage_index(curtab.get());
                }
            }
        }
    }
    return tab_number;
}
unsafe extern "C" fn ex_autocmd(mut eap: *mut exarg_T) {
    if secure.get() != 0 {
        secure.set(2 as ::core::ffi::c_int);
        (*eap).errmsg = gettext(&raw const e_curdir as *const ::core::ffi::c_char);
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_autocmd as ::core::ffi::c_int {
        do_autocmd(eap, (*eap).arg, (*eap).forceit);
    } else {
        do_augroup((*eap).arg, (*eap).forceit != 0);
    };
}
unsafe extern "C" fn ex_doautocmd(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut call_do_modelines: ::core::ffi::c_int =
        check_nomodeline(&raw mut arg) as ::core::ffi::c_int;
    let mut did_aucmd: bool = false;
    do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
    if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
        do_modelines(0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn ex_bunload(mut eap: *mut exarg_T) {
    (*eap).errmsg = do_bufdel(
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_bdelete as ::core::ffi::c_int {
            DOBUF_DEL as ::core::ffi::c_int
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_bwipeout as ::core::ffi::c_int {
            DOBUF_WIPE as ::core::ffi::c_int
        } else {
            DOBUF_UNLOAD as ::core::ffi::c_int
        },
        (*eap).arg,
        (*eap).addr_count,
        (*eap).line1 as ::core::ffi::c_int,
        (*eap).line2 as ::core::ffi::c_int,
        (*eap).forceit,
    );
}
unsafe extern "C" fn ex_buffer(mut eap: *mut exarg_T) {
    do_exbuffer(eap);
}
unsafe extern "C" fn do_exbuffer(mut eap: *mut exarg_T) {
    if *(*eap).arg != 0 {
        (*eap).errmsg = ex_errmsg(
            &raw const e_trailing_arg as *const ::core::ffi::c_char,
            (*eap).arg,
        );
    } else {
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            goto_buffer(
                eap,
                DOBUF_CURRENT as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            goto_buffer(
                eap,
                DOBUF_FIRST as ::core::ffi::c_int,
                FORWARD as ::core::ffi::c_int,
                (*eap).line2 as ::core::ffi::c_int,
            );
        }
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
    };
}
unsafe extern "C" fn ex_bmodified(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_MOD as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*eap).line2 as ::core::ffi::c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_bnext(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        (*eap).line2 as ::core::ffi::c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_bprevious(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_CURRENT as ::core::ffi::c_int,
        BACKWARD as ::core::ffi::c_int,
        (*eap).line2 as ::core::ffi::c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_brewind(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
unsafe extern "C" fn ex_blast(mut eap: *mut exarg_T) {
    goto_buffer(
        eap,
        DOBUF_LAST as ::core::ffi::c_int,
        BACKWARD as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if !(*eap).do_ecmd_cmd.is_null() {
        do_cmdline_cmd((*eap).do_ecmd_cmd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ends_excmd(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return (c == NUL
        || c == '|' as ::core::ffi::c_int
        || c == '"' as ::core::ffi::c_int
        || c == '\n' as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn find_nextcmd(
    mut p: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        if *p as ::core::ffi::c_int == NUL {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        p = p.offset(1);
    }
    return (p as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn check_nextcmd(
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = skipwhite(p);
    if *s as ::core::ffi::c_int == '|' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
    {
        return s.offset(1 as ::core::ffi::c_int as isize);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn check_more(mut message: bool, mut forceit: bool) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = (*(*curwin.get()).w_alist).al_ga.ga_len
        - (*curwin.get()).w_arg_idx
        - 1 as ::core::ffi::c_int;
    if !forceit
        && only_one_window() as ::core::ffi::c_int != 0
        && (*(*curwin.get()).w_alist).al_ga.ga_len > 1 as ::core::ffi::c_int
        && !arg_had_last.get()
        && n > 0 as ::core::ffi::c_int
        && quitmore.get() == 0 as ::core::ffi::c_int
    {
        if message {
            if (p_confirm.get() != 0
                || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
                && !(*curbuf.get()).b_fname.is_null()
            {
                let mut buff: [::core::ffi::c_char; 1000] = [0; 1000];
                vim_snprintf(
                    &raw mut buff as *mut ::core::ffi::c_char,
                    DIALOG_MSG_SIZE as ::core::ffi::c_int as size_t,
                    ngettext(
                        b"%d more file to edit.  Quit anyway?\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"%d more files to edit.  Quit anyway?\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        n as ::core::ffi::c_ulong,
                    ),
                    n,
                );
                if vim_dialog_yesno(
                    VIM_QUESTION as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut buff as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                ) == VIM_YES as ::core::ffi::c_int
                {
                    return OK;
                }
                return FAIL;
            }
            semsg(
                ngettext(
                    b"E173: %d more file to edit\0".as_ptr() as *const ::core::ffi::c_char,
                    b"E173: %d more files to edit\0".as_ptr() as *const ::core::ffi::c_char,
                    n as ::core::ffi::c_ulong,
                ),
                n,
            );
            quitmore.set(2 as ::core::ffi::c_int);
        }
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn get_command_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx >= CMD_SIZE as ::core::ffi::c_int {
        return expand_user_command_name(idx);
    }
    return (*cmdnames.ptr())[idx as usize].cmd_name;
}
unsafe extern "C" fn ex_colorscheme(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        let mut expr: *mut ::core::ffi::c_char =
            xstrdup(b"g:colors_name\0".as_ptr() as *const ::core::ffi::c_char);
        (*emsg_off.ptr()) += 1;
        let mut p: *mut ::core::ffi::c_char = eval_to_string(expr, false_0 != 0, false_0 != 0);
        (*emsg_off.ptr()) -= 1;
        xfree(expr as *mut ::core::ffi::c_void);
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        if !p.is_null() {
            msg(p, 0 as ::core::ffi::c_int);
            xfree(p as *mut ::core::ffi::c_void);
        } else {
            msg(
                b"default\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else if load_colors((*eap).arg) == FAIL {
        semsg(
            gettext(b"E185: Cannot find color scheme '%s'\0".as_ptr() as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
}
unsafe extern "C" fn ex_highlight(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL
        && *(*eap).cmd.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '!' as ::core::ffi::c_int
    {
        msg(
            gettext(b"Greetings, Vim user!\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    }
    do_highlight((*eap).arg, (*eap).forceit != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn not_exiting(mut save_exiting: bool) {
    exiting.set(save_exiting);
    set_vim_var_string(
        VV_EXITREASON,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn before_quit_autocmds(
    mut wp: *mut win_T,
    mut quit_all: bool,
    mut forceit: bool,
) -> bool {
    if *get_vim_var_str(VV_EXITREASON) as ::core::ffi::c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
    }
    apply_autocmds(
        EVENT_QUITPRE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        (*wp).w_buffer,
    );
    if !win_valid(wp)
        || curbuf_locked() as ::core::ffi::c_int != 0
        || (*(*wp).w_buffer).b_nwindows == 1 as ::core::ffi::c_int
            && (*(*wp).w_buffer).b_locked > 0 as ::core::ffi::c_int
    {
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        return true_0 != 0;
    }
    if quit_all as ::core::ffi::c_int != 0
        || check_more(false_0 != 0, forceit) == OK && only_one_window() as ::core::ffi::c_int != 0
    {
        apply_autocmds(
            EVENT_EXITPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if !win_valid(wp)
            || curbuf_locked() as ::core::ffi::c_int != 0
            || (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                && (*curbuf.get()).b_locked > 0 as ::core::ffi::c_int
        {
            set_vim_var_string(
                VV_EXITREASON,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ptrdiff_t,
            );
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn ex_quit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            wp = (*wp).w_next;
        }
    } else {
        wp = curwin.get();
    }
    if curbuf_locked() {
        return;
    }
    if before_quit_autocmds(wp, false_0 != 0, (*eap).forceit != 0) {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK
        && only_one_window() as ::core::ffi::c_int != 0
    {
        exiting.set(true_0 != 0);
    }
    if !buf_hide((*wp).w_buffer)
        && check_changed(
            (*wp).w_buffer,
            (if p_awa.get() != 0 {
                CCGD_AW as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | CCGD_EXCMD as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as ::core::ffi::c_int != 0
            && check_changed_any((*eap).forceit != 0, true_0 != 0) as ::core::ffi::c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() as ::core::ffi::c_int != 0
            && (firstwin.get() == lastwin.get() || (*eap).addr_count == 0 as ::core::ffi::c_int)
        {
            getout(0 as ::core::ffi::c_int);
        }
        not_exiting(save_exiting);
        win_close(
            wp,
            !buf_hide((*wp).w_buffer) || (*eap).forceit != 0,
            (*eap).forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_cquit(mut eap: *mut exarg_T) -> ! {
    let mut status: ::core::ffi::c_int = if (*eap).addr_count > 0 as ::core::ffi::c_int {
        (*eap).line2 as ::core::ffi::c_int
    } else {
        EXIT_FAILURE
    };
    ui_call_error_exit(status as Integer);
    getout(status);
}
#[no_mangle]
pub unsafe extern "C" fn before_quit_all(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(if (*eap).forceit != 0 {
            -(253 as ::core::ffi::c_int
                + ((KE_XF1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        } else {
            -(253 as ::core::ffi::c_int
                + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        });
        return FAIL;
    }
    if text_locked() {
        text_locked_msg();
        return FAIL;
    }
    if before_quit_autocmds(curwin.get(), true_0 != 0, (*eap).forceit != 0) {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn ex_quitall(mut eap: *mut exarg_T) {
    if before_quit_all(eap) == FAIL {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    exiting.set(true_0 != 0);
    if (*eap).forceit != 0 || !check_changed_any(false_0 != 0, false_0 != 0) {
        getout(0 as ::core::ffi::c_int);
    }
    not_exiting(save_exiting);
}
unsafe extern "C" fn ex_restart(mut eap: *mut exarg_T) {
    let mut servername_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut servername_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut result: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    };
    let mut listen_addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut quit_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut quit_cmd_copy: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut result_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
    let mut detach_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut detach_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut chanclose_expr_args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chanclose_expr_args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let no_ui: bool = ui_active() == 0;
    let mut exepath: *const ::core::ffi::c_char = get_vim_var_str(VV_PROGPATH);
    let mut l: *const list_T = get_vim_var_list(VV_ARGV);
    let mut argc: ::core::ffi::c_int = tv_list_len(l);
    let mut argv: *mut *mut ::core::ffi::c_char = xcalloc(
        (argc as size_t).wrapping_add(3 as size_t),
        ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: size_t = 0 as size_t;
    let mut listen_arg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut li: *const listitem_T = (*l).lv_first;
    while !li.is_null() {
        let mut arg: *const ::core::ffi::c_char = tv_get_string(&raw const (*li).li_tv);
        if i > 0 as size_t
            && strequal(arg, b"--\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
                != 0
        {
            break;
        }
        if i > 0 as size_t
            && strequal(arg, b"-s\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
                != 0
        {
            li = (*li).li_next;
        } else {
            if i > 0 as size_t
                && strequal(arg, b"--listen\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int
                    != 0
            {
                let mut next_li: *const listitem_T = (*li).li_next;
                if !next_li.is_null() {
                    let mut addr: *const ::core::ffi::c_char =
                        tv_get_string(&raw const (*next_li).li_tv);
                    if !strstr(addr, b":\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                        || !strstr(addr, b"/\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                        || !strstr(addr, b"\\\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                    {
                        listen_arg = addr;
                    }
                }
            }
            if i == 0 as size_t
                || !strequal(arg, b"--embed\0".as_ptr() as *const ::core::ffi::c_char)
                    && !strequal(arg, b"--headless\0".as_ptr() as *const ::core::ffi::c_char)
                    && !strequal(arg, b"-\0".as_ptr() as *const ::core::ffi::c_char)
            {
                let c2rust_fresh4 = i;
                i = i.wrapping_add(1);
                let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = xstrdup(arg);
                if i == 1 as size_t {
                    let c2rust_fresh5 = i;
                    i = i.wrapping_add(1);
                    let c2rust_lvalue_ptr_0 = &raw mut *argv.offset(c2rust_fresh5 as isize);
                    *c2rust_lvalue_ptr_0 =
                        xstrdup(b"--embed\0".as_ptr() as *const ::core::ffi::c_char);
                    if no_ui {
                        let c2rust_fresh6 = i;
                        i = i.wrapping_add(1);
                        let c2rust_lvalue_ptr_1 = &raw mut *argv.offset(c2rust_fresh6 as isize);
                        *c2rust_lvalue_ptr_1 =
                            xstrdup(b"--headless\0".as_ptr() as *const ::core::ffi::c_char);
                    }
                }
            }
        }
        li = (*li).li_next;
    }
    let mut server_stopped: bool = if !listen_arg.is_null() {
        server_stop(listen_arg, true_0 != 0) as ::core::ffi::c_int
    } else {
        false_0
    } != 0;
    let mut on_err: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    on_err.fwd_err = true_0 != 0;
    let mut detach: bool = true_0 != 0;
    let mut exit_status: varnumber_T = 0;
    let mut channel: *mut Channel = channel_job_start(
        argv,
        exepath,
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_20 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            self_0: ::core::ptr::null_mut::<dict_T>(),
            buffer: GA_EMPTY_INIT_VALUE,
            eof: false,
            buffered: false_0 != 0,
            fwd_err: false_0 != 0,
            type_0: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        on_err,
        Callback {
            data: C2Rust_Unnamed_20 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        detach,
        kChannelStdinPipe,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as uint16_t,
        0 as uint16_t,
        ::core::ptr::null_mut::<dict_T>(),
        &raw mut exit_status,
    );
    if channel.is_null() {
        emsg(b"cannot create a channel job\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        detach_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        detach_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        detach_args.capacity = 1 as size_t;
        detach_args.items = &raw mut detach_args__items as *mut Object;
        let c2rust_fresh7 = detach_args.size;
        detach_args.size = detach_args.size.wrapping_add(1);
        *detach_args.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed_14 { boolean: true },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim__chan_set_detach\0".as_ptr() as *const ::core::ffi::c_char,
            detach_args,
            &raw mut result_mem,
            &raw mut err,
        );
        '_fail_2: {
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                arena_mem_free(result_mem);
                result_mem = ::core::ptr::null_mut::<consumed_blk>();
                if *(*eap).arg as ::core::ffi::c_int != NUL {
                    let mut autocmd_opts: Dict = Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    };
                    let mut autocmd_opts__items: [KeyValuePair; 3] = [KeyValuePair {
                        key: String_0 {
                            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            size: 0,
                        },
                        value: Object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed_14 { boolean: false },
                        },
                    }; 3];
                    autocmd_opts.capacity = 3 as size_t;
                    autocmd_opts.items = &raw mut autocmd_opts__items as *mut KeyValuePair;
                    let c2rust_fresh8 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                        key: cstr_as_string(b"once\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh9 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                        key: cstr_as_string(b"nested\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeBoolean,
                            data: C2Rust_Unnamed_14 { boolean: true },
                        },
                    };
                    let c2rust_fresh10 = autocmd_opts.size;
                    autocmd_opts.size = autocmd_opts.size.wrapping_add(1);
                    *autocmd_opts.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                        key: cstr_as_string(b"command\0".as_ptr() as *const ::core::ffi::c_char),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed_14 {
                                string: cstr_as_string((*eap).arg),
                            },
                        },
                    };
                    let mut autocmd_args: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut autocmd_args__items: [Object; 2] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_14 { boolean: false },
                    }; 2];
                    autocmd_args.capacity = 2 as size_t;
                    autocmd_args.items = &raw mut autocmd_args__items as *mut Object;
                    let c2rust_fresh11 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh11 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: cstr_as_string(
                                b"UIEnter\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                        },
                    };
                    let c2rust_fresh12 = autocmd_args.size;
                    autocmd_args.size = autocmd_args.size.wrapping_add(1);
                    *autocmd_args.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed_14 { dict: autocmd_opts },
                    };
                    rpc_send_call(
                        (*channel).id,
                        b"nvim_create_autocmd\0".as_ptr() as *const ::core::ffi::c_char,
                        autocmd_args,
                        &raw mut result_mem,
                        &raw mut err,
                    );
                    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_fail_2;
                    } else {
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                    }
                }
                servername_args = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                servername_args__items = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_14 { boolean: false },
                }; 1];
                servername_args.capacity = 1 as size_t;
                servername_args.items = &raw mut servername_args__items as *mut Object;
                let c2rust_fresh13 = servername_args.size;
                servername_args.size = servername_args.size.wrapping_add(1);
                *servername_args.items.offset(c2rust_fresh13 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(
                            b"servername\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                    },
                };
                result = rpc_send_call(
                    (*channel).id,
                    b"nvim_get_vvar\0".as_ptr() as *const ::core::ffi::c_char,
                    servername_args,
                    &raw mut result_mem,
                    &raw mut err,
                );
                if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                    if result.type_0 as ::core::ffi::c_uint
                        != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        || result.data.string.size == 0 as size_t
                    {
                        emsg(
                            b"restart failed: could not get listen address from new server\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else {
                        listen_addr = xmemdupz(
                            result.data.string.data as *const ::core::ffi::c_void,
                            result.data.string.size,
                        ) as *mut ::core::ffi::c_char;
                        arena_mem_free(result_mem);
                        result_mem = ::core::ptr::null_mut::<consumed_blk>();
                        ui_call_restart(cstr_as_string(listen_addr));
                        ui_flush();
                        xfree(listen_addr as *mut ::core::ffi::c_void);
                        set_vim_var_string(
                            VV_EXITREASON,
                            b"restart\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as usize) as ptrdiff_t,
                        );
                        quit_cmd = (if !(*eap).do_ecmd_cmd.is_null() {
                            (*eap).do_ecmd_cmd as *const ::core::ffi::c_char
                        } else {
                            b"qall\0".as_ptr() as *const ::core::ffi::c_char
                        }) as *mut ::core::ffi::c_char;
                        quit_cmd_copy = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0 {
                            quit_cmd_copy = concat_str(
                                b"confirm \0".as_ptr() as *const ::core::ffi::c_char,
                                quit_cmd,
                            );
                            quit_cmd = quit_cmd_copy;
                        }
                        nvim_command(cstr_as_string(quit_cmd), &raw mut err);
                        xfree(quit_cmd_copy as *mut ::core::ffi::c_void);
                        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                        {
                            emsg(err.msg);
                            api_clear_error(&raw mut err);
                        } else if !exiting.get() {
                            emsg(b"restart failed: +cmd did not quit the server\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        }
                    }
                }
            }
        }
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            emsg(err.msg);
            api_clear_error(&raw mut err);
        }
        arena_mem_free(result_mem);
        result_mem = ::core::ptr::null_mut::<consumed_blk>();
        chanclose_expr_args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        chanclose_expr_args__items = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_14 { boolean: false },
        }; 1];
        chanclose_expr_args.capacity = 1 as size_t;
        chanclose_expr_args.items = &raw mut chanclose_expr_args__items as *mut Object;
        let c2rust_fresh14 = chanclose_expr_args.size;
        chanclose_expr_args.size = chanclose_expr_args.size.wrapping_add(1);
        *chanclose_expr_args.items.offset(c2rust_fresh14 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_14 {
                string: cstr_as_string(
                    b"chanclose(v:stderr)\0".as_ptr() as *const ::core::ffi::c_char
                ),
            },
        };
        rpc_send_call(
            (*channel).id,
            b"nvim_eval\0".as_ptr() as *const ::core::ffi::c_char,
            chanclose_expr_args,
            &raw mut result_mem,
            &raw mut err,
        );
        api_clear_error(&raw mut err);
        arena_mem_free(result_mem);
        proc_stop(&raw mut (*channel).stream.proc);
        if proc_wait(
            &raw mut (*channel).stream.proc,
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<MultiQueue>(),
        ) < 0 as ::core::ffi::c_int
        {
            emsg(b"killing new nvim server failed\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    if server_stopped as ::core::ffi::c_int != 0
        && server_start(listen_arg) != 0 as ::core::ffi::c_int
    {
        semsg(
            b"couldn't resume listening on %s\0".as_ptr() as *const ::core::ffi::c_char,
            listen_arg,
        );
    }
}
unsafe extern "C" fn ex_close(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut winnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(Ctrl_C);
    } else if !text_locked() && !curbuf_locked() {
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            ex_win_close(
                (*eap).forceit,
                curwin.get(),
                ::core::ptr::null_mut::<tabpage_T>(),
            );
        } else {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                winnr += 1;
                if winnr as linenr_T == (*eap).line2 {
                    win = wp;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
            if win.is_null() {
                win = lastwin.get();
            }
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
        }
    }
}
unsafe extern "C" fn ex_pclose(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_onebuf_opt.wo_pvw != 0 {
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
            break;
        } else {
            win = (*win).w_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_win_close(
    mut forceit: ::core::ffi::c_int,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
) {
    if is_aucmd_win(win) {
        emsg(gettext(
            &raw const e_autocmd_close as *const ::core::ffi::c_char,
        ));
        return;
    }
    if !(*win).w_floating && window_layout_locked(CMD_close) as ::core::ffi::c_int != 0 {
        return;
    }
    let mut buf: *mut buf_T = (*win).w_buffer;
    let mut need_hide: bool = bufIsChanged(buf) as ::core::ffi::c_int != 0
        && (*buf).b_nwindows <= 1 as ::core::ffi::c_int;
    if need_hide as ::core::ffi::c_int != 0 && !buf_hide(buf) && forceit == 0 {
        if (p_confirm.get() != 0
            || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0)
            && p_write.get() != 0
        {
            let mut bufref: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, buf);
            dialog_changed(buf, false_0 != 0);
            if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && bufIsChanged(buf) as ::core::ffi::c_int != 0
            {
                return;
            }
            need_hide = false_0 != 0;
        } else {
            no_write_message();
            return;
        }
    }
    if tp.is_null() {
        win_close(win, !need_hide && !buf_hide(buf), forceit != 0);
    } else {
        win_close_othertab(
            win,
            (!need_hide && !buf_hide(buf)) as ::core::ffi::c_int,
            tp,
            forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_tabclose(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(
            -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        emsg(gettext(
            b"E784: Cannot close last tab page\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    let mut tab_number: ::core::ffi::c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    let mut tp: *mut tabpage_T = find_tabpage(tab_number);
    if tp.is_null() {
        beep_flush();
        return;
    }
    if tp != curtab.get() {
        tabpage_close_other(tp, (*eap).forceit);
        return;
    } else if !text_locked() && !curbuf_locked() {
        tabpage_close((*eap).forceit);
    }
}
unsafe extern "C" fn ex_tabonly(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(
            -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        msg(
            gettext(b"Already only one tab page\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
        return;
    }
    if window_layout_locked(CMD_tabonly) {
        return;
    }
    let mut tab_number: ::core::ffi::c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    goto_tabpage(tab_number);
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while done < 1000 as ::core::ffi::c_int {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if (*tp).tp_topframe != topframe.get() {
                tabpage_close_other(tp as *mut tabpage_T, (*eap).forceit);
                if valid_tabpage(tp as *mut tabpage_T) {
                    done = 1000 as ::core::ffi::c_int;
                }
                break;
            } else {
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        '_c2rust_label: {
            if !(*first_tabpage.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"first_tabpage\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    5361 as ::core::ffi::c_uint,
                    b"void ex_tabonly(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*first_tabpage.get()).tp_next.is_null() {
            break;
        }
        done += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tabpage_close(mut forceit: ::core::ffi::c_int) {
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    trigger_tabclosedpre(curtab.get());
    (*curtab.get()).tp_did_tabclosedpre = true_0 != 0;
    let save_curtab: *mut tabpage_T = curtab.get();
    while (*curwin.get()).w_floating {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if !(firstwin.get() == lastwin.get()) {
        close_others(true_0, forceit);
    }
    if firstwin.get() == lastwin.get() {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if curtab.get() == save_curtab {
        (*curtab.get()).tp_did_tabclosedpre = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tabpage_close_other(
    mut tp: *mut tabpage_T,
    mut forceit: ::core::ffi::c_int,
) {
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prev_idx: [::core::ffi::c_char; 65] = [0; 65];
    if window_layout_locked(CMD_SIZE) {
        return;
    }
    trigger_tabclosedpre(tp);
    (*tp).tp_did_tabclosedpre = true_0 != 0;
    loop {
        done += 1;
        if done >= 1000 as ::core::ffi::c_int {
            break;
        }
        snprintf(
            &raw mut prev_idx as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
            b"%i\0".as_ptr() as *const ::core::ffi::c_char,
            tabpage_index(tp),
        );
        let mut wp: *mut win_T = (*tp).tp_lastwin;
        ex_win_close(forceit, wp, tp);
        if !valid_tabpage(tp) {
            break;
        }
        if (*tp).tp_lastwin != wp {
            continue;
        }
        done = 1000 as ::core::ffi::c_int;
        break;
    }
    if done >= 1000 as ::core::ffi::c_int {
        (*tp).tp_did_tabclosedpre = false_0 != 0;
        return;
    }
}
unsafe extern "C" fn ex_only(mut eap: *mut exarg_T) {
    if window_layout_locked(CMD_only) {
        return;
    }
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        loop {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            if (*wp).w_next.is_null() {
                break;
            }
            wp = (*wp).w_next;
        }
        if wp != curwin.get() {
            win_goto(wp);
        }
    }
    close_others(true_0, (*eap).forceit);
}
unsafe extern "C" fn ex_hide(mut eap: *mut exarg_T) {
    if (*eap).skip != 0 {
        return;
    }
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        win = curwin.get();
    } else {
        let mut winnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            winnr += 1;
            if winnr as linenr_T == (*eap).line2 {
                win = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if win.is_null() {
            win = lastwin.get();
        }
    }
    if !(*win).w_floating && window_layout_locked(CMD_hide) as ::core::ffi::c_int != 0 {
        return;
    }
    win_close(win, false_0 != 0, (*eap).forceit != 0);
}
unsafe extern "C" fn ex_stop(mut eap: *mut exarg_T) {
    if (*eap).forceit == 0 {
        autowrite_all();
    }
    may_trigger_vim_suspend_resume(true_0 != 0);
    ui_call_suspend();
    ui_flush();
}
unsafe extern "C" fn ex_exit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK
        && only_one_window() as ::core::ffi::c_int != 0
    {
        exiting.set(true_0 != 0);
    }
    if ((*eap).cmdidx as ::core::ffi::c_int == CMD_wq as ::core::ffi::c_int
        || curbufIsChanged() as ::core::ffi::c_int != 0)
        && do_write(eap) == FAIL
        || before_quit_autocmds(curwin.get(), false_0 != 0, (*eap).forceit != 0)
            as ::core::ffi::c_int
            != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as ::core::ffi::c_int != 0
            && check_changed_any((*eap).forceit != 0, false_0 != 0) as ::core::ffi::c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() {
            getout(0 as ::core::ffi::c_int);
        }
        not_exiting(save_exiting);
        win_close(
            curwin.get(),
            !buf_hide((*curwin.get()).w_buffer),
            (*eap).forceit != 0,
        );
    };
}
unsafe extern "C" fn ex_print(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        emsg(gettext(
            &raw const e_empty_buffer as *const ::core::ffi::c_char,
        ));
    } else {
        let mut line: linenr_T = (*eap).line1;
        while line <= (*eap).line2 && !got_int.get() {
            print_line(
                line,
                (*eap).cmdidx as ::core::ffi::c_int == CMD_number as ::core::ffi::c_int
                    || (*eap).cmdidx as ::core::ffi::c_int == CMD_pound as ::core::ffi::c_int
                    || (*eap).flags & EXFLAG_NR != 0,
                (*eap).cmdidx as ::core::ffi::c_int == CMD_list as ::core::ffi::c_int
                    || (*eap).flags & EXFLAG_LIST != 0,
                line == (*eap).line1,
            );
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
    ex_no_reprint.set(true_0 != 0);
}
unsafe extern "C" fn ex_goto(mut eap: *mut exarg_T) {
    goto_byte((*eap).line2 as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_preserve(mut _eap: *mut exarg_T) {
    ml_preserve(curbuf.get(), true_0 != 0, true_0 != 0);
}
unsafe extern "C" fn ex_recover(mut eap: *mut exarg_T) {
    recoverymode.set(true_0 != 0);
    if !check_changed(
        curbuf.get(),
        (if p_awa.get() != 0 {
            CCGD_AW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | CCGD_MULTWIN as ::core::ffi::c_int
            | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            })
            | CCGD_EXCMD as ::core::ffi::c_int,
    ) && (*(*eap).arg as ::core::ffi::c_int == NUL
        || setfname(
            curbuf.get(),
            (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
        ) == OK)
    {
        ml_recover(true_0 != 0);
    }
    recoverymode.set(false_0 != 0);
}
unsafe extern "C" fn ex_wrongmodifier(mut eap: *mut exarg_T) {
    (*eap).errmsg = gettext(&raw const e_invcmd as *const ::core::ffi::c_char);
}
static ffu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_20 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
unsafe extern "C" fn get_findfunc_callback() -> *mut Callback {
    return if *(*curbuf.get()).b_p_ffu as ::core::ffi::c_int != NUL {
        &raw mut (*curbuf.get()).b_ffu_cb
    } else {
        ffu_cb.ptr()
    };
}
unsafe extern "C" fn call_findfunc(
    mut pat: *mut ::core::ffi::c_char,
    mut cmdcomplete: BoolVarValue,
) -> *mut list_T {
    let saved_sctx: sctx_T = current_sctx.get();
    let mut args: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_BOOL;
    args[1 as ::core::ffi::c_int as usize].vval.v_bool = cmdcomplete;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    (*textlock.ptr()) += 1;
    let mut ctx: *mut sctx_T = get_option_sctx(kOptFindfunc);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut cb: *mut Callback = get_findfunc_callback();
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: ::core::ffi::c_int = callback_call(
        cb,
        2 as ::core::ffi::c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) as ::core::ffi::c_int;
    current_sctx.set(saved_sctx);
    (*textlock.ptr()) -= 1;
    let mut retlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if retval == OK {
        if rettv.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            retlist = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                rettv.vval.v_list,
                false_0 != 0,
                get_copyID(),
            );
        } else {
            emsg(gettext(
                &raw const e_invalid_return_type_from_findfunc as *const ::core::ffi::c_char,
            ));
        }
        tv_clear(&raw mut rettv);
    }
    return retlist;
}
#[no_mangle]
pub unsafe extern "C" fn expand_findfunc(
    mut pat: *mut ::core::ffi::c_char,
    mut files: *mut *mut *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    *numMatches = 0 as ::core::ffi::c_int;
    *files = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut l: *mut list_T = call_findfunc(pat, kBoolVarTrue);
    if l.is_null() {
        return FAIL;
    }
    let mut len: ::core::ffi::c_int = tv_list_len(l);
    if len == 0 as ::core::ffi::c_int {
        tv_list_free(l);
        return FAIL;
    }
    *files = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(len as size_t))
        as *mut *mut ::core::ffi::c_char;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *(*files).offset(idx as isize) = xstrdup((*li).li_tv.vval.v_string);
                idx += 1;
            }
            li = (*li).li_next;
        }
    }
    *numMatches = idx;
    tv_list_free(l);
    return OK;
}
unsafe extern "C" fn findfunc_find_file(
    mut findarg: *mut ::core::ffi::c_char,
    mut findarg_len: size_t,
    mut count: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut ret_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let cc: ::core::ffi::c_char = *findarg.offset(findarg_len as isize);
    *findarg.offset(findarg_len as isize) = NUL as ::core::ffi::c_char;
    let mut fname_list: *mut list_T = call_findfunc(findarg, kBoolVarFalse);
    let mut fname_count: ::core::ffi::c_int = tv_list_len(fname_list);
    if fname_count == 0 as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_cant_find_file_str_in_path as *const ::core::ffi::c_char),
            findarg,
        );
    } else if count > fname_count {
        semsg(
            gettext(&raw const e_no_more_file_str_found_in_path as *const ::core::ffi::c_char),
            findarg,
        );
    } else {
        let mut li: *mut listitem_T = tv_list_find(fname_list, count - 1 as ::core::ffi::c_int);
        if !li.is_null()
            && (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ret_fname = xstrdup((*li).li_tv.vval.v_string);
        }
    }
    if !fname_list.is_null() {
        tv_list_free(fname_list);
    }
    *findarg.offset(findarg_len as isize) = cc;
    return ret_fname;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_findfunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*buf).b_p_ffu, &raw mut (*buf).b_ffu_cb);
    } else {
        retval = option_set_callback_func(p_ffu.get(), ffu_cb.ptr());
        if (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            callback_free(&raw mut (*buf).b_ffu_cb);
        }
    }
    if retval == FAIL {
        return &raw const e_invarg as *const ::core::ffi::c_char;
    }
    let mut varp: *mut *mut ::core::ffi::c_char = (*args).os_varp as *mut *mut ::core::ffi::c_char;
    let mut name: *mut ::core::ffi::c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn free_findfunc_option() {
    callback_free(ffu_cb.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_findfunc(mut copyID: ::core::ffi::c_int) -> bool {
    let mut abort_0: bool = false_0 != 0;
    abort_0 = set_ref_in_callback(
        ffu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
    return abort_0;
}
#[no_mangle]
pub unsafe extern "C" fn ex_splitview(mut eap: *mut exarg_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let use_tab: bool = (*eap).cmdidx as ::core::ffi::c_int == CMD_tabedit as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabfind as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabnew as ::core::ffi::c_int;
    if bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
        && (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int
    {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_split as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_new;
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_vsplit as ::core::ffi::c_int {
            (*eap).cmdidx = CMD_vnew;
        }
    }
    '_theend: {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_sfind as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabfind as ::core::ffi::c_int
        {
            if *get_findfunc() as ::core::ffi::c_int != NUL {
                fname = findfunc_find_file(
                    (*eap).arg,
                    strlen((*eap).arg),
                    if (*eap).addr_count > 0 as ::core::ffi::c_int {
                        (*eap).line2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    },
                );
            } else {
                let mut file_to_find: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut search_ctx: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                fname = find_file_in_path(
                    (*eap).arg,
                    strlen((*eap).arg),
                    FNAME_MESS as ::core::ffi::c_int,
                    true_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
                xfree(file_to_find as *mut ::core::ffi::c_void);
                vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
            }
            if fname.is_null() {
                break '_theend;
            } else {
                (*eap).arg = fname;
            }
        }
        if use_tab {
            if !win_new_tabpage(
                if (*cmdmod.ptr()).cmod_tab != 0 as ::core::ffi::c_int {
                    (*cmdmod.ptr()).cmod_tab
                } else if (*eap).addr_count == 0 as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    (*eap).line2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                },
                (*eap).arg,
                true_0 != 0,
                ::core::ptr::null_mut::<*mut win_T>(),
            )
            .is_null()
            {
                do_exedit(eap, old_curwin);
                apply_autocmds(
                    EVENT_TABNEWENTERED,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                if curwin.get() != old_curwin
                    && win_valid(old_curwin) as ::core::ffi::c_int != 0
                    && (*old_curwin).w_buffer != curbuf.get()
                    && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                {
                    (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
                }
            }
        } else if win_split(
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            if *(*eap).cmd as ::core::ffi::c_int == 'v' as ::core::ffi::c_int {
                WSP_VERT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) != FAIL
        {
            if *(*eap).arg as ::core::ffi::c_int != NUL {
                (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
                (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
            } else {
                do_check_scrollbind(false_0 != 0);
            }
            do_exedit(eap, old_curwin);
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tabpage_new() {
    let mut ea: exarg_T = exarg {
        arg: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: b"tabn\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_tabnew,
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
    ex_splitview(&raw mut ea);
}
unsafe extern "C" fn ex_tabnext(mut eap: *mut exarg_T) {
    let mut tab_number: ::core::ffi::c_int = 0;
    match (*eap).cmdidx as ::core::ffi::c_int {
        458 | 466 => {
            goto_tabpage(1 as ::core::ffi::c_int);
        }
        460 => {
            goto_tabpage(9999 as ::core::ffi::c_int);
        }
        464 | 465 => {
            if !(*eap).arg.is_null() && *(*eap).arg as ::core::ffi::c_int != NUL {
                let mut p: *mut ::core::ffi::c_char = (*eap).arg;
                let mut p_save: *mut ::core::ffi::c_char = p;
                tab_number =
                    getdigits(&raw mut p, false_0 != 0, 0 as intmax_t) as ::core::ffi::c_int;
                if p == p_save
                    || *p_save as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    || *p_save as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int != NUL
                    || tab_number == 0 as ::core::ffi::c_int
                {
                    (*eap).errmsg = ex_errmsg(
                        &raw const e_invarg2 as *const ::core::ffi::c_char,
                        (*eap).arg,
                    );
                    return;
                }
            } else if (*eap).addr_count == 0 as ::core::ffi::c_int {
                tab_number = 1 as ::core::ffi::c_int;
            } else {
                tab_number = (*eap).line2 as ::core::ffi::c_int;
                if tab_number < 1 as ::core::ffi::c_int {
                    (*eap).errmsg = gettext(&raw const e_invrange as *const ::core::ffi::c_char);
                    return;
                }
            }
            goto_tabpage(-tab_number);
        }
        _ => {
            tab_number = get_tabpage_arg(eap);
            if (*eap).errmsg.is_null() {
                goto_tabpage(tab_number);
            }
        }
    };
}
unsafe extern "C" fn ex_tabmove(mut eap: *mut exarg_T) {
    let mut tab_number: ::core::ffi::c_int = get_tabpage_arg(eap);
    if (*eap).errmsg.is_null() {
        tabpage_move(tab_number);
    }
}
unsafe extern "C" fn ex_tabs(mut _eap: *mut exarg_T) {
    let mut tabcount: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    msg_scroll.set(true_0);
    let mut lastused_win: *mut win_T =
        if valid_tabpage(lastused_tabpage.get()) as ::core::ffi::c_int != 0 {
            (*lastused_tabpage.get()).tp_curwin
        } else {
            ::core::ptr::null_mut::<win_T>()
        };
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if got_int.get() {
            break;
        }
        if msg_col.get() > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        let c2rust_fresh1 = tabcount;
        tabcount = tabcount + 1;
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            gettext(b"Tab page %d\0".as_ptr() as *const ::core::ffi::c_char),
            c2rust_fresh1,
        );
        msg_outtrans(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        os_breakcheck();
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if got_int.get() {
                break;
            }
            if !(!(*wp).w_config.focusable || (*wp).w_config.hide as ::core::ffi::c_int != 0) {
                msg_putchar('\n' as ::core::ffi::c_int);
                msg_putchar(if wp == curwin.get() {
                    '>' as ::core::ffi::c_int
                } else if wp == lastused_win {
                    '#' as ::core::ffi::c_int
                } else {
                    ' ' as ::core::ffi::c_int
                });
                msg_putchar(' ' as ::core::ffi::c_int);
                msg_putchar(if bufIsChanged((*wp).w_buffer) as ::core::ffi::c_int != 0 {
                    '+' as ::core::ffi::c_int
                } else {
                    ' ' as ::core::ffi::c_int
                });
                msg_putchar(' ' as ::core::ffi::c_int);
                if !buf_spname((*wp).w_buffer).is_null() {
                    xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        buf_spname((*wp).w_buffer),
                        IOSIZE as size_t,
                    );
                } else {
                    home_replace(
                        (*wp).w_buffer,
                        (*(*wp).w_buffer).b_fname,
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        true_0 != 0,
                    );
                }
                msg_outtrans(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                os_breakcheck();
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn ex_detach(mut eap: *mut exarg_T) {
    if !eap.is_null() && (*eap).forceit != 0 {
        emsg(b"bang (!) not supported yet\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        if current_ui.get() == 0 {
            emsg(b"UI not attached\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        let mut chan: *mut Channel = find_channel(current_ui.get());
        if chan.is_null() {
            emsg(&raw const e_invchan as *const ::core::ffi::c_char);
            return;
        }
        let mut detach_err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        nvim__chan_set_detach((*chan).id, true_0 != 0, &raw mut detach_err);
        api_clear_error(&raw mut detach_err);
        let mut err2: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        remote_ui_disconnect((*chan).id, &raw mut err2, true_0 != 0);
        if err2.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            emsg(err2.msg);
            api_clear_error(&raw mut err2);
            return;
        }
        let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut rv: bool = channel_close((*chan).id, kChannelPartAll, &raw mut err);
        if !rv && !err.is_null() {
            emsg(err);
            return;
        }
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ex_detach\0".as_ptr() as *const ::core::ffi::c_char,
            6019 as ::core::ffi::c_int,
            true_0 != 0,
            b"detach current_ui=%ld\0".as_ptr() as *const ::core::ffi::c_char,
            (*chan).id,
        );
    };
}
unsafe extern "C" fn ex_connect(mut eap: *mut exarg_T) {
    let mut stop_server: bool = if (*eap).forceit != 0 {
        (ui_active() == 1 as size_t) as ::core::ffi::c_int
    } else {
        false_0
    } != 0;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    remote_ui_connect(current_ui.get(), (*eap).arg, &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg(err.msg);
        api_clear_error(&raw mut err);
        return;
    }
    ex_detach(::core::ptr::null_mut::<exarg_T>());
    if stop_server {
        exiting.set(true_0 != 0);
        getout(0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn ex_mode(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        must_redraw.set(UPD_CLEAR as ::core::ffi::c_int);
        ex_redraw(eap);
    } else {
        emsg(gettext(
            &raw const e_screenmode as *const ::core::ffi::c_char,
        ));
    };
}
unsafe extern "C" fn ex_resize(mut eap: *mut exarg_T) {
    let mut wp: *mut win_T = curwin.get();
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = (*eap).line2 as ::core::ffi::c_int;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() && {
            n -= 1;
            n > 0 as ::core::ffi::c_int
        } {
            wp = (*wp).w_next;
        }
    }
    let mut n_0: ::core::ffi::c_int = atol((*eap).arg) as ::core::ffi::c_int;
    if (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        if *(*eap).arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *(*eap).arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        {
            n_0 += (*wp).w_width;
        } else if n_0 == 0 as ::core::ffi::c_int
            && *(*eap).arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            n_0 = Columns.get();
        }
        win_setwidth_win(n_0, wp);
    } else {
        if *(*eap).arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *(*eap).arg as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        {
            n_0 += (*wp).w_height;
        } else if n_0 == 0 as ::core::ffi::c_int
            && *(*eap).arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            n_0 = Rows.get() - 1 as ::core::ffi::c_int;
        }
        win_setheight_win(n_0, wp);
    };
}
unsafe extern "C" fn ex_find(mut eap: *mut exarg_T) {
    if !check_can_set_curbuf_forceit((*eap).forceit) {
        return;
    }
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *get_findfunc() as ::core::ffi::c_int != NUL {
        fname = findfunc_find_file(
            (*eap).arg,
            strlen((*eap).arg),
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            },
        );
    } else {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        fname = find_file_in_path(
            (*eap).arg,
            strlen((*eap).arg),
            FNAME_MESS as ::core::ffi::c_int,
            true_0,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            let mut count: linenr_T = (*eap).line2;
            while !fname.is_null() && {
                count -= 1;
                count > 0 as linenr_T
            } {
                xfree(fname as *mut ::core::ffi::c_void);
                fname = find_file_in_path(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as size_t,
                    FNAME_MESS as ::core::ffi::c_int,
                    false_0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    }
    if fname.is_null() {
        return;
    }
    (*eap).arg = fname;
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    xfree(fname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ex_edit(mut eap: *mut exarg_T) {
    let mut ffname: *mut ::core::ffi::c_char =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_enew as ::core::ffi::c_int {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*eap).arg
        };
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_badd as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_balt as ::core::ffi::c_int
        && (is_other_file(0 as ::core::ffi::c_int, ffname) as ::core::ffi::c_int != 0
            && !check_can_set_curbuf_forceit((*eap).forceit))
    {
        return;
    }
    if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
        && (*eap).cmdidx as ::core::ffi::c_int == CMD_edit as ::core::ffi::c_int
        && *(*eap).arg as ::core::ffi::c_int == NUL
    {
        emsg(b"cannot :edit a prompt buffer\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
}
#[no_mangle]
pub unsafe extern "C" fn do_exedit(mut eap: *mut exarg_T, mut old_curwin: *mut win_T) {
    if exmode_active.get() as ::core::ffi::c_int != 0
        && ((*eap).cmdidx as ::core::ffi::c_int == CMD_visual as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_view as ::core::ffi::c_int)
    {
        exmode_active.set(false_0 != 0);
        ex_pressedreturn.set(false_0 != 0);
        if ui_has(kUICmdline) {
            ui_ext_cmdline_block_leave();
        }
        if *(*eap).arg as ::core::ffi::c_int == NUL {
            if global_busy.get() != 0 {
                if !(*eap).nextcmd.is_null() {
                    stuffReadbuff((*eap).nextcmd);
                    (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                let save_rd: ::core::ffi::c_int = RedrawingDisabled.get();
                RedrawingDisabled.set(0 as ::core::ffi::c_int);
                let save_nwr: ::core::ffi::c_int = no_wait_return.get();
                no_wait_return.set(0 as ::core::ffi::c_int);
                need_wait_return.set(false_0 != 0);
                let save_ms: ::core::ffi::c_int = msg_scroll.get();
                msg_scroll.set(0 as ::core::ffi::c_int);
                redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
                pending_exmode_active.set(true_0 != 0);
                normal_enter(false_0 != 0, true_0 != 0);
                pending_exmode_active.set(false_0 != 0);
                RedrawingDisabled.set(save_rd);
                no_wait_return.set(save_nwr);
                msg_scroll.set(save_ms);
            }
            return;
        }
    }
    if ((*eap).cmdidx as ::core::ffi::c_int == CMD_new as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabnew as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_tabedit as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_vnew as ::core::ffi::c_int)
        && *(*eap).arg as ::core::ffi::c_int == NUL
    {
        setpcmark();
        do_ecmd(
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            eap,
            ECMD_ONE as ::core::ffi::c_int as linenr_T,
            ECMD_HIDE as ::core::ffi::c_int
                + (if (*eap).forceit != 0 {
                    ECMD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        );
    } else if (*eap).cmdidx as ::core::ffi::c_int != CMD_split as ::core::ffi::c_int
        && (*eap).cmdidx as ::core::ffi::c_int != CMD_vsplit as ::core::ffi::c_int
        || *(*eap).arg as ::core::ffi::c_int != NUL
    {
        if *(*eap).arg as ::core::ffi::c_int != NUL
            && text_or_buf_locked() as ::core::ffi::c_int != 0
        {
            return;
        }
        let mut n: ::core::ffi::c_int = readonlymode.get() as ::core::ffi::c_int;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_view as ::core::ffi::c_int
            || (*eap).cmdidx as ::core::ffi::c_int == CMD_sview as ::core::ffi::c_int
        {
            readonlymode.set(true_0 != 0);
        } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_enew as ::core::ffi::c_int {
            readonlymode.set(false_0 != 0);
        }
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_balt as ::core::ffi::c_int
            && (*eap).cmdidx as ::core::ffi::c_int != CMD_badd as ::core::ffi::c_int
        {
            setpcmark();
        }
        if do_ecmd(
            0 as ::core::ffi::c_int,
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_enew as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                (*eap).arg
            },
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            eap,
            (*eap).do_ecmd_lnum,
            (if buf_hide(curbuf.get()) as ::core::ffi::c_int != 0 {
                ECMD_HIDE as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if (*eap).forceit != 0 {
                ECMD_FORCEIT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if !old_curwin.is_null() {
                ECMD_OLDBUF as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if (*eap).cmdidx as ::core::ffi::c_int == CMD_badd as ::core::ffi::c_int {
                ECMD_ADDBUF as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) + (if (*eap).cmdidx as ::core::ffi::c_int == CMD_balt as ::core::ffi::c_int {
                ECMD_ALTBUF as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
            if old_curwin.is_null() {
                curwin.get()
            } else {
                ::core::ptr::null_mut::<win_T>()
            },
        ) == FAIL
        {
            if !old_curwin.is_null() {
                let mut need_hide: bool = curbufIsChanged() as ::core::ffi::c_int != 0
                    && (*curbuf.get()).b_nwindows <= 1 as ::core::ffi::c_int;
                if !need_hide || buf_hide(curbuf.get()) as ::core::ffi::c_int != 0 {
                    let mut cs: cleanup_T = cleanup_T {
                        pending: 0,
                        exception: ::core::ptr::null_mut::<except_T>(),
                    };
                    enter_cleanup(&raw mut cs);
                    win_close(
                        curwin.get(),
                        !need_hide && !buf_hide(curbuf.get()),
                        false_0 != 0,
                    );
                    leave_cleanup(&raw mut cs);
                }
            }
        } else if readonlymode.get() as ::core::ffi::c_int != 0
            && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_p_ro = true_0;
        }
        readonlymode.set(n != 0);
    } else {
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
        let mut n_0: ::core::ffi::c_int = (*curwin.get()).w_arg_idx_invalid;
        check_arg_idx(curwin.get());
        if n_0 != (*curwin.get()).w_arg_idx_invalid {
            maketitle();
        }
    }
    if !old_curwin.is_null()
        && *(*eap).arg as ::core::ffi::c_int != NUL
        && curwin.get() != old_curwin
        && win_valid(old_curwin) as ::core::ffi::c_int != 0
        && (*old_curwin).w_buffer != curbuf.get()
        && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
    {
        (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
    }
    ex_no_reprint.set(true_0 != 0);
}
unsafe extern "C" fn ex_nogui(mut eap: *mut exarg_T) {
    (*eap).errmsg =
        gettext(b"E25: Nvim does not have a built-in GUI\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn ex_popup(mut eap: *mut exarg_T) {
    pum_make_popup((*eap).arg, (*eap).forceit);
}
unsafe extern "C" fn ex_swapname(mut _eap: *mut exarg_T) {
    if (*curbuf.get()).b_ml.ml_mfp.is_null() || (*(*curbuf.get()).b_ml.ml_mfp).mf_fname.is_null() {
        msg(
            gettext(b"No swap file\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    } else {
        msg(
            (*(*curbuf.get()).b_ml.ml_mfp).mf_fname,
            0 as ::core::ffi::c_int,
        );
    };
}
unsafe extern "C" fn ex_syncbind(mut _eap: *mut exarg_T) {
    let mut vtopline: linenr_T = 0;
    let mut old_linenr: linenr_T = (*curwin.get()).w_cursor.lnum;
    setpcmark();
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        vtopline = get_vtopline(curwin.get()) as linenr_T;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_scb != 0 && !(*wp).w_buffer.is_null() {
                let mut y: linenr_T =
                    plines_m_win_fill(wp, 1 as linenr_T, (*(*wp).w_buffer).b_ml.ml_line_count)
                        as linenr_T
                        - get_scrolloff_value(curwin.get()) as linenr_T;
                vtopline = if vtopline < y { vtopline } else { y };
            }
            wp = (*wp).w_next;
        }
        vtopline = if vtopline > 1 as linenr_T {
            vtopline
        } else {
            1 as linenr_T
        };
    } else {
        vtopline = 1 as ::core::ffi::c_int as linenr_T;
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        if (*wp_0).w_onebuf_opt.wo_scb != 0 {
            let mut y_0: ::core::ffi::c_int = vtopline as ::core::ffi::c_int - get_vtopline(wp_0);
            if y_0 > 0 as ::core::ffi::c_int {
                scrollup(wp_0, y_0 as linenr_T, true_0 != 0);
            } else {
                scrolldown(wp_0, -(y_0 as linenr_T), true_0);
            }
            (*wp_0).w_scbind_pos = vtopline as ::core::ffi::c_int;
            redraw_later(wp_0, UPD_VALID as ::core::ffi::c_int);
            cursor_correct(wp_0);
            (*wp_0).w_redr_status = true_0 != 0;
        }
        wp_0 = (*wp_0).w_next;
    }
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        did_syncbind.set(true_0 != 0);
        checkpcmark();
        if old_linenr != (*curwin.get()).w_cursor.lnum {
            let mut ctrl_o: [::core::ffi::c_char; 2] = [0; 2];
            ctrl_o[0 as ::core::ffi::c_int as usize] = Ctrl_O as ::core::ffi::c_char;
            ctrl_o[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
            ins_typebuf(
                &raw mut ctrl_o as *mut ::core::ffi::c_char,
                REMAP_NONE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                false_0 != 0,
            );
        }
    }
}
unsafe extern "C" fn ex_read(mut eap: *mut exarg_T) {
    let mut empty: ::core::ffi::c_int = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;
    if (*eap).usefilter != 0 {
        do_bang(
            1 as ::core::ffi::c_int,
            eap,
            false_0 != 0,
            false_0 != 0,
            true_0 != 0,
        );
        return;
    }
    if u_save((*eap).line2, (*eap).line2 + 1 as linenr_T) == FAIL {
        return;
    }
    let mut i: ::core::ffi::c_int = 0;
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        if check_fname() == FAIL {
            return;
        }
        i = readfile(
            (*curbuf.get()).b_ffname,
            (*curbuf.get()).b_fname,
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            eap,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    } else {
        if !vim_strchr(p_cpo.get(), CPO_ALTREAD).is_null() {
            setaltfname((*eap).arg, (*eap).arg, 1 as linenr_T);
        }
        i = readfile(
            (*eap).arg,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            (*eap).line2,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            eap,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    if i != OK {
        if !aborting() {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
    } else {
        if empty != 0 && exmode_active.get() as ::core::ffi::c_int != 0 {
            let mut lnum: linenr_T = 0;
            if (*eap).line2 == 0 as linenr_T {
                lnum = (*curbuf.get()).b_ml.ml_line_count;
            } else {
                lnum = 1 as ::core::ffi::c_int as linenr_T;
            }
            if *ml_get(lnum) as ::core::ffi::c_int == NUL && u_savedel(lnum, 1 as linenr_T) == OK {
                ml_delete(lnum);
                if (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                    && (*curwin.get()).w_cursor.lnum >= lnum
                {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                deleted_lines_mark(lnum, 1 as ::core::ffi::c_int);
            }
        }
        redraw_curbuf_later(UPD_VALID as ::core::ffi::c_int);
    };
}
static prev_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
unsafe extern "C" fn get_prevdir(mut scope: CdScope) -> *mut ::core::ffi::c_char {
    match scope as ::core::ffi::c_int {
        1 => return (*curtab.get()).tp_prevdir,
        0 => return (*curwin.get()).w_prevdir,
        _ => return prev_dir.get(),
    };
}
unsafe extern "C" fn post_chdir(mut scope: CdScope, mut trigger_dirchanged: bool) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*curwin.get()).w_localdir as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_1;
    let _ = *ptr_;
    if scope as ::core::ffi::c_int >= kCdScopeTabpage as ::core::ffi::c_int {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*curtab.get()).tp_localdir as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_1;
        let _ = *ptr__0;
    }
    if (scope as ::core::ffi::c_int) < kCdScopeGlobal as ::core::ffi::c_int {
        let mut pdir: *mut ::core::ffi::c_char = get_prevdir(scope);
        if (*globaldir.ptr()).is_null() && !pdir.is_null() {
            globaldir.set(xstrdup(pdir));
        }
    }
    let mut cwd: [::core::ffi::c_char; 4096] = [0; 4096];
    if os_dirname(&raw mut cwd as *mut ::core::ffi::c_char, MAXPATHL as size_t) != OK {
        return;
    }
    match scope as ::core::ffi::c_int {
        2 => {
            let mut ptr__1: *mut *mut ::core::ffi::c_void =
                globaldir.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__1);
            *ptr__1 = NULL_1;
            let _ = *ptr__1;
        }
        1 => {
            (*curtab.get()).tp_localdir = xstrdup(&raw mut cwd as *mut ::core::ffi::c_char);
        }
        0 => {
            (*curwin.get()).w_localdir = xstrdup(&raw mut cwd as *mut ::core::ffi::c_char);
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    last_chdir_reason.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    shorten_fnames(vim_strchr(p_cpo.get(), CPO_NOSYMLINKS).is_null() as ::core::ffi::c_int);
    if trigger_dirchanged {
        do_autocmd_dirchanged(
            &raw mut cwd as *mut ::core::ffi::c_char,
            scope,
            kCdCauseManual,
            false_0 != 0,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn changedir_func(
    mut new_dir: *mut ::core::ffi::c_char,
    mut scope: CdScope,
) -> bool {
    if new_dir.is_null() || allbuf_locked() as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    let mut pdir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strcmp(new_dir, b"-\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        pdir = get_prevdir(scope);
        if pdir.is_null() {
            emsg(gettext(
                b"E186: No previous directory\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return false_0 != 0;
        }
        new_dir = pdir;
    }
    if os_dirname(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == OK
    {
        pdir = xstrdup(NameBuff.ptr() as *mut ::core::ffi::c_char);
    } else {
        pdir = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *new_dir as ::core::ffi::c_int == NUL && p_cdh.get() != 0 {
        expand_env(
            b"$HOME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL,
        );
        new_dir = NameBuff.ptr() as *mut ::core::ffi::c_char;
    }
    let mut dir_differs: bool = pdir.is_null()
        || pathcmp(pdir, new_dir, -1 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int;
    if dir_differs {
        do_autocmd_dirchanged(new_dir, scope, kCdCauseManual, true_0 != 0);
        if vim_chdir(new_dir) != 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_failed as *const ::core::ffi::c_char));
            xfree(pdir as *mut ::core::ffi::c_void);
            return false_0 != 0;
        }
    }
    let mut pp: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    match scope as ::core::ffi::c_int {
        1 => {
            pp = &raw mut (*curtab.get()).tp_prevdir;
        }
        0 => {
            pp = &raw mut (*curwin.get()).w_prevdir;
        }
        _ => {
            pp = prev_dir.ptr();
        }
    }
    xfree(*pp as *mut ::core::ffi::c_void);
    *pp = pdir;
    post_chdir(scope, dir_differs);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ex_cd(mut eap: *mut exarg_T) {
    let mut new_dir: *mut ::core::ffi::c_char = (*eap).arg;
    if *new_dir as ::core::ffi::c_int == NUL && p_cdh.get() == 0 {
        ex_pwd(::core::ptr::null_mut::<exarg_T>());
        return;
    }
    let mut scope: CdScope = kCdScopeGlobal;
    match (*eap).cmdidx as ::core::ffi::c_int {
        448 | 449 => {
            scope = kCdScopeTabpage;
        }
        225 | 226 => {
            scope = kCdScopeWindow;
        }
        _ => {}
    }
    if changedir_func(new_dir, scope) {
        if KeyTyped.get() as ::core::ffi::c_int != 0 || p_verbose.get() >= 5 as OptInt {
            ex_pwd(eap);
        }
    }
}
unsafe extern "C" fn ex_pwd(mut _eap: *mut exarg_T) {
    if os_dirname(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == OK
    {
        if p_verbose.get() > 0 as OptInt {
            let mut context: *mut ::core::ffi::c_char =
                b"global\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            if !(*last_chdir_reason.ptr()).is_null() {
                context = last_chdir_reason.get();
            } else if !(*curwin.get()).w_localdir.is_null() {
                context =
                    b"window\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if !(*curtab.get()).tp_localdir.is_null() {
                context =
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            smsg(
                0 as ::core::ffi::c_int,
                b"[%s] %s\0".as_ptr() as *const ::core::ffi::c_char,
                context,
                NameBuff.ptr() as *mut ::core::ffi::c_char,
            );
        } else {
            msg(
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
    } else {
        emsg(gettext(
            b"E187: Unknown\0".as_ptr() as *const ::core::ffi::c_char
        ));
    };
}
unsafe extern "C" fn ex_equal(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int != NUL
        && *(*eap).arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        ex_lua(eap);
    } else {
        (*eap).nextcmd = find_nextcmd((*eap).arg);
        smsg(
            0 as ::core::ffi::c_int,
            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
            (*eap).line2 as int64_t,
        );
    };
}
unsafe extern "C" fn ex_sleep(mut eap: *mut exarg_T) {
    if cursor_valid(curwin.get()) != 0 {
        setcursor_mayforce(curwin.get(), true_0 != 0);
    }
    let mut len: int64_t = (*eap).line2 as int64_t;
    match *(*eap).arg as ::core::ffi::c_int {
        109 => {}
        NUL => {
            len *= 1000 as int64_t;
        }
        _ => {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
    }
    do_sleep(len, (*eap).forceit != 0);
}
#[no_mangle]
pub unsafe extern "C" fn do_sleep(mut msec: int64_t, mut hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    ui_flush();
    let mut remaining: int64_t = msec;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !got_int.get() {
        if !(*main_loop.ptr()).events.is_null() && !multiqueue_empty((*main_loop.ptr()).events) {
            multiqueue_process_events((*main_loop.ptr()).events);
        } else {
            loop_poll_events(main_loop.ptr(), remaining);
        }
        if remaining == 0 as int64_t {
            break;
        }
        if remaining <= 0 as int64_t {
            continue;
        }
        let mut now: uint64_t = os_hrtime();
        remaining -= now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
        before = now;
        if remaining <= 0 as int64_t {
            break;
        }
    }
    if got_int.get() {
        vpeekc();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}
unsafe extern "C" fn ex_winsize(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    if !ascii_isdigit(*arg as ::core::ffi::c_int) {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    let mut w: ::core::ffi::c_int =
        getdigits_int(&raw mut arg, false_0 != 0, 10 as ::core::ffi::c_int);
    arg = skipwhite(arg);
    let mut p: *mut ::core::ffi::c_char = arg;
    let mut h: ::core::ffi::c_int =
        getdigits_int(&raw mut arg, false_0 != 0, 10 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int != NUL && *arg as ::core::ffi::c_int == NUL {
        screen_resize(w, h);
    } else {
        emsg(gettext(
            b"E465: :winsize requires two number arguments\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    };
}
unsafe extern "C" fn ex_wincmd(mut eap: *mut exarg_T) {
    let mut xchar: ::core::ffi::c_int = NUL;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *(*eap).arg as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
        || *(*eap).arg as ::core::ffi::c_int == Ctrl_G
    {
        if *(*eap).arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        xchar =
            *(*eap).arg.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
        p = (*eap).arg.offset(2 as ::core::ffi::c_int as isize);
    } else {
        p = (*eap).arg.offset(1 as ::core::ffi::c_int as isize);
    }
    (*eap).nextcmd = check_nextcmd(p);
    p = skipwhite(p);
    if *p as ::core::ffi::c_int != NUL
        && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
        && (*eap).nextcmd.is_null()
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
    } else if (*eap).skip == 0 {
        postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
        postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
        do_window(
            *(*eap).arg as ::core::ffi::c_int,
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            xchar,
        );
        postponed_split_flags.set(0 as ::core::ffi::c_int);
        postponed_split_tab.set(0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn ex_operators(mut eap: *mut exarg_T) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
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
    clear_oparg(&raw mut oa);
    oa.regname = (*eap).regname;
    oa.start.lnum = (*eap).line1;
    oa.end.lnum = (*eap).line2;
    oa.line_count = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
    oa.motion_type = kMTLineWise;
    virtual_op.set(kFalse);
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_yank as ::core::ffi::c_int {
        setpcmark();
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    }
    if VIsual_active.get() {
        end_visual_mode();
    }
    match (*eap).cmdidx as ::core::ffi::c_int {
        109 => {
            oa.op_type = OP_DELETE as ::core::ffi::c_int;
            op_delete(&raw mut oa);
        }
        546 => {
            oa.op_type = OP_YANK as ::core::ffi::c_int;
            op_yank(&raw mut oa, true_0 != 0);
        }
        _ => {
            if ((*eap).cmdidx as ::core::ffi::c_int == CMD_rshift as ::core::ffi::c_int)
                as ::core::ffi::c_int
                ^ (*curwin.get()).w_onebuf_opt.wo_rl
                != 0
            {
                oa.op_type = OP_RSHIFT as ::core::ffi::c_int;
            } else {
                oa.op_type = OP_LSHIFT as ::core::ffi::c_int;
            }
            op_shift(&raw mut oa, false_0 != 0, (*eap).amount);
        }
    }
    virtual_op.set(kNone);
    ex_may_print(eap);
}
unsafe extern "C" fn ex_put(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as ::core::ffi::c_int
        } else {
            FORWARD as ::core::ffi::c_int
        },
        1 as ::core::ffi::c_int,
        PUT_LINE as ::core::ffi::c_int | PUT_CURSLINE as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn ex_iput(mut eap: *mut exarg_T) {
    if (*eap).line2 == 0 as linenr_T {
        (*eap).line2 = 1 as ::core::ffi::c_int as linenr_T;
        (*eap).forceit = true_0;
    }
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    do_put(
        (*eap).regname,
        ::core::ptr::null_mut::<yankreg_T>(),
        if (*eap).forceit != 0 {
            BACKWARD as ::core::ffi::c_int
        } else {
            FORWARD as ::core::ffi::c_int
        },
        1 as ::core::ffi::c_int,
        PUT_LINE as ::core::ffi::c_int
            | PUT_CURSLINE as ::core::ffi::c_int
            | PUT_FIXINDENT as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn ex_copymove(mut eap: *mut exarg_T) {
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: linenr_T = get_address(
        eap,
        &raw mut (*eap).arg,
        (*eap).addr_type,
        false_0 != 0,
        false_0 != 0,
        false_0,
        1 as ::core::ffi::c_int,
        &raw mut errormsg,
    );
    if (*eap).arg.is_null() {
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return;
    }
    get_flags(eap);
    if n == MAXLNUM as ::core::ffi::c_int as linenr_T
        || n < 0 as linenr_T
        || n > (*curbuf.get()).b_ml.ml_line_count
    {
        emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_move as ::core::ffi::c_int {
        if do_move((*eap).line1, (*eap).line2, n) == FAIL {
            return;
        }
    } else {
        ex_copy((*eap).line1, (*eap).line2, n);
    }
    u_clearline(curbuf.get());
    beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    ex_may_print(eap);
}
#[no_mangle]
pub unsafe extern "C" fn ex_may_print(mut eap: *mut exarg_T) {
    if (*eap).flags != 0 as ::core::ffi::c_int {
        print_line(
            (*curwin.get()).w_cursor.lnum,
            (*eap).flags & EXFLAG_NR != 0,
            (*eap).flags & EXFLAG_LIST != 0,
            true_0 != 0,
        );
        ex_no_reprint.set(true_0 != 0);
    }
}
unsafe extern "C" fn ex_submagic(mut eap: *mut exarg_T) {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as ::core::ffi::c_int == CMD_smagic as ::core::ffi::c_int {
            OPTION_MAGIC_ON as ::core::ffi::c_int
        } else {
            OPTION_MAGIC_OFF as ::core::ffi::c_int
        }) as optmagic_T,
    );
    ex_substitute(eap);
    magic_overruled.set(saved);
}
unsafe extern "C" fn ex_submagic_preview(
    mut eap: *mut exarg_T,
    mut cmdpreview_ns: ::core::ffi::c_int,
    mut cmdpreview_bufnr: handle_T,
) -> ::core::ffi::c_int {
    let saved: optmagic_T = magic_overruled.get();
    magic_overruled.set(
        (if (*eap).cmdidx as ::core::ffi::c_int == CMD_smagic as ::core::ffi::c_int {
            OPTION_MAGIC_ON as ::core::ffi::c_int
        } else {
            OPTION_MAGIC_OFF as ::core::ffi::c_int
        }) as optmagic_T,
    );
    let mut retv: ::core::ffi::c_int = ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
    magic_overruled.set(saved);
    return retv;
}
unsafe extern "C" fn ex_join(mut eap: *mut exarg_T) {
    (*curwin.get()).w_cursor.lnum = (*eap).line1;
    if (*eap).line1 == (*eap).line2 {
        if (*eap).addr_count >= 2 as ::core::ffi::c_int {
            return;
        }
        if (*eap).line2 == (*curbuf.get()).b_ml.ml_line_count {
            beep_flush();
            return;
        }
        (*eap).line2 += 1;
    }
    do_join(
        ((*eap).line2 as ssize_t - (*eap).line1 as ssize_t + 1 as ssize_t) as size_t,
        (*eap).forceit == 0,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    ex_may_print(eap);
}
unsafe extern "C" fn ex_at(mut eap: *mut exarg_T) {
    let mut prev_len: ::core::ffi::c_int = (*typebuf.ptr()).tb_len;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    check_cursor_col(curwin.get());
    let mut c: ::core::ffi::c_int = *(*eap).arg as uint8_t as ::core::ffi::c_int;
    if c == NUL {
        c = '@' as ::core::ffi::c_int;
    }
    if do_execreg(
        c,
        true_0,
        !vim_strchr(p_cpo.get(), CPO_EXECBUF).is_null() as ::core::ffi::c_int,
        true_0,
    ) == FAIL
    {
        beep_flush();
        return;
    }
    let save_efr: bool = exec_from_reg.get();
    exec_from_reg.set(true_0 != 0);
    while !stuff_empty() || (*typebuf.ptr()).tb_len > prev_len {
        do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            Some(
                getexline
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
            NULL_1,
            DOCMD_NOWAIT as ::core::ffi::c_int | DOCMD_VERBOSE as ::core::ffi::c_int,
        );
    }
    exec_from_reg.set(save_efr);
}
unsafe extern "C" fn ex_bang(mut eap: *mut exarg_T) {
    do_bang(
        (*eap).addr_count,
        eap,
        (*eap).forceit != 0,
        true_0 != 0,
        true_0 != 0,
    );
}
unsafe extern "C" fn ex_undo(mut eap: *mut exarg_T) {
    if (*eap).addr_count != 1 as ::core::ffi::c_int {
        if (*eap).forceit != 0 {
            u_undo_and_forget(1 as ::core::ffi::c_int, true_0 != 0);
        } else {
            u_undo(1 as ::core::ffi::c_int);
        }
        return;
    }
    let mut step: linenr_T = (*eap).line2;
    if (*eap).forceit != 0 {
        if step >= (*curbuf.get()).b_u_seq_cur as linenr_T {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        uhp = if !(*curbuf.get()).b_u_curhead.is_null() {
            (*curbuf.get()).b_u_curhead
        } else {
            (*curbuf.get()).b_u_newhead
        };
        while !uhp.is_null() && (*uhp).uh_seq as linenr_T > step {
            uhp = (*uhp).uh_next.ptr;
            count += 1;
        }
        if step != 0 as linenr_T && (uhp.is_null() || ((*uhp).uh_seq as linenr_T) < step) {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const ::core::ffi::c_char,
            ));
            return;
        }
        u_undo_and_forget(count, true_0 != 0);
    } else {
        undo_time(
            step as ::core::ffi::c_int,
            false_0 != 0,
            false_0 != 0,
            true_0 != 0,
        );
    };
}
unsafe extern "C" fn ex_wundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_write_undo(
        (*eap).arg,
        (*eap).forceit != 0,
        curbuf.get(),
        &raw mut hash as *mut uint8_t,
    );
}
unsafe extern "C" fn ex_rundo(mut eap: *mut exarg_T) {
    let mut hash: [uint8_t; 32] = [0; 32];
    u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
    u_read_undo(
        (*eap).arg,
        &raw mut hash as *mut uint8_t,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
}
unsafe extern "C" fn ex_redo(mut _eap: *mut exarg_T) {
    u_redo(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_later(mut eap: *mut exarg_T) {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sec: bool = false_0 != 0;
    let mut file: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    if *p as ::core::ffi::c_int == NUL {
        count = 1 as ::core::ffi::c_int;
    } else if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        count = getdigits_int(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_int);
        match *p as ::core::ffi::c_int {
            115 => {
                p = p.offset(1);
                sec = true_0 != 0;
            }
            109 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as ::core::ffi::c_int;
            }
            104 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *= 60 as ::core::ffi::c_int * 60 as ::core::ffi::c_int;
            }
            100 => {
                p = p.offset(1);
                sec = true_0 != 0;
                count *=
                    24 as ::core::ffi::c_int * 60 as ::core::ffi::c_int * 60 as ::core::ffi::c_int;
            }
            102 => {
                p = p.offset(1);
                file = true_0 != 0;
            }
            _ => {}
        }
    }
    if *p as ::core::ffi::c_int != NUL {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    } else {
        undo_time(
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_earlier as ::core::ffi::c_int {
                -count
            } else {
                count
            },
            sec,
            file,
            false_0 != 0,
        );
    };
}
unsafe extern "C" fn ex_redir(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    if strcasecmp(
        (*eap).arg,
        b"END\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        close_redir();
    } else if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
        arg = arg.offset(1);
        let mut mode: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            arg = arg.offset(1);
            mode = b"a\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            mode = b"w\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        arg = skipwhite(arg);
        close_redir();
        let mut fname: *mut ::core::ffi::c_char = expand_env_save(arg);
        if fname.is_null() {
            return;
        }
        redir_fd.set(open_exfile(fname, (*eap).forceit, mode));
        xfree(fname as *mut ::core::ffi::c_void);
    } else if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
        close_redir();
        arg = arg.offset(1);
        if valid_yank_reg(*arg as ::core::ffi::c_int, true_0 != 0) as ::core::ffi::c_int != 0
            && *arg as ::core::ffi::c_int != '_' as ::core::ffi::c_int
        {
            let c2rust_fresh15 = arg;
            arg = arg.offset(1);
            redir_reg.set(*c2rust_fresh15 as uint8_t as ::core::ffi::c_int);
            if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '>' as ::core::ffi::c_int
            {
                arg = arg.offset(2 as ::core::ffi::c_int as isize);
            } else {
                if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                    arg = arg.offset(1);
                }
                if *arg as ::core::ffi::c_int == NUL
                    && *(*__ctype_b_loc()).offset(redir_reg.get() as isize) as ::core::ffi::c_int
                        & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                {
                    write_reg_contents(
                        redir_reg.get(),
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ssize_t,
                        false_0,
                    );
                }
            }
        }
        if *arg as ::core::ffi::c_int != NUL {
            redir_reg.set(0 as ::core::ffi::c_int);
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
    } else if *arg as ::core::ffi::c_int == '=' as ::core::ffi::c_int
        && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '>' as ::core::ffi::c_int
    {
        let mut append: bool = false;
        close_redir();
        arg = arg.offset(2 as ::core::ffi::c_int as isize);
        if *arg as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            arg = arg.offset(1);
            append = true_0 != 0;
        } else {
            append = false_0 != 0;
        }
        if var_redir_start(skipwhite(arg), append) == OK {
            redir_vname.set(true_0 != 0);
        }
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    }
    if !(*redir_fd.ptr()).is_null()
        || redir_reg.get() != 0
        || redir_vname.get() as ::core::ffi::c_int != 0
    {
        redir_off.set(false_0 != 0);
    }
}
unsafe extern "C" fn ex_redraw(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: ::core::ffi::c_int = RedrawingDisabled.get();
    let mut p: ::core::ffi::c_int = p_lz.get();
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    p_lz.set(false_0);
    validate_cursor(curwin.get());
    update_topline(curwin.get());
    if (*eap).forceit != 0 {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
        redraw_cmdline.set(true_0 != 0);
    } else if VIsual_active.get() {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    update_screen();
    if need_maketitle.get() {
        maketitle();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    msg_didout.set(false_0 != 0);
    msg_col.set(0 as ::core::ffi::c_int);
    need_wait_return.set(false_0 != 0);
    ui_flush();
}
unsafe extern "C" fn ex_redrawstatus(mut eap: *mut exarg_T) {
    if cmdpreview.get() {
        return;
    }
    let mut r: ::core::ffi::c_int = RedrawingDisabled.get();
    let mut p: ::core::ffi::c_int = p_lz.get();
    if (*eap).forceit != 0 {
        status_redraw_all();
    } else {
        status_redraw_curbuf();
    }
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    p_lz.set(false_0);
    if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        redraw_statuslines();
    } else {
        if VIsual_active.get() {
            redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
        }
        update_screen();
    }
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}
unsafe extern "C" fn ex_redrawtabline(mut _eap: *mut exarg_T) {
    let r: ::core::ffi::c_int = RedrawingDisabled.get();
    let p: ::core::ffi::c_int = p_lz.get();
    RedrawingDisabled.set(0 as ::core::ffi::c_int);
    p_lz.set(false_0);
    draw_tabline();
    RedrawingDisabled.set(r);
    p_lz.set(p);
    ui_flush();
}
unsafe extern "C" fn close_redir() {
    if !(*redir_fd.ptr()).is_null() {
        fclose(redir_fd.get());
        redir_fd.set(::core::ptr::null_mut::<FILE>());
    }
    redir_reg.set(0 as ::core::ffi::c_int);
    if redir_vname.get() {
        var_redir_stop();
        redir_vname.set(false_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn vim_mkdir_emsg(
    name: *const ::core::ffi::c_char,
    prot: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    ret = os_mkdir(name, prot as int32_t);
    if ret != 0 as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
            name,
            uv_strerror(ret),
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn open_exfile(
    mut fname: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut mode: *mut ::core::ffi::c_char,
) -> *mut FILE {
    if os_isdir(fname) {
        semsg(
            gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
            fname,
        );
        return ::core::ptr::null_mut::<FILE>();
    }
    if forceit == 0
        && *mode as ::core::ffi::c_int != 'a' as ::core::ffi::c_int
        && os_path_exists(fname) as ::core::ffi::c_int != 0
    {
        semsg(
            gettext(
                b"E189: \"%s\" exists (add ! to override)\0".as_ptr() as *const ::core::ffi::c_char
            ),
            fname,
        );
        return ::core::ptr::null_mut::<FILE>();
    }
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    fd = os_fopen(fname, mode);
    if fd.is_null() {
        semsg(
            gettext(
                b"E190: Cannot open \"%s\" for writing\0".as_ptr() as *const ::core::ffi::c_char
            ),
            fname,
        );
    }
    return fd;
}
unsafe extern "C" fn ex_mark(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        return;
    }
    if *(*eap).arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            (*eap).arg,
        );
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    (*curwin.get()).w_cursor.lnum = (*eap).line2;
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    if setmark(*(*eap).arg as ::core::ffi::c_int) == FAIL {
        emsg(gettext(
            b"E191: Argument must be a letter or forward/backward quote\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    }
    (*curwin.get()).w_cursor = pos;
}
#[no_mangle]
pub unsafe extern "C" fn update_topline_cursor() {
    check_cursor(curwin.get());
    update_topline(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
        validate_cursor(curwin.get());
    }
    update_curswant();
}
#[no_mangle]
pub unsafe extern "C" fn save_current_state(mut sst: *mut save_state_T) -> bool {
    (*sst).save_msg_scroll = msg_scroll.get();
    (*sst).save_restart_edit = restart_edit.get();
    (*sst).save_msg_didout = msg_didout.get();
    (*sst).save_State = State.get();
    (*sst).save_finish_op = finish_op.get();
    (*sst).save_opcount = opcount.get();
    (*sst).save_reg_executing = reg_executing.get();
    (*sst).save_pending_end_reg_executing = pending_end_reg_executing.get();
    msg_scroll.set(false_0);
    restart_edit.set(0 as ::core::ffi::c_int);
    save_typeahead(&raw mut (*sst).tabuf);
    return (*sst).tabuf.typebuf_valid;
}
#[no_mangle]
pub unsafe extern "C" fn restore_current_state(mut sst: *mut save_state_T) {
    restore_typeahead(&raw mut (*sst).tabuf);
    msg_scroll.set((*sst).save_msg_scroll);
    if force_restart_edit.get() {
        force_restart_edit.set(false_0 != 0);
    } else {
        restart_edit.set((*sst).save_restart_edit);
    }
    finish_op.set((*sst).save_finish_op);
    opcount.set((*sst).save_opcount);
    reg_executing.set((*sst).save_reg_executing);
    pending_end_reg_executing.set((*sst).save_pending_end_reg_executing);
    msg_didout.set(
        msg_didout.get() as ::core::ffi::c_int | (*sst).save_msg_didout as ::core::ffi::c_int != 0,
    );
    State.set((*sst).save_State);
    ui_cursor_shape();
}
#[no_mangle]
pub unsafe extern "C" fn expr_map_locked() -> bool {
    return expr_map_lock.get() > 0 as ::core::ffi::c_int
        && (*curbuf.get()).b_flags & BF_DUMMY == 0;
}
unsafe extern "C" fn ex_normal(mut eap: *mut exarg_T) {
    if !(*curbuf.get()).terminal.is_null() && State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
    {
        emsg(b"Can't re-enter normal mode from terminal mode\0".as_ptr()
            as *const ::core::ffi::c_char);
        return;
    }
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if expr_map_locked() {
        emsg(gettext(&raw const e_secure as *const ::core::ffi::c_char));
        return;
    }
    if ex_normal_busy.get() as OptInt >= p_mmd.get() {
        emsg(gettext(
            b"E192: Recursive use of :normal too deep\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut l: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    while *p as ::core::ffi::c_int != NUL {
        l = utfc_ptr2len(p) - 1 as ::core::ffi::c_int;
        while l > 0 as ::core::ffi::c_int {
            p = p.offset(1);
            if *p as ::core::ffi::c_int == K_SPECIAL as ::core::ffi::c_char as ::core::ffi::c_int {
                len += 2 as ::core::ffi::c_int;
            }
            l -= 1;
        }
        p = p.offset(1);
    }
    if len > 0 as ::core::ffi::c_int {
        arg = xmalloc(
            strlen((*eap).arg)
                .wrapping_add(len as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        len = 0 as ::core::ffi::c_int;
        let mut p_0: *mut ::core::ffi::c_char = (*eap).arg;
        while *p_0 as ::core::ffi::c_int != NUL {
            let c2rust_fresh17 = len;
            len = len + 1;
            *arg.offset(c2rust_fresh17 as isize) = *p_0;
            l = utfc_ptr2len(p_0) - 1 as ::core::ffi::c_int;
            while l > 0 as ::core::ffi::c_int {
                p_0 = p_0.offset(1);
                let c2rust_fresh18 = len;
                len = len + 1;
                *arg.offset(c2rust_fresh18 as isize) = *p_0;
                if *p_0 as ::core::ffi::c_int
                    == K_SPECIAL as ::core::ffi::c_char as ::core::ffi::c_int
                {
                    let c2rust_fresh19 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh19 as isize) = KS_SPECIAL as ::core::ffi::c_char;
                    let c2rust_fresh20 = len;
                    len = len + 1;
                    *arg.offset(c2rust_fresh20 as isize) = KE_FILLER as ::core::ffi::c_char;
                }
                l -= 1;
            }
            *arg.offset(len as isize) = NUL as ::core::ffi::c_char;
            p_0 = p_0.offset(1);
        }
    }
    (*ex_normal_busy.ptr()) += 1;
    let mut save_state: save_state_T = save_state_T {
        save_msg_scroll: 0,
        save_restart_edit: 0,
        save_msg_didout: false,
        save_State: 0,
        save_finish_op: false,
        save_opcount: 0,
        save_reg_executing: 0,
        save_pending_end_reg_executing: false,
        tabuf: tasave_T {
            save_typebuf: typebuf_T {
                tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                tb_buflen: 0,
                tb_off: 0,
                tb_len: 0,
                tb_maplen: 0,
                tb_silent: 0,
                tb_no_abbr_cnt: 0,
                tb_change_cnt: 0,
            },
            typebuf_valid: false,
            old_char: 0,
            old_mod_mask: 0,
            save_readbuf1: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_readbuf2: buffheader_T {
                bh_first: buffblock_T {
                    b_next: ::core::ptr::null_mut::<buffblock>(),
                    b_strlen: 0,
                    b_str: [0; 1],
                },
                bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                bh_index: 0,
                bh_space: 0,
                bh_create_newblock: false,
            },
            save_inputbuf: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
    };
    if save_current_state(&raw mut save_state) {
        loop {
            if (*eap).addr_count != 0 as ::core::ffi::c_int {
                let c2rust_fresh21 = (*eap).line1;
                (*eap).line1 = (*eap).line1 + 1;
                (*curwin.get()).w_cursor.lnum = c2rust_fresh21;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                check_cursor_moved(curwin.get());
            }
            exec_normal_cmd(
                if !arg.is_null() { arg } else { (*eap).arg },
                if (*eap).forceit != 0 {
                    REMAP_NONE as ::core::ffi::c_int
                } else {
                    REMAP_YES as ::core::ffi::c_int
                },
                false_0 != 0,
            );
            if !((*eap).addr_count > 0 as ::core::ffi::c_int
                && (*eap).line1 <= (*eap).line2
                && !got_int.get())
            {
                break;
            }
        }
    }
    update_topline_cursor();
    restore_current_state(&raw mut save_state);
    (*ex_normal_busy.ptr()) -= 1;
    setmouse();
    ui_cursor_shape();
    xfree(arg as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ex_startinsert(mut eap: *mut exarg_T) {
    if (*eap).forceit != 0 {
        if (*curwin.get()).w_cursor.lnum == 0 {
            (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        }
        set_cursor_for_append_to_line();
    }
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_startinsert as ::core::ffi::c_int {
        restart_edit.set('a' as ::core::ffi::c_int);
    } else if (*eap).cmdidx as ::core::ffi::c_int == CMD_startreplace as ::core::ffi::c_int {
        restart_edit.set('R' as ::core::ffi::c_int);
    } else {
        restart_edit.set('V' as ::core::ffi::c_int);
    }
    if (*eap).forceit == 0 {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_startinsert as ::core::ffi::c_int {
            restart_edit.set('i' as ::core::ffi::c_int);
        }
        (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
    }
    if VIsual_active.get() {
        showmode();
    }
}
unsafe extern "C" fn ex_stopinsert(mut _eap: *mut exarg_T) {
    restart_edit.set(0 as ::core::ffi::c_int);
    stop_insert_mode.set(true_0 != 0);
    clearmode();
}
#[no_mangle]
pub unsafe extern "C" fn exec_normal_cmd(
    mut cmd: *mut ::core::ffi::c_char,
    mut remap: ::core::ffi::c_int,
    mut silent: bool,
) {
    ins_typebuf(cmd, remap, 0 as ::core::ffi::c_int, true_0 != 0, silent);
    exec_normal(false_0 != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn exec_normal(mut was_typed: bool, mut use_vpeekc: bool) {
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
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
    let mut c: ::core::ffi::c_int = 0;
    clear_oparg(&raw mut oa);
    finish_op.set(false_0 != 0);
    while (!stuff_empty()
        || (was_typed as ::core::ffi::c_int != 0 || typebuf_typed() == 0)
            && (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int
        || use_vpeekc as ::core::ffi::c_int != 0
            && {
                c = vpeekc();
                c != NUL
            }
            && c != Ctrl_C)
        && !got_int.get()
    {
        update_topline_cursor();
        normal_cmd(&raw mut oa, true_0 != 0);
    }
}
unsafe extern "C" fn ex_checkpath(mut eap: *mut exarg_T) {
    find_pattern_in_path(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        kDirectionNotSet,
        0 as size_t,
        false_0 != 0,
        false_0 != 0,
        CHECK_PATH as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        if (*eap).forceit != 0 {
            ACTION_SHOW_ALL as ::core::ffi::c_int
        } else {
            ACTION_SHOW as ::core::ffi::c_int
        },
        1 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        (*eap).forceit != 0,
        false_0 != 0,
    );
}
unsafe extern "C" fn ex_psearch(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
    ex_findpat(eap);
    g_do_tagpreview.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_findpat(mut eap: *mut exarg_T) {
    let mut whole: bool = true_0 != 0;
    let mut action: ::core::ffi::c_int = 0;
    match *(*cmdnames.ptr())[(*eap).cmdidx as usize]
        .cmd_name
        .offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        101 => {
            if *(*cmdnames.ptr())[(*eap).cmdidx as usize]
                .cmd_name
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int
            {
                action = ACTION_GOTO as ::core::ffi::c_int;
            } else {
                action = ACTION_SHOW as ::core::ffi::c_int;
            }
        }
        105 => {
            action = ACTION_SHOW_ALL as ::core::ffi::c_int;
        }
        117 => {
            action = ACTION_GOTO as ::core::ffi::c_int;
        }
        _ => {
            action = ACTION_SPLIT as ::core::ffi::c_int;
        }
    }
    let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if ascii_isdigit(*(*eap).arg as ::core::ffi::c_int) {
        n = getdigits_int(&raw mut (*eap).arg, false_0 != 0, 0 as ::core::ffi::c_int);
        (*eap).arg = skipwhite((*eap).arg);
    }
    if *(*eap).arg as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        whole = false_0 != 0;
        (*eap).arg = (*eap).arg.offset(1);
        let mut p: *mut ::core::ffi::c_char = skip_regexp(
            (*eap).arg,
            '/' as ::core::ffi::c_int,
            magic_isset() as ::core::ffi::c_int,
        );
        if *p != 0 {
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = NUL as ::core::ffi::c_char;
            p = skipwhite(p);
            if ends_excmd(*p as ::core::ffi::c_int) == 0 {
                (*eap).errmsg =
                    ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, p);
            } else {
                (*eap).nextcmd = check_nextcmd(p);
            }
        }
    }
    if (*eap).skip == 0 {
        find_pattern_in_path(
            (*eap).arg,
            kDirectionNotSet,
            strlen((*eap).arg),
            whole,
            (*eap).forceit == 0,
            if *(*eap).cmd as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                FIND_DEFINE as ::core::ffi::c_int
            } else {
                FIND_ANY as ::core::ffi::c_int
            },
            n,
            action,
            (*eap).line1,
            (*eap).line2,
            (*eap).forceit != 0,
            false_0 != 0,
        );
    }
}
unsafe extern "C" fn ex_ptag(mut eap: *mut exarg_T) {
    g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as ::core::ffi::c_int as isize),
    );
}
unsafe extern "C" fn ex_pedit(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exedit(eap, ::core::ptr::null_mut::<win_T>());
    back_to_current_window(curwin_save);
}
unsafe extern "C" fn ex_pbuffer(mut eap: *mut exarg_T) {
    let mut curwin_save: *mut win_T = curwin.get();
    prepare_preview_window();
    do_exbuffer(eap);
    back_to_current_window(curwin_save);
}
unsafe extern "C" fn prepare_preview_window() {
    g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
    prepare_tagpreview(true_0 != 0);
}
unsafe extern "C" fn back_to_current_window(mut curwin_save: *mut win_T) {
    if curwin.get() != curwin_save && win_valid(curwin_save) as ::core::ffi::c_int != 0 {
        validate_cursor(curwin.get());
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
        win_enter(curwin_save, true_0 != 0);
    }
    g_do_tagpreview.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_stag(mut eap: *mut exarg_T) {
    postponed_split.set(-1 as ::core::ffi::c_int);
    postponed_split_flags.set((*cmdmod.ptr()).cmod_split);
    postponed_split_tab.set((*cmdmod.ptr()).cmod_tab);
    ex_tag_cmd(
        eap,
        (*cmdnames.ptr())[(*eap).cmdidx as usize]
            .cmd_name
            .offset(1 as ::core::ffi::c_int as isize),
    );
    postponed_split_flags.set(0 as ::core::ffi::c_int);
    postponed_split_tab.set(0 as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_tag(mut eap: *mut exarg_T) {
    ex_tag_cmd(eap, (*cmdnames.ptr())[(*eap).cmdidx as usize].cmd_name);
}
unsafe extern "C" fn ex_tag_cmd(mut eap: *mut exarg_T, mut name: *const ::core::ffi::c_char) {
    let mut cmd: ::core::ffi::c_int = 0;
    match *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
        106 => {
            cmd = DT_JUMP as ::core::ffi::c_int;
        }
        115 => {
            cmd = DT_SELECT as ::core::ffi::c_int;
        }
        112 | 78 => {
            cmd = DT_PREV as ::core::ffi::c_int;
        }
        110 => {
            cmd = DT_NEXT as ::core::ffi::c_int;
        }
        111 => {
            cmd = DT_POP as ::core::ffi::c_int;
        }
        102 | 114 => {
            cmd = DT_FIRST as ::core::ffi::c_int;
        }
        108 => {
            cmd = DT_LAST as ::core::ffi::c_int;
        }
        _ => {
            cmd = DT_TAG as ::core::ffi::c_int;
        }
    }
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'l' as ::core::ffi::c_int
    {
        cmd = DT_LTAG as ::core::ffi::c_int;
    }
    do_tag(
        (*eap).arg,
        cmd,
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        },
        (*eap).forceit,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_cmdline_var(
    mut src: *const ::core::ffi::c_char,
    mut usedlen: *mut size_t,
) -> ssize_t {
    static spec_str: GlobalCell<[*mut ::core::ffi::c_char; 15]> = GlobalCell::new([
        b"%\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"#\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<cword>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<cWORD>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<cexpr>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<cfile>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<sfile>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<slnum>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<stack>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<script>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<afile>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<abuf>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<amatch>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<sflnum>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"<SID>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 15]>()
        .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut ::core::ffi::c_char; 15]>()
                .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut len: size_t = strlen((*spec_str.ptr())[i as usize] as *const ::core::ffi::c_char);
        if strncmp(
            src,
            (*spec_str.ptr())[i as usize] as *const ::core::ffi::c_char,
            len,
        ) == 0 as ::core::ffi::c_int
        {
            *usedlen = len;
            '_c2rust_label: {
                if i <= 9223372036854775807 as ::core::ffi::c_long as size_t {
                } else {
                    __assert_fail(
                        b"i <= SSIZE_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        7692 as ::core::ffi::c_uint,
                        b"ssize_t find_cmdline_var(const char *, size_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            return i as ssize_t;
        }
        i = i.wrapping_add(1);
    }
    return -1 as ssize_t;
}
#[no_mangle]
pub unsafe extern "C" fn eval_vars(
    mut src: *mut ::core::ffi::c_char,
    mut srcstart: *const ::core::ffi::c_char,
    mut usedlen: *mut size_t,
    mut lnump: *mut linenr_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
    mut escaped: *mut ::core::ffi::c_int,
    mut empty_is_error: bool,
) -> *mut ::core::ffi::c_char {
    let mut result: *mut ::core::ffi::c_char =
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    let mut resultbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut resultlen: size_t = 0;
    let mut valid: ::core::ffi::c_int =
        VALID_HEAD as ::core::ffi::c_int | VALID_PATH as ::core::ffi::c_int;
    let mut tilde_file: bool = false_0 != 0;
    let mut skip_mod: bool = false_0 != 0;
    let mut strbuf: [::core::ffi::c_char; 30] = [0; 30];
    *errormsg = ::core::ptr::null::<::core::ffi::c_char>();
    if !escaped.is_null() {
        *escaped = false_0;
    }
    let mut spec_idx: ssize_t = find_cmdline_var(src, usedlen);
    if spec_idx < 0 as ssize_t {
        *usedlen = 1 as size_t;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if src > srcstart as *mut ::core::ffi::c_char
        && *src.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
    {
        *usedlen = 0 as size_t;
        memmove(
            src.offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            strlen(src).wrapping_add(1 as size_t),
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if spec_idx == SPEC_CWORD as ::core::ffi::c_int as ssize_t
        || spec_idx == SPEC_CCWORD as ::core::ffi::c_int as ssize_t
        || spec_idx == SPEC_CEXPR as ::core::ffi::c_int as ssize_t
    {
        resultlen = find_ident_under_cursor(
            &raw mut result,
            if spec_idx == SPEC_CWORD as ::core::ffi::c_int as ssize_t {
                FIND_IDENT as ::core::ffi::c_int | FIND_STRING as ::core::ffi::c_int
            } else if spec_idx == SPEC_CEXPR as ::core::ffi::c_int as ssize_t {
                FIND_IDENT as ::core::ffi::c_int
                    | FIND_STRING as ::core::ffi::c_int
                    | FIND_EVAL as ::core::ffi::c_int
            } else {
                FIND_STRING as ::core::ffi::c_int
            },
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if resultlen == 0 as size_t {
            *errormsg = b"\0".as_ptr() as *const ::core::ffi::c_char;
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    } else {
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: ::core::ffi::c_int = 0;
        match spec_idx {
            0 => {
                if (*curbuf.get()).b_fname.is_null() {
                    result =
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                    valid = 0 as ::core::ffi::c_int;
                } else {
                    result = (*curbuf.get()).b_fname;
                    tilde_file = strcmp(result, b"~\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int;
                }
            }
            1 => {
                if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '#' as ::core::ffi::c_int
                {
                    result = arg_all();
                    resultbuf = result;
                    *usedlen = 2 as size_t;
                    if !escaped.is_null() {
                        *escaped = true_0;
                    }
                    skip_mod = true_0 != 0;
                } else {
                    s = src.offset(1 as ::core::ffi::c_int as isize);
                    if *s as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                        s = s.offset(1);
                    }
                    i = getdigits_int(&raw mut s, false_0 != 0, 0 as ::core::ffi::c_int);
                    if s == src.offset(2 as ::core::ffi::c_int as isize)
                        && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '-' as ::core::ffi::c_int
                    {
                        s = s.offset(-1);
                    }
                    *usedlen = s.offset_from(src) as size_t;
                    if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '<' as ::core::ffi::c_int
                        && i != 0 as ::core::ffi::c_int
                    {
                        if *usedlen < 2 as size_t {
                            *usedlen = 1 as size_t;
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        result = tv_list_find_str(
                            get_vim_var_list(VV_OLDFILES),
                            i - 1 as ::core::ffi::c_int,
                        ) as *mut ::core::ffi::c_char;
                        if result.is_null() {
                            *errormsg = b"\0".as_ptr() as *const ::core::ffi::c_char;
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                    } else {
                        if i == 0 as ::core::ffi::c_int
                            && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '<' as ::core::ffi::c_int
                            && *usedlen > 1 as size_t
                        {
                            *usedlen = 1 as size_t;
                        }
                        let mut buf: *mut buf_T = buflist_findnr(i);
                        if buf.is_null() {
                            *errormsg = gettext(
                                b"E194: No alternate file name to substitute for '#'\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        if !lnump.is_null() {
                            *lnump = ECMD_LAST as ::core::ffi::c_int as linenr_T;
                        }
                        if (*buf).b_fname.is_null() {
                            result = b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                            valid = 0 as ::core::ffi::c_int;
                        } else {
                            result = (*buf).b_fname;
                            tilde_file =
                                strcmp(result, b"~\0".as_ptr() as *const ::core::ffi::c_char)
                                    == 0 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            5 => {
                result = file_name_at_cursor(
                    FNAME_MESS as ::core::ffi::c_int | FNAME_HYP as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<linenr_T>(),
                );
                if result.is_null() {
                    *errormsg = b"\0".as_ptr() as *const ::core::ffi::c_char;
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                resultbuf = result;
            }
            10 => {
                if !(*autocmd_fname.ptr()).is_null() && !autocmd_fname_full.get() {
                    autocmd_fname_full.set(true_0 != 0);
                    result = FullName_save(autocmd_fname.get(), false_0 != 0);
                    xstrlcpy(autocmd_fname.get(), result, MAXPATHL as size_t);
                    xfree(result as *mut ::core::ffi::c_void);
                }
                result = autocmd_fname.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_file_name_to_substitute_for_afile.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                result = path_try_shorten_fname(result);
            }
            11 => {
                if autocmd_bufnr.get() <= 0 as ::core::ffi::c_int {
                    *errormsg = gettext(
                        (e_no_autocommand_buffer_number_to_substitute_for_abuf.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    autocmd_bufnr.get(),
                );
                result = &raw mut strbuf as *mut ::core::ffi::c_char;
            }
            12 => {
                result = autocmd_match.get();
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_autocommand_match_name_to_substitute_for_amatch.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
            6 => {
                result = estack_sfile(ESTACK_SFILE);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_source_file_name_to_substitute_for_sfile.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                resultbuf = result;
            }
            8 => {
                result = estack_sfile(ESTACK_STACK);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_call_stack_to_substitute_for_stack.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                resultbuf = result;
            }
            9 => {
                result = estack_sfile(ESTACK_SCRIPT);
                if result.is_null() {
                    *errormsg = gettext(
                        (e_no_script_file_name_to_substitute_for_script.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                resultbuf = result;
            }
            7 => {
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                    || (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum
                        == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_slnum.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum,
                );
                result = &raw mut strbuf as *mut ::core::ffi::c_char;
            }
            13 => {
                if (*current_sctx.ptr()).sc_lnum
                    + (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum
                    == 0 as linenr_T
                {
                    *errormsg = gettext(
                        (e_no_line_number_to_use_for_sflnum.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    );
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*current_sctx.ptr()).sc_lnum
                        + (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum,
                );
                result = &raw mut strbuf as *mut ::core::ffi::c_char;
            }
            14 => {
                if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int {
                    *errormsg = gettext(&raw const e_usingsid as *const ::core::ffi::c_char);
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                snprintf(
                    &raw mut strbuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"<SNR>%d_\0".as_ptr() as *const ::core::ffi::c_char,
                    (*current_sctx.ptr()).sc_sid,
                );
                result = &raw mut strbuf as *mut ::core::ffi::c_char;
            }
            _ => {
                *errormsg = b"\0".as_ptr() as *const ::core::ffi::c_char;
            }
        }
        resultlen = strlen(result);
        if *src.offset(*usedlen as isize) as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            *usedlen = (*usedlen).wrapping_add(1);
            let mut s_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            s_0 = strrchr(result, '.' as ::core::ffi::c_int);
            if !s_0.is_null() && s_0 >= path_tail(result) {
                resultlen = s_0.offset_from(result) as size_t;
            }
        } else if !skip_mod {
            valid |= modify_fname(
                src,
                tilde_file,
                usedlen,
                &raw mut result,
                &raw mut resultbuf,
                &raw mut resultlen,
            );
            if result.is_null() {
                *errormsg = b"\0".as_ptr() as *const ::core::ffi::c_char;
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
    }
    if resultlen == 0 as size_t
        || valid != VALID_HEAD as ::core::ffi::c_int + VALID_PATH as ::core::ffi::c_int
    {
        if empty_is_error {
            if valid != VALID_HEAD as ::core::ffi::c_int + VALID_PATH as ::core::ffi::c_int {
                *errormsg = gettext(
                    b"E499: Empty file name for '%' or '#', only works with \":p:h\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            } else {
                *errormsg =
                    gettext(b"E500: Evaluates to an empty string\0".as_ptr()
                        as *const ::core::ffi::c_char);
            }
        }
        result = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        result =
            xmemdupz(result as *const ::core::ffi::c_void, resultlen) as *mut ::core::ffi::c_char;
    }
    xfree(resultbuf as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn expand_sfile(
    mut arg: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut result: *mut ::core::ffi::c_char = xstrdup(arg);
    let mut p: *mut ::core::ffi::c_char = result;
    while *p != 0 {
        if strncmp(
            p,
            b"<sfile>\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) != 0 as ::core::ffi::c_int
        {
            p = p.offset(1);
        } else {
            let mut srclen: size_t = 0;
            let mut errormsg: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut repl: *mut ::core::ffi::c_char = eval_vars(
                p,
                result,
                &raw mut srclen,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut errormsg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                true_0 != 0,
            );
            if !errormsg.is_null() {
                if *errormsg != 0 {
                    emsg(errormsg);
                }
                xfree(result as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if repl.is_null() {
                p = p.offset(srclen as isize);
            } else {
                let mut len: size_t = strlen(result)
                    .wrapping_sub(srclen)
                    .wrapping_add(strlen(repl))
                    .wrapping_add(1 as size_t);
                let mut newres: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
                memmove(
                    newres as *mut ::core::ffi::c_void,
                    result as *const ::core::ffi::c_void,
                    p.offset_from(result) as size_t,
                );
                strcpy(newres.offset(p.offset_from(result) as isize), repl);
                len = strlen(newres);
                strcat(newres, p.offset(srclen as isize));
                xfree(repl as *mut ::core::ffi::c_void);
                xfree(result as *mut ::core::ffi::c_void);
                result = newres;
                p = newres.offset(len as isize);
            }
        }
    }
    return result;
}
unsafe extern "C" fn ex_shada(mut eap: *mut exarg_T) {
    let mut save_shada: *mut ::core::ffi::c_char = p_shada.get();
    if *p_shada.get() as ::core::ffi::c_int == NUL {
        p_shada.set(b"'100\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_rviminfo as ::core::ffi::c_int
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_rshada as ::core::ffi::c_int
    {
        shada_read_everything((*eap).arg, (*eap).forceit != 0, false_0 != 0);
    } else {
        shada_write_file((*eap).arg, (*eap).forceit != 0);
    }
    p_shada.set(save_shada);
}
#[no_mangle]
pub unsafe extern "C" fn dialog_msg(
    mut buff: *mut ::core::ffi::c_char,
    mut format: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
) {
    if fname.is_null() {
        fname = gettext(b"Untitled\0".as_ptr() as *const ::core::ffi::c_char);
    }
    vim_snprintf(
        buff,
        DIALOG_MSG_SIZE as ::core::ffi::c_int as size_t,
        format,
        fname,
    );
}
static filetype_detect: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_plugin: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_indent: GlobalCell<TriState> = GlobalCell::new(kNone);
unsafe extern "C" fn ex_filetype(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        smsg(
            0 as ::core::ffi::c_int,
            b"filetype detection:%s  plugin:%s  indent:%s\0".as_ptr() as *const ::core::ffi::c_char,
            if filetype_detect.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                b"ON\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"OFF\0".as_ptr() as *const ::core::ffi::c_char
            },
            if filetype_plugin.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                if filetype_detect.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                    b"ON\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"(on)\0".as_ptr() as *const ::core::ffi::c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const ::core::ffi::c_char
            },
            if filetype_indent.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                if filetype_detect.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                    b"ON\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"(on)\0".as_ptr() as *const ::core::ffi::c_char
                }
            } else {
                b"OFF\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut plugin: bool = false_0 != 0;
    let mut indent: bool = false_0 != 0;
    loop {
        if strncmp(
            arg,
            b"plugin\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            plugin = true_0 != 0;
            arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
        } else {
            if strncmp(
                arg,
                b"indent\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            indent = true_0 != 0;
            arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
        }
    }
    if strcmp(arg, b"on\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        || strcmp(arg, b"detect\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        if *arg as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
            || filetype_detect.get() as ::core::ffi::c_int != kTrue as ::core::ffi::c_int
        {
            source_runtime(
                FILETYPE_FILE.as_ptr() as *mut ::core::ffi::c_char,
                DIP_ALL as ::core::ffi::c_int,
            );
            filetype_detect.set(kTrue);
            if plugin {
                source_runtime(
                    FTPLUGIN_FILE.as_ptr() as *mut ::core::ffi::c_char,
                    DIP_ALL as ::core::ffi::c_int,
                );
                filetype_plugin.set(kTrue);
            }
            if indent {
                source_runtime(
                    INDENT_FILE.as_ptr() as *mut ::core::ffi::c_char,
                    DIP_ALL as ::core::ffi::c_int,
                );
                filetype_indent.set(kTrue);
            }
        }
        if *arg as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
            do_doautocmd(
                b"filetypedetect BufRead\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            do_modelines(0 as ::core::ffi::c_int);
        }
    } else if strcmp(arg, b"off\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        if plugin as ::core::ffi::c_int != 0 || indent as ::core::ffi::c_int != 0 {
            if plugin {
                source_runtime(
                    FTPLUGOF_FILE.as_ptr() as *mut ::core::ffi::c_char,
                    DIP_ALL as ::core::ffi::c_int,
                );
                filetype_plugin.set(kFalse);
            }
            if indent {
                source_runtime(
                    INDOFF_FILE.as_ptr() as *mut ::core::ffi::c_char,
                    DIP_ALL as ::core::ffi::c_int,
                );
                filetype_indent.set(kFalse);
            }
        } else {
            source_runtime(
                FTOFF_FILE.as_ptr() as *mut ::core::ffi::c_char,
                DIP_ALL as ::core::ffi::c_int,
            );
            filetype_detect.set(kFalse);
        }
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn filetype_plugin_enable() {
    if filetype_plugin.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        source_runtime(
            FTPLUGIN_FILE.as_ptr() as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int,
        );
        filetype_plugin.set(kTrue);
    }
    if filetype_indent.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        source_runtime(
            INDENT_FILE.as_ptr() as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int,
        );
        filetype_indent.set(kTrue);
    }
}
#[no_mangle]
pub unsafe extern "C" fn filetype_maybe_enable() {
    if filetype_detect.get() as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
        source_runtime(
            FILETYPE_FILE.as_ptr() as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int,
        );
        filetype_detect.set(kTrue);
    }
}
unsafe extern "C" fn ex_setfiletype(mut eap: *mut exarg_T) {
    if (*curbuf.get()).b_did_filetype {
        return;
    }
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    if strncmp(
        arg,
        b"FALLBACK \0".as_ptr() as *const ::core::ffi::c_char,
        9 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(9 as ::core::ffi::c_int as isize);
    }
    set_option_value_give_err(
        kOptFiletype,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(arg),
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    if arg != (*eap).arg {
        (*curbuf.get()).b_did_filetype = false_0 != 0;
    }
}
unsafe extern "C" fn ex_digraphs(mut eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        putdigraph((*eap).arg);
    } else {
        listdigraphs((*eap).forceit != 0);
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_no_hlsearch(mut flag: bool) {
    no_hlsearch.set(flag);
    set_vim_var_nr(
        VV_HLSEARCH,
        (!no_hlsearch.get() && p_hls.get() != 0) as ::core::ffi::c_int as varnumber_T,
    );
}
unsafe extern "C" fn ex_nohlsearch(mut _eap: *mut exarg_T) {
    set_no_hlsearch(true_0 != 0);
    redraw_all_later(UPD_SOME_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn ex_fold(mut eap: *mut exarg_T) {
    if foldManualAllowed(true_0 != 0) != 0 {
        let mut start: pos_T = pos_T {
            lnum: (*eap).line1,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        let mut end: pos_T = pos_T {
            lnum: (*eap).line2,
            col: 1 as colnr_T,
            coladd: 0 as colnr_T,
        };
        foldCreate(curwin.get(), start, end);
    }
}
unsafe extern "C" fn ex_foldopen(mut eap: *mut exarg_T) {
    let mut start: pos_T = pos_T {
        lnum: (*eap).line1,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    let mut end: pos_T = pos_T {
        lnum: (*eap).line2,
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    opFoldRange(
        start,
        end,
        ((*eap).cmdidx as ::core::ffi::c_int == CMD_foldopen as ::core::ffi::c_int)
            as ::core::ffi::c_int,
        (*eap).forceit,
        false_0 != 0,
    );
}
unsafe extern "C" fn ex_folddo(mut eap: *mut exarg_T) {
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= (*eap).line2 {
        if hasFolding(
            curwin.get(),
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            ::core::ptr::null_mut::<linenr_T>(),
        ) as ::core::ffi::c_int
            == ((*eap).cmdidx as ::core::ffi::c_int == CMD_folddoclosed as ::core::ffi::c_int)
                as ::core::ffi::c_int
        {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    global_exe((*eap).arg);
    ml_clearmarked();
}
#[no_mangle]
pub unsafe extern "C" fn is_loclist_cmd(mut cmdidx: ::core::ffi::c_int) -> bool {
    if cmdidx < 0 as ::core::ffi::c_int || cmdidx >= CMD_SIZE as ::core::ffi::c_int {
        return false_0 != 0;
    }
    return *(*cmdnames.ptr())[cmdidx as usize]
        .cmd_name
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'l' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn get_pressedreturn() -> bool {
    return ex_pressedreturn.get();
}
#[no_mangle]
pub unsafe extern "C" fn set_pressedreturn(mut val: bool) {
    ex_pressedreturn.set(val);
}
unsafe extern "C" fn ex_checkhealth(mut eap: *mut exarg_T) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut mods: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut mods_len: size_t = 0 as size_t;
    mods[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if (*cmdmod.ptr()).cmod_tab > 0 as ::core::ffi::c_int
        || (*cmdmod.ptr()).cmod_split != 0 as ::core::ffi::c_int
    {
        let mut multi_mods: bool = false_0 != 0;
        mods_len = add_win_cmd_modifiers(
            &raw mut mods as *mut ::core::ffi::c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        '_c2rust_label: {
            if mods_len < ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() {
            } else {
                __assert_fail(
                    b"mods_len < sizeof(mods)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    8263 as ::core::ffi::c_uint,
                    b"void ex_checkhealth(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    let c2rust_fresh23 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh23 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: String_0 {
                data: &raw mut mods as *mut ::core::ffi::c_char,
                size: mods_len,
            },
        },
    };
    let c2rust_fresh24 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh24 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: cstr_as_string((*eap).arg),
        },
    };
    nlua_exec(
        String_0 {
            data: b"vim.health._check(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
        return;
    }
    let mut vimruntime_env: *mut ::core::ffi::c_char =
        os_getenv_noalloc(b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char);
    if vimruntime_env.is_null() {
        emsg(gettext(
            b"E5009: $VIMRUNTIME is empty or unset\0".as_ptr() as *const ::core::ffi::c_char
        ));
    } else {
        let mut rtp_ok: bool = !strstr(p_rtp.get(), vimruntime_env).is_null();
        if rtp_ok {
            semsg(
                gettext(b"E5009: Invalid $VIMRUNTIME: %s\0".as_ptr() as *const ::core::ffi::c_char),
                vimruntime_env,
            );
        } else {
            emsg(gettext(
                b"E5009: Invalid 'runtimepath'\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    semsg_multiline(b"emsg\0".as_ptr() as *const ::core::ffi::c_char, err.msg);
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn ex_terminal(mut eap: *mut exarg_T) {
    let mut ex_cmd: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut len: size_t = 0 as size_t;
    if (*cmdmod.ptr()).cmod_tab > 0 as ::core::ffi::c_int
        || (*cmdmod.ptr()).cmod_split != 0 as ::core::ffi::c_int
    {
        let mut multi_mods: bool = false_0 != 0;
        ex_cmd[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        len = add_win_cmd_modifiers(
            &raw mut ex_cmd as *mut ::core::ffi::c_char,
            cmdmod.ptr(),
            &raw mut multi_mods,
        );
        '_c2rust_label: {
            if len < ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() {
            } else {
                __assert_fail(
                    b"len < sizeof(ex_cmd)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    8298 as ::core::ffi::c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut result: ::core::ffi::c_int = snprintf(
            (&raw mut ex_cmd as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>().wrapping_sub(len),
            b" new\0".as_ptr() as *const ::core::ffi::c_char,
        );
        '_c2rust_label_0: {
            if result > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"result > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    8300 as ::core::ffi::c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        len = len.wrapping_add(result as size_t);
    } else {
        let mut result_0: ::core::ffi::c_int = snprintf(
            &raw mut ex_cmd as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            b"enew%s\0".as_ptr() as *const ::core::ffi::c_char,
            if (*eap).forceit != 0 {
                b"!\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        '_c2rust_label_1: {
            if result_0 > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"result > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    8304 as ::core::ffi::c_uint,
                    b"void ex_terminal(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        len = len.wrapping_add(result_0 as size_t);
    }
    '_c2rust_label_2: {
        if len < ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() {
        } else {
            __assert_fail(
                b"len < sizeof(ex_cmd)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_docmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                8308 as ::core::ffi::c_uint,
                b"void ex_terminal(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        let mut name: *mut ::core::ffi::c_char =
            vim_strsave_escaped((*eap).arg, b"\"\\\0".as_ptr() as *const ::core::ffi::c_char);
        snprintf(
            (&raw mut ex_cmd as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>().wrapping_sub(len),
            b" | call jobstart(\"%s\",{'term':v:true})\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
        xfree(name as *mut ::core::ffi::c_void);
    } else {
        if *p_sh.get() as ::core::ffi::c_int == NUL {
            emsg(gettext(
                &raw const e_shellempty as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut argv: *mut *mut ::core::ffi::c_char = shell_build_argv(
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        let mut p: *mut *mut ::core::ffi::c_char = argv;
        let mut tempstring: [::core::ffi::c_char; 512] = [0; 512];
        let mut shell_argv: [::core::ffi::c_char; 512] = [
            0 as ::core::ffi::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        while !(*p).is_null() {
            let mut escaped: *mut ::core::ffi::c_char =
                vim_strsave_escaped(*p, b"\"\\\0".as_ptr() as *const ::core::ffi::c_char);
            snprintf(
                &raw mut tempstring as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>(),
                b",\"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                escaped,
            );
            xfree(escaped as *mut ::core::ffi::c_void);
            xstrlcat(
                &raw mut shell_argv as *mut ::core::ffi::c_char,
                &raw mut tempstring as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>(),
            );
            p = p.offset(1);
        }
        shell_free_argv(argv);
        snprintf(
            (&raw mut ex_cmd as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>().wrapping_sub(len),
            b" | call jobstart([%s], {'term':v:true})\0".as_ptr() as *const ::core::ffi::c_char,
            (&raw mut shell_argv as *mut ::core::ffi::c_char)
                .offset(1 as ::core::ffi::c_int as isize),
        );
    }
    do_cmdline_cmd(&raw mut ex_cmd as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn ex_lsp(mut eap: *mut exarg_T) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_14 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh22 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_14 {
            string: cstr_as_string((*eap).arg),
        },
    };
    nlua_exec(
        String_0 {
            data: b"require'vim._core.ex_cmd'.ex_lsp(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 38]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg_multiline(
            err.msg,
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E as ::core::ffi::c_int,
            true_0 != 0,
        );
    }
    api_clear_error(&raw mut err);
}
unsafe extern "C" fn ex_fclose(mut eap: *mut exarg_T) {
    win_float_remove((*eap).forceit != 0, (*eap).line1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn verify_command(mut cmd: *mut ::core::ffi::c_char) {
    if strcmp(b"smile\0".as_ptr() as *const ::core::ffi::c_char, cmd) != 0 as ::core::ffi::c_int {
        return;
    }
    let mut a: ::core::ffi::c_int = HLF_E as ::core::ffi::c_int;
    msg(
        b" #xxn`          #xnxx`        ,+x@##@Mz;`        .xxxxxxxxxnz+,      znnnnnnnnnnnnnnnn.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n###z          x####`      :x##########W+`      ,#############M;    W################.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####;         x####`    `z##############W:     ,################   W################.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####W.        x####`   ,W#################+    ,#################  W################.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n#####n        x####`   @###################    ,#################i W################.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n######i       x####`  .#########@W@########*   ,#################W`W################.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n######@.      x####`  x######W*.  `;n#######:  ,####x,,,,:*M######iW###@:,,,,,,,,,,,`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n#######n      x####` *######+`       :M#####M  ,####n      `x#####xW###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n########*     x####``@####@;          `x#####i ,####n       ,#####@W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n########@     x####`*#####i            `M####M ,####n        x#########@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n#########     x####`M####z              :#####:,####n        z#########@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n#########*    x####,#####.               n####+,####n        n#########@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####@####@,   x####i####x                ;####x,####n       `W#####@####+++++++++++i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####*#####M`  x#########*                `####@,####n       i#####MW###############W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.######+  x####z####;                 W####,####n      i@######W###############W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.`W#####: x####n####:                 M####:####@nnnnnW#######,W###############W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####. :#####M`x####z####;                 W####,#################z W###############W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.  #######x#########*                `####W,################W` W###############W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.  `M#####W####i####x                ;####x,###############W,  W####+**********i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.   ,##########,#####.               n####+,##############n.   W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.    ##########`M####z              :#####:,###########Wz:     W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.    x#########`*#####i            `M####M ,####x.....`        W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.    ,@########``@####@;          `x#####i ,####n              W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.     *########` *#####@+`       ,M#####M  ,####n              W###@`\0".as_ptr()
            as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.      x#######`  x######W*.  `;n######@:  ,####n              W###@,,,,,,,,,,,,`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.      .@######`  .#########@W@########*   ,####n              W################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.       i######`   @###################    ,####n              W################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.        n#####`   ,W#################+    ,####n              W################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.        .@####`    .n##############W;     ,####n              W################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" n####.         i####`      :x##########W+`      ,####n              W################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" +nnnn`          +nnn`        ,+x@##@Mz;`        .nnnn+              zxxxxxxxxxxxxxxxx.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(b" \0".as_ptr() as *const ::core::ffi::c_char, a);
    msg(
        b"                                                                                   ,+M@#Mi\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"                                                                                 .z########\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"                                                                                i@#########i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"                                                                              `############W`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"                                                                             `n#############i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"                                                                            `n##############n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     ``                                                                     z###############@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    `W@z,                                                                  ##################,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    *#####`                                                               i############@x@###i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    ######M.                                                             :#############n`,W##+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    +######@:                                                           .W#########M@##+  *##z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    :#######@:                                                         `x########@#x###*  ,##n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    `@#######@;                                                        z#########M*@nW#i  .##x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     z########@i                                                      *###########WM#@#,  `##x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     i##########+                                                    ;###########*n###@   `##x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     `@#MM#######x,                                                 ,@#########zM,`z##M   `@#x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      n##M#W#######n.               `.:i*+#zzzz##+i:.`             ,W#########Wii,`n@#@` n@##n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ;###@#x#######n         `,i#nW@#####@@WWW@@####@Mzi.        ,W##########@z.. ;zM#+i####z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       x####nz########    .;#x@##@Wn#*;,.`      ``,:*#x@##M+,    ;@########xz@WM+#` `n@#######\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       ,@####M########xi#@##@Mzi,`                     .+x###Mi:n##########Mz```.:i  *@######*\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        *#####W#########ix+:`                             :n#############z:       `*.`M######i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        i#W##nW@+@##@#M@;                                   ;W@@##########W,        i`x@#####,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        `@@n@Wn#@iMW*#*:                                     `iz#z@######x.           M######`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"         z##zM###x`*, .`                                          `iW#####W;:`        +#####M\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"         ,###nn##n`                                                ,#####x;`        ,;@######\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"          x###xz#.                                                   in###+        `:######@.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"          ;####n+                                                    `Mnx##xi`   , zM#######\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"          `W####+                i.                                   `.+x###@#. :n,z######:\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           z####@`              ;#:                                     .ii@###@;.*M*z####@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           i####M         `   `i@#,           ::                           +#n##@+@##W####n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           :####x    ,i. ##xzM###@`     i.   .@@,                           .z####x#######*\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           ,###W;   i##Wz#########     :##   z##n                           ,@########x###:\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"            n##n   `W###########M`;n,  i#x  ,###@i                           *W########W#@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           .@##+  `x###########@. z#+ .M#W``x#####n`                         `;#######@z#x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"           n###z :W############@  z#*  @##xM#######@n;                        `########nW+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"          ;####nW##############W :@#* `@#############*                        :########z@i`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"          M##################### M##:  @#############@:                       *W########M#\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"         ;#####################i.##x`  W#############W,                       :n########zx\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"         x####################@.`x;    @#############z.                       .@########W#\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        ,######################`       W###############x*,`                    W######zM#i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        #######################:       z##################@x+*#zzi            `@#########.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"        W########W#z#M#########;       *##########################z            :@#######@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       `@#######x`;#z ,x#######;       z###########M###xnM@########*            :M######@\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       i########, x#@`  z######;       *##########i *#@`  `+########+`            n######.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       n#######@` M##,  `W#####.       *#########z  ###;    z########M:           :W####n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       M#######M  n##.   x####x        `x########:  z##+    M#########@;           .n###+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       W#######@` :#W   `@####:         `@######W   i###   ;###########@.            n##n\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       W########z` ,,  .x####z           @######@`  `W#;  `W############*            *###;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      `@#########Mi,:*n@####W`           W#######*   ..  `n#############i            i###x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .#####################z           `@#######@*`    .x############n:`            ;####.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      :####################x`,,`        `W#########@x#+#@#############i              ,####:\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ;###################x#@###xi`      *############################:              `####i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      i##################+########M,      x##########################@`               W###i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *################@; @########@,     .W#########################@                x###:\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .+M#############z.  M#########x      ,W########################@`               ####.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *M*;z@########x:    :W#######i        .M########################i               i###:\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *##@z;#@####x:        :z###@i          `########################x               .###;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *#####n;#@##            ;##*             ,x#####################@`               W##*\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *#######n;*            :M##W*,             *W####################`               n##z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      i########@.         ,*n#######M*`           `###################M                *##M\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      i########n        `z#####@@#####Wi            ,M################;                ,##@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ;WMWW@###*       .x##@ni.``.:+zW##z`           `n##############z                  @##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .*++*i;;;.      .M#@+`          .##n            `x############x`                  n##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      :########*      x#W,              *#+            *###########M`                   +##+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ,#########     :#@:                ##:           #nzzzzzzzzzz.                    :##x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .#####Wz+`     ##+                 `MM`          .znnnnnnnnn.                     `@#@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      `@@ni;*nMz`    @W`                  :#+           .x#######n                       x##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       i;z@#####,   .#*                    z#:           ;;;*zW##;                       ###i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       z########:   :#;                    `Wx          +###Wni;n.                       ;##z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"       n########W:  .#*                     ,#,        ;#######@+                        `@#M\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .###########n;.MM                      n*        ;iM#######*                        x#@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      :#############@;;                      .n`      ,#W*iW#####W`                       +##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ,##############.                        ix.    `x###M;#######                       ,##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .#############@`                         x@n**#W######z;M###@.                       W##\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      .##############W:                        .x############@*;zW#;                       z#x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ,###############@;                        `##############@n*;.                       i#@\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ,#################i                         :n##############W`                       .##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ,###################`                         .+W##########W,                        `##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      :###################@zi,`                        ;zM@@@WMn*`                          @#z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      :#######################@x+*i;;:i#M,                 ``                               M#W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ;################################@x.                                                  n##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      i#####################@W@@@@Wxz*:`                                                    *##+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      *######################+```                                                           :##M\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      ########################M;                                                            `@##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      z#########################x,                                                           z###\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      n###########################n:                                                         ;##W`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      x#############################Mz#++##*                                                 `W##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      M####################################@`                                                 ###x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      W#####################################`                                                 .###,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      @####################################M                                                   n##z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"      @##################z*i@WMMMx#x@#####,.                                                   :##@.\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     `#####################@xi`     `::,*                                                       x##+\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     .#####################@#M.                                                                 ;##@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     ,#####################:.                                                                    M##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     ;###################ni`                                                                     i##M\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     *#################W#`                                                                       `W##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     z#################@Wx+.                                                                      +###\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"     x######################z.                                                                    .@#@`\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    `@#######################@;                                                                    z##;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    :##########################:                                                                   :##z\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    +#########################W#                                                                    M#W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"    W################@n+*i;:,`                                                                      +##,\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"   :##################WMxz+,                                                                        ,##i\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"   n#######################W..,                                                                      W##\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"  +#########################WW@+. .:.                                                                z#x\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" `@#############################@@###:                                                               *#W\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b" #################################Wz:                                                                :#@\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b",@###############################i                                                                   .##\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"n@@@@@@@#########################+                                                                   `##\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
    msg(
        b"`      `.:.`.,:iii;;;;;;;;iii;;;:`       `.``                                                        `nW\0"
            .as_ptr() as *const ::core::ffi::c_char,
        a,
    );
}
#[no_mangle]
pub unsafe extern "C" fn is_map_cmd(mut cmdidx: cmdidx_T) -> bool {
    if (cmdidx as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut func: ex_func_T = (*cmdnames.ptr())[cmdidx as usize].cmd_func;
    return func == Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())
        || func == Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())
        || func == Some(ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> ())
        || func == Some(ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> ())
        || func == Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ());
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const FILETYPE_FILE: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"filetype.lua filetype.vim\0")
};
pub const FTPLUGIN_FILE: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"ftplugin.vim\0") };
pub const INDENT_FILE: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"indent.vim\0") };
pub const FTOFF_FILE: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"ftoff.vim\0") };
pub const FTPLUGOF_FILE: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"ftplugof.vim\0") };
pub const INDOFF_FILE: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"indoff.vim\0") };
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
static command_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(557 as ::core::ffi::c_int);
static cmdnames: GlobalCell<[CommandDefinition; 557]> = GlobalCell::new(unsafe {
    [
        CommandDefinition {
            cmd_name: b"append\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_append as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18354435 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"abbreviate\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"abclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_all as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"amenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"anoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"args\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_argadd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4367 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdelete\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argdelete as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 271 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argdedupe\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argdedupe as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argedit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_argedit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 151951 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"argglobal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"arglocal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_args as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147726 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"argument\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argument as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"ascii\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_ascii as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"autocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_autocmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17311750 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"augroup\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_autocmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"aunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"buffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_buffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ball\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"badd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17318300 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"balt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17318300 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bdelete\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 34055 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"belowright\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"blast\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_blast as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"bmodified\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bmodified as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"bnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"botright\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17667 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"brewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16643 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"break\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_break as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breakadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakadd as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breakdel\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakdel as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"breaklist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breaklist as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"browse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303684 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"buffers\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"bufdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bunload\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 34055 as uint32_t,
            cmd_addr_type: ADDR_LOADED_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"bwipeout\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bunload as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 99591 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"change\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_change as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18351427 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cNfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cabclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cabove\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"caddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"caddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"caddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cafter\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"call\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_call as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565829 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"catch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_catch as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563652 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 279 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cbefore\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cbelow\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cbottom\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbottom as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX,
        },
        CommandDefinition {
            cmd_name: b"cclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"center\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2198 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cfdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"cfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"cgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"chdir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"changes\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_changes as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checkhealth\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checkhealth as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 260 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checkpath\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checkpath as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"checktime\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_checktime as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 34053 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"chistory\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"clist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_list as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"clast\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"close\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_close as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302787 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"clearjumps\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_clearjumps as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cmenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnewer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cnoreabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cnoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"copy\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"colder\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"colorscheme\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_colorscheme as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"command\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_command as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17311750 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"comclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_comclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"compiler\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_compiler as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"continue\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_continue as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303684 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"connect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_connect as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2206 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"const\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_let as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"copen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_copen as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cpfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"cquit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> !>,
                ex_func_T,
            >(Some(ex_cquit as unsafe extern "C" fn(*mut exarg_T) -> !)),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"crewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"cunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cunabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"cwindow\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cwindow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"delete\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18351937 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"delmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delmarks as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"debug\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_debug as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"debuggreedy\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_debuggreedy as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17305857 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"defer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_call as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"delcommand\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delcommand as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"delfunction\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_delfunction as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301654 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"detach\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_detach as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"display\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_display as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565956 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffupdate\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffupdate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffget\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffgetput as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1052933 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"diffoff\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_diffoff as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffpatch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffpatch as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1048860 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffput\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffgetput as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 4357 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"diffsplit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffsplit as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"diffthis\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_diffthis as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"digraphs\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_digraphs as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"djump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"dlist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"doautocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_doautocmd as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"doautoall\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_doautoall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"drop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_drop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147854 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"dsearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"dsplit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"edit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"earlier\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_later as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echoerr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echohl\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echohl as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echomsg\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"echon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_echo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"else\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_else as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"elseif\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_else as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"emenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_emenu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303941 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"endif\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_endif as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endfunction\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endfunction as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endfor\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endwhile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endtry\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_endtry as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"endwhile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_endwhile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"enew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"eval\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_eval as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ex\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"execute\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_execute as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"exit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"exusage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exusage as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4383 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"files\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"filetype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_filetype as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"filter\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2182 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"find\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_find as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147871 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"finally\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_finally as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"finish\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_finish as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"first\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"fold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_fold as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563969 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"foldclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_foldopen as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563971 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"folddoopen\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_folddo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"folddoclosed\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_folddo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"foldopen\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_foldopen as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563971 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"for\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_while as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"function\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_function as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563654 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"fclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_fclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 259 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"global\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_global as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563751 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"goto\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_goto as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564929 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"grep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"grepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"gui\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_nogui as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17449230 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"gvim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_nogui as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17449230 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"help\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_help as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2054 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helpclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpclose as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helpgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpgrep as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"helptags\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helptags as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301900 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"highlight\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_highlight as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_hide as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1287 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"history\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"horizontal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"insert\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_append as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350339 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"iabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iabclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_abclear as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"if\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_if as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ijump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"ilist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"imap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"imapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"imenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"inoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"inoreabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"inoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"intro\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_intro as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iput\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_iput as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18355011 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"isearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301607 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"isplit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_findpat as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"iunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iunabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"iunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"join\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_join as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 20448579 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"jumps\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_jumps as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"k\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mark as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563925 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lNfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"last\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_last as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"labove\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"language\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_language as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"laddexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"laddbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"laddfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lafter\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"later\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_later as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 279 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lbefore\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lbelow\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbelow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lbottom\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbottom as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lcd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lchdir\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ldo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"left\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"leftabove\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"let\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_let as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lexpr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2198 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lfile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lfdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2215 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX_VALID,
        },
        CommandDefinition {
            cmd_name: b"lfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lgetfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cfile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lgetbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 277 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lgetexpr\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cexpr as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2196 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lgrep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2447 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lhelpgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_helpgrep as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lhistory\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_history as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"ll\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_QUICKFIX,
        },
        CommandDefinition {
            cmd_name: b"llast\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"llist\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_list as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lmake\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2318 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lnewer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lnfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"loadview\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_loadview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 284 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"loadkeymap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_loadkeymap as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301504 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lockvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lockvar as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lolder\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(qf_age as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lopen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_copen as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"lpfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lrewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_UNSIGNED,
        },
        CommandDefinition {
            cmd_name: b"ltag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lua\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lua as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301509 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"luado\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_luado as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"luafile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_luafile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"lvimgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lvimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"lwindow\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cwindow as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ls\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                buflist_list as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"lsp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lsp as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 132 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"move\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"mark\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mark as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563925 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"make\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_make as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2318 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"map\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"marks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_marks as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"match\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_match as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301509 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"menu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316103 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"menutranslate\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_menutranslate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"messages\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_messages as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301765 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"mkexrc\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mksession\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkspell\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkspell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2446 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkvimrc\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mkview\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mkrc as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_mode as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"mzscheme\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_script_ni as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563813 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"mzfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"next\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_next as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147727 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"new\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"nmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nmenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"nnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nnoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"noremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nohlsearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_nohlsearch as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noreabbrev\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"noremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316103 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"normal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_normal as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17574023 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"number\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"nunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"nunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"oldfiles\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_oldfiles as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563906 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"omenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"only\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_only as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"onoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"onoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"options\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_options as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ounmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ounmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ownsyntax\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_ownsyntax as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"print\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19662145 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"packadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_packadd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564062 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"packloadall\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_packloadall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17563906 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"pbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pbuffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"pclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pclose as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"perl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_perl as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563813 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"perldo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_perldo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"perlfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_perlfile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pedit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pedit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"pop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"popup\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_popup as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303942 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ppop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 5379 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"preserve\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_preserve as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"previous\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"profile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_profile as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301766 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"profdel\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_breakdel as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"psearch\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_psearch as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 103 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"ptag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptNext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptjump\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ptlast\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ptnext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptrewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"ptselect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ptag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"put\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_put as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18355011 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pwd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pwd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"python\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pydo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3do\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"python3\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"py3file\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyx\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyxdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_pydo3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pythonx\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_python3 as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"pyxfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_py3file as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"quit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302787 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"quitall\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quitall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"qall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_quitall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"read\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_read as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18485599 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"recover\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_recover as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 286 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redir as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301774 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redraw\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_redraw as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redrawstatus\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_redrawstatus as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"redrawtabline\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_redrawtabline as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"registers\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_display as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565956 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"resize\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_resize as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301781 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"restart\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_restart as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18436 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"retab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_retab as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"return\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_return as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"right\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_align as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rshada\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"runtime\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_runtime as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17564046 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"rundo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rundo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 156 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"ruby\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ruby as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rubydo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rubydo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rubyfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_rubyfile as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"rviminfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"substitute\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_substitute_preview
                    as unsafe extern "C" fn(
                        *mut exarg_T,
                        ::core::ffi::c_int,
                        handle_T,
                    ) -> ::core::ffi::c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"sNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sargument\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_argument as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_ARGUMENTS,
        },
        CommandDefinition {
            cmd_name: b"sall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_all as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1283 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sandbox\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"saveas\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_write as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432862 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sbuffer\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_buffer as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 115975 as uint32_t,
            cmd_addr_type: ADDR_BUFFERS,
        },
        CommandDefinition {
            cmd_name: b"sbNext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sball\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sblast\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_blast as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sbmodified\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bmodified as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbnext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_bprevious as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17665 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sbrewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_brewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16640 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"scriptnames\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_scriptnames as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17302799 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"scriptencoding\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_scriptencoding as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"set\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setfiletype\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_setfiletype as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301892 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setglobal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"setlocal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_set as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563910 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sfind\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147871 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"simalt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sign\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sign as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"silent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565830 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sleep\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sleep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302791 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"slast\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_last as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smagic\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_submagic as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_submagic_preview
                    as unsafe extern "C" fn(
                        *mut exarg_T,
                        ::core::ffi::c_int,
                        handle_T,
                    ) -> ::core::ffi::c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"smap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"smenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"snext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_next as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147727 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"snomagic\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_submagic as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: Some(
                ex_submagic_preview
                    as unsafe extern "C" fn(
                        *mut exarg_T,
                        ::core::ffi::c_int,
                        handle_T,
                    ) -> ::core::ffi::c_int,
            ),
            cmd_argt: 151519301 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"snoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"snoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"source\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_source as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563967 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"sort\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_sort as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1050727 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"split\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellgood\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spelldump\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spelldump as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellinfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spellinfo as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellrepall\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_spellrepall as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"spellrare\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellundo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"spellwrong\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_spell as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 391 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"srewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_rewind as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147718 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stop\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"startinsert\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"startgreplace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"startreplace\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_startinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stopinsert\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_stopinsert as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stjump\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"stselect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sunhide\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"sunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"suspend\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_stop as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301762 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"sview\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"swapname\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_swapname as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syntax\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_syntax as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17303556 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syntime\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_syntime as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301908 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"syncbind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_syncbind as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_copymove as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350405 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tcd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tchdir\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_cd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tag\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4375 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_tags as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 6277 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabclose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_tabclose as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17305879 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabdo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabedit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151839 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabfind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151967 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabmove\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabmove as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tablast\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabnext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabnew\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 151839 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabonly\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabonly as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17305879 as uint32_t,
            cmd_addr_type: ADDR_TABS,
        },
        CommandDefinition {
            cmd_name: b"tabprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS_RELATIVE,
        },
        CommandDefinition {
            cmd_name: b"tabNext\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4373 as uint32_t,
            cmd_addr_type: ADDR_TABS_RELATIVE,
        },
        CommandDefinition {
            cmd_name: b"tabrewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tabs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tabs as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tcl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_script_ni as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301637 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tcldo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301669 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"tclfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301661 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"terminal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_terminal as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301518 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tfirst\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"throw\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_throw as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563780 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tjump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tlast\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tlmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tlnoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tlunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tmenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"tnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"topleft\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"trewind\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 4355 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"trust\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_trust as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16777500 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"try\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_try as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563904 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tselect\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_tag as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 278 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"tunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"undo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_undo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17306883 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"undojoin\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_undojoin as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"undolist\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_undolist as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301760 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unabbreviate\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_abbreviate as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unhide\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_buffer_all as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 1281 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"uniq\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_uniq as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 1050727 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"unlet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unlet as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unlockvar\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_lockvar as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17563782 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312006 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"unsilent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565828 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"update\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_update as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"vglobal\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_global as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301605 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"version\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_version as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"verbose\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17565829 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vertical\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_wrongmodifier as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 2180 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"visual\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"view\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_edit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 147742 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vimgrep\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vimgrepadd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_vimgrep as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 16779663 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"viusage\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_viusage as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 256 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vmenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vnew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vnoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vsplit\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_splitview as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 147743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"vunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"vunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"write\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_write as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"wNext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131423 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432834 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"while\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_while as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17565700 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"winsize\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_winsize as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 388 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wincmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wincmd as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17302677 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"windo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_listdo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 2213 as uint32_t,
            cmd_addr_type: ADDR_WINDOWS,
        },
        CommandDefinition {
            cmd_name: b"winpos\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_ni as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wnext\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131359 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wprevious\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wnext as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131359 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"wq\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131455 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"wqall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 131358 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wshada\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wundo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_wundo as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 158 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"wviminfo\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_shada as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301790 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xit\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_exit as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432959 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"xall\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(do_wqall as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 258 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmapclear\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_mapclear as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17301764 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xmenu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"xnoremap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_map as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xnoremenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17316101 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
        CommandDefinition {
            cmd_name: b"xunmap\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_unmap as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"xunmenu\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_menu as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17312004 as uint32_t,
            cmd_addr_type: ADDR_NONE,
        },
        CommandDefinition {
            cmd_name: b"yank\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 17303361 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"z\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_z as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19398983 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"!\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_bang as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301583 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"#\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_print as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 19400001 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"&\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350149 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 20448577 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_equal as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17432613 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b">\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_operators as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 20448577 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"@\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(ex_at as unsafe extern "C" fn(*mut exarg_T) -> ())),
            cmd_preview_func: None,
            cmd_argt: 17301829 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"~\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_substitute as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 18350149 as uint32_t,
            cmd_addr_type: ADDR_LINES,
        },
        CommandDefinition {
            cmd_name: b"Next\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            cmd_func: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut exarg_T) -> ()>,
                ex_func_T,
            >(Some(
                ex_previous as unsafe extern "C" fn(*mut exarg_T) -> (),
            )),
            cmd_preview_func: None,
            cmd_argt: 148743 as uint32_t,
            cmd_addr_type: ADDR_OTHER,
        },
    ]
});
static cmdidxs1: GlobalCell<[uint16_t; 26]> = GlobalCell::new([
    0 as uint16_t,
    20 as uint16_t,
    43 as uint16_t,
    109 as uint16_t,
    133 as uint16_t,
    154 as uint16_t,
    170 as uint16_t,
    176 as uint16_t,
    184 as uint16_t,
    203 as uint16_t,
    205 as uint16_t,
    210 as uint16_t,
    272 as uint16_t,
    290 as uint16_t,
    307 as uint16_t,
    318 as uint16_t,
    357 as uint16_t,
    360 as uint16_t,
    382 as uint16_t,
    447 as uint16_t,
    492 as uint16_t,
    504 as uint16_t,
    522 as uint16_t,
    537 as uint16_t,
    546 as uint16_t,
    547 as uint16_t,
]);
static cmdidxs2: GlobalCell<[[uint8_t; 26]; 26]> = GlobalCell::new([
    [
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        11 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        22 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        3 as uint8_t,
        12 as uint8_t,
        16 as uint8_t,
        18 as uint8_t,
        20 as uint8_t,
        22 as uint8_t,
        25 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        34 as uint8_t,
        38 as uint8_t,
        41 as uint8_t,
        47 as uint8_t,
        58 as uint8_t,
        60 as uint8_t,
        61 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        62 as uint8_t,
        0 as uint8_t,
        65 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        18 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        19 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        21 as uint8_t,
        22 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        3 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        6 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        3 as uint8_t,
        11 as uint8_t,
        15 as uint8_t,
        18 as uint8_t,
        19 as uint8_t,
        23 as uint8_t,
        26 as uint8_t,
        31 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        33 as uint8_t,
        36 as uint8_t,
        39 as uint8_t,
        43 as uint8_t,
        49 as uint8_t,
        0 as uint8_t,
        51 as uint8_t,
        60 as uint8_t,
        52 as uint8_t,
        53 as uint8_t,
        57 as uint8_t,
        59 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        5 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        3 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        11 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        17 as uint8_t,
        26 as uint8_t,
        0 as uint8_t,
        27 as uint8_t,
        0 as uint8_t,
        28 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        21 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        6 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        17 as uint8_t,
        21 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        23 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        26 as uint8_t,
        28 as uint8_t,
        32 as uint8_t,
        36 as uint8_t,
        38 as uint8_t,
        0 as uint8_t,
        47 as uint8_t,
        0 as uint8_t,
        48 as uint8_t,
        0 as uint8_t,
        60 as uint8_t,
        61 as uint8_t,
        0 as uint8_t,
        62 as uint8_t,
        0 as uint8_t,
    ],
    [
        4 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        24 as uint8_t,
        25 as uint8_t,
        0 as uint8_t,
        26 as uint8_t,
        0 as uint8_t,
        27 as uint8_t,
        0 as uint8_t,
        28 as uint8_t,
        32 as uint8_t,
        35 as uint8_t,
        37 as uint8_t,
        38 as uint8_t,
        0 as uint8_t,
        39 as uint8_t,
        42 as uint8_t,
        0 as uint8_t,
        43 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        11 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        15 as uint8_t,
        0 as uint8_t,
        16 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        2 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        3 as uint8_t,
        4 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        8 as uint8_t,
        0 as uint8_t,
        9 as uint8_t,
        10 as uint8_t,
        0 as uint8_t,
        12 as uint8_t,
        0 as uint8_t,
        13 as uint8_t,
        14 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        1 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        2 as uint8_t,
        5 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        7 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
    [
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
        0 as uint8_t,
    ],
]);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
