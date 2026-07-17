extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
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
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fgets(
        __s: *mut ::core::ffi::c_char,
        __n: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> *mut ::core::ffi::c_char;
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
    fn memchr(
        __s: *const ::core::ffi::c_void,
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
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
    fn uv_mutex_init(handle: *mut uv_mutex_t) -> ::core::ffi::c_int;
    fn uv_mutex_lock(handle: *mut uv_mutex_t);
    fn uv_mutex_unlock(handle: *mut uv_mutex_t);
    fn try_malloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcnt(data: *const ::core::ffi::c_void, c: ::core::ffi::c_char, len: size_t) -> size_t;
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
    fn mh_get_String(set: *mut Set_String, key: String_0) -> uint32_t;
    fn mh_put_String(set: *mut Set_String, key: String_0, new: *mut MHPutStatus) -> uint32_t;
    fn map_put_ref_String_int(
        map: *mut Map_String_int,
        key: String_0,
        key_alloc: *mut *mut String_0,
        new_item: *mut bool,
    ) -> *mut ::core::ffi::c_int;
    fn map_ref_String_int(
        map: *mut Map_String_int,
        key: String_0,
        key_alloc: *mut *mut String_0,
    ) -> *mut ::core::ffi::c_int;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_autocmd(event: event_T, sfname: *mut ::core::ffi::c_char, buf: *mut buf_T) -> bool;
    static mut p_enc: *mut ::core::ffi::c_char;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_ic: ::core::ffi::c_int;
    static mut p_lpl: ::core::ffi::c_int;
    static mut p_pp: *mut ::core::ffi::c_char;
    static mut p_rtp: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
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
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipwhite_len(p: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite_esc(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skip_to_newline(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn globpath(
        path: *mut ::core::ffi::c_char,
        file: *mut ::core::ffi::c_char,
        ga: *mut garray_T,
        expand_options: ::core::ffi::c_int,
        dirs: bool,
    );
    fn dbg_find_breakpoint(
        file: bool,
        fname: *mut ::core::ffi::c_char,
        after: linenr_T,
    ) -> linenr_T;
    fn has_profiling(file: bool, fname: *mut ::core::ffi::c_char, fp: *mut bool) -> bool;
    fn dbg_breakpoint(name: *mut ::core::ffi::c_char, lnum: linenr_T);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_argreq: [::core::ffi::c_char; 0];
    static e_interr: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_norange: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    static e_dirnotf: [::core::ffi::c_char; 0];
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn api_clear_error(value: *mut Error);
    fn eval_to_number(expr: *mut ::core::ffi::c_char, use_simple_function: bool) -> varnumber_T;
    fn get_copyID() -> ::core::ffi::c_int;
    static mut hash_removed: ::core::ffi::c_char;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn message_filtered(msg: *const ::core::ffi::c_char) -> bool;
    fn msg_ext_ui_flush();
    fn verbose_enter();
    fn verbose_leave();
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T);
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_dict(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        dict: *mut dict_T,
    ) -> ::core::ffi::c_int;
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
    fn tv_dict_add_func(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        fp: *mut ufunc_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_copy(
        conv: *const vimconv_T,
        orig: *mut dict_T,
        deep: bool,
        copyID: ::core::ffi::c_int,
    ) -> *mut dict_T;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_dict_alloc_lock(lock: VarLockStatus) -> *mut dict_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_opt_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn func_tbl_get() -> *mut hashtab_T;
    fn save_funccal(entry: *mut funccal_entry_T);
    fn restore_funccal();
    fn new_script_vars(id: scid_T);
    fn aborting() -> bool;
    fn report_make_pending(pending: ::core::ffi::c_int, value: *mut ::core::ffi::c_void);
    fn cleanup_conditionals(
        cstack: *mut cstack_T,
        searched_cond: ::core::ffi::c_int,
        inclusive: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn getline_equal(
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        func: LineGetter,
    ) -> bool;
    fn getline_cookie(
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
    ) -> *mut ::core::ffi::c_void;
    fn do_exedit(eap: *mut exarg_T, old_curwin: *mut win_T);
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_set_growsize(gap: *mut garray_T, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_remove_duplicate_strings(gap: *mut garray_T);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn openscript(name: *mut ::core::ffi::c_char, directly: bool);
    static mut msg_col: ::core::ffi::c_int;
    static mut ex_nesting_level: ::core::ffi::c_int;
    static mut debug_break_level: ::core::ffi::c_int;
    static mut debug_tick: ::core::ffi::c_int;
    static mut do_profiling: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut did_source_packages: bool;
    static mut curbuf: *mut buf_T;
    static mut cmdmod: cmdmod_T;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut got_int: bool;
    static mut global_busy: ::core::ffi::c_int;
    static mut listcmd_busy: bool;
    static mut time_fd: *mut FILE;
    fn add_win_cmd_modifiers(
        buf: *mut ::core::ffi::c_char,
        cmod: *const cmdmod_T,
        multi_mods: *mut bool,
    ) -> size_t;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn enc_canonize(enc: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn convert_setup(
        vcp: *mut vimconv_T,
        from: *mut ::core::ffi::c_char,
        to: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn string_convert(
        vcp: *const vimconv_T,
        ptr: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn nlua_exec_ga(ga: *mut garray_T, name: *mut ::core::ffi::c_char);
    fn nlua_exec(
        str: String_0,
        chunkname: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_is_deferred_safe() -> bool;
    fn nlua_exec_file(path: *const ::core::ffi::c_char) -> bool;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_open(
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_file_is_readable(name: *const ::core::ffi::c_char) -> bool;
    fn set_option_value_give_err(opt_idx: OptIndex, value: OptVal, opt_flags: ::core::ffi::c_int);
    fn vimrc_found(fname: *mut ::core::ffi::c_char, envname: *mut ::core::ffi::c_char);
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn line_breakcheck();
    static mut default_lib_dir: *mut ::core::ffi::c_char;
    fn os_setenv(
        name: *const ::core::ffi::c_char,
        value: *const ::core::ffi::c_char,
        overwrite: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
    fn vim_env_iter(
        delim: ::core::ffi::c_char,
        val: *const ::core::ffi::c_char,
        iter: *const ::core::ffi::c_void,
        dir: *mut *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_void;
    fn vim_env_iter_rev(
        delim: ::core::ffi::c_char,
        val: *const ::core::ffi::c_char,
        iter: *const ::core::ffi::c_void,
        dir: *mut *const ::core::ffi::c_char,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_void;
    fn vim_get_prefix_from_exepath(exe_name: *mut ::core::ffi::c_char);
    fn vim_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn home_replace_save(
        buf: *mut buf_T,
        src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn get_appname(namelike: bool) -> *const ::core::ffi::c_char;
    fn stdpaths_get_xdg_var(idx: XDGVarType) -> *mut ::core::ffi::c_char;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_past_head(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn vim_ispathsep_nocolon(c: ::core::ffi::c_int) -> bool;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn path_fnamencmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn concat_fnames(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        sep: bool,
    ) -> *mut ::core::ffi::c_char;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn gen_expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn path_with_extension(
        path: *const ::core::ffi::c_char,
        extension: *const ::core::ffi::c_char,
    ) -> bool;
    fn fix_fname(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn append_path(
        path: *mut ::core::ffi::c_char,
        to_append: *const ::core::ffi::c_char,
        max_len: size_t,
    ) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn profile_start() -> proftime_T;
    fn profile_end(tm: proftime_T) -> proftime_T;
    fn profile_zero() -> proftime_T;
    fn profile_add(tm1: proftime_T, tm2: proftime_T) -> proftime_T;
    fn profile_self(self_0: proftime_T, total: proftime_T, children: proftime_T) -> proftime_T;
    fn profile_sub_wait(tm: proftime_T, tma: proftime_T) -> proftime_T;
    fn prof_child_enter(tm: *mut proftime_T);
    fn prof_child_exit(tm: *mut proftime_T);
    fn profile_init(si: *mut scriptitem_T);
    fn script_line_start();
    fn script_line_end();
    fn time_push(rel: *mut proftime_T, start: *mut proftime_T);
    fn time_pop(tp: proftime_T);
    fn time_msg(mesg: *const ::core::ffi::c_char, start: *const proftime_T);
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type size_t = usize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
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
pub type ssize_t = isize;
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
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
pub type uv_mutex_t = pthread_mutex_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_String {
    pub h: MapHash,
    pub keys: *mut String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_String_int {
    pub set: Set_String,
    pub values: *mut ::core::ffi::c_int,
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_13 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_13 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_13 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_13 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_13 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_13 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_13 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_13 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_13 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_13 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_13 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_13 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_13 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_13 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_13 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_13 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_13 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_13 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_13 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_13 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_13 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_13 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_13 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_13 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_13 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_13 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_13 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_13 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_13 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_13 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_13 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_13 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_13 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_13 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_13 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_13 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_13 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_13 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_13 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_13 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_13 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_13 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_13 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_13 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_13 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_13 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_13 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_13 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_13 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_13 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_13 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_13 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_13 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_13 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_13 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_13 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_13 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_13 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_13 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_13 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_13 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_13 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_13 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_13 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_13 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_13 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_13 = -2;
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
    pub cs_pend: C2Rust_Unnamed_14,
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
pub union C2Rust_Unnamed_14 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CSTP_FINISH: C2Rust_Unnamed_15 = 32;
pub const CSTP_RETURN: C2Rust_Unnamed_15 = 24;
pub const CSTP_CONTINUE: C2Rust_Unnamed_15 = 16;
pub const CSTP_BREAK: C2Rust_Unnamed_15 = 8;
pub const CSTP_THROW: C2Rust_Unnamed_15 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_15 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_15 = 1;
pub const CSTP_NONE: C2Rust_Unnamed_15 = 0;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_16 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_16 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_16 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_16 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_16 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_16 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_16 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_16 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_16 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_16 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_16 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_16 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_16 = 1;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_17 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_17 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_17 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_17 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_17 = 1;
pub const CONV_NONE: C2Rust_Unnamed_17 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
pub type XDGVarType = ::core::ffi::c_int;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccal_entry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut funccal_entry_T,
}
pub type funccal_entry_T = funccal_entry;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_18 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_18 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_18 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_18 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_18 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_18 = 1;
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
    pub es_info: C2Rust_Unnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type estack_arg_T = ::core::ffi::c_uint;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct scriptvar_T {
    pub sv_var: ScopeDictDictItem,
    pub sv_dict: dict_T,
}
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
pub type DoInRuntimepathCB = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut *mut ::core::ffi::c_char,
        bool,
        *mut ::core::ffi::c_void,
    ) -> bool,
>;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_20 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_20 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_20 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_20 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_20 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_20 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_20 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_20 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_20 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_20 = 512;
pub const EW_ICASE: C2Rust_Unnamed_20 = 256;
pub const EW_PATH: C2Rust_Unnamed_20 = 128;
pub const EW_EXEC: C2Rust_Unnamed_20 = 64;
pub const EW_SILENT: C2Rust_Unnamed_20 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_20 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_20 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_20 = 4;
pub const EW_FILE: C2Rust_Unnamed_20 = 2;
pub const EW_DIR: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const DOSO_VIMRC: C2Rust_Unnamed_21 = 1;
pub const DOSO_NONE: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_22 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_22 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_22 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_22 = 32;
pub const DIP_OPT: C2Rust_Unnamed_22 = 16;
pub const DIP_START: C2Rust_Unnamed_22 = 8;
pub const DIP_ERR: C2Rust_Unnamed_22 = 4;
pub const DIP_DIR: C2Rust_Unnamed_22 = 2;
pub const DIP_ALL: C2Rust_Unnamed_22 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct source_cookie_T {
    pub fp: *mut FILE,
    pub nextline: *mut ::core::ffi::c_char,
    pub sourcing_lnum: linenr_T,
    pub finished: bool,
    pub source_from_buf_or_str: bool,
    pub buf_lnum: ::core::ffi::c_int,
    pub buflines: garray_T,
    pub breakpoint: linenr_T,
    pub fname: *mut ::core::ffi::c_char,
    pub dbg_tick: ::core::ffi::c_int,
    pub level: ::core::ffi::c_int,
    pub conv: vimconv_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RuntimeSearchPath {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SearchPathItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchPathItem {
    pub path: *mut ::core::ffi::c_char,
    pub after: bool,
    pub pack_inserted: bool,
    pub has_lua: TriState,
    pub pos_in_rtp: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut String_0,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
static mut value_init_int: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_String = Set_String {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<String_0>(),
};
pub const MAP_INIT: Map_String_int = Map_String_int {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_String(mut set: *mut Set_String, mut key: String_0) -> bool {
    return mh_get_String(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_String(
    mut set: *mut Set_String,
    mut key: String_0,
    mut key_alloc: *mut *mut String_0,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_String(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut String_0>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const SYS_OPTWIN_FILE: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"$VIMRUNTIME/scripts/optwin.lua\0",
    )
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_CONCAT: ::core::ffi::c_int = 'C' as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline(always)]
unsafe extern "C" fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    (*tv).v_type = VAR_LIST;
    (*tv).vval.v_list = l;
    tv_list_ref(l);
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_MODELINE: ::core::ffi::c_int = -1;
pub const SID_CMDARG: ::core::ffi::c_int = -2;
pub const SID_CARG: ::core::ffi::c_int = -3;
pub const SID_ENV: ::core::ffi::c_int = -4;
pub const SID_ERROR: ::core::ffi::c_int = -5;
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7;
pub const SID_LUA: ::core::ffi::c_int = -8;
pub const SID_API_CLIENT: ::core::ffi::c_int = -9;
pub const SID_STR: ::core::ffi::c_int = -10;
#[no_mangle]
pub static mut exestack: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<estack_T>() as ::core::ffi::c_int,
    ga_growsize: 50 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
#[no_mangle]
pub static mut script_items: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<*mut scriptitem_T>() as ::core::ffi::c_int,
    ga_growsize: 20 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
static mut ga_loaded: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
static mut last_current_SID_seq: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn estack_init() {
    ga_grow(&raw mut exestack, 10 as ::core::ffi::c_int);
    let mut entry: *mut estack_T =
        (exestack.ga_data as *mut estack_T).offset(exestack.ga_len as isize);
    (*entry).es_type = ETYPE_TOP;
    (*entry).es_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*entry).es_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*entry).es_info.ufunc = ::core::ptr::null_mut::<ufunc_T>();
    exestack.ga_len += 1;
}
#[no_mangle]
pub unsafe extern "C" fn estack_push(
    mut type_0: etype_T,
    mut name: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
) -> *mut estack_T {
    ga_grow(&raw mut exestack, 1 as ::core::ffi::c_int);
    let mut entry: *mut estack_T =
        (exestack.ga_data as *mut estack_T).offset(exestack.ga_len as isize);
    (*entry).es_type = type_0;
    (*entry).es_name = name;
    (*entry).es_lnum = lnum;
    (*entry).es_info.ufunc = ::core::ptr::null_mut::<ufunc_T>();
    exestack.ga_len += 1;
    return entry;
}
#[no_mangle]
pub unsafe extern "C" fn estack_push_ufunc(mut ufunc: *mut ufunc_T, mut lnum: linenr_T) {
    let mut entry: *mut estack_T = estack_push(
        ETYPE_UFUNC,
        if !(*ufunc).uf_name_exp.is_null() {
            (*ufunc).uf_name_exp
        } else {
            &raw mut (*ufunc).uf_name as *mut ::core::ffi::c_char
        },
        lnum,
    );
    if !entry.is_null() {
        (*entry).es_info.ufunc = ufunc;
    }
}
#[no_mangle]
pub unsafe extern "C" fn estack_pop() {
    if exestack.ga_len > 1 as ::core::ffi::c_int {
        exestack.ga_len -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn estack_sfile(mut which: estack_arg_T) -> *mut ::core::ffi::c_char {
    let mut entry: *const estack_T = (exestack.ga_data as *mut estack_T)
        .offset(exestack.ga_len as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    if which as ::core::ffi::c_uint == ESTACK_SFILE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*entry).es_type as ::core::ffi::c_uint
            != ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return if !(*entry).es_name.is_null() {
            xstrdup((*entry).es_name)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
    if which as ::core::ffi::c_uint == ESTACK_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint {
        let mut idx: ::core::ffi::c_int = exestack.ga_len - 1 as ::core::ffi::c_int;
        while idx >= 0 as ::core::ffi::c_int {
            if (*entry).es_type as ::core::ffi::c_uint
                == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*entry).es_type as ::core::ffi::c_uint
                    == ETYPE_AUCMD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let def_ctx: *const sctx_T = if (*entry).es_type as ::core::ffi::c_uint
                    == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    &raw mut (*(*entry).es_info.ufunc).uf_script_ctx
                } else {
                    &raw mut (*(*entry).es_info.aucmd).script_ctx
                };
                return if (*def_ctx).sc_sid > 0 as ::core::ffi::c_int {
                    xstrdup(
                        (**(script_items.ga_data as *mut *mut scriptitem_T).offset(
                            ((*def_ctx).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as isize,
                        ))
                        .sn_name,
                    )
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                };
            } else if (*entry).es_type as ::core::ffi::c_uint
                == ETYPE_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return xstrdup((*entry).es_name);
            }
            idx -= 1;
            entry = entry.offset(-1);
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
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
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    let mut last_type: etype_T = ETYPE_SCRIPT;
    let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx_0 < exestack.ga_len {
        entry = (exestack.ga_data as *mut estack_T).offset(idx_0 as isize);
        if !(*entry).es_name.is_null() {
            let mut type_name: String_0 = String_0 {
                data: b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 1]>().wrapping_sub(1 as size_t),
            };
            let mut es_name: String_0 = cstr_as_string((*entry).es_name);
            if (*entry).es_type as ::core::ffi::c_uint != last_type as ::core::ffi::c_uint {
                match (*entry).es_type as ::core::ffi::c_uint {
                    1 => {
                        type_name = String_0 {
                            data: b"script \0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                                .wrapping_sub(1 as size_t),
                        };
                    }
                    2 => {
                        type_name = String_0 {
                            data: b"function \0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                                .wrapping_sub(1 as size_t),
                        };
                    }
                    _ => {}
                }
                last_type = (*entry).es_type;
            }
            let mut lnum: linenr_T = if idx_0 == exestack.ga_len - 1 as ::core::ffi::c_int {
                if which as ::core::ffi::c_uint
                    == ESTACK_STACK as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum
                } else {
                    0 as linenr_T
                }
            } else {
                (*entry).es_lnum
            };
            let mut len: size_t = es_name
                .size
                .wrapping_add(type_name.size)
                .wrapping_add(26 as size_t);
            ga_grow(&raw mut ga, len as ::core::ffi::c_int);
            ga_concat_len(&raw mut ga, type_name.data, type_name.size);
            ga_concat_len(&raw mut ga, es_name.data, es_name.size);
            if lnum != 0 as linenr_T {
                ga.ga_len += vim_snprintf_safelen(
                    (ga.ga_data as *mut ::core::ffi::c_char).offset(ga.ga_len as isize),
                    (ga.ga_maxlen - ga.ga_len) as size_t,
                    b"[%d]\0".as_ptr() as *const ::core::ffi::c_char,
                    lnum,
                ) as ::core::ffi::c_int;
            }
            if idx_0 != exestack.ga_len - 1 as ::core::ffi::c_int {
                ga_concat_len(
                    &raw mut ga,
                    b"..\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                );
            }
        }
        idx_0 += 1;
    }
    if !ga.ga_data.is_null() {
        ga_append(&raw mut ga, NUL as uint8_t);
    }
    return ga.ga_data as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn stacktrace_push_item(
    l: *mut list_T,
    fp: *mut ufunc_T,
    event: *const ::core::ffi::c_char,
    lnum: linenr_T,
    filepath: *mut ::core::ffi::c_char,
) {
    let d: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
    let mut tv: typval_T = typval_T {
        v_type: VAR_DICT,
        v_lock: VAR_LOCKED,
        vval: typval_vval_union { v_dict: d },
    };
    if !fp.is_null() {
        tv_dict_add_func(
            d,
            b"funcref\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            fp,
        );
    }
    if !event.is_null() {
        tv_dict_add_str(
            d,
            b"event\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            event,
        );
    }
    tv_dict_add_nr(
        d,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        lnum as varnumber_T,
    );
    tv_dict_add_str(
        d,
        b"filepath\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        filepath,
    );
    tv_list_append_tv(l, &raw mut tv);
}
#[no_mangle]
pub unsafe extern "C" fn stacktrace_create() -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(exestack.ga_len as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < exestack.ga_len {
        let entry: *mut estack_T = (exestack.ga_data as *mut estack_T).offset(i as isize);
        let mut lnum: linenr_T = (*entry).es_lnum;
        if (*entry).es_type as ::core::ffi::c_uint
            == ETYPE_SCRIPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            stacktrace_push_item(
                l,
                ::core::ptr::null_mut::<ufunc_T>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                lnum,
                (*entry).es_name,
            );
        } else if (*entry).es_type as ::core::ffi::c_uint
            == ETYPE_UFUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let fp: *mut ufunc_T = (*entry).es_info.ufunc;
            let sctx: sctx_T = (*fp).uf_script_ctx;
            let mut filepath: *mut ::core::ffi::c_char = (if sctx.sc_sid > 0 as ::core::ffi::c_int {
                get_scriptname(sctx, ::core::ptr::null_mut::<bool>()) as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            lnum += sctx.sc_lnum;
            stacktrace_push_item(
                l,
                fp,
                ::core::ptr::null::<::core::ffi::c_char>(),
                lnum,
                filepath,
            );
        } else if (*entry).es_type as ::core::ffi::c_uint
            == ETYPE_AUCMD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let sctx_0: sctx_T = (*(*entry).es_info.aucmd).script_ctx;
            let mut filepath_0: *mut ::core::ffi::c_char =
                (if sctx_0.sc_sid > 0 as ::core::ffi::c_int {
                    get_scriptname(sctx_0, ::core::ptr::null_mut::<bool>())
                        as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
            lnum += sctx_0.sc_lnum;
            stacktrace_push_item(
                l,
                ::core::ptr::null_mut::<ufunc_T>(),
                (*entry).es_name,
                lnum,
                filepath_0,
            );
        }
        i += 1;
    }
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn f_getstacktrace(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_set_ret(rettv, stacktrace_create());
}
static mut runtime_search_path_valid: bool = false_0 != 0;
static mut runtime_search_path_valid_thread: bool = false_0 != 0;
static mut runtime_search_path_ref: *mut ::core::ffi::c_int =
    ::core::ptr::null_mut::<::core::ffi::c_int>();
static mut runtime_search_path: RuntimeSearchPath = RuntimeSearchPath {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<SearchPathItem>(),
};
static mut runtime_search_path_thread: RuntimeSearchPath = RuntimeSearchPath {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<SearchPathItem>(),
};
static mut runtime_search_path_mutex: uv_mutex_t = pthread_mutex_t {
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
};
#[no_mangle]
pub unsafe extern "C" fn runtime_init() {
    uv_mutex_init(&raw mut runtime_search_path_mutex);
}
unsafe extern "C" fn get_runtime_cmd_flags(
    mut argp: *mut *mut ::core::ffi::c_char,
    mut where_len: size_t,
) -> ::core::ffi::c_int {
    let mut arg: *mut ::core::ffi::c_char = *argp;
    if where_len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    if strncmp(
        arg,
        b"START\0".as_ptr() as *const ::core::ffi::c_char,
        where_len,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite(arg.offset(where_len as isize));
        return DIP_START as ::core::ffi::c_int + DIP_NORTP as ::core::ffi::c_int;
    }
    if strncmp(
        arg,
        b"OPT\0".as_ptr() as *const ::core::ffi::c_char,
        where_len,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite(arg.offset(where_len as isize));
        return DIP_OPT as ::core::ffi::c_int + DIP_NORTP as ::core::ffi::c_int;
    }
    if strncmp(
        arg,
        b"PACK\0".as_ptr() as *const ::core::ffi::c_char,
        where_len,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite(arg.offset(where_len as isize));
        return DIP_START as ::core::ffi::c_int
            + DIP_OPT as ::core::ffi::c_int
            + DIP_NORTP as ::core::ffi::c_int;
    }
    if strncmp(
        arg,
        b"ALL\0".as_ptr() as *const ::core::ffi::c_char,
        where_len,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite(arg.offset(where_len as isize));
        return DIP_START as ::core::ffi::c_int + DIP_OPT as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ex_runtime(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut flags: ::core::ffi::c_int = if (*eap).forceit != 0 {
        DIP_ALL as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
    flags += get_runtime_cmd_flags(&raw mut arg, p.offset_from(arg) as size_t);
    '_c2rust_label: {
        if !arg.is_null() {
        } else {
            __assert_fail(
                b"arg != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                345 as ::core::ffi::c_uint,
                b"void ex_runtime(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    source_runtime(arg, flags);
}
static mut runtime_expand_flags: ::core::ffi::c_int = 0;
#[no_mangle]
pub unsafe extern "C" fn set_context_in_runtime_cmd(
    mut xp: *mut expand_T,
    mut arg: *const ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
    runtime_expand_flags = if *p as ::core::ffi::c_int != NUL {
        get_runtime_cmd_flags(
            &raw mut arg as *mut *mut ::core::ffi::c_char,
            p.offset_from(arg) as size_t,
        )
    } else {
        0 as ::core::ffi::c_int
    };
    loop {
        p = skiptowhite_esc(arg);
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        if runtime_expand_flags == 0 as ::core::ffi::c_int {
            runtime_expand_flags = DIP_ALL as ::core::ffi::c_int;
        }
        arg = skipwhite(p);
    }
    (*xp).xp_context = EXPAND_RUNTIME as ::core::ffi::c_int;
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn source_callback_vim_lua(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut did_one: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        if path_with_extension(
            *fnames.offset(i as isize),
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            do_source(
                *fnames.offset(i as isize),
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                cookie as *mut ::core::ffi::c_int,
            );
            did_one = true_0 != 0;
            if !all {
                return true_0 != 0;
            }
        }
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < num_fnames {
        if path_with_extension(
            *fnames.offset(i_0 as isize),
            b"lua\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            do_source(
                *fnames.offset(i_0 as isize),
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                cookie as *mut ::core::ffi::c_int,
            );
            did_one = true_0 != 0;
            if !all {
                return true_0 != 0;
            }
        }
        i_0 += 1;
    }
    return did_one;
}
unsafe extern "C" fn source_callback(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut did_one: bool = source_callback_vim_lua(num_fnames, fnames, all, cookie);
    if !all && did_one as ::core::ffi::c_int != 0 {
        return true_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        if !path_with_extension(
            *fnames.offset(i as isize),
            b"vim\0".as_ptr() as *const ::core::ffi::c_char,
        ) && !path_with_extension(
            *fnames.offset(i as isize),
            b"lua\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            do_source(
                *fnames.offset(i as isize),
                false_0 != 0,
                DOSO_NONE as ::core::ffi::c_int,
                cookie as *mut ::core::ffi::c_int,
            );
            did_one = true_0 != 0;
            if !all {
                return true_0 != 0;
            }
        }
        i += 1;
    }
    return did_one;
}
#[no_mangle]
pub unsafe extern "C" fn do_in_path(
    mut path: *const ::core::ffi::c_char,
    mut prefix: *const ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut did_one: bool = false_0 != 0;
    let mut rtp_copy: *mut ::core::ffi::c_char = xstrdup(path);
    let mut buf: *mut ::core::ffi::c_char =
        xmallocz(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if p_verbose > 10 as OptInt && !name.is_null() {
        verbose_enter();
        if *prefix as ::core::ffi::c_int != NUL {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Searching for \"%s\" under \"%s\" in \"%s\"\0".as_ptr()
                    as *const ::core::ffi::c_char),
                name,
                prefix,
                path,
            );
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Searching for \"%s\" in \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                name,
                path,
            );
        }
        verbose_leave();
    }
    let mut do_all: bool = flags & DIP_ALL as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    let mut rtp: *mut ::core::ffi::c_char = rtp_copy;
    while *rtp as ::core::ffi::c_int != NUL && (do_all as ::core::ffi::c_int != 0 || !did_one) {
        let mut buflen: size_t = copy_option_part(
            &raw mut rtp,
            buf,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if flags & (DIP_NOAFTER as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int) != 0 {
            let mut is_after: bool = path_is_after(buf, buflen);
            if is_after as ::core::ffi::c_int != 0 && flags & DIP_NOAFTER as ::core::ffi::c_int != 0
                || !is_after && flags & DIP_AFTER as ::core::ffi::c_int != 0
            {
                continue;
            }
        }
        if name.is_null() {
            Some(callback.expect("non-null function pointer")).expect("non-null function pointer")(
                1 as ::core::ffi::c_int,
                &raw mut buf,
                do_all,
                cookie,
            );
            did_one = true_0 != 0;
        } else if buflen
            .wrapping_add(2 as size_t)
            .wrapping_add(strlen(prefix))
            .wrapping_add(strlen(name))
            < MAXPATHL as size_t
        {
            add_pathsep(buf);
            strcat(buf, prefix);
            tail = buf.offset(strlen(buf) as isize);
            let mut np: *mut ::core::ffi::c_char = name;
            while *np as ::core::ffi::c_int != NUL
                && (do_all as ::core::ffi::c_int != 0 || !did_one)
            {
                '_c2rust_label: {
                    if 4096 as isize >= tail.offset_from(buf) {
                    } else {
                        __assert_fail(
                            b"MAXPATHL >= (tail - buf)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            482 as ::core::ffi::c_uint,
                            b"int do_in_path(const char *, const char *, char *, int, DoInRuntimepathCB, void *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                copy_option_part(
                    &raw mut np,
                    tail,
                    (MAXPATHL as isize - tail.offset_from(buf)) as size_t,
                    b"\t \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if p_verbose > 10 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Searching for \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                        buf,
                    );
                    verbose_leave();
                }
                let mut ew_flags: ::core::ffi::c_int =
                    (if flags & DIP_DIR as ::core::ffi::c_int != 0 {
                        EW_DIR as ::core::ffi::c_int
                    } else {
                        EW_FILE as ::core::ffi::c_int
                    }) | (if flags & DIP_DIRFILE as ::core::ffi::c_int != 0 {
                        EW_DIR as ::core::ffi::c_int | EW_FILE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    });
                did_one = did_one as ::core::ffi::c_int
                    | (gen_expand_wildcards_and_cb(
                        1 as ::core::ffi::c_int,
                        &raw mut buf,
                        ew_flags,
                        do_all,
                        callback,
                        cookie,
                    ) == OK) as ::core::ffi::c_int
                    != 0;
            }
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(rtp_copy as *mut ::core::ffi::c_void);
    if !did_one && !name.is_null() {
        let mut basepath: *mut ::core::ffi::c_char = (if path == p_rtp as *const ::core::ffi::c_char
        {
            b"runtimepath\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"packpath\0".as_ptr() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        if flags & DIP_ERR as ::core::ffi::c_int != 0 {
            semsg(
                gettext(&raw const e_dirnotf as *const ::core::ffi::c_char),
                basepath,
                name,
            );
        } else if p_verbose > 1 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"not found in '%s': \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                basepath,
                name,
            );
            verbose_leave();
        }
    }
    return if did_one as ::core::ffi::c_int != 0 {
        OK
    } else {
        FAIL
    };
}
unsafe extern "C" fn runtime_search_path_get_cached(
    mut ref_0: *mut ::core::ffi::c_int,
) -> RuntimeSearchPath {
    runtime_search_path_validate();
    *ref_0 = 0 as ::core::ffi::c_int;
    if runtime_search_path_ref.is_null() {
        *ref_0 += 1;
        runtime_search_path_ref = ref_0;
    }
    return runtime_search_path;
}
unsafe extern "C" fn copy_runtime_search_path(src: RuntimeSearchPath) -> RuntimeSearchPath {
    let mut dst: RuntimeSearchPath = RuntimeSearchPath {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<SearchPathItem>(),
    };
    let mut j: size_t = 0 as size_t;
    while j < src.size {
        let mut item: SearchPathItem = *src.items.offset(j as isize);
        if dst.size == dst.capacity {
            dst.capacity = if dst.capacity != 0 {
                dst.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            dst.items = xrealloc(
                dst.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<SearchPathItem>().wrapping_mul(dst.capacity),
            ) as *mut SearchPathItem;
        } else {
        };
        let c2rust_fresh4 = dst.size;
        dst.size = dst.size.wrapping_add(1);
        *dst.items.offset(c2rust_fresh4 as isize) = SearchPathItem {
            path: xstrdup(item.path),
            after: item.after,
            pack_inserted: item.pack_inserted,
            has_lua: item.has_lua,
            pos_in_rtp: item.pos_in_rtp,
        };
        j = j.wrapping_add(1);
    }
    return dst;
}
unsafe extern "C" fn runtime_search_path_unref(
    mut path: RuntimeSearchPath,
    mut ref_0: *const ::core::ffi::c_int,
) {
    if *ref_0 != 0 {
        if runtime_search_path_ref == ref_0 as *mut ::core::ffi::c_int {
            runtime_search_path_ref = ::core::ptr::null_mut::<::core::ffi::c_int>();
        } else {
            runtime_search_path_free(path);
        }
    }
}
unsafe extern "C" fn do_in_cached_path(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut did_one: bool = false_0 != 0;
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if p_verbose > 10 as OptInt && !name.is_null() {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"Searching for \"%s\" in runtime path\0".as_ptr() as *const ::core::ffi::c_char
            ),
            name,
        );
        verbose_leave();
    }
    let mut ref_0: ::core::ffi::c_int = 0;
    let mut path: RuntimeSearchPath = runtime_search_path_get_cached(&raw mut ref_0);
    let mut do_all: bool = flags & DIP_ALL as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    let mut j: size_t = 0 as size_t;
    while j < path.size {
        let mut item: SearchPathItem = *path.items.offset(j as isize);
        let mut buflen: size_t = strlen(item.path);
        's_32: {
            if flags & (DIP_NOAFTER as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int) != 0 {
                if item.after as ::core::ffi::c_int != 0
                    && flags & DIP_NOAFTER as ::core::ffi::c_int != 0
                    || !item.after && flags & DIP_AFTER as ::core::ffi::c_int != 0
                {
                    break 's_32;
                }
            }
            if name.is_null() {
                Some(callback.expect("non-null function pointer"))
                    .expect("non-null function pointer")(
                    1 as ::core::ffi::c_int,
                    &raw mut item.path,
                    do_all,
                    cookie,
                );
            } else if buflen.wrapping_add(strlen(name)).wrapping_add(2 as size_t)
                < MAXPATHL as size_t
            {
                strcpy(&raw mut buf as *mut ::core::ffi::c_char, item.path);
                add_pathsep(&raw mut buf as *mut ::core::ffi::c_char);
                let mut tail: *mut ::core::ffi::c_char = (&raw mut buf as *mut ::core::ffi::c_char)
                    .offset(strlen(&raw mut buf as *mut ::core::ffi::c_char) as isize);
                let mut np: *mut ::core::ffi::c_char = name;
                while *np as ::core::ffi::c_int != NUL
                    && (do_all as ::core::ffi::c_int != 0 || !did_one)
                {
                    '_c2rust_label: {
                        if 4096 as isize
                            >= tail.offset_from(&raw mut buf as *mut ::core::ffi::c_char)
                        {
                        } else {
                            __assert_fail(
                                b"MAXPATHL >= (tail - buf)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                606 as ::core::ffi::c_uint,
                                b"int do_in_cached_path(char *, int, DoInRuntimepathCB, void *)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    copy_option_part(
                        &raw mut np,
                        tail,
                        (MAXPATHL as isize
                            - tail.offset_from(&raw mut buf as *mut ::core::ffi::c_char))
                            as size_t,
                        b"\t \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    if p_verbose > 10 as OptInt {
                        verbose_enter();
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Searching for \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            &raw mut buf as *mut ::core::ffi::c_char,
                        );
                        verbose_leave();
                    }
                    let mut ew_flags: ::core::ffi::c_int =
                        (if flags & DIP_DIR as ::core::ffi::c_int != 0 {
                            EW_DIR as ::core::ffi::c_int
                        } else {
                            EW_FILE as ::core::ffi::c_int
                        }) | (if flags & DIP_DIRFILE as ::core::ffi::c_int != 0 {
                            EW_DIR as ::core::ffi::c_int | EW_FILE as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) | EW_NOBREAK as ::core::ffi::c_int;
                    let mut pat: [*mut ::core::ffi::c_char; 1] =
                        [&raw mut buf as *mut ::core::ffi::c_char];
                    did_one = did_one as ::core::ffi::c_int
                        | (gen_expand_wildcards_and_cb(
                            1 as ::core::ffi::c_int,
                            &raw mut pat as *mut *mut ::core::ffi::c_char,
                            ew_flags,
                            do_all,
                            callback,
                            cookie,
                        ) == OK) as ::core::ffi::c_int
                        != 0;
                }
            }
        }
        j = j.wrapping_add(1);
    }
    if !did_one && !name.is_null() {
        if flags & DIP_ERR as ::core::ffi::c_int != 0 {
            semsg(
                gettext(&raw const e_dirnotf as *const ::core::ffi::c_char),
                b"runtime path\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
        } else if p_verbose > 1 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(
                    b"not found in runtime path: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                ),
                name,
            );
            verbose_leave();
        }
    }
    runtime_search_path_unref(path, &raw mut ref_0);
    return if did_one as ::core::ffi::c_int != 0 {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn runtime_inspect(mut arena: *mut Arena) -> Array {
    let mut path: RuntimeSearchPath = runtime_search_path;
    let mut rv: Array = arena_array(arena, path.size);
    let mut i: size_t = 0 as size_t;
    while i < path.size {
        let mut item: *mut SearchPathItem = path.items.offset(i as isize);
        let mut entry: Dict = arena_dict(arena, 5 as size_t);
        let c2rust_fresh8 = entry.size;
        entry.size = entry.size.wrapping_add(1);
        *entry.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"path\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*item).path),
                },
            },
        };
        if (*item).after {
            let c2rust_fresh9 = entry.size;
            entry.size = entry.size.wrapping_add(1);
            *entry.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                key: cstr_as_string(b"after\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        if (*item).pack_inserted {
            let c2rust_fresh10 = entry.size;
            entry.size = entry.size.wrapping_add(1);
            *entry.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                key: cstr_as_string(b"pack_inserted\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        if (*item).has_lua as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            let c2rust_fresh11 = entry.size;
            entry.size = entry.size.wrapping_add(1);
            *entry.items.offset(c2rust_fresh11 as isize) = key_value_pair {
                key: cstr_as_string(b"has_lua\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed {
                        boolean: (*item).has_lua as ::core::ffi::c_int
                            == kTrue as ::core::ffi::c_int,
                    },
                },
            };
        }
        let c2rust_fresh12 = entry.size;
        entry.size = entry.size.wrapping_add(1);
        *entry.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"pos_in_rtp\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*item).pos_in_rtp as Integer,
                },
            },
        };
        let c2rust_fresh13 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh13 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: entry },
        };
        i = i.wrapping_add(1);
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn runtime_get_named(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
    mut arena: *mut Arena,
) -> Array {
    let mut ref_0: ::core::ffi::c_int = 0;
    let mut path: RuntimeSearchPath = runtime_search_path_get_cached(&raw mut ref_0);
    static mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut rv: Array = runtime_get_named_common(
        lua,
        pat,
        all,
        path,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        arena,
    );
    runtime_search_path_unref(path, &raw mut ref_0);
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn runtime_get_named_thread(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
) -> Array {
    uv_mutex_lock(&raw mut runtime_search_path_mutex);
    static mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut rv: Array = runtime_get_named_common(
        lua,
        pat,
        all,
        runtime_search_path_thread,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ::core::ptr::null_mut::<Arena>(),
    );
    uv_mutex_unlock(&raw mut runtime_search_path_mutex);
    return rv;
}
unsafe extern "C" fn runtime_get_named_common(
    mut lua: bool,
    mut pat: Array,
    mut all: bool,
    mut path: RuntimeSearchPath,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
    mut arena: *mut Arena,
) -> Array {
    let mut rv: Array = arena_array(arena, path.size.wrapping_mul(pat.size));
    let mut i: size_t = 0 as size_t;
    '_done: while i < path.size {
        let mut item: *mut SearchPathItem = path.items.offset(i as isize);
        's_6: {
            if lua {
                if (*item).has_lua as ::core::ffi::c_int == kNone as ::core::ffi::c_int {
                    let mut size: size_t = snprintf(
                        buf,
                        buf_len,
                        b"%s/lua/\0".as_ptr() as *const ::core::ffi::c_char,
                        (*item).path,
                    ) as size_t;
                    (*item).has_lua = (size < buf_len && os_isdir(buf) as ::core::ffi::c_int != 0)
                        as ::core::ffi::c_int as TriState;
                }
                if (*item).has_lua as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
                    break 's_6;
                }
            }
            let mut j: size_t = 0 as size_t;
            loop {
                if j >= pat.size {
                    break 's_6;
                }
                let mut pat_item: Object = *pat.items.offset(j as isize);
                if pat_item.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut size_0: size_t = snprintf(
                        buf,
                        buf_len,
                        b"%s/%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*item).path,
                        pat_item.data.string.data,
                    ) as size_t;
                    if size_0 < buf_len {
                        if os_file_is_readable(buf) {
                            let c2rust_fresh14 = rv.size;
                            rv.size = rv.size.wrapping_add(1);
                            *rv.items.offset(c2rust_fresh14 as isize) = object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(arena, cstr_as_string(buf)),
                                },
                            };
                            if !all {
                                break '_done;
                            }
                        }
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn do_in_path_and_pp(
    mut path: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut done: ::core::ffi::c_int = FAIL;
    if flags & DIP_NORTP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        done |= do_in_path(
            path,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            if !name.is_null() && *name == 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                name
            },
            flags,
            callback,
            cookie,
        );
    }
    if (done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
        && flags & DIP_START as ::core::ffi::c_int != 0
    {
        let mut prefix: *const ::core::ffi::c_char = if flags & DIP_AFTER as ::core::ffi::c_int != 0
        {
            b"pack/*/start/*/after/\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"pack/*/start/*/\0".as_ptr() as *const ::core::ffi::c_char
        };
        done |= do_in_path(
            p_pp,
            prefix,
            name,
            flags & !(DIP_AFTER as ::core::ffi::c_int),
            callback,
            cookie,
        );
        if done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0 {
            prefix = if flags & DIP_AFTER as ::core::ffi::c_int != 0 {
                b"start/*/after/\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"start/*/\0".as_ptr() as *const ::core::ffi::c_char
            };
            done |= do_in_path(
                p_pp,
                prefix,
                name,
                flags & !(DIP_AFTER as ::core::ffi::c_int),
                callback,
                cookie,
            );
        }
    }
    if (done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
        && flags & DIP_OPT as ::core::ffi::c_int != 0
    {
        done |= do_in_path(
            p_pp,
            b"pack/*/opt/*/\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            flags,
            callback,
            cookie,
        );
        if done == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0 {
            done |= do_in_path(
                p_pp,
                b"opt/*/\0".as_ptr() as *const ::core::ffi::c_char,
                name,
                flags,
                callback,
                cookie,
            );
        }
    }
    return done;
}
unsafe extern "C" fn push_path(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut entry: *mut ::core::ffi::c_char,
    mut after: bool,
    mut pos_in_rtp: size_t,
) -> bool {
    let mut key_alloc: *mut String_0 = ::core::ptr::null_mut::<String_0>();
    if set_put_String(rtp_used, cstr_as_string(entry), &raw mut key_alloc) {
        *key_alloc = cstr_to_string(entry);
        if (*search_path).size == (*search_path).capacity {
            (*search_path).capacity = if (*search_path).capacity != 0 {
                (*search_path).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*search_path).items = xrealloc(
                (*search_path).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<SearchPathItem>().wrapping_mul((*search_path).capacity),
            ) as *mut SearchPathItem;
        } else {
        };
        let c2rust_fresh6 = (*search_path).size;
        (*search_path).size = (*search_path).size.wrapping_add(1);
        *(*search_path).items.offset(c2rust_fresh6 as isize) = SearchPathItem {
            path: (*key_alloc).data,
            after: after,
            pack_inserted: false,
            has_lua: kNone,
            pos_in_rtp: pos_in_rtp,
        };
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn expand_rtp_entry(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut entry: *mut ::core::ffi::c_char,
    mut after: bool,
    mut pos_in_rtp: size_t,
) {
    if set_has_String(rtp_used, cstr_as_string(entry)) {
        return;
    }
    if *entry == 0 {
        push_path(search_path, rtp_used, entry, after, pos_in_rtp);
    }
    let mut num_files: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut pat: [*mut ::core::ffi::c_char; 1] = [entry as *mut ::core::ffi::c_char];
    if gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut pat as *mut *mut ::core::ffi::c_char,
        &raw mut num_files,
        &raw mut files,
        EW_DIR as ::core::ffi::c_int | EW_NOBREAK as ::core::ffi::c_int,
    ) == OK
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_files {
            push_path(
                search_path,
                rtp_used,
                *files.offset(i as isize),
                after,
                pos_in_rtp,
            );
            i += 1;
        }
        FreeWild(num_files, files);
    }
}
unsafe extern "C" fn expand_pack_entry(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut after_path: *mut CharVec,
    mut pack_entry: *mut ::core::ffi::c_char,
    mut pack_entry_len: size_t,
    mut pos_in_rtp: size_t,
) {
    static mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut start_pat: [*mut ::core::ffi::c_char; 2] = [
        b"/pack/*/start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"/start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        if pack_entry_len
            .wrapping_add(strlen(start_pat[i as usize] as *const ::core::ffi::c_char))
            .wrapping_add(1 as size_t)
            <= ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
        {
            xstrlcpy(
                &raw mut buf as *mut ::core::ffi::c_char,
                pack_entry,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
            );
            xstrlcpy(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(pack_entry_len as isize),
                start_pat[i as usize] as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>().wrapping_sub(pack_entry_len),
            );
            expand_rtp_entry(
                search_path,
                rtp_used,
                &raw mut buf as *mut ::core::ffi::c_char,
                false_0 != 0,
                pos_in_rtp,
            );
            let mut after_size: size_t =
                strlen(&raw mut buf as *mut ::core::ffi::c_char).wrapping_add(7 as size_t);
            let mut after: *mut ::core::ffi::c_char =
                xmallocz(after_size) as *mut ::core::ffi::c_char;
            xstrlcpy(after, &raw mut buf as *mut ::core::ffi::c_char, after_size);
            xstrlcat(
                after,
                b"/after\0".as_ptr() as *const ::core::ffi::c_char,
                after_size,
            );
            if (*after_path).size == (*after_path).capacity {
                (*after_path).capacity = if (*after_path).capacity != 0 {
                    (*after_path).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*after_path).items = xrealloc(
                    (*after_path).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                        .wrapping_mul((*after_path).capacity),
                ) as *mut *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh7 = (*after_path).size;
            (*after_path).size = (*after_path).size.wrapping_add(1);
            let c2rust_lvalue_ptr = &raw mut *(*after_path).items.offset(c2rust_fresh7 as isize);
            *c2rust_lvalue_ptr = after;
        }
        i += 1;
    }
}
unsafe extern "C" fn path_is_after(mut buf: *mut ::core::ffi::c_char, mut buflen: size_t) -> bool {
    return buflen >= 5 as size_t
        && (!(buflen >= 6 as size_t)
            || vim_ispathsep(
                *buf.offset(buflen.wrapping_sub(6 as size_t) as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0)
        && strcmp(
            buf.offset(buflen as isize)
                .offset(-(5 as ::core::ffi::c_int as isize)),
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn runtime_search_path_build() -> RuntimeSearchPath {
    let mut pack_entries: C2Rust_Unnamed_23 = C2Rust_Unnamed_23 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<String_0>(),
    };
    let mut pack_used: Map_String_int = MAP_INIT;
    let mut rtp_used: Set_String = SET_INIT;
    let mut search_path: RuntimeSearchPath = RuntimeSearchPath {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<SearchPathItem>(),
    };
    let mut after_path: CharVec = CharVec {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    };
    static mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut entry: *mut ::core::ffi::c_char = p_pp;
    while *entry as ::core::ffi::c_int != NUL {
        let mut cur_entry: *mut ::core::ffi::c_char = entry;
        let mut buflen: size_t = copy_option_part(
            &raw mut entry,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut the_entry: String_0 = String_0 {
            data: cur_entry,
            size: buflen,
        };
        if pack_entries.size == pack_entries.capacity {
            pack_entries.capacity = if pack_entries.capacity != 0 {
                pack_entries.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            pack_entries.items = xrealloc(
                pack_entries.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<String_0>().wrapping_mul(pack_entries.capacity),
            ) as *mut String_0;
        } else {
        };
        let c2rust_fresh5 = pack_entries.size;
        pack_entries.size = pack_entries.size.wrapping_add(1);
        *pack_entries.items.offset(c2rust_fresh5 as isize) = the_entry;
        map_put_String_int(&raw mut pack_used, the_entry, 0 as ::core::ffi::c_int);
    }
    let mut rtp_entry: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    rtp_entry = p_rtp;
    while *rtp_entry as ::core::ffi::c_int != NUL {
        let mut cur_entry_0: *mut ::core::ffi::c_char = rtp_entry;
        let mut buflen_0: size_t = copy_option_part(
            &raw mut rtp_entry,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if path_is_after(&raw mut buf as *mut ::core::ffi::c_char, buflen_0) {
            rtp_entry = cur_entry_0;
            break;
        } else {
            let mut pos_in_rtp: size_t = cur_entry_0.offset_from(p_rtp) as size_t;
            expand_rtp_entry(
                &raw mut search_path,
                &raw mut rtp_used,
                &raw mut buf as *mut ::core::ffi::c_char,
                false_0 != 0,
                pos_in_rtp,
            );
            let mut h: *mut handle_T = map_ref_String_int(
                &raw mut pack_used,
                cstr_as_string(&raw mut buf as *mut ::core::ffi::c_char),
                ::core::ptr::null_mut::<*mut String_0>(),
            ) as *mut handle_T;
            if !h.is_null() {
                *h += 1;
                expand_pack_entry(
                    &raw mut search_path,
                    &raw mut rtp_used,
                    &raw mut after_path,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen_0,
                    pos_in_rtp,
                );
            }
        }
    }
    let mut sentinel_pos_in_rtp: size_t = rtp_entry.offset_from(p_rtp) as size_t;
    sentinel_pos_in_rtp = sentinel_pos_in_rtp.wrapping_sub(
        (if sentinel_pos_in_rtp > 0 as size_t {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as size_t,
    );
    let mut i: size_t = 0 as size_t;
    while i < pack_entries.size {
        let mut item: String_0 = *pack_entries.items.offset(i as isize);
        let mut h_0: handle_T = map_get_String_int(&raw mut pack_used, item);
        if h_0 == 0 as ::core::ffi::c_int {
            expand_pack_entry(
                &raw mut search_path,
                &raw mut rtp_used,
                &raw mut after_path,
                item.data,
                item.size,
                sentinel_pos_in_rtp,
            );
        }
        i = i.wrapping_add(1);
    }
    let mut i_0: size_t = 0 as size_t;
    while i_0 < after_path.size {
        expand_rtp_entry(
            &raw mut search_path,
            &raw mut rtp_used,
            *after_path.items.offset(i_0 as isize),
            true_0 != 0,
            sentinel_pos_in_rtp,
        );
        xfree(*after_path.items.offset(i_0 as isize) as *mut ::core::ffi::c_void);
        i_0 = i_0.wrapping_add(1);
    }
    while *rtp_entry as ::core::ffi::c_int != NUL {
        let mut cur_entry_1: *mut ::core::ffi::c_char = rtp_entry;
        let mut buflen_1: size_t = copy_option_part(
            &raw mut rtp_entry,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut pos_in_rtp_0: size_t = cur_entry_1.offset_from(p_rtp) as size_t;
        expand_rtp_entry(
            &raw mut search_path,
            &raw mut rtp_used,
            &raw mut buf as *mut ::core::ffi::c_char,
            path_is_after(&raw mut buf as *mut ::core::ffi::c_char, buflen_1),
            pos_in_rtp_0,
        );
    }
    xfree(pack_entries.items as *mut ::core::ffi::c_void);
    pack_entries.capacity = 0 as size_t;
    pack_entries.size = pack_entries.capacity;
    pack_entries.items = ::core::ptr::null_mut::<String_0>();
    xfree(after_path.items as *mut ::core::ffi::c_void);
    after_path.capacity = 0 as size_t;
    after_path.size = after_path.capacity;
    after_path.items = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    xfree(pack_used.set.keys as *mut ::core::ffi::c_void);
    xfree(pack_used.set.h.hash as *mut ::core::ffi::c_void);
    pack_used.set = SET_INIT;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut pack_used.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree(rtp_used.keys as *mut ::core::ffi::c_void);
    xfree(rtp_used.h.hash as *mut ::core::ffi::c_void);
    rtp_used = SET_INIT;
    return search_path;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_runtimepackpath(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    runtime_search_path_valid = false_0 != 0;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn runtime_search_path_free(mut path: RuntimeSearchPath) {
    let mut j: size_t = 0 as size_t;
    while j < path.size {
        let mut item: SearchPathItem = *path.items.offset(j as isize);
        xfree(item.path as *mut ::core::ffi::c_void);
        j = j.wrapping_add(1);
    }
    xfree(path.items as *mut ::core::ffi::c_void);
    path.capacity = 0 as size_t;
    path.size = path.capacity;
    path.items = ::core::ptr::null_mut::<SearchPathItem>();
}
#[no_mangle]
pub unsafe extern "C" fn runtime_search_path_validate() {
    if !nlua_is_deferred_safe() {
        return;
    }
    if !runtime_search_path_valid {
        if runtime_search_path_ref.is_null() {
            msg_ext_ui_flush();
            runtime_search_path_free(runtime_search_path);
        }
        runtime_search_path = runtime_search_path_build();
        runtime_search_path_valid = true_0 != 0;
        runtime_search_path_ref = ::core::ptr::null_mut::<::core::ffi::c_int>();
        update_runtime_search_path_thread(true_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn update_runtime_search_path_thread(mut force: bool) {
    if !force
        && !(runtime_search_path_valid as ::core::ffi::c_int != 0
            && !runtime_search_path_valid_thread)
    {
        return;
    }
    uv_mutex_lock(&raw mut runtime_search_path_mutex);
    runtime_search_path_free(runtime_search_path_thread);
    runtime_search_path_thread = copy_runtime_search_path(runtime_search_path);
    uv_mutex_unlock(&raw mut runtime_search_path_mutex);
    runtime_search_path_valid_thread = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_in_runtimepath(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut success: ::core::ffi::c_int = FAIL;
    if flags & DIP_NORTP as ::core::ffi::c_int == 0 {
        success |= do_in_cached_path(
            if !name.is_null() && *name == 0 {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                name
            },
            flags,
            callback,
            cookie,
        );
        flags = flags & !(DIP_START as ::core::ffi::c_int) | DIP_NORTP as ::core::ffi::c_int;
    }
    if flags & (DIP_START as ::core::ffi::c_int | DIP_OPT as ::core::ffi::c_int) != 0
        && (success == FAIL || flags & DIP_ALL as ::core::ffi::c_int != 0)
    {
        success |= do_in_path_and_pp(p_rtp, name, flags, callback, cookie);
    }
    return success;
}
#[no_mangle]
pub unsafe extern "C" fn source_runtime(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return do_in_runtimepath(
        name,
        flags,
        Some(
            source_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        NULL_0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn source_runtime_vim_lua(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return do_in_runtimepath(
        name,
        flags,
        Some(
            source_callback_vim_lua
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        NULL_0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn source_in_path_vim_lua(
    mut path: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return do_in_path_and_pp(
        path,
        name,
        flags,
        Some(
            source_callback_vim_lua
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        NULL_0,
    );
}
unsafe extern "C" fn gen_expand_wildcards_and_cb(
    mut num_pat: ::core::ffi::c_int,
    mut pats: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut all: bool,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut num_files: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if gen_expand_wildcards(num_pat, pats, &raw mut num_files, &raw mut files, flags) != OK {
        return FAIL;
    }
    Some(callback.expect("non-null function pointer")).expect("non-null function pointer")(
        num_files, files, all, cookie,
    );
    FreeWild(num_files, files);
    return OK;
}
unsafe extern "C" fn add_pack_dir_to_rtp(
    mut fname: *mut ::core::ffi::c_char,
    mut is_pack: bool,
) -> ::core::ffi::c_int {
    let mut afterlen: size_t = 0;
    let mut oldlen: size_t = 0;
    let mut addlen: size_t = 0;
    let mut new_rtp_capacity: size_t = 0;
    let mut new_rtp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keep: size_t = 0;
    let mut first_pos: size_t = 0;
    let mut new_rtp_len: size_t = 0;
    let mut after_pos: size_t = 0;
    let mut was_valid: bool = false;
    let mut afterdir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut p1: *mut ::core::ffi::c_char = get_past_head(fname);
    let mut p2: *mut ::core::ffi::c_char = p1;
    let mut p3: *mut ::core::ffi::c_char = p1;
    let mut p4: *mut ::core::ffi::c_char = p1;
    let mut p: *mut ::core::ffi::c_char = p1;
    while *p != 0 {
        if vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
            p4 = p3;
            p3 = p2;
            p2 = p1;
            p1 = p;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    p4 = p4.offset(1);
    let mut c: ::core::ffi::c_char = *p4;
    *p4 = NUL as ::core::ffi::c_char;
    let ffname: *mut ::core::ffi::c_char = fix_fname(fname);
    *p4 = c;
    if ffname.is_null() {
        return FAIL;
    }
    let mut fname_len: size_t = strlen(ffname);
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut insp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut after_insp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut entry: *const ::core::ffi::c_char = p_rtp;
    '_theend: {
        while *entry as ::core::ffi::c_int != NUL {
            let mut cur_entry: *const ::core::ffi::c_char = entry;
            copy_option_part(
                &raw mut entry as *mut *mut ::core::ffi::c_char,
                &raw mut buf as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut p_0: *mut ::core::ffi::c_char = strstr(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"after\0".as_ptr() as *const ::core::ffi::c_char,
            );
            let mut is_after: bool = !p_0.is_null()
                && p_0 > &raw mut buf as *mut ::core::ffi::c_char
                && vim_ispathsep(
                    *p_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && (vim_ispathsep(
                    *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                    || *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *p_0.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int);
            if is_after {
                if insp.is_null() {
                    insp = cur_entry;
                }
                after_insp = cur_entry;
                break;
            } else {
                if !insp.is_null() {
                    continue;
                }
                add_pathsep(&raw mut buf as *mut ::core::ffi::c_char);
                let rtp_ffname: *mut ::core::ffi::c_char =
                    fix_fname(&raw mut buf as *mut ::core::ffi::c_char);
                if rtp_ffname.is_null() {
                    break '_theend;
                }
                if path_fnamencmp(rtp_ffname, ffname, fname_len) == 0 as ::core::ffi::c_int {
                    insp = entry;
                }
                xfree(rtp_ffname as *mut ::core::ffi::c_void);
            }
        }
        if insp.is_null() {
            insp = p_rtp.offset(strlen(p_rtp) as isize);
        }
        afterdir = concat_fnames(
            fname,
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        afterlen = 0 as size_t;
        if if is_pack as ::core::ffi::c_int != 0 {
            pack_has_entries(afterdir) as ::core::ffi::c_int
        } else {
            os_isdir(afterdir) as ::core::ffi::c_int
        } != 0
        {
            afterlen = strlen(afterdir).wrapping_add(1 as size_t);
        }
        oldlen = strlen(p_rtp);
        addlen = strlen(fname).wrapping_add(1 as size_t);
        new_rtp_capacity = oldlen
            .wrapping_add(addlen)
            .wrapping_add(afterlen)
            .wrapping_add(1 as size_t);
        new_rtp = try_malloc(new_rtp_capacity) as *mut ::core::ffi::c_char;
        if !new_rtp.is_null() {
            keep = insp.offset_from(p_rtp) as size_t;
            first_pos = keep;
            memmove(
                new_rtp as *mut ::core::ffi::c_void,
                p_rtp as *const ::core::ffi::c_void,
                keep,
            );
            new_rtp_len = keep;
            if *insp as ::core::ffi::c_int == NUL {
                let c2rust_fresh15 = new_rtp_len;
                new_rtp_len = new_rtp_len.wrapping_add(1);
                *new_rtp.offset(c2rust_fresh15 as isize) = ',' as ::core::ffi::c_char;
                first_pos = first_pos.wrapping_add(1);
            }
            memmove(
                new_rtp.offset(new_rtp_len as isize) as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                addlen.wrapping_sub(1 as size_t),
            );
            new_rtp_len = new_rtp_len.wrapping_add(addlen.wrapping_sub(1 as size_t));
            if *insp as ::core::ffi::c_int != NUL {
                let c2rust_fresh16 = new_rtp_len;
                new_rtp_len = new_rtp_len.wrapping_add(1);
                *new_rtp.offset(c2rust_fresh16 as isize) = ',' as ::core::ffi::c_char;
            }
            after_pos = 0 as size_t;
            if afterlen > 0 as size_t && !after_insp.is_null() {
                let mut keep_after: size_t = after_insp.offset_from(p_rtp) as size_t;
                memmove(
                    new_rtp.offset(new_rtp_len as isize) as *mut ::core::ffi::c_void,
                    p_rtp.offset(keep as isize) as *const ::core::ffi::c_void,
                    keep_after.wrapping_sub(keep),
                );
                new_rtp_len = new_rtp_len.wrapping_add(keep_after.wrapping_sub(keep));
                memmove(
                    new_rtp.offset(new_rtp_len as isize) as *mut ::core::ffi::c_void,
                    afterdir as *const ::core::ffi::c_void,
                    afterlen.wrapping_sub(1 as size_t),
                );
                new_rtp_len = new_rtp_len.wrapping_add(afterlen.wrapping_sub(1 as size_t));
                let c2rust_fresh17 = new_rtp_len;
                new_rtp_len = new_rtp_len.wrapping_add(1);
                *new_rtp.offset(c2rust_fresh17 as isize) = ',' as ::core::ffi::c_char;
                keep = keep_after;
                after_pos = keep_after;
            }
            if *p_rtp.offset(keep as isize) as ::core::ffi::c_int != NUL {
                memmove(
                    new_rtp.offset(new_rtp_len as isize) as *mut ::core::ffi::c_void,
                    p_rtp.offset(keep as isize) as *const ::core::ffi::c_void,
                    oldlen.wrapping_sub(keep).wrapping_add(1 as size_t),
                );
            } else {
                *new_rtp.offset(new_rtp_len as isize) = NUL as ::core::ffi::c_char;
            }
            if afterlen > 0 as size_t && after_insp.is_null() {
                after_pos = xstrlcat(
                    new_rtp,
                    b",\0".as_ptr() as *const ::core::ffi::c_char,
                    new_rtp_capacity,
                );
                xstrlcat(new_rtp, afterdir, new_rtp_capacity);
            }
            was_valid = runtime_search_path_valid;
            set_option_value_give_err(
                kOptRuntimepath,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(new_rtp),
                    },
                },
                0 as ::core::ffi::c_int,
            );
            '_c2rust_label: {
                if !runtime_search_path_valid {
                } else {
                    __assert_fail(
                        b"!runtime_search_path_valid\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1174 as ::core::ffi::c_uint,
                        b"int add_pack_dir_to_rtp(char *, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if was_valid as ::core::ffi::c_int != 0 && !is_pack && runtime_search_path_ref.is_null()
            {
                runtime_search_path_valid = true_0 != 0;
                runtime_search_path_valid_thread = false_0 != 0;
                if runtime_search_path.size == runtime_search_path.capacity {
                    runtime_search_path.capacity = if runtime_search_path.capacity != 0 {
                        runtime_search_path.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    runtime_search_path.items = xrealloc(
                        runtime_search_path.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<SearchPathItem>()
                            .wrapping_mul(runtime_search_path.capacity),
                    ) as *mut SearchPathItem;
                } else {
                };
                runtime_search_path.size = runtime_search_path.size.wrapping_add(1);
                let mut i: ssize_t = runtime_search_path.size as ssize_t - 1 as ssize_t;
                if afterlen > 0 as size_t {
                    if runtime_search_path.size == runtime_search_path.capacity {
                        runtime_search_path.capacity = if runtime_search_path.capacity != 0 {
                            runtime_search_path.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        runtime_search_path.items = xrealloc(
                            runtime_search_path.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<SearchPathItem>()
                                .wrapping_mul(runtime_search_path.capacity),
                        )
                            as *mut SearchPathItem;
                    } else {
                    };
                    runtime_search_path.size = runtime_search_path.size.wrapping_add(1);
                    i += 1 as ssize_t;
                    while i >= 1 as ssize_t {
                        if i > 1 as ssize_t
                            && (*runtime_search_path
                                .items
                                .offset((i - 2 as ssize_t) as isize))
                            .pos_in_rtp
                                >= after_pos
                        {
                            *runtime_search_path.items.offset(i as isize) = *runtime_search_path
                                .items
                                .offset((i - 2 as ssize_t) as isize);
                            (*runtime_search_path.items.offset(i as isize)).pos_in_rtp =
                                (*runtime_search_path.items.offset(i as isize))
                                    .pos_in_rtp
                                    .wrapping_add(addlen.wrapping_add(afterlen));
                            i -= 1;
                        } else {
                            *runtime_search_path.items.offset(i as isize) = SearchPathItem {
                                path: xstrdup(afterdir),
                                after: true_0 != 0,
                                pack_inserted: true_0 != 0,
                                has_lua: kNone,
                                pos_in_rtp: after_pos.wrapping_add(addlen),
                            };
                            i -= 1;
                            break;
                        }
                    }
                }
                while i >= 0 as ssize_t {
                    if i > 0 as ssize_t
                        && (*runtime_search_path
                            .items
                            .offset((i - 1 as ssize_t) as isize))
                        .pos_in_rtp
                            >= first_pos
                    {
                        *runtime_search_path.items.offset(i as isize) = *runtime_search_path
                            .items
                            .offset((i - 1 as ssize_t) as isize);
                        (*runtime_search_path.items.offset(i as isize)).pos_in_rtp =
                            (*runtime_search_path.items.offset(i as isize))
                                .pos_in_rtp
                                .wrapping_add(addlen);
                        i -= 1;
                    } else {
                        *runtime_search_path.items.offset(i as isize) = SearchPathItem {
                            path: xstrdup(fname),
                            after: false_0 != 0,
                            pack_inserted: true_0 != 0,
                            has_lua: kNone,
                            pos_in_rtp: first_pos,
                        };
                        break;
                    }
                }
            }
            xfree(new_rtp as *mut ::core::ffi::c_void);
            retval = OK;
        }
    }
    xfree(ffname as *mut ::core::ffi::c_void);
    xfree(afterdir as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn load_pack_plugin(
    mut opt: bool,
    mut fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    static mut plugpat: [::core::ffi::c_char; 15] = unsafe {
        ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"%s/plugin/**/*\0")
    };
    static mut ftpat: [::core::ffi::c_char; 14] = unsafe {
        ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"%s/ftdetect/*\0")
    };
    let ffname: *mut ::core::ffi::c_char = fix_fname(fname);
    let mut len: size_t =
        strlen(ffname).wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 15]>());
    let mut pat: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
    vim_snprintf(
        pat,
        len,
        &raw const plugpat as *const ::core::ffi::c_char,
        ffname,
    );
    gen_expand_wildcards_and_cb(
        1 as ::core::ffi::c_int,
        &raw mut pat,
        EW_FILE as ::core::ffi::c_int,
        true_0 != 0,
        Some(
            source_callback_vim_lua
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        NULL_0,
    );
    let mut cmd: *mut ::core::ffi::c_char =
        xstrdup(b"g:did_load_filetypes\0".as_ptr() as *const ::core::ffi::c_char);
    if opt as ::core::ffi::c_int != 0 && eval_to_number(cmd, false_0 != 0) > 0 as varnumber_T {
        do_cmdline_cmd(b"augroup filetypedetect\0".as_ptr() as *const ::core::ffi::c_char);
        vim_snprintf(
            pat,
            len,
            &raw const ftpat as *const ::core::ffi::c_char,
            ffname,
        );
        gen_expand_wildcards_and_cb(
            1 as ::core::ffi::c_int,
            &raw mut pat,
            EW_FILE as ::core::ffi::c_int,
            true_0 != 0,
            Some(
                source_callback_vim_lua
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            NULL_0,
        );
        do_cmdline_cmd(b"augroup END\0".as_ptr() as *const ::core::ffi::c_char);
    }
    xfree(cmd as *mut ::core::ffi::c_void);
    xfree(pat as *mut ::core::ffi::c_void);
    xfree(ffname as *mut ::core::ffi::c_void);
    return OK;
}
static mut APP_ADD_DIR: ::core::ffi::c_int = 0;
static mut APP_LOAD: ::core::ffi::c_int = 0;
static mut APP_BOTH: ::core::ffi::c_int = 0;
unsafe extern "C" fn add_pack_plugins(
    mut opt: bool,
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) {
    let mut did_one: bool = false_0 != 0;
    if cookie != &raw mut APP_LOAD as *mut ::core::ffi::c_void {
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_fnames {
            let mut found: bool = false_0 != 0;
            let mut p: *const ::core::ffi::c_char = p_rtp;
            while *p as ::core::ffi::c_int != NUL {
                copy_option_part(
                    &raw mut p as *mut *mut ::core::ffi::c_char,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if path_fnamecmp(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    *fnames.offset(i as isize),
                ) != 0 as ::core::ffi::c_int
                {
                    continue;
                }
                found = true_0 != 0;
                break;
            }
            if !found {
                if add_pack_dir_to_rtp(*fnames.offset(i as isize), false_0 != 0) == FAIL {
                    return;
                }
            }
            did_one = true_0 != 0;
            if !all {
                break;
            }
            i += 1;
        }
    }
    if !all && did_one as ::core::ffi::c_int != 0 {
        return;
    }
    if cookie != &raw mut APP_ADD_DIR as *mut ::core::ffi::c_void {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < num_fnames {
            load_pack_plugin(opt, *fnames.offset(i_0 as isize));
            if !all {
                break;
            }
            i_0 += 1;
        }
    }
}
unsafe extern "C" fn add_start_pack_plugins(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    add_pack_plugins(false_0 != 0, num_fnames, fnames, all, cookie);
    return num_fnames > 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn add_opt_pack_plugins(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    add_pack_plugins(true_0 != 0, num_fnames, fnames, all, cookie);
    return num_fnames > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn add_pack_start_dirs() {
    do_in_path(
        p_pp,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
        Some(
            add_pack_start_dir
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        NULL_0,
    );
}
unsafe extern "C" fn pack_has_entries(mut buf: *mut ::core::ffi::c_char) -> bool {
    let mut num_files: ::core::ffi::c_int = 0;
    let mut files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut pat: [*mut ::core::ffi::c_char; 1] = [buf as *mut ::core::ffi::c_char];
    if gen_expand_wildcards(
        1 as ::core::ffi::c_int,
        &raw mut pat as *mut *mut ::core::ffi::c_char,
        &raw mut num_files,
        &raw mut files,
        EW_DIR as ::core::ffi::c_int,
    ) == OK
    {
        FreeWild(num_files, files);
    }
    return num_files > 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn add_pack_start_dir(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut _cookie: *mut ::core::ffi::c_void,
) -> bool {
    static mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        let mut start_pat: [*mut ::core::ffi::c_char; 2] = [
            b"/start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"/pack/*/start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 2 as ::core::ffi::c_int {
            if strlen(*fnames.offset(i as isize))
                .wrapping_add(strlen(start_pat[j as usize] as *const ::core::ffi::c_char))
                .wrapping_add(1 as size_t)
                <= MAXPATHL as size_t
            {
                xstrlcpy(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    *fnames.offset(i as isize),
                    MAXPATHL as size_t,
                );
                xstrlcat(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    start_pat[j as usize] as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                if pack_has_entries(&raw mut buf as *mut ::core::ffi::c_char) {
                    add_pack_dir_to_rtp(&raw mut buf as *mut ::core::ffi::c_char, true_0 != 0);
                }
            }
            j += 1;
        }
        if !all {
            break;
        }
        i += 1;
    }
    return num_fnames > 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn load_start_packages() {
    did_source_packages = true_0 != 0;
    do_in_path(
        p_pp,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        b"pack/*/start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
        Some(
            add_start_pack_plugins
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        &raw mut APP_LOAD as *mut ::core::ffi::c_void,
    );
    do_in_path(
        p_pp,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        b"start/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
        Some(
            add_start_pack_plugins
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        &raw mut APP_LOAD as *mut ::core::ffi::c_void,
    );
    update_runtime_search_path_thread(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_packloadall(mut eap: *mut exarg_T) {
    if !did_source_packages || (*eap).forceit != 0 {
        add_pack_start_dirs();
        load_start_packages();
    }
}
#[no_mangle]
pub unsafe extern "C" fn load_plugins() {
    if p_lpl != 0 {
        let mut rtp_copy: *mut ::core::ffi::c_char = p_rtp;
        let plugin_pattern: *mut ::core::ffi::c_char =
            b"plugin/**/*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        if !did_source_packages {
            rtp_copy = xstrdup(p_rtp);
            add_pack_start_dirs();
        }
        source_in_path_vim_lua(
            rtp_copy,
            plugin_pattern,
            DIP_ALL as ::core::ffi::c_int | DIP_NOAFTER as ::core::ffi::c_int,
        );
        if !time_fd.is_null() {
            time_msg(
                b"loading rtp plugins\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        if !did_source_packages {
            xfree(rtp_copy as *mut ::core::ffi::c_void);
            load_start_packages();
        }
        if !time_fd.is_null() {
            time_msg(
                b"loading packages\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
        source_runtime_vim_lua(
            plugin_pattern,
            DIP_ALL as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int,
        );
        if !time_fd.is_null() {
            time_msg(
                b"loading after plugins\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<proftime_T>(),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_packadd(mut eap: *mut exarg_T) {
    static mut plugpat: [::core::ffi::c_char; 13] = unsafe {
        ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"pack/*/%s/%s\0")
    };
    let mut res: ::core::ffi::c_int = OK;
    let len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 13]>()
        .wrapping_add(strlen((*eap).arg))
        .wrapping_add(5 as size_t);
    let mut pat: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
    let mut cookie: *mut ::core::ffi::c_void = (if (*eap).forceit != 0 {
        &raw mut APP_ADD_DIR
    } else {
        &raw mut APP_BOTH
    }) as *mut ::core::ffi::c_void;
    if !did_source_packages {
        vim_snprintf(
            pat,
            len,
            &raw const plugpat as *const ::core::ffi::c_char,
            b"start\0".as_ptr() as *const ::core::ffi::c_char,
            (*eap).arg,
        );
        res = do_in_path(
            p_pp,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            pat,
            DIP_ALL as ::core::ffi::c_int + DIP_DIR as ::core::ffi::c_int,
            Some(
                add_start_pack_plugins
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            cookie,
        );
    }
    vim_snprintf(
        pat,
        len,
        &raw const plugpat as *const ::core::ffi::c_char,
        b"opt\0".as_ptr() as *const ::core::ffi::c_char,
        (*eap).arg,
    );
    do_in_path(
        p_pp,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        pat,
        DIP_ALL as ::core::ffi::c_int
            + DIP_DIR as ::core::ffi::c_int
            + (if res == FAIL {
                DIP_ERR as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
        Some(
            add_opt_pack_plugins
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut *mut ::core::ffi::c_char,
                    bool,
                    *mut ::core::ffi::c_void,
                ) -> bool,
        ),
        cookie,
    );
    update_runtime_search_path_thread(false_0 != 0);
    xfree(pat as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ExpandRTDir_int(
    mut pat: *mut ::core::ffi::c_char,
    mut pat_len: size_t,
    mut flags: ::core::ffi::c_int,
    mut keep_ext: bool,
    mut gap: *mut garray_T,
    mut dirnames: *mut *mut ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*dirnames.offset(i as isize)).is_null() {
        let buf_len: size_t = strlen(*dirnames.offset(i as isize))
            .wrapping_add(pat_len)
            .wrapping_add(64 as size_t);
        let mut buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
        let mut glob_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut expand_dirs: bool = false_0 != 0;
        snprintf(
            buf,
            buf_len,
            b"%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                *dirnames.offset(i as isize) as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                b"/\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            pat,
            b"*.{vim,lua}\0".as_ptr() as *const ::core::ffi::c_char,
        );
        loop {
            if flags & DIP_NORTP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                globpath(p_rtp, buf, gap, glob_flags, expand_dirs);
            }
            if flags & DIP_START as ::core::ffi::c_int != 0 {
                snprintf(
                    buf,
                    buf_len,
                    b"pack/*/start/*/%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        b"/\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    pat,
                    if expand_dirs as ::core::ffi::c_int != 0 {
                        b"*\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"*.{vim,lua}\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
                snprintf(
                    buf,
                    buf_len,
                    b"start/*/%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        b"/\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    pat,
                    if expand_dirs as ::core::ffi::c_int != 0 {
                        b"*\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"*.{vim,lua}\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
            }
            if flags & DIP_OPT as ::core::ffi::c_int != 0 {
                snprintf(
                    buf,
                    buf_len,
                    b"pack/*/opt/*/%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        b"/\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    pat,
                    if expand_dirs as ::core::ffi::c_int != 0 {
                        b"*\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"*.{vim,lua}\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
                snprintf(
                    buf,
                    buf_len,
                    b"opt/*/%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                        b"/\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    pat,
                    if expand_dirs as ::core::ffi::c_int != 0 {
                        b"*\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"*.{vim,lua}\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                globpath(p_pp, buf, gap, glob_flags, expand_dirs);
            }
            if !(**dirnames.offset(i as isize) as ::core::ffi::c_int == NUL && !expand_dirs) {
                break;
            }
            snprintf(
                buf,
                buf_len,
                b"%s*\0".as_ptr() as *const ::core::ffi::c_char,
                pat,
            );
            glob_flags = WILD_ADD_SLASH as ::core::ffi::c_int;
            expand_dirs = true_0 != 0;
        }
        xfree(buf as *mut ::core::ffi::c_void);
        i += 1;
    }
    let mut pat_pathsep_cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < pat_len {
        if vim_ispathsep(*pat.offset(i_0 as isize) as ::core::ffi::c_int) {
            pat_pathsep_cnt += 1;
        }
        i_0 = i_0.wrapping_add(1);
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < (*gap).ga_len {
        let mut match_0: *mut ::core::ffi::c_char =
            *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i_1 as isize);
        let mut s: *mut ::core::ffi::c_char = match_0;
        let mut e: *mut ::core::ffi::c_char = s.offset(strlen(s) as isize);
        if e.offset_from(s) > 4 as isize
            && !keep_ext
            && (strncasecmp(
                e.offset(-(4 as ::core::ffi::c_int as isize)),
                b".vim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                4 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncasecmp(
                    e.offset(-(4 as ::core::ffi::c_int as isize)),
                    b".lua\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    4 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int)
        {
            e = e.offset(-(4 as ::core::ffi::c_int as isize));
            *e = NUL as ::core::ffi::c_char;
        }
        let mut match_pathsep_cnt: ::core::ffi::c_int = if e > s
            && *e.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int
        {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        s = e;
        while s > match_0 {
            if vim_ispathsep(*s as ::core::ffi::c_int) as ::core::ffi::c_int != 0 && {
                match_pathsep_cnt += 1;
                match_pathsep_cnt > pat_pathsep_cnt
            } {
                break;
            }
            s = s.offset(
                -((utf_head_off(match_0, s.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
        s = s.offset(1);
        if s != match_0 {
            '_c2rust_label: {
                if e.offset_from(s) + 1 as isize >= 0 as isize {
                } else {
                    __assert_fail(
                        b"(e - s) + 1 >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        1515 as ::core::ffi::c_uint,
                        b"void ExpandRTDir_int(char *, size_t, int, _Bool, garray_T *, char **)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            memmove(
                match_0 as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                (e.offset_from(s) as size_t).wrapping_add(1 as size_t),
            );
        }
        i_1 += 1;
    }
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        return;
    }
    ga_remove_duplicate_strings(gap);
}
#[no_mangle]
pub unsafe extern "C" fn ExpandRTDir(
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut dirnames: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    *num_file = 0 as ::core::ffi::c_int;
    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
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
    ExpandRTDir_int(pat, strlen(pat), flags, false_0 != 0, &raw mut ga, dirnames);
    if ga.ga_len <= 0 as ::core::ffi::c_int {
        return FAIL;
    }
    *file = ga.ga_data as *mut *mut ::core::ffi::c_char;
    *num_file = ga.ga_len;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn expand_runtime_cmd(
    mut pat: *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    *numMatches = 0 as ::core::ffi::c_int;
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
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
    let pat_len: size_t = strlen(pat);
    let mut dirnames: [*mut ::core::ffi::c_char; 2] = [
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ];
    ExpandRTDir_int(
        pat,
        pat_len,
        runtime_expand_flags,
        true_0 != 0,
        &raw mut ga,
        &raw mut dirnames as *mut *mut ::core::ffi::c_char,
    );
    if runtime_expand_flags == 0 as ::core::ffi::c_int {
        let mut where_values: [*mut ::core::ffi::c_char; 4] = [
            b"START\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"OPT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"PACK\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"ALL\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ];
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            if strncmp(pat, where_values[i as usize], pat_len) == 0 as ::core::ffi::c_int {
                ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                    xstrdup(where_values[i as usize]);
                ga.ga_len += 1;
            }
            i = i.wrapping_add(1);
        }
    }
    if ga.ga_len <= 0 as ::core::ffi::c_int {
        return FAIL;
    }
    *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
    *numMatches = ga.ga_len;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ExpandPackAddDir(
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    *num_file = 0 as ::core::ffi::c_int;
    *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut pat_len: size_t = strlen(pat);
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut buflen: size_t = pat_len.wrapping_add(26 as size_t);
    let mut s: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
    snprintf(
        s,
        buflen,
        b"pack/*/opt/%s*\0".as_ptr() as *const ::core::ffi::c_char,
        pat,
    );
    globpath(p_pp, s, &raw mut ga, 0 as ::core::ffi::c_int, true_0 != 0);
    snprintf(
        s,
        buflen,
        b"opt/%s*\0".as_ptr() as *const ::core::ffi::c_char,
        pat,
    );
    globpath(p_pp, s, &raw mut ga, 0 as ::core::ffi::c_int, true_0 != 0);
    xfree(s as *mut ::core::ffi::c_void);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ga.ga_len {
        let mut match_0: *mut ::core::ffi::c_char =
            *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        s = path_tail(match_0);
        memmove(
            match_0 as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            strlen(s).wrapping_add(1 as size_t),
        );
        i += 1;
    }
    if ga.ga_len <= 0 as ::core::ffi::c_int {
        return FAIL;
    }
    ga_remove_duplicate_strings(&raw mut ga);
    *file = ga.ga_data as *mut *mut ::core::ffi::c_char;
    *num_file = ga.ga_len;
    return OK;
}
unsafe extern "C" fn strcpy_comma_escaped(
    mut dest: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut shift: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < len {
        if *src.offset(i as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            let c2rust_fresh22 = shift;
            shift = shift.wrapping_add(1);
            *dest.offset(i.wrapping_add(c2rust_fresh22) as isize) = '\\' as ::core::ffi::c_char;
        }
        *dest.offset(i.wrapping_add(shift) as isize) = *src.offset(i as isize);
        i = i.wrapping_add(1);
    }
    return dest.offset(len.wrapping_add(shift) as isize);
}
#[inline]
unsafe extern "C" fn compute_double_env_sep_len(
    val: *const ::core::ffi::c_char,
    common_suf_len: size_t,
    single_suf_len: size_t,
) -> size_t {
    if val.is_null() || *val as ::core::ffi::c_int == NUL {
        return 0 as size_t;
    }
    let mut ret: size_t = 0 as size_t;
    let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    loop {
        let mut dir_len: size_t = 0;
        let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        iter = vim_env_iter(
            ENV_SEPCHAR as ::core::ffi::c_char,
            val,
            iter,
            &raw mut dir,
            &raw mut dir_len,
        );
        if !dir.is_null() && dir_len > 0 as size_t {
            ret = ret.wrapping_add(
                dir_len
                    .wrapping_add(memcnt(
                        dir as *const ::core::ffi::c_void,
                        ',' as ::core::ffi::c_char,
                        dir_len,
                    ))
                    .wrapping_add(common_suf_len)
                    .wrapping_add(
                        (after_pathsep(dir, dir.offset(dir_len as isize)) == 0)
                            as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_mul(2 as size_t)
                    .wrapping_add(single_suf_len),
            );
        }
        if iter.is_null() {
            break;
        }
    }
    return ret;
}
#[inline]
unsafe extern "C" fn add_env_sep_dirs(
    mut dest: *mut ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    suf1: *const ::core::ffi::c_char,
    len1: size_t,
    suf2: *const ::core::ffi::c_char,
    len2: size_t,
    forward: bool,
) -> *mut ::core::ffi::c_char {
    if val.is_null() || *val as ::core::ffi::c_int == NUL {
        return dest;
    }
    let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
    let appname_len: size_t = strlen(appname);
    loop {
        let mut dir_len: size_t = 0;
        let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        iter = if forward as ::core::ffi::c_int != 0 {
            Some(
                vim_env_iter
                    as unsafe extern "C" fn(
                        ::core::ffi::c_char,
                        *const ::core::ffi::c_char,
                        *const ::core::ffi::c_void,
                        *mut *const ::core::ffi::c_char,
                        *mut size_t,
                    ) -> *const ::core::ffi::c_void,
            )
        } else {
            Some(
                vim_env_iter_rev
                    as unsafe extern "C" fn(
                        ::core::ffi::c_char,
                        *const ::core::ffi::c_char,
                        *const ::core::ffi::c_void,
                        *mut *const ::core::ffi::c_char,
                        *mut size_t,
                    ) -> *const ::core::ffi::c_void,
            )
        }
        .expect("non-null function pointer")(
            ENV_SEPCHAR as ::core::ffi::c_char,
            val,
            iter,
            &raw mut dir,
            &raw mut dir_len,
        );
        if !dir.is_null() && dir_len > 0 as size_t {
            dest = strcpy_comma_escaped(dest, dir, dir_len);
            if after_pathsep(dest.offset(-(1 as ::core::ffi::c_int as isize)), dest) == 0 {
                let c2rust_fresh23 = dest;
                dest = dest.offset(1);
                *c2rust_fresh23 = PATHSEP as ::core::ffi::c_char;
            }
            memmove(
                dest as *mut ::core::ffi::c_void,
                appname as *const ::core::ffi::c_void,
                appname_len,
            );
            dest = dest.offset(appname_len as isize);
            if !suf1.is_null() {
                let c2rust_fresh24 = dest;
                dest = dest.offset(1);
                *c2rust_fresh24 = PATHSEP as ::core::ffi::c_char;
                memmove(
                    dest as *mut ::core::ffi::c_void,
                    suf1 as *const ::core::ffi::c_void,
                    len1,
                );
                dest = dest.offset(len1 as isize);
                if !suf2.is_null() {
                    let c2rust_fresh25 = dest;
                    dest = dest.offset(1);
                    *c2rust_fresh25 = PATHSEP as ::core::ffi::c_char;
                    memmove(
                        dest as *mut ::core::ffi::c_void,
                        suf2 as *const ::core::ffi::c_void,
                        len2,
                    );
                    dest = dest.offset(len2 as isize);
                }
            }
            let c2rust_fresh26 = dest;
            dest = dest.offset(1);
            *c2rust_fresh26 = ',' as ::core::ffi::c_char;
        }
        if iter.is_null() {
            break;
        }
    }
    return dest;
}
#[inline]
unsafe extern "C" fn add_dir(
    mut dest: *mut ::core::ffi::c_char,
    dir: *const ::core::ffi::c_char,
    dir_len: size_t,
    type_0: XDGVarType,
    suf1: *const ::core::ffi::c_char,
    len1: size_t,
    suf2: *const ::core::ffi::c_char,
    len2: size_t,
) -> *mut ::core::ffi::c_char {
    if dir.is_null() || dir_len == 0 as size_t {
        return dest;
    }
    dest = strcpy_comma_escaped(dest, dir, dir_len);
    let mut append_nvim: bool = type_0 as ::core::ffi::c_int == kXDGDataHome as ::core::ffi::c_int
        || type_0 as ::core::ffi::c_int == kXDGConfigHome as ::core::ffi::c_int;
    if append_nvim {
        if after_pathsep(dest.offset(-(1 as ::core::ffi::c_int as isize)), dest) == 0 {
            let c2rust_fresh18 = dest;
            dest = dest.offset(1);
            *c2rust_fresh18 = PATHSEP as ::core::ffi::c_char;
        }
        let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
        let mut appname_len: size_t = strlen(appname);
        '_c2rust_label: {
            if appname_len
                < ((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
                    .wrapping_sub(::core::mem::size_of::<[::core::ffi::c_char; 6]>())
            {
            } else {
                __assert_fail(
                    b"appname_len < (IOSIZE - sizeof(\"-data\"))\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1773 as ::core::ffi::c_uint,
                    b"char *add_dir(char *, const char *const, const size_t, const XDGVarType, const char *const, const size_t, const char *const, const size_t)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        xmemcpyz(
            &raw mut IObuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            appname as *const ::core::ffi::c_void,
            appname_len,
        );
        xmemcpyz(
            dest as *mut ::core::ffi::c_void,
            &raw mut IObuff as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            appname_len,
        );
        dest = dest.offset(appname_len as isize);
        if !suf1.is_null() {
            let c2rust_fresh19 = dest;
            dest = dest.offset(1);
            *c2rust_fresh19 = PATHSEP as ::core::ffi::c_char;
            memmove(
                dest as *mut ::core::ffi::c_void,
                suf1 as *const ::core::ffi::c_void,
                len1,
            );
            dest = dest.offset(len1 as isize);
            if !suf2.is_null() {
                let c2rust_fresh20 = dest;
                dest = dest.offset(1);
                *c2rust_fresh20 = PATHSEP as ::core::ffi::c_char;
                memmove(
                    dest as *mut ::core::ffi::c_void,
                    suf2 as *const ::core::ffi::c_void,
                    len2,
                );
                dest = dest.offset(len2 as isize);
            }
        }
    }
    let c2rust_fresh21 = dest;
    dest = dest.offset(1);
    *c2rust_fresh21 = ',' as ::core::ffi::c_char;
    return dest;
}
#[no_mangle]
pub unsafe extern "C" fn get_lib_dir() -> *mut ::core::ffi::c_char {
    if strlen(default_lib_dir) != 0 as size_t
        && os_isdir(default_lib_dir) as ::core::ffi::c_int != 0
    {
        return xstrdup(default_lib_dir);
    }
    let mut exe_name: [::core::ffi::c_char; 4096] = [0; 4096];
    vim_get_prefix_from_exepath(&raw mut exe_name as *mut ::core::ffi::c_char);
    if append_path(
        &raw mut exe_name as *mut ::core::ffi::c_char,
        b"lib/nvim\0".as_ptr() as *const ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == OK
    {
        return xstrdup(&raw mut exe_name as *mut ::core::ffi::c_char);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn runtimepath_default(mut clean_arg: bool) -> *mut ::core::ffi::c_char {
    let mut rtp_cur: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rtp_size: size_t = 0 as size_t;
    let data_home: *mut ::core::ffi::c_char = if clean_arg as ::core::ffi::c_int != 0 {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        stdpaths_get_xdg_var(kXDGDataHome)
    };
    let config_home: *mut ::core::ffi::c_char = if clean_arg as ::core::ffi::c_int != 0 {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        stdpaths_get_xdg_var(kXDGConfigHome)
    };
    let vimruntime: *mut ::core::ffi::c_char =
        vim_getenv(b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char);
    let libdir: *mut ::core::ffi::c_char = get_lib_dir();
    let data_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGDataDirs);
    let config_dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGConfigDirs);
    let mut data_len: size_t = 0 as size_t;
    let mut config_len: size_t = 0 as size_t;
    let mut vimruntime_len: size_t = 0 as size_t;
    let mut libdir_len: size_t = 0 as size_t;
    let mut appname_len: size_t = strlen(get_appname(false_0 != 0));
    if !data_home.is_null() {
        data_len = strlen(data_home);
        let mut nvim_data_size: size_t = appname_len;
        if data_len != 0 as size_t {
            rtp_size = (rtp_size as ::core::ffi::c_ulong).wrapping_add(
                data_len
                    .wrapping_add(memcnt(
                        data_home as *const ::core::ffi::c_void,
                        ',' as ::core::ffi::c_char,
                        data_len,
                    ))
                    .wrapping_add(nvim_data_size)
                    .wrapping_add(1 as size_t)
                    .wrapping_add(SITE_SIZE)
                    .wrapping_add(1 as size_t)
                    .wrapping_add(
                        (after_pathsep(data_home, data_home.offset(data_len as isize)) == 0)
                            as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_mul(2 as size_t)
                    .wrapping_add(AFTER_SIZE)
                    .wrapping_add(1 as size_t) as ::core::ffi::c_ulong,
            ) as size_t;
        }
    }
    if !config_home.is_null() {
        config_len = strlen(config_home);
        if config_len != 0 as size_t {
            rtp_size = (rtp_size as ::core::ffi::c_ulong).wrapping_add(
                config_len
                    .wrapping_add(memcnt(
                        config_home as *const ::core::ffi::c_void,
                        ',' as ::core::ffi::c_char,
                        config_len,
                    ))
                    .wrapping_add(appname_len)
                    .wrapping_add(1 as size_t)
                    .wrapping_add(
                        (after_pathsep(config_home, config_home.offset(config_len as isize)) == 0)
                            as ::core::ffi::c_int as size_t,
                    )
                    .wrapping_mul(2 as size_t)
                    .wrapping_add(AFTER_SIZE)
                    .wrapping_add(1 as size_t) as ::core::ffi::c_ulong,
            ) as size_t;
        }
    }
    if !vimruntime.is_null() {
        vimruntime_len = strlen(vimruntime);
        if vimruntime_len != 0 as size_t {
            rtp_size = rtp_size.wrapping_add(
                vimruntime_len
                    .wrapping_add(memcnt(
                        vimruntime as *const ::core::ffi::c_void,
                        ',' as ::core::ffi::c_char,
                        vimruntime_len,
                    ))
                    .wrapping_add(1 as size_t),
            );
        }
    }
    if !libdir.is_null() {
        libdir_len = strlen(libdir);
        if libdir_len != 0 as size_t {
            rtp_size = rtp_size.wrapping_add(
                libdir_len
                    .wrapping_add(memcnt(
                        libdir as *const ::core::ffi::c_void,
                        ',' as ::core::ffi::c_char,
                        libdir_len,
                    ))
                    .wrapping_add(1 as size_t),
            );
        }
    }
    rtp_size = rtp_size.wrapping_add(compute_double_env_sep_len(
        data_dirs,
        appname_len
            .wrapping_add(1 as size_t)
            .wrapping_add(SITE_SIZE)
            .wrapping_add(1 as size_t),
        AFTER_SIZE.wrapping_add(1 as size_t),
    ));
    rtp_size = rtp_size.wrapping_add(compute_double_env_sep_len(
        config_dirs,
        appname_len.wrapping_add(1 as size_t),
        AFTER_SIZE.wrapping_add(1 as size_t),
    ));
    let mut rtp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if rtp_size != 0 as size_t {
        rtp = xmalloc(rtp_size) as *mut ::core::ffi::c_char;
        rtp_cur = rtp;
        rtp_cur = add_dir(
            rtp_cur,
            config_home,
            config_len,
            kXDGConfigHome,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        );
        rtp_cur = add_env_sep_dirs(
            rtp_cur,
            config_dirs,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            true_0 != 0,
        );
        rtp_cur = add_dir(
            rtp_cur,
            data_home,
            data_len,
            kXDGDataHome,
            b"site\0".as_ptr() as *const ::core::ffi::c_char,
            SITE_SIZE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        );
        rtp_cur = add_env_sep_dirs(
            rtp_cur,
            data_dirs,
            b"site\0".as_ptr() as *const ::core::ffi::c_char,
            SITE_SIZE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            true_0 != 0,
        );
        rtp_cur = add_dir(
            rtp_cur,
            vimruntime,
            vimruntime_len,
            kXDGNone,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        );
        rtp_cur = add_dir(
            rtp_cur,
            libdir,
            libdir_len,
            kXDGNone,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        );
        rtp_cur = add_env_sep_dirs(
            rtp_cur,
            data_dirs,
            b"site\0".as_ptr() as *const ::core::ffi::c_char,
            SITE_SIZE,
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
            AFTER_SIZE,
            false_0 != 0,
        );
        rtp_cur = add_dir(
            rtp_cur,
            data_home,
            data_len,
            kXDGDataHome,
            b"site\0".as_ptr() as *const ::core::ffi::c_char,
            SITE_SIZE,
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
            AFTER_SIZE,
        );
        rtp_cur = add_env_sep_dirs(
            rtp_cur,
            config_dirs,
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
            AFTER_SIZE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
            false_0 != 0,
        );
        rtp_cur = add_dir(
            rtp_cur,
            config_home,
            config_len,
            kXDGConfigHome,
            b"after\0".as_ptr() as *const ::core::ffi::c_char,
            AFTER_SIZE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        );
        *rtp_cur.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        '_c2rust_label: {
            if rtp_cur.offset_from(rtp) as size_t == rtp_size {
            } else {
                __assert_fail(
                    b"(size_t)(rtp_cur - rtp) == rtp_size\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1908 as ::core::ffi::c_uint,
                    b"char *runtimepath_default(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    xfree(data_dirs as *mut ::core::ffi::c_void);
    xfree(config_dirs as *mut ::core::ffi::c_void);
    xfree(data_home as *mut ::core::ffi::c_void);
    xfree(config_home as *mut ::core::ffi::c_void);
    xfree(vimruntime as *mut ::core::ffi::c_void);
    xfree(libdir as *mut ::core::ffi::c_void);
    return rtp;
}
pub const SITE_SIZE: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize);
pub const AFTER_SIZE: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize);
unsafe extern "C" fn cmd_source(mut fname: *mut ::core::ffi::c_char, mut eap: *mut exarg_T) {
    if *fname as ::core::ffi::c_int != NUL
        && !eap.is_null()
        && (*eap).addr_count > 0 as ::core::ffi::c_int
    {
        emsg(gettext(&raw const e_norange as *const ::core::ffi::c_char));
        return;
    }
    if !eap.is_null() && *fname as ::core::ffi::c_int == NUL {
        if (*eap).forceit != 0 {
            emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        } else {
            cmd_source_buffer(eap, false_0 != 0);
        }
    } else if !eap.is_null() && (*eap).forceit != 0 {
        openscript(
            fname,
            global_busy != 0
                || listcmd_busy as ::core::ffi::c_int != 0
                || !(*eap).nextcmd.is_null()
                || (*(*eap).cstack).cs_idx >= 0 as ::core::ffi::c_int,
        );
    } else if do_source(
        fname,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) == FAIL
    {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_source(mut eap: *mut exarg_T) {
    cmd_source((*eap).arg, eap);
}
#[no_mangle]
pub unsafe extern "C" fn ex_options(mut _eap: *mut exarg_T) {
    let mut buf: [::core::ffi::c_char; 500] = [0; 500];
    let mut multi_mods: bool = false;
    buf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    add_win_cmd_modifiers(
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut cmdmod,
        &raw mut multi_mods,
    );
    os_setenv(
        b"OPTWIN_CMD\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut buf as *mut ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    cmd_source(
        SYS_OPTWIN_FILE.as_ptr() as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<exarg_T>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn source_breakpoint(mut cookie: *mut ::core::ffi::c_void) -> *mut linenr_T {
    return &raw mut (*(cookie as *mut source_cookie_T)).breakpoint;
}
#[no_mangle]
pub unsafe extern "C" fn source_dbg_tick(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_int {
    return &raw mut (*(cookie as *mut source_cookie_T)).dbg_tick;
}
#[no_mangle]
pub unsafe extern "C" fn source_level(mut cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    return (*(cookie as *mut source_cookie_T)).level;
}
unsafe extern "C" fn fopen_noinh_readbin(mut filename: *mut ::core::ffi::c_char) -> *mut FILE {
    let mut fd_tmp: ::core::ffi::c_int = os_open(filename, O_RDONLY, 0 as ::core::ffi::c_int);
    if fd_tmp < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<FILE>();
    }
    os_set_cloexec(fd_tmp);
    return fdopen(fd_tmp, READBIN.as_ptr());
}
unsafe extern "C" fn concat_continued_line(
    ga: *mut garray_T,
    init_growsize: ::core::ffi::c_int,
    p: *const ::core::ffi::c_char,
    mut len: size_t,
) -> bool {
    let line: *const ::core::ffi::c_char = skipwhite_len(p, len);
    len = len.wrapping_sub(line.offset_from(p) as size_t);
    if len >= 3 as size_t
        && strncmp(
            line,
            b"\"\\ \0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    } else if len == 0 as size_t
        || *line.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if (*ga).ga_len > init_growsize {
        ga_set_growsize(
            ga,
            if (*ga).ga_len < 8000 as ::core::ffi::c_int {
                (*ga).ga_len
            } else {
                8000 as ::core::ffi::c_int
            },
        );
    }
    ga_concat_len(
        ga,
        line.offset(1 as ::core::ffi::c_int as isize),
        len.wrapping_sub(1 as size_t),
    );
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn new_script_item(
    name: *mut ::core::ffi::c_char,
    sid_out: *mut scid_T,
) -> *mut scriptitem_T {
    static mut last_current_SID: scid_T = 0 as scid_T;
    last_current_SID += 1;
    let sid: scid_T = last_current_SID;
    if !sid_out.is_null() {
        *sid_out = sid;
    }
    ga_grow(
        &raw mut script_items,
        sid as ::core::ffi::c_int - script_items.ga_len,
    );
    while script_items.ga_len < sid {
        let mut si: *mut scriptitem_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<scriptitem_T>()) as *mut scriptitem_T;
        script_items.ga_len += 1;
        *(script_items.ga_data as *mut *mut scriptitem_T)
            .offset((script_items.ga_len - 1 as ::core::ffi::c_int) as isize) = si;
        (*si).sn_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
        new_script_vars(script_items.ga_len as scid_T);
        (*si).sn_prof_on = false_0 != 0;
    }
    (**(script_items.ga_data as *mut *mut scriptitem_T)
        .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
    .sn_name = name;
    return *(script_items.ga_data as *mut *mut scriptitem_T)
        .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize);
}
unsafe extern "C" fn do_source_buffer_init(
    mut sp: *mut source_cookie_T,
    mut eap: *const exarg_T,
    mut ex_lua: bool,
) -> *mut ::core::ffi::c_char {
    if curbuf.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*curbuf).b_ffname.is_null() {
        fname = xstrdup((*curbuf).b_ffname);
    } else {
        if ex_lua {
            snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b":{range}lua buffer=%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*curbuf).handle,
            );
        } else {
            snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b":source buffer=%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*curbuf).handle,
            );
        }
        fname = xstrdup(&raw mut IObuff as *mut ::core::ffi::c_char);
    }
    ga_init(
        &raw mut (*sp).buflines,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    let mut curr_lnum: linenr_T = (*eap).line1;
    while curr_lnum <= (*eap).line2 {
        ga_grow(&raw mut (*sp).buflines, 1 as ::core::ffi::c_int);
        *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
            .offset((*sp).buflines.ga_len as isize) = xstrdup(ml_get(curr_lnum));
        (*sp).buflines.ga_len += 1;
        curr_lnum += 1;
    }
    (*sp).buf_lnum = 0 as ::core::ffi::c_int;
    (*sp).source_from_buf_or_str = true_0 != 0;
    (*sp).sourcing_lnum = (*eap).line1 - 1 as linenr_T;
    return fname;
}
unsafe extern "C" fn do_source_str_init(
    mut sp: *mut source_cookie_T,
    mut str: *const ::core::ffi::c_char,
) {
    ga_init(
        &raw mut (*sp).buflines,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    while *str as ::core::ffi::c_int != NUL {
        let mut eol: *const ::core::ffi::c_char = skip_to_newline(str);
        ga_grow(&raw mut (*sp).buflines, 1 as ::core::ffi::c_int);
        *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
            .offset((*sp).buflines.ga_len as isize) = xmemdupz(
            str as *const ::core::ffi::c_void,
            eol.offset_from(str) as size_t,
        ) as *mut ::core::ffi::c_char;
        (*sp).buflines.ga_len += 1;
        str = eol.offset((*eol as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as isize);
    }
    (*sp).buf_lnum = 0 as ::core::ffi::c_int;
    (*sp).source_from_buf_or_str = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_source_buffer(eap: *const exarg_T, mut ex_lua: bool) {
    do_source_ext(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        eap,
        ex_lua,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn do_source_str(
    mut str: *const ::core::ffi::c_char,
    mut traceback_name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let sourcing_name: *mut ::core::ffi::c_char = (*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_name;
    let sourcing_lnum: linenr_T = (*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    let mut sname_buf: [::core::ffi::c_char; 256] = [0; 256];
    if !sourcing_name.is_null() {
        snprintf(
            &raw mut sname_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"%s called at %s:%d\0".as_ptr() as *const ::core::ffi::c_char,
            traceback_name,
            sourcing_name,
            sourcing_lnum,
        );
        traceback_name = &raw mut sname_buf as *mut ::core::ffi::c_char;
    }
    return do_source_ext(
        traceback_name,
        false_0 != 0,
        DOSO_NONE as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null::<exarg_T>(),
        false_0 != 0,
        str,
    );
}
unsafe extern "C" fn do_source_ext(
    fname: *mut ::core::ffi::c_char,
    check_other: bool,
    is_vimrc: ::core::ffi::c_int,
    ret_sid: *mut ::core::ffi::c_int,
    eap: *const exarg_T,
    ex_lua: bool,
    str: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sid: ::core::ffi::c_int = 0;
    let mut rel_time: proftime_T = 0;
    let mut start_time: proftime_T = 0;
    let mut l_time_fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut l_do_profiling: ::core::ffi::c_int = 0;
    let mut funccalp_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut ts_lua: bool = false;
    let mut cookie: source_cookie_T = source_cookie_T {
        fp: ::core::ptr::null_mut::<FILE>(),
        nextline: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sourcing_lnum: 0,
        finished: false,
        source_from_buf_or_str: false,
        buf_lnum: 0,
        buflines: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        breakpoint: 0,
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        dbg_tick: 0,
        level: 0,
        conv: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
    };
    let mut firstline: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut save_debug_break_level: ::core::ffi::c_int = debug_break_level;
    let mut si: *mut scriptitem_T = ::core::ptr::null_mut::<scriptitem_T>();
    let mut wait_start: proftime_T = 0;
    let mut trigger_source_post: bool = false_0 != 0;
    memset(
        &raw mut cookie as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<source_cookie_T>(),
    );
    let mut fname_exp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_theend: {
        if fname.is_null() {
            '_c2rust_label: {
                if str.is_null() {
                } else {
                    __assert_fail(
                        b"str == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        2184 as ::core::ffi::c_uint,
                        b"int do_source_ext(char *const, const _Bool, const int, int *const, const exarg_T *const, const _Bool, const char *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            fname_exp = do_source_buffer_init(&raw mut cookie, eap, ex_lua);
            if fname_exp.is_null() {
                return FAIL;
            }
        } else if !str.is_null() {
            do_source_str_init(&raw mut cookie, str);
            fname_exp = xstrdup(fname);
        } else {
            let mut p: *mut ::core::ffi::c_char = expand_env_save(fname);
            if p.is_null() {
                return retval;
            }
            fname_exp = fix_fname(p);
            xfree(p as *mut ::core::ffi::c_void);
            if fname_exp.is_null() {
                return retval;
            }
            if os_isdir(fname_exp) {
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Cannot source a directory: \"%s\"\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                );
                break '_theend;
            }
        }
        sid = if !str.is_null() {
            SID_STR
        } else {
            find_script_by_name(fname_exp)
        };
        if sid > 0 as ::core::ffi::c_int && !ret_sid.is_null() {
            *ret_sid = sid;
            retval = OK;
        } else {
            if str.is_null() {
                if has_autocmd(EVENT_SOURCECMD, fname_exp, ::core::ptr::null_mut::<buf_T>())
                    as ::core::ffi::c_int
                    != 0
                    && apply_autocmds(EVENT_SOURCECMD, fname_exp, fname_exp, false_0 != 0, curbuf)
                        as ::core::ffi::c_int
                        != 0
                {
                    retval = if aborting() as ::core::ffi::c_int != 0 {
                        FAIL
                    } else {
                        OK
                    };
                    if retval == OK {
                        apply_autocmds(
                            EVENT_SOURCEPOST,
                            fname_exp,
                            fname_exp,
                            false_0 != 0,
                            curbuf,
                        );
                    }
                    break '_theend;
                } else {
                    apply_autocmds(EVENT_SOURCEPRE, fname_exp, fname_exp, false_0 != 0, curbuf);
                }
            }
            if !cookie.source_from_buf_or_str {
                cookie.fp = fopen_noinh_readbin(fname_exp);
            }
            if cookie.fp.is_null() && check_other as ::core::ffi::c_int != 0 {
                let mut p_0: *mut ::core::ffi::c_char = path_tail(fname_exp);
                if (*p_0 as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                    || *p_0 as ::core::ffi::c_int == '_' as ::core::ffi::c_int)
                    && (strcasecmp(
                        p_0.offset(1 as ::core::ffi::c_int as isize),
                        b"nvimrc\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                        || strcasecmp(
                            p_0.offset(1 as ::core::ffi::c_int as isize),
                            b"exrc\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int)
                {
                    *p_0 = (if *p_0 as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                        '.' as ::core::ffi::c_int
                    } else {
                        '_' as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                    cookie.fp = fopen_noinh_readbin(fname_exp);
                }
            }
            if cookie.fp.is_null() && !cookie.source_from_buf_or_str {
                if p_verbose > 1 as OptInt {
                    verbose_enter();
                    if (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name
                    .is_null()
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"could not source \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            fname,
                        );
                    } else {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"line %ld: could not source \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*(exestack.ga_data as *mut estack_T)
                                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                            .es_lnum as int64_t,
                            fname,
                        );
                    }
                    verbose_leave();
                }
            } else {
                if p_verbose > 1 as OptInt {
                    verbose_enter();
                    if (*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name
                    .is_null()
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"sourcing \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                            fname,
                        );
                    } else {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"line %ld: sourcing \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            (*(exestack.ga_data as *mut estack_T)
                                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                            .es_lnum as int64_t,
                            fname,
                        );
                    }
                    verbose_leave();
                }
                if is_vimrc == DOSO_VIMRC as ::core::ffi::c_int {
                    vimrc_found(
                        fname_exp,
                        b"MYVIMRC\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                }
                cookie.breakpoint = dbg_find_breakpoint(true_0 != 0, fname_exp, 0 as linenr_T);
                cookie.fname = fname_exp;
                cookie.dbg_tick = debug_tick;
                cookie.level = ex_nesting_level;
                rel_time = 0;
                start_time = 0;
                l_time_fd = time_fd;
                if !l_time_fd.is_null() {
                    time_push(&raw mut rel_time, &raw mut start_time);
                }
                l_do_profiling = do_profiling;
                if l_do_profiling == PROF_YES {
                    prof_child_enter(&raw mut wait_start);
                }
                funccalp_entry = funccal_entry_T {
                    top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    next: ::core::ptr::null_mut::<funccal_entry_T>(),
                };
                save_funccal(&raw mut funccalp_entry);
                save_current_sctx = current_sctx;
                last_current_SID_seq += 1;
                current_sctx.sc_seq = last_current_SID_seq;
                if sid > 0 as ::core::ffi::c_int {
                    si = *(script_items.ga_data as *mut *mut scriptitem_T)
                        .offset((sid - 1 as ::core::ffi::c_int) as isize);
                } else if str.is_null() {
                    si = new_script_item(fname_exp, &raw mut sid);
                    (*si).sn_lua = path_with_extension(
                        fname_exp,
                        b"lua\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    fname_exp = xstrdup((*si).sn_name);
                    if !ret_sid.is_null() {
                        *ret_sid = sid;
                    }
                }
                '_c2rust_label_0: {
                    if !si.is_null() as ::core::ffi::c_int == str.is_null() as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"(si != NULL) == (str == NULL)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2332 as ::core::ffi::c_uint,
                            b"int do_source_ext(char *const, const _Bool, const int, int *const, const exarg_T *const, const _Bool, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if str.is_null() || !script_is_lua(current_sctx.sc_sid) {
                    current_sctx.sc_sid = sid as scid_T;
                    current_sctx.sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
                }
                estack_push(
                    ETYPE_SCRIPT,
                    if !si.is_null() {
                        (*si).sn_name
                    } else {
                        fname_exp
                    },
                    0 as linenr_T,
                );
                if l_do_profiling == PROF_YES && !si.is_null() {
                    let mut forceit: bool = false_0 != 0;
                    if !(*si).sn_prof_on
                        && has_profiling(true_0 != 0, (*si).sn_name, &raw mut forceit)
                            as ::core::ffi::c_int
                            != 0
                    {
                        profile_init(si);
                        (*si).sn_pr_force = forceit;
                    }
                    if (*si).sn_prof_on {
                        (*si).sn_pr_count += 1;
                        (*si).sn_pr_start = profile_start();
                        (*si).sn_pr_children = profile_zero();
                    }
                }
                cookie.conv.vc_type = CONV_NONE as ::core::ffi::c_int;
                ts_lua = false_0 != 0;
                if fname.is_null()
                    && !eap.is_null()
                    && !ex_lua
                    && !strequal(
                        (*curbuf).b_p_ft,
                        b"lua\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                    && !(!(*curbuf).b_fname.is_null()
                        && path_with_extension(
                            (*curbuf).b_fname,
                            b"lua\0".as_ptr() as *const ::core::ffi::c_char,
                        ) as ::core::ffi::c_int
                            != 0)
                {
                    let mut args: Array = ARRAY_DICT_INIT;
                    let mut args__items: [Object; 3] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    }; 3];
                    args.capacity = 3 as size_t;
                    args.items = &raw mut args__items as *mut Object;
                    let c2rust_fresh0 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh0 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*curbuf).handle as Integer,
                        },
                    };
                    let c2rust_fresh1 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*eap).line1 as Integer,
                        },
                    };
                    let c2rust_fresh2 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*eap).line2 as Integer,
                        },
                    };
                    let mut err: Error = Error {
                        type_0: kErrorTypeNone,
                        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    };
                    let mut result: Object = nlua_exec(
                        String_0 {
                            data: b"return require('vim._core.util').source_is_lua(...)\0".as_ptr()
                                as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 52]>()
                                .wrapping_sub(1 as size_t),
                        },
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        args,
                        kRetNilBool,
                        ::core::ptr::null_mut::<Arena>(),
                        &raw mut err,
                    );
                    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
                        && (result.type_0 as ::core::ffi::c_uint
                            == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                            && result.data.boolean as ::core::ffi::c_int == true_0)
                    {
                        ts_lua = true_0 != 0;
                    }
                    api_clear_error(&raw mut err);
                }
                if fname.is_null()
                    && (ex_lua as ::core::ffi::c_int != 0
                        || ts_lua as ::core::ffi::c_int != 0
                        || strequal(
                            (*curbuf).b_p_ft,
                            b"lua\0".as_ptr() as *const ::core::ffi::c_char,
                        ) as ::core::ffi::c_int
                            != 0
                        || !(*curbuf).b_fname.is_null()
                            && path_with_extension(
                                (*curbuf).b_fname,
                                b"lua\0".as_ptr() as *const ::core::ffi::c_char,
                            ) as ::core::ffi::c_int
                                != 0)
                {
                    nlua_exec_ga(&raw mut cookie.buflines, fname_exp);
                } else if !si.is_null() && (*si).sn_lua as ::core::ffi::c_int != 0 {
                    nlua_exec_file(fname_exp);
                } else {
                    firstline = getsourceline(
                        0 as ::core::ffi::c_int,
                        &raw mut cookie as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        true_0 != 0,
                    ) as *mut uint8_t;
                    if !firstline.is_null()
                        && strlen(firstline as *mut ::core::ffi::c_char) >= 3 as size_t
                        && *firstline.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 0xef as ::core::ffi::c_int
                        && *firstline.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 0xbb as ::core::ffi::c_int
                        && *firstline.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 0xbf as ::core::ffi::c_int
                    {
                        convert_setup(
                            &raw mut cookie.conv,
                            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            p_enc,
                        );
                        let mut p_1: *mut ::core::ffi::c_char = string_convert(
                            &raw mut cookie.conv,
                            (firstline as *mut ::core::ffi::c_char)
                                .offset(3 as ::core::ffi::c_int as isize),
                            ::core::ptr::null_mut::<size_t>(),
                        );
                        if p_1.is_null() {
                            p_1 = xstrdup(
                                (firstline as *mut ::core::ffi::c_char)
                                    .offset(3 as ::core::ffi::c_int as isize),
                            );
                        }
                        xfree(firstline as *mut ::core::ffi::c_void);
                        firstline = p_1 as *mut uint8_t;
                    }
                    do_cmdline(
                        firstline as *mut ::core::ffi::c_char,
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
                        &raw mut cookie as *mut ::core::ffi::c_void,
                        DOCMD_VERBOSE as ::core::ffi::c_int
                            | DOCMD_NOWAIT as ::core::ffi::c_int
                            | DOCMD_REPEAT as ::core::ffi::c_int,
                    );
                }
                retval = OK;
                if l_do_profiling == PROF_YES && !si.is_null() {
                    si = *(script_items.ga_data as *mut *mut scriptitem_T).offset(
                        (current_sctx.sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as isize,
                    );
                    if (*si).sn_prof_on {
                        (*si).sn_pr_start = profile_end((*si).sn_pr_start);
                        (*si).sn_pr_start = profile_sub_wait(wait_start, (*si).sn_pr_start);
                        (*si).sn_pr_total = profile_add((*si).sn_pr_total, (*si).sn_pr_start);
                        (*si).sn_pr_self =
                            profile_self((*si).sn_pr_self, (*si).sn_pr_start, (*si).sn_pr_children);
                    }
                }
                if got_int {
                    emsg(gettext(&raw const e_interr as *const ::core::ffi::c_char));
                }
                estack_pop();
                if p_verbose > 1 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"finished sourcing %s\0".as_ptr() as *const ::core::ffi::c_char),
                        fname,
                    );
                    if !(*(exestack.ga_data as *mut estack_T)
                        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name
                    .is_null()
                    {
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"continuing in %s\0".as_ptr() as *const ::core::ffi::c_char),
                            (*(exestack.ga_data as *mut estack_T)
                                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                            .es_name,
                        );
                    }
                    verbose_leave();
                }
                if !l_time_fd.is_null() {
                    vim_snprintf(
                        &raw mut IObuff as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"sourcing %s\0".as_ptr() as *const ::core::ffi::c_char,
                        fname,
                    );
                    time_msg(
                        &raw mut IObuff as *mut ::core::ffi::c_char,
                        &raw mut start_time,
                    );
                    time_pop(rel_time);
                }
                if !got_int {
                    trigger_source_post = true_0 != 0;
                }
                if save_debug_break_level > ex_nesting_level
                    && debug_break_level == ex_nesting_level
                {
                    debug_break_level += 1;
                }
                current_sctx = save_current_sctx;
                restore_funccal();
                if l_do_profiling == PROF_YES {
                    prof_child_exit(&raw mut wait_start);
                }
                if !cookie.fp.is_null() {
                    fclose(cookie.fp);
                }
                if cookie.source_from_buf_or_str {
                    ga_clear_strings(&raw mut cookie.buflines);
                }
                xfree(cookie.nextline as *mut ::core::ffi::c_void);
                xfree(firstline as *mut ::core::ffi::c_void);
                convert_setup(
                    &raw mut cookie.conv,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                if str.is_null() && trigger_source_post as ::core::ffi::c_int != 0 {
                    apply_autocmds(EVENT_SOURCEPOST, fname_exp, fname_exp, false_0 != 0, curbuf);
                }
            }
        }
    }
    xfree(fname_exp as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn do_source(
    mut fname: *mut ::core::ffi::c_char,
    mut check_other: bool,
    mut is_vimrc: ::core::ffi::c_int,
    mut ret_sid: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return do_source_ext(
        fname,
        check_other,
        is_vimrc,
        ret_sid,
        ::core::ptr::null::<exarg_T>(),
        false_0 != 0,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn script_is_lua(mut sid: scid_T) -> bool {
    if sid == SID_LUA {
        return true_0 != 0;
    }
    if !(sid > 0 as ::core::ffi::c_int && sid <= script_items.ga_len) {
        return false_0 != 0;
    }
    return (**(script_items.ga_data as *mut *mut scriptitem_T)
        .offset((sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
    .sn_lua;
}
#[no_mangle]
pub unsafe extern "C" fn find_script_by_name(
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if script_items.ga_len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"script_items.ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/runtime.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2498 as ::core::ffi::c_uint,
                b"int find_script_by_name(char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut sid: ::core::ffi::c_int = script_items.ga_len;
    while sid > 0 as ::core::ffi::c_int {
        let mut si: *mut scriptitem_T = *(script_items.ga_data as *mut *mut scriptitem_T)
            .offset((sid - 1 as ::core::ffi::c_int) as isize);
        if !(*si).sn_name.is_null() && path_fnamecmp((*si).sn_name, name) == 0 as ::core::ffi::c_int
        {
            return sid;
        }
        sid -= 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ex_scriptnames(mut eap: *mut exarg_T) {
    if (*eap).addr_count > 0 as ::core::ffi::c_int || *(*eap).arg as ::core::ffi::c_int != NUL {
        if (*eap).addr_count > 0 as ::core::ffi::c_int
            && !((*eap).line2 > 0 as linenr_T && (*eap).line2 <= script_items.ga_len as linenr_T)
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).arg = (**(script_items.ga_data as *mut *mut scriptitem_T)
                    .offset(((*eap).line2 - 1 as linenr_T) as isize))
                .sn_name;
            } else {
                expand_env(
                    (*eap).arg,
                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                    MAXPATHL,
                );
                (*eap).arg = &raw mut NameBuff as *mut ::core::ffi::c_char;
            }
            do_exedit(eap, ::core::ptr::null_mut::<win_T>());
        }
        return;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= script_items.ga_len && !got_int {
        if !(**(script_items.ga_data as *mut *mut scriptitem_T)
            .offset((i - 1 as ::core::ffi::c_int) as isize))
        .sn_name
        .is_null()
        {
            home_replace(
                ::core::ptr::null::<buf_T>(),
                (**(script_items.ga_data as *mut *mut scriptitem_T)
                    .offset((i - 1 as ::core::ffi::c_int) as isize))
                .sn_name,
                &raw mut NameBuff as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
            vim_snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%3d: %s\0".as_ptr() as *const ::core::ffi::c_char,
                i,
                &raw mut NameBuff as *mut ::core::ffi::c_char,
            );
            if !message_filtered(&raw mut IObuff as *mut ::core::ffi::c_char) {
                if msg_col > 0 as ::core::ffi::c_int {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                msg_outtrans(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                line_breakcheck();
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_scriptname(
    mut script_ctx: sctx_T,
    mut should_free: *mut bool,
) -> *mut ::core::ffi::c_char {
    if !should_free.is_null() {
        *should_free = false_0 != 0;
    }
    match script_ctx.sc_sid {
        SID_MODELINE => {
            return gettext(b"modeline\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_CMDARG => {
            return gettext(b"--cmd argument\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_CARG => {
            return gettext(b"-c argument\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_ENV => {
            return gettext(b"environment variable\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_ERROR => {
            return gettext(b"error handler\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_WINLAYOUT => {
            return gettext(b"changed window size\0".as_ptr() as *const ::core::ffi::c_char);
        }
        SID_LUA => return gettext(b"Lua\0".as_ptr() as *const ::core::ffi::c_char),
        SID_API_CLIENT => {
            snprintf(
                &raw mut IObuff as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"API client (channel id %lu)\0".as_ptr() as *const ::core::ffi::c_char),
                script_ctx.sc_chan,
            );
            return &raw mut IObuff as *mut ::core::ffi::c_char;
        }
        SID_STR => {
            return gettext(b"anonymous :source\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {
            let sname: *mut ::core::ffi::c_char =
                (**(script_items.ga_data as *mut *mut scriptitem_T).offset(
                    (script_ctx.sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize,
                ))
                .sn_name;
            if sname.is_null() {
                snprintf(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"anonymous :source (script id %d)\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    script_ctx.sc_sid,
                );
                return &raw mut IObuff as *mut ::core::ffi::c_char;
            }
            if !should_free.is_null() {
                *should_free = true_0 != 0;
                return home_replace_save(::core::ptr::null_mut::<buf_T>(), sname);
            } else {
                return sname;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn free_autoload_scriptnames() {
    ga_clear_strings(&raw mut ga_loaded);
}
#[no_mangle]
pub unsafe extern "C" fn get_sourced_lnum(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> linenr_T {
    return if fgetline
        == Some(
            getsourceline
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ) {
        (*(cookie as *mut source_cookie_T)).sourcing_lnum
    } else {
        (*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
    };
}
unsafe extern "C" fn get_script_local_funcs(mut sid: scid_T) -> *mut list_T {
    let functbl: *mut hashtab_T = func_tbl_get();
    let mut l: *mut list_T = tv_list_alloc((*functbl).ht_used as ptrdiff_t);
    let hiht_: *mut hashtab_T = functbl;
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
            hitodo_ = hitodo_.wrapping_sub(1);
            let fp: *const ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            if (*fp).uf_script_ctx.sc_sid == sid {
                let name: *const ::core::ffi::c_char = if !(*fp).uf_name_exp.is_null() {
                    (*fp).uf_name_exp as *const ::core::ffi::c_char
                } else {
                    &raw const (*fp).uf_name as *const ::core::ffi::c_char
                };
                tv_list_append_string(l, name, -1 as ssize_t);
            }
        }
        hi = hi.offset(1);
    }
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn f_getscriptinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, script_items.ga_len as ptrdiff_t);
    if tv_check_for_opt_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut l: *mut list_T = (*rettv).vval.v_list;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: p_ic != 0,
    };
    let mut filterpat: bool = false_0 != 0;
    let mut sid: varnumber_T = -1 as varnumber_T;
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut sid_di: *mut dictitem_T = tv_dict_find(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"sid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !sid_di.is_null() {
            let mut error: bool = false_0 != 0;
            sid = tv_get_number_chk(&raw mut (*sid_di).di_tv, &raw mut error);
            if error {
                return;
            }
            if sid <= 0 as varnumber_T {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"sid\0".as_ptr() as *const ::core::ffi::c_char,
                    tv_get_string(&raw mut (*sid_di).di_tv),
                );
                return;
            }
        } else {
            pat = tv_dict_get_string(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
                b"name\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
            if !pat.is_null() {
                regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
            }
            if !regmatch.regprog.is_null() {
                filterpat = true_0 != 0;
            }
        }
    }
    let mut i: varnumber_T = if sid > 0 as varnumber_T {
        sid
    } else {
        1 as varnumber_T
    };
    while (i == sid || sid <= 0 as varnumber_T) && i <= script_items.ga_len as varnumber_T {
        let mut si: *mut scriptitem_T = *(script_items.ga_data as *mut *mut scriptitem_T)
            .offset((i - 1 as varnumber_T) as isize);
        if !(*si).sn_name.is_null() {
            if !(filterpat as ::core::ffi::c_int != 0
                && !vim_regexec(&raw mut regmatch, (*si).sn_name, 0 as colnr_T))
            {
                let mut d: *mut dict_T = tv_dict_alloc();
                tv_list_append_dict(l, d);
                tv_dict_add_str(
                    d,
                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                    (*si).sn_name,
                );
                tv_dict_add_nr(
                    d,
                    b"sid\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    i,
                );
                tv_dict_add_nr(
                    d,
                    b"version\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    1 as varnumber_T,
                );
                tv_dict_add_bool(
                    d,
                    b"autoload\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    kBoolVarFalse,
                );
                if sid > 0 as varnumber_T {
                    let mut var_dict: *mut dict_T = tv_dict_copy(
                        ::core::ptr::null::<vimconv_T>(),
                        &raw mut (*(*si).sn_vars).sv_dict,
                        true_0 != 0,
                        get_copyID(),
                    );
                    tv_dict_add_dict(
                        d,
                        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                        var_dict,
                    );
                    tv_dict_add_list(
                        d,
                        b"functions\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 10]>()
                            .wrapping_sub(1 as size_t),
                        get_script_local_funcs(sid as scid_T),
                    );
                }
            }
        }
        i += 1;
    }
    vim_regfree(regmatch.regprog);
    xfree(pat as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn getsourceline(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut sp: *mut source_cookie_T = cookie as *mut source_cookie_T;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*sp).dbg_tick < debug_tick && !(*sp).source_from_buf_or_str {
        (*sp).breakpoint = dbg_find_breakpoint(
            true_0 != 0,
            (*sp).fname,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*sp).dbg_tick = debug_tick;
    }
    if do_profiling == PROF_YES {
        script_line_end();
    }
    (*(exestack.ga_data as *mut estack_T)
        .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum = (*sp).sourcing_lnum + 1 as linenr_T;
    if (*sp).finished as ::core::ffi::c_int != 0
        || !(*sp).source_from_buf_or_str && (*sp).fp.is_null()
    {
        line = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if (*sp).nextline.is_null() {
        line = get_one_sourceline(sp);
    } else {
        line = (*sp).nextline;
        (*sp).nextline = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*sp).sourcing_lnum += 1;
    }
    if !line.is_null() && do_profiling == PROF_YES {
        script_line_start();
    }
    if !line.is_null()
        && do_concat as ::core::ffi::c_int != 0
        && vim_strchr(p_cpo, CPO_CONCAT).is_null()
    {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*sp).sourcing_lnum -= 1;
        (*sp).nextline = get_one_sourceline(sp);
        if !(*sp).nextline.is_null() && {
            p = skipwhite((*sp).nextline);
            *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '"' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
        } {
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
                400 as ::core::ffi::c_int,
            );
            ga_concat(&raw mut ga, line);
            while !(*sp).nextline.is_null()
                && concat_continued_line(
                    &raw mut ga,
                    400 as ::core::ffi::c_int,
                    (*sp).nextline,
                    strlen((*sp).nextline),
                ) as ::core::ffi::c_int
                    != 0
            {
                xfree((*sp).nextline as *mut ::core::ffi::c_void);
                (*sp).nextline = get_one_sourceline(sp);
            }
            ga_append(&raw mut ga, NUL as uint8_t);
            xfree(line as *mut ::core::ffi::c_void);
            line = ga.ga_data as *mut ::core::ffi::c_char;
        }
    }
    if !line.is_null() && (*sp).conv.vc_type != CONV_NONE as ::core::ffi::c_int {
        let mut s: *mut ::core::ffi::c_char =
            string_convert(&raw mut (*sp).conv, line, ::core::ptr::null_mut::<size_t>());
        if !s.is_null() {
            xfree(line as *mut ::core::ffi::c_void);
            line = s;
        }
    }
    if !(*sp).source_from_buf_or_str
        && (*sp).breakpoint != 0 as linenr_T
        && (*sp).breakpoint
            <= (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
    {
        dbg_breakpoint(
            (*sp).fname,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*sp).breakpoint = dbg_find_breakpoint(
            true_0 != 0,
            (*sp).fname,
            (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*sp).dbg_tick = debug_tick;
    }
    return line;
}
unsafe extern "C" fn get_one_sourceline(mut sp: *mut source_cookie_T) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut len: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut have_read: bool = false_0 != 0;
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        250 as ::core::ffi::c_int,
    );
    (*sp).sourcing_lnum += 1;
    's_138: loop {
        ga_grow(&raw mut ga, 120 as ::core::ffi::c_int);
        if (*sp).source_from_buf_or_str {
            if (*sp).buf_lnum >= (*sp).buflines.ga_len {
                break;
            }
            ga_concat(
                &raw mut ga,
                *((*sp).buflines.ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((*sp).buf_lnum as isize),
            );
            (*sp).buf_lnum += 1;
            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
            buf = ga.ga_data as *mut ::core::ffi::c_char;
            let c2rust_fresh3 = ga.ga_len;
            ga.ga_len = ga.ga_len + 1;
            *buf.offset(c2rust_fresh3 as isize) = NUL as ::core::ffi::c_char;
            len = ga.ga_len;
        } else {
            buf = ga.ga_data as *mut ::core::ffi::c_char;
            loop {
                *__errno_location() = 0 as ::core::ffi::c_int;
                if !fgets(
                    buf.offset(ga.ga_len as isize),
                    ga.ga_maxlen - ga.ga_len,
                    (*sp).fp,
                )
                .is_null()
                {
                    break;
                }
                if *__errno_location() != EINTR {
                    break 's_138;
                }
            }
            len = ga.ga_len + strlen(buf.offset(ga.ga_len as isize)) as ::core::ffi::c_int;
        }
        have_read = true_0 != 0;
        ga.ga_len = len;
        if ga.ga_maxlen - ga.ga_len == 1 as ::core::ffi::c_int
            && *buf.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                != '\n' as ::core::ffi::c_int
        {
            continue;
        }
        if len >= 1 as ::core::ffi::c_int
            && *buf.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '\n' as ::core::ffi::c_int
        {
            c = len - 2 as ::core::ffi::c_int;
            while c >= 0 as ::core::ffi::c_int
                && *buf.offset(c as isize) as ::core::ffi::c_int == Ctrl_V
            {
                c -= 1;
            }
            if len & 1 as ::core::ffi::c_int != c & 1 as ::core::ffi::c_int {
                (*sp).sourcing_lnum += 1;
                continue;
            } else {
                *buf.offset((len - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
            }
        }
        line_breakcheck();
        break;
    }
    if have_read {
        return ga.ga_data as *mut ::core::ffi::c_char;
    }
    xfree(ga.ga_data);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn sourcing_a_script(mut eap: *mut exarg_T) -> ::core::ffi::c_int {
    return getline_equal(
        (*eap).ea_getline,
        (*eap).cookie,
        Some(
            getsourceline
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ),
    ) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ex_scriptencoding(mut eap: *mut exarg_T) {
    if sourcing_a_script(eap) == 0 {
        emsg(gettext(
            b"E167: :scriptencoding used outside of a sourced file\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut name: *mut ::core::ffi::c_char = if *(*eap).arg as ::core::ffi::c_int != NUL {
        enc_canonize((*eap).arg)
    } else {
        (*eap).arg
    };
    let mut sp: *mut source_cookie_T =
        getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T;
    convert_setup(&raw mut (*sp).conv, name, p_enc);
    if name != (*eap).arg {
        xfree(name as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_finish(mut eap: *mut exarg_T) {
    if sourcing_a_script(eap) != 0 {
        do_finish(eap, false_0 != 0);
    } else {
        emsg(gettext(
            b"E168: :finish used outside of a sourced file\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_finish(mut eap: *mut exarg_T, mut reanimate: bool) {
    if reanimate {
        (*(getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T)).finished =
            false_0 != 0;
    }
    let mut idx: ::core::ffi::c_int =
        cleanup_conditionals((*eap).cstack, 0 as ::core::ffi::c_int, true_0);
    if idx >= 0 as ::core::ffi::c_int {
        (*(*eap).cstack).cs_pending[idx as usize] =
            CSTP_FINISH as ::core::ffi::c_int as ::core::ffi::c_char;
        report_make_pending(CSTP_FINISH as ::core::ffi::c_int, NULL_0);
    } else {
        (*(getline_cookie((*eap).ea_getline, (*eap).cookie) as *mut source_cookie_T)).finished =
            true_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn source_finished(
    mut fgetline: LineGetter,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    return getline_equal(
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
        && (*(getline_cookie(fgetline, cookie) as *mut source_cookie_T)).finished
            as ::core::ffi::c_int
            != 0;
}
#[no_mangle]
pub unsafe extern "C" fn autoload_name(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
) -> *mut ::core::ffi::c_char {
    let scriptname: *mut ::core::ffi::c_char =
        xmalloc(name_len.wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 14]>()))
            as *mut ::core::ffi::c_char;
    memcpy(
        scriptname as *mut ::core::ffi::c_void,
        b"autoload/\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
    );
    memcpy(
        scriptname
            .offset(::core::mem::size_of::<[::core::ffi::c_char; 10]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        name as *const ::core::ffi::c_void,
        name_len,
    );
    let mut auchar_idx: size_t = 0 as size_t;
    let mut i: size_t =
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t);
    while i
        .wrapping_sub(::core::mem::size_of::<[::core::ffi::c_char; 10]>())
        .wrapping_add(1 as size_t)
        < name_len
    {
        if *scriptname.offset(i as isize) as ::core::ffi::c_int == AUTOLOAD_CHAR {
            *scriptname.offset(i as isize) = '/' as ::core::ffi::c_char;
            auchar_idx = i;
        }
        i = i.wrapping_add(1);
    }
    memcpy(
        scriptname.offset(auchar_idx as isize) as *mut ::core::ffi::c_void,
        b".vim\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>(),
    );
    return scriptname;
}
#[no_mangle]
pub unsafe extern "C" fn script_autoload(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    reload: bool,
) -> bool {
    let mut p: *const ::core::ffi::c_char =
        memchr(name as *const ::core::ffi::c_void, AUTOLOAD_CHAR, name_len)
            as *const ::core::ffi::c_char;
    if p.is_null() || p == name {
        return false_0 != 0;
    }
    let mut ret: bool = false_0 != 0;
    let mut tofree: *mut ::core::ffi::c_char = autoload_name(name, name_len);
    let mut scriptname: *mut ::core::ffi::c_char = tofree;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ga_loaded.ga_len {
        if strcmp(
            (*(ga_loaded.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize))
                .offset(9 as ::core::ffi::c_int as isize),
            scriptname.offset(9 as ::core::ffi::c_int as isize),
        ) == 0 as ::core::ffi::c_int
        {
            break;
        }
        i += 1;
    }
    if !reload && i < ga_loaded.ga_len {
        ret = false_0 != 0;
    } else {
        if i == ga_loaded.ga_len {
            ga_grow(&raw mut ga_loaded, 1 as ::core::ffi::c_int);
            *(ga_loaded.ga_data as *mut *mut ::core::ffi::c_char)
                .offset(ga_loaded.ga_len as isize) = scriptname;
            ga_loaded.ga_len += 1;
            tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut ret_sid: ::core::ffi::c_int = 0;
        if do_in_runtimepath(
            scriptname,
            DIP_START as ::core::ffi::c_int,
            Some(
                source_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut ret_sid as *mut ::core::ffi::c_void,
        ) == OK
        {
            ret = true_0 != 0;
        }
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return ret;
}
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
