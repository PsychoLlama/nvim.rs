extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn abort() -> !;
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
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strnlen(__string: *const ::core::ffi::c_char, __maxlen: size_t) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
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
    fn xmemrchr(
        src: *const ::core::ffi::c_void,
        c: uint8_t,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn strnequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char, n: size_t) -> bool;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn ga_take_string(ga: *mut garray_T) -> String_0;
    fn api_free_array(value: Array);
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn nvim_echo(
        chunks: Array,
        history: Boolean,
        opts: *mut KeyDict_echo_opts,
        err: *mut Error,
    ) -> Object;
    fn has_event(event: event_T) -> bool;
    fn apply_autocmds_group(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        group: ::core::ffi::c_int,
        buf: *mut buf_T,
        eap: *mut exarg_T,
        data: *mut Object,
    ) -> bool;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    static mut on_print: Callback;
    static mut p_ch: OptInt;
    static mut p_debug: *mut ::core::ffi::c_char;
    static mut p_eb: ::core::ffi::c_int;
    static mut p_lz: ::core::ffi::c_int;
    static mut p_mopt: *mut ::core::ffi::c_char;
    static mut p_more: ::core::ffi::c_int;
    static mut rdb_flags: ::core::ffi::c_uint;
    static mut p_report: OptInt;
    static mut p_verbose: OptInt;
    static mut p_vfile: *mut ::core::ffi::c_char;
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
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn vim_vsnprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ap: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn transchar_buf(buf: *const buf_T, c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn transchar_byte_buf(buf: *const buf_T, c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn byte2cells(b: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_strsize(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    static e_intern2: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn set_must_redraw(type_0: ::core::ffi::c_int);
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    static mut msg_ext_skip_flush: bool;
    static mut msg_ext_overwrite: bool;
    static mut msg_ext_skip_verbose: bool;
    static mut msg_grid: ScreenGrid;
    static mut msg_grid_pos: ::core::ffi::c_int;
    static mut msg_grid_adj: GridView;
    static mut msg_scrolled_at_flush: ::core::ffi::c_int;
    static mut msg_grid_scroll_discount: ::core::ffi::c_int;
    fn tv_clear(tv: *mut typval_T);
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn var_redir_str(value: *const ::core::ffi::c_char, value_len: ::core::ffi::c_int);
    fn loop_schedule_deferred(loop_0: *mut Loop, event: Event);
    fn os_delay(ms: uint64_t, ignoreinput: bool);
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn cause_errthrow(
        mesg: *const ::core::ffi::c_char,
        multiline: bool,
        concat: bool,
        severe: bool,
        ignore: *mut bool,
    ) -> bool;
    fn do_sleep(msec: int64_t, hide_cursor: bool);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn stuff_empty() -> bool;
    fn typeahead_noflush(c: ::core::ffi::c_int);
    fn flush_buffers(flush_typeahead: flush_buffers_T);
    fn beep_flush();
    fn ins_char_typebuf(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
        on_key_ignore: bool,
    ) -> ::core::ffi::c_int;
    fn safe_vgetc() -> ::core::ffi::c_int;
    fn char_avail() -> bool;
    static mut Rows: ::core::ffi::c_int;
    static mut Columns: ::core::ffi::c_int;
    static mut vgetc_mod_mask: ::core::ffi::c_int;
    static mut vgetc_char: ::core::ffi::c_int;
    static mut cmdline_row: ::core::ffi::c_int;
    static mut redraw_cmdline: bool;
    static mut clear_cmdline: bool;
    static mut mode_displayed: bool;
    static mut redrawing_cmdline: bool;
    static mut cmdline_was_last_drawn: bool;
    static mut cmdmsg_rl: bool;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_scrolled_ign: bool;
    static mut msg_did_scroll: bool;
    static mut keep_msg: *mut ::core::ffi::c_char;
    static mut keep_msg_hl_id: ::core::ffi::c_int;
    static mut need_fileinfo: bool;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didout: bool;
    static mut msg_didany: bool;
    static mut msg_nowait: bool;
    static mut emsg_off: ::core::ffi::c_int;
    static mut info_message: bool;
    static mut msg_hist_off: bool;
    static mut need_clr_eos: bool;
    static mut emsg_skip: ::core::ffi::c_int;
    static mut emsg_severe: bool;
    static mut emsg_assert_fails_msg: *mut ::core::ffi::c_char;
    static mut emsg_assert_fails_lnum: ::core::ffi::c_long;
    static mut emsg_assert_fails_context: *mut ::core::ffi::c_char;
    static mut did_emsg: ::core::ffi::c_int;
    static mut called_emsg: ::core::ffi::c_int;
    static mut ex_exitval: ::core::ffi::c_int;
    static mut emsg_on_display: bool;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut did_wait_return: bool;
    static mut quit_more: bool;
    static mut vgetc_busy: ::core::ffi::c_int;
    static mut lines_left: ::core::ffi::c_int;
    static mut msg_no_more: bool;
    static mut need_check_timestamps: bool;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut sc_col: ::core::ffi::c_int;
    static mut exiting: bool;
    static mut full_screen: bool;
    static mut silent_mode: bool;
    static mut State: ::core::ffi::c_int;
    static mut exmode_active: bool;
    static mut reg_recording: ::core::ffi::c_int;
    static mut no_mapping: ::core::ffi::c_int;
    static mut allow_keys: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut emsg_noredir: bool;
    static mut cmd_silent: bool;
    static mut in_assert_fails: bool;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut msg_buf: [::core::ffi::c_char; 480];
    static mut KeyTyped: bool;
    static mut skip_redraw: bool;
    static mut do_redraw: bool;
    static mut need_highlight_changed: bool;
    static mut scriptout: *mut FILE;
    static mut got_int: bool;
    static mut global_busy: ::core::ffi::c_int;
    static mut redir_off: bool;
    static mut redir_fd: *mut FILE;
    static mut redir_reg: ::core::ffi::c_int;
    static mut redir_vname: bool;
    static mut capture_ga: *mut garray_T;
    static mut embedded_mode: bool;
    static mut headless_mode: bool;
    static mut default_grid: ScreenGrid;
    fn grid_adjust(
        grid: *mut GridView,
        row_off: *mut ::core::ffi::c_int,
        col_off: *mut ::core::ffi::c_int,
    ) -> *mut ScreenGrid;
    fn schar_get(buf_out: *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn grid_clear_line(grid: *mut ScreenGrid, off: size_t, width: ::core::ffi::c_int, valid: bool);
    fn grid_line_start(view: *mut GridView, row: ::core::ffi::c_int);
    fn grid_line_puts(
        col: ::core::ffi::c_int,
        text: *const ::core::ffi::c_char,
        textlen: ::core::ffi::c_int,
        attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn grid_line_cursor_goto(col: ::core::ffi::c_int);
    fn grid_line_mirror(width: ::core::ffi::c_int);
    fn grid_line_flush();
    fn grid_line_flush_if_valid_row();
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
    fn grid_free(grid: *mut ScreenGrid);
    fn grid_assign_handle(grid: *mut ScreenGrid);
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
    static mut hl_attr_active: *mut ::core::ffi::c_int;
    fn hl_combine_attr(
        char_attr: ::core::ffi::c_int,
        prim_attr: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn highlight_changed();
    fn tabstop_padding(col: colnr_T, ts_arg: OptInt, vts: *const colnr_T) -> ::core::ffi::c_int;
    fn get_keystroke(events: *mut MultiQueue) -> ::core::ffi::c_int;
    fn prompt_for_input(
        prompt: *mut ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> ::core::ffi::c_int;
    fn get_special_key_name(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static mut main_loop: Loop;
    fn utf_char2cells(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn mb_string2cells_len(str: *const ::core::ffi::c_char, size: size_t) -> size_t;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_unescape(pp: *mut *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    static utf8len_tab: [uint8_t; 256];
    fn jump_to_mouse(
        flags: ::core::ffi::c_int,
        inclusive: *mut bool,
        which_button: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn setmouse();
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn os_breakcheck();
    fn input_available() -> size_t;
    static mut nvim_testing: bool;
    fn home_replace_save(
        buf: *mut buf_T,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn write_reg_contents(
        name: ::core::ffi::c_int,
        str: *const ::core::ffi::c_char,
        len: ssize_t,
        must_append: ::core::ffi::c_int,
    );
    static mut exestack: garray_T;
    fn ui_active() -> size_t;
    fn ui_refresh();
    fn vim_beep(val: ::core::ffi::c_uint);
    fn ui_line(
        grid: *mut ScreenGrid,
        row: ::core::ffi::c_int,
        invalid_row: bool,
        startcol: ::core::ffi::c_int,
        endcol: ::core::ffi::c_int,
        clearcol: ::core::ffi::c_int,
        clearattr: ::core::ffi::c_int,
        wrap: bool,
    );
    fn ui_cursor_goto(new_row: ::core::ffi::c_int, new_col: ::core::ffi::c_int);
    fn ui_grid_cursor_goto(
        grid_handle: handle_T,
        new_row: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn estack_sfile(which: estack_arg_T) -> *mut ::core::ffi::c_char;
    static mut resize_events: *mut MultiQueue;
    fn ui_call_grid_resize(grid: Integer, width: Integer, height: Integer);
    fn ui_call_grid_scroll(
        grid: Integer,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn ui_call_grid_destroy(grid: Integer);
    fn ui_call_msg_set_pos(
        grid: Integer,
        row: Integer,
        scrolled: Boolean,
        sep_char: String_0,
        zindex: Integer,
        compindex: Integer,
    );
    fn ui_call_msg_show(
        kind: String_0,
        content: Array,
        replace_last: Boolean,
        history: Boolean,
        append: Boolean,
        id: Object,
        trigger: String_0,
    );
    fn ui_call_msg_showmode(content: Array);
    fn ui_call_msg_history_show(entries: Array, prev_cmd: Boolean);
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
    fn check_timestamps(focus: ::core::ffi::c_int) -> ::core::ffi::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __gnuc_va_list;
pub type ptrdiff_t = isize;
pub type size_t = usize;
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
pub type ssize_t = isize;
pub type time_t = __time_t;
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
    pub active_reqs: C2Rust_Unnamed_4,
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
    pub timer_heap: C2Rust_Unnamed_2,
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
    pub u: C2Rust_Unnamed_1,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
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
    pub u: C2Rust_Unnamed_0,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
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
pub union C2Rust_Unnamed_1 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
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
    pub u: C2Rust_Unnamed_3,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_mutex_t = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
    pub u: C2Rust_Unnamed_5,
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
pub union C2Rust_Unnamed_5 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
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
    pub u: C2Rust_Unnamed_9,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_8,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
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
    pub u: C2Rust_Unnamed_10,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
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
    pub data: C2Rust_Unnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
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
    pub b_wininfo: C2Rust_Unnamed_23,
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
    pub b_signcols: C2Rust_Unnamed_15,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_13,
    pub update_callbacks: C2Rust_Unnamed_12,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_12 {
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
pub struct C2Rust_Unnamed_13 {
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
    pub data: C2Rust_Unnamed_14,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
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
pub struct C2Rust_Unnamed_15 {
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
    pub sst_union: C2Rust_Unnamed_16,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_16 {
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
    pub data: C2Rust_Unnamed_17,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
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
    pub fc_fixvar: [C2Rust_Unnamed_18; 12],
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
pub struct C2Rust_Unnamed_18 {
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
    pub uh_next: C2Rust_Unnamed_22,
    pub uh_prev: C2Rust_Unnamed_21,
    pub uh_alt_next: C2Rust_Unnamed_20,
    pub uh_alt_prev: C2Rust_Unnamed_19,
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
pub union C2Rust_Unnamed_19 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
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
pub struct C2Rust_Unnamed_23 {
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
    pub type_0: C2Rust_Unnamed_24,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_24 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_24 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_24 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_24 = 0;
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
    pub children: C2Rust_Unnamed_25,
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
pub struct C2Rust_Unnamed_25 {
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
    pub uv: C2Rust_Unnamed_26,
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
pub union C2Rust_Unnamed_26 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type OptionalKeys = uint64_t;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_27 = 250;
pub const kZIndexMessages: C2Rust_Unnamed_27 = 200;
pub const kZIndexPopupMenu: C2Rust_Unnamed_27 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_27 = 50;
pub const kZIndexDefaultGrid: C2Rust_Unnamed_27 = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_28 = 76;
pub const HLF_PRE: C2Rust_Unnamed_28 = 75;
pub const HLF_OK: C2Rust_Unnamed_28 = 74;
pub const HLF_SO: C2Rust_Unnamed_28 = 73;
pub const HLF_SE: C2Rust_Unnamed_28 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_28 = 71;
pub const HLF_TS: C2Rust_Unnamed_28 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_28 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_28 = 68;
pub const HLF_CU: C2Rust_Unnamed_28 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_28 = 66;
pub const HLF_WBR: C2Rust_Unnamed_28 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_28 = 64;
pub const HLF_MSG: C2Rust_Unnamed_28 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_28 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_28 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_28 = 60;
pub const HLF_0: C2Rust_Unnamed_28 = 59;
pub const HLF_QFL: C2Rust_Unnamed_28 = 58;
pub const HLF_MC: C2Rust_Unnamed_28 = 57;
pub const HLF_CUL: C2Rust_Unnamed_28 = 56;
pub const HLF_CUC: C2Rust_Unnamed_28 = 55;
pub const HLF_TPF: C2Rust_Unnamed_28 = 54;
pub const HLF_TPS: C2Rust_Unnamed_28 = 53;
pub const HLF_TP: C2Rust_Unnamed_28 = 52;
pub const HLF_PBR: C2Rust_Unnamed_28 = 51;
pub const HLF_PST: C2Rust_Unnamed_28 = 50;
pub const HLF_PSB: C2Rust_Unnamed_28 = 49;
pub const HLF_PSX: C2Rust_Unnamed_28 = 48;
pub const HLF_PNX: C2Rust_Unnamed_28 = 47;
pub const HLF_PSK: C2Rust_Unnamed_28 = 46;
pub const HLF_PNK: C2Rust_Unnamed_28 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_28 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_28 = 43;
pub const HLF_PSI: C2Rust_Unnamed_28 = 42;
pub const HLF_PNI: C2Rust_Unnamed_28 = 41;
pub const HLF_SPL: C2Rust_Unnamed_28 = 40;
pub const HLF_SPR: C2Rust_Unnamed_28 = 39;
pub const HLF_SPC: C2Rust_Unnamed_28 = 38;
pub const HLF_SPB: C2Rust_Unnamed_28 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_28 = 36;
pub const HLF_SC: C2Rust_Unnamed_28 = 35;
pub const HLF_TXA: C2Rust_Unnamed_28 = 34;
pub const HLF_TXD: C2Rust_Unnamed_28 = 33;
pub const HLF_DED: C2Rust_Unnamed_28 = 32;
pub const HLF_CHD: C2Rust_Unnamed_28 = 31;
pub const HLF_ADD: C2Rust_Unnamed_28 = 30;
pub const HLF_FC: C2Rust_Unnamed_28 = 29;
pub const HLF_FL: C2Rust_Unnamed_28 = 28;
pub const HLF_WM: C2Rust_Unnamed_28 = 27;
pub const HLF_W: C2Rust_Unnamed_28 = 26;
pub const HLF_VNC: C2Rust_Unnamed_28 = 25;
pub const HLF_V: C2Rust_Unnamed_28 = 24;
pub const HLF_T: C2Rust_Unnamed_28 = 23;
pub const HLF_VSP: C2Rust_Unnamed_28 = 22;
pub const HLF_C: C2Rust_Unnamed_28 = 21;
pub const HLF_SNC: C2Rust_Unnamed_28 = 20;
pub const HLF_S: C2Rust_Unnamed_28 = 19;
pub const HLF_R: C2Rust_Unnamed_28 = 18;
pub const HLF_CLF: C2Rust_Unnamed_28 = 17;
pub const HLF_CLS: C2Rust_Unnamed_28 = 16;
pub const HLF_CLN: C2Rust_Unnamed_28 = 15;
pub const HLF_LNB: C2Rust_Unnamed_28 = 14;
pub const HLF_LNA: C2Rust_Unnamed_28 = 13;
pub const HLF_N: C2Rust_Unnamed_28 = 12;
pub const HLF_CM: C2Rust_Unnamed_28 = 11;
pub const HLF_M: C2Rust_Unnamed_28 = 10;
pub const HLF_LC: C2Rust_Unnamed_28 = 9;
pub const HLF_L: C2Rust_Unnamed_28 = 8;
pub const HLF_I: C2Rust_Unnamed_28 = 7;
pub const HLF_E: C2Rust_Unnamed_28 = 6;
pub const HLF_D: C2Rust_Unnamed_28 = 5;
pub const HLF_AT: C2Rust_Unnamed_28 = 4;
pub const HLF_TERM: C2Rust_Unnamed_28 = 3;
pub const HLF_EOB: C2Rust_Unnamed_28 = 2;
pub const HLF_8: C2Rust_Unnamed_28 = 1;
pub const HLF_NONE: C2Rust_Unnamed_28 = 0;
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
    pub cs_pend: C2Rust_Unnamed_29,
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
pub union C2Rust_Unnamed_29 {
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
pub struct HlMessageChunk {
    pub text: String_0,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlMessage {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlMessageChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_data {
    pub source: String_0,
    pub percent: Integer,
    pub title: String_0,
    pub status: String_0,
    pub data: Dict,
}
pub type MessageData = msg_data;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_hist {
    pub next: *mut msg_hist,
    pub prev: *mut msg_hist,
    pub msg: HlMessage,
    pub kind: *mut ::core::ffi::c_char,
    pub temp: bool,
    pub append: bool,
}
pub type MessageHistoryEntry = msg_hist;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_echo_opts {
    pub is_set__echo_opts_: OptionalKeys,
    pub err: Boolean,
    pub verbose: Boolean,
    pub _truncate: Boolean,
    pub kind: String_0,
    pub id: Object,
    pub title: String_0,
    pub status: String_0,
    pub percent: Integer,
    pub source: String_0,
    pub data: Dict,
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
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_30 = -4;
pub const AUGROUP_ALL: C2Rust_Unnamed_30 = -3;
pub const AUGROUP_ERROR: C2Rust_Unnamed_30 = -2;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_30 = -1;
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
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_31 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_31 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_31 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_31 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_31 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_31 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_31 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_31 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_31 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_31 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_31 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_31 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_31 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_31 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_31 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_31 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_31 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_31 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_31 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_31 = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const kOptMoptFlagProgress: C2Rust_Unnamed_32 = 8;
pub const kOptMoptFlagHistory: C2Rust_Unnamed_32 = 4;
pub const kOptMoptFlagWait: C2Rust_Unnamed_32 = 2;
pub const kOptMoptFlagHitEnter: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const kOptRdbFlagFlush: C2Rust_Unnamed_33 = 32;
pub const kOptRdbFlagLine: C2Rust_Unnamed_33 = 16;
pub const kOptRdbFlagNodelta: C2Rust_Unnamed_33 = 8;
pub const kOptRdbFlagInvalid: C2Rust_Unnamed_33 = 4;
pub const kOptRdbFlagNothrottle: C2Rust_Unnamed_33 = 2;
pub const kOptRdbFlagCompositor: C2Rust_Unnamed_33 = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_34 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_34 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_34 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_34 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_34 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_34 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_34 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_34 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_34 = 79;
pub const SHM_OVER: C2Rust_Unnamed_34 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_34 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_34 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_34 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_34 = 97;
pub const SHM_WRI: C2Rust_Unnamed_34 = 119;
pub const SHM_LINES: C2Rust_Unnamed_34 = 108;
pub const SHM_MOD: C2Rust_Unnamed_34 = 109;
pub const SHM_RO: C2Rust_Unnamed_34 = 114;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_35 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_35 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_35 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_35 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_35 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_35 = 20;
pub const UPD_VALID: C2Rust_Unnamed_35 = 10;
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
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_36 = 6;
pub const MB_MAXBYTES: C2Rust_Unnamed_36 = 21;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const VIM_DISCARDALL: C2Rust_Unnamed_37 = 6;
pub const VIM_ALL: C2Rust_Unnamed_37 = 5;
pub const VIM_CANCEL: C2Rust_Unnamed_37 = 4;
pub const VIM_NO: C2Rust_Unnamed_37 = 3;
pub const VIM_YES: C2Rust_Unnamed_37 = 2;
pub const MODE_ASKMORE: C2Rust_Unnamed_39 = 12288;
pub const MODE_SETWSIZE: C2Rust_Unnamed_39 = 16384;
pub const MOUSE_SETPOS: C2Rust_Unnamed_40 = 8;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_IGNORE: key_extra = 53;
pub type msgchunk_T = msgchunk_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msgchunk_S {
    pub sb_next: *mut msgchunk_T,
    pub sb_prev: *mut msgchunk_T,
    pub sb_eol: ::core::ffi::c_char,
    pub sb_msg_col: ::core::ffi::c_int,
    pub sb_hl_id: ::core::ffi::c_int,
    pub sb_text: [::core::ffi::c_char; 0],
}
pub type sb_clear_T = ::core::ffi::c_uint;
pub const SB_CLEAR_CMDLINE_DONE: sb_clear_T = 3;
pub const SB_CLEAR_CMDLINE_BUSY: sb_clear_T = 2;
pub const SB_CLEAR_ALL: sb_clear_T = 1;
pub const SB_CLEAR_NONE: sb_clear_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_38 {
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
pub const KE_EVENT: key_extra = 102;
pub const MODE_HITRETURN: C2Rust_Unnamed_39 = 8193;
pub type estack_arg_T = ::core::ffi::c_uint;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
pub type flush_buffers_T = ::core::ffi::c_uint;
pub const FLUSH_INPUT: flush_buffers_T = 2;
pub const FLUSH_TYPEAHEAD: flush_buffers_T = 1;
pub const FLUSH_MINIMAL: flush_buffers_T = 0;
pub const MODE_CMDLINE: C2Rust_Unnamed_39 = 8;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_39 = 20480;
pub const DLG_HOTKEY_CHAR: C2Rust_Unnamed_41 = 38;
pub const DLG_BUTTON_SEP: C2Rust_Unnamed_41 = 10;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_39 = 24592;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_39 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_39 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_39 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_39 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_39 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_39 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_39 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_39 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_39 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_39 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_39 = 16;
pub const MODE_OP_PENDING: C2Rust_Unnamed_39 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_39 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_39 = 1;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
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
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const MOUSE_RELEASED: C2Rust_Unnamed_40 = 32;
pub const MOUSE_MAY_STOP_VIS: C2Rust_Unnamed_40 = 16;
pub const MOUSE_DID_MOVE: C2Rust_Unnamed_40 = 4;
pub const MOUSE_MAY_VIS: C2Rust_Unnamed_40 = 2;
pub const MOUSE_FOCUS: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BELL: ::core::ffi::c_int = '\u{7}' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = 10;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = 27;
pub const Ctrl_B: ::core::ffi::c_int = 2;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_F: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut confirm_msg_used: ::core::ffi::c_int = false_0;
static mut confirm_msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
static mut confirm_buttons: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut msg_hist_last: *mut MessageHistoryEntry =
    ::core::ptr::null_mut::<MessageHistoryEntry>();
static mut msg_hist_first: *mut MessageHistoryEntry =
    ::core::ptr::null_mut::<MessageHistoryEntry>();
static mut msg_hist_temp: *mut MessageHistoryEntry = ::core::ptr::null_mut::<MessageHistoryEntry>();
static mut msg_hist_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut msg_hist_max: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const PROGRESS_TARGET_CMD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static mut msg_flags: ::core::ffi::c_int = kOptMoptFlagHitEnter as ::core::ffi::c_int
    | kOptMoptFlagHistory as ::core::ffi::c_int
    | kOptMoptFlagProgress as ::core::ffi::c_int;
static mut msg_wait: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut progress_msg_target: ::core::ffi::c_int = PROGRESS_TARGET_CMD;
static mut verbose_fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
static mut verbose_did_open: bool = false_0 != 0;
static mut keep_msg_more: bool = false_0 != 0;
static mut msg_ext_kind: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
static mut msg_ext_trigger: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
static mut msg_ext_id: Object = object {
    type_0: kObjectTypeInteger,
    data: C2Rust_Unnamed_11 {
        integer: 1 as Integer,
    },
};
static mut msg_ext_chunks: *mut Array = ::core::ptr::null_mut::<Array>();
static mut msg_ext_last_chunk: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
    ga_growsize: 40 as ::core::ffi::c_int,
    ga_data: NULL,
};
static mut msg_ext_last_attr: sattr_T = -1 as sattr_T;
static mut msg_ext_last_hl_id: ::core::ffi::c_int = 0;
static mut msg_ext_history: bool = false_0 != 0;
static mut msg_ext_append: bool = false_0 != 0;
static mut msg_grid_pos_at_flush: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut msg_id_next: int64_t = 1 as int64_t;
#[no_mangle]
pub unsafe extern "C" fn msg_id_exists(mut id: int64_t) -> bool {
    return id > 0 as int64_t && id < msg_id_next;
}
unsafe extern "C" fn ui_ext_msg_set_pos(mut row: ::core::ffi::c_int, mut scrolled: bool) {
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    let mut size: size_t = schar_get(
        &raw mut buf as *mut ::core::ffi::c_char,
        (*curwin).w_p_fcs_chars.msgsep,
    );
    ui_call_msg_set_pos(
        msg_grid.handle as Integer,
        row as Integer,
        scrolled as Boolean,
        String_0 {
            data: &raw mut buf as *mut ::core::ffi::c_char,
            size: size,
        },
        msg_grid.zindex as Integer,
        msg_grid.comp_index as ::core::ffi::c_int as Integer,
    );
    msg_grid.pending_comp_index_update = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn msg_grid_set_pos(mut row: ::core::ffi::c_int, mut scrolled: bool) {
    if !msg_grid.throttled {
        ui_ext_msg_set_pos(row, scrolled);
        msg_grid_pos_at_flush = row;
    }
    msg_grid_pos = row;
    if !msg_grid.chars.is_null() {
        msg_grid_adj.row_offset = -row;
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_use_grid() -> bool {
    return !default_grid.chars.is_null() && !ui_has(kUIMessages);
}
#[no_mangle]
pub unsafe extern "C" fn msg_grid_validate() {
    grid_assign_handle(&raw mut msg_grid);
    let mut should_alloc: bool = msg_use_grid();
    let mut max_rows: ::core::ffi::c_int = Rows - p_ch as ::core::ffi::c_int;
    if should_alloc as ::core::ffi::c_int != 0
        && (msg_grid.rows != Rows || msg_grid.cols != Columns || msg_grid.chars.is_null())
    {
        grid_alloc(&raw mut msg_grid, Rows, Columns, false_0 != 0, true_0 != 0);
        msg_grid.zindex = kZIndexMessages as ::core::ffi::c_int;
        xfree(msg_grid.dirty_col as *mut ::core::ffi::c_void);
        msg_grid.dirty_col = xcalloc(Rows as size_t, ::core::mem::size_of::<::core::ffi::c_int>())
            as *mut ::core::ffi::c_int;
        let mut pos: ::core::ffi::c_int = if State & MODE_ASKMORE as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else if max_rows - msg_scrolled > 0 as ::core::ffi::c_int {
            max_rows - msg_scrolled
        } else {
            0 as ::core::ffi::c_int
        };
        msg_grid.throttled = false_0 != 0;
        msg_grid_set_pos(pos, msg_scrolled != 0);
        ui_comp_put_grid(
            &raw mut msg_grid,
            pos,
            0 as ::core::ffi::c_int,
            msg_grid.rows,
            msg_grid.cols,
            false_0 != 0,
            true_0 != 0,
        );
        ui_call_grid_resize(
            msg_grid.handle as Integer,
            msg_grid.cols as Integer,
            msg_grid.rows as Integer,
        );
        msg_scrolled_at_flush = msg_scrolled;
        msg_grid.mouse_enabled = false_0 != 0;
        msg_grid_adj.target = &raw mut msg_grid;
    } else if !should_alloc && !msg_grid.chars.is_null() {
        ui_comp_remove_grid(&raw mut msg_grid);
        grid_free(&raw mut msg_grid);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut msg_grid.dirty_col as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        ui_call_grid_destroy(msg_grid.handle as Integer);
        msg_grid.throttled = false_0 != 0;
        msg_grid_adj.row_offset = 0 as ::core::ffi::c_int;
        msg_grid_adj.target = &raw mut default_grid;
        redraw_cmdline = true_0 != 0;
    } else if !msg_grid.chars.is_null() && msg_scrolled == 0 && msg_grid_pos != max_rows {
        let mut diff: ::core::ffi::c_int = msg_grid_pos - max_rows;
        msg_grid_set_pos(max_rows, false_0 != 0);
        if diff > 0 as ::core::ffi::c_int {
            grid_clear(
                &raw mut msg_grid_adj,
                Rows - diff,
                Rows,
                0 as ::core::ffi::c_int,
                Columns,
                *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
            );
        }
    }
    if !msg_grid.chars.is_null() && msg_scrolled == 0 && cmdline_row < msg_grid_pos {
        cmdline_row = msg_grid_pos;
    }
}
#[no_mangle]
pub unsafe extern "C" fn verb_msg(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    verbose_enter();
    let mut n: ::core::ffi::c_int =
        msg_keep(s, 0 as ::core::ffi::c_int, false_0 != 0, false_0 != 0) as ::core::ffi::c_int;
    verbose_leave();
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn msg(mut s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool {
    return msg_keep(s, hl_id, false_0 != 0, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn msg_multiline(
    mut str: String_0,
    mut hl_id: ::core::ffi::c_int,
    mut check_int: bool,
    mut hist: bool,
    mut need_clear: *mut bool,
) {
    let mut s: *const ::core::ffi::c_char = str.data;
    let mut chunk: *const ::core::ffi::c_char = s;
    while (s.offset_from(str.data) as size_t) < str.size {
        if check_int as ::core::ffi::c_int != 0 && got_int as ::core::ffi::c_int != 0 {
            return;
        }
        if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == TAB
            || *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == BELL
        {
            msg_outtrans_len(
                chunk,
                s.offset_from(chunk) as ::core::ffi::c_int,
                hl_id,
                hist,
            );
            if *s as ::core::ffi::c_int != TAB && *need_clear as ::core::ffi::c_int != 0 {
                msg_clr_eos();
                *need_clear = false_0 != 0;
            }
            if *s as ::core::ffi::c_int == BELL {
                vim_beep(kOptBoFlagShell as ::core::ffi::c_int as ::core::ffi::c_uint);
            } else {
                msg_putchar_hl(*s as uint8_t as ::core::ffi::c_int, hl_id);
            }
            chunk = s.offset(1 as ::core::ffi::c_int as isize);
        }
        s = s.offset(1);
    }
    if *chunk as ::core::ffi::c_int != NUL || chunk == str.data as *const ::core::ffi::c_char {
        msg_outtrans_len(
            chunk,
            str.size.wrapping_sub(chunk.offset_from(str.data) as size_t) as ::core::ffi::c_int,
            hl_id,
            hist,
        );
    }
}
static mut is_multihl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn format_progress_message(
    mut hl_msg: HlMessage,
    mut msg_data: *mut MessageData,
) -> HlMessage {
    let mut updated_msg: HlMessage = HlMessage {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<HlMessageChunk>(),
    };
    if (*msg_data).title.size != 0 as size_t {
        let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*msg_data).status.data.is_null() {
            hl_id = 0 as ::core::ffi::c_int;
        } else if strequal(
            (*msg_data).status.data,
            b"success\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"OkMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"failed\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"running\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
        } else if strequal(
            (*msg_data).status.data,
            b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            hl_id = syn_check_group(
                b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            );
        }
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh9 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh9 as isize) = HlMessageChunk {
            text: copy_string((*msg_data).title, ::core::ptr::null_mut::<Arena>()),
            hl_id: hl_id,
        };
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh10 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh10 as isize) = HlMessageChunk {
            text: cstr_to_string(b": \0".as_ptr() as *const ::core::ffi::c_char),
            hl_id: 0 as ::core::ffi::c_int,
        };
    }
    if (*msg_data).percent > 0 as Integer {
        let mut percent_buf: [::core::ffi::c_char; 10] = [0; 10];
        vim_snprintf(
            &raw mut percent_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            b"%3ld%% \0".as_ptr() as *const ::core::ffi::c_char,
            (*msg_data).percent as ::core::ffi::c_long,
        );
        let mut percent: String_0 =
            cstr_to_string(&raw mut percent_buf as *mut ::core::ffi::c_char);
        let mut hl_id_0: ::core::ffi::c_int = syn_check_group(
            b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
        );
        if updated_msg.size == updated_msg.capacity {
            updated_msg.capacity = if updated_msg.capacity != 0 {
                updated_msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            updated_msg.items = xrealloc(
                updated_msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh11 = updated_msg.size;
        updated_msg.size = updated_msg.size.wrapping_add(1);
        *updated_msg.items.offset(c2rust_fresh11 as isize) = HlMessageChunk {
            text: percent,
            hl_id: hl_id_0,
        };
    }
    if updated_msg.size != 0 as size_t {
        let mut i: uint32_t = 0 as uint32_t;
        while (i as size_t) < hl_msg.size {
            if updated_msg.size == updated_msg.capacity {
                updated_msg.capacity = if updated_msg.capacity != 0 {
                    updated_msg.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                updated_msg.items = xrealloc(
                    updated_msg.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(updated_msg.capacity),
                ) as *mut HlMessageChunk;
            } else {
            };
            let c2rust_fresh12 = updated_msg.size;
            updated_msg.size = updated_msg.size.wrapping_add(1);
            *updated_msg.items.offset(c2rust_fresh12 as isize) = HlMessageChunk {
                text: copy_string(
                    (*hl_msg.items.offset(i as isize)).text,
                    ::core::ptr::null_mut::<Arena>(),
                ),
                hl_id: (*hl_msg.items.offset(i as isize)).hl_id,
            };
            i = i.wrapping_add(1);
        }
        return updated_msg;
    } else {
        return hl_msg;
    };
}
#[no_mangle]
pub unsafe extern "C" fn msg_multihl(
    mut id: Object,
    mut hl_msg: HlMessage,
    mut kind: *const ::core::ffi::c_char,
    mut history: bool,
    mut err: bool,
    mut msg_data: *mut MessageData,
    mut needs_msg_clear: *mut bool,
) -> Object {
    if id.type_0 as ::core::ffi::c_uint
        == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let c2rust_fresh8 = msg_id_next;
        msg_id_next = msg_id_next + 1;
        id = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: c2rust_fresh8,
            },
        };
    } else if id.type_0 as ::core::ffi::c_uint
        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        && !msg_id_exists(id.data.integer as int64_t)
    {
        abort();
    }
    if strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        && progress_msg_target & PROGRESS_TARGET_CMD == 0 as ::core::ffi::c_int
    {
        *needs_msg_clear = true_0 != 0;
        return id;
    }
    no_wait_return += 1;
    msg_start();
    msg_clr_eos();
    let mut need_clear: bool = false_0 != 0;
    let mut hl_msg_updated: bool = false_0 != 0;
    if !kind.is_null() {
        msg_ext_set_kind(kind);
    }
    msg_ext_skip_flush = true_0 != 0;
    msg_ext_id = id;
    if strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        && !msg_data.is_null()
    {
        let mut formated_message: HlMessage = format_progress_message(hl_msg, msg_data);
        if formated_message.items != hl_msg.items {
            *needs_msg_clear = true_0 != 0;
            hl_msg_updated = true_0 != 0;
            hl_msg = formated_message;
        }
    }
    let mut i: uint32_t = 0 as uint32_t;
    while (i as size_t) < hl_msg.size {
        let mut chunk: HlMessageChunk = *hl_msg.items.offset(i as isize);
        is_multihl += 1;
        if err {
            emsg_multiline(chunk.text.data, kind, chunk.hl_id, true_0 != 0);
        } else {
            msg_multiline(
                chunk.text,
                chunk.hl_id,
                true_0 != 0,
                false_0 != 0,
                &raw mut need_clear,
            );
        }
        '_c2rust_label: {
            if !ui_has(kUIMessages) || kind.is_null() || msg_ext_kind == kind {
            } else {
                __assert_fail(
                    b"!ui_has(kUIMessages) || kind == NULL || msg_ext_kind == kind\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    416 as ::core::ffi::c_uint,
                    b"Object msg_multihl(Object, HlMessage, const char *, _Bool, _Bool, MessageData *, _Bool *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        i = i.wrapping_add(1);
    }
    if history as ::core::ffi::c_int != 0 && hl_msg.size != 0 {
        msg_hist_add_multihl(hl_msg, false_0 != 0, msg_data);
    }
    msg_ext_skip_flush = false_0 != 0;
    is_multihl = 0 as ::core::ffi::c_int;
    no_wait_return -= 1;
    msg_end();
    if hl_msg_updated as ::core::ffi::c_int != 0
        && !(history as ::core::ffi::c_int != 0 && hl_msg.size != 0)
    {
        hl_msg_free(hl_msg);
    }
    return id;
}
#[no_mangle]
pub unsafe extern "C" fn msg_keep(
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut keep: bool,
    mut multiline: bool,
) -> bool {
    static mut entered: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if keep as ::core::ffi::c_int != 0 && multiline as ::core::ffi::c_int != 0 {
        abort();
    }
    if !emsg_on_display && message_filtered(s) as ::core::ffi::c_int != 0 {
        return true_0 != 0;
    }
    if hl_id == 0 as ::core::ffi::c_int {
        set_vim_var_string(VV_STATUSMSG, s, -1 as ptrdiff_t);
    }
    if entered >= 3 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    entered += 1;
    if is_multihl == 0
        && (s != keep_msg as *const ::core::ffi::c_char
            || *s as ::core::ffi::c_int != '<' as ::core::ffi::c_int
                && !msg_hist_last.is_null()
                && strcmp(
                    s,
                    (*(*msg_hist_last)
                        .msg
                        .items
                        .offset(0 as ::core::ffi::c_int as isize))
                    .text
                    .data,
                ) != 0 as ::core::ffi::c_int)
    {
        msg_hist_add(s, -1 as ::core::ffi::c_int, hl_id);
    }
    if is_multihl == 0 {
        msg_start();
    }
    let mut buf: *mut ::core::ffi::c_char = msg_strtrunc(s, false_0);
    if !buf.is_null() {
        s = buf;
    }
    let mut need_clear: bool = true_0 != 0;
    if multiline {
        msg_multiline(
            cstr_as_string(s),
            hl_id,
            false_0 != 0,
            false_0 != 0,
            &raw mut need_clear,
        );
    } else {
        msg_outtrans(s, hl_id, false_0 != 0);
    }
    if need_clear {
        msg_clr_eos();
    }
    let mut retval: bool = true_0 != 0;
    if is_multihl == 0 {
        retval = msg_end();
    }
    if keep as ::core::ffi::c_int != 0
        && retval as ::core::ffi::c_int != 0
        && vim_strsize(s) < (Rows - cmdline_row - 1 as ::core::ffi::c_int) * Columns + sc_col
    {
        set_keep_msg(s, 0 as ::core::ffi::c_int);
    }
    need_fileinfo = false_0 != 0;
    xfree(buf as *mut ::core::ffi::c_void);
    entered -= 1;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn msg_strtrunc(
    mut s: *const ::core::ffi::c_char,
    mut force: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if msg_scroll == 0
        && !need_wait_return
        && shortmess(SHM_TRUNCALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && !exmode_active
        && msg_silent == 0 as ::core::ffi::c_int
        && !ui_has(kUIMessages)
        || force != 0
    {
        let mut room: ::core::ffi::c_int = 0;
        let mut len: ::core::ffi::c_int = vim_strsize(s);
        if msg_scrolled != 0 as ::core::ffi::c_int {
            room = (Rows - msg_row) * Columns - 1 as ::core::ffi::c_int;
        } else {
            room = (Rows - msg_row - 1 as ::core::ffi::c_int) * Columns + sc_col
                - 1 as ::core::ffi::c_int;
        }
        if len > room && room > 0 as ::core::ffi::c_int {
            len = (room + 2 as ::core::ffi::c_int) * 18 as ::core::ffi::c_int;
            buf = xmalloc(len as size_t) as *mut ::core::ffi::c_char;
            trunc_string(s, buf, room, len);
        }
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn trunc_string(
    mut s: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut room_in: ::core::ffi::c_int,
    mut buflen: ::core::ffi::c_int,
) {
    let mut room: ::core::ffi::c_int = room_in - 3 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut e: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if *s as ::core::ffi::c_int == NUL {
        if buflen > 0 as ::core::ffi::c_int {
            *buf = NUL as ::core::ffi::c_char;
        }
        return;
    }
    if room_in < 3 as ::core::ffi::c_int {
        room = 0 as ::core::ffi::c_int;
    }
    let mut half: ::core::ffi::c_int = room / 2 as ::core::ffi::c_int;
    e = 0 as ::core::ffi::c_int;
    while len < half && e < buflen {
        if *s.offset(e as isize) as ::core::ffi::c_int == NUL {
            *buf.offset(e as isize) = NUL as ::core::ffi::c_char;
            return;
        }
        n = ptr2cells(s.offset(e as isize));
        if len + n > half {
            break;
        }
        len += n;
        *buf.offset(e as isize) = *s.offset(e as isize);
        n = utfc_ptr2len(s.offset(e as isize));
        loop {
            n -= 1;
            if n <= 0 as ::core::ffi::c_int {
                break;
            }
            e += 1;
            if e == buflen {
                break;
            }
            *buf.offset(e as isize) = *s.offset(e as isize);
        }
        e += 1;
    }
    i = strlen(s) as ::core::ffi::c_int;
    half = i;
    loop {
        half =
            half - utf_head_off(
                s,
                s.offset(half as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize)),
            ) - 1 as ::core::ffi::c_int;
        n = ptr2cells(s.offset(half as isize));
        if len + n > room || half == 0 as ::core::ffi::c_int {
            break;
        }
        len += n;
        i = half;
    }
    if i <= e + 3 as ::core::ffi::c_int {
        if s != buf as *const ::core::ffi::c_char {
            len = strlen(s) as ::core::ffi::c_int;
            if len >= buflen {
                len = buflen - 1 as ::core::ffi::c_int;
            }
            len = len - e + 1 as ::core::ffi::c_int;
            if len < 1 as ::core::ffi::c_int {
                *buf.offset((e - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
            } else {
                memmove(
                    buf.offset(e as isize) as *mut ::core::ffi::c_void,
                    s.offset(e as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
        }
    } else if (e + 3 as ::core::ffi::c_int) < buflen {
        memmove(
            buf.offset(e as isize) as *mut ::core::ffi::c_void,
            b"...\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            3 as size_t,
        );
        len = strlen(s.offset(i as isize)) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        if len >= buflen - e - 3 as ::core::ffi::c_int {
            len = buflen - e - 3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
        memmove(
            buf.offset(e as isize)
                .offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            s.offset(i as isize) as *const ::core::ffi::c_void,
            len as size_t,
        );
        *buf.offset((e + 3 as ::core::ffi::c_int + len - 1 as ::core::ffi::c_int) as isize) =
            NUL as ::core::ffi::c_char;
    } else {
        *buf.offset((buflen - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
    };
}
#[no_mangle]
pub unsafe extern "C" fn smsg(
    mut hl_id: ::core::ffi::c_int,
    mut s: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut arglist: ::core::ffi::VaListImpl;
    arglist = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        s,
        arglist.as_va_list(),
    );
    return msg(&raw mut IObuff as *mut ::core::ffi::c_char, hl_id) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn smsg_keep(
    mut hl_id: ::core::ffi::c_int,
    mut s: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    let mut arglist: ::core::ffi::VaListImpl;
    arglist = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        s,
        arglist.as_va_list(),
    );
    return msg_keep(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        hl_id,
        true_0 != 0,
        false_0 != 0,
    ) as ::core::ffi::c_int;
}
static mut last_sourcing_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut last_sourcing_name: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub unsafe extern "C" fn reset_last_sourcing() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut last_sourcing_name as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    last_sourcing_lnum = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn other_sourcing_name() -> bool {
    if !exestack.ga_data.is_null()
        && exestack.ga_len > 0 as ::core::ffi::c_int
        && !(*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
    {
        if !last_sourcing_name.is_null() {
            return strcmp(
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                last_sourcing_name,
            ) != 0 as ::core::ffi::c_int;
        }
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn get_emsg_source() -> *mut ::core::ffi::c_char {
    if !exestack.ga_data.is_null()
        && exestack.ga_len > 0 as ::core::ffi::c_int
        && !(*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
        && other_sourcing_name() as ::core::ffi::c_int != 0
    {
        let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
        let mut tofree: *mut ::core::ffi::c_char = sname;
        if sname.is_null() {
            sname = (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name;
        }
        let p: *const ::core::ffi::c_char =
            gettext(b"Error in %s:\0".as_ptr() as *const ::core::ffi::c_char);
        let buf_len: size_t = strlen(sname)
            .wrapping_add(strlen(p))
            .wrapping_add(1 as size_t);
        let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
        snprintf(buf, buf_len, p, sname);
        xfree(tofree as *mut ::core::ffi::c_void);
        return buf;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_emsg_lnum() -> *mut ::core::ffi::c_char {
    if !(*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name
    .is_null()
        && (other_sourcing_name() as ::core::ffi::c_int != 0
            || (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
                != last_sourcing_lnum as linenr_T)
        && (*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            != 0 as linenr_T
    {
        let p: *const ::core::ffi::c_char =
            gettext(b"line %4d:\0".as_ptr() as *const ::core::ffi::c_char);
        let buf_len: size_t = (20 as size_t).wrapping_add(strlen(p));
        let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
        snprintf(
            buf,
            buf_len,
            p,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        return buf;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn msg_source(mut hl_id: ::core::ffi::c_int) {
    static mut recursive: bool = false_0 != 0;
    if recursive {
        return;
    }
    recursive = true_0 != 0;
    no_wait_return += 1;
    let mut p: *mut ::core::ffi::c_char = get_emsg_source();
    if !p.is_null() {
        msg_scroll = true_0;
        msg(p, hl_id);
        xfree(p as *mut ::core::ffi::c_void);
    }
    p = get_emsg_lnum();
    if !p.is_null() {
        msg(p, HLF_N as ::core::ffi::c_int);
        xfree(p as *mut ::core::ffi::c_void);
        last_sourcing_lnum = (*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum as ::core::ffi::c_int;
    }
    if (*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name
    .is_null()
        || other_sourcing_name() as ::core::ffi::c_int != 0
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut last_sourcing_name as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        if !(*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
        {
            last_sourcing_name = xstrdup(
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            );
            if redirecting() == 0 {
                msg_putchar_hl('\n' as ::core::ffi::c_int, hl_id);
            }
        }
    }
    no_wait_return -= 1;
    recursive = false_0 != 0;
}
unsafe extern "C" fn emsg_not_now() -> ::core::ffi::c_int {
    if emsg_off > 0 as ::core::ffi::c_int
        && vim_strchr(p_debug, 'm' as ::core::ffi::c_int).is_null()
        && vim_strchr(p_debug, 't' as ::core::ffi::c_int).is_null()
        || emsg_skip > 0 as ::core::ffi::c_int
    {
        return true_0;
    }
    return false_0;
}
#[no_mangle]
pub unsafe extern "C" fn emsg_multiline(
    mut s: *const ::core::ffi::c_char,
    mut kind: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut multiline: bool,
) -> bool {
    let mut ignore: bool = false_0 != 0;
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    called_emsg += 1;
    let mut severe: bool = emsg_severe;
    emsg_severe = false_0 != 0;
    if emsg_off == 0 || !vim_strchr(p_debug, 't' as ::core::ffi::c_int).is_null() {
        if cause_errthrow(
            s,
            multiline,
            is_multihl > 1 as ::core::ffi::c_int,
            severe,
            &raw mut ignore,
        ) {
            if !ignore {
                did_emsg += 1;
            }
            return true_0 != 0;
        }
        if in_assert_fails as ::core::ffi::c_int != 0 && emsg_assert_fails_msg.is_null() {
            emsg_assert_fails_msg = xstrdup(s);
            emsg_assert_fails_lnum = (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as ::core::ffi::c_long;
            xfree(emsg_assert_fails_context as *mut ::core::ffi::c_void);
            emsg_assert_fails_context = xstrdup(
                if (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name as *const ::core::ffi::c_char
                },
            );
        }
        set_vim_var_string(VV_ERRMSG, s, -1 as ptrdiff_t);
        if emsg_silent != 0 as ::core::ffi::c_int {
            if !emsg_noredir {
                msg_start();
                let mut p: *mut ::core::ffi::c_char = get_emsg_source();
                if !p.is_null() {
                    let p_len: size_t = strlen(p);
                    *p.offset(p_len as isize) = '\n' as ::core::ffi::c_char;
                    redir_write(p, p_len as ptrdiff_t + 1 as ptrdiff_t);
                    xfree(p as *mut ::core::ffi::c_void);
                }
                p = get_emsg_lnum();
                if !p.is_null() {
                    let p_len_0: size_t = strlen(p);
                    *p.offset(p_len_0 as isize) = '\n' as ::core::ffi::c_char;
                    redir_write(p, p_len_0 as ptrdiff_t + 1 as ptrdiff_t);
                    xfree(p as *mut ::core::ffi::c_void);
                }
                redir_write(s, strlen(s) as ptrdiff_t);
            }
            if !(*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
                && (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    != 0 as linenr_T
            {
                logmsg(
                    LOGLVL_DBG,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    845 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"(:silent) %s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                    (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum,
                );
            } else {
                logmsg(
                    LOGLVL_DBG,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    847 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"(:silent) %s\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
            return true_0 != 0;
        }
        if !(*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
            && (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
                != 0 as linenr_T
        {
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                855 as ::core::ffi::c_int,
                true_0 != 0,
                b"%s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                s,
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
        } else {
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                857 as ::core::ffi::c_int,
                true_0 != 0,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                s,
            );
        }
        ex_exitval = 1 as ::core::ffi::c_int;
        msg_silent = 0 as ::core::ffi::c_int;
        cmd_silent = false_0 != 0;
        if global_busy != 0 {
            global_busy += 1;
        }
        if p_eb != 0 {
            beep_flush();
        } else {
            flush_buffers(FLUSH_MINIMAL);
        }
        did_emsg += 1;
    }
    emsg_on_display = true_0 != 0;
    if msg_scrolled != 0 as ::core::ffi::c_int {
        need_wait_return = true_0 != 0;
    }
    msg_ext_set_kind(kind);
    msg_scroll = true_0;
    let mut save_msg_skip_flush: bool = msg_ext_skip_flush;
    msg_ext_skip_flush = true_0 != 0;
    msg_source(hl_id);
    msg_nowait = false_0 != 0;
    let mut rv: ::core::ffi::c_int =
        msg_keep(s, hl_id, false_0 != 0, multiline) as ::core::ffi::c_int;
    msg_ext_skip_flush = save_msg_skip_flush;
    return rv != 0;
}
#[no_mangle]
pub unsafe extern "C" fn emsg(mut s: *const ::core::ffi::c_char) -> bool {
    return emsg_multiline(
        s,
        b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
        HLF_E as ::core::ffi::c_int,
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn emsg_invreg(mut name: ::core::ffi::c_int) {
    semsg(
        gettext(b"E354: Invalid register name: '%s'\0".as_ptr() as *const ::core::ffi::c_char),
        transchar_buf(::core::ptr::null::<buf_T>(), name),
    );
}
#[no_mangle]
pub unsafe extern "C" fn semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) -> bool {
    let mut ret: bool = false;
    let mut ap: ::core::ffi::VaListImpl;
    ap = c2rust_args.clone();
    ret = semsgv(fmt, ap.as_va_list());
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn semsg_multiline(
    mut kind: *const ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> bool {
    let mut ret: bool = false;
    let mut ap: ::core::ffi::VaListImpl;
    static mut errbuf: [::core::ffi::c_char; 8192] = [0; 8192];
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    ap = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut errbuf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>(),
        fmt,
        ap.as_va_list(),
    );
    ret = emsg_multiline(
        &raw mut errbuf as *mut ::core::ffi::c_char,
        kind,
        HLF_E as ::core::ffi::c_int,
        true_0 != 0,
    );
    return ret;
}
unsafe extern "C" fn semsgv(
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> bool {
    static mut errbuf: [::core::ffi::c_char; 1025] = [0; 1025];
    if emsg_not_now() != 0 {
        return true_0 != 0;
    }
    vim_vsnprintf(
        &raw mut errbuf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
        fmt,
        ap.as_va_list(),
    );
    return emsg(&raw mut errbuf as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn iemsg(mut s: *const ::core::ffi::c_char) {
    if emsg_not_now() != 0 {
        return;
    }
    emsg(s);
}
#[no_mangle]
pub unsafe extern "C" fn siemsg(mut s: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    if emsg_not_now() != 0 {
        return;
    }
    let mut ap: ::core::ffi::VaListImpl;
    ap = c2rust_args.clone();
    semsgv(s, ap.as_va_list());
}
#[no_mangle]
pub unsafe extern "C" fn internal_error(mut where_0: *const ::core::ffi::c_char) {
    siemsg(
        gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
        where_0,
    );
}
unsafe extern "C" fn msg_semsg_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut s: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    emsg(s);
    xfree(s as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn msg_schedule_semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    let mut ap: ::core::ffi::VaListImpl;
    ap = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        ap.as_va_list(),
    );
    let mut s: *mut ::core::ffi::c_char = xstrdup(&raw mut IObuff as *mut ::core::ffi::c_char);
    loop_schedule_deferred(
        &raw mut main_loop,
        Event {
            handler: Some(
                msg_semsg_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                s as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
unsafe extern "C" fn msg_semsg_multiline_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut s: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    emsg_multiline(
        s,
        b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
        HLF_E as ::core::ffi::c_int,
        true_0 != 0,
    );
    xfree(s as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn msg_schedule_semsg_multiline(
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut ap: ::core::ffi::VaListImpl;
    ap = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        ap.as_va_list(),
    );
    let mut s: *mut ::core::ffi::c_char = xstrdup(&raw mut IObuff as *mut ::core::ffi::c_char);
    loop_schedule_deferred(
        &raw mut main_loop,
        Event {
            handler: Some(
                msg_semsg_multiline_event
                    as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                s as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn msg_trunc(
    mut s: *mut ::core::ffi::c_char,
    mut force: bool,
    mut hl_id: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    msg_hist_add(s, -1 as ::core::ffi::c_int, hl_id);
    let mut ts: *mut ::core::ffi::c_char = msg_may_trunc(force, s);
    msg_hist_off = true_0 != 0;
    let mut n: bool = msg(ts, hl_id);
    msg_hist_off = false_0 != 0;
    if n {
        return ts;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn msg_may_trunc(
    mut force: bool,
    mut s: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if ui_has(kUIMessages) {
        return s;
    }
    let mut room: ::core::ffi::c_int =
        (Rows - cmdline_row - 1 as ::core::ffi::c_int) * Columns + sc_col - 1 as ::core::ffi::c_int;
    if room > 0 as ::core::ffi::c_int
        && (force as ::core::ffi::c_int != 0
            || shortmess(SHM_TRUNC as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && !exmode_active)
        && strlen(s) as ::core::ffi::c_int - room > 0 as ::core::ffi::c_int
    {
        let mut size: ::core::ffi::c_int = vim_strsize(s);
        if size <= room {
            return s;
        }
        let mut n: ::core::ffi::c_int = 0;
        n = 0 as ::core::ffi::c_int;
        while size >= room {
            size -= utf_ptr2cells(s.offset(n as isize));
            n += utfc_ptr2len(s.offset(n as isize));
        }
        n -= 1;
        s = s.offset(n as isize);
        *s = '<' as ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn msg_progress(
    mut s: *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_char,
    mut status: *mut ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
    mut trunc: bool,
) -> *mut ::core::ffi::c_char {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut opts: KeyDict_echo_opts = KeyDict_echo_opts {
        is_set__echo_opts_: 0,
        err: false,
        verbose: false,
        _truncate: false,
        kind: cstr_as_string(b"progress\0".as_ptr() as *const ::core::ffi::c_char),
        id: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: cstr_as_string(id),
            },
        },
        title: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        status: cstr_as_string(status),
        percent: 0,
        source: cstr_as_string(b"nvim\0".as_ptr() as *const ::core::ffi::c_char),
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    if hist as ::core::ffi::c_int != 0 && (!trunc || ui_has(kUIMessages) as ::core::ffi::c_int != 0)
    {
        msg_hist_add(s, -1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    }
    if trunc {
        s = msg_may_trunc(false_0 != 0, s);
    }
    let mut chunk: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chunk__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_11 { boolean: false },
    }; 2];
    chunk.capacity = 2 as size_t;
    chunk.items = &raw mut chunk__items as *mut Object;
    let mut chunks: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chunks__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_11 { boolean: false },
    }; 1];
    chunks.capacity = 1 as size_t;
    chunks.items = &raw mut chunks__items as *mut Object;
    let c2rust_fresh13 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh13 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_11 {
            string: cstr_as_string(s),
        },
    };
    let c2rust_fresh14 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: hl_id as Integer,
        },
    };
    let c2rust_fresh15 = chunks.size;
    chunks.size = chunks.size.wrapping_add(1);
    *chunks.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_11 { array: chunk },
    };
    nvim_echo(chunks, false_0 != 0, &raw mut opts, &raw mut err);
    ui_flush();
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn hl_msg_free(mut hl_msg: HlMessage) {
    let mut i: size_t = 0 as size_t;
    while i < hl_msg.size {
        xfree((*hl_msg.items.offset(i as isize)).text.data as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    xfree(hl_msg.items as *mut ::core::ffi::c_void);
    hl_msg.capacity = 0 as size_t;
    hl_msg.size = hl_msg.capacity;
    hl_msg.items = ::core::ptr::null_mut::<HlMessageChunk>();
}
unsafe extern "C" fn msg_hist_add(
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut text: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: if len < 0 as ::core::ffi::c_int {
            strlen(s)
        } else {
            len as size_t
        },
    };
    while text.size > 0 as size_t && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
        text.size = text.size.wrapping_sub(1);
        s = s.offset(1);
    }
    while text.size > 0 as size_t
        && *s.offset(text.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '\n' as ::core::ffi::c_int
    {
        text.size = text.size.wrapping_sub(1);
    }
    if text.size == 0 as size_t {
        return;
    }
    text.data = xmemdupz(s as *const ::core::ffi::c_void, text.size) as *mut ::core::ffi::c_char;
    let mut msg_0: HlMessage = HlMessage {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<HlMessageChunk>(),
    };
    if msg_0.size == msg_0.capacity {
        msg_0.capacity = if msg_0.capacity != 0 {
            msg_0.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        msg_0.items = xrealloc(
            msg_0.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
        ) as *mut HlMessageChunk;
    } else {
    };
    let c2rust_fresh7 = msg_0.size;
    msg_0.size = msg_0.size.wrapping_add(1);
    *msg_0.items.offset(c2rust_fresh7 as isize) = HlMessageChunk {
        text: text,
        hl_id: hl_id,
    };
    msg_hist_add_multihl(msg_0, false_0 != 0, ::core::ptr::null_mut::<MessageData>());
}
static mut do_clear_hist_temp: bool = true_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_progress(
    mut msg_id: Object,
    mut msg_0: HlMessage,
    mut msg_data: *mut MessageData,
) {
    if !has_event(EVENT_PROGRESS) {
        return;
    }
    let mut data: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut data__items: [KeyValuePair; 7] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        },
    }; 7];
    data.capacity = 7 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let mut messages: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut i: size_t = 0 as size_t;
    while i < msg_0.size {
        if messages.size == messages.capacity {
            messages.capacity = if messages.capacity != 0 {
                messages.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            messages.items = xrealloc(
                messages.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(messages.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh16 = messages.size;
        messages.size = messages.size.wrapping_add(1);
        *messages.items.offset(c2rust_fresh16 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_11 {
                string: (*msg_0.items.offset(i as isize)).text,
            },
        };
        i = i.wrapping_add(1);
    }
    let c2rust_fresh17 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: msg_id,
    };
    let c2rust_fresh18 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh18 as isize) = key_value_pair {
        key: cstr_as_string(b"text\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_11 { array: messages },
        },
    };
    if !msg_data.is_null() {
        let c2rust_fresh19 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"percent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_11 {
                    integer: (*msg_data).percent,
                },
            },
        };
        let c2rust_fresh20 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"source\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).source,
                },
            },
        };
        let c2rust_fresh21 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"status\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).status,
                },
            },
        };
        let c2rust_fresh22 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"title\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_11 {
                    string: (*msg_data).title,
                },
            },
        };
        let c2rust_fresh23 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed_11 {
                    dict: (*msg_data).data,
                },
            },
        };
    }
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed_11 { dict: data },
    };
    apply_autocmds_group(
        EVENT_PROGRESS,
        (if !msg_data.is_null() && (*msg_data).source.size > 0 as size_t {
            (*msg_data).source.data as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        AUGROUP_ALL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<buf_T>(),
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
    xfree(messages.items as *mut ::core::ffi::c_void);
    messages.capacity = 0 as size_t;
    messages.size = messages.capacity;
    messages.items = ::core::ptr::null_mut::<Object>();
}
unsafe extern "C" fn msg_hist_add_multihl(
    mut msg_0: HlMessage,
    mut temp: bool,
    mut _msg_data: *mut MessageData,
) {
    if do_clear_hist_temp {
        msg_hist_clear_temp();
        do_clear_hist_temp = false_0 != 0;
    }
    if msg_hist_off as ::core::ffi::c_int != 0 || msg_silent != 0 as ::core::ffi::c_int {
        hl_msg_free(msg_0);
        return;
    }
    let mut entry: *mut MessageHistoryEntry =
        xmalloc(::core::mem::size_of::<MessageHistoryEntry>()) as *mut MessageHistoryEntry;
    (*entry).msg = msg_0;
    (*entry).temp = temp;
    (*entry).kind = if !msg_ext_kind.is_null() {
        xstrdup(msg_ext_kind)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*entry).prev = msg_hist_last as *mut msg_hist;
    (*entry).next = ::core::ptr::null_mut::<msg_hist>();
    (*entry).append = msg_ext_append;
    if msg_hist_first.is_null() {
        msg_hist_first = entry;
    }
    if !msg_hist_last.is_null() {
        (*msg_hist_last).next = entry as *mut msg_hist;
    }
    if msg_hist_temp.is_null() {
        msg_hist_temp = entry;
    }
    msg_hist_len += !temp as ::core::ffi::c_int;
    msg_hist_last = entry;
    msg_ext_history = true_0 != 0;
    msg_hist_clear(msg_hist_max);
}
unsafe extern "C" fn msg_hist_free_msg(mut entry: *mut MessageHistoryEntry) {
    if (*entry).next.is_null() {
        msg_hist_last = (*entry).prev as *mut MessageHistoryEntry;
    } else {
        (*(*entry).next).prev = (*entry).prev;
    }
    if (*entry).prev.is_null() {
        msg_hist_first = (*entry).next as *mut MessageHistoryEntry;
    } else {
        (*(*entry).prev).next = (*entry).next;
    }
    if entry == msg_hist_temp {
        msg_hist_temp = (*entry).next as *mut MessageHistoryEntry;
    }
    hl_msg_free((*entry).msg);
    xfree((*entry).kind as *mut ::core::ffi::c_void);
    xfree(entry as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn msg_hist_clear(mut keep: ::core::ffi::c_int) {
    while msg_hist_len > keep || keep == 0 as ::core::ffi::c_int && !msg_hist_first.is_null() {
        msg_hist_len -= !(*msg_hist_first).temp as ::core::ffi::c_int;
        msg_hist_free_msg(msg_hist_first);
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_hist_clear_temp() {
    while !msg_hist_temp.is_null() {
        let mut next: *mut MessageHistoryEntry = (*msg_hist_temp).next as *mut MessageHistoryEntry;
        if (*msg_hist_temp).temp {
            msg_hist_free_msg(msg_hist_temp);
        }
        msg_hist_temp = next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn messagesopt_changed() -> ::core::ffi::c_int {
    let mut messages_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut messages_wait_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut messages_history_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut progress_target_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = p_mopt;
    while *p as ::core::ffi::c_int != NUL {
        if strnequal(
            p,
            b"hit-enter\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        ) {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_flags_new |= kOptMoptFlagHitEnter as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"wait:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        ) as ::core::ffi::c_int
            != 0
            && ascii_isdigit(*p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as isize,
            ) as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0
        {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_wait_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
            messages_flags_new |= kOptMoptFlagWait as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"history:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        ) as ::core::ffi::c_int
            != 0
            && ascii_isdigit(*p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                    as isize,
            ) as ::core::ffi::c_int) as ::core::ffi::c_int
                != 0
        {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_history_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
            messages_flags_new |= kOptMoptFlagHistory as ::core::ffi::c_int;
        } else if strnequal(
            p,
            b"progress:\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        ) {
            p = p.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as isize,
            );
            messages_flags_new |= kOptMoptFlagProgress as ::core::ffi::c_int;
            if *p as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                progress_target_flag |= PROGRESS_TARGET_CMD;
                p = p.offset(1);
            }
        }
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
    if messages_flags_new
        & (kOptMoptFlagHitEnter as ::core::ffi::c_int | kOptMoptFlagWait as ::core::ffi::c_int)
        == 0
    {
        return FAIL;
    }
    if messages_flags_new & kOptMoptFlagHistory as ::core::ffi::c_int == 0 {
        return FAIL;
    }
    '_c2rust_label: {
        if messages_history_new >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"messages_history_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1322 as ::core::ffi::c_uint,
                b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if messages_history_new > 10000 as ::core::ffi::c_int {
        return FAIL;
    }
    '_c2rust_label_0: {
        if messages_wait_new >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"messages_wait_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1328 as ::core::ffi::c_uint,
                b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if messages_wait_new > 10000 as ::core::ffi::c_int {
        return FAIL;
    }
    msg_flags = messages_flags_new;
    msg_wait = messages_wait_new;
    progress_msg_target = progress_target_flag;
    msg_hist_max = messages_history_new;
    msg_hist_clear(msg_hist_max);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_messages(mut eap: *mut exarg_T) {
    if strcmp(
        (*eap).arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        msg_hist_clear(if (*eap).addr_count != 0 {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut entries: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut p: *mut MessageHistoryEntry = if (*eap).skip != 0 {
        msg_hist_temp
    } else {
        msg_hist_first
    };
    let mut skip: ::core::ffi::c_int = if (*eap).addr_count != 0 {
        msg_hist_len - (*eap).line2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    while !p.is_null() {
        if !((*p).temp as ::core::ffi::c_int != 0 && (*eap).skip == 0 || {
            let c2rust_fresh24 = skip;
            skip = skip - 1;
            c2rust_fresh24 > 0 as ::core::ffi::c_int
        }) {
            if ui_has(kUIMessages) as ::core::ffi::c_int != 0 && msg_silent == 0 {
                let mut entry: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh25 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh25 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_11 {
                        string: cstr_to_string((*p).kind),
                    },
                };
                let mut content: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                let mut i: uint32_t = 0 as uint32_t;
                while (i as size_t) < (*p).msg.size {
                    let mut chunk: HlMessageChunk = *(*p).msg.items.offset(i as isize);
                    let mut content_entry: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh26 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh26 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_11 {
                            integer: (if chunk.hl_id != 0 {
                                syn_id2attr(chunk.hl_id)
                            } else {
                                0 as ::core::ffi::c_int
                            }) as Integer,
                        },
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh27 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh27 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_11 {
                            string: copy_string(chunk.text, ::core::ptr::null_mut::<Arena>()),
                        },
                    };
                    if content_entry.size == content_entry.capacity {
                        content_entry.capacity = if content_entry.capacity != 0 {
                            content_entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        content_entry.items = xrealloc(
                            content_entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(content_entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh28 = content_entry.size;
                    content_entry.size = content_entry.size.wrapping_add(1);
                    *content_entry.items.offset(c2rust_fresh28 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_11 {
                            integer: chunk.hl_id as Integer,
                        },
                    };
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
                    let c2rust_fresh29 = content.size;
                    content.size = content.size.wrapping_add(1);
                    *content.items.offset(c2rust_fresh29 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_11 {
                            array: content_entry,
                        },
                    };
                    i = i.wrapping_add(1);
                }
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh30 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh30 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_11 { array: content },
                };
                if entry.size == entry.capacity {
                    entry.capacity = if entry.capacity != 0 {
                        entry.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entry.items = xrealloc(
                        entry.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh31 = entry.size;
                entry.size = entry.size.wrapping_add(1);
                *entry.items.offset(c2rust_fresh31 as isize) = object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_11 {
                        boolean: (*p).append,
                    },
                };
                if entries.size == entries.capacity {
                    entries.capacity = if entries.capacity != 0 {
                        entries.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    entries.items = xrealloc(
                        entries.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(entries.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh32 = entries.size;
                entries.size = entries.size.wrapping_add(1);
                *entries.items.offset(c2rust_fresh32 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_11 { array: entry },
                };
            }
            if redirecting() != 0 || !ui_has(kUIMessages) {
                msg_silent += ui_has(kUIMessages) as ::core::ffi::c_int;
                let mut needs_clear: bool = false_0 != 0;
                msg_multihl(
                    object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_11 { boolean: false },
                    },
                    (*p).msg,
                    (*p).kind,
                    false_0 != 0,
                    false_0 != 0,
                    ::core::ptr::null_mut::<MessageData>(),
                    &raw mut needs_clear,
                );
                msg_silent -= ui_has(kUIMessages) as ::core::ffi::c_int;
            }
        }
        p = (*p).next as *mut MessageHistoryEntry;
    }
    if entries.size > 0 as size_t {
        ui_call_msg_history_show(entries, (*eap).skip != 0 as ::core::ffi::c_int);
        api_free_array(entries);
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_end_prompt() {
    need_wait_return = false_0 != 0;
    emsg_on_display = false_0 != 0;
    cmdline_row = msg_row;
    msg_col = 0 as ::core::ffi::c_int;
    msg_clr_eos();
    lines_left = -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn wait_return(mut redraw: ::core::ffi::c_int) {
    let mut c: ::core::ffi::c_int = 0;
    let mut had_got_int: ::core::ffi::c_int = 0;
    let mut save_scriptout: *mut FILE = ::core::ptr::null_mut::<FILE>();
    if redraw == true_0 {
        redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    if ui_has(kUIMessages) {
        prompt_for_input(
            b"Press any key to continue\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            HLF_M as ::core::ffi::c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        return;
    }
    if msg_silent != 0 as ::core::ffi::c_int {
        return;
    }
    if headless_mode as ::core::ffi::c_int != 0 && ui_active() == 0 {
        return;
    }
    if vgetc_busy > 0 as ::core::ffi::c_int {
        return;
    }
    need_wait_return = true_0 != 0;
    if no_wait_return != 0 {
        if !exmode_active {
            cmdline_row = msg_row;
        }
        return;
    }
    redir_off = true_0 != 0;
    let mut oldState: ::core::ffi::c_int = State;
    if quit_more {
        c = CAR;
        quit_more = false_0 != 0;
        got_int = false_0 != 0;
    } else if exmode_active {
        msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        c = CAR;
        got_int = false_0 != 0;
    } else if !stuff_empty() {
        c = CAR;
    } else {
        State = MODE_HITRETURN as ::core::ffi::c_int;
        setmouse();
        cmdline_row = msg_row;
        if need_check_timestamps {
            check_timestamps(false_0);
        }
        if p_ch == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled == 0 {
            msg_grid_validate();
            msg_scroll_up(false_0 != 0, true_0 != 0);
            msg_scrolled += 1;
            cmdline_row = Rows - 1 as ::core::ffi::c_int;
        }
        if msg_flags & kOptMoptFlagHitEnter as ::core::ffi::c_int != 0 {
            hit_return_msg(true_0 != 0);
            loop {
                had_got_int = got_int as ::core::ffi::c_int;
                no_mapping += 1;
                allow_keys += 1;
                let save_reg_recording: ::core::ffi::c_int = reg_recording;
                save_scriptout = scriptout;
                reg_recording = 0 as ::core::ffi::c_int;
                scriptout = ::core::ptr::null_mut::<FILE>();
                c = safe_vgetc();
                if had_got_int != 0 && global_busy == 0 {
                    got_int = false_0 != 0;
                }
                no_mapping -= 1;
                allow_keys -= 1;
                reg_recording = save_reg_recording;
                scriptout = save_scriptout;
                if p_more != 0 {
                    if c == 'b' as ::core::ffi::c_int
                        || c == Ctrl_B
                        || c == 'k' as ::core::ffi::c_int
                        || c == 'u' as ::core::ffi::c_int
                        || c == 'g' as ::core::ffi::c_int
                        || c == K_UP
                        || c == K_PAGEUP
                    {
                        if msg_scrolled > Rows {
                            do_more_prompt(c);
                        } else {
                            msg_didout = false_0 != 0;
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            msg_col = 0 as ::core::ffi::c_int;
                        }
                        if quit_more {
                            c = CAR;
                            quit_more = false_0 != 0;
                            got_int = false_0 != 0;
                        } else if c
                            != -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                        {
                            c = -(253 as ::core::ffi::c_int
                                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                            hit_return_msg(false_0 != 0);
                        }
                    } else if msg_scrolled > Rows - 2 as ::core::ffi::c_int
                        && (c == 'j' as ::core::ffi::c_int
                            || c == 'd' as ::core::ffi::c_int
                            || c == 'f' as ::core::ffi::c_int
                            || c == Ctrl_F
                            || c == K_DOWN
                            || c == K_PAGEDOWN)
                    {
                        c = -(253 as ::core::ffi::c_int
                            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
                    }
                }
                if !(had_got_int != 0 && c == Ctrl_C
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                    || c == -(253 as ::core::ffi::c_int
                        + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
                {
                    break;
                }
            }
            os_breakcheck();
            if c == -(253 as ::core::ffi::c_int
                + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                jump_to_mouse(
                    MOUSE_SETPOS as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<bool>(),
                    0 as ::core::ffi::c_int,
                );
            } else if vim_strchr(b"\r\n \0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
                && c != Ctrl_C
                && c != 'q' as ::core::ffi::c_int
            {
                ins_char_typebuf(vgetc_char, vgetc_mod_mask, true_0 != 0);
                do_redraw = true_0 != 0;
            }
        } else {
            c = CAR;
            do_sleep(msg_wait as int64_t, true_0 != 0);
        }
    }
    redir_off = false_0 != 0;
    if c == ':' as ::core::ffi::c_int
        || c == '?' as ::core::ffi::c_int
        || c == '/' as ::core::ffi::c_int
    {
        if !exmode_active {
            cmdline_row = msg_row;
        }
        skip_redraw = true_0 != 0;
        do_redraw = false_0 != 0;
    }
    let mut tmpState: ::core::ffi::c_int = State;
    State = oldState;
    setmouse();
    msg_check();
    need_wait_return = false_0 != 0;
    did_wait_return = true_0 != 0;
    emsg_on_display = false_0 != 0;
    lines_left = -1 as ::core::ffi::c_int;
    reset_last_sourcing();
    if !keep_msg.is_null()
        && vim_strsize(keep_msg)
            >= (Rows - cmdline_row - 1 as ::core::ffi::c_int) * Columns + sc_col
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut keep_msg as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
    }
    if tmpState == MODE_SETWSIZE as ::core::ffi::c_int {
        ui_refresh();
    } else if !skip_redraw {
        if redraw == true_0
            || msg_scrolled != 0 as ::core::ffi::c_int && redraw != -1 as ::core::ffi::c_int
        {
            redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
        }
    }
}
unsafe extern "C" fn hit_return_msg(mut newline_sb: bool) {
    let mut save_p_more: ::core::ffi::c_int = p_more;
    if !newline_sb {
        p_more = false_0;
    }
    if msg_didout {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    p_more = false_0;
    if got_int {
        msg_puts(gettext(
            b"Interrupt: \0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    msg_puts_hl(
        gettext(b"Press ENTER or type command to continue\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_R as ::core::ffi::c_int,
        false_0 != 0,
    );
    if msg_use_printf() == 0 {
        msg_clr_eos();
    }
    p_more = save_p_more;
}
#[no_mangle]
pub unsafe extern "C" fn set_keep_msg(
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    if ui_has(kUIMessages) {
        return;
    }
    xfree(keep_msg as *mut ::core::ffi::c_void);
    if !s.is_null() && msg_silent == 0 as ::core::ffi::c_int {
        keep_msg = xstrdup(s);
    } else {
        keep_msg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    keep_msg_more = false_0 != 0;
    keep_msg_hl_id = hl_id;
}
#[no_mangle]
pub unsafe extern "C" fn messaging() -> bool {
    return !(p_lz != 0 && char_avail() as ::core::ffi::c_int != 0 && !KeyTyped)
        && (p_ch > 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn msgmore(mut n: ::core::ffi::c_int) {
    let mut pn: ::core::ffi::c_int = 0;
    if global_busy != 0 || !messaging() {
        return;
    }
    if !keep_msg.is_null() && !keep_msg_more {
        return;
    }
    pn = abs(n);
    if pn as OptInt > p_report {
        if n > 0 as ::core::ffi::c_int {
            vim_snprintf(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                ngettext(
                    b"%d more line\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%d more lines\0".as_ptr() as *const ::core::ffi::c_char,
                    pn as ::core::ffi::c_ulong,
                ),
                pn,
            );
        } else {
            vim_snprintf(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                ngettext(
                    b"%d line less\0".as_ptr() as *const ::core::ffi::c_char,
                    b"%d fewer lines\0".as_ptr() as *const ::core::ffi::c_char,
                    pn as ::core::ffi::c_ulong,
                ),
                pn,
            );
        }
        if got_int {
            xstrlcat(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                gettext(b" (Interrupted)\0".as_ptr() as *const ::core::ffi::c_char),
                MSG_BUF_LEN as size_t,
            );
        }
        if msg(
            &raw mut msg_buf as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        ) {
            set_keep_msg(
                &raw mut msg_buf as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            keep_msg_more = true_0 != 0;
        }
    }
}
static mut redir_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn msg_ext_set_kind(mut msg_kind: *const ::core::ffi::c_char) {
    msg_ext_ui_flush();
    msg_ext_kind = msg_kind;
    redir_col = if msg_ext_append as ::core::ffi::c_int != 0 {
        redir_col
    } else {
        0 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn msg_ext_set_append(mut append: bool) {
    msg_ext_ui_flush();
    msg_ext_append = append;
}
#[no_mangle]
pub unsafe extern "C" fn msg_ext_set_trigger(mut trigger: *const ::core::ffi::c_char) {
    msg_ext_ui_flush();
    msg_ext_trigger = trigger;
}
#[no_mangle]
pub unsafe extern "C" fn msg_start() {
    let mut did_return: bool = false_0 != 0;
    msg_row = if msg_row > cmdline_row {
        msg_row
    } else {
        cmdline_row
    };
    if msg_silent == 0 {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut keep_msg as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        need_fileinfo = false_0 != 0;
    }
    if need_highlight_changed {
        highlight_changed();
    }
    if need_clr_eos as ::core::ffi::c_int != 0
        || p_ch == 0 as OptInt && redrawing_cmdline as ::core::ffi::c_int != 0
    {
        need_clr_eos = false_0 != 0;
        msg_clr_eos();
    }
    if p_ch == 0 as OptInt && !ui_has(kUIMessages) && msg_scrolled == 0 {
        msg_grid_validate();
        msg_scroll_up(false_0 != 0, true_0 != 0);
        msg_scrolled += 1;
        cmdline_row = Rows - 1 as ::core::ffi::c_int;
    }
    if msg_scroll == 0 && full_screen as ::core::ffi::c_int != 0 {
        msg_row = cmdline_row;
        msg_col = 0 as ::core::ffi::c_int;
    } else if (msg_didout as ::core::ffi::c_int != 0 || p_ch == 0 as OptInt) && !ui_has(kUIMessages)
    {
        if p_ch == 0 as OptInt && !msg_didout && msg_use_printf() != 0 {
            msg_puts_display(
                b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0,
            );
        } else {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        did_return = true_0 != 0;
        cmdline_row = msg_row;
    }
    if !msg_didany || lines_left < 0 as ::core::ffi::c_int {
        msg_starthere();
    }
    if msg_silent == 0 as ::core::ffi::c_int {
        msg_didout = false_0 != 0;
    }
    if ui_has(kUIMessages) {
        msg_ext_ui_flush();
    }
    if !did_return {
        redir_write(
            b"\n\0".as_ptr() as *const ::core::ffi::c_char,
            1 as ptrdiff_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_starthere() {
    lines_left = cmdline_row;
    msg_didany = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn msg_putchar(mut c: ::core::ffi::c_int) {
    msg_putchar_hl(c, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn msg_putchar_hl(mut c: ::core::ffi::c_int, mut hl_id: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    if c < 0 as ::core::ffi::c_int {
        buf[0 as ::core::ffi::c_int as usize] = K_SPECIAL as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL {
            KS_SPECIAL
        } else if c == NUL {
            KS_ZERO
        } else {
            -c & 0xff as ::core::ffi::c_int
        }) as ::core::ffi::c_char;
        buf[2 as ::core::ffi::c_int as usize] = (if c == K_SPECIAL || c == NUL {
            KE_FILLER as ::core::ffi::c_uint
        } else {
            -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint
        }) as ::core::ffi::c_char;
        buf[3 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    } else {
        buf[utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
            NUL as ::core::ffi::c_char;
    }
    msg_puts_hl(
        &raw mut buf as *mut ::core::ffi::c_char,
        hl_id,
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn msg_outnum(mut n: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 20] = [0; 20];
    snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        n,
    );
    msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn msg_home_replace(mut fname: *const ::core::ffi::c_char) {
    msg_home_replace_hl(fname, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn msg_home_replace_hl(
    mut fname: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut name: *mut ::core::ffi::c_char =
        home_replace_save(::core::ptr::null_mut::<buf_T>(), fname);
    msg_outtrans(name, hl_id, false_0 != 0);
    xfree(name as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn msg_outtrans(
    mut str: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    return msg_outtrans_len(str, strlen(str) as ::core::ffi::c_int, hl_id, hist);
}
#[no_mangle]
pub unsafe extern "C" fn msg_outtrans_one(
    mut p: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> *const ::core::ffi::c_char {
    let mut l: ::core::ffi::c_int = 0;
    l = utfc_ptr2len(p);
    if l > 1 as ::core::ffi::c_int {
        msg_outtrans_len(p, l, hl_id, hist);
        return p.offset(l as isize);
    }
    msg_puts_hl(
        transchar_byte_buf(
            ::core::ptr::null::<buf_T>(),
            *p as uint8_t as ::core::ffi::c_int,
        ),
        hl_id,
        hist,
    );
    return p.offset(1 as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn msg_outtrans_len(
    mut msgstr: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut str: *const ::core::ffi::c_char = msgstr;
    let mut plain_start: *const ::core::ffi::c_char = msgstr;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c: ::core::ffi::c_int = 0;
    let mut save_got_int: ::core::ffi::c_int = got_int as ::core::ffi::c_int;
    got_int = false_0 != 0;
    if hist {
        msg_hist_add(str, len, hl_id);
    }
    if msg_silent == 0 as ::core::ffi::c_int
        && len > 0 as ::core::ffi::c_int
        && msg_row >= cmdline_row
        && msg_col == 0 as ::core::ffi::c_int
    {
        clear_cmdline = false_0 != 0;
        mode_displayed = false_0 != 0;
    }
    loop {
        len -= 1;
        if !(len >= 0 as ::core::ffi::c_int && !got_int) {
            break;
        }
        let mut mb_l: ::core::ffi::c_int = utfc_ptr2len_len(str, len + 1 as ::core::ffi::c_int);
        if mb_l > 1 as ::core::ffi::c_int {
            c = utf_ptr2char(str);
            if vim_isprintc(c) {
                retval += utf_ptr2cells(str);
            } else {
                if str > plain_start {
                    msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                }
                plain_start = str.offset(mb_l as isize);
                msg_puts_hl(
                    transchar_buf(::core::ptr::null::<buf_T>(), c),
                    if hl_id == 0 as ::core::ffi::c_int {
                        HLF_8 as ::core::ffi::c_int
                    } else {
                        hl_id
                    },
                    false_0 != 0,
                );
                retval += char2cells(c);
            }
            len -= mb_l - 1 as ::core::ffi::c_int;
            str = str.offset(mb_l as isize);
        } else {
            s = transchar_byte_buf(
                ::core::ptr::null::<buf_T>(),
                *str as uint8_t as ::core::ffi::c_int,
            );
            if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                if str > plain_start {
                    msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
                }
                plain_start = str.offset(1 as ::core::ffi::c_int as isize);
                msg_puts_hl(
                    s,
                    if hl_id == 0 as ::core::ffi::c_int {
                        HLF_8 as ::core::ffi::c_int
                    } else {
                        hl_id
                    },
                    false_0 != 0,
                );
                retval += strlen(s) as ::core::ffi::c_int;
            } else {
                retval += 1;
            }
            str = str.offset(1);
        }
    }
    if (str > plain_start || plain_start == msgstr) && !got_int {
        msg_puts_len(plain_start, str.offset_from(plain_start), hl_id, hist);
    }
    got_int = got_int as ::core::ffi::c_int | save_got_int != 0;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn msg_make(mut arg: *const ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0;
    static mut str: *const ::core::ffi::c_char = b"eeffoc\0".as_ptr() as *const ::core::ffi::c_char;
    static mut rs: *const ::core::ffi::c_char =
        b"Plon#dqg#vxjduB\0".as_ptr() as *const ::core::ffi::c_char;
    arg = skipwhite(arg);
    i = 5 as ::core::ffi::c_int;
    while *arg as ::core::ffi::c_int != 0 && i >= 0 as ::core::ffi::c_int {
        let c2rust_fresh33 = arg;
        arg = arg.offset(1);
        if *c2rust_fresh33 as ::core::ffi::c_int != *str.offset(i as isize) as ::core::ffi::c_int {
            break;
        }
        i -= 1;
    }
    if i < 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        i = 0 as ::core::ffi::c_int;
        while *rs.offset(i as isize) != 0 {
            msg_putchar(*rs.offset(i as isize) as ::core::ffi::c_int - 3 as ::core::ffi::c_int);
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_outtrans_special(
    mut strstart: *const ::core::ffi::c_char,
    mut from: bool,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if strstart.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut str: *const ::core::ffi::c_char = strstart;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hl_id: ::core::ffi::c_int = HLF_8 as ::core::ffi::c_int;
    while *str as ::core::ffi::c_int != NUL {
        let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (str == strstart
            || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            && *str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        {
            text = b"<Space>\0".as_ptr() as *const ::core::ffi::c_char;
            str = str.offset(1);
        } else {
            text = str2special(&raw mut str, from, false_0 != 0);
        }
        if *text.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *text.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            text = transchar_byte_buf(
                ::core::ptr::null::<buf_T>(),
                *text.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            );
        }
        let len: ::core::ffi::c_int = vim_strsize(text);
        if maxlen > 0 as ::core::ffi::c_int && retval + len >= maxlen {
            break;
        }
        msg_puts_hl(
            text,
            if len > 1 as ::core::ffi::c_int && utfc_ptr2len(text) <= 1 as ::core::ffi::c_int {
                hl_id
            } else {
                0 as ::core::ffi::c_int
            },
            false_0 != 0,
        );
        retval += len;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn str2special_save(
    str: *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut ::core::ffi::c_char {
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
        40 as ::core::ffi::c_int,
    );
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        ga_concat(
            &raw mut ga,
            str2special(&raw mut p, replace_spaces, replace_lt),
        );
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn str2special_arena(
    mut str: *const ::core::ffi::c_char,
    mut replace_spaces: bool,
    mut replace_lt: bool,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    let mut p: *const ::core::ffi::c_char = str;
    let mut len: size_t = 0 as size_t;
    while *p != 0 {
        len = len.wrapping_add(strlen(str2special(&raw mut p, replace_spaces, replace_lt)));
    }
    let mut buf: *mut ::core::ffi::c_char =
        arena_alloc(arena, len.wrapping_add(1 as size_t), false_0 != 0) as *mut ::core::ffi::c_char;
    let mut pos: size_t = 0 as size_t;
    p = str;
    while *p != 0 {
        let mut s: *const ::core::ffi::c_char = str2special(&raw mut p, replace_spaces, replace_lt);
        let mut s_len: size_t = strlen(s);
        memcpy(
            buf.offset(pos as isize) as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            s_len,
        );
        pos = pos.wrapping_add(s_len);
    }
    *buf.offset(pos as isize) = NUL as ::core::ffi::c_char;
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn str2special(
    sp: *mut *const ::core::ffi::c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *const ::core::ffi::c_char {
    static mut buf: [::core::ffi::c_char; 7] = [0; 7];
    let p: *const ::core::ffi::c_char = mb_unescape(sp);
    if !p.is_null() {
        return p;
    }
    let mut str: *const ::core::ffi::c_char = *sp;
    let mut c: ::core::ffi::c_int = *str as uint8_t as ::core::ffi::c_int;
    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut special: bool = false_0 != 0;
    if c == K_SPECIAL
        && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == KS_MODIFIER
        {
            modifiers =
                *str.offset(2 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
            str = str.offset(3 as ::core::ffi::c_int as isize);
            c = *str as uint8_t as ::core::ffi::c_int;
        }
        if c == K_SPECIAL
            && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            c = if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_SPECIAL
            {
                K_SPECIAL
            } else if *str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_ZERO
            {
                K_ZERO
            } else {
                -(*str.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                    + ((*str.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            str = str.offset(2 as ::core::ffi::c_int as isize);
        }
        if c < 0 as ::core::ffi::c_int || modifiers != 0 {
            special = true_0 != 0;
        }
    }
    if !(c < 0 as ::core::ffi::c_int)
        && utf8len_tab[c as usize] as ::core::ffi::c_int > 1 as ::core::ffi::c_int
    {
        *sp = str;
        let mut p_0: *const ::core::ffi::c_char = mb_unescape(sp);
        if !p_0.is_null() {
            c = utf_ptr2char(p_0);
        } else {
            *sp = str.offset(1 as ::core::ffi::c_int as isize);
        }
    } else {
        *sp = str.offset(
            (if *str as ::core::ffi::c_int == NUL {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as isize,
        );
    }
    if special as ::core::ffi::c_int != 0
        || c < ' ' as ::core::ffi::c_int
        || replace_spaces as ::core::ffi::c_int != 0 && c == ' ' as ::core::ffi::c_int
        || replace_lt as ::core::ffi::c_int != 0 && c == '<' as ::core::ffi::c_int
    {
        return get_special_key_name(c, modifiers);
    }
    buf[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return &raw mut buf as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn str2specialbuf(
    mut sp: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    while *sp != 0 {
        let mut s: *const ::core::ffi::c_char =
            str2special(&raw mut sp, false_0 != 0, false_0 != 0);
        let s_len: size_t = strlen(s);
        if len <= s_len {
            break;
        }
        memcpy(
            buf as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            s_len,
        );
        buf = buf.offset(s_len as isize);
        len = len.wrapping_sub(s_len);
    }
    *buf = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn msg_prt_line(mut s: *const ::core::ffi::c_char, mut list: bool) {
    let mut sc: schar_T = 0;
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sc_extra: schar_T = 0 as schar_T;
    let mut sc_final: schar_T = 0 as schar_T;
    let mut p_extra: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lead: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut in_multispace: bool = false_0 != 0;
    let mut multispace_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut trail: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut l: ::core::ffi::c_int = 0;
    if (*curwin).w_onebuf_opt.wo_list != 0 {
        list = true_0 != 0;
    }
    if list {
        if (*curwin).w_p_lcs_chars.trail != 0 {
            trail = s.offset(strlen(s) as isize);
            while trail > s
                && ascii_iswhite(
                    *trail.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
            {
                trail = trail.offset(-1);
            }
        }
        if (*curwin).w_p_lcs_chars.lead != 0
            || !(*curwin).w_p_lcs_chars.leadmultispace.is_null()
            || (*curwin).w_p_lcs_chars.leadtab1 != NUL as schar_T
        {
            lead = s;
            while ascii_iswhite(*lead.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            {
                lead = lead.offset(1);
            }
            if *lead as ::core::ffi::c_int == NUL {
                lead = ::core::ptr::null::<::core::ffi::c_char>();
            }
        }
    }
    if *s as ::core::ffi::c_int == NUL
        && !(list as ::core::ffi::c_int != 0 && (*curwin).w_p_lcs_chars.eol != NUL as schar_T)
    {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    while !got_int {
        if n_extra > 0 as ::core::ffi::c_int {
            n_extra -= 1;
            if n_extra == 0 as ::core::ffi::c_int && sc_final != 0 {
                sc = sc_final;
            } else if sc_extra != 0 {
                sc = sc_extra;
            } else {
                '_c2rust_label: {
                    if !p_extra.is_null() {
                    } else {
                        __assert_fail(
                            b"p_extra != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2209 as ::core::ffi::c_uint,
                            b"void msg_prt_line(const char *, _Bool)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                let c2rust_fresh34 = p_extra;
                p_extra = p_extra.offset(1);
                sc = *c2rust_fresh34 as ::core::ffi::c_uchar as schar_T;
            }
        } else {
            l = utfc_ptr2len(s);
            if l > 1 as ::core::ffi::c_int {
                col += utf_ptr2cells(s);
                let mut buf: [::core::ffi::c_char; 22] = [0; 22];
                if l >= MB_MAXBYTES as ::core::ffi::c_int {
                    xstrlcpy(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        b"?\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 22]>(),
                    );
                } else if (*curwin).w_p_lcs_chars.nbsp != NUL as schar_T
                    && list as ::core::ffi::c_int != 0
                    && (utf_ptr2char(s) == 160 as ::core::ffi::c_int
                        || utf_ptr2char(s) == 0x202f as ::core::ffi::c_int)
                {
                    schar_get(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        (*curwin).w_p_lcs_chars.nbsp,
                    );
                } else {
                    memmove(
                        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        s as *const ::core::ffi::c_void,
                        l as size_t,
                    );
                    buf[l as usize] = NUL as ::core::ffi::c_char;
                }
                msg_puts(&raw mut buf as *mut ::core::ffi::c_char);
                s = s.offset(l as isize);
                continue;
            } else {
                hl_id = 0 as ::core::ffi::c_int;
                let c2rust_fresh35 = s;
                s = s.offset(1);
                let mut c: ::core::ffi::c_int = *c2rust_fresh35 as uint8_t as ::core::ffi::c_int;
                if c >= 0x80 as ::core::ffi::c_int {
                    col += utf_char2cells(c);
                    msg_putchar(c);
                    continue;
                } else {
                    sc_extra = NUL as schar_T;
                    sc_final = NUL as schar_T;
                    if list {
                        in_multispace = c == ' ' as ::core::ffi::c_int
                            && (*s as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                                || col > 0 as ::core::ffi::c_int
                                    && *s.offset(-2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ' ' as ::core::ffi::c_int);
                        if !in_multispace {
                            multispace_pos = 0 as ::core::ffi::c_int;
                        }
                    }
                    if c == TAB && (!list || (*curwin).w_p_lcs_chars.tab1 != 0) {
                        n_extra = tabstop_padding(
                            col as colnr_T,
                            (*curbuf).b_p_ts,
                            (*curbuf).b_p_vts_array,
                        ) - 1 as ::core::ffi::c_int;
                        if !list {
                            sc = ' ' as ::core::ffi::c_int as schar_T;
                            sc_extra = ' ' as ::core::ffi::c_int as schar_T;
                        } else {
                            let mut lcs_tab1: schar_T = (*curwin).w_p_lcs_chars.tab1;
                            let mut lcs_tab2: schar_T = (*curwin).w_p_lcs_chars.tab2;
                            let mut lcs_tab3: schar_T = (*curwin).w_p_lcs_chars.tab3;
                            if !lead.is_null()
                                && s <= lead
                                && (*curwin).w_p_lcs_chars.leadtab1 != NUL as schar_T
                            {
                                lcs_tab1 = (*curwin).w_p_lcs_chars.leadtab1;
                                lcs_tab2 = (*curwin).w_p_lcs_chars.leadtab2;
                                lcs_tab3 = (*curwin).w_p_lcs_chars.leadtab3;
                            }
                            sc = if n_extra == 0 as ::core::ffi::c_int && lcs_tab3 != 0 {
                                lcs_tab3
                            } else {
                                lcs_tab1
                            };
                            sc_extra = lcs_tab2;
                            sc_final = lcs_tab3;
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        }
                    } else if c == NUL
                        && list as ::core::ffi::c_int != 0
                        && (*curwin).w_p_lcs_chars.eol != NUL as schar_T
                    {
                        p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char;
                        n_extra = 1 as ::core::ffi::c_int;
                        sc = (*curwin).w_p_lcs_chars.eol;
                        hl_id = HLF_AT as ::core::ffi::c_int;
                        s = s.offset(-1);
                    } else if c != NUL && {
                        n = byte2cells(c);
                        n > 1 as ::core::ffi::c_int
                    } {
                        n_extra = n - 1 as ::core::ffi::c_int;
                        p_extra = transchar_byte_buf(::core::ptr::null::<buf_T>(), c);
                        let c2rust_fresh36 = p_extra;
                        p_extra = p_extra.offset(1);
                        sc = *c2rust_fresh36 as schar_T;
                        hl_id = HLF_0 as ::core::ffi::c_int;
                    } else if c == ' ' as ::core::ffi::c_int {
                        if !lead.is_null()
                            && s <= lead
                            && in_multispace as ::core::ffi::c_int != 0
                            && !(*curwin).w_p_lcs_chars.leadmultispace.is_null()
                        {
                            let c2rust_fresh37 = multispace_pos;
                            multispace_pos = multispace_pos + 1;
                            sc = *(*curwin)
                                .w_p_lcs_chars
                                .leadmultispace
                                .offset(c2rust_fresh37 as isize);
                            if *(*curwin)
                                .w_p_lcs_chars
                                .leadmultispace
                                .offset(multispace_pos as isize)
                                == NUL as schar_T
                            {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        } else if !lead.is_null()
                            && s <= lead
                            && (*curwin).w_p_lcs_chars.lead != NUL as schar_T
                        {
                            sc = (*curwin).w_p_lcs_chars.lead;
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        } else if !trail.is_null() && s > trail {
                            sc = (*curwin).w_p_lcs_chars.trail;
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        } else if in_multispace as ::core::ffi::c_int != 0
                            && !(*curwin).w_p_lcs_chars.multispace.is_null()
                        {
                            let c2rust_fresh38 = multispace_pos;
                            multispace_pos = multispace_pos + 1;
                            sc = *(*curwin)
                                .w_p_lcs_chars
                                .multispace
                                .offset(c2rust_fresh38 as isize);
                            if *(*curwin)
                                .w_p_lcs_chars
                                .multispace
                                .offset(multispace_pos as isize)
                                == NUL as schar_T
                            {
                                multispace_pos = 0 as ::core::ffi::c_int;
                            }
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        } else if list as ::core::ffi::c_int != 0
                            && (*curwin).w_p_lcs_chars.space != NUL as schar_T
                        {
                            sc = (*curwin).w_p_lcs_chars.space;
                            hl_id = HLF_0 as ::core::ffi::c_int;
                        } else {
                            sc = ' ' as ::core::ffi::c_int as schar_T;
                        }
                    } else {
                        sc = c as schar_T;
                    }
                }
            }
        }
        if sc == NUL as schar_T {
            break;
        }
        let mut buf_0: [::core::ffi::c_char; 32] = [0; 32];
        schar_get(&raw mut buf_0 as *mut ::core::ffi::c_char, sc);
        msg_puts_hl(
            &raw mut buf_0 as *mut ::core::ffi::c_char,
            hl_id,
            false_0 != 0,
        );
        col += 1;
    }
    msg_clr_eos();
}
#[no_mangle]
pub unsafe extern "C" fn msg_puts(mut s: *const ::core::ffi::c_char) {
    msg_puts_hl(s, 0 as ::core::ffi::c_int, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn msg_puts_title(mut s: *const ::core::ffi::c_char) {
    s = s.offset(
        (ui_has(kUIMessages) as ::core::ffi::c_int != 0
            && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int) as ::core::ffi::c_int
            as isize,
    );
    msg_puts_hl(s, HLF_T as ::core::ffi::c_int, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn msg_outtrans_long(
    mut longstr: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
) {
    let mut len: ::core::ffi::c_int = strlen(longstr) as ::core::ffi::c_int;
    let mut slen: ::core::ffi::c_int = len;
    let mut room: ::core::ffi::c_int = Columns - msg_col;
    if !ui_has(kUIMessages) && len > room && room >= 20 as ::core::ffi::c_int {
        slen = (room - 3 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
        msg_outtrans_len(longstr, slen, hl_id, false_0 != 0);
        msg_puts_hl(
            b"...\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    msg_outtrans_len(
        longstr.offset(len as isize).offset(-(slen as isize)),
        slen,
        hl_id,
        false_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn msg_puts_hl(
    s: *const ::core::ffi::c_char,
    hl_id: ::core::ffi::c_int,
    hist: bool,
) {
    msg_puts_len(s, -1 as ptrdiff_t, hl_id, hist);
}
#[no_mangle]
pub unsafe extern "C" fn msg_puts_len(
    str: *const ::core::ffi::c_char,
    len: ptrdiff_t,
    mut hl_id: ::core::ffi::c_int,
    mut hist: bool,
) {
    '_c2rust_label: {
        if len < 0 as ptrdiff_t
            || memchr(
                str as *const ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                len as size_t,
            )
            .is_null()
        {
        } else {
            __assert_fail(
                b"len < 0 || memchr(str, 0, (size_t)len) == NULL\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2367 as ::core::ffi::c_uint,
                b"void msg_puts_len(const char *const, const ptrdiff_t, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    redir_write(str, len);
    if msg_silent != 0 as ::core::ffi::c_int || *str as ::core::ffi::c_int == NUL {
        if *str as ::core::ffi::c_int == NUL && ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            msg_ext_ui_flush();
            ui_call_msg_show(
                cstr_as_string(b"empty\0".as_ptr() as *const ::core::ffi::c_char),
                Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                },
                false_0 != 0,
                false_0 != 0,
                false_0 != 0,
                object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_11 {
                        integer: -1 as Integer,
                    },
                },
                String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                },
            );
            cmdline_was_last_drawn = false_0 != 0;
        }
        return;
    }
    if hist {
        msg_hist_add(str, len as ::core::ffi::c_int, hl_id);
    }
    let mut overflow: bool = !ui_has(kUIMessages)
        && msg_scrolled
            > (if p_ch == 0 as OptInt {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
    if overflow as ::core::ffi::c_int != 0
        && !msg_scrolled_ign
        && strcmp(str, b"\r\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int
    {
        need_wait_return = true_0 != 0;
    }
    msg_didany = true_0 != 0;
    if msg_use_printf() != 0 {
        let mut saved_msg_col: ::core::ffi::c_int = msg_col;
        msg_puts_printf(str, len);
        if headless_mode {
            msg_col = saved_msg_col;
        }
    }
    if msg_use_printf() == 0
        || headless_mode as ::core::ffi::c_int != 0 && !default_grid.chars.is_null()
    {
        msg_puts_display(str, len as ::core::ffi::c_int, hl_id, false_0);
    }
    need_fileinfo = false_0 != 0;
}
unsafe extern "C" fn msg_ext_emit_chunk() {
    if msg_ext_chunks.is_null() {
        msg_ext_init_chunks();
    }
    if msg_ext_last_attr == -1 as sattr_T {
        return;
    }
    let mut chunk: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh1 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh1 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: msg_ext_last_attr as Integer,
        },
    };
    msg_ext_last_attr = -1 as ::core::ffi::c_int as sattr_T;
    let mut text: String_0 = ga_take_string(&raw mut msg_ext_last_chunk);
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh2 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_11 { string: text },
    };
    if chunk.size == chunk.capacity {
        chunk.capacity = if chunk.capacity != 0 {
            chunk.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        chunk.items = xrealloc(
            chunk.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul(chunk.capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh3 = chunk.size;
    chunk.size = chunk.size.wrapping_add(1);
    *chunk.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed_11 {
            integer: msg_ext_last_hl_id as Integer,
        },
    };
    if (*msg_ext_chunks).size == (*msg_ext_chunks).capacity {
        (*msg_ext_chunks).capacity = if (*msg_ext_chunks).capacity != 0 {
            (*msg_ext_chunks).capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*msg_ext_chunks).items = xrealloc(
            (*msg_ext_chunks).items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<Object>().wrapping_mul((*msg_ext_chunks).capacity),
        ) as *mut Object;
    } else {
    };
    let c2rust_fresh4 = (*msg_ext_chunks).size;
    (*msg_ext_chunks).size = (*msg_ext_chunks).size.wrapping_add(1);
    *(*msg_ext_chunks).items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_11 { array: chunk },
    };
}
unsafe extern "C" fn msg_puts_display(
    mut str: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut recurse: ::core::ffi::c_int,
) {
    let mut s: *const ::core::ffi::c_char = str;
    let mut sb_str: *const ::core::ffi::c_char = str;
    let mut sb_col: ::core::ffi::c_int = msg_col;
    let mut attr: ::core::ffi::c_int = if hl_id != 0 {
        syn_id2attr(hl_id)
    } else {
        0 as ::core::ffi::c_int
    };
    did_wait_return = false_0 != 0;
    if ui_has(kUIMessages) {
        if attr as sattr_T != msg_ext_last_attr {
            msg_ext_emit_chunk();
            msg_ext_last_attr = attr as sattr_T;
            msg_ext_last_hl_id = hl_id;
        }
        let mut len: size_t = if maxlen < 0 as ::core::ffi::c_int {
            strlen(str)
        } else {
            strnlen(str, maxlen as size_t)
        };
        ga_concat_len(&raw mut msg_ext_last_chunk, str, len);
        let mut lastline: *const ::core::ffi::c_char =
            xmemrchr(str as *const ::core::ffi::c_void, '\n' as uint8_t, len)
                as *const ::core::ffi::c_char;
        maxlen -= (if !lastline.is_null() {
            lastline.offset_from(str)
        } else {
            0 as isize
        }) as ::core::ffi::c_int;
        let mut p: *const ::core::ffi::c_char = if !lastline.is_null() {
            lastline.offset(1 as ::core::ffi::c_int as isize)
        } else {
            str
        };
        let mut col: ::core::ffi::c_int = (if maxlen < 0 as ::core::ffi::c_int {
            mb_string2cells(p)
        } else {
            mb_string2cells_len(p, maxlen as size_t)
        }) as ::core::ffi::c_int;
        msg_col = (if !lastline.is_null() {
            0 as ::core::ffi::c_int
        } else {
            msg_col
        }) + col;
        return;
    }
    let mut print_attr: ::core::ffi::c_int = hl_combine_attr(
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
        attr,
    );
    msg_grid_validate();
    cmdline_was_last_drawn = redrawing_cmdline;
    let mut msg_row_pending: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    loop {
        if msg_col >= Columns {
            if p_more != 0 && recurse == 0 {
                store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
            }
            if msg_no_more as ::core::ffi::c_int != 0 && lines_left == 0 as ::core::ffi::c_int {
                break;
            }
            msg_col = 0 as ::core::ffi::c_int;
            msg_row += 1;
            msg_didout = false_0 != 0;
        }
        if msg_row >= Rows {
            msg_row = Rows - 1 as ::core::ffi::c_int;
            if msg_no_more as ::core::ffi::c_int != 0 && lines_left == 0 as ::core::ffi::c_int {
                break;
            }
            if recurse == 0 {
                if msg_row_pending >= 0 as ::core::ffi::c_int {
                    msg_line_flush();
                    msg_row_pending = -1 as ::core::ffi::c_int;
                }
                msg_scroll_up(true_0 != 0, false_0 != 0);
                inc_msg_scrolled();
                need_wait_return = true_0 != 0;
                redraw_cmdline = true_0 != 0;
                if cmdline_row > 0 as ::core::ffi::c_int && !exmode_active {
                    cmdline_row -= 1;
                }
                if lines_left > 0 as ::core::ffi::c_int {
                    lines_left -= 1;
                }
                if p_more != 0
                    && lines_left == 0 as ::core::ffi::c_int
                    && State != MODE_HITRETURN as ::core::ffi::c_int
                    && !msg_no_more
                    && !exmode_active
                {
                    if do_more_prompt(NUL) {
                        s = confirm_buttons;
                    }
                    if quit_more {
                        return;
                    }
                }
            }
        }
        if !((maxlen < 0 as ::core::ffi::c_int
            || (s.offset_from(str) as ::core::ffi::c_int) < maxlen)
            && *s as ::core::ffi::c_int != NUL)
        {
            break;
        }
        if msg_row != msg_row_pending
            && (*s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == TAB)
        {
            if msg_row_pending >= 0 as ::core::ffi::c_int {
                msg_line_flush();
            }
            grid_line_start(&raw mut msg_grid_adj, msg_row);
            msg_row_pending = msg_row;
        }
        if *s as uint8_t as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
            let mut cw: ::core::ffi::c_int = utf_ptr2cells(s);
            let mut l: ::core::ffi::c_int = if maxlen >= 0 as ::core::ffi::c_int {
                utfc_ptr2len_len(
                    s,
                    str.offset(maxlen as isize).offset_from(s) as ::core::ffi::c_int,
                )
            } else {
                utfc_ptr2len(s)
            };
            if cw > 1 as ::core::ffi::c_int && msg_col == Columns - 1 as ::core::ffi::c_int {
                grid_line_puts(
                    msg_col,
                    b">\0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    *hl_attr_active.offset(HLF_AT as ::core::ffi::c_int as isize),
                );
                cw = 1 as ::core::ffi::c_int;
            } else {
                grid_line_puts(msg_col, s, l, print_attr);
                s = s.offset(l as isize);
            }
            msg_didout = true_0 != 0;
            msg_col += cw;
        } else {
            let c2rust_fresh5 = s;
            s = s.offset(1);
            let mut c: ::core::ffi::c_char = *c2rust_fresh5;
            if c as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                msg_didout = false_0 != 0;
                msg_col = 0 as ::core::ffi::c_int;
                msg_row += 1;
                if p_more != 0 && recurse == 0 {
                    store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, true_0);
                }
            } else if c as ::core::ffi::c_int == '\r' as ::core::ffi::c_int {
                msg_col = 0 as ::core::ffi::c_int;
            } else if c as ::core::ffi::c_int == '\u{8}' as ::core::ffi::c_int {
                if msg_col != 0 {
                    msg_col -= 1;
                }
            } else if c as ::core::ffi::c_int == TAB {
                loop {
                    grid_line_puts(
                        msg_col,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        print_attr,
                    );
                    msg_col += 1 as ::core::ffi::c_int;
                    if msg_col == Columns {
                        break;
                    }
                    if msg_col & 7 as ::core::ffi::c_int == 0 {
                        break;
                    }
                }
            } else if c as ::core::ffi::c_int == BELL {
                vim_beep(kOptBoFlagShell as ::core::ffi::c_int as ::core::ffi::c_uint);
            }
        }
    }
    if msg_row_pending >= 0 as ::core::ffi::c_int {
        msg_line_flush();
    }
    msg_cursor_goto(msg_row, msg_col);
    if p_more != 0 && recurse == 0 {
        store_sb_text(&raw mut sb_str, s, hl_id, &raw mut sb_col, false_0);
    }
    msg_check();
}
#[no_mangle]
pub unsafe extern "C" fn msg_line_flush() {
    if cmdmsg_rl {
        grid_line_mirror(msg_grid.cols);
    }
    grid_line_flush_if_valid_row();
}
#[no_mangle]
pub unsafe extern "C" fn msg_cursor_goto(mut row: ::core::ffi::c_int, mut col: ::core::ffi::c_int) {
    if cmdmsg_rl {
        col = Columns - 1 as ::core::ffi::c_int - col;
    }
    let mut grid: *mut ScreenGrid = grid_adjust(&raw mut msg_grid_adj, &raw mut row, &raw mut col);
    ui_grid_cursor_goto((*grid).handle, row, col);
}
#[no_mangle]
pub unsafe extern "C" fn message_filtered(mut msg_0: *const ::core::ffi::c_char) -> bool {
    if cmdmod.cmod_filter_regmatch.regprog.is_null() {
        return false_0 != 0;
    }
    let mut match_0: bool = vim_regexec(&raw mut cmdmod.cmod_filter_regmatch, msg_0, 0 as colnr_T);
    return if cmdmod.cmod_filter_force as ::core::ffi::c_int != 0 {
        match_0 as ::core::ffi::c_int
    } else {
        !match_0 as ::core::ffi::c_int
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn msg_scrollsize() -> ::core::ffi::c_int {
    return msg_scrolled
        + p_ch as ::core::ffi::c_int
        + (if p_ch > 0 as OptInt || msg_scrolled > 1 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}
#[no_mangle]
pub unsafe extern "C" fn msg_do_throttle() -> bool {
    return msg_use_grid() as ::core::ffi::c_int != 0
        && rdb_flags & kOptRdbFlagNothrottle as ::core::ffi::c_int as ::core::ffi::c_uint == 0;
}
#[no_mangle]
pub unsafe extern "C" fn msg_scroll_up(mut may_throttle: bool, mut zerocmd: bool) {
    if may_throttle as ::core::ffi::c_int != 0 && msg_do_throttle() as ::core::ffi::c_int != 0 {
        msg_grid.throttled = true_0 != 0;
    }
    msg_did_scroll = true_0 != 0;
    if msg_grid_pos > 0 as ::core::ffi::c_int {
        msg_grid_set_pos(msg_grid_pos - 1 as ::core::ffi::c_int, !zerocmd);
        if zerocmd as ::core::ffi::c_int != 0 && !msg_grid.chars.is_null() {
            grid_clear_line(
                &raw mut msg_grid,
                *msg_grid
                    .line_offset
                    .offset(0 as ::core::ffi::c_int as isize),
                msg_grid.cols,
                false_0 != 0,
            );
        }
    } else {
        grid_del_lines(
            &raw mut msg_grid,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            msg_grid.rows,
            0 as ::core::ffi::c_int,
            msg_grid.cols,
        );
        memmove(
            msg_grid.dirty_col as *mut ::core::ffi::c_void,
            msg_grid.dirty_col.offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ((msg_grid.rows - 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        *msg_grid
            .dirty_col
            .offset((msg_grid.rows - 1 as ::core::ffi::c_int) as isize) = 0 as ::core::ffi::c_int;
    }
    grid_clear(
        &raw mut msg_grid_adj,
        Rows - 1 as ::core::ffi::c_int,
        Rows,
        0 as ::core::ffi::c_int,
        Columns,
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn msg_scroll_flush() {
    if msg_grid.throttled {
        msg_grid.throttled = false_0 != 0;
        let mut pos_delta: ::core::ffi::c_int = msg_grid_pos_at_flush - msg_grid_pos;
        '_c2rust_label: {
            if pos_delta >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"pos_delta >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2689 as ::core::ffi::c_uint,
                    b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut delta: ::core::ffi::c_int = if msg_scrolled - msg_scrolled_at_flush < msg_grid.rows
        {
            msg_scrolled - msg_scrolled_at_flush
        } else {
            msg_grid.rows
        };
        if pos_delta > 0 as ::core::ffi::c_int {
            ui_ext_msg_set_pos(msg_grid_pos, true_0 != 0);
        }
        let mut to_scroll: ::core::ffi::c_int = delta - pos_delta - msg_grid_scroll_discount;
        '_c2rust_label_0: {
            if to_scroll >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"to_scroll >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2697 as ::core::ffi::c_uint,
                    b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if to_scroll > 0 as ::core::ffi::c_int && msg_grid_pos == 0 as ::core::ffi::c_int {
            ui_call_grid_scroll(
                msg_grid.handle as Integer,
                0 as Integer,
                Rows as Integer,
                0 as Integer,
                Columns as Integer,
                to_scroll as Integer,
                0 as Integer,
            );
        }
        let mut i: ::core::ffi::c_int = if Rows
            - (if delta > 1 as ::core::ffi::c_int {
                delta
            } else {
                1 as ::core::ffi::c_int
            })
            > 0 as ::core::ffi::c_int
        {
            Rows - (if delta > 1 as ::core::ffi::c_int {
                delta
            } else {
                1 as ::core::ffi::c_int
            })
        } else {
            0 as ::core::ffi::c_int
        };
        while i < Rows {
            let mut row: ::core::ffi::c_int = i - msg_grid_pos;
            '_c2rust_label_1: {
                if row >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2707 as ::core::ffi::c_uint,
                        b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            ui_line(
                &raw mut msg_grid,
                row,
                false_0 != 0,
                0 as ::core::ffi::c_int,
                *msg_grid.dirty_col.offset(row as isize),
                msg_grid.cols,
                *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
                false_0 != 0,
            );
            *msg_grid.dirty_col.offset(row as isize) = 0 as ::core::ffi::c_int;
            i += 1;
        }
    }
    msg_scrolled_at_flush = msg_scrolled;
    msg_grid_scroll_discount = 0 as ::core::ffi::c_int;
    msg_grid_pos_at_flush = msg_grid_pos;
}
#[no_mangle]
pub unsafe extern "C" fn msg_reset_scroll() {
    if ui_has(kUIMessages) {
        return;
    }
    msg_grid.throttled = false_0 != 0;
    msg_grid_set_pos(Rows - p_ch as ::core::ffi::c_int, false_0 != 0);
    clear_cmdline = true_0 != 0;
    if !msg_grid.chars.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i
            < (if msg_scrollsize() < msg_grid.rows {
                msg_scrollsize()
            } else {
                msg_grid.rows
            })
        {
            grid_clear_line(
                &raw mut msg_grid,
                *msg_grid.line_offset.offset(i as isize),
                msg_grid.cols,
                false_0 != 0,
            );
            i += 1;
        }
    }
    msg_scrolled = 0 as ::core::ffi::c_int;
    msg_scrolled_at_flush = 0 as ::core::ffi::c_int;
    msg_grid_scroll_discount = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn msg_ui_refresh() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && !msg_grid.chars.is_null() {
        ui_call_grid_resize(
            msg_grid.handle as Integer,
            msg_grid.cols as Integer,
            msg_grid.rows as Integer,
        );
        ui_ext_msg_set_pos(msg_grid_pos, msg_scrolled != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_ui_flush() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        && !msg_grid.chars.is_null()
        && msg_grid.pending_comp_index_update as ::core::ffi::c_int != 0
    {
        ui_ext_msg_set_pos(msg_grid_pos, msg_scrolled != 0);
    }
}
unsafe extern "C" fn inc_msg_scrolled() {
    if *get_vim_var_str(VV_SCROLLSTART) as ::core::ffi::c_int == NUL {
        let mut p: String_0 = String_0 {
            data: (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
            size: 0,
        };
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if p.data.is_null() {
            p = cstr_as_string(gettext(b"Unknown\0".as_ptr() as *const ::core::ffi::c_char));
        } else {
            let mut tofreesize: size_t = strlen(p.data).wrapping_add(40 as size_t);
            tofree = xmalloc(tofreesize) as *mut ::core::ffi::c_char;
            p.size = vim_snprintf_safelen(
                tofree,
                tofreesize,
                gettext(b"%s line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                p.data,
                (*(exestack.ga_data as *mut estack_T)
                    .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum as int64_t,
            );
            p.data = tofree;
        }
        set_vim_var_string(VV_SCROLLSTART, p.data, p.size as ptrdiff_t);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    msg_scrolled += 1;
    set_must_redraw(UPD_VALID as ::core::ffi::c_int);
}
static mut last_msgchunk: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
static mut do_clear_sb_text: sb_clear_T = SB_CLEAR_NONE;
unsafe extern "C" fn store_sb_text(
    mut sb_str: *mut *const ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut sb_col: *mut ::core::ffi::c_int,
    mut finish: ::core::ffi::c_int,
) {
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    if do_clear_sb_text as ::core::ffi::c_uint
        == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
        || do_clear_sb_text as ::core::ffi::c_uint
            == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        clear_sb_text(
            do_clear_sb_text as ::core::ffi::c_uint
                == SB_CLEAR_ALL as ::core::ffi::c_int as ::core::ffi::c_uint,
        );
        msg_sb_eol();
        if do_clear_sb_text as ::core::ffi::c_uint
            == SB_CLEAR_CMDLINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && s > *sb_str
            && **sb_str as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            *sb_str = (*sb_str).offset(1);
        }
        do_clear_sb_text = SB_CLEAR_NONE;
    }
    if s > *sb_str {
        mp = xmalloc(
            (28 as size_t)
                .wrapping_add(s.offset_from(*sb_str) as size_t)
                .wrapping_add(1 as size_t),
        ) as *mut msgchunk_T;
        (*mp).sb_eol = finish as ::core::ffi::c_char;
        (*mp).sb_msg_col = *sb_col;
        (*mp).sb_hl_id = hl_id;
        memcpy(
            &raw mut (*mp).sb_text as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            *sb_str as *const ::core::ffi::c_void,
            s.offset_from(*sb_str) as size_t,
        );
        *(&raw mut (*mp).sb_text as *mut ::core::ffi::c_char)
            .offset(s.offset_from(*sb_str) as isize) = NUL as ::core::ffi::c_char;
        if last_msgchunk.is_null() {
            last_msgchunk = mp;
            (*mp).sb_prev = ::core::ptr::null_mut::<msgchunk_T>();
        } else {
            (*mp).sb_prev = last_msgchunk;
            (*last_msgchunk).sb_next = mp;
            last_msgchunk = mp;
        }
        (*mp).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
    } else if finish != 0 && !last_msgchunk.is_null() {
        (*last_msgchunk).sb_eol = true_0 as ::core::ffi::c_char;
    }
    *sb_str = s;
    *sb_col = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn may_clear_sb_text() {
    msg_ext_ui_flush();
    do_clear_sb_text = SB_CLEAR_ALL;
    do_clear_hist_temp = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn sb_text_start_cmdline() {
    if do_clear_sb_text as ::core::ffi::c_uint
        == SB_CLEAR_CMDLINE_BUSY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        sb_text_restart_cmdline();
    } else {
        msg_sb_eol();
        do_clear_sb_text = SB_CLEAR_CMDLINE_BUSY;
    };
}
#[no_mangle]
pub unsafe extern "C" fn sb_text_restart_cmdline() {
    do_clear_sb_text = SB_CLEAR_CMDLINE_BUSY;
    if last_msgchunk.is_null() || (*last_msgchunk).sb_eol as ::core::ffi::c_int != 0 {
        return;
    }
    let mut tofree: *mut msgchunk_T = msg_sb_start(last_msgchunk);
    last_msgchunk = (*tofree).sb_prev;
    if !last_msgchunk.is_null() {
        (*last_msgchunk).sb_next = ::core::ptr::null_mut::<msgchunk_T>();
    }
    while !tofree.is_null() {
        let mut tofree_next: *mut msgchunk_T = (*tofree).sb_next;
        xfree(tofree as *mut ::core::ffi::c_void);
        tofree = tofree_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn sb_text_end_cmdline() {
    do_clear_sb_text = SB_CLEAR_CMDLINE_DONE;
}
#[no_mangle]
pub unsafe extern "C" fn clear_sb_text(mut all: bool) {
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut lastp: *mut *mut msgchunk_T = ::core::ptr::null_mut::<*mut msgchunk_T>();
    if all {
        lastp = &raw mut last_msgchunk;
    } else {
        if last_msgchunk.is_null() {
            return;
        }
        lastp = &raw mut (*(msg_sb_start
            as unsafe extern "C" fn(*mut msgchunk_T) -> *mut msgchunk_T)(
            last_msgchunk
        ))
        .sb_prev;
    }
    while !(*lastp).is_null() {
        mp = (**lastp).sb_prev;
        xfree(*lastp as *mut ::core::ffi::c_void);
        *lastp = mp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn show_sb_text() {
    if ui_has(kUIMessages) {
        let mut ea: exarg_T = exarg {
            arg: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: true_0,
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
        ex_messages(&raw mut ea);
        return;
    }
    let mut mp: *mut msgchunk_T = msg_sb_start(last_msgchunk);
    if mp.is_null() || (*mp).sb_prev.is_null() {
        vim_beep(kOptBoFlagMess as ::core::ffi::c_int as ::core::ffi::c_uint);
    } else {
        do_more_prompt('G' as ::core::ffi::c_int);
        wait_return(false_0);
    };
}
unsafe extern "C" fn msg_sb_start(mut mps: *mut msgchunk_T) -> *mut msgchunk_T {
    let mut mp: *mut msgchunk_T = mps;
    while !mp.is_null() && !(*mp).sb_prev.is_null() && (*(*mp).sb_prev).sb_eol == 0 {
        mp = (*mp).sb_prev;
    }
    return mp;
}
#[no_mangle]
pub unsafe extern "C" fn msg_sb_eol() {
    if !last_msgchunk.is_null() {
        (*last_msgchunk).sb_eol = true_0 as ::core::ffi::c_char;
    }
}
unsafe extern "C" fn disp_sb_line(
    mut row: ::core::ffi::c_int,
    mut smp: *mut msgchunk_T,
) -> *mut msgchunk_T {
    let mut mp: *mut msgchunk_T = smp;
    loop {
        msg_row = row;
        msg_col = (*mp).sb_msg_col;
        let mut p: *mut ::core::ffi::c_char = &raw mut (*mp).sb_text as *mut ::core::ffi::c_char;
        msg_puts_display(p, -1 as ::core::ffi::c_int, (*mp).sb_hl_id, true_0);
        if (*mp).sb_eol as ::core::ffi::c_int != 0 || (*mp).sb_next.is_null() {
            break;
        }
        mp = (*mp).sb_next;
    }
    return (*mp).sb_next;
}
#[no_mangle]
pub unsafe extern "C" fn msg_use_printf() -> ::core::ffi::c_int {
    return (!embedded_mode && ui_active() == 0 && !ui_has(kUIMessages)) as ::core::ffi::c_int;
}
unsafe extern "C" fn msg_puts_printf(mut str: *const ::core::ffi::c_char, maxlen: ptrdiff_t) {
    let mut s: *const ::core::ffi::c_char = str;
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if on_print.type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut argv: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[0 as ::core::ffi::c_int as usize].vval.v_string = str as *mut ::core::ffi::c_char;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        callback_call(
            &raw mut on_print,
            1 as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear(&raw mut rettv);
        return;
    }
    while (maxlen < 0 as ptrdiff_t || s.offset_from(str) < maxlen)
        && *s as ::core::ffi::c_int != NUL
    {
        let mut len: ::core::ffi::c_int = utf_ptr2len(s);
        if !(silent_mode as ::core::ffi::c_int != 0 && p_verbose == 0 as OptInt) {
            p = (&raw mut buf as *mut ::core::ffi::c_char).offset(0 as ::core::ffi::c_int as isize);
            if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                && !info_message
                && !silent_mode
                && !headless_mode
            {
                let c2rust_fresh6 = p;
                p = p.offset(1);
                *c2rust_fresh6 = '\r' as ::core::ffi::c_char;
            }
            memcpy(
                p as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                len as size_t,
            );
            *p.offset(len as isize) = NUL as ::core::ffi::c_char;
            if info_message {
                printf(
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            } else {
                fprintf(
                    stderr,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
            }
        }
        let mut cw: ::core::ffi::c_int = utf_char2cells(utf_ptr2char(s));
        if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
            || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
        {
            msg_col = 0 as ::core::ffi::c_int;
            msg_didout = false_0 != 0;
        } else {
            msg_col += cw;
            msg_didout = true_0 != 0;
        }
        s = s.offset(len as isize);
    }
}
unsafe extern "C" fn do_more_prompt(mut typed_char: ::core::ffi::c_int) -> bool {
    static mut entered: bool = false_0 != 0;
    let mut used_typed_char: ::core::ffi::c_int = typed_char;
    let mut oldState: ::core::ffi::c_int = State;
    let mut c: ::core::ffi::c_int = 0;
    let mut retval: bool = false_0 != 0;
    let mut to_redraw: bool = false_0 != 0;
    let mut mp_last: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut mp: *mut msgchunk_T = ::core::ptr::null_mut::<msgchunk_T>();
    let mut no_need_more: bool =
        headless_mode as ::core::ffi::c_int != 0 && !embedded_mode && ui_active() == 0;
    if no_need_more as ::core::ffi::c_int != 0
        || entered as ::core::ffi::c_int != 0
        || State == MODE_HITRETURN as ::core::ffi::c_int && typed_char == 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    entered = true_0 != 0;
    if typed_char == 'G' as ::core::ffi::c_int {
        mp_last = msg_sb_start(last_msgchunk);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < Rows - 2 as ::core::ffi::c_int
            && !mp_last.is_null()
            && !(*mp_last).sb_prev.is_null()
        {
            mp_last = msg_sb_start((*mp_last).sb_prev);
            i += 1;
        }
    }
    State = MODE_ASKMORE as ::core::ffi::c_int;
    setmouse();
    if typed_char == NUL {
        msg_moremsg(false_0 != 0);
    }
    's_528: loop {
        if used_typed_char != NUL {
            c = used_typed_char;
            used_typed_char = NUL;
        } else {
            c = get_keystroke(resize_events);
        }
        let mut toscroll: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        's_276: {
            match c {
                BS | K_BS | 107 | K_UP => {
                    toscroll = -1 as ::core::ffi::c_int;
                    break 's_276;
                }
                CAR | NL | 106 | K_DOWN => {
                    toscroll = 1 as ::core::ffi::c_int;
                    break 's_276;
                }
                117 => {
                    toscroll = -(Rows / 2 as ::core::ffi::c_int);
                    break 's_276;
                }
                100 => {
                    toscroll = Rows / 2 as ::core::ffi::c_int;
                    break 's_276;
                }
                98 | Ctrl_B | K_PAGEUP => {
                    toscroll = -(Rows - 1 as ::core::ffi::c_int);
                    break 's_276;
                }
                32 | 102 | Ctrl_F | K_PAGEDOWN | -11517 => {
                    toscroll = Rows - 1 as ::core::ffi::c_int;
                    break 's_276;
                }
                103 => {
                    toscroll = -999999 as ::core::ffi::c_int;
                    break 's_276;
                }
                71 => {
                    toscroll = 999999 as ::core::ffi::c_int;
                    lines_left = 999999 as ::core::ffi::c_int;
                    break 's_276;
                }
                58 => {
                    if confirm_msg_used == 0 {
                        typeahead_noflush(':' as ::core::ffi::c_int);
                        cmdline_row = Rows - 1 as ::core::ffi::c_int;
                        skip_redraw = true_0 != 0;
                        need_wait_return = false_0 != 0;
                    }
                }
                113 | Ctrl_C | ESC => {}
                K_EVENT => {
                    multiqueue_process_events(resize_events);
                    to_redraw = true_0 != 0;
                    break 's_276;
                }
                _ => {
                    msg_moremsg(true_0 != 0);
                    continue 's_528;
                }
            }
            if confirm_msg_used != 0 {
                retval = true_0 != 0;
            } else {
                got_int = true_0 != 0;
                quit_more = true_0 != 0;
            }
            lines_left = Rows - 1 as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if toscroll == 0 as ::core::ffi::c_int || !to_redraw {
            } else {
                __assert_fail(
                    b"(toscroll == 0) || !to_redraw\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3168 as ::core::ffi::c_uint,
                    b"_Bool do_more_prompt(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if !(toscroll != 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0) {
            break;
        }
        if toscroll < 0 as ::core::ffi::c_int || to_redraw as ::core::ffi::c_int != 0 {
            if mp_last.is_null() {
                mp = msg_sb_start(last_msgchunk);
            } else if !(*mp_last).sb_prev.is_null() {
                mp = msg_sb_start((*mp_last).sb_prev);
            } else {
                mp = ::core::ptr::null_mut::<msgchunk_T>();
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < Rows - 2 as ::core::ffi::c_int && !mp.is_null() && !(*mp).sb_prev.is_null()
            {
                mp = msg_sb_start((*mp).sb_prev);
                i_0 += 1;
            }
            if !mp.is_null() && (!(*mp).sb_prev.is_null() || to_redraw as ::core::ffi::c_int != 0) {
                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_1 > toscroll {
                    if mp.is_null() || (*mp).sb_prev.is_null() {
                        break;
                    }
                    mp = msg_sb_start((*mp).sb_prev);
                    if mp_last.is_null() {
                        mp_last = msg_sb_start(last_msgchunk);
                    } else {
                        mp_last = msg_sb_start((*mp_last).sb_prev);
                    }
                    i_1 -= 1;
                }
                if toscroll == -1 as ::core::ffi::c_int && !to_redraw {
                    grid_ins_lines(
                        &raw mut msg_grid,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        Rows,
                        0 as ::core::ffi::c_int,
                        Columns,
                    );
                    grid_clear(
                        &raw mut msg_grid_adj,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        Columns,
                        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
                    );
                    disp_sb_line(0 as ::core::ffi::c_int, mp);
                } else {
                    grid_clear(
                        &raw mut msg_grid_adj,
                        0 as ::core::ffi::c_int,
                        Rows,
                        0 as ::core::ffi::c_int,
                        Columns,
                        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
                    );
                    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while !mp.is_null() && i_2 < Rows - 1 as ::core::ffi::c_int {
                        mp = disp_sb_line(i_2, mp);
                        msg_scrolled += 1;
                        i_2 += 1;
                    }
                    to_redraw = false_0 != 0;
                }
                toscroll = 0 as ::core::ffi::c_int;
            }
        } else {
            if cmdline_row >= Rows && !ui_has(kUIMessages) {
                msg_scroll_up(true_0 != 0, false_0 != 0);
                msg_scrolled += 1;
            }
            while toscroll > 0 as ::core::ffi::c_int && !mp_last.is_null() {
                if msg_do_throttle() as ::core::ffi::c_int != 0 && !msg_grid.throttled {
                    msg_scrolled_at_flush -= 1;
                    msg_grid_scroll_discount += 1;
                }
                msg_scroll_up(true_0 != 0, false_0 != 0);
                inc_msg_scrolled();
                grid_clear(
                    &raw mut msg_grid_adj,
                    Rows - 2 as ::core::ffi::c_int,
                    Rows - 1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    Columns,
                    *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
                );
                mp_last = disp_sb_line(Rows - 2 as ::core::ffi::c_int, mp_last);
                toscroll -= 1;
            }
        }
        if toscroll <= 0 as ::core::ffi::c_int {
            grid_clear(
                &raw mut msg_grid_adj,
                Rows - 1 as ::core::ffi::c_int,
                Rows,
                0 as ::core::ffi::c_int,
                Columns,
                *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
            );
            msg_moremsg(false_0 != 0);
        } else {
            lines_left = toscroll;
            break;
        }
    }
    grid_clear(
        &raw mut msg_grid_adj,
        Rows - 1 as ::core::ffi::c_int,
        Rows,
        0 as ::core::ffi::c_int,
        Columns,
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
    );
    redraw_cmdline = true_0 != 0;
    clear_cmdline = false_0 != 0;
    mode_displayed = false_0 != 0;
    State = oldState;
    setmouse();
    if quit_more {
        msg_row = Rows - 1 as ::core::ffi::c_int;
        msg_col = 0 as ::core::ffi::c_int;
    }
    entered = false_0 != 0;
    return retval;
}
unsafe extern "C" fn msg_moremsg(mut full: bool) {
    let mut attr: ::core::ffi::c_int = hl_combine_attr(
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
        *hl_attr_active.offset(HLF_M as ::core::ffi::c_int as isize),
    );
    grid_line_start(&raw mut msg_grid_adj, Rows - 1 as ::core::ffi::c_int);
    let mut len: ::core::ffi::c_int = grid_line_puts(
        0 as ::core::ffi::c_int,
        gettext(b"-- More --\0".as_ptr() as *const ::core::ffi::c_char),
        -1 as ::core::ffi::c_int,
        attr,
    );
    if full {
        len += grid_line_puts(
            len,
            gettext(
                b" SPACE/d/j: screen/page/line down, b/u/k: up, q: quit \0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            -1 as ::core::ffi::c_int,
            attr,
        );
    }
    grid_line_cursor_goto(len);
    grid_line_flush();
}
#[no_mangle]
pub unsafe extern "C" fn repeat_message() {
    if ui_has(kUIMessages) {
        return;
    }
    if State == MODE_ASKMORE as ::core::ffi::c_int {
        msg_moremsg(true_0 != 0);
        msg_row = Rows - 1 as ::core::ffi::c_int;
    } else if State & MODE_CMDLINE as ::core::ffi::c_int != 0 && !confirm_msg.is_null() {
        display_confirm_msg();
        msg_row = Rows - 1 as ::core::ffi::c_int;
    } else if State == MODE_EXTERNCMD as ::core::ffi::c_int {
        ui_cursor_goto(msg_row, msg_col);
    } else if State == MODE_HITRETURN as ::core::ffi::c_int
        || State == MODE_SETWSIZE as ::core::ffi::c_int
    {
        if msg_row == Rows - 1 as ::core::ffi::c_int {
            msg_didout = false_0 != 0;
            msg_col = 0 as ::core::ffi::c_int;
            msg_clr_eos();
        }
        hit_return_msg(false_0 != 0);
        msg_row = Rows - 1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_clr_eos() {
    if msg_silent == 0 as ::core::ffi::c_int {
        msg_clr_eos_force();
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_clr_eos_force() {
    if ui_has(kUIMessages) {
        return;
    }
    let mut msg_startcol: ::core::ffi::c_int = if cmdmsg_rl as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        msg_col
    };
    let mut msg_endcol: ::core::ffi::c_int = if cmdmsg_rl as ::core::ffi::c_int != 0 {
        Columns - msg_col
    } else {
        Columns
    };
    if !msg_grid.chars.is_null() && msg_row < msg_grid_pos {
        msg_grid_validate();
        if msg_row < msg_grid_pos {
            msg_row = msg_grid_pos;
        }
    }
    grid_clear(
        &raw mut msg_grid_adj,
        msg_row,
        msg_row + 1 as ::core::ffi::c_int,
        msg_startcol,
        msg_endcol,
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
    );
    grid_clear(
        &raw mut msg_grid_adj,
        msg_row + 1 as ::core::ffi::c_int,
        Rows,
        0 as ::core::ffi::c_int,
        Columns,
        *hl_attr_active.offset(HLF_MSG as ::core::ffi::c_int as isize),
    );
    redraw_cmdline = true_0 != 0;
    if msg_row < Rows - 1 as ::core::ffi::c_int || msg_col == 0 as ::core::ffi::c_int {
        clear_cmdline = false_0 != 0;
        mode_displayed = false_0 != 0;
        cmdline_was_last_drawn = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_clr_cmdline() {
    msg_row = cmdline_row;
    msg_col = 0 as ::core::ffi::c_int;
    msg_clr_eos_force();
}
#[no_mangle]
pub unsafe extern "C" fn msg_end() -> bool {
    if !exiting
        && need_wait_return as ::core::ffi::c_int != 0
        && State & MODE_CMDLINE as ::core::ffi::c_int == 0
    {
        wait_return(false_0);
        return false_0 != 0;
    }
    msg_ext_ui_flush();
    return true_0 != 0;
}
unsafe extern "C" fn msg_ext_init_chunks() -> *mut Array {
    let mut tofree: *mut Array = msg_ext_chunks;
    msg_ext_chunks = xcalloc(1 as size_t, ::core::mem::size_of::<Array>()) as *mut Array;
    msg_col = 0 as ::core::ffi::c_int;
    return tofree;
}
#[no_mangle]
pub unsafe extern "C" fn msg_ext_ui_flush() {
    if !ui_has(kUIMessages) {
        msg_ext_kind = ::core::ptr::null::<::core::ffi::c_char>();
        return;
    } else if msg_ext_skip_flush {
        return;
    }
    msg_ext_emit_chunk();
    if (*msg_ext_chunks).size > 0 as size_t {
        let mut tofree: *mut Array = msg_ext_init_chunks();
        ui_call_msg_show(
            cstr_as_string(msg_ext_kind),
            *tofree,
            msg_ext_overwrite as Boolean,
            msg_ext_history as Boolean,
            msg_ext_append as Boolean,
            msg_ext_id,
            cstr_as_string(msg_ext_trigger),
        );
        if msg_ext_history {
            api_free_array(*tofree);
        } else {
            let mut msg_0: HlMessage = HlMessage {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<HlMessageChunk>(),
            };
            let mut i: size_t = 0 as size_t;
            while i < (*tofree).size {
                let mut chunk: *mut Object = (*(*tofree).items.offset(i as isize)).data.array.items;
                if msg_0.size == msg_0.capacity {
                    msg_0.capacity = if msg_0.capacity != 0 {
                        msg_0.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    msg_0.items = xrealloc(
                        msg_0.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
                    ) as *mut HlMessageChunk;
                } else {
                };
                let c2rust_fresh0 = msg_0.size;
                msg_0.size = msg_0.size.wrapping_add(1);
                *msg_0.items.offset(c2rust_fresh0 as isize) = HlMessageChunk {
                    text: (*chunk.offset(1 as ::core::ffi::c_int as isize))
                        .data
                        .string,
                    hl_id: (*chunk.offset(2 as ::core::ffi::c_int as isize))
                        .data
                        .integer as ::core::ffi::c_int,
                };
                xfree(chunk as *mut ::core::ffi::c_void);
                i = i.wrapping_add(1);
            }
            xfree((*tofree).items as *mut ::core::ffi::c_void);
            msg_hist_add_multihl(msg_0, true_0 != 0, ::core::ptr::null_mut::<MessageData>());
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        msg_ext_overwrite = false_0 != 0;
        msg_ext_history = false_0 != 0;
        msg_ext_append = false_0 != 0;
        msg_ext_kind = ::core::ptr::null::<::core::ffi::c_char>();
        msg_id_next += (msg_ext_id.data.integer == msg_id_next) as ::core::ffi::c_int as int64_t;
        msg_ext_id = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_11 {
                integer: msg_id_next,
            },
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_ext_flush_showmode() {
    static mut clear: bool = false_0 != 0;
    if ui_has(kUIMessages) as ::core::ffi::c_int != 0
        && (msg_ext_last_attr != -1 as sattr_T || clear as ::core::ffi::c_int != 0)
    {
        clear = msg_ext_last_attr != -1 as sattr_T;
        msg_ext_emit_chunk();
        let mut tofree: *mut Array = msg_ext_init_chunks();
        ui_call_msg_showmode(*tofree);
        api_free_array(*tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn msg_check() {
    if ui_has(kUIMessages) {
        return;
    }
    if msg_row == Rows - 1 as ::core::ffi::c_int && msg_col >= sc_col {
        need_wait_return = true_0 != 0;
        redraw_cmdline = true_0 != 0;
    }
}
unsafe extern "C" fn redir_write(str: *const ::core::ffi::c_char, maxlen: ptrdiff_t) {
    let mut s: *const ::core::ffi::c_char = str;
    if maxlen == 0 as ptrdiff_t {
        return;
    }
    if redir_off {
        return;
    }
    if *p_vfile as ::core::ffi::c_int != NUL && verbose_fd.is_null() {
        verbose_open();
    }
    if redirecting() != 0 {
        if *s as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
        {
            while redir_col < msg_col {
                if !capture_ga.is_null() {
                    ga_concat_len(
                        capture_ga,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as size_t,
                    );
                }
                if redir_reg != 0 {
                    write_reg_contents(
                        redir_reg,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ssize_t,
                        true_0,
                    );
                } else if redir_vname {
                    var_redir_str(
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        -1 as ::core::ffi::c_int,
                    );
                } else if !redir_fd.is_null() {
                    fputs(b" \0".as_ptr() as *const ::core::ffi::c_char, redir_fd);
                }
                if !verbose_fd.is_null() {
                    fputs(b" \0".as_ptr() as *const ::core::ffi::c_char, verbose_fd);
                }
                redir_col += 1;
            }
        }
        let mut len: size_t = if maxlen == -1 as ptrdiff_t {
            strlen(s)
        } else {
            maxlen as size_t
        };
        if !capture_ga.is_null() {
            ga_concat_len(capture_ga, str, len);
        }
        if redir_reg != 0 {
            write_reg_contents(redir_reg, s, len as ssize_t, true_0);
        }
        if redir_vname {
            var_redir_str(s, maxlen as ::core::ffi::c_int);
        }
        while *s as ::core::ffi::c_int != NUL
            && (maxlen < 0 as ptrdiff_t
                || (s.offset_from(str) as ::core::ffi::c_int as ptrdiff_t) < maxlen)
        {
            if redir_reg == 0 && !redir_vname && capture_ga.is_null() {
                if !redir_fd.is_null() {
                    putc(*s as ::core::ffi::c_int, redir_fd);
                }
            }
            if !verbose_fd.is_null() {
                putc(*s as ::core::ffi::c_int, verbose_fd);
            }
            if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                redir_col = 0 as ::core::ffi::c_int;
            } else if *s as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
                redir_col += 8 as ::core::ffi::c_int - redir_col % 8 as ::core::ffi::c_int;
            } else {
                redir_col += 1;
            }
            s = s.offset(1);
        }
        if msg_silent != 0 as ::core::ffi::c_int {
            msg_col = redir_col;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn redirecting() -> ::core::ffi::c_int {
    return (!redir_fd.is_null()
        || *p_vfile as ::core::ffi::c_int != NUL
        || redir_reg != 0
        || redir_vname as ::core::ffi::c_int != 0
        || !capture_ga.is_null()) as ::core::ffi::c_int;
}
static mut pre_verbose_kind: *const ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>();
static mut verbose_kind: *const ::core::ffi::c_char =
    b"verbose\0".as_ptr() as *const ::core::ffi::c_char;
#[no_mangle]
pub unsafe extern "C" fn verbose_enter() {
    if *p_vfile as ::core::ffi::c_int != NUL {
        msg_silent += 1;
    }
    if !msg_ext_skip_verbose {
        if msg_ext_kind != verbose_kind {
            pre_verbose_kind = msg_ext_kind;
        }
        msg_ext_set_kind(b"verbose\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_ext_skip_verbose = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn verbose_leave() {
    if *p_vfile as ::core::ffi::c_int != NUL {
        msg_silent -= 1;
        if msg_silent < 0 as ::core::ffi::c_int {
            msg_silent = 0 as ::core::ffi::c_int;
        }
    }
    if !pre_verbose_kind.is_null() {
        msg_ext_set_kind(pre_verbose_kind);
        pre_verbose_kind = ::core::ptr::null::<::core::ffi::c_char>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn verbose_enter_scroll() {
    verbose_enter();
    if *p_vfile as ::core::ffi::c_int == NUL {
        msg_scroll = true_0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn verbose_leave_scroll() {
    verbose_leave();
    if *p_vfile as ::core::ffi::c_int == NUL {
        cmdline_row = msg_row;
    }
}
#[no_mangle]
pub unsafe extern "C" fn verbose_stop() {
    if !verbose_fd.is_null() {
        fclose(verbose_fd);
        verbose_fd = ::core::ptr::null_mut::<FILE>();
    }
    verbose_did_open = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn verbose_open() -> ::core::ffi::c_int {
    if verbose_fd.is_null() && !verbose_did_open {
        verbose_did_open = true_0 != 0;
        verbose_fd = os_fopen(p_vfile, b"a\0".as_ptr() as *const ::core::ffi::c_char);
        if verbose_fd.is_null() {
            semsg(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                p_vfile,
            );
            return FAIL;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn give_warning(
    mut message: *const ::core::ffi::c_char,
    mut hl: bool,
    mut hist: bool,
) {
    if msg_silent != 0 as ::core::ffi::c_int {
        return;
    }
    let mut save_msg_hist_off: bool = msg_hist_off;
    msg_hist_off = !hist;
    no_wait_return += 1;
    set_vim_var_string(VV_WARNINGMSG, message, -1 as ptrdiff_t);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut keep_msg as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    if hl {
        keep_msg_hl_id = HLF_W as ::core::ffi::c_int;
    } else {
        keep_msg_hl_id = 0 as ::core::ffi::c_int;
    }
    if msg_ext_kind.is_null() {
        msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if msg(message, keep_msg_hl_id) as ::core::ffi::c_int != 0
        && msg_scrolled == 0 as ::core::ffi::c_int
    {
        set_keep_msg(message, keep_msg_hl_id);
    }
    msg_didout = false_0 != 0;
    msg_nowait = true_0 != 0;
    msg_col = 0 as ::core::ffi::c_int;
    no_wait_return -= 1;
    msg_hist_off = save_msg_hist_off;
}
#[no_mangle]
pub unsafe extern "C" fn swmsg(
    mut hl: bool,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    let mut args: ::core::ffi::VaListImpl;
    args = c2rust_args.clone();
    vim_vsnprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        fmt,
        args.as_va_list(),
    );
    give_warning(&raw mut IObuff as *mut ::core::ffi::c_char, hl, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn msg_advance(mut col: ::core::ffi::c_int) {
    if msg_silent != 0 as ::core::ffi::c_int {
        msg_col = col;
        return;
    }
    col = if col < Columns - 1 as ::core::ffi::c_int {
        col
    } else {
        Columns - 1 as ::core::ffi::c_int
    };
    while msg_col < col {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_dialog(
    mut _type_0: ::core::ffi::c_int,
    mut _title: *const ::core::ffi::c_char,
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
    mut _textfield: *const ::core::ffi::c_char,
    mut ex_cmd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    if silent_mode {
        return dfltbutton;
    }
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent;
    let mut oldState: ::core::ffi::c_int = State;
    msg_silent = 0 as ::core::ffi::c_int;
    no_wait_return += 1;
    let mut hotkeys: *mut ::core::ffi::c_char =
        msg_show_console_dialog(message, buttons, dfltbutton);
    loop {
        if ui_active() == 0 && input_available() == 0 {
            retval = dfltbutton;
            break;
        } else {
            let mut c: ::core::ffi::c_int = prompt_for_input(
                confirm_buttons,
                HLF_M as ::core::ffi::c_int,
                true_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            match c {
                CAR | NUL => {
                    retval = dfltbutton;
                    break;
                }
                Ctrl_C | ESC => {
                    retval = 0 as ::core::ffi::c_int;
                    break;
                }
                _ => {
                    if c < 0 as ::core::ffi::c_int {
                        msg_didany = false_0 != 0;
                        msg_didout = msg_didany;
                    } else if c == ':' as ::core::ffi::c_int && ex_cmd != 0 {
                        retval = dfltbutton;
                        ins_char_typebuf(
                            ':' as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                        break;
                    } else {
                        c = mb_tolower(c);
                        retval = 1 as ::core::ffi::c_int;
                        i = 0 as ::core::ffi::c_int;
                        while *hotkeys.offset(i as isize) != 0 {
                            if utf_ptr2char(hotkeys.offset(i as isize)) == c {
                                break;
                            }
                            i += utfc_ptr2len(hotkeys.offset(i as isize)) - 1 as ::core::ffi::c_int;
                            retval += 1;
                            i += 1;
                        }
                        if *hotkeys.offset(i as isize) != 0 {
                            break;
                        }
                        msg_didany = false_0 != 0;
                        msg_didout = msg_didany;
                    }
                }
            }
        }
    }
    xfree(hotkeys as *mut ::core::ffi::c_void);
    xfree(confirm_msg as *mut ::core::ffi::c_void);
    confirm_msg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    msg_silent = save_msg_silent;
    State = oldState;
    setmouse();
    no_wait_return -= 1;
    msg_end_prompt();
    return retval;
}
unsafe extern "C" fn copy_char(
    mut from: *const ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
    mut lowercase: bool,
) -> ::core::ffi::c_int {
    if lowercase {
        let mut c: ::core::ffi::c_int = mb_tolower(utf_ptr2char(from));
        return utf_char2bytes(c, to);
    }
    let mut len: ::core::ffi::c_int = utfc_ptr2len(from);
    memmove(
        to as *mut ::core::ffi::c_void,
        from as *const ::core::ffi::c_void,
        len as size_t,
    );
    return len;
}
pub const HAS_HOTKEY_LEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
unsafe extern "C" fn console_dialog_alloc(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut has_hotkey: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut lenhotkey: ::core::ffi::c_int = MB_MAXBYTES as ::core::ffi::c_int;
    *has_hotkey.offset(0 as ::core::ffi::c_int as isize) = false_0 != 0;
    let mut msg_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut button_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *const ::core::ffi::c_char = buttons;
    while *r != 0 {
        if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
            button_len += 3 as ::core::ffi::c_int;
            lenhotkey += MB_MAXBYTES as ::core::ffi::c_int;
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                idx += 1;
                *has_hotkey.offset(idx as isize) = false_0 != 0;
            }
        } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
            r = r.offset(1);
            button_len += 1;
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                *has_hotkey.offset(idx as isize) = true_0 != 0;
            }
        }
        r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
    }
    msg_len += strlen(message) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
    button_len += strlen(buttons) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
    lenhotkey += 1;
    if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
        button_len += 2 as ::core::ffi::c_int;
    }
    confirm_msg = xmalloc(msg_len as size_t) as *mut ::core::ffi::c_char;
    snprintf(
        confirm_msg,
        msg_len as size_t,
        if ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            b"%s\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\n%s\n\0".as_ptr() as *const ::core::ffi::c_char
        },
        message,
    );
    xfree(confirm_buttons as *mut ::core::ffi::c_void);
    confirm_buttons = xmalloc(button_len as size_t) as *mut ::core::ffi::c_char;
    return xmalloc(lenhotkey as size_t) as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn msg_show_console_dialog(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut has_hotkey: [bool; 30] = [
        false_0 != 0,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    ];
    let mut hotk: *mut ::core::ffi::c_char =
        console_dialog_alloc(message, buttons, &raw mut has_hotkey as *mut bool);
    copy_confirm_hotkeys(
        buttons,
        dfltbutton,
        &raw mut has_hotkey as *mut bool as *const bool,
        hotk,
    );
    display_confirm_msg();
    return hotk;
}
unsafe extern "C" fn copy_confirm_hotkeys(
    mut buttons: *const ::core::ffi::c_char,
    mut default_button_idx: ::core::ffi::c_int,
    mut has_hotkey: *const bool,
    mut hotkeys_ptr: *mut ::core::ffi::c_char,
) {
    *hotkeys_ptr.offset(copy_char(buttons, hotkeys_ptr, true_0 != 0) as isize) =
        NUL as ::core::ffi::c_char;
    let mut first_hotkey: bool = false_0 != 0;
    if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
        first_hotkey = true_0 != 0;
    }
    let mut msgp: *mut ::core::ffi::c_char = confirm_buttons;
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *const ::core::ffi::c_char = buttons;
    while *r != 0 {
        if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
            let c2rust_fresh39 = msgp;
            msgp = msgp.offset(1);
            *c2rust_fresh39 = ',' as ::core::ffi::c_char;
            let c2rust_fresh40 = msgp;
            msgp = msgp.offset(1);
            *c2rust_fresh40 = ' ' as ::core::ffi::c_char;
            hotkeys_ptr = hotkeys_ptr.offset(strlen(hotkeys_ptr) as isize);
            *hotkeys_ptr.offset(copy_char(
                r.offset(1 as ::core::ffi::c_int as isize),
                hotkeys_ptr,
                true_0 != 0,
            ) as isize) = NUL as ::core::ffi::c_char;
            if default_button_idx != 0 {
                default_button_idx -= 1;
            }
            if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int && {
                idx += 1;
                !*has_hotkey.offset(idx as isize)
            } {
                first_hotkey = true_0 != 0;
            }
        } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int
            || first_hotkey as ::core::ffi::c_int != 0
        {
            if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                r = r.offset(1);
            }
            first_hotkey = false_0 != 0;
            if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                let c2rust_fresh41 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh41 = *r;
            } else {
                let c2rust_fresh42 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh42 = (if default_button_idx == 1 as ::core::ffi::c_int {
                    '[' as ::core::ffi::c_int
                } else {
                    '(' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
                let c2rust_fresh43 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh43 = (if default_button_idx == 1 as ::core::ffi::c_int {
                    ']' as ::core::ffi::c_int
                } else {
                    ')' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
                *hotkeys_ptr.offset(copy_char(r, hotkeys_ptr, true_0 != 0) as isize) =
                    NUL as ::core::ffi::c_char;
            }
        } else {
            msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
        }
        r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
    }
    let c2rust_fresh44 = msgp;
    msgp = msgp.offset(1);
    *c2rust_fresh44 = ':' as ::core::ffi::c_char;
    let c2rust_fresh45 = msgp;
    msgp = msgp.offset(1);
    *c2rust_fresh45 = ' ' as ::core::ffi::c_char;
    *msgp = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn display_confirm_msg() {
    confirm_msg_used += 1;
    if !confirm_msg.is_null() {
        msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts_hl(confirm_msg, HLF_M as ::core::ffi::c_int, false_0 != 0);
    }
    confirm_msg_used -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn vim_dialog_yesno(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if do_dialog(
        type_0,
        if title.is_null() {
            gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            title
        },
        message,
        gettext(b"&Yes\n&No\0".as_ptr() as *const ::core::ffi::c_char),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) == 1 as ::core::ffi::c_int
    {
        return VIM_YES as ::core::ffi::c_int;
    }
    return VIM_NO as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_dialog_yesnocancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match do_dialog(
        type_0,
        if title.is_null() {
            gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            title
        },
        message,
        gettext(b"&Yes\n&No\n&Cancel\0".as_ptr() as *const ::core::ffi::c_char),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) {
        1 => return VIM_YES as ::core::ffi::c_int,
        2 => return VIM_NO as ::core::ffi::c_int,
        _ => {}
    }
    return VIM_CANCEL as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_dialog_yesnoallcancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match do_dialog(
        type_0,
        if title.is_null() {
            b"Question\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            title as *const ::core::ffi::c_char
        },
        message,
        gettext(
            b"&Yes\n&No\nSave &All\n&Discard All\n&Cancel\0".as_ptr() as *const ::core::ffi::c_char
        ),
        dflt,
        ::core::ptr::null::<::core::ffi::c_char>(),
        false_0,
    ) {
        1 => return VIM_YES as ::core::ffi::c_int,
        2 => return VIM_NO as ::core::ffi::c_int,
        3 => return VIM_ALL as ::core::ffi::c_int,
        4 => return VIM_DISCARDALL as ::core::ffi::c_int,
        _ => {}
    }
    return VIM_CANCEL as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn msg_delay(mut ms: uint64_t, mut ignoreinput: bool) {
    if ui_has(kUIMessages) {
        return;
    }
    if nvim_testing {
        ms = 100 as uint64_t;
    }
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"msg_delay\0".as_ptr() as *const ::core::ffi::c_char,
        4047 as ::core::ffi::c_int,
        true_0 != 0,
        b"%lu ms%s\0".as_ptr() as *const ::core::ffi::c_char,
        ms,
        if nvim_testing as ::core::ffi::c_int != 0 {
            b" (skipped by NVIM_TEST)\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    ui_flush();
    os_delay(ms, ignoreinput);
}
#[no_mangle]
pub unsafe extern "C" fn msg_check_for_delay(mut check_msg_scroll: bool) {
    if (emsg_on_display as ::core::ffi::c_int != 0
        || check_msg_scroll as ::core::ffi::c_int != 0 && msg_scroll != 0)
        && !did_wait_return
        && emsg_silent == 0 as ::core::ffi::c_int
        && !in_assert_fails
        && !ui_has(kUIMessages)
    {
        msg_delay(1006 as uint64_t, true_0 != 0);
        emsg_on_display = false_0 != 0;
        if check_msg_scroll {
            msg_scroll = false_0;
        }
    }
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_UP: ::core::ffi::c_int = -30059;
pub const K_DOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('d' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_BS: ::core::ffi::c_int = -25195;
pub const K_PAGEUP: ::core::ffi::c_int = -20587;
pub const K_PAGEDOWN: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('N' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const K_EVENT: ::core::ffi::c_int = -26365;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
