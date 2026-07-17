extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    pub type multiqueue;
    pub type lua_State;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
    fn setbuf(__stream: *mut FILE, __buf: *mut ::core::ffi::c_char);
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
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn abort() -> !;
    fn exit(__status: ::core::ffi::c_int) -> !;
    fn memcpy(
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
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn tcdrain(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn log_init();
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_free_object(value: Object);
    fn api_metadata_raw() -> String_0;
    fn remote_ui_wait_for_attach();
    fn alist_init(al: *mut alist_T);
    fn alist_add(al: *mut alist_T, fname: *mut ::core::ffi::c_char, set_fnum: ::core::ffi::c_int);
    fn alist_name(aep: *mut aentry_T) -> *mut ::core::ffi::c_char;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn autocmd_init();
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn block_autocmds();
    fn unblock_autocmds();
    fn is_autocmd_blocked() -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn open_buffer(
        read_stdin_0: bool,
        eap: *mut exarg_T,
        flags_arg: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buf_valid(buf: *mut buf_T) -> bool;
    fn handle_swap_exists(old_curbuf: *mut bufref_T);
    fn set_curbuf(buf: *mut buf_T, action: ::core::ffi::c_int, update_jumplist: bool);
    fn do_autochdir();
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn setfname(
        buf: *mut buf_T,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        message: bool,
    ) -> ::core::ffi::c_int;
    fn do_modelines(flags: ::core::ffi::c_int);
    fn set_buflisted(on: ::core::ffi::c_int);
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn buf_set_changedtick(buf: *mut buf_T, changedtick: varnumber_T);
    fn channel_teardown();
    fn channel_init();
    fn channel_connect(
        tcp: bool,
        address: *const ::core::ffi::c_char,
        rpc: bool,
        on_output: CallbackReader,
        timeout: ::core::ffi::c_int,
        error: *mut *const ::core::ffi::c_char,
    ) -> uint64_t;
    fn channel_from_stdio(
        rpc: bool,
        on_output: CallbackReader,
        error: *mut *const ::core::ffi::c_char,
    ) -> uint64_t;
    fn diff_win_options(wp: *mut win_T, addbuf: bool);
    fn diffopt_horizontal() -> bool;
    fn default_grid_alloc() -> bool;
    fn screenclear();
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    fn eval_init();
    fn garbage_collect(testing: bool) -> bool;
    fn timer_teardown();
    fn set_argv_var(argv: *mut *mut ::core::ffi::c_char, argc: ::core::ffi::c_int);
    fn eval_has_provider(feat: *const ::core::ffi::c_char, throw_if_fast: bool) -> bool;
    fn hash_debug_results();
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn wait_return(redraw: ::core::ffi::c_int);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn invoke_all_defer();
    fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_type(idx: VimVarIndex, type_0: VarType);
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T);
    fn set_reg_var(c: ::core::ffi::c_int);
    fn loop_init(loop_0: *mut Loop, data: *mut ::core::ffi::c_void);
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    fn loop_close(loop_0: *mut Loop, wait: bool) -> bool;
    fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue;
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn os_realtime() -> int64_t;
    fn socket_address_tcp_host_end(address: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn proc_teardown(loop_0: *mut Loop);
    fn stream_set_blocking(fd: ::core::ffi::c_int, blocking: bool) -> ::core::ffi::c_int;
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn filetype_plugin_enable();
    fn filetype_maybe_enable();
    fn cmdline_init();
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
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn stuffcharReadbuff(c: ::core::ffi::c_int);
    fn open_scriptin(scriptin_name: *mut ::core::ffi::c_char) -> bool;
    fn vgetc() -> ::core::ffi::c_int;
    fn highlight_init();
    fn init_highlight(both: bool, reset: bool);
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_getfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn get_global_lstate() -> *mut lua_State;
    fn nlua_pcall(
        lstate: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn nlua_init(
        argv: *mut *mut ::core::ffi::c_char,
        argc: ::core::ffi::c_int,
        lua_arg0: ::core::ffi::c_int,
    );
    fn nlua_run_script(
        argv: *mut *mut ::core::ffi::c_char,
        argc: ::core::ffi::c_int,
        lua_arg0: ::core::ffi::c_int,
    ) -> !;
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_exec_file(path: *const ::core::ffi::c_char) -> bool;
    fn nlua_init_defaults();
    fn setpcmark();
    fn ml_close_all(del_file: bool);
    fn ml_close_notmod();
    fn ml_recover(checkext: bool);
    fn recover_names(
        fname: *mut ::core::ffi::c_char,
        do_list: bool,
        ret_list: *mut list_T,
        nr: ::core::ffi::c_int,
        fname_out: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn ml_sync_all(check_file: ::core::ffi::c_int, check_char: ::core::ffi::c_int, do_fsync: bool);
    fn update_topline(wp: *mut win_T);
    fn setmouse();
    fn server_init(listen_addr: *const ::core::ffi::c_char) -> bool;
    fn server_teardown();
    fn init_normal_cmds();
    fn normal_enter(cmdwin: bool, noexmode: bool);
    fn check_scrollbind(vtopline_diff: linenr_T, leftcol_diff: ::core::ffi::c_int);
    fn set_init_tablocal();
    fn set_init_1(clean_arg: bool);
    fn set_init_2(headless: bool);
    fn set_init_3();
    fn set_options_bin(
        oldval: ::core::ffi::c_int,
        newval: ::core::ffi::c_int,
        opt_flags: ::core::ffi::c_int,
    );
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn reset_modifiable();
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_exepath(buffer: *mut ::core::ffi::c_char, size: *mut size_t) -> ::core::ffi::c_int;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_write(
        fd: ::core::ffi::c_int,
        buf: *const ::core::ffi::c_char,
        size: size_t,
        non_blocking: bool,
    ) -> ptrdiff_t;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn input_start();
    fn input_stop();
    fn os_breakcheck();
    fn os_isatty(fd: ::core::ffi::c_int) -> bool;
    fn set_lang_var();
    fn init_locale();
    fn env_init();
    fn os_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_getenv_noalloc(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_hint_priority();
    fn init_homedir();
    fn vim_env_iter(
        delim: ::core::ffi::c_char,
        val: *const ::core::ffi::c_char,
        iter: *const ::core::ffi::c_void,
        dir: *mut *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_void;
    fn get_appname(namelike: bool) -> *const ::core::ffi::c_char;
    fn appname_is_valid() -> bool;
    fn stdpaths_get_xdg_var(idx: XDGVarType) -> *mut ::core::ffi::c_char;
    fn stdpaths_user_conf_subpath(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn signal_init();
    fn signal_teardown();
    fn signal_stop();
    fn signal_reject_deadly();
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn concat_fnames(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        sep: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_FullName(
        fname: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
        force: bool,
    ) -> ::core::ffi::c_int;
    fn path_guess_exepath(
        argv0_0: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        bufsize: size_t,
    );
    fn profile_dump();
    fn time_start(message: *const ::core::ffi::c_char);
    fn time_msg(mesg: *const ::core::ffi::c_char, start: *const proftime_T);
    fn time_init(fname: *const ::core::ffi::c_char, proc_name: *const ::core::ffi::c_char);
    fn time_finish();
    fn get_default_register_name() -> ::core::ffi::c_int;
    fn qf_init(
        wp: *mut win_T,
        efile: *const ::core::ffi::c_char,
        errorformat: *mut ::core::ffi::c_char,
        newlist: ::core::ffi::c_int,
        qf_title: *const ::core::ffi::c_char,
        enc: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn qf_init_stack();
    fn qf_jump(
        qi: *mut qf_info_T,
        dir: ::core::ffi::c_int,
        errornr: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    );
    fn estack_init();
    fn estack_push(
        type_0: etype_T,
        name: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut estack_T;
    fn estack_pop();
    fn runtime_init();
    fn load_plugins();
    fn do_source(
        fname: *mut ::core::ffi::c_char,
        check_other: bool,
        is_vimrc: ::core::ffi::c_int,
        ret_sid: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn shada_write_file(file: *const ::core::ffi::c_char, nomerge: bool) -> ::core::ffi::c_int;
    fn shada_read_everything(
        fname: *const ::core::ffi::c_char,
        forceit: bool,
        missing_ok: bool,
    ) -> ::core::ffi::c_int;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn syn_maybe_enable();
    fn terminal_init();
    fn terminal_teardown();
    fn ui_call_set_title(title: String_0);
    fn ui_call_stop();
    fn ui_call_error_exit(status: Integer);
    fn ui_init();
    fn do_autocmd_uienter_all();
    fn ui_flush();
    fn ui_client_start_server(
        exepath: *const ::core::ffi::c_char,
        argc: size_t,
        argv: *mut *mut ::core::ffi::c_char,
    ) -> uint64_t;
    fn ui_client_run() -> !;
    fn ui_client_stop();
    fn ui_comp_syn_init();
    fn list_version();
    fn win_count() -> ::core::ffi::c_int;
    fn make_windows(count: ::core::ffi::c_int, vertical: bool) -> ::core::ffi::c_int;
    fn win_equal(next_curwin: *mut win_T, current: bool, dir: ::core::ffi::c_int);
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn win_alloc_first();
    fn win_init_size();
    fn make_tabpages(maxcount: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn goto_tabpage(n: ::core::ffi::c_int);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_new_screensize();
    fn only_one_window() -> bool;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_22,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
pub type uv_mutex_t = pthread_mutex_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_19,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_14,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
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
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_18,
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
    pub timer_heap: C2Rust_Unnamed_17,
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
    pub u: C2Rust_Unnamed_16,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_15,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_16 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_17 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_21,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_20,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_24,
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
pub type uv_file = ::core::ffi::c_int;
pub type uv_stream_t = uv_stream_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_23,
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
pub union C2Rust_Unnamed_23 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_25,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_25 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_26,
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
pub union C2Rust_Unnamed_26 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_27,
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
pub union C2Rust_Unnamed_27 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aentry_T {
    pub ae_fname: *mut ::core::ffi::c_char,
    pub ae_fnum: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_28 = 2147483647;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
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
pub type RgbValue = int32_t;
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
pub struct Set_int {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_String {
    pub h: MapHash,
    pub keys: *mut String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int_ptr_t {
    pub set: Set_int,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_String_int {
    pub set: Set_String,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_29; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_29 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
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
    pub cs_pend: C2Rust_Unnamed_30,
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
pub union C2Rust_Unnamed_30 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aucmdwin_T {
    pub auc_win: *mut win_T,
    pub auc_win_used: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_31 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
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
    pub data: C2Rust_Unnamed_32,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_32 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_33 {
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
    pub slots: C2Rust_Unnamed_35,
    pub ranges_i: C2Rust_Unnamed_34,
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
pub struct C2Rust_Unnamed_34 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_35 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_36 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
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
pub struct C2Rust_Unnamed_37 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_38 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_38 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_38 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_38 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_38 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_38 = 20;
pub const UPD_VALID: C2Rust_Unnamed_38 = 10;
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
pub type XDGVarType = ::core::ffi::c_int;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_39 = 1;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_40 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_40 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_40 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_40 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_40 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_40 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_40 = 1;
pub type C2Rust_Unnamed_41 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_41 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_41 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_41 = 0;
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
pub type C2Rust_Unnamed_42 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_42 = 256;
pub const READ_NOWINENTER: C2Rust_Unnamed_42 = 128;
pub const READ_FIFO: C2Rust_Unnamed_42 = 64;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_42 = 32;
pub const READ_DUMMY: C2Rust_Unnamed_42 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_42 = 8;
pub const READ_STDIN: C2Rust_Unnamed_42 = 4;
pub const READ_FILTER: C2Rust_Unnamed_42 = 2;
pub const READ_NEW: C2Rust_Unnamed_42 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_43,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_43 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_44 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_44 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_44 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_44 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_44 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_44 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_44 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_44 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_44 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_44 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_44 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_44 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_44 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_44 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_44 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_44 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_44 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_44 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_44 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_44 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nvim_stats_s {
    pub fsync: int64_t,
    pub redraw: int64_t,
    pub log_skip: int16_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct caller_scope {
    pub script_ctx: sctx_T,
    pub es_entry: estack_T,
    pub autocmd_fname: *mut ::core::ffi::c_char,
    pub autocmd_match: *mut ::core::ffi::c_char,
    pub autocmd_fname_full: bool,
    pub autocmd_bufnr: ::core::ffi::c_int,
    pub funccalp: *mut ::core::ffi::c_void,
}
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const kOptCbFlagUnnamedplus: C2Rust_Unnamed_45 = 2;
pub const kOptCbFlagUnnamed: C2Rust_Unnamed_45 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nlua_ref_state_t {
    pub nil_ref: LuaRef,
    pub empty_dict_ref: LuaRef,
    pub ref_count: ::core::ffi::c_int,
}
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mparm_T {
    pub argc: ::core::ffi::c_int,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub use_vimrc: *mut ::core::ffi::c_char,
    pub clean: bool,
    pub n_commands: ::core::ffi::c_int,
    pub commands: [*mut ::core::ffi::c_char; 10],
    pub cmds_tofree: [::core::ffi::c_char; 10],
    pub n_pre_commands: ::core::ffi::c_int,
    pub pre_commands: [*mut ::core::ffi::c_char; 10],
    pub luaf: *mut ::core::ffi::c_char,
    pub lua_arg0: ::core::ffi::c_int,
    pub edit_type: ::core::ffi::c_int,
    pub tagname: *mut ::core::ffi::c_char,
    pub use_ef: *mut ::core::ffi::c_char,
    pub input_istext: bool,
    pub no_swap_file: ::core::ffi::c_int,
    pub use_debug_break_level: ::core::ffi::c_int,
    pub window_count: ::core::ffi::c_int,
    pub window_layout: ::core::ffi::c_int,
    pub diff_mode: ::core::ffi::c_int,
    pub listen_addr: *mut ::core::ffi::c_char,
    pub remote: ::core::ffi::c_int,
    pub server_addr: *mut ::core::ffi::c_char,
    pub scriptin: *mut ::core::ffi::c_char,
    pub scriptout: *mut ::core::ffi::c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}
pub const EDIT_QF: C2Rust_Unnamed_49 = 4;
pub const WIN_TABS: C2Rust_Unnamed_48 = 3;
pub const WIN_VER: C2Rust_Unnamed_48 = 2;
pub const WIN_HOR: C2Rust_Unnamed_48 = 1;
pub const EDIT_STDIN: C2Rust_Unnamed_49 = 2;
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const DOSO_VIMRC: C2Rust_Unnamed_47 = 1;
pub const DOSO_NONE: C2Rust_Unnamed_47 = 0;
pub const EDIT_FILE: C2Rust_Unnamed_49 = 1;
pub const EDIT_TAG: C2Rust_Unnamed_49 = 3;
pub const EDIT_NONE: C2Rust_Unnamed_49 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_46 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub type C2Rust_Unnamed_47 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_48 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_49 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub static mut arena_alloc_count: size_t = 0 as size_t;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_min_log_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SESSION_FILE: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Session.vim\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut global_opt_idx: [OptIndex; 264] = [
    kOptAleph,
    kOptAllowrevins,
    kOptAmbiwidth,
    kOptArabicshape,
    kOptAutochdir,
    kOptAutocomplete,
    kOptAutocompletedelay,
    kOptAutocompletetimeout,
    kOptAutoread,
    kOptAutowrite,
    kOptAutowriteall,
    kOptBackground,
    kOptBackspace,
    kOptBackup,
    kOptBackupcopy,
    kOptBackupdir,
    kOptBackupext,
    kOptBackupskip,
    kOptBelloff,
    kOptBreakat,
    kOptBrowsedir,
    kOptCasemap,
    kOptCdhome,
    kOptCdpath,
    kOptCedit,
    kOptCharconvert,
    kOptChistory,
    kOptClipboard,
    kOptCmdheight,
    kOptCmdwinheight,
    kOptColumns,
    kOptCompatible,
    kOptCompleteitemalign,
    kOptCompleteopt,
    kOptCompletetimeout,
    kOptConfirm,
    kOptCpoptions,
    kOptDebug,
    kOptDefine,
    kOptDelcombine,
    kOptDictionary,
    kOptDiffanchors,
    kOptDiffexpr,
    kOptDiffopt,
    kOptDigraph,
    kOptDirectory,
    kOptDisplay,
    kOptEadirection,
    kOptEdcompatible,
    kOptEmoji,
    kOptEncoding,
    kOptEqualalways,
    kOptEqualprg,
    kOptErrorbells,
    kOptErrorfile,
    kOptErrorformat,
    kOptEventignore,
    kOptExrc,
    kOptFileencodings,
    kOptFileformats,
    kOptFileignorecase,
    kOptFillchars,
    kOptFindfunc,
    kOptFoldclose,
    kOptFoldlevelstart,
    kOptFoldopen,
    kOptFormatprg,
    kOptFsync,
    kOptGdefault,
    kOptGrepformat,
    kOptGrepprg,
    kOptGuicursor,
    kOptGuifont,
    kOptGuifontwide,
    kOptGuioptions,
    kOptGuitablabel,
    kOptGuitabtooltip,
    kOptHelpfile,
    kOptHelpheight,
    kOptHelplang,
    kOptHidden,
    kOptHighlight,
    kOptHistory,
    kOptHkmap,
    kOptHkmapp,
    kOptHlsearch,
    kOptIcon,
    kOptIconstring,
    kOptIgnorecase,
    kOptImcmdline,
    kOptImdisable,
    kOptInccommand,
    kOptInclude,
    kOptIncsearch,
    kOptInsertmode,
    kOptIsfname,
    kOptIsident,
    kOptIsprint,
    kOptJoinspaces,
    kOptJumpoptions,
    kOptKeymodel,
    kOptKeywordprg,
    kOptLangmap,
    kOptLangmenu,
    kOptLangnoremap,
    kOptLangremap,
    kOptLaststatus,
    kOptLazyredraw,
    kOptLines,
    kOptLinespace,
    kOptLispwords,
    kOptListchars,
    kOptLoadplugins,
    kOptMagic,
    kOptMakeef,
    kOptMakeencoding,
    kOptMakeprg,
    kOptMatchtime,
    kOptMaxcombine,
    kOptMaxfuncdepth,
    kOptMaxmapdepth,
    kOptMaxmempattern,
    kOptMaxsearchcount,
    kOptMenuitems,
    kOptMessagesopt,
    kOptMkspellmem,
    kOptModelineexpr,
    kOptModelines,
    kOptMore,
    kOptMouse,
    kOptMousefocus,
    kOptMousehide,
    kOptMousemodel,
    kOptMousemoveevent,
    kOptMousescroll,
    kOptMouseshape,
    kOptMousetime,
    kOptOpendevice,
    kOptOperatorfunc,
    kOptPackpath,
    kOptParagraphs,
    kOptPaste,
    kOptPastetoggle,
    kOptPatchexpr,
    kOptPatchmode,
    kOptPath,
    kOptPreviewheight,
    kOptPrompt,
    kOptPumblend,
    kOptPumborder,
    kOptPumheight,
    kOptPummaxwidth,
    kOptPumwidth,
    kOptPyxversion,
    kOptQuickfixtextfunc,
    kOptRedrawdebug,
    kOptRedrawtime,
    kOptRegexpengine,
    kOptRemap,
    kOptReport,
    kOptRevins,
    kOptRuler,
    kOptRulerformat,
    kOptRuntimepath,
    kOptScrolljump,
    kOptScrolloff,
    kOptScrollopt,
    kOptSections,
    kOptSecure,
    kOptSelection,
    kOptSelectmode,
    kOptSessionoptions,
    kOptShada,
    kOptShadafile,
    kOptShell,
    kOptShellcmdflag,
    kOptShellpipe,
    kOptShellquote,
    kOptShellredir,
    kOptShellslash,
    kOptShelltemp,
    kOptShellxescape,
    kOptShellxquote,
    kOptShiftround,
    kOptShortmess,
    kOptShowbreak,
    kOptShowcmd,
    kOptShowcmdloc,
    kOptShowfulltag,
    kOptShowmatch,
    kOptShowmode,
    kOptShowtabline,
    kOptSidescroll,
    kOptSidescrolloff,
    kOptSmartcase,
    kOptSmarttab,
    kOptSpellsuggest,
    kOptSplitbelow,
    kOptSplitkeep,
    kOptSplitright,
    kOptStartofline,
    kOptStatusline,
    kOptSuffixes,
    kOptSwitchbuf,
    kOptTabclose,
    kOptTabline,
    kOptTabpagemax,
    kOptTagbsearch,
    kOptTagcase,
    kOptTaglength,
    kOptTagrelative,
    kOptTags,
    kOptTagstack,
    kOptTermbidi,
    kOptTermencoding,
    kOptTermguicolors,
    kOptTermpastefilter,
    kOptTermsync,
    kOptTerse,
    kOptThesaurus,
    kOptThesaurusfunc,
    kOptTildeop,
    kOptTimeout,
    kOptTimeoutlen,
    kOptTitle,
    kOptTitlelen,
    kOptTitleold,
    kOptTitlestring,
    kOptTtimeout,
    kOptTtimeoutlen,
    kOptTtyfast,
    kOptUndodir,
    kOptUndolevels,
    kOptUndoreload,
    kOptUpdatecount,
    kOptUpdatetime,
    kOptVerbose,
    kOptVerbosefile,
    kOptViewdir,
    kOptViewoptions,
    kOptVirtualedit,
    kOptVisualbell,
    kOptWarn,
    kOptWhichwrap,
    kOptWildchar,
    kOptWildcharm,
    kOptWildignore,
    kOptWildignorecase,
    kOptWildmenu,
    kOptWildmode,
    kOptWildoptions,
    kOptWinaltkeys,
    kOptWinbar,
    kOptWinborder,
    kOptWindow,
    kOptWinheight,
    kOptWinminheight,
    kOptWinminwidth,
    kOptWinwidth,
    kOptWrapscan,
    kOptWrite,
    kOptWriteany,
    kOptWritebackup,
    kOptWritedelay,
];
#[no_mangle]
pub static mut buf_opt_idx: [OptIndex; 92] = [
    kOptAutocomplete,
    kOptAutoindent,
    kOptAutoread,
    kOptBackupcopy,
    kOptBinary,
    kOptBomb,
    kOptBufhidden,
    kOptBuflisted,
    kOptBuftype,
    kOptBusy,
    kOptChannel,
    kOptCindent,
    kOptCinkeys,
    kOptCinoptions,
    kOptCinscopedecls,
    kOptCinwords,
    kOptComments,
    kOptCommentstring,
    kOptComplete,
    kOptCompletefunc,
    kOptCompleteopt,
    kOptCompleteslash,
    kOptCopyindent,
    kOptDefine,
    kOptDictionary,
    kOptDiffanchors,
    kOptEndoffile,
    kOptEndofline,
    kOptEqualprg,
    kOptErrorformat,
    kOptExpandtab,
    kOptFileencoding,
    kOptFileformat,
    kOptFiletype,
    kOptFindfunc,
    kOptFixendofline,
    kOptFormatexpr,
    kOptFormatlistpat,
    kOptFormatoptions,
    kOptFormatprg,
    kOptFsync,
    kOptGrepformat,
    kOptGrepprg,
    kOptIminsert,
    kOptImsearch,
    kOptInclude,
    kOptIncludeexpr,
    kOptIndentexpr,
    kOptIndentkeys,
    kOptInfercase,
    kOptIskeyword,
    kOptKeymap,
    kOptKeywordprg,
    kOptLisp,
    kOptLispoptions,
    kOptLispwords,
    kOptMakeencoding,
    kOptMakeprg,
    kOptMatchpairs,
    kOptModeline,
    kOptModifiable,
    kOptModified,
    kOptNrformats,
    kOptOmnifunc,
    kOptPath,
    kOptPreserveindent,
    kOptQuoteescape,
    kOptReadonly,
    kOptScrollback,
    kOptShiftwidth,
    kOptSmartindent,
    kOptSofttabstop,
    kOptSpellcapcheck,
    kOptSpellfile,
    kOptSpelllang,
    kOptSpelloptions,
    kOptSuffixesadd,
    kOptSwapfile,
    kOptSynmaxcol,
    kOptSyntax,
    kOptTabstop,
    kOptTagcase,
    kOptTagfunc,
    kOptTags,
    kOptTextwidth,
    kOptThesaurus,
    kOptThesaurusfunc,
    kOptUndofile,
    kOptUndolevels,
    kOptVarsofttabstop,
    kOptVartabstop,
    kOptWrapmargin,
];
#[no_mangle]
pub static mut win_opt_idx: [OptIndex; 51] = [
    kOptArabic,
    kOptBreakindent,
    kOptBreakindentopt,
    kOptColorcolumn,
    kOptConcealcursor,
    kOptConceallevel,
    kOptCursorbind,
    kOptCursorcolumn,
    kOptCursorline,
    kOptCursorlineopt,
    kOptDiff,
    kOptEventignorewin,
    kOptFillchars,
    kOptFoldcolumn,
    kOptFoldenable,
    kOptFoldexpr,
    kOptFoldignore,
    kOptFoldlevel,
    kOptFoldmarker,
    kOptFoldmethod,
    kOptFoldminlines,
    kOptFoldnestmax,
    kOptFoldtext,
    kOptLhistory,
    kOptLinebreak,
    kOptList,
    kOptListchars,
    kOptNumber,
    kOptNumberwidth,
    kOptPreviewwindow,
    kOptRelativenumber,
    kOptRightleft,
    kOptRightleftcmd,
    kOptScroll,
    kOptScrollbind,
    kOptScrolloff,
    kOptShowbreak,
    kOptSidescrolloff,
    kOptSigncolumn,
    kOptSmoothscroll,
    kOptSpell,
    kOptStatuscolumn,
    kOptStatusline,
    kOptVirtualedit,
    kOptWinbar,
    kOptWinblend,
    kOptWinfixbuf,
    kOptWinfixheight,
    kOptWinfixwidth,
    kOptWinhighlight,
    kOptWrap,
];
#[no_mangle]
pub static mut namespace_ids: Map_String_int = Map_String_int {
    set: Set_String {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<String_0>(),
    },
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
#[no_mangle]
pub static mut namespace_localscope: Set_uint32_t = Set_uint32_t {
    h: MapHash {
        n_buckets: 0 as uint32_t,
        size: 0 as uint32_t,
        n_occupied: 0 as uint32_t,
        upper_bound: 0 as uint32_t,
        n_keys: 0 as uint32_t,
        keys_capacity: 0 as uint32_t,
        hash: ::core::ptr::null_mut::<uint32_t>(),
    },
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
#[no_mangle]
pub static mut next_namespace_id: handle_T = 1 as handle_T;
#[no_mangle]
pub static mut buffer_handles: Map_int_ptr_t = Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
#[no_mangle]
pub static mut window_handles: Map_int_ptr_t = Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
#[no_mangle]
pub static mut tabpage_handles: Map_int_ptr_t = Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
#[no_mangle]
pub static mut ui_ext_names: [*const ::core::ffi::c_char; 10] = [
    b"ext_cmdline\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_popupmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_tabline\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_wildmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_messages\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_linegrid\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_multigrid\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_hlstate\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext_termcolors\0".as_ptr() as *const ::core::ffi::c_char,
    b"_debug_float\0".as_ptr() as *const ::core::ffi::c_char,
];
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[no_mangle]
pub static mut last_cursormoved_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut last_cursormoved: pos_T = pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
};
#[no_mangle]
pub static mut autocmd_busy: bool = false;
#[no_mangle]
pub static mut autocmd_no_enter: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut autocmd_no_leave: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut au_new_curbuf: bufref_T = bufref_T {
    br_buf: ::core::ptr::null_mut::<buf_T>(),
    br_fnum: 0 as ::core::ffi::c_int,
    br_buf_free_count: 0 as ::core::ffi::c_int,
};
#[no_mangle]
pub static mut au_pending_free_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
#[no_mangle]
pub static mut au_pending_free_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut autocmd_fname: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut autocmd_fname_full: bool = false;
#[no_mangle]
pub static mut autocmd_bufnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut autocmd_match: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut did_cursorhold: bool = true;
#[no_mangle]
pub static mut aucmd_win_vec: C2Rust_Unnamed_31 = C2Rust_Unnamed_31 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<aucmdwin_T>(),
};
#[no_mangle]
pub static mut deferred_events: *mut MultiQueue = ::core::ptr::null_mut::<MultiQueue>();
#[no_mangle]
pub static mut msg_loclist: *mut ::core::ffi::c_char =
    b"[Location List]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut msg_qflist: *mut ::core::ffi::c_char =
    b"[Quickfix List]\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[no_mangle]
pub static mut channels: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: Set_uint64_t {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<uint64_t>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
};
#[no_mangle]
pub static mut on_print: Callback = Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
};
#[no_mangle]
pub static mut virt_text_pos_str: [*const ::core::ffi::c_char; 6] = [
    b"eol\0".as_ptr() as *const ::core::ffi::c_char,
    b"eol_right_align\0".as_ptr() as *const ::core::ffi::c_char,
    b"inline\0".as_ptr() as *const ::core::ffi::c_char,
    b"overlay\0".as_ptr() as *const ::core::ffi::c_char,
    b"right_align\0".as_ptr() as *const ::core::ffi::c_char,
    b"win_col\0".as_ptr() as *const ::core::ffi::c_char,
];
#[no_mangle]
pub static mut hl_mode_str: [*const ::core::ffi::c_char; 4] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"replace\0".as_ptr() as *const ::core::ffi::c_char,
    b"combine\0".as_ptr() as *const ::core::ffi::c_char,
    b"blend\0".as_ptr() as *const ::core::ffi::c_char,
];
#[no_mangle]
pub static mut decor_state: DecorState = DecorState {
    itr: [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_29 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }],
    slots: C2Rust_Unnamed_35 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<DecorRangeSlot>(),
    },
    ranges_i: C2Rust_Unnamed_34 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    current_end: 0,
    future_begin: 0,
    free_slot_i: 0,
    new_range_ordering: 0,
    win: ::core::ptr::null_mut::<win_T>(),
    top_row: 0,
    row: 0,
    col_last: 0,
    current: 0,
    eol_col: 0,
    conceal: 0,
    conceal_char: 0,
    conceal_attr: 0,
    spell: kFalse,
    running_decor_provider: false,
    itr_valid: false,
};
#[no_mangle]
pub static mut decor_items: C2Rust_Unnamed_36 = C2Rust_Unnamed_36 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<DecorSignHighlight>(),
};
#[no_mangle]
pub static mut diff_context: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
#[no_mangle]
pub static mut diff_foldcolumn: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut diff_need_scrollbind: bool = false;
#[no_mangle]
pub static mut need_diff_redraw: bool = false;
#[no_mangle]
pub static mut win_extmark_arr: C2Rust_Unnamed_37 = C2Rust_Unnamed_37 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<WinExtmark>(),
};
#[no_mangle]
pub static mut updating_screen: bool = false;
#[no_mangle]
pub static mut redraw_not_allowed: bool = false;
#[no_mangle]
pub static mut screen_search_hl: match_T = match_T {
    rm: regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    },
    buf: ::core::ptr::null_mut::<buf_T>(),
    lnum: 0,
    attr: 0,
    attr_cur: 0,
    first_lnum: 0,
    startcol: 0,
    endcol: 0,
    is_addpos: false,
    has_cursor: false,
    tm: 0,
};
#[no_mangle]
pub static mut search_hl_has_cursor_lnum: linenr_T = 0 as linenr_T;
#[no_mangle]
pub static mut e_abort: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E470: Command aborted\0")
};
#[no_mangle]
pub static mut e_afterinit: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E905: Cannot set this option after startup\0",
    )
};
#[no_mangle]
pub static mut e_api_spawn_failed: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E903: Could not spawn API job\0",
    )
};
#[no_mangle]
pub static mut e_argreq: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E471: Argument required\0")
};
#[no_mangle]
pub static mut e_backslash: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E10: \\ should be followed by /, ? or &\0",
    )
};
#[no_mangle]
pub static mut e_cmdwin: [::core::ffi::c_char; 65] = unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E11: Invalid in command-line window; <CR> executes, CTRL-C quits\0",
    )
};
#[no_mangle]
pub static mut e_curdir: [::core::ffi::c_char; 69] = unsafe {
    ::core::mem::transmute::<[u8; 69], [::core::ffi::c_char; 69]>(
        *b"E12: Command not allowed in secure mode in current dir or tag search\0",
    )
};
#[no_mangle]
pub static mut e_invalid_buffer_name_str: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E158: Invalid buffer name: %s\0",
    )
};
#[no_mangle]
pub static mut e_command_too_recursive: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E169: Command too recursive\0")
};
#[no_mangle]
pub static mut e_buffer_is_not_loaded: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E681: Buffer is not loaded\0")
};
#[no_mangle]
pub static mut e_endif: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E171: Missing :endif\0")
};
#[no_mangle]
pub static mut e_endtry: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E600: Missing :endtry\0")
};
#[no_mangle]
pub static mut e_endwhile: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E170: Missing :endwhile\0")
};
#[no_mangle]
pub static mut e_endfor: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E170: Missing :endfor\0")
};
#[no_mangle]
pub static mut e_while: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E588: :endwhile without :while\0",
    )
};
#[no_mangle]
pub static mut e_for: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E588: :endfor without :for\0")
};
#[no_mangle]
pub static mut e_exists: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E13: File exists (add ! to override)\0",
    )
};
#[no_mangle]
pub static mut e_failed: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E472: Command failed\0")
};
#[no_mangle]
pub static mut e_intern2: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E685: Internal error: %s\0")
};
#[no_mangle]
pub static mut e_interr: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"Interrupted\0") };
#[no_mangle]
pub static mut e_invarg: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E474: Invalid argument\0")
};
#[no_mangle]
pub static mut e_invarg2: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E475: Invalid argument: %s\0")
};
#[no_mangle]
pub static mut e_invargval: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E475: Invalid value for argument %s\0",
    )
};
#[no_mangle]
pub static mut e_invargNval: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E475: Invalid value for argument %s: %s\0",
    )
};
#[no_mangle]
pub static mut e_duparg2: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E983: Duplicate argument: %s\0",
    )
};
#[no_mangle]
pub static mut e_invexpr2: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E15: Invalid expression: \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_invrange: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E16: Invalid range\0")
};
#[no_mangle]
pub static mut e_internal_error_in_regexp: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E473: Internal error in regexp\0",
    )
};
#[no_mangle]
pub static mut e_invcmd: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E476: Invalid command\0")
};
#[no_mangle]
pub static mut e_isadir2: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E17: \"%s\" is a directory\0")
};
#[no_mangle]
pub static mut e_no_spell: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E756: Spell checking is not possible\0",
    )
};
#[no_mangle]
pub static mut e_invchan: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E900: Invalid channel id\0")
};
#[no_mangle]
pub static mut e_invchanjob: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E900: Invalid channel id: not a job\0",
    )
};
#[no_mangle]
pub static mut e_jobtblfull: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E901: Job table is full\0")
};
#[no_mangle]
pub static mut e_jobspawn: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E903: Process failed to start: %s: \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_channotpty: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E904: channel is not a pty\0")
};
#[no_mangle]
pub static mut e_stdiochan2: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E905: Couldn't open stdio channel: %s\0",
    )
};
#[no_mangle]
pub static mut e_invstream: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E906: invalid stream for channel\0",
    )
};
#[no_mangle]
pub static mut e_invstreamrpc: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E906: invalid stream for rpc channel, use 'rpc'\0",
    )
};
#[no_mangle]
pub static mut e_streamkey: [::core::ffi::c_char; 68] = unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"E5210: dict key '%s' already set for buffered stream in channel %lu\0",
    )
};
#[no_mangle]
pub static mut e_libcall: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E364: Library call failed for \"%s()\"\0",
    )
};
#[no_mangle]
pub static mut e_fsync: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E667: Fsync failed: %s\0")
};
#[no_mangle]
pub static mut e_mkdir: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E739: Cannot create directory %s: %s\0",
    )
};
#[no_mangle]
pub static mut e_markinval: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E19: Mark has invalid line number\0",
    )
};
#[no_mangle]
pub static mut e_marknotset: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E20: Mark not set\0")
};
#[no_mangle]
pub static mut e_modifiable: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"E21: Cannot make changes, 'modifiable' is off\0",
    )
};
#[no_mangle]
pub static mut e_nesting: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E22: Scripts nested too deep\0",
    )
};
#[no_mangle]
pub static mut e_noalt: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E23: No alternate file\0")
};
#[no_mangle]
pub static mut e_noabbr: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E24: No such abbreviation\0")
};
#[no_mangle]
pub static mut e_nobang: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E477: No ! allowed\0")
};
#[no_mangle]
pub static mut e_nogroup: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E28: No such highlight group name: %s\0",
    )
};
#[no_mangle]
pub static mut e_noinstext: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E29: No inserted text yet\0")
};
#[no_mangle]
pub static mut e_nolastcmd: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E30: No previous command line\0",
    )
};
#[no_mangle]
pub static mut e_nomap: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E31: No such mapping\0")
};
#[no_mangle]
pub static mut e_noident: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E349: No identifier under cursor\0",
    )
};
#[no_mangle]
pub static mut e_nomatch: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"E479: No match\0") };
#[no_mangle]
pub static mut e_nomatch2: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E480: No match: %s\0")
};
#[no_mangle]
pub static mut e_noname: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E32: No file name\0")
};
#[no_mangle]
pub static mut e_nopresub: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"E33: No previous substitute regular expression\0",
    )
};
#[no_mangle]
pub static mut e_noprev: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E34: No previous command\0")
};
#[no_mangle]
pub static mut e_noprevre: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E35: No previous regular expression\0",
    )
};
#[no_mangle]
pub static mut e_norange: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E481: No range allowed\0")
};
#[no_mangle]
pub static mut e_noroom: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E36: Not enough room\0")
};
#[no_mangle]
pub static mut e_notmp: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E483: Can't get temp file name\0",
    )
};
#[no_mangle]
pub static mut e_notopen: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E484: Can't open file %s\0")
};
#[no_mangle]
pub static mut e_notopen_2: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E484: Can't open file %s: %s\0",
    )
};
#[no_mangle]
pub static mut e_cant_read_file_str: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E485: Can't read file %s\0")
};
#[no_mangle]
pub static mut e_null: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E38: Null argument\0")
};
#[no_mangle]
pub static mut e_number_exp: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E39: Number expected\0")
};
#[no_mangle]
pub static mut e_openerrf: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E40: Can't open errorfile %s\0",
    )
};
#[no_mangle]
pub static mut e_outofmem: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E41: Out of memory!\0")
};
#[no_mangle]
pub static mut e_patnotf: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"Pattern not found\0")
};
#[no_mangle]
pub static mut e_patnotf2: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E486: Pattern not found: %s\0")
};
#[no_mangle]
pub static mut e_positive: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E487: Argument must be positive\0",
    )
};
#[no_mangle]
pub static mut e_prev_dir: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E459: Cannot go back to previous directory\0",
    )
};
#[no_mangle]
pub static mut e_no_errors: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"E42: No Errors\0") };
#[no_mangle]
pub static mut e_loclist: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E776: No location list\0")
};
#[no_mangle]
pub static mut e_re_damg: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E43: Damaged match string\0")
};
#[no_mangle]
pub static mut e_re_corr: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E44: Corrupted regexp program\0",
    )
};
#[no_mangle]
pub static mut e_readonly: [::core::ffi::c_char; 50] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E45: 'readonly' option is set (add ! to override)\0",
    )
};
#[no_mangle]
pub static mut e_letwrong: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E734: Wrong variable type for %s=\0",
    )
};
#[no_mangle]
pub static mut e_illvar: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E461: Illegal variable name: %s\0",
    )
};
#[no_mangle]
pub static mut e_cannot_mod: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E995: Cannot modify existing variable\0",
    )
};
#[no_mangle]
pub static mut e_cannot_change_readonly_variable_str: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E46: Cannot change read-only variable \"%.*s\"\0",
    )
};
#[no_mangle]
pub static mut e_dictreq: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E715: Dictionary required\0")
};
#[no_mangle]
pub static mut e_blobidx: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E979: Blob index out of range: %ld\0",
    )
};
#[no_mangle]
pub static mut e_invalblob: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E978: Invalid operation for Blob\0",
    )
};
#[no_mangle]
pub static mut e_toomanyarg: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E118: Too many arguments for function: %s\0",
    )
};
#[no_mangle]
pub static mut e_toofewarg: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E119: Not enough arguments for function: %s\0",
    )
};
#[no_mangle]
pub static mut e_dictkey: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E716: Key not present in Dictionary: \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_dictkey_len: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E716: Key not present in Dictionary: \"%.*s\"\0",
    )
};
#[no_mangle]
pub static mut e_listreq: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E714: List required\0")
};
#[no_mangle]
pub static mut e_listblobreq: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E897: List or Blob required\0")
};
#[no_mangle]
pub static mut e_listblobarg: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E899: Argument of %s must be a List or Blob\0",
    )
};
#[no_mangle]
pub static mut e_listdictarg: [::core::ffi::c_char; 50] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E712: Argument of %s must be a List or Dictionary\0",
    )
};
#[no_mangle]
pub static mut e_listdictblobarg: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"E896: Argument of %s must be a List, Dictionary or Blob\0",
    )
};
#[no_mangle]
pub static mut e_readerrf: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E47: Error while reading errorfile\0",
    )
};
#[no_mangle]
pub static mut e_sandbox: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E48: Not allowed in sandbox\0")
};
#[no_mangle]
pub static mut e_secure: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E523: Not allowed here\0")
};
#[no_mangle]
pub static mut e_textlock: [::core::ffi::c_char; 50] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E565: Not allowed to change text or change window\0",
    )
};
#[no_mangle]
pub static mut e_screenmode: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E359: Screen mode setting not supported\0",
    )
};
#[no_mangle]
pub static mut e_scroll: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E49: Invalid scroll size\0")
};
#[no_mangle]
pub static mut e_shellempty: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E91: 'shell' option is empty\0",
    )
};
#[no_mangle]
pub static mut e_signdata: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E255: Couldn't read in sign data!\0",
    )
};
#[no_mangle]
pub static mut e_swapclose: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E72: Close error on swap file\0",
    )
};
#[no_mangle]
pub static mut e_toocompl: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E74: Command too complex\0")
};
#[no_mangle]
pub static mut e_longname: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E75: Name too long\0")
};
#[no_mangle]
pub static mut e_toomsbra: [::core::ffi::c_char; 16] =
    unsafe { ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"E76: Too many [\0") };
#[no_mangle]
pub static mut e_toomany: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E77: Too many file names\0")
};
#[no_mangle]
pub static mut e_trailing: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E488: Trailing characters\0")
};
#[no_mangle]
pub static mut e_trailing_arg: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E488: Trailing characters: %s\0",
    )
};
#[no_mangle]
pub static mut e_umark: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"E78: Unknown mark\0")
};
#[no_mangle]
pub static mut e_wildexpand: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E79: Cannot expand wildcards\0",
    )
};
#[no_mangle]
pub static mut e_winheight: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"E591: 'winheight' cannot be smaller than 'winminheight'\0",
    )
};
#[no_mangle]
pub static mut e_winwidth: [::core::ffi::c_char; 54] = unsafe {
    ::core::mem::transmute::<[u8; 54], [::core::ffi::c_char; 54]>(
        *b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0",
    )
};
#[no_mangle]
pub static mut e_write: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E80: Error while writing\0")
};
#[no_mangle]
pub static mut e_zerocount: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E939: Positive count required\0",
    )
};
#[no_mangle]
pub static mut e_usingsid: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E81: Using <SID> not in a script context\0",
    )
};
#[no_mangle]
pub static mut e_missingparen: [::core::ffi::c_char; 30] = unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E107: Missing parentheses: %s\0",
    )
};
#[no_mangle]
pub static mut e_empty_buffer: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E749: Empty buffer\0")
};
#[no_mangle]
pub static mut e_nobufnr: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E86: Buffer %ld does not exist\0",
    )
};
#[no_mangle]
pub static mut e_no_write_since_last_change: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E37: No write since last change\0",
    )
};
#[no_mangle]
pub static mut e_no_write_since_last_change_add_bang_to_override: [::core::ffi::c_char; 52] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"E37: No write since last change (add ! to override)\0",
    )
};
#[no_mangle]
pub static mut e_no_write_since_last_change_for_buffer_nr_add_bang_to_override:
    [::core::ffi::c_char; 66] = unsafe {
    ::core::mem::transmute::<[u8; 66], [::core::ffi::c_char; 66]>(
        *b"E89: No write since last change for buffer %d (add ! to override)\0",
    )
};
#[no_mangle]
pub static mut e_buffer_nr_not_found: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E92: Buffer %d not found\0")
};
#[no_mangle]
pub static mut e_unknown_function_str: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E117: Unknown function: %s\0")
};
#[no_mangle]
pub static mut e_str_not_inside_function: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E193: %s not inside a function\0",
    )
};
#[no_mangle]
pub static mut e_job_still_running: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E948: Job still running\0")
};
#[no_mangle]
pub static mut e_job_still_running_add_bang_to_end_the_job: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"E948: Job still running (add ! to end the job)\0",
    )
};
#[no_mangle]
pub static mut e_invalpat: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E682: Invalid search pattern or delimiter\0",
    )
};
#[no_mangle]
pub static mut e_bufloaded: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E139: File is loaded in another buffer\0",
    )
};
#[no_mangle]
pub static mut e_notset: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E764: Option '%s' is not set\0",
    )
};
#[no_mangle]
pub static mut e_invalidreg: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E850: Invalid register name\0")
};
#[no_mangle]
pub static mut e_dirnotf: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"E919: Directory not found in '%s': \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_au_recursive: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E952: Autocommand caused recursive behavior\0",
    )
};
#[no_mangle]
pub static mut e_menu_only_exists_in_another_mode: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E328: Menu only exists in another mode\0",
    )
};
#[no_mangle]
pub static mut e_autocmd_close: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E813: Cannot close autocmd window\0",
    )
};
#[no_mangle]
pub static mut e_list_index_out_of_range_nr: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E684: List index out of range: %ld\0",
    )
};
#[no_mangle]
pub static mut e_listarg: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E686: Argument of %s must be a List\0",
    )
};
#[no_mangle]
pub static mut e_unsupportedoption: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E519: Option not supported\0")
};
#[no_mangle]
pub static mut e_fnametoolong: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E856: Filename too long\0")
};
#[no_mangle]
pub static mut e_using_float_as_string: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E806: Using a Float as a String\0",
    )
};
#[no_mangle]
pub static mut e_cannot_edit_other_buf: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E788: Not allowed to edit another buffer now\0",
    )
};
#[no_mangle]
pub static mut e_using_number_as_bool_nr: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E1023: Using a Number as a Bool: %d\0",
    )
};
#[no_mangle]
pub static mut e_not_callable_type_str: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E1085: Not a callable type: %s\0",
    )
};
#[no_mangle]
pub static mut e_auabort: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E855: Autocommands caused command to abort\0",
    )
};
#[no_mangle]
pub static mut e_api_error: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"E5555: API call: %s\0")
};
#[no_mangle]
pub static mut e_fast_api_disabled: [::core::ffi::c_char; 53] = unsafe {
    ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
        *b"E5560: %s must not be called in a fast event context\0",
    )
};
#[no_mangle]
pub static mut e_floatonly: [::core::ffi::c_char; 62] = unsafe {
    ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
        *b"E5601: Cannot close window, only floating window would remain\0",
    )
};
#[no_mangle]
pub static mut e_floatexchange: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E5602: Cannot exchange or rotate float\0",
    )
};
#[no_mangle]
pub static mut e_cant_find_directory_str_in_cdpath: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"E344: Can't find directory \"%s\" in cdpath\0",
    )
};
#[no_mangle]
pub static mut e_cant_find_file_str_in_path: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E345: Can't find file \"%s\" in path\0",
    )
};
#[no_mangle]
pub static mut e_no_more_directory_str_found_in_cdpath: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E346: No more directory \"%s\" found in cdpath\0",
    )
};
#[no_mangle]
pub static mut e_no_more_file_str_found_in_path: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E347: No more file \"%s\" found in path\0",
    )
};
#[no_mangle]
pub static mut e_value_is_locked: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E741: Value is locked\0")
};
#[no_mangle]
pub static mut e_value_is_locked_str: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E741: Value is locked: %.*s\0")
};
#[no_mangle]
pub static mut e_cannot_change_value: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E742: Cannot change value\0")
};
#[no_mangle]
pub static mut e_cannot_change_value_of_str: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E742: Cannot change value of %.*s\0",
    )
};
#[no_mangle]
pub static mut e_cannot_set_variable_in_sandbox_str: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"E794: Cannot set variable in the sandbox: \"%.*s\"\0",
    )
};
#[no_mangle]
pub static mut e_cannot_delete_variable_str: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"E795: Cannot delete variable %.*s\0",
    )
};
#[no_mangle]
pub static mut e_invalwindow: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(*b"E957: Invalid window number\0")
};
#[no_mangle]
pub static mut e_problem_creating_internal_diff: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E960: Problem creating the internal diff\0",
    )
};
#[no_mangle]
pub static mut e_cannot_define_autocommands_for_all_events: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
        *b"E1155: Cannot define autocommands for ALL events\0",
    )
};
#[no_mangle]
pub static mut e_cannot_change_arglist_recursively: [::core::ffi::c_char; 51] = unsafe {
    ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
        *b"E1156: Cannot change the argument list recursively\0",
    )
};
#[no_mangle]
pub static mut e_resulting_text_too_long: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E1240: Resulting text too long\0",
    )
};
#[no_mangle]
pub static mut e_line_number_out_of_range: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"E1247: Line number out of range\0",
    )
};
#[no_mangle]
pub static mut e_highlight_group_name_invalid_char: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E5248: Invalid character in group name\0",
    )
};
#[no_mangle]
pub static mut e_highlight_group_name_too_long: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E1249: Highlight group name too long\0",
    )
};
#[no_mangle]
pub static mut e_string_required: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(*b"E928: String required\0")
};
#[no_mangle]
pub static mut e_invalid_column_number_nr: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"E964: Invalid column number: %ld\0",
    )
};
#[no_mangle]
pub static mut e_invalid_line_number_nr: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"E966: Invalid line number: %ld\0",
    )
};
#[no_mangle]
pub static mut e_reduce_of_an_empty_str_with_no_initial_value: [::core::ffi::c_char; 50] = unsafe {
    ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
        *b"E998: Reduce of an empty %s with no initial value\0",
    )
};
#[no_mangle]
pub static mut e_invalid_value_for_blob_nr: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E1239: Invalid value for blob: 0xlX\0",
    )
};
#[no_mangle]
pub static mut e_stray_closing_curly_str: [::core::ffi::c_char; 44] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E1278: Stray '}' without a matching '{': %s\0",
    )
};
#[no_mangle]
pub static mut e_missing_close_curly_str: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"E1279: Missing '}': %s\0")
};
#[no_mangle]
pub static mut e_cannot_change_menus_while_listing: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E1310: Cannot change menus while listing\0",
    )
};
#[no_mangle]
pub static mut e_not_allowed_to_change_window_layout_in_this_autocmd: [::core::ffi::c_char; 63] = unsafe {
    ::core::mem::transmute::<[u8; 63], [::core::ffi::c_char; 63]>(
        *b"E1312: Not allowed to change the window layout in this autocmd\0",
    )
};
#[no_mangle]
pub static mut e_val_too_large: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(*b"E1510: Value too large: %s\0")
};
#[no_mangle]
pub static mut e_val_too_large_len: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E1510: Value too large: %.*s\0",
    )
};
#[no_mangle]
pub static mut e_undobang_cannot_redo_or_move_branch: [::core::ffi::c_char; 68] = unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"E5767: Cannot use :undo! to redo or move to a different undo branch\0",
    )
};
#[no_mangle]
pub static mut e_winfixbuf_cannot_go_to_buffer: [::core::ffi::c_char; 52] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"E1513: Cannot switch buffer. 'winfixbuf' is enabled\0",
    )
};
#[no_mangle]
pub static mut e_invalid_return_type_from_findfunc: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E1514: 'findfunc' did not return a List type\0",
    )
};
#[no_mangle]
pub static mut e_cannot_switch_to_a_closing_buffer: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E1546: Cannot switch to a closing buffer\0",
    )
};
#[no_mangle]
pub static mut e_cannot_have_more_than_nr_diff_anchors: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E1549: Cannot have more than %d diff anchors\0",
    )
};
#[no_mangle]
pub static mut e_failed_to_find_all_diff_anchors: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"E1550: Failed to find all diff anchors\0",
    )
};
#[no_mangle]
pub static mut e_diff_anchors_with_hidden_windows: [::core::ffi::c_char; 60] = unsafe {
    ::core::mem::transmute::<[u8; 60], [::core::ffi::c_char; 60]>(
        *b"E1562: Diff anchors cannot be used with hidden diff windows\0",
    )
};
#[no_mangle]
pub static mut e_leadtab_requires_tab: [::core::ffi::c_char; 66] = unsafe {
    ::core::mem::transmute::<[u8; 66], [::core::ffi::c_char; 66]>(
        *b"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified\0",
    )
};
#[no_mangle]
pub static mut e_invalid_format_string_single_percent_s: [::core::ffi::c_char; 55] = unsafe {
    ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
        *b"E1577: Invalid format string, only one \"%s\" is allowed\0",
    )
};
#[no_mangle]
pub static mut e_trustfile: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E5570: Cannot update trust file: %s\0",
    )
};
#[no_mangle]
pub static mut e_cannot_read_from_str_2: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(
        *b"E282: Cannot read from \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_conflicting_configs: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"E5422: Conflicting configs: \"%s\" \"%s\"\0",
    )
};
#[no_mangle]
pub static mut e_unknown_option2: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(*b"E355: Unknown option: %s\0")
};
#[no_mangle]
pub static mut top_bot_msg: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"search hit TOP, continuing at BOTTOM\0",
    )
};
#[no_mangle]
pub static mut bot_top_msg: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"search hit BOTTOM, continuing at TOP\0",
    )
};
#[no_mangle]
pub static mut line_msg: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b" line \0") };
#[no_mangle]
pub static mut EVALARG_EVALUATE: evalarg_T = evalarg_T {
    eval_flags: EVAL_EVALUATE as ::core::ffi::c_int,
    eval_getline: None,
    eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
#[no_mangle]
pub static mut msg_ext_need_clear: bool = false;
#[no_mangle]
pub static mut msg_ext_skip_flush: bool = false;
#[no_mangle]
pub static mut msg_ext_overwrite: bool = false;
#[no_mangle]
pub static mut msg_ext_skip_verbose: bool = false;
#[no_mangle]
pub static mut msg_grid: ScreenGrid = ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
};
#[no_mangle]
pub static mut msg_grid_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut msg_grid_adj: GridView = GridView {
    target: ::core::ptr::null_mut::<ScreenGrid>(),
    row_offset: 0,
    col_offset: 0,
};
#[no_mangle]
pub static mut msg_scrolled_at_flush: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut msg_grid_scroll_discount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut msg_listdo_overwrite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
#[no_mangle]
pub static mut kTVCstring: size_t = 0;
#[no_mangle]
pub static mut kTVTranslate: size_t = 18446744073709551615 as size_t;
#[no_mangle]
pub static mut disable_fold_update: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut test_disable_char_avail: bool = false;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SYS_VIMRC_FILE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"$VIM/sysinit.vim\0")
};
pub const VIMRC_FILE: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b".nvimrc\0") };
#[no_mangle]
pub static mut g_stats: nvim_stats_s = nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
};
pub const NO_BUFFERS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut Rows: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
#[no_mangle]
pub static mut Columns: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
#[no_mangle]
pub static mut mod_mask: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut vgetc_mod_mask: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut vgetc_char: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmdline_row: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut redraw_cmdline: bool = false;
#[no_mangle]
pub static mut redraw_mode: bool = false;
#[no_mangle]
pub static mut clear_cmdline: bool = false;
#[no_mangle]
pub static mut mode_displayed: bool = false;
#[no_mangle]
pub static mut cmdline_star: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut redrawing_cmdline: bool = false;
#[no_mangle]
pub static mut cmdline_was_last_drawn: bool = false;
#[no_mangle]
pub static mut exec_from_reg: bool = false;
#[no_mangle]
pub static mut dollar_vcol: colnr_T = -1 as colnr_T;
#[no_mangle]
pub static mut edit_submode: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut edit_submode_pre: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut edit_submode_extra: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut edit_submode_highl: hlf_T = HLF_NONE;
#[no_mangle]
pub static mut cmdmsg_rl: bool = false;
#[no_mangle]
pub static mut msg_col: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut msg_row: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut msg_scrolled: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut msg_scrolled_ign: bool = false;
#[no_mangle]
pub static mut msg_did_scroll: bool = false;
#[no_mangle]
pub static mut keep_msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut keep_msg_hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut need_fileinfo: bool = false;
#[no_mangle]
pub static mut msg_scroll: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut msg_didout: bool = false;
#[no_mangle]
pub static mut msg_didany: bool = false;
#[no_mangle]
pub static mut msg_nowait: bool = false;
#[no_mangle]
pub static mut emsg_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut info_message: bool = false;
#[no_mangle]
pub static mut msg_hist_off: bool = false;
#[no_mangle]
pub static mut need_clr_eos: bool = false;
#[no_mangle]
pub static mut emsg_skip: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut emsg_severe: bool = false;
#[no_mangle]
pub static mut emsg_assert_fails_msg: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut emsg_assert_fails_lnum: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
#[no_mangle]
pub static mut emsg_assert_fails_context: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut did_endif: bool = false;
#[no_mangle]
pub static mut did_emsg: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut called_vim_beep: bool = false;
#[no_mangle]
pub static mut did_emsg_syntax: bool = false;
#[no_mangle]
pub static mut called_emsg: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut ex_exitval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut emsg_on_display: bool = false;
#[no_mangle]
pub static mut rc_did_emsg: bool = false;
#[no_mangle]
pub static mut no_wait_return: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut need_wait_return: bool = false;
#[no_mangle]
pub static mut did_wait_return: bool = false;
#[no_mangle]
pub static mut need_maketitle: bool = true;
#[no_mangle]
pub static mut quit_more: bool = false;
#[no_mangle]
pub static mut vgetc_busy: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut didset_vim: bool = false;
#[no_mangle]
pub static mut didset_vimruntime: bool = false;
#[no_mangle]
pub static mut lines_left: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut msg_no_more: bool = false;
#[no_mangle]
pub static mut ex_nesting_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut debug_break_level: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut debug_did_msg: bool = false;
#[no_mangle]
pub static mut debug_tick: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut debug_backtrace_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut do_profiling: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut current_exception: *mut except_T = ::core::ptr::null_mut::<except_T>();
#[no_mangle]
pub static mut did_throw: bool = false;
#[no_mangle]
pub static mut need_rethrow: bool = false;
#[no_mangle]
pub static mut check_cstack: bool = false;
#[no_mangle]
pub static mut trylevel: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut force_abort: bool = false;
#[no_mangle]
pub static mut msg_list: *mut *mut msglist_T = ::core::ptr::null_mut::<*mut msglist_T>();
#[no_mangle]
pub static mut suppress_errthrow: bool = false;
#[no_mangle]
pub static mut caught_stack: *mut except_T = ::core::ptr::null_mut::<except_T>();
#[no_mangle]
pub static mut may_garbage_collect: bool = false;
#[no_mangle]
pub static mut want_garbage_collect: bool = false;
#[no_mangle]
pub static mut garbage_collect_at_exit: bool = false;
pub const SID_CMDARG: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const SID_CARG: ::core::ffi::c_int = -3 as ::core::ffi::c_int;
pub const SID_ENV: ::core::ffi::c_int = -4 as ::core::ffi::c_int;
#[no_mangle]
pub static mut current_sctx: sctx_T = sctx_T {
    sc_sid: 0 as scid_T,
    sc_seq: 0 as ::core::ffi::c_int,
    sc_lnum: 0 as linenr_T,
    sc_chan: 0 as uint64_t,
};
#[no_mangle]
pub static mut current_ui: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut did_source_packages: bool = false;
#[no_mangle]
pub static mut provider_caller_scope: caller_scope = caller_scope {
    script_ctx: sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    },
    es_entry: estack_T {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        es_type: ETYPE_TOP,
        es_info: C2Rust_Unnamed_43 {
            sctx: ::core::ptr::null_mut::<sctx_T>(),
        },
    },
    autocmd_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    autocmd_match: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: ::core::ptr::null_mut::<::core::ffi::c_void>(),
};
#[no_mangle]
pub static mut provider_call_nesting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut t_colors: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
#[no_mangle]
pub static mut include_none: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut include_default: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut include_link: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut highlight_match: bool = false;
#[no_mangle]
pub static mut search_match_lines: linenr_T = 0;
#[no_mangle]
pub static mut search_match_endcol: colnr_T = 0;
#[no_mangle]
pub static mut search_first_line: linenr_T = 0 as linenr_T;
#[no_mangle]
pub static mut search_last_line: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
#[no_mangle]
pub static mut no_smartcase: bool = false;
#[no_mangle]
pub static mut need_check_timestamps: bool = false;
#[no_mangle]
pub static mut did_check_timestamps: bool = false;
#[no_mangle]
pub static mut no_check_timestamps: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut mouse_grid: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut mouse_row: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut mouse_col: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut mouse_past_bottom: bool = false;
#[no_mangle]
pub static mut mouse_past_eol: bool = false;
#[no_mangle]
pub static mut mouse_dragging: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut root_menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
#[no_mangle]
pub static mut sys_menu: bool = false;
#[no_mangle]
pub static mut firstwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut lastwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut prevwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut curwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut topframe: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
#[no_mangle]
pub static mut first_tabpage: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
#[no_mangle]
pub static mut curtab: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
#[no_mangle]
pub static mut lastused_tabpage: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
#[no_mangle]
pub static mut redraw_tabline: bool = false;
#[no_mangle]
pub static mut firstbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
#[no_mangle]
pub static mut lastbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
#[no_mangle]
pub static mut curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
#[no_mangle]
pub static mut global_alist: alist_T = alist_T {
    al_ga: garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    },
    al_refcount: 0,
    id: 0,
};
#[no_mangle]
pub static mut max_alist_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut arg_had_last: bool = false;
#[no_mangle]
pub static mut ru_col: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut ru_wid: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut sc_col: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut starting: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut exiting: bool = false;
#[no_mangle]
pub static mut v_dying: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut stdin_isatty: bool = true;
#[no_mangle]
pub static mut stdout_isatty: bool = true;
#[no_mangle]
pub static mut stderr_isatty: bool = true;
#[no_mangle]
pub static mut stdin_fd: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut full_screen: bool = false;
#[no_mangle]
pub static mut secure: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut textlock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut allbuf_lock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut sandbox: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut silent_mode: bool = false;
#[no_mangle]
pub static mut VIsual: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
#[no_mangle]
pub static mut VIsual_active: bool = false;
#[no_mangle]
pub static mut VIsual_select: bool = false;
#[no_mangle]
pub static mut VIsual_select_reg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut VIsual_select_exclu_adj: bool = false;
#[no_mangle]
pub static mut restart_VIsual_select: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut VIsual_reselect: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut VIsual_mode: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
#[no_mangle]
pub static mut redo_VIsual_busy: bool = false;
#[no_mangle]
pub static mut resel_VIsual_mode: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[no_mangle]
pub static mut resel_VIsual_line_count: linenr_T = 0;
#[no_mangle]
pub static mut resel_VIsual_vcol: colnr_T = 0;
#[no_mangle]
pub static mut where_paste_started: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
#[no_mangle]
pub static mut did_ai: bool = false;
#[no_mangle]
pub static mut ai_col: colnr_T = 0 as colnr_T;
#[no_mangle]
pub static mut end_comment_pending: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[no_mangle]
pub static mut did_syncbind: bool = false;
#[no_mangle]
pub static mut did_si: bool = false;
#[no_mangle]
pub static mut can_si: bool = false;
#[no_mangle]
pub static mut can_si_back: bool = false;
#[no_mangle]
pub static mut old_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut saved_cursor: pos_T = pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
};
#[no_mangle]
pub static mut Insstart: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
#[no_mangle]
pub static mut Insstart_orig: pos_T = pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
};
#[no_mangle]
pub static mut orig_line_count: linenr_T = 0 as linenr_T;
#[no_mangle]
pub static mut vr_lines_changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut inhibit_delete_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut fenc_default: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut State: ::core::ffi::c_int = MODE_NORMAL as ::core::ffi::c_int;
#[no_mangle]
pub static mut debug_mode: bool = false;
#[no_mangle]
pub static mut finish_op: bool = false;
#[no_mangle]
pub static mut opcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut motion_force: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut exmode_active: bool = false;
#[no_mangle]
pub static mut pending_exmode_active: bool = false;
#[no_mangle]
pub static mut ex_no_reprint: bool = false;
#[no_mangle]
pub static mut cmdpreview: bool = false;
#[no_mangle]
pub static mut reg_recording: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut reg_executing: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut pending_end_reg_executing: bool = false;
#[no_mangle]
pub static mut reg_recorded: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut no_mapping: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut no_zero_mapping: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut allow_keys: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut no_u_sync: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut u_sync_once: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut force_restart_edit: bool = false;
#[no_mangle]
pub static mut restart_edit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut arrow_used: bool = false;
#[no_mangle]
pub static mut ins_at_eol: bool = false;
#[no_mangle]
pub static mut no_abbr: bool = true;
#[no_mangle]
pub static mut mapped_ctrl_c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut ctrl_c_interrupts: bool = true;
#[no_mangle]
pub static mut cmdmod: cmdmod_T = cmdmod_T {
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
#[no_mangle]
pub static mut msg_silent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut emsg_silent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut emsg_noredir: bool = false;
#[no_mangle]
pub static mut cmd_silent: bool = false;
#[no_mangle]
pub static mut in_assert_fails: bool = false;
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_DIALOG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut swap_exists_action: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut swap_exists_did_quit: bool = false;
#[no_mangle]
pub static mut IObuff: [::core::ffi::c_char; 1025] = [0; 1025];
#[no_mangle]
pub static mut NameBuff: [::core::ffi::c_char; 4096] = [0; 4096];
#[no_mangle]
pub static mut msg_buf: [::core::ffi::c_char; 480] = [0; 480];
#[no_mangle]
pub static mut os_buf: [::core::ffi::c_char; 4096] = [0; 4096];
#[no_mangle]
pub static mut RedrawingDisabled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut readonlymode: bool = false;
#[no_mangle]
pub static mut recoverymode: bool = false;
#[no_mangle]
pub static mut typebuf: typebuf_T = typebuf_T {
    tb_buf: ::core::ptr::null_mut::<uint8_t>(),
    tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
    tb_buflen: 0 as ::core::ffi::c_int,
    tb_off: 0 as ::core::ffi::c_int,
    tb_len: 0 as ::core::ffi::c_int,
    tb_maplen: 0 as ::core::ffi::c_int,
    tb_silent: 0 as ::core::ffi::c_int,
    tb_no_abbr_cnt: 0 as ::core::ffi::c_int,
    tb_change_cnt: 0 as ::core::ffi::c_int,
};
#[no_mangle]
pub static mut typebuf_was_empty: bool = false;
#[no_mangle]
pub static mut ex_normal_busy: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut expr_map_lock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut ignore_script: bool = false;
#[no_mangle]
pub static mut stop_insert_mode: bool = false;
#[no_mangle]
pub static mut KeyTyped: bool = false;
#[no_mangle]
pub static mut KeyStuffed: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut maptick: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut must_redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut skip_redraw: bool = false;
#[no_mangle]
pub static mut do_redraw: bool = false;
#[no_mangle]
pub static mut must_redraw_pum: bool = false;
#[no_mangle]
pub static mut need_highlight_changed: bool = true;
#[no_mangle]
pub static mut scriptout: *mut FILE = ::core::ptr::null_mut::<FILE>();
#[no_mangle]
pub static mut got_int: bool = false;
#[no_mangle]
pub static mut bangredo: bool = false;
#[no_mangle]
pub static mut searchcmdlen: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut reg_do_extmatch: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut re_extmatch_in: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
#[no_mangle]
pub static mut re_extmatch_out: *mut reg_extmatch_T = ::core::ptr::null_mut::<reg_extmatch_T>();
#[no_mangle]
pub static mut did_outofmem_msg: bool = false;
#[no_mangle]
pub static mut did_swapwrite_msg: bool = false;
#[no_mangle]
pub static mut global_busy: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut listcmd_busy: bool = false;
#[no_mangle]
pub static mut need_start_insertmode: bool = false;
#[no_mangle]
pub static mut last_mode: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"n\0\0\0") };
#[no_mangle]
pub static mut last_cmdline: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut repeat_cmdline: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut new_last_cmdline: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut postponed_split: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut postponed_split_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut postponed_split_tab: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_do_tagpreview: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_tag_at_cursor: bool = false;
#[no_mangle]
pub static mut replace_offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut escape_chars: *mut ::core::ffi::c_char =
    b" \t\\\"|\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut keep_help_flag: bool = false;
#[no_mangle]
pub static mut redir_off: bool = false;
#[no_mangle]
pub static mut redir_fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
#[no_mangle]
pub static mut redir_reg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut redir_vname: bool = false;
#[no_mangle]
pub static mut capture_ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
#[no_mangle]
pub static mut langmap_mapchar: [uint8_t; 256] = [0; 256];
#[no_mangle]
pub static mut save_p_ls: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut save_p_wmh: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut wild_menu_showing: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut globaldir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut last_chdir_reason: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut km_stopsel: bool = false;
#[no_mangle]
pub static mut km_startsel: bool = false;
#[no_mangle]
pub static mut cmdwin_type: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmdwin_result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmdwin_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmdwin_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
#[no_mangle]
pub static mut cmdwin_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut cmdwin_old_curwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut cmdline_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
#[no_mangle]
pub static mut no_lines_msg: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"--No lines in buffer--\0")
};
#[no_mangle]
pub static mut sub_nsubs: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut sub_nlines: linenr_T = 0;
#[no_mangle]
pub static mut wim_flags: [uint8_t; 4] = [0; 4];
#[no_mangle]
pub static mut stl_syntax: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut no_hlsearch: bool = false;
#[no_mangle]
pub static mut typebuf_was_filled: bool = false;
#[no_mangle]
pub static mut virtual_op: TriState = kNone;
#[no_mangle]
pub static mut display_tick: disptick_T = 0 as disptick_T;
#[no_mangle]
pub static mut spell_redraw_lnum: linenr_T = 0 as linenr_T;
#[no_mangle]
pub static mut time_fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
#[no_mangle]
pub static mut vim_ignored: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut embedded_mode: bool = false;
#[no_mangle]
pub static mut headless_mode: bool = false;
#[no_mangle]
pub static mut windowsVersion: [::core::ffi::c_char; 20] = [
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
];
#[no_mangle]
pub static mut magic_overruled: optmagic_T = OPTION_MAGIC_NOT_SET;
#[no_mangle]
pub static mut skip_win_fix_cursor: bool = false;
#[no_mangle]
pub static mut skip_win_fix_scroll: bool = false;
#[no_mangle]
pub static mut skip_update_topline: bool = false;
#[no_mangle]
pub static mut default_grid: ScreenGrid = ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
};
#[no_mangle]
pub static mut default_gridview: GridView = unsafe {
    GridView {
        target: &raw const default_grid as *mut ScreenGrid,
        row_offset: 0,
        col_offset: 0,
    }
};
#[no_mangle]
pub static mut resizing_screen: bool = false;
#[no_mangle]
pub static mut linebuf_char: *mut schar_T = ::core::ptr::null_mut::<schar_T>();
#[no_mangle]
pub static mut linebuf_attr: *mut sattr_T = ::core::ptr::null_mut::<sattr_T>();
#[no_mangle]
pub static mut linebuf_vcol: *mut colnr_T = ::core::ptr::null_mut::<colnr_T>();
#[no_mangle]
pub static mut linebuf_scratch: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut opt_ambw_values: [*const ::core::ffi::c_char; 3] = [
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bg_values: [*const ::core::ffi::c_char; 3] = [
    b"light\0".as_ptr() as *const ::core::ffi::c_char,
    b"dark\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bs_values: [*const ::core::ffi::c_char; 5] = [
    b"indent\0".as_ptr() as *const ::core::ffi::c_char,
    b"eol\0".as_ptr() as *const ::core::ffi::c_char,
    b"start\0".as_ptr() as *const ::core::ffi::c_char,
    b"nostop\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bkc_values: [*const ::core::ffi::c_char; 6] = [
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    b"breaksymlink\0".as_ptr() as *const ::core::ffi::c_char,
    b"breakhardlink\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bo_values: [*const ::core::ffi::c_char; 21] = [
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"backspace\0".as_ptr() as *const ::core::ffi::c_char,
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"complete\0".as_ptr() as *const ::core::ffi::c_char,
    b"copy\0".as_ptr() as *const ::core::ffi::c_char,
    b"ctrlg\0".as_ptr() as *const ::core::ffi::c_char,
    b"error\0".as_ptr() as *const ::core::ffi::c_char,
    b"esc\0".as_ptr() as *const ::core::ffi::c_char,
    b"ex\0".as_ptr() as *const ::core::ffi::c_char,
    b"hangul\0".as_ptr() as *const ::core::ffi::c_char,
    b"insertmode\0".as_ptr() as *const ::core::ffi::c_char,
    b"lang\0".as_ptr() as *const ::core::ffi::c_char,
    b"mess\0".as_ptr() as *const ::core::ffi::c_char,
    b"showmatch\0".as_ptr() as *const ::core::ffi::c_char,
    b"operator\0".as_ptr() as *const ::core::ffi::c_char,
    b"register\0".as_ptr() as *const ::core::ffi::c_char,
    b"shell\0".as_ptr() as *const ::core::ffi::c_char,
    b"spell\0".as_ptr() as *const ::core::ffi::c_char,
    b"term\0".as_ptr() as *const ::core::ffi::c_char,
    b"wildmode\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_briopt_values: [*const ::core::ffi::c_char; 6] = [
    b"shift:\0".as_ptr() as *const ::core::ffi::c_char,
    b"min:\0".as_ptr() as *const ::core::ffi::c_char,
    b"sbr\0".as_ptr() as *const ::core::ffi::c_char,
    b"list:\0".as_ptr() as *const ::core::ffi::c_char,
    b"column:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bh_values: [*const ::core::ffi::c_char; 6] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"hide\0".as_ptr() as *const ::core::ffi::c_char,
    b"unload\0".as_ptr() as *const ::core::ffi::c_char,
    b"delete\0".as_ptr() as *const ::core::ffi::c_char,
    b"wipe\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_bt_values: [*const ::core::ffi::c_char; 9] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"acwrite\0".as_ptr() as *const ::core::ffi::c_char,
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"nofile\0".as_ptr() as *const ::core::ffi::c_char,
    b"nowrite\0".as_ptr() as *const ::core::ffi::c_char,
    b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
    b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
    b"prompt\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_cmp_values: [*const ::core::ffi::c_char; 3] = [
    b"internal\0".as_ptr() as *const ::core::ffi::c_char,
    b"keepascii\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_cb_values: [*const ::core::ffi::c_char; 3] = [
    b"unnamed\0".as_ptr() as *const ::core::ffi::c_char,
    b"unnamedplus\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_cpt_values: [*const ::core::ffi::c_char; 16] = [
    b".\0".as_ptr() as *const ::core::ffi::c_char,
    b"w\0".as_ptr() as *const ::core::ffi::c_char,
    b"b\0".as_ptr() as *const ::core::ffi::c_char,
    b"u\0".as_ptr() as *const ::core::ffi::c_char,
    b"k\0".as_ptr() as *const ::core::ffi::c_char,
    b"kspell\0".as_ptr() as *const ::core::ffi::c_char,
    b"s\0".as_ptr() as *const ::core::ffi::c_char,
    b"i\0".as_ptr() as *const ::core::ffi::c_char,
    b"d\0".as_ptr() as *const ::core::ffi::c_char,
    b"]\0".as_ptr() as *const ::core::ffi::c_char,
    b"t\0".as_ptr() as *const ::core::ffi::c_char,
    b"U\0".as_ptr() as *const ::core::ffi::c_char,
    b"f\0".as_ptr() as *const ::core::ffi::c_char,
    b"F\0".as_ptr() as *const ::core::ffi::c_char,
    b"o\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_cot_values: [*const ::core::ffi::c_char; 12] = [
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    b"menuone\0".as_ptr() as *const ::core::ffi::c_char,
    b"longest\0".as_ptr() as *const ::core::ffi::c_char,
    b"preview\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup\0".as_ptr() as *const ::core::ffi::c_char,
    b"noinsert\0".as_ptr() as *const ::core::ffi::c_char,
    b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
    b"fuzzy\0".as_ptr() as *const ::core::ffi::c_char,
    b"nosort\0".as_ptr() as *const ::core::ffi::c_char,
    b"preinsert\0".as_ptr() as *const ::core::ffi::c_char,
    b"nearest\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_csl_values: [*const ::core::ffi::c_char; 4] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"slash\0".as_ptr() as *const ::core::ffi::c_char,
    b"backslash\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_culopt_values: [*const ::core::ffi::c_char; 5] = [
    b"line\0".as_ptr() as *const ::core::ffi::c_char,
    b"screenline\0".as_ptr() as *const ::core::ffi::c_char,
    b"number\0".as_ptr() as *const ::core::ffi::c_char,
    b"both\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_debug_values: [*const ::core::ffi::c_char; 4] = [
    b"msg\0".as_ptr() as *const ::core::ffi::c_char,
    b"throw\0".as_ptr() as *const ::core::ffi::c_char,
    b"beep\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_dip_values: [*const ::core::ffi::c_char; 20] = [
    b"filler\0".as_ptr() as *const ::core::ffi::c_char,
    b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
    b"context:\0".as_ptr() as *const ::core::ffi::c_char,
    b"iblank\0".as_ptr() as *const ::core::ffi::c_char,
    b"icase\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhite\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhiteall\0".as_ptr() as *const ::core::ffi::c_char,
    b"iwhiteeol\0".as_ptr() as *const ::core::ffi::c_char,
    b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
    b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
    b"closeoff\0".as_ptr() as *const ::core::ffi::c_char,
    b"hiddenoff\0".as_ptr() as *const ::core::ffi::c_char,
    b"foldcolumn:\0".as_ptr() as *const ::core::ffi::c_char,
    b"followwrap\0".as_ptr() as *const ::core::ffi::c_char,
    b"internal\0".as_ptr() as *const ::core::ffi::c_char,
    b"indent-heuristic\0".as_ptr() as *const ::core::ffi::c_char,
    b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
    b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
    b"linematch:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_dip_algorithm_values: [*const ::core::ffi::c_char; 5] = [
    b"myers\0".as_ptr() as *const ::core::ffi::c_char,
    b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
    b"patience\0".as_ptr() as *const ::core::ffi::c_char,
    b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_dip_inline_values: [*const ::core::ffi::c_char; 5] = [
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    b"simple\0".as_ptr() as *const ::core::ffi::c_char,
    b"char\0".as_ptr() as *const ::core::ffi::c_char,
    b"word\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_dy_values: [*const ::core::ffi::c_char; 5] = [
    b"lastline\0".as_ptr() as *const ::core::ffi::c_char,
    b"truncate\0".as_ptr() as *const ::core::ffi::c_char,
    b"uhex\0".as_ptr() as *const ::core::ffi::c_char,
    b"msgsep\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_ead_values: [*const ::core::ffi::c_char; 4] = [
    b"both\0".as_ptr() as *const ::core::ffi::c_char,
    b"ver\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_ff_values: [*const ::core::ffi::c_char; 4] = [
    b"unix\0".as_ptr() as *const ::core::ffi::c_char,
    b"dos\0".as_ptr() as *const ::core::ffi::c_char,
    b"mac\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_fcl_values: [*const ::core::ffi::c_char; 2] = [
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_fdc_values: [*const ::core::ffi::c_char; 21] = [
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"0\0".as_ptr() as *const ::core::ffi::c_char,
    b"1\0".as_ptr() as *const ::core::ffi::c_char,
    b"2\0".as_ptr() as *const ::core::ffi::c_char,
    b"3\0".as_ptr() as *const ::core::ffi::c_char,
    b"4\0".as_ptr() as *const ::core::ffi::c_char,
    b"5\0".as_ptr() as *const ::core::ffi::c_char,
    b"6\0".as_ptr() as *const ::core::ffi::c_char,
    b"7\0".as_ptr() as *const ::core::ffi::c_char,
    b"8\0".as_ptr() as *const ::core::ffi::c_char,
    b"9\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_fdm_values: [*const ::core::ffi::c_char; 7] = [
    b"manual\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr\0".as_ptr() as *const ::core::ffi::c_char,
    b"marker\0".as_ptr() as *const ::core::ffi::c_char,
    b"indent\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
    b"diff\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_fdo_values: [*const ::core::ffi::c_char; 12] = [
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"block\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    b"mark\0".as_ptr() as *const ::core::ffi::c_char,
    b"percent\0".as_ptr() as *const ::core::ffi::c_char,
    b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
    b"search\0".as_ptr() as *const ::core::ffi::c_char,
    b"tag\0".as_ptr() as *const ::core::ffi::c_char,
    b"insert\0".as_ptr() as *const ::core::ffi::c_char,
    b"undo\0".as_ptr() as *const ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_icm_values: [*const ::core::ffi::c_char; 4] = [
    b"nosplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"split\0".as_ptr() as *const ::core::ffi::c_char,
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_jop_values: [*const ::core::ffi::c_char; 4] = [
    b"stack\0".as_ptr() as *const ::core::ffi::c_char,
    b"view\0".as_ptr() as *const ::core::ffi::c_char,
    b"clean\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_km_values: [*const ::core::ffi::c_char; 3] = [
    b"startsel\0".as_ptr() as *const ::core::ffi::c_char,
    b"stopsel\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_lop_values: [*const ::core::ffi::c_char; 3] = [
    b"expr:0\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr:1\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_mopt_values: [*const ::core::ffi::c_char; 5] = [
    b"hit-enter\0".as_ptr() as *const ::core::ffi::c_char,
    b"wait:\0".as_ptr() as *const ::core::ffi::c_char,
    b"history:\0".as_ptr() as *const ::core::ffi::c_char,
    b"progress:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_mousem_values: [*const ::core::ffi::c_char; 4] = [
    b"extend\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup\0".as_ptr() as *const ::core::ffi::c_char,
    b"popup_setpos\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_mousescroll_values: [*const ::core::ffi::c_char; 3] = [
    b"hor:\0".as_ptr() as *const ::core::ffi::c_char,
    b"ver:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_nf_values: [*const ::core::ffi::c_char; 7] = [
    b"bin\0".as_ptr() as *const ::core::ffi::c_char,
    b"octal\0".as_ptr() as *const ::core::ffi::c_char,
    b"hex\0".as_ptr() as *const ::core::ffi::c_char,
    b"alpha\0".as_ptr() as *const ::core::ffi::c_char,
    b"unsigned\0".as_ptr() as *const ::core::ffi::c_char,
    b"blank\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_pumborder_values: [*const ::core::ffi::c_char; 9] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"shadow\0".as_ptr() as *const ::core::ffi::c_char,
    b"rounded\0".as_ptr() as *const ::core::ffi::c_char,
    b"solid\0".as_ptr() as *const ::core::ffi::c_char,
    b"bold\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_rdb_values: [*const ::core::ffi::c_char; 7] = [
    b"compositor\0".as_ptr() as *const ::core::ffi::c_char,
    b"nothrottle\0".as_ptr() as *const ::core::ffi::c_char,
    b"invalid\0".as_ptr() as *const ::core::ffi::c_char,
    b"nodelta\0".as_ptr() as *const ::core::ffi::c_char,
    b"line\0".as_ptr() as *const ::core::ffi::c_char,
    b"flush\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_rlc_values: [*const ::core::ffi::c_char; 2] = [
    b"search\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_sbo_values: [*const ::core::ffi::c_char; 4] = [
    b"ver\0".as_ptr() as *const ::core::ffi::c_char,
    b"hor\0".as_ptr() as *const ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_sel_values: [*const ::core::ffi::c_char; 4] = [
    b"inclusive\0".as_ptr() as *const ::core::ffi::c_char,
    b"exclusive\0".as_ptr() as *const ::core::ffi::c_char,
    b"old\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_slm_values: [*const ::core::ffi::c_char; 4] = [
    b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
    b"key\0".as_ptr() as *const ::core::ffi::c_char,
    b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_ssop_values: [*const ::core::ffi::c_char; 19] = [
    b"buffers\0".as_ptr() as *const ::core::ffi::c_char,
    b"winpos\0".as_ptr() as *const ::core::ffi::c_char,
    b"resize\0".as_ptr() as *const ::core::ffi::c_char,
    b"winsize\0".as_ptr() as *const ::core::ffi::c_char,
    b"localoptions\0".as_ptr() as *const ::core::ffi::c_char,
    b"options\0".as_ptr() as *const ::core::ffi::c_char,
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"blank\0".as_ptr() as *const ::core::ffi::c_char,
    b"globals\0".as_ptr() as *const ::core::ffi::c_char,
    b"slash\0".as_ptr() as *const ::core::ffi::c_char,
    b"unix\0".as_ptr() as *const ::core::ffi::c_char,
    b"sesdir\0".as_ptr() as *const ::core::ffi::c_char,
    b"curdir\0".as_ptr() as *const ::core::ffi::c_char,
    b"folds\0".as_ptr() as *const ::core::ffi::c_char,
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"tabpages\0".as_ptr() as *const ::core::ffi::c_char,
    b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
    b"skiprtp\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_sloc_values: [*const ::core::ffi::c_char; 4] = [
    b"last\0".as_ptr() as *const ::core::ffi::c_char,
    b"statusline\0".as_ptr() as *const ::core::ffi::c_char,
    b"tabline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_scl_values: [*const ::core::ffi::c_char; 23] = [
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"auto:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:1\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:2\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:3\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:4\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:5\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:6\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:7\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:8\0".as_ptr() as *const ::core::ffi::c_char,
    b"yes:9\0".as_ptr() as *const ::core::ffi::c_char,
    b"number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_spo_values: [*const ::core::ffi::c_char; 3] = [
    b"camel\0".as_ptr() as *const ::core::ffi::c_char,
    b"noplainbuffer\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_sps_values: [*const ::core::ffi::c_char; 7] = [
    b"best\0".as_ptr() as *const ::core::ffi::c_char,
    b"fast\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
    b"file:\0".as_ptr() as *const ::core::ffi::c_char,
    b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_spk_values: [*const ::core::ffi::c_char; 4] = [
    b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"screen\0".as_ptr() as *const ::core::ffi::c_char,
    b"topline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_swb_values: [*const ::core::ffi::c_char; 7] = [
    b"useopen\0".as_ptr() as *const ::core::ffi::c_char,
    b"usetab\0".as_ptr() as *const ::core::ffi::c_char,
    b"split\0".as_ptr() as *const ::core::ffi::c_char,
    b"newtab\0".as_ptr() as *const ::core::ffi::c_char,
    b"vsplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"uselast\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_tcl_values: [*const ::core::ffi::c_char; 3] = [
    b"left\0".as_ptr() as *const ::core::ffi::c_char,
    b"uselast\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_tc_values: [*const ::core::ffi::c_char; 6] = [
    b"followic\0".as_ptr() as *const ::core::ffi::c_char,
    b"ignore\0".as_ptr() as *const ::core::ffi::c_char,
    b"match\0".as_ptr() as *const ::core::ffi::c_char,
    b"followscs\0".as_ptr() as *const ::core::ffi::c_char,
    b"smart\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_tpf_values: [*const ::core::ffi::c_char; 8] = [
    b"BS\0".as_ptr() as *const ::core::ffi::c_char,
    b"HT\0".as_ptr() as *const ::core::ffi::c_char,
    b"FF\0".as_ptr() as *const ::core::ffi::c_char,
    b"ESC\0".as_ptr() as *const ::core::ffi::c_char,
    b"DEL\0".as_ptr() as *const ::core::ffi::c_char,
    b"C0\0".as_ptr() as *const ::core::ffi::c_char,
    b"C1\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_ve_values: [*const ::core::ffi::c_char; 7] = [
    b"block\0".as_ptr() as *const ::core::ffi::c_char,
    b"insert\0".as_ptr() as *const ::core::ffi::c_char,
    b"all\0".as_ptr() as *const ::core::ffi::c_char,
    b"onemore\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_wim_values: [*const ::core::ffi::c_char; 6] = [
    b"full\0".as_ptr() as *const ::core::ffi::c_char,
    b"longest\0".as_ptr() as *const ::core::ffi::c_char,
    b"list\0".as_ptr() as *const ::core::ffi::c_char,
    b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
    b"noselect\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_wop_values: [*const ::core::ffi::c_char; 5] = [
    b"fuzzy\0".as_ptr() as *const ::core::ffi::c_char,
    b"tagfile\0".as_ptr() as *const ::core::ffi::c_char,
    b"pum\0".as_ptr() as *const ::core::ffi::c_char,
    b"exacttext\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_wak_values: [*const ::core::ffi::c_char; 4] = [
    b"yes\0".as_ptr() as *const ::core::ffi::c_char,
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    b"no\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut opt_winborder_values: [*const ::core::ffi::c_char; 9] = [
    b"\0".as_ptr() as *const ::core::ffi::c_char,
    b"double\0".as_ptr() as *const ::core::ffi::c_char,
    b"single\0".as_ptr() as *const ::core::ffi::c_char,
    b"shadow\0".as_ptr() as *const ::core::ffi::c_char,
    b"rounded\0".as_ptr() as *const ::core::ffi::c_char,
    b"solid\0".as_ptr() as *const ::core::ffi::c_char,
    b"bold\0".as_ptr() as *const ::core::ffi::c_char,
    b"none\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[no_mangle]
pub static mut empty_string_option: [::core::ffi::c_char; 1] =
    unsafe { ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0") };
#[no_mangle]
pub static mut p_ambw: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_acd: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ai: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_bin: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_bomb: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_bl: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_cin: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_channel: OptInt = 0;
#[no_mangle]
pub static mut p_cink: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cinsd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cinw: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cfu: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ofu: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tsrfu: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ci: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ar: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_aw: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_awa: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_bs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bk: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_bkc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut bkc_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_bdir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bex: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut breakat_flags: [::core::ffi::c_char; 256] = [0; 256];
#[no_mangle]
pub static mut bo_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_bsk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_breakat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bh: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_bt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_busy: OptInt = 0;
#[no_mangle]
pub static mut p_cmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut cmp_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_enc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_deco: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ccv: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cino: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cedit: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cb: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut cb_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_cwh: OptInt = 0;
#[no_mangle]
pub static mut p_ch: OptInt = 0;
#[no_mangle]
pub static mut p_cms: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cpt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cto: OptInt = 0;
#[no_mangle]
pub static mut p_columns: OptInt = 0;
#[no_mangle]
pub static mut p_confirm: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_cia: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut cia_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_cot: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut cot_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_ac: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_act: OptInt = 0;
#[no_mangle]
pub static mut p_acl: OptInt = 0;
#[no_mangle]
pub static mut p_pumborder: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_pb: OptInt = 0;
#[no_mangle]
pub static mut p_ph: OptInt = 0;
#[no_mangle]
pub static mut p_pw: OptInt = 0;
#[no_mangle]
pub static mut p_pmw: OptInt = 0;
#[no_mangle]
pub static mut p_com: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cpo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_debug: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_def: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_inc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dia: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dip: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dex: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dict: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dg: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_dir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_dy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut dy_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_ead: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_emoji: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ea: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ep: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_eb: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ef: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_efm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_gefm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_gp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_eof: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_eol: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ei: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_et: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_exrc: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fencs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ff: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ffs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fic: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ft: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fcs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ffu: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fixeol: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_fcl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fdls: OptInt = 0;
#[no_mangle]
pub static mut p_fdo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut fdo_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_fex: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_flp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_fs: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_gd: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_guicursor: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_guifont: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_guifontwide: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_hf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_hh: OptInt = 0;
#[no_mangle]
pub static mut p_hlg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_hid: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_hl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_hls: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_hi: OptInt = 0;
#[no_mangle]
pub static mut p_arshape: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_icon: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_iconstring: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ic: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_iminsert: OptInt = 0;
#[no_mangle]
pub static mut p_imsearch: OptInt = 0;
#[no_mangle]
pub static mut p_inf: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_inex: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_is: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_inde: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_indk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_icm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_isf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_isi: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_isk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_isp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_js: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_jop: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut jop_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_keymap: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_kp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_km: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_langmap: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_lnr: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_lrm: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_lm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_lines: OptInt = 0;
#[no_mangle]
pub static mut p_linespace: OptInt = 0;
#[no_mangle]
pub static mut p_lisp: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_lop: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_lispwords: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ls: OptInt = 0;
#[no_mangle]
pub static mut p_stal: OptInt = 0;
#[no_mangle]
pub static mut p_lcs: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_lz: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_lpl: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_magic: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_menc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mef: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mps: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mat: OptInt = 0;
#[no_mangle]
pub static mut p_mco: OptInt = 0;
#[no_mangle]
pub static mut p_mfd: OptInt = 0;
#[no_mangle]
pub static mut p_mmd: OptInt = 0;
#[no_mangle]
pub static mut p_mmp: OptInt = 0;
#[no_mangle]
pub static mut p_mis: OptInt = 0;
#[no_mangle]
pub static mut p_mopt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_msc: OptInt = 0;
#[no_mangle]
pub static mut p_msm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ml: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mle: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mls: OptInt = 0;
#[no_mangle]
pub static mut p_ma: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mod: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mouse: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mousem: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mousemev: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mousef: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mh: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_mousescroll: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_mousescroll_vert: OptInt = 3 as OptInt;
#[no_mangle]
pub static mut p_mousescroll_hor: OptInt = 6 as OptInt;
#[no_mangle]
pub static mut p_mouset: OptInt = 0;
#[no_mangle]
pub static mut p_more: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_nf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_opfunc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_para: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_paste: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_pex: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_pm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_cdpath: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_pi: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_pyx: OptInt = 0;
#[no_mangle]
pub static mut p_qe: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ro: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_rdb: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut rdb_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_rdt: OptInt = 0;
#[no_mangle]
pub static mut p_re: OptInt = 0;
#[no_mangle]
pub static mut p_report: OptInt = 0;
#[no_mangle]
pub static mut p_pvh: OptInt = 0;
#[no_mangle]
pub static mut p_chi: OptInt = 0;
#[no_mangle]
pub static mut p_ari: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ri: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ru: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ruf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_pp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_qftf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_rtp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_scbk: OptInt = 0;
#[no_mangle]
pub static mut p_sj: OptInt = 0;
#[no_mangle]
pub static mut p_so: OptInt = 0;
#[no_mangle]
pub static mut p_sbo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sections: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_secure: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sel: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_slm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ssop: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut ssop_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_sh: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_shcf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_shq: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sxq: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sxe: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_srr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_stmp: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_stl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_wbr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sr: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sw: OptInt = 0;
#[no_mangle]
pub static mut p_shm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sbr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sc: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sloc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_sft: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sm: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_smd: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ss: OptInt = 0;
#[no_mangle]
pub static mut p_siso: OptInt = 0;
#[no_mangle]
pub static mut p_scs: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_si: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sta: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sts: OptInt = 0;
#[no_mangle]
pub static mut p_sb: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sua: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_swf: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_smc: OptInt = 0;
#[no_mangle]
pub static mut p_tpm: OptInt = 0;
#[no_mangle]
pub static mut p_tal: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tpf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut tpf_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_tfu: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_spc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_spl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_spo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut spo_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_sps: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_spr: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_sol: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_su: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_swb: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut swb_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_spk: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_syn: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tcl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut tcl_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_ts: OptInt = 0;
#[no_mangle]
pub static mut p_tbs: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_tc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut tc_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_tl: OptInt = 0;
#[no_mangle]
pub static mut p_tr: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_tags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tgst: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_tbidi: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_tw: OptInt = 0;
#[no_mangle]
pub static mut p_to: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_timeout: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_tm: OptInt = 0;
#[no_mangle]
pub static mut p_title: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_titlelen: OptInt = 0;
#[no_mangle]
pub static mut p_titleold: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_titlestring: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tsr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_tgc: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ttimeout: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ttm: OptInt = 0;
#[no_mangle]
pub static mut p_tf: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_udir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_udf: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ul: OptInt = 0;
#[no_mangle]
pub static mut p_ur: OptInt = 0;
#[no_mangle]
pub static mut p_uc: OptInt = 0;
#[no_mangle]
pub static mut p_ut: OptInt = 0;
#[no_mangle]
pub static mut p_shada: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_shadafile: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_termsync: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_vsts: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_vts: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_vdir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_vop: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut vop_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_vb: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_ve: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut ve_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_verbose: OptInt = 0;
#[no_mangle]
pub static mut p_warn: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_wop: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut wop_flags: ::core::ffi::c_uint = 0;
#[no_mangle]
pub static mut p_window: OptInt = 0;
#[no_mangle]
pub static mut p_wak: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_wig: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_ww: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_wc: OptInt = 0;
#[no_mangle]
pub static mut p_wcm: OptInt = 0;
#[no_mangle]
pub static mut p_wic: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_wim: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_wmnu: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_winborder: *mut ::core::ffi::c_char =
    ::core::ptr::null_mut::<::core::ffi::c_char>();
#[no_mangle]
pub static mut p_wh: OptInt = 0;
#[no_mangle]
pub static mut p_wmh: OptInt = 0;
#[no_mangle]
pub static mut p_wmw: OptInt = 0;
#[no_mangle]
pub static mut p_wiw: OptInt = 0;
#[no_mangle]
pub static mut p_wm: OptInt = 0;
#[no_mangle]
pub static mut p_ws: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_write: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_wa: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_wb: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut p_wd: OptInt = 0;
#[no_mangle]
pub static mut p_cdh: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut hlf_names: [*const ::core::ffi::c_char; 76] = [
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"SpecialKey\0".as_ptr() as *const ::core::ffi::c_char,
    b"EndOfBuffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"TermCursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"NonText\0".as_ptr() as *const ::core::ffi::c_char,
    b"Directory\0".as_ptr() as *const ::core::ffi::c_char,
    b"ErrorMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"IncSearch\0".as_ptr() as *const ::core::ffi::c_char,
    b"Search\0".as_ptr() as *const ::core::ffi::c_char,
    b"CurSearch\0".as_ptr() as *const ::core::ffi::c_char,
    b"MoreMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"ModeMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNrAbove\0".as_ptr() as *const ::core::ffi::c_char,
    b"LineNrBelow\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineNr\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineSign\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLineFold\0".as_ptr() as *const ::core::ffi::c_char,
    b"Question\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinSeparator\0".as_ptr() as *const ::core::ffi::c_char,
    b"VertSplit\0".as_ptr() as *const ::core::ffi::c_char,
    b"Title\0".as_ptr() as *const ::core::ffi::c_char,
    b"Visual\0".as_ptr() as *const ::core::ffi::c_char,
    b"VisualNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"WarningMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"WildMenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"Folded\0".as_ptr() as *const ::core::ffi::c_char,
    b"FoldColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffAdd\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffChange\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffDelete\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffText\0".as_ptr() as *const ::core::ffi::c_char,
    b"DiffTextAdd\0".as_ptr() as *const ::core::ffi::c_char,
    b"SignColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"Conceal\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellBad\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellCap\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellRare\0".as_ptr() as *const ::core::ffi::c_char,
    b"SpellLocal\0".as_ptr() as *const ::core::ffi::c_char,
    b"Pmenu\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuMatch\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuMatchSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuKind\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuKindSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuExtra\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuExtraSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuSbar\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuThumb\0".as_ptr() as *const ::core::ffi::c_char,
    b"PmenuBorder\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLineSel\0".as_ptr() as *const ::core::ffi::c_char,
    b"TabLineFill\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"CursorLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"ColorColumn\0".as_ptr() as *const ::core::ffi::c_char,
    b"QuickFixLine\0".as_ptr() as *const ::core::ffi::c_char,
    b"Whitespace\0".as_ptr() as *const ::core::ffi::c_char,
    b"NormalNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"MsgSeparator\0".as_ptr() as *const ::core::ffi::c_char,
    b"NormalFloat\0".as_ptr() as *const ::core::ffi::c_char,
    b"MsgArea\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatBorder\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinBar\0".as_ptr() as *const ::core::ffi::c_char,
    b"WinBarNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"Cursor\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatTitle\0".as_ptr() as *const ::core::ffi::c_char,
    b"FloatFooter\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineTerm\0".as_ptr() as *const ::core::ffi::c_char,
    b"StatusLineTermNC\0".as_ptr() as *const ::core::ffi::c_char,
    b"StderrMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"StdoutMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"OkMsg\0".as_ptr() as *const ::core::ffi::c_char,
    b"PreInsert\0".as_ptr() as *const ::core::ffi::c_char,
];
#[no_mangle]
pub static mut highlight_attr: [::core::ffi::c_int; 76] = [0; 76];
#[no_mangle]
pub static mut highlight_attr_last: [::core::ffi::c_int; 76] = [0; 76];
#[no_mangle]
pub static mut highlight_user: [::core::ffi::c_int; 9] = [0; 9];
#[no_mangle]
pub static mut highlight_stlnc: [::core::ffi::c_int; 9] = [0; 9];
#[no_mangle]
pub static mut cterm_normal_fg_color: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cterm_normal_bg_color: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut normal_fg: RgbValue = -1 as RgbValue;
#[no_mangle]
pub static mut normal_bg: RgbValue = -1 as RgbValue;
#[no_mangle]
pub static mut normal_sp: RgbValue = -1 as RgbValue;
#[no_mangle]
pub static mut ns_hl_global: NS = 0 as NS;
#[no_mangle]
pub static mut ns_hl_win: NS = -1 as NS;
#[no_mangle]
pub static mut ns_hl_fast: NS = -1 as NS;
#[no_mangle]
pub static mut ns_hl_active: NS = 0 as NS;
#[no_mangle]
pub static mut hl_attr_active: *mut ::core::ffi::c_int =
    unsafe { &raw const highlight_attr as *mut ::core::ffi::c_int };
#[no_mangle]
pub static mut curbuf_splice_pending: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
#[no_mangle]
pub static mut nlua_global_refs: *mut nlua_ref_state_t =
    ::core::ptr::null_mut::<nlua_ref_state_t>();
#[no_mangle]
pub static mut nlua_disable_preload: bool = false;
#[no_mangle]
pub static mut main_loop: Loop = Loop {
    uv: uv_loop_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        active_handles: 0,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        active_reqs: C2Rust_Unnamed_18 {
            unused: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        internal_fields: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        stop_flag: 0,
        flags: 0,
        backend_fd: 0,
        pending_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watcher_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watchers: ::core::ptr::null_mut::<*mut uv__io_t>(),
        nwatchers: 0,
        nfds: 0,
        wq: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        wq_mutex: pthread_mutex_t {
            __data: __pthread_mutex_s {
                __lock: 0,
                __count: 0,
                __owner: 0,
                __nusers: 0,
                __kind: 0,
                __spins: 0,
                __elision: 0,
                __list: __pthread_list_t {
                    __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                    __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
                },
            },
        },
        wq_async: uv_async_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_19 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            async_cb: None,
            queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pending: 0,
        },
        cloexec_lock: pthread_rwlock_t {
            __data: __pthread_rwlock_arch_t {
                __readers: 0,
                __writers: 0,
                __wrphase_futex: 0,
                __writers_futex: 0,
                __pad3: 0,
                __pad4: 0,
                __cur_writer: 0,
                __shared: 0,
                __rwelision: 0,
                __pad1: [0; 7],
                __pad2: 0,
                __flags: 0,
            },
        },
        closing_handles: ::core::ptr::null_mut::<uv_handle_t>(),
        process_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        prepare_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        check_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        idle_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_unused: None,
        async_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        async_wfd: 0,
        timer_heap: C2Rust_Unnamed_17 {
            min: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            nelts: 0,
        },
        timer_counter: 0,
        time: 0,
        signal_pipefd: [0; 2],
        signal_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        child_watcher: uv_signal_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_16 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            signal_cb: None,
            signum: 0,
            tree_entry: C2Rust_Unnamed_15 {
                rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_color: 0,
            },
            caught_signals: 0,
            dispatched_signals: 0,
        },
        emfile_fd: 0,
        inotify_read_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        inotify_watchers: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        inotify_fd: 0,
    },
    events: ::core::ptr::null_mut::<MultiQueue>(),
    thread_events: ::core::ptr::null_mut::<MultiQueue>(),
    fast_events: ::core::ptr::null_mut::<MultiQueue>(),
    children: C2Rust_Unnamed_22 {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<*mut Proc>(),
    },
    children_watcher: uv_signal_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_16 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: C2Rust_Unnamed_15 {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    children_kill_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    poll_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    exit_delay_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_21 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: C2Rust_Unnamed_20 {
            heap: [::core::ptr::null_mut::<::core::ffi::c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    async_0: uv_async_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: C2Rust_Unnamed_19 { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        async_cb: None,
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        pending: 0,
    },
    mutex: pthread_mutex_t {
        __data: __pthread_mutex_s {
            __lock: 0,
            __count: 0,
            __owner: 0,
            __nusers: 0,
            __kind: 0,
            __spins: 0,
            __elision: 0,
            __list: __pthread_list_t {
                __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
            },
        },
    },
    recursive: 0,
    closing: false,
};
static mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
static mut err_arg_missing: *const ::core::ffi::c_char =
    b"Argument missing after\0".as_ptr() as *const ::core::ffi::c_char;
static mut err_opt_garbage: *const ::core::ffi::c_char =
    b"Garbage after option argument\0".as_ptr() as *const ::core::ffi::c_char;
static mut err_opt_unknown: *const ::core::ffi::c_char =
    b"Unknown option argument\0".as_ptr() as *const ::core::ffi::c_char;
static mut err_too_many_args: *const ::core::ffi::c_char =
    b"Too many edit arguments\0".as_ptr() as *const ::core::ffi::c_char;
static mut err_extra_cmd: *const ::core::ffi::c_char =
    b"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\0".as_ptr()
        as *const ::core::ffi::c_char;
#[no_mangle]
pub unsafe extern "C" fn event_init() {
    loop_init(&raw mut main_loop, NULL_0);
    env_init();
    resize_events = multiqueue_new_child(main_loop.events);
    autocmd_init();
    signal_init();
    channel_init();
    terminal_init();
    ui_init();
    if !time_fd.is_null() {
        time_msg(
            b"event init\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn event_teardown() -> bool {
    if main_loop.events.is_null() {
        input_stop();
        return true_0 != 0;
    }
    multiqueue_process_events(main_loop.events);
    loop_poll_events(&raw mut main_loop, 0 as int64_t);
    input_stop();
    server_teardown();
    channel_teardown();
    proc_teardown(&raw mut main_loop);
    timer_teardown();
    signal_teardown();
    terminal_teardown();
    return loop_close(&raw mut main_loop, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn early_init(mut paramp: *mut mparm_T) {
    os_hint_priority();
    estack_init();
    cmdline_init();
    eval_init();
    set_vim_var_nr(VV_STARTTIME, os_realtime());
    init_path(if !argv0.is_null() {
        argv0 as *const ::core::ffi::c_char
    } else {
        b"nvim\0".as_ptr() as *const ::core::ffi::c_char
    });
    init_normal_cmds();
    runtime_init();
    highlight_init();
    if !time_fd.is_null() {
        time_msg(
            b"early init\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    init_locale();
    set_init_tablocal();
    win_alloc_first();
    if !time_fd.is_null() {
        time_msg(
            b"init first window\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    alist_init(&raw mut global_alist);
    global_alist.id = 0 as ::core::ffi::c_int;
    init_homedir();
    set_init_1(
        if !paramp.is_null() {
            (*paramp).clean as ::core::ffi::c_int
        } else {
            false_0
        } != 0,
    );
    log_init();
    if !time_fd.is_null() {
        time_msg(
            b"inits 1\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_lang_var();
    qf_init_stack();
}
unsafe fn main_0(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    argv0 = *argv.offset(0 as ::core::ffi::c_int as isize);
    if !appname_is_valid() {
        fprintf(
            stderr,
            b"$NVIM_APPNAME must be a name or relative path.\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        exit(1 as ::core::ffi::c_int);
    }
    if argc > 1 as ::core::ffi::c_int
        && strcasecmp(
            *argv.offset(1 as ::core::ffi::c_int as isize),
            b"-ll\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        if argc == 2 as ::core::ffi::c_int {
            print_mainerr(
                err_arg_missing,
                *argv.offset(1 as ::core::ffi::c_int as isize),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            exit(1 as ::core::ffi::c_int);
        }
        nlua_run_script(argv, argc, 3 as ::core::ffi::c_int);
    }
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut params: mparm_T = mparm_T {
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        use_vimrc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        clean: false,
        n_commands: 0,
        commands: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        cmds_tofree: [0; 10],
        n_pre_commands: 0,
        pre_commands: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        luaf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lua_arg0: 0,
        edit_type: 0,
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_ef: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        input_istext: false,
        no_swap_file: 0,
        use_debug_break_level: 0,
        window_count: 0,
        window_layout: 0,
        diff_mode: 0,
        listen_addr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        remote: 0,
        server_addr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptin: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptout: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        scriptout_append: false,
        had_stdin_file: false,
    };
    init_params(&raw mut params, argc, argv);
    init_startuptime(&raw mut params);
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < params.argc {
        if strcasecmp(
            *params.argv.offset(i as isize),
            b"--clean\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            params.clean = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    event_init();
    early_init(&raw mut params);
    set_argv_var(argv, argc);
    check_and_set_isatty(&raw mut params);
    command_line_scan(&raw mut params);
    set_argf_var();
    nlua_init(argv, argc, params.lua_arg0);
    if !time_fd.is_null() {
        time_msg(
            b"init lua interpreter\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if embedded_mode {
        let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if channel_from_stdio(
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: C2Rust_Unnamed_5 {
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
            &raw mut err,
        ) == 0
        {
            abort();
        }
    }
    if global_alist.al_ga.ga_len > 0 as ::core::ffi::c_int {
        fname = get_fname(&raw mut params);
    }
    if recoverymode as ::core::ffi::c_int != 0 && fname.is_null() {
        headless_mode = true_0 != 0;
    }
    let mut has_term: bool = stdin_isatty as ::core::ffi::c_int != 0
        || stdout_isatty as ::core::ffi::c_int != 0
        || stderr_isatty as ::core::ffi::c_int != 0;
    let mut use_builtin_ui: bool =
        has_term as ::core::ffi::c_int != 0 && !headless_mode && !embedded_mode && !silent_mode;
    if params.remote != 0 {
        remote_request(
            &raw mut params,
            params.remote,
            params.server_addr,
            argc,
            argv,
            use_builtin_ui,
        );
    }
    let mut remote_ui: bool = ui_client_channel_id != 0 as uint64_t;
    if use_builtin_ui as ::core::ffi::c_int != 0 && !remote_ui {
        ui_client_forward_stdin = !stdin_isatty;
        let mut rv: uint64_t = ui_client_start_server(
            get_vim_var_str(VV_PROGPATH),
            params.argc as size_t,
            params.argv,
        );
        if rv == 0 {
            fprintf(
                stderr,
                b"Failed to start Nvim server!\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            os_exit(1 as ::core::ffi::c_int);
        }
        ui_client_channel_id = rv;
    }
    if ui_client_channel_id != 0 {
        ui_client_run();
    }
    '_c2rust_label: {
        if ui_client_channel_id == 0 && !use_builtin_ui {
        } else {
            __assert_fail(
                b"!ui_client_channel_id && !use_builtin_ui\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                369 as ::core::ffi::c_uint,
                b"int main(int, char **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !server_init(params.listen_addr) {
        mainerr(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if !time_fd.is_null() {
        time_msg(
            b"expanding arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.diff_mode != 0 && params.window_count == -1 as ::core::ffi::c_int {
        params.window_count = 0 as ::core::ffi::c_int;
    }
    RedrawingDisabled += 1;
    setbuf(stdout, ::core::ptr::null_mut::<::core::ffi::c_char>());
    full_screen = !silent_mode;
    win_init_size();
    if params.diff_mode != 0 {
        diff_win_options(firstwin, false_0 != 0);
    }
    '_c2rust_label_0: {
        if p_ch >= 0 as OptInt
            && Rows as OptInt >= p_ch
            && Rows as OptInt - p_ch <= 2147483647 as OptInt
        {
        } else {
            __assert_fail(
                b"p_ch >= 0 && Rows >= p_ch && Rows - p_ch <= INT_MAX\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                414 as ::core::ffi::c_uint,
                b"int main(int, char **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    cmdline_row = Rows - p_ch as ::core::ffi::c_int;
    msg_row = cmdline_row;
    default_grid_alloc();
    set_init_2(headless_mode);
    if !time_fd.is_null() {
        time_msg(
            b"inits 2\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    msg_scroll = true_0;
    no_wait_return = true_0;
    init_highlight(true_0 != 0, false_0 != 0);
    ui_comp_syn_init();
    if !time_fd.is_null() {
        time_msg(
            b"init highlight\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    debug_break_level = params.use_debug_break_level;
    if !stdin_isatty
        && !params.input_istext
        && silent_mode as ::core::ffi::c_int != 0
        && exmode_active as ::core::ffi::c_int != 0
    {
        input_start();
    }
    let mut use_remote_ui: bool = embedded_mode as ::core::ffi::c_int != 0 && !headless_mode;
    if use_remote_ui {
        if !time_fd.is_null() {
            time_msg(
                b"waiting for UI\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        remote_ui_wait_for_attach();
        if !time_fd.is_null() {
            time_msg(
                b"done waiting for UI\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        (*firstwin).w_prev_height = (*firstwin).w_height;
    }
    starting = NO_BUFFERS;
    screenclear();
    win_new_screensize();
    if !time_fd.is_null() {
        time_msg(
            b"clear screen\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if edit_stdin(&raw mut params) {
        params.edit_type = EDIT_STDIN as ::core::ffi::c_int;
    }
    if !params.scriptin.is_null() {
        if !open_scriptin(params.scriptin) {
            os_exit(2 as ::core::ffi::c_int);
        }
    }
    if !params.scriptout.is_null() {
        scriptout = os_fopen(
            params.scriptout,
            if params.scriptout_append as ::core::ffi::c_int != 0 {
                APPENDBIN.as_ptr()
            } else {
                WRITEBIN.as_ptr()
            },
        );
        if scriptout.is_null() {
            fprintf(
                stderr,
                gettext(
                    b"Cannot open for script output: \"\0".as_ptr() as *const ::core::ffi::c_char
                ),
            );
            fprintf(
                stderr,
                b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
                params.scriptout,
            );
            os_exit(2 as ::core::ffi::c_int);
        }
    }
    nlua_init_defaults();
    if !time_fd.is_null() {
        time_msg(
            b"init default mappings & autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut vimrc_none: bool = strequal(
        params.use_vimrc,
        b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
    );
    p_lpl = if vimrc_none as ::core::ffi::c_int != 0 {
        params.clean as ::core::ffi::c_int
    } else {
        p_lpl
    };
    exe_pre_commands(&raw mut params);
    if !vimrc_none || params.clean as ::core::ffi::c_int != 0 {
        filetype_plugin_enable();
    }
    source_startup_scripts(&raw mut params);
    if !vimrc_none || params.clean as ::core::ffi::c_int != 0 {
        filetype_maybe_enable();
        syn_maybe_enable();
    }
    set_vim_var_nr(VV_VIM_DID_INIT, 1 as varnumber_T);
    load_plugins();
    set_window_layout(&raw mut params);
    if recoverymode as ::core::ffi::c_int != 0 && fname.is_null() {
        recover_names(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            ::core::ptr::null_mut::<list_T>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        os_exit(0 as ::core::ffi::c_int);
    }
    set_init_3();
    if !time_fd.is_null() {
        time_msg(
            b"inits 3\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if params.no_swap_file != 0 {
        p_uc = 0 as OptInt;
    }
    if silent_mode {
        p_ut = 1 as OptInt;
    }
    if *p_shada as ::core::ffi::c_int != NUL {
        shada_read_everything(
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        );
        if !time_fd.is_null() {
            time_msg(
                b"reading ShaDa\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if get_vim_var_list(VV_OLDFILES).is_null() {
        set_vim_var_list(VV_OLDFILES, tv_list_alloc(0 as ptrdiff_t));
    }
    handle_quickfix(&raw mut params);
    starting = NO_BUFFERS;
    no_wait_return = false_0;
    if !exmode_active {
        msg_scroll = false_0;
    }
    if params.edit_type == EDIT_STDIN as ::core::ffi::c_int && !recoverymode {
        read_stdin();
    }
    setmouse();
    redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
    no_wait_return = true_0;
    create_windows(&raw mut params);
    if !time_fd.is_null() {
        time_msg(
            b"opening buffers\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    set_vim_var_string(
        VV_SWAPCOMMAND,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    if exmode_active {
        (*curwin).w_cursor.lnum = (*curbuf).b_ml.ml_line_count;
    }
    apply_autocmds(
        EVENT_BUFENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if !time_fd.is_null() {
        time_msg(
            b"BufEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    setpcmark();
    if params.edit_type == EDIT_QF as ::core::ffi::c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0,
        );
        if !time_fd.is_null() {
            time_msg(
                b"jump to first error\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    edit_buffers(&raw mut params);
    if params.diff_mode != 0 {
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_arg_idx_invalid == 0 {
                diff_win_options(wp, true_0 != 0);
            }
            wp = (*wp).w_next;
        }
    }
    shorten_fnames(false_0);
    handle_tag(params.tagname);
    if params.n_commands > 0 as ::core::ffi::c_int {
        exe_commands(&raw mut params);
    }
    starting = 0 as ::core::ffi::c_int;
    RedrawingDisabled = 0 as ::core::ffi::c_int;
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    no_wait_return = false_0;
    do_autochdir();
    set_vim_var_nr(VV_VIM_DID_ENTER, 1 as varnumber_T);
    apply_autocmds(
        EVENT_VIMENTER,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if !time_fd.is_null() {
        time_msg(
            b"VimEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if use_remote_ui {
        do_autocmd_uienter_all();
        if !time_fd.is_null() {
            time_msg(
                b"UIEnter autocommands\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    set_reg_var(get_default_register_name());
    if (*curwin).w_onebuf_opt.wo_diff != 0 && (*curwin).w_onebuf_opt.wo_scb != 0 {
        update_topline(curwin);
        check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
        if !time_fd.is_null() {
            time_msg(
                b"diff scrollbinding\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
    if restart_edit != 0 as ::core::ffi::c_int {
        stuffcharReadbuff(
            -(253 as ::core::ffi::c_int
                + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
        );
    }
    if cb_flags
        & (kOptCbFlagUnnamed as ::core::ffi::c_int | kOptCbFlagUnnamedplus as ::core::ffi::c_int)
            as ::core::ffi::c_uint
        != 0
    {
        eval_has_provider(
            b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
    }
    if !params.luaf.is_null() {
        msg_scroll = true_0;
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"main\0".as_ptr() as *const ::core::ffi::c_char,
            678 as ::core::ffi::c_int,
            true_0 != 0,
            b"executing Lua -l script\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut lua_ok: bool = nlua_exec_file(params.luaf);
        if !time_fd.is_null() {
            time_msg(
                b"executing Lua -l script\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if msg_didout {
            msg_putchar('\n' as ::core::ffi::c_int);
            msg_didout = false_0 != 0;
        }
        getout(if lua_ok as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        });
    }
    if !time_fd.is_null() {
        time_msg(
            b"before starting main loop\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"main\0".as_ptr() as *const ::core::ffi::c_char,
        689 as ::core::ffi::c_int,
        true_0 != 0,
        b"starting main loop\0".as_ptr() as *const ::core::ffi::c_char,
    );
    normal_enter(false_0 != 0, false_0 != 0);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_exit(mut r: ::core::ffi::c_int) -> ! {
    exiting = true_0 != 0;
    if ui_client_channel_id != 0 {
        ui_client_stop();
        if r == 0 as ::core::ffi::c_int {
            r = ui_client_exit_status;
        }
    } else {
        ui_flush();
        ui_call_stop();
    }
    if !event_teardown() && r == 0 as ::core::ffi::c_int {
        r = 1 as ::core::ffi::c_int;
    }
    if ui_client_channel_id != 0 {
        if stdout_isatty {
            tcdrain(STDOUT_FILENO);
        }
        if stderr_isatty {
            tcdrain(STDERR_FILENO);
        }
    } else {
        ml_close_all(true_0 != 0);
    }
    if used_stdin {
        stream_set_blocking(STDIN_FILENO, true_0 != 0);
    }
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_exit\0".as_ptr() as *const ::core::ffi::c_char,
        737 as ::core::ffi::c_int,
        true_0 != 0,
        b"Nvim exit: %d\0".as_ptr() as *const ::core::ffi::c_char,
        r,
    );
    exit(r);
}
#[no_mangle]
pub unsafe extern "C" fn getout(mut exitval: ::core::ffi::c_int) -> ! {
    '_c2rust_label: {
        if ui_client_channel_id == 0 {
        } else {
            __assert_fail(
                b"!ui_client_channel_id\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                750 as ::core::ffi::c_uint,
                b"void getout(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    exiting = true_0 != 0;
    time_finish();
    if exmode_active {
        exitval += ex_exitval;
    }
    set_vim_var_type(VV_EXITING, VAR_NUMBER);
    set_vim_var_nr(VV_EXITING, exitval as varnumber_T);
    if *get_vim_var_str(VV_EXITREASON) as ::core::ffi::c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
    }
    invoke_all_defer();
    hash_debug_results();
    if v_dying <= 1 as ::core::ffi::c_int {
        let mut next_tp: *const tabpage_T = ::core::ptr::null::<tabpage_T>();
        let mut tp: *const tabpage_T = first_tabpage;
        while !tp.is_null() {
            next_tp = (*tp).tp_next;
            let mut wp: *mut win_T = if tp == curtab as *const tabpage_T {
                firstwin
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if !((*wp).w_buffer.is_null() || !buf_valid((*wp).w_buffer)) {
                    let mut buf: *mut buf_T = (*wp).w_buffer;
                    if buf_get_changedtick(buf) != -1 as varnumber_T {
                        let mut bufref: bufref_T = bufref_T {
                            br_buf: ::core::ptr::null_mut::<buf_T>(),
                            br_fnum: 0,
                            br_buf_free_count: 0,
                        };
                        set_bufref(&raw mut bufref, buf);
                        apply_autocmds(
                            EVENT_BUFWINLEAVE,
                            (*buf).b_fname,
                            (*buf).b_fname,
                            false_0 != 0,
                            buf,
                        );
                        if bufref_valid(&raw mut bufref) {
                            buf_set_changedtick(buf, -1 as varnumber_T);
                        }
                        next_tp = first_tabpage;
                        break;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = next_tp;
        }
        let mut buf_0: *mut buf_T = firstbuf;
        while !buf_0.is_null() {
            if !(*buf_0).b_ml.ml_mfp.is_null() {
                let mut bufref_0: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref_0, buf_0);
                apply_autocmds(
                    EVENT_BUFUNLOAD,
                    (*buf_0).b_fname,
                    (*buf_0).b_fname,
                    false_0 != 0,
                    buf_0,
                );
                if !bufref_valid(&raw mut bufref_0) {
                    break;
                }
            }
            buf_0 = (*buf_0).b_next;
        }
        let mut unblock: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
        if unblock != 0 {
            block_autocmds();
        }
    }
    if !p_shada.is_null() && *p_shada as ::core::ffi::c_int != NUL {
        shada_write_file(::core::ptr::null::<::core::ffi::c_char>(), false_0 != 0);
    }
    if v_dying <= 1 as ::core::ffi::c_int {
        let mut unblock_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if is_autocmd_blocked() {
            unblock_autocmds();
            unblock_0 += 1;
        }
        apply_autocmds(
            EVENT_VIMLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
        if unblock_0 != 0 {
            block_autocmds();
        }
    }
    profile_dump();
    if did_emsg != 0 {
        no_wait_return = false_0;
        wait_return(false_0);
    }
    if p_title != 0 && *p_titleold as ::core::ffi::c_int != NUL {
        ui_call_set_title(cstr_as_string(p_titleold));
    }
    if garbage_collect_at_exit {
        garbage_collect(false_0 != 0);
    }
    os_exit(exitval);
}
#[no_mangle]
pub unsafe extern "C" fn preserve_exit(mut errmsg: *const ::core::ffi::c_char) -> ! {
    static mut really_exiting: bool = false_0 != 0;
    if really_exiting {
        if used_stdin {
            stream_set_blocking(STDIN_FILENO, true_0 != 0);
        }
        exit(2 as ::core::ffi::c_int);
    }
    really_exiting = true_0 != 0;
    signal_reject_deadly();
    if ui_client_channel_id != 0 {
        ui_client_stop();
    }
    if !errmsg.is_null()
        && *errmsg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        let mut has_eol: bool = '\n' as ::core::ffi::c_int
            == *errmsg.offset(strlen(errmsg).wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int;
        fprintf(
            stderr,
            if has_eol as ::core::ffi::c_int != 0 {
                b"%s\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char
            },
            errmsg,
        );
    }
    if ui_client_channel_id != 0 {
        os_exit(1 as ::core::ffi::c_int);
    }
    ml_close_notmod();
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        if !(*buf).b_ml.ml_mfp.is_null() && !(*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
            if !errmsg.is_null() {
                fprintf(
                    stderr,
                    b"Nvim: preserving files...\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            ml_sync_all(false_0, false_0, true_0 != 0);
            break;
        } else {
            buf = (*buf).b_next;
        }
    }
    ml_close_all(false_0 != 0);
    if !errmsg.is_null() {
        fprintf(
            stderr,
            b"Nvim: Finished.\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    getout(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn get_number_arg(
    mut p: *const ::core::ffi::c_char,
    mut idx: *mut ::core::ffi::c_int,
    mut def: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ascii_isdigit(*p.offset(*idx as isize) as ::core::ffi::c_int) {
        def = atoi(p.offset(*idx as isize));
        while ascii_isdigit(*p.offset(*idx as isize) as ::core::ffi::c_int) {
            *idx = *idx + 1 as ::core::ffi::c_int;
        }
    }
    return def;
}
unsafe extern "C" fn server_connect(
    mut server_addr: *mut ::core::ffi::c_char,
    mut errmsg: *mut *const ::core::ffi::c_char,
) -> uint64_t {
    if server_addr.is_null() {
        *errmsg = b"no address specified\0".as_ptr() as *const ::core::ffi::c_char;
        return 0 as uint64_t;
    }
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_5 {
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
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut is_tcp: bool = !socket_address_tcp_host_end(server_addr).is_null();
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        500 as ::core::ffi::c_int,
        &raw mut error,
    );
    if !error.is_null() {
        *errmsg = error;
        return 0 as uint64_t;
    }
    return chan;
}
unsafe extern "C" fn remote_request(
    mut params: *mut mparm_T,
    mut remote_args: ::core::ffi::c_int,
    mut server_addr: *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut ui_only: bool,
) {
    let mut is_ui: bool = strequal(
        *argv.offset(remote_args as isize),
        b"--remote-ui\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if ui_only as ::core::ffi::c_int != 0 && !is_ui {
        return;
    }
    let mut connect_error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut chan: uint64_t = server_connect(server_addr, &raw mut connect_error);
    let mut rvobj: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if is_ui {
        if chan == 0 {
            fprintf(
                stderr,
                b"Remote ui failed to start: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                connect_error,
            );
            os_exit(1 as ::core::ffi::c_int);
        } else if strequal(
            server_addr,
            os_getenv_noalloc(b"NVIM\0".as_ptr() as *const ::core::ffi::c_char),
        ) {
            fprintf(
                stderr,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Cannot attach UI of :terminal child to its parent. \0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                b"(Unset $NVIM to skip this check)\0".as_ptr() as *const ::core::ffi::c_char,
            );
            os_exit(1 as ::core::ffi::c_int);
        }
        ui_client_channel_id = chan;
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    args.capacity = (argc - remote_args) as size_t;
    args.items = xrealloc(
        args.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
    ) as *mut Object;
    let mut t_argc: ::core::ffi::c_int = remote_args;
    while t_argc < argc {
        let c2rust_fresh1 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(*argv.offset(t_argc as isize)),
            },
        };
        t_argc += 1;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut a: Array = ARRAY_DICT_INIT;
    let mut a__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 4];
    a.capacity = 4 as size_t;
    a.items = &raw mut a__items as *mut Object;
    let c2rust_fresh2 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh2 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: chan as ::core::ffi::c_int as Integer,
        },
    };
    let c2rust_fresh3 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(server_addr),
        },
    };
    let c2rust_fresh4 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(connect_error),
        },
    };
    let c2rust_fresh5 = a.size;
    a.size = a.size.wrapping_add(1);
    *a.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed { array: args },
    };
    let mut s: String_0 = String_0 {
        data: b"return vim._cs_remote(...)\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        size: ::core::mem::size_of::<[::core::ffi::c_char; 27]>().wrapping_sub(1 as size_t),
    };
    let mut o: Object = nlua_exec(
        s,
        ::core::ptr::null::<::core::ffi::c_char>(),
        a,
        kRetObject,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    xfree(args.items as *mut ::core::ffi::c_void);
    args.capacity = 0 as size_t;
    args.size = args.capacity;
    args.items = ::core::ptr::null_mut::<Object>();
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            err.msg,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    if o.type_0 as ::core::ffi::c_uint
        == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        rvobj.data.dict = o.data.dict;
    } else {
        fprintf(
            stderr,
            b"vim._cs_remote returned unexpected value\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    let mut should_exit: TriState = kNone;
    let mut tabbed: TriState = kNone;
    let mut i: size_t = 0 as size_t;
    while i < rvobj.data.dict.size {
        if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"errmsg\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'errmsg'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
            os_exit(2 as ::core::ffi::c_int);
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"result\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'result'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            printf(
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*rvobj.data.dict.items.offset(i as isize))
                    .value
                    .data
                    .string
                    .data,
            );
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"tabbed\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'tabbed'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            tabbed = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as ::core::ffi::c_int
                != 0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState;
        } else if strequal(
            (*rvobj.data.dict.items.offset(i as isize)).key.data,
            b"should_exit\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            if (*rvobj.data.dict.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                fprintf(
                    stderr,
                    b"vim._cs_remote returned an unexpected type for 'should_exit'\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                os_exit(2 as ::core::ffi::c_int);
            }
            should_exit = (if (*rvobj.data.dict.items.offset(i as isize))
                .value
                .data
                .boolean as ::core::ffi::c_int
                != 0
            {
                kTrue as ::core::ffi::c_int
            } else {
                kFalse as ::core::ffi::c_int
            }) as TriState;
        }
        i = i.wrapping_add(1);
    }
    if should_exit as ::core::ffi::c_int == kNone as ::core::ffi::c_int
        || tabbed as ::core::ffi::c_int == kNone as ::core::ffi::c_int
    {
        fprintf(
            stderr,
            b"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        os_exit(2 as ::core::ffi::c_int);
    }
    api_free_object(o);
    if should_exit as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
        os_exit(0 as ::core::ffi::c_int);
    }
    if tabbed as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
        (*params).window_count = argc - remote_args - 1 as ::core::ffi::c_int;
        (*params).window_layout = WIN_TABS as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn edit_stdin(mut parmp: *mut mparm_T) -> bool {
    let mut implicit: bool = !headless_mode
        && !(embedded_mode as ::core::ffi::c_int != 0 && stdin_fd <= 0 as ::core::ffi::c_int)
        && (!exmode_active || (*parmp).input_istext as ::core::ffi::c_int != 0)
        && !stdin_isatty
        && (*parmp).edit_type <= EDIT_STDIN as ::core::ffi::c_int
        && (*parmp).scriptin.is_null();
    return (*parmp).had_stdin_file as ::core::ffi::c_int != 0
        || implicit as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn command_line_scan(mut parmp: *mut mparm_T) {
    let mut argc: ::core::ffi::c_int = (*parmp).argc;
    let mut argv: *mut *mut ::core::ffi::c_char = (*parmp).argv;
    let mut argv_idx: ::core::ffi::c_int = 0;
    let mut had_minmin: bool = false_0 != 0;
    let mut want_argument: bool = false;
    let mut n: ::core::ffi::c_int = 0;
    argc -= 1;
    argv = argv.offset(1);
    argv_idx = 1 as ::core::ffi::c_int;
    while argc > 0 as ::core::ffi::c_int {
        if *(*argv.offset(0 as ::core::ffi::c_int as isize))
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '+' as ::core::ffi::c_int
            && !had_minmin
        {
            if (*parmp).n_commands >= MAX_ARG_CMDS {
                mainerr(
                    err_extra_cmd,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
            }
            argv_idx = -1 as ::core::ffi::c_int;
            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            {
                let c2rust_fresh6 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr = &raw mut (*parmp).commands[c2rust_fresh6 as usize];
                *c2rust_lvalue_ptr =
                    b"$\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                let c2rust_fresh7 = (*parmp).n_commands;
                (*parmp).n_commands = (*parmp).n_commands + 1;
                let c2rust_lvalue_ptr_0 = &raw mut (*parmp).commands[c2rust_fresh7 as usize];
                *c2rust_lvalue_ptr_0 = (*argv.offset(0 as ::core::ffi::c_int as isize))
                    .offset(1 as ::core::ffi::c_int as isize);
            }
        } else if *(*argv.offset(0 as ::core::ffi::c_int as isize))
            .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
            && !had_minmin
        {
            want_argument = false_0 != 0;
            let c2rust_fresh8 = argv_idx;
            argv_idx = argv_idx + 1;
            let mut c: ::core::ffi::c_char =
                *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(c2rust_fresh8 as isize);
            's_747: {
                'c_49604: {
                    match c as ::core::ffi::c_int {
                        NUL => {
                            if exmode_active {
                                silent_mode = true_0 != 0;
                                (*parmp).no_swap_file = true_0;
                            } else {
                                if (*parmp).edit_type > EDIT_STDIN as ::core::ffi::c_int {
                                    mainerr(
                                        err_too_many_args,
                                        *argv.offset(0 as ::core::ffi::c_int as isize),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                }
                                (*parmp).had_stdin_file = true_0 != 0;
                                (*parmp).edit_type = EDIT_STDIN as ::core::ffi::c_int;
                            }
                            argv_idx = -1 as ::core::ffi::c_int;
                            break 's_747;
                        }
                        45 => {
                            if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"help\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                usage();
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"version\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                version();
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"api-info\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                let mut data: String_0 = api_metadata_raw();
                                let written_bytes: ptrdiff_t =
                                    os_write(STDOUT_FILENO, data.data, data.size, false_0 != 0);
                                if written_bytes < 0 as ptrdiff_t {
                                    semsg(
                                        gettext(b"E5420: Failed to write to file: %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        uv_strerror(written_bytes as ::core::ffi::c_int),
                                    );
                                }
                                os_exit(0 as ::core::ffi::c_int);
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"headless\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                headless_mode = true_0 != 0;
                            } else if strcasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"embed\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            ) == 0 as ::core::ffi::c_int
                            {
                                embedded_mode = true_0 != 0;
                            } else if strncasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"listen\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                6 as ::core::ffi::c_int as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                want_argument = true_0 != 0;
                                argv_idx += 6 as ::core::ffi::c_int;
                            } else if strncasecmp(
                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize),
                                b"literal\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                7 as ::core::ffi::c_int as size_t,
                            ) != 0 as ::core::ffi::c_int
                            {
                                if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"remote\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    6 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    (*parmp).remote = (*parmp).argc - argc;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"server\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    6 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 6 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"noplugin\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    8 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    p_lpl = false_0;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    3 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 3 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"startuptime\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    11 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    want_argument = true_0 != 0;
                                    argv_idx += 11 as ::core::ffi::c_int;
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"clean\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    5 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    (*parmp).use_vimrc = b"NONE\0".as_ptr()
                                        as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                    (*parmp).clean = true_0 != 0;
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                    size: ::core::mem::size_of::<
                                                        [::core::ffi::c_char; 5],
                                                    >(
                                                    )
                                                    .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                } else if strncasecmp(
                                    (*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize),
                                    b"luamod-dev\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    9 as ::core::ffi::c_int as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    nlua_disable_preload = true_0 != 0;
                                } else {
                                    if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                        .offset(argv_idx as isize)
                                        != 0
                                    {
                                        mainerr(
                                            err_opt_unknown,
                                            *argv.offset(0 as ::core::ffi::c_int as isize),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                    }
                                    had_minmin = true_0 != 0;
                                }
                            }
                            if !want_argument {
                                argv_idx = -1 as ::core::ffi::c_int;
                            }
                            break 's_747;
                        }
                        65 => {
                            set_option_value_give_err(
                                kOptArabic,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as ::core::ffi::c_int,
                            );
                            break 's_747;
                        }
                        98 => {
                            set_options_bin(
                                (*curbuf).b_p_bin,
                                1 as ::core::ffi::c_int,
                                0 as ::core::ffi::c_int,
                            );
                            (*curbuf).b_p_bin = 1 as ::core::ffi::c_int;
                            break 's_747;
                        }
                        68 => {
                            (*parmp).use_debug_break_level = 9999 as ::core::ffi::c_int;
                            break 's_747;
                        }
                        100 => {
                            (*parmp).diff_mode = true_0;
                            break 's_747;
                        }
                        101 => {
                            exmode_active = true_0 != 0;
                            break 's_747;
                        }
                        69 => {
                            exmode_active = true_0 != 0;
                            (*parmp).input_istext = true_0 != 0;
                            break 's_747;
                        }
                        63 | 104 => {
                            usage();
                            os_exit(0 as ::core::ffi::c_int);
                        }
                        72 => {
                            set_option_value_give_err(
                                kOptKeymap,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"hebrew\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>(
                                            )
                                            .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                0 as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptRightleft,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kTrue },
                                },
                                0 as ::core::ffi::c_int,
                            );
                            break 's_747;
                        }
                        77 => {
                            reset_modifiable();
                        }
                        109 => {}
                        102 | 78 | 88 => {
                            break 's_747;
                        }
                        110 => {
                            (*parmp).no_swap_file = true_0;
                            break 's_747;
                        }
                        112 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_TABS as ::core::ffi::c_int;
                            break 's_747;
                        }
                        111 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_HOR as ::core::ffi::c_int;
                            break 's_747;
                        }
                        79 => {
                            (*parmp).window_count = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                0 as ::core::ffi::c_int,
                            );
                            (*parmp).window_layout = WIN_VER as ::core::ffi::c_int;
                            break 's_747;
                        }
                        113 => {
                            if (*parmp).edit_type != EDIT_NONE as ::core::ffi::c_int {
                                mainerr(
                                    err_too_many_args,
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_QF as ::core::ffi::c_int;
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                != 0
                            {
                                (*parmp).use_ef = (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
                            } else if argc > 1 as ::core::ffi::c_int {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        82 => {
                            readonlymode = true_0 != 0;
                            (*curbuf).b_p_ro = true_0;
                            p_uc = 10000 as OptInt;
                            break 's_747;
                        }
                        114 | 76 => {
                            recoverymode = true;
                            break 's_747;
                        }
                        115 => {
                            if exmode_active {
                                silent_mode = true_0 != 0;
                                (*parmp).no_swap_file = true_0;
                                if p_shadafile.is_null()
                                    || *p_shadafile as ::core::ffi::c_int == NUL
                                {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: String_0 {
                                                    data: b"NONE\0".as_ptr()
                                                        as *const ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_char,
                                                    size: ::core::mem::size_of::<
                                                        [::core::ffi::c_char; 5],
                                                    >(
                                                    )
                                                    .wrapping_sub(1 as size_t),
                                                },
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        116 => {
                            if (*parmp).edit_type != EDIT_NONE as ::core::ffi::c_int {
                                mainerr(
                                    err_too_many_args,
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                            }
                            (*parmp).edit_type = EDIT_TAG as ::core::ffi::c_int;
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                != 0
                            {
                                (*parmp).tagname = (*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
                            } else {
                                want_argument = true_0 != 0;
                            }
                            break 's_747;
                        }
                        118 => {
                            version();
                            os_exit(0 as ::core::ffi::c_int);
                        }
                        86 => {
                            p_verbose = get_number_arg(
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                &raw mut argv_idx,
                                10 as ::core::ffi::c_int,
                            ) as OptInt;
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                set_option_value_give_err(
                                    kOptVerbosefile,
                                    OptVal {
                                        type_0: kOptValTypeString,
                                        data: OptValData {
                                            string: cstr_as_string(
                                                (*argv.offset(0 as ::core::ffi::c_int as isize))
                                                    .offset(argv_idx as isize),
                                            ),
                                        },
                                    },
                                    0 as ::core::ffi::c_int,
                                );
                                argv_idx = strlen(*argv.offset(0 as ::core::ffi::c_int as isize))
                                    as ::core::ffi::c_int;
                            }
                            break 's_747;
                        }
                        119 => {
                            if ascii_isdigit(
                                *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                    .offset(argv_idx as isize)
                                    as ::core::ffi::c_int,
                            ) {
                                n = get_number_arg(
                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                    &raw mut argv_idx,
                                    10 as ::core::ffi::c_int,
                                );
                                set_option_value_give_err(
                                    kOptWindow,
                                    OptVal {
                                        type_0: kOptValTypeNumber,
                                        data: OptValData {
                                            number: n as OptInt,
                                        },
                                    },
                                    0 as ::core::ffi::c_int,
                                );
                                break 's_747;
                            } else {
                                want_argument = true_0 != 0;
                                break 's_747;
                            }
                        }
                        99 => {
                            if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                if (*parmp).n_commands >= MAX_ARG_CMDS {
                                    mainerr(
                                        err_extra_cmd,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                }
                                let c2rust_fresh9 = (*parmp).n_commands;
                                (*parmp).n_commands = (*parmp).n_commands + 1;
                                let c2rust_lvalue_ptr_1 =
                                    &raw mut (*parmp).commands[c2rust_fresh9 as usize];
                                *c2rust_lvalue_ptr_1 = (*argv
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .offset(argv_idx as isize);
                                argv_idx = -1 as ::core::ffi::c_int;
                                break 's_747;
                            } else {
                                break 'c_49604;
                            }
                        }
                        83 | 105 | 108 | 117 | 85 | 87 => {
                            break 'c_49604;
                        }
                        _ => {
                            mainerr(
                                err_opt_unknown,
                                *argv.offset(0 as ::core::ffi::c_int as isize),
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                        }
                    }
                    p_write = false_0;
                    break 's_747;
                }
                want_argument = true_0 != 0;
            }
            if want_argument {
                if *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(argv_idx as isize)
                    as ::core::ffi::c_int
                    != NUL
                {
                    mainerr(
                        err_opt_garbage,
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                }
                argc -= 1;
                if argc < 1 as ::core::ffi::c_int
                    && c as ::core::ffi::c_int != 'S' as ::core::ffi::c_int
                {
                    mainerr(
                        err_arg_missing,
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                }
                argv = argv.offset(1);
                argv_idx = -1 as ::core::ffi::c_int;
                's_1076: {
                    '_scripterror: {
                        's_1075: {
                            match c as ::core::ffi::c_int {
                                99 | 83 => {
                                    if (*parmp).n_commands >= MAX_ARG_CMDS {
                                        mainerr(
                                            err_extra_cmd,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                    }
                                    if c as ::core::ffi::c_int == 'S' as ::core::ffi::c_int {
                                        let mut a: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        if argc < 1 as ::core::ffi::c_int {
                                            a = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
                                        } else if *(*argv.offset(0 as ::core::ffi::c_int as isize))
                                            .offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int
                                        {
                                            a = SESSION_FILE.as_ptr() as *mut ::core::ffi::c_char;
                                            argc += 1;
                                            argv = argv.offset(-1);
                                        } else {
                                            a = *argv.offset(0 as ::core::ffi::c_int as isize);
                                        }
                                        let mut s_size: size_t =
                                            strlen(a).wrapping_add(9 as size_t);
                                        let mut s: *mut ::core::ffi::c_char =
                                            xmalloc(s_size) as *mut ::core::ffi::c_char;
                                        snprintf(
                                            s,
                                            s_size,
                                            b"so %s\0".as_ptr() as *const ::core::ffi::c_char,
                                            a,
                                        );
                                        (*parmp).cmds_tofree[(*parmp).n_commands as usize] =
                                            true_0 as ::core::ffi::c_char;
                                        let c2rust_fresh10 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_2 =
                                            &raw mut (*parmp).commands[c2rust_fresh10 as usize];
                                        *c2rust_lvalue_ptr_2 = s;
                                    } else {
                                        let c2rust_fresh11 = (*parmp).n_commands;
                                        (*parmp).n_commands = (*parmp).n_commands + 1;
                                        let c2rust_lvalue_ptr_3 =
                                            &raw mut (*parmp).commands[c2rust_fresh11 as usize];
                                        *c2rust_lvalue_ptr_3 =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                45 => {
                                    if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--cmd\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        if (*parmp).n_pre_commands >= MAX_ARG_CMDS {
                                            mainerr(
                                                err_extra_cmd,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                        }
                                        let c2rust_fresh12 = (*parmp).n_pre_commands;
                                        (*parmp).n_pre_commands = (*parmp).n_pre_commands + 1;
                                        let c2rust_lvalue_ptr_4 =
                                            &raw mut (*parmp).pre_commands[c2rust_fresh12 as usize];
                                        *c2rust_lvalue_ptr_4 =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--listen\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        (*parmp).listen_addr =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    } else if strequal(
                                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                                        b"--server\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) {
                                        (*parmp).server_addr =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                    }
                                    break 's_1075;
                                }
                                113 => {
                                    (*parmp).use_ef =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                105 => {
                                    set_option_value_give_err(
                                        kOptShadafile,
                                        OptVal {
                                            type_0: kOptValTypeString,
                                            data: OptValData {
                                                string: cstr_as_string(
                                                    *argv.offset(0 as ::core::ffi::c_int as isize),
                                                ),
                                            },
                                        },
                                        0 as ::core::ffi::c_int,
                                    );
                                    break 's_1075;
                                }
                                108 => {
                                    headless_mode = true_0 != 0;
                                    silent_mode = true_0 != 0;
                                    p_verbose = 1 as OptInt;
                                    (*parmp).no_swap_file = true_0;
                                    (*parmp).use_vimrc = (if !(*parmp).use_vimrc.is_null() {
                                        (*parmp).use_vimrc as *const ::core::ffi::c_char
                                    } else {
                                        b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                                    })
                                        as *mut ::core::ffi::c_char;
                                    if p_shadafile.is_null()
                                        || *p_shadafile as ::core::ffi::c_int == NUL
                                    {
                                        set_option_value_give_err(
                                            kOptShadafile,
                                            OptVal {
                                                type_0: kOptValTypeString,
                                                data: OptValData {
                                                    string: String_0 {
                                                        data: b"NONE\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                            as *mut ::core::ffi::c_char,
                                                        size: ::core::mem::size_of::<
                                                            [::core::ffi::c_char; 5],
                                                        >(
                                                        )
                                                        .wrapping_sub(1 as size_t),
                                                    },
                                                },
                                            },
                                            0 as ::core::ffi::c_int,
                                        );
                                    }
                                    (*parmp).luaf = *argv.offset(0 as ::core::ffi::c_int as isize);
                                    argc -= 1;
                                    if argc >= 0 as ::core::ffi::c_int {
                                        (*parmp).lua_arg0 = (*parmp).argc - argc;
                                        argc = 0 as ::core::ffi::c_int;
                                    }
                                    break 's_1075;
                                }
                                115 => {
                                    if !(*parmp).scriptin.is_null() {
                                        break '_scripterror;
                                    } else {
                                        (*parmp).scriptin =
                                            *argv.offset(0 as ::core::ffi::c_int as isize);
                                        break 's_1075;
                                    }
                                }
                                116 => {
                                    (*parmp).tagname =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                117 => {
                                    (*parmp).use_vimrc =
                                        *argv.offset(0 as ::core::ffi::c_int as isize);
                                    break 's_1075;
                                }
                                119 => {
                                    if ascii_isdigit(
                                        **argv.offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) {
                                        argv_idx = 0 as ::core::ffi::c_int;
                                        n = get_number_arg(
                                            *argv.offset(0 as ::core::ffi::c_int as isize),
                                            &raw mut argv_idx,
                                            10 as ::core::ffi::c_int,
                                        );
                                        set_option_value_give_err(
                                            kOptWindow,
                                            OptVal {
                                                type_0: kOptValTypeNumber,
                                                data: OptValData {
                                                    number: n as OptInt,
                                                },
                                            },
                                            0 as ::core::ffi::c_int,
                                        );
                                        argv_idx = -1 as ::core::ffi::c_int;
                                        break 's_1075;
                                    }
                                }
                                87 => {}
                                85 | _ => {
                                    break 's_1075;
                                }
                            }
                            if !(*parmp).scriptout.is_null() {
                                break '_scripterror;
                            } else {
                                (*parmp).scriptout = *argv.offset(0 as ::core::ffi::c_int as isize);
                                (*parmp).scriptout_append =
                                    c as ::core::ffi::c_int == 'w' as ::core::ffi::c_int;
                            }
                        }
                        break 's_1076;
                    }
                    vim_snprintf(
                        &raw mut IObuff as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        gettext(b"Attempt to open script file again: \"%s %s\"\n\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *argv.offset(-1 as ::core::ffi::c_int as isize),
                        *argv.offset(0 as ::core::ffi::c_int as isize),
                    );
                    fprintf(
                        stderr,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut IObuff as *mut ::core::ffi::c_char,
                    );
                    os_exit(2 as ::core::ffi::c_int);
                }
            }
        } else {
            argv_idx = -1 as ::core::ffi::c_int;
            if (*parmp).edit_type > EDIT_STDIN as ::core::ffi::c_int {
                mainerr(
                    err_too_many_args,
                    *argv.offset(0 as ::core::ffi::c_int as isize),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
            }
            (*parmp).edit_type = EDIT_FILE as ::core::ffi::c_int;
            ga_grow(&raw mut global_alist.al_ga, 1 as ::core::ffi::c_int);
            let mut p: *mut ::core::ffi::c_char =
                xstrdup(*argv.offset(0 as ::core::ffi::c_int as isize));
            if (*parmp).diff_mode != 0
                && os_isdir(p) as ::core::ffi::c_int != 0
                && global_alist.al_ga.ga_len > 0 as ::core::ffi::c_int
                && !os_isdir(alist_name(
                    (global_alist.al_ga.ga_data as *mut aentry_T)
                        .offset(0 as ::core::ffi::c_int as isize),
                ))
            {
                let mut r: *mut ::core::ffi::c_char = concat_fnames(
                    p,
                    path_tail(alist_name(
                        (global_alist.al_ga.ga_data as *mut aentry_T)
                            .offset(0 as ::core::ffi::c_int as isize),
                    )),
                    true_0 != 0,
                );
                xfree(p as *mut ::core::ffi::c_void);
                p = r;
            }
            let mut alist_fnum_flag: ::core::ffi::c_int =
                if edit_stdin(parmp) as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                };
            alist_add(&raw mut global_alist, p, alist_fnum_flag);
        }
        if argv_idx <= 0 as ::core::ffi::c_int
            || *(*argv.offset(0 as ::core::ffi::c_int as isize)).offset(argv_idx as isize)
                as ::core::ffi::c_int
                == NUL
        {
            argc -= 1;
            argv = argv.offset(1);
            argv_idx = 1 as ::core::ffi::c_int;
        }
    }
    if embedded_mode as ::core::ffi::c_int != 0
        && (silent_mode as ::core::ffi::c_int != 0 || !(*parmp).luaf.is_null())
    {
        mainerr(
            gettext(b"--embed conflicts with -es/-Es/-l\0".as_ptr() as *const ::core::ffi::c_char),
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if (*parmp).n_commands > 0 as ::core::ffi::c_int {
        let swcmd_len: size_t =
            strlen((*parmp).commands[0 as ::core::ffi::c_int as usize]).wrapping_add(2 as size_t);
        let swcmd: *mut ::core::ffi::c_char =
            xmalloc(swcmd_len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        snprintf(
            swcmd,
            swcmd_len.wrapping_add(1 as size_t),
            b":%s\r\0".as_ptr() as *const ::core::ffi::c_char,
            (*parmp).commands[0 as ::core::ffi::c_int as usize],
        );
        set_vim_var_string(VV_SWAPCOMMAND, swcmd, swcmd_len as ptrdiff_t);
        xfree(swcmd as *mut ::core::ffi::c_void);
    }
    if !time_fd.is_null() {
        time_msg(
            b"parsing arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn set_argf_var() {
    let mut list: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < global_alist.al_ga.ga_len {
        let mut fname: *mut ::core::ffi::c_char =
            alist_name((global_alist.al_ga.ga_data as *mut aentry_T).offset(i as isize));
        if !fname.is_null() {
            vim_FullName(
                fname,
                &raw mut NameBuff as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                false_0 != 0,
            );
            tv_list_append_string(
                list,
                &raw mut NameBuff as *mut ::core::ffi::c_char,
                -1 as ssize_t,
            );
        }
        i += 1;
    }
    tv_list_set_lock(list, VAR_FIXED);
    set_vim_var_list(VV_ARGF, list);
}
unsafe extern "C" fn init_params(
    mut paramp: *mut mparm_T,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    memset(
        paramp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<mparm_T>(),
    );
    (*paramp).argc = argc;
    (*paramp).argv = argv;
    (*paramp).use_debug_break_level = -1 as ::core::ffi::c_int;
    (*paramp).window_count = -1 as ::core::ffi::c_int;
    (*paramp).listen_addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).server_addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).remote = 0 as ::core::ffi::c_int;
    (*paramp).luaf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*paramp).lua_arg0 = -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn init_startuptime(mut paramp: *mut mparm_T) {
    let mut is_embed: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*paramp).argc - 1 as ::core::ffi::c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i as isize),
            b"--embed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            is_embed = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < (*paramp).argc - 1 as ::core::ffi::c_int {
        if strcasecmp(
            *(*paramp).argv.offset(i_0 as isize),
            b"--startuptime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            time_init(
                *(*paramp)
                    .argv
                    .offset((i_0 + 1 as ::core::ffi::c_int) as isize),
                if is_embed as ::core::ffi::c_int != 0 {
                    b"Embedded\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"Primary (or UI client)\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            time_start(b"--- NVIM STARTING ---\0".as_ptr() as *const ::core::ffi::c_char);
            break;
        } else {
            i_0 += 1;
        }
    }
}
unsafe extern "C" fn check_and_set_isatty(mut _paramp: *mut mparm_T) {
    stdin_isatty = os_isatty(STDIN_FILENO);
    stdout_isatty = os_isatty(STDOUT_FILENO);
    stderr_isatty = os_isatty(STDERR_FILENO);
    if !time_fd.is_null() {
        time_msg(
            b"window checked\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn init_path(mut exename: *const ::core::ffi::c_char) {
    let mut exepath: [::core::ffi::c_char; 4096] = [
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
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
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
    let mut exepathlen: size_t = MAXPATHL as size_t;
    if os_exepath(
        &raw mut exepath as *mut ::core::ffi::c_char,
        &raw mut exepathlen,
    ) != 0 as ::core::ffi::c_int
    {
        path_guess_exepath(
            exename,
            &raw mut exepath as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        );
    }
    set_vim_var_string(
        VV_PROGPATH,
        &raw mut exepath as *mut ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    set_vim_var_string(VV_PROGNAME, path_tail(exename), -1 as ptrdiff_t);
}
unsafe extern "C" fn get_fname(mut _parmp: *mut mparm_T) -> *mut ::core::ffi::c_char {
    return alist_name(
        (global_alist.al_ga.ga_data as *mut aentry_T).offset(0 as ::core::ffi::c_int as isize),
    );
}
unsafe extern "C" fn set_window_layout(mut paramp: *mut mparm_T) {
    if (*paramp).diff_mode != 0 && (*paramp).window_layout == 0 as ::core::ffi::c_int {
        if diffopt_horizontal() {
            (*paramp).window_layout = WIN_HOR as ::core::ffi::c_int;
        } else {
            (*paramp).window_layout = WIN_VER as ::core::ffi::c_int;
        }
    }
}
unsafe extern "C" fn handle_quickfix(mut paramp: *mut mparm_T) {
    if (*paramp).edit_type == EDIT_QF as ::core::ffi::c_int {
        if !(*paramp).use_ef.is_null() {
            set_option_direct(
                kOptErrorfile,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string((*paramp).use_ef),
                    },
                },
                0 as ::core::ffi::c_int,
                SID_CARG,
            );
        }
        vim_snprintf(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"cfile %s\0".as_ptr() as *const ::core::ffi::c_char,
            p_ef,
        );
        if qf_init(
            ::core::ptr::null_mut::<win_T>(),
            p_ef,
            p_efm,
            true_0,
            &raw mut IObuff as *mut ::core::ffi::c_char,
            p_menc,
        ) < 0 as ::core::ffi::c_int
        {
            msg_putchar('\n' as ::core::ffi::c_int);
            os_exit(3 as ::core::ffi::c_int);
        }
        if !time_fd.is_null() {
            time_msg(
                b"reading errorfile\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
}
unsafe extern "C" fn handle_tag(mut tagname: *mut ::core::ffi::c_char) {
    if !tagname.is_null() {
        swap_exists_did_quit = false_0 != 0;
        vim_snprintf(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"ta %s\0".as_ptr() as *const ::core::ffi::c_char,
            tagname,
        );
        do_cmdline_cmd(&raw mut IObuff as *mut ::core::ffi::c_char);
        if !time_fd.is_null() {
            time_msg(
                b"jumping to tag\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if swap_exists_did_quit {
            ui_call_error_exit(1 as Integer);
            getout(1 as ::core::ffi::c_int);
        }
    }
}
unsafe extern "C" fn read_stdin() {
    swap_exists_action = SEA_DIALOG;
    no_wait_return = true_0;
    let mut save_msg_didany: bool = msg_didany;
    if !(*curbuf).b_ffname.is_null() {
        let mut stdin_buf: *mut buf_T = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            BLN_LISTED as ::core::ffi::c_int,
        );
        if stdin_buf.is_null() {
            semsg(b"Failed to create buffer for stdin\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        let mut initial_buf_handle: handle_T = (*curbuf).handle;
        set_curbuf(stdin_buf, 0 as ::core::ffi::c_int, false_0 != 0);
        readfile(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            ::core::ptr::null_mut::<exarg_T>(),
            READ_NEW as ::core::ffi::c_int + READ_STDIN as ::core::ffi::c_int,
            true_0 != 0,
        );
        let mut stdin_buf_handle: handle_T = (*stdin_buf).handle;
        let mut stdin_buf_empty: bool = buf_is_empty(curbuf);
        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
            b"silent! buffer %d\0".as_ptr() as *const ::core::ffi::c_char,
            initial_buf_handle,
        );
        do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
        if stdin_buf_empty {
            vim_snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                b"silent! bwipeout! %d\0".as_ptr() as *const ::core::ffi::c_char,
                stdin_buf_handle,
            );
            do_cmdline_cmd(&raw mut buf as *mut ::core::ffi::c_char);
        }
    } else {
        set_buflisted(true_0);
        open_buffer(
            true_0 != 0,
            ::core::ptr::null_mut::<exarg_T>(),
            0 as ::core::ffi::c_int,
        );
        if buf_is_empty(curbuf) as ::core::ffi::c_int != 0 && !(*curbuf).b_next.is_null() {
            do_cmdline_cmd(b"silent! bnext\0".as_ptr() as *const ::core::ffi::c_char);
            do_cmdline_cmd(b"silent! bwipeout 1\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    no_wait_return = false_0;
    msg_didany = save_msg_didany;
    if !time_fd.is_null() {
        time_msg(
            b"reading stdin\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    check_swap_exists_action();
}
unsafe extern "C" fn create_windows(mut parmp: *mut mparm_T) {
    if (*parmp).window_count == -1 as ::core::ffi::c_int {
        (*parmp).window_count = 1 as ::core::ffi::c_int;
    }
    if (*parmp).window_count == 0 as ::core::ffi::c_int {
        (*parmp).window_count = global_alist.al_ga.ga_len;
    }
    if (*parmp).window_count > 1 as ::core::ffi::c_int {
        if (*parmp).window_layout == 0 as ::core::ffi::c_int {
            (*parmp).window_layout = WIN_HOR as ::core::ffi::c_int;
        }
        if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
            (*parmp).window_count = make_tabpages((*parmp).window_count);
            if !time_fd.is_null() {
                time_msg(
                    b"making tab pages\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else if (*firstwin).w_next.is_null()
            || (*(*firstwin).w_next).w_floating as ::core::ffi::c_int != 0
        {
            (*parmp).window_count = make_windows(
                (*parmp).window_count,
                (*parmp).window_layout == WIN_VER as ::core::ffi::c_int,
            );
            if !time_fd.is_null() {
                time_msg(
                    b"making windows\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
        } else {
            (*parmp).window_count = win_count();
        }
    } else {
        (*parmp).window_count = 1 as ::core::ffi::c_int;
    }
    if recoverymode {
        msg_scroll = true_0;
        ml_recover(true_0 != 0);
        if (*curbuf).b_ml.ml_mfp.is_null() {
            getout(1 as ::core::ffi::c_int);
        }
        do_modelines(0 as ::core::ffi::c_int);
    } else {
        let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        autocmd_no_enter += 1;
        autocmd_no_leave += 1;
        let mut dorewind: bool = true_0 != 0;
        loop {
            let c2rust_fresh0 = done;
            done = done + 1;
            if c2rust_fresh0 >= 1000 as ::core::ffi::c_int {
                break;
            }
            if dorewind {
                if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                    goto_tabpage(1 as ::core::ffi::c_int);
                } else {
                    curwin = firstwin;
                }
            } else if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                if (*curtab).tp_next.is_null() {
                    break;
                }
                goto_tabpage(0 as ::core::ffi::c_int);
            } else {
                if (*curwin).w_next.is_null() {
                    break;
                }
                curwin = (*curwin).w_next;
            }
            dorewind = false_0 != 0;
            curbuf = (*curwin).w_buffer;
            if (*curbuf).b_ml.ml_mfp.is_null() {
                if p_fdls >= 0 as OptInt {
                    (*curwin).w_onebuf_opt.wo_fdl = p_fdls;
                }
                swap_exists_action = SEA_DIALOG;
                set_buflisted(true_0);
                open_buffer(
                    false_0 != 0,
                    ::core::ptr::null_mut::<exarg_T>(),
                    0 as ::core::ffi::c_int,
                );
                if swap_exists_action == SEA_QUIT {
                    if got_int as ::core::ffi::c_int != 0
                        || only_one_window() as ::core::ffi::c_int != 0
                    {
                        did_emsg = false_0;
                        ui_call_error_exit(1 as Integer);
                        getout(1 as ::core::ffi::c_int);
                    }
                    setfname(
                        curbuf,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        false_0 != 0,
                    );
                    (*curwin).w_arg_idx = -1 as ::core::ffi::c_int;
                    swap_exists_action = SEA_NONE;
                } else {
                    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
                }
                dorewind = true_0 != 0;
            }
            os_breakcheck();
            if !got_int {
                continue;
            }
            vgetc();
            break;
        }
        if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
            goto_tabpage(1 as ::core::ffi::c_int);
        } else {
            curwin = firstwin;
        }
        curbuf = (*curwin).w_buffer;
        autocmd_no_enter -= 1;
        autocmd_no_leave -= 1;
    };
}
unsafe extern "C" fn edit_buffers(mut parmp: *mut mparm_T) {
    let mut arg_idx: ::core::ffi::c_int = 0;
    let mut advance: bool = true_0 != 0;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut p_shm_save: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    autocmd_no_enter += 1;
    autocmd_no_leave += 1;
    if (*curwin).w_arg_idx == -1 as ::core::ffi::c_int {
        win_close(curwin, true_0 != 0, false_0 != 0);
        advance = false_0 != 0;
    }
    arg_idx = 1 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*parmp).window_count {
        if (*curwin).w_arg_idx == -1 as ::core::ffi::c_int {
            arg_idx += 1;
            win_close(curwin, true_0 != 0, false_0 != 0);
            advance = false_0 != 0;
        } else {
            if advance {
                if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
                    if (*curtab).tp_next.is_null() {
                        break;
                    }
                    goto_tabpage(0 as ::core::ffi::c_int);
                    if i == 1 as ::core::ffi::c_int {
                        let mut buf: [::core::ffi::c_char; 100] = [0; 100];
                        p_shm_save = xstrdup(p_shm);
                        snprintf(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                            b"F%s\0".as_ptr() as *const ::core::ffi::c_char,
                            p_shm,
                        );
                        set_option_value_give_err(
                            kOptShortmess,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(
                                        &raw mut buf as *mut ::core::ffi::c_char,
                                    ),
                                },
                            },
                            0 as ::core::ffi::c_int,
                        );
                    }
                } else {
                    if (*curwin).w_next.is_null() {
                        break;
                    }
                    win_enter((*curwin).w_next, false_0 != 0);
                }
            }
            advance = true_0 != 0;
            if curbuf == (*firstwin).w_buffer || (*curbuf).b_ffname.is_null() {
                (*curwin).w_arg_idx = arg_idx;
                swap_exists_did_quit = false_0 != 0;
                do_ecmd(
                    0 as ::core::ffi::c_int,
                    if arg_idx < global_alist.al_ga.ga_len {
                        alist_name(
                            (global_alist.al_ga.ga_data as *mut aentry_T).offset(arg_idx as isize),
                        )
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    },
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_LASTL as ::core::ffi::c_int as linenr_T,
                    ECMD_HIDE as ::core::ffi::c_int,
                    curwin,
                );
                if swap_exists_did_quit {
                    if got_int as ::core::ffi::c_int != 0
                        || only_one_window() as ::core::ffi::c_int != 0
                    {
                        did_emsg = false_0;
                        ui_call_error_exit(1 as Integer);
                        getout(1 as ::core::ffi::c_int);
                    }
                    win_close(curwin, true_0 != 0, false_0 != 0);
                    advance = false_0 != 0;
                }
                if arg_idx == global_alist.al_ga.ga_len - 1 as ::core::ffi::c_int {
                    arg_had_last = true_0 != 0;
                }
                arg_idx += 1;
            }
            os_breakcheck();
            if got_int {
                vgetc();
                break;
            }
        }
        i += 1;
    }
    if !p_shm_save.is_null() {
        set_option_value_give_err(
            kOptShortmess,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(p_shm_save),
                },
            },
            0 as ::core::ffi::c_int,
        );
        xfree(p_shm_save as *mut ::core::ffi::c_void);
    }
    if (*parmp).window_layout == WIN_TABS as ::core::ffi::c_int {
        goto_tabpage(1 as ::core::ffi::c_int);
    }
    autocmd_no_enter -= 1;
    win = firstwin;
    while (*win).w_onebuf_opt.wo_pvw != 0 {
        win = (*win).w_next;
        if !win.is_null() {
            continue;
        }
        win = firstwin;
        break;
    }
    win_enter(win, false_0 != 0);
    autocmd_no_leave -= 1;
    if !time_fd.is_null() {
        time_msg(
            b"editing files in windows\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    if (*parmp).window_count > 1 as ::core::ffi::c_int
        && (*parmp).window_layout != WIN_TABS as ::core::ffi::c_int
    {
        win_equal(curwin, false_0 != 0, 'b' as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn exe_pre_commands(mut parmp: *mut mparm_T) {
    let mut cmds: *mut *mut ::core::ffi::c_char =
        &raw mut (*parmp).pre_commands as *mut *mut ::core::ffi::c_char;
    let mut cnt: ::core::ffi::c_int = (*parmp).n_pre_commands;
    if cnt <= 0 as ::core::ffi::c_int {
        return;
    }
    (*curwin).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    estack_push(
        ETYPE_ARGS,
        gettext(b"pre-vimrc command line\0".as_ptr() as *const ::core::ffi::c_char),
        0 as linenr_T,
    );
    current_sctx.sc_sid = SID_CMDARG as scid_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < cnt {
        do_cmdline_cmd(*cmds.offset(i as isize));
        i += 1;
    }
    estack_pop();
    current_sctx.sc_sid = 0 as ::core::ffi::c_int as scid_T;
    if !time_fd.is_null() {
        time_msg(
            b"--cmd commands\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn exe_commands(mut parmp: *mut mparm_T) {
    msg_scroll = true_0;
    if (*parmp).tagname.is_null() && (*curwin).w_cursor.lnum <= 1 as linenr_T {
        (*curwin).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    estack_push(
        ETYPE_ARGS,
        b"command line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        0 as linenr_T,
    );
    current_sctx.sc_sid = SID_CARG as scid_T;
    current_sctx.sc_seq = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*parmp).n_commands {
        do_cmdline_cmd((*parmp).commands[i as usize]);
        if (*parmp).cmds_tofree[i as usize] != 0 {
            xfree((*parmp).commands[i as usize] as *mut ::core::ffi::c_void);
        }
        i += 1;
    }
    estack_pop();
    current_sctx.sc_sid = 0 as ::core::ffi::c_int as scid_T;
    if (*curwin).w_cursor.lnum == 0 as linenr_T {
        (*curwin).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    if !exmode_active {
        msg_scroll = false_0;
    }
    if (*parmp).edit_type == EDIT_QF as ::core::ffi::c_int {
        qf_jump(
            ::core::ptr::null_mut::<qf_info_T>(),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0,
        );
    }
    if !time_fd.is_null() {
        time_msg(
            b"executing command arguments\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn do_system_initialization() {
    let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let sysinit_suffix: [::core::ffi::c_char; 13] = [
            PATHSEP as ::core::ffi::c_char,
            's' as ::core::ffi::c_char,
            'y' as ::core::ffi::c_char,
            's' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            'n' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            't' as ::core::ffi::c_char,
            '.' as ::core::ffi::c_char,
            'v' as ::core::ffi::c_char,
            'i' as ::core::ffi::c_char,
            'm' as ::core::ffi::c_char,
            NUL as ::core::ffi::c_char,
        ];
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as ::core::ffi::c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let mut path_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 13]>());
            let mut vimrc: *mut ::core::ffi::c_char = xmalloc(path_len) as *mut ::core::ffi::c_char;
            memcpy(
                vimrc as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            if *vimrc.offset(dir_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                != PATHSEP
            {
                *vimrc.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
                dir_len = dir_len.wrapping_add(1 as size_t);
            }
            memcpy(
                vimrc.offset(dir_len as isize) as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                vimrc.offset(dir_len as isize).offset(appname_len as isize)
                    as *mut ::core::ffi::c_void,
                &raw const sysinit_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>(),
            );
            if do_source(
                vimrc,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != FAIL
            {
                xfree(vimrc as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                return;
            }
            xfree(vimrc as *mut ::core::ffi::c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut ::core::ffi::c_void);
    }
    do_source(
        SYS_VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
}
unsafe extern "C" fn do_user_initialization() -> bool {
    let mut do_exrc: bool = p_exrc != 0;
    if execute_env(b"VIMINIT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char)
        == OK
    {
        do_exrc = p_exrc != 0;
        return do_exrc;
    }
    let mut init_lua_path: *mut ::core::ffi::c_char =
        stdpaths_user_conf_subpath(b"init.lua\0".as_ptr() as *const ::core::ffi::c_char);
    let mut user_vimrc: *mut ::core::ffi::c_char =
        stdpaths_user_conf_subpath(b"init.vim\0".as_ptr() as *const ::core::ffi::c_char);
    if os_path_exists(init_lua_path) as ::core::ffi::c_int != 0
        && do_source(
            init_lua_path,
            true_0 != 0,
            DOSO_VIMRC as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) != 0
    {
        if os_path_exists(user_vimrc) {
            semsg(
                &raw const e_conflicting_configs as *const ::core::ffi::c_char,
                init_lua_path,
                user_vimrc,
            );
        }
        xfree(user_vimrc as *mut ::core::ffi::c_void);
        xfree(init_lua_path as *mut ::core::ffi::c_void);
        do_exrc = p_exrc != 0;
        return do_exrc;
    }
    xfree(init_lua_path as *mut ::core::ffi::c_void);
    if do_source(
        user_vimrc,
        true_0 != 0,
        DOSO_VIMRC as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) != FAIL
    {
        do_exrc = p_exrc != 0;
        if do_exrc {
            do_exrc = path_full_compare(
                VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
                user_vimrc,
                false_0 != 0,
                true_0 != 0,
            ) as ::core::ffi::c_uint
                != kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        xfree(user_vimrc as *mut ::core::ffi::c_void);
        return do_exrc;
    }
    xfree(user_vimrc as *mut ::core::ffi::c_void);
    let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    if !config_dirs.is_null() {
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ':' as ::core::ffi::c_char,
                config_dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            let init_lua_suffix: [::core::ffi::c_char; 10] = [
                PATHSEP as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'n' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                't' as ::core::ffi::c_char,
                '.' as ::core::ffi::c_char,
                'l' as ::core::ffi::c_char,
                'u' as ::core::ffi::c_char,
                'a' as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            let mut init_lua_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>());
            let mut init_lua: *mut ::core::ffi::c_char =
                xmalloc(init_lua_len) as *mut ::core::ffi::c_char;
            memcpy(
                init_lua as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            *init_lua.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                init_lua
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset(appname_len as isize) as *mut ::core::ffi::c_void,
                &raw const init_lua_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            );
            let init_vim_suffix: [::core::ffi::c_char; 10] = [
                PATHSEP as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'n' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                't' as ::core::ffi::c_char,
                '.' as ::core::ffi::c_char,
                'v' as ::core::ffi::c_char,
                'i' as ::core::ffi::c_char,
                'm' as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            let mut init_vim_len: size_t = dir_len
                .wrapping_add(1 as size_t)
                .wrapping_add(appname_len)
                .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 10]>());
            let mut init_vim: *mut ::core::ffi::c_char =
                xmalloc(init_vim_len) as *mut ::core::ffi::c_char;
            memcpy(
                init_vim as *mut ::core::ffi::c_void,
                dir as *const ::core::ffi::c_void,
                dir_len,
            );
            *init_vim.offset(dir_len as isize) = PATHSEP as ::core::ffi::c_char;
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            memcpy(
                init_vim
                    .offset(dir_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    .offset(appname_len as isize) as *mut ::core::ffi::c_void,
                &raw const init_vim_suffix as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>(),
            );
            if os_path_exists(init_lua) as ::core::ffi::c_int != 0
                && do_source(
                    init_lua,
                    true_0 != 0,
                    DOSO_VIMRC as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ) != 0
            {
                if os_path_exists(init_vim) {
                    semsg(
                        &raw const e_conflicting_configs as *const ::core::ffi::c_char,
                        init_lua,
                        init_vim,
                    );
                }
                xfree(init_vim as *mut ::core::ffi::c_void);
                xfree(init_lua as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                do_exrc = p_exrc != 0;
                return do_exrc;
            }
            xfree(init_lua as *mut ::core::ffi::c_void);
            if do_source(
                init_vim,
                true_0 != 0,
                DOSO_VIMRC as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != FAIL
            {
                do_exrc = p_exrc != 0;
                if do_exrc {
                    do_exrc = path_full_compare(
                        VIMRC_FILE.as_ptr() as *mut ::core::ffi::c_char,
                        init_vim,
                        false_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        != kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint;
                }
                xfree(init_vim as *mut ::core::ffi::c_void);
                xfree(config_dirs as *mut ::core::ffi::c_void);
                return do_exrc;
            }
            xfree(init_vim as *mut ::core::ffi::c_void);
            if iter.is_null() {
                break;
            }
        }
        xfree(config_dirs as *mut ::core::ffi::c_void);
    }
    if execute_env(b"EXINIT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char)
        == OK
    {
        do_exrc = p_exrc != 0;
        return do_exrc;
    }
    return do_exrc;
}
unsafe extern "C" fn do_exrc_initialization() {
    let L: *mut lua_State = get_global_lstate();
    '_c2rust_label: {
        if !L.is_null() {
        } else {
            __assert_fail(
                b"L\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/main.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2207 as ::core::ffi::c_uint,
                b"void do_exrc_initialization(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_getfield(
        L,
        LUA_GLOBALSINDEX,
        b"require\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushstring(
        L,
        b"vim._core.exrc\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if nlua_pcall(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            lua_tolstring(
                L,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ),
        );
    }
}
unsafe extern "C" fn source_startup_scripts(parmp: *const mparm_T) {
    if !(*parmp).use_vimrc.is_null() {
        if !(strequal(
            (*parmp).use_vimrc,
            b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
            || strequal(
                (*parmp).use_vimrc,
                b"NORC\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0)
        {
            if do_source(
                (*parmp).use_vimrc,
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) != OK
            {
                semsg(
                    gettext(&raw const e_cannot_read_from_str_2 as *const ::core::ffi::c_char),
                    (*parmp).use_vimrc,
                );
            }
        }
    } else if !silent_mode {
        do_system_initialization();
        if do_user_initialization() {
            do_exrc_initialization();
        }
    }
    if !time_fd.is_null() {
        time_msg(
            b"sourcing vimrc file(s)\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
unsafe extern "C" fn execute_env(mut env: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut initstr: *mut ::core::ffi::c_char = os_getenv(env);
    if initstr.is_null() {
        return FAIL;
    }
    estack_push(ETYPE_ENV, env, 0 as linenr_T);
    let save_current_sctx: sctx_T = current_sctx;
    current_sctx.sc_sid = SID_ENV as scid_T;
    current_sctx.sc_seq = 0 as ::core::ffi::c_int;
    current_sctx.sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
    do_cmdline_cmd(initstr);
    estack_pop();
    current_sctx = save_current_sctx;
    xfree(initstr as *mut ::core::ffi::c_void);
    return OK;
}
unsafe extern "C" fn mainerr(
    mut msg1: *const ::core::ffi::c_char,
    mut msg2: *const ::core::ffi::c_char,
    mut msg3: *const ::core::ffi::c_char,
) -> ! {
    print_mainerr(msg1, msg2, msg3);
    os_exit(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn print_mainerr(
    mut msg1: *const ::core::ffi::c_char,
    mut msg2: *const ::core::ffi::c_char,
    mut msg3: *const ::core::ffi::c_char,
) {
    let mut prgname: *mut ::core::ffi::c_char = path_tail(argv0);
    signal_stop();
    fprintf(
        stderr,
        b"%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
        prgname,
        gettext(msg1),
    );
    if !msg2.is_null() {
        fprintf(
            stderr,
            b": \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            msg2,
        );
    }
    if !msg3.is_null() {
        fprintf(
            stderr,
            b": \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            msg3,
        );
    }
    fprintf(
        stderr,
        gettext(b"\nMore info with \"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    fprintf(
        stderr,
        b"%s -h\"\n\0".as_ptr() as *const ::core::ffi::c_char,
        prgname,
    );
}
unsafe extern "C" fn version() {
    nlua_init(
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        -1 as ::core::ffi::c_int,
    );
    info_message = true_0 != 0;
    list_version();
    msg_putchar('\n' as ::core::ffi::c_int);
    msg_didout = false_0 != 0;
}
unsafe extern "C" fn usage() {
    signal_stop();
    printf(gettext(b"Usage:\n\0".as_ptr() as *const ::core::ffi::c_char));
    printf(gettext(
        b"  nvim [options] [file ...]\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"\nOptions:\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"  --cmd <cmd>           Execute <cmd> before any config\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  +<cmd>, -c <cmd>      Execute <cmd> after config and first file\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -l <script> [args...] Execute Lua <script> (with optional args)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -S <session>          Source <session> after loading the first file\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -s <scriptin>         Read Normal mode commands from <scriptin>\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -u <config>           Use this config file\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    printf(gettext(
        b"  -d                    Diff mode\n\0".as_ptr() as *const ::core::ffi::c_char
    ));
    printf(gettext(
        b"  -es, -Es              Silent (batch) mode\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -h, --help            Print this help message\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -i <shada>            Use this shada file\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -n                    No swap file, use memory only\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -o[N]                 Open N windows (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -O[N]                 Open N vertical windows (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -p[N]                 Open N tab pages (default: one per file)\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -R                    Read-only (view) mode\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -v, --version         Print version information\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  -V[N][file]           Verbose [level][file]\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    printf(gettext(
        b"  --                    Only file names after this\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --api-info            Write msgpack-encoded API metadata to stdout\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --clean               \"Factory defaults\" (skip user config and plugins, shada)\n\0"
            .as_ptr() as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --embed               Use stdin/stdout as a msgpack-rpc channel\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --headless            Don't start a user interface\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --listen <address>    Serve RPC API from this address\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --remote[-subcommand] Execute commands remotely on a server\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --server <address>    Connect to this Nvim server\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"  --startuptime <file>  Write startup timing messages to <file>\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
    printf(gettext(
        b"\nSee \":help startup-options\" for all options.\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    ));
}
unsafe extern "C" fn check_swap_exists_action() {
    if swap_exists_action == SEA_QUIT {
        ui_call_error_exit(1 as Integer);
        getout(1 as ::core::ffi::c_int);
    }
    handle_swap_exists(::core::ptr::null_mut::<bufref_T>());
}
#[no_mangle]
pub static mut tslua_query_parse_count: uint64_t = 0 as uint64_t;
pub const MAX_ARG_CMDS: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
#[no_mangle]
pub static mut namedfm: [xfmark_T; 36] = [
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0 as linenr_T,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
];
#[no_mangle]
pub static mut ch_before_blocking_events: *mut MultiQueue = ::core::ptr::null_mut::<MultiQueue>();
#[no_mangle]
pub static mut showcmd_buf: [::core::ffi::c_char; 41] = [0; 41];
#[no_mangle]
pub static mut repeat_luaref: LuaRef = -2 as LuaRef;
#[no_mangle]
pub static mut used_stdin: bool = false;
#[no_mangle]
pub static mut nvim_testing: bool = false;
#[no_mangle]
pub static mut pum_grid: ScreenGrid = ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    rows: 0 as ::core::ffi::c_int,
    cols: 0 as ::core::ffi::c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as ::core::ffi::c_int,
    comp_row: 0 as ::core::ffi::c_int,
    comp_col: 0 as ::core::ffi::c_int,
    comp_width: 0 as ::core::ffi::c_int,
    comp_height: 0 as ::core::ffi::c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
};
#[no_mangle]
pub static mut pum_want: C2Rust_Unnamed_46 = C2Rust_Unnamed_46 {
    active: false,
    item: 0,
    insert: false,
    finish: false,
};
#[no_mangle]
pub static mut tab_page_click_defs: *mut StlClickDefinition =
    ::core::ptr::null_mut::<StlClickDefinition>();
#[no_mangle]
pub static mut tab_page_click_defs_size: size_t = 0 as size_t;
#[no_mangle]
pub static mut noargs: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
#[no_mangle]
pub static mut ui_event_ns_id: uint32_t = 0 as uint32_t;
#[no_mangle]
pub static mut resize_events: *mut MultiQueue = ::core::ptr::null_mut::<MultiQueue>();
#[no_mangle]
pub static mut ui_refresh_cmdheight: bool = true;
#[no_mangle]
pub static mut grid_line_buf_size: size_t = 0 as size_t;
#[no_mangle]
pub static mut grid_line_buf_char: *mut schar_T = ::core::ptr::null_mut::<schar_T>();
#[no_mangle]
pub static mut grid_line_buf_attr: *mut sattr_T = ::core::ptr::null_mut::<sattr_T>();
#[no_mangle]
pub static mut ui_client_channel_id: uint64_t = 0 as uint64_t;
#[no_mangle]
pub static mut ui_client_error_exit: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut ui_client_exit_status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut ui_client_attached: bool = false;
#[no_mangle]
pub static mut ui_client_forward_stdin: bool = false;
#[no_mangle]
pub static mut tabpage_move_disallowed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub static mut float_anchor_str: [*const ::core::ffi::c_char; 4] = [
    b"NW\0".as_ptr() as *const ::core::ffi::c_char,
    b"NE\0".as_ptr() as *const ::core::ffi::c_char,
    b"SW\0".as_ptr() as *const ::core::ffi::c_char,
    b"SE\0".as_ptr() as *const ::core::ffi::c_char,
];
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WRITEBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wb\0") };
pub const APPENDBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ab\0") };
pub fn main() {
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args()
        .map(|arg| {
            ::std::ffi::CString::new(arg)
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut ::core::ffi::c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut ::core::ffi::c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe {
        ::std::process::exit(main_0(
            (args_ptrs.len() - 1) as ::core::ffi::c_int,
            args_ptrs.as_mut_ptr() as *mut *mut ::core::ffi::c_char,
        ) as i32)
    }
}
unsafe extern "C" fn c2rust_run_static_initializers() {
    kTVCstring = (18446744073709551615 as size_t).wrapping_sub(1 as size_t);
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
