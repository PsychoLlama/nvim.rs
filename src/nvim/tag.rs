use crate::src::nvim::global_cell::GlobalCell;
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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fseek(
        __stream: *mut FILE,
        __off: ::core::ffi::c_long,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fseeko(
        __stream: *mut FILE,
        __off: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftello(__stream: *mut FILE) -> __off_t;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
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
    fn has_autocmd(event: event_T, sfname: *mut ::core::ffi::c_char, buf: *mut buf_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buflist_getfile(
        n: ::core::ffi::c_int,
        lnum: linenr_T,
        options: ::core::ffi::c_int,
        forceit: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn buflist_findname_exp(fname: *mut ::core::ffi::c_char) -> *mut buf_T;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn bt_help(buf: *const buf_T) -> bool;
    static mut p_enc: *mut ::core::ffi::c_char;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut fdo_flags: ::core::ffi::c_uint;
    static mut p_hf: *mut ::core::ffi::c_char;
    static mut p_hlg: *mut ::core::ffi::c_char;
    static mut p_ic: ::core::ffi::c_int;
    static mut jop_flags: ::core::ffi::c_uint;
    static mut p_sft: ::core::ffi::c_int;
    static mut p_scs: ::core::ffi::c_int;
    static mut swb_flags: ::core::ffi::c_uint;
    static mut p_tbs: ::core::ffi::c_int;
    static mut tc_flags: ::core::ffi::c_uint;
    static mut p_tl: OptInt;
    static mut p_tr: ::core::ffi::c_int;
    static mut p_tags: *mut ::core::ffi::c_char;
    static mut p_tgst: ::core::ffi::c_int;
    static mut p_verbose: OptInt;
    static mut p_ws: ::core::ffi::c_int;
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
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn ptr2cells(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_isblankline(lbuf: *mut ::core::ffi::c_char) -> bool;
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    fn check_cursor(wp: *mut win_T);
    static e_invarg: [::core::ffi::c_char; 0];
    static e_listreq: [::core::ffi::c_char; 0];
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
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
    fn list2fpos(
        arg: *mut typval_T,
        posp: *mut pos_T,
        fnump: *mut ::core::ffi::c_int,
        curswantp: *mut colnr_T,
        charcol: bool,
    ) -> ::core::ffi::c_int;
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
    static mut hash_removed: ::core::ffi::c_char;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn wait_return(redraw: ::core::ffi::c_int);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_outtrans_one(
        p: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> *const ::core::ffi::c_char;
    fn msg_outtrans_len(
        msgstr: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_clr_eos();
    fn verbose_enter();
    fn verbose_leave();
    fn give_warning(message: *const ::core::ffi::c_char, hl: bool, hist: bool);
    fn msg_advance(col: ::core::ffi::c_int);
    fn msg_delay(ms: uint64_t, ignoreinput: bool);
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_free(l: *mut list_T);
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn callback_free(callback: *mut Callback);
    fn callback_copy(dest: *mut Callback, src: *mut Callback);
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
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
    fn tv_dict_alloc_lock(lock: VarLockStatus) -> *mut dict_T;
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn getfile(
        fnum: ::core::ffi::c_int,
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        setpm: bool,
        lnum: linenr_T,
        forceit: bool,
    ) -> ::core::ffi::c_int;
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn set_no_hlsearch(flag: bool);
    fn vim_findfile_init(
        path: *mut ::core::ffi::c_char,
        filename: *mut ::core::ffi::c_char,
        filenamelen: size_t,
        stopdirs: *mut ::core::ffi::c_char,
        level: ::core::ffi::c_int,
        free_visited: ::core::ffi::c_int,
        find_what: ::core::ffi::c_int,
        search_ctx_arg: *mut ::core::ffi::c_void,
        tagfile: ::core::ffi::c_int,
        rel_fname: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn vim_findfile_stopdir(buf: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_findfile_cleanup(ctx: *mut ::core::ffi::c_void);
    fn vim_findfile(search_ctx_arg: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_char;
    fn vim_fgets(buf: *mut ::core::ffi::c_char, size: ::core::ffi::c_int, fp: *mut FILE) -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn foldOpenCursor();
    static mut Columns: ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut msg_didout: bool;
    static mut emsg_off: ::core::ffi::c_int;
    static mut curwin: *mut win_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut secure: ::core::ffi::c_int;
    static mut sandbox: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut KeyTyped: bool;
    static mut got_int: bool;
    static mut postponed_split: ::core::ffi::c_int;
    static mut postponed_split_flags: ::core::ffi::c_int;
    static mut g_do_tagpreview: ::core::ffi::c_int;
    static mut g_tag_at_cursor: bool;
    static mut keep_help_flag: bool;
    static mut no_hlsearch: bool;
    static mut vim_ignored: ::core::ffi::c_int;
    static mut magic_overruled: optmagic_T;
    fn help_heuristic(
        matched_string: *mut ::core::ffi::c_char,
        offset: ::core::ffi::c_int,
        wrong_case: bool,
    ) -> ::core::ffi::c_int;
    fn ins_compl_interrupted() -> bool;
    fn ins_compl_check_keys(frequency: ::core::ffi::c_int, in_compl_func: bool);
    fn prompt_for_input(
        prompt: *mut ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> ::core::ffi::c_int;
    fn setpcmark();
    fn mark_view_restore(fm: *mut fmark_T);
    fn mark_view_make(wp: *const win_T, pos: pos_T) -> fmarkv_T;
    fn fm_getname(fmark: *mut fmark_T, lead_len: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_strnicmp(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        nn: size_t,
    ) -> ::core::ffi::c_int;
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
    fn set_topline(wp: *mut win_T, lnum: linenr_T);
    fn validate_cursor(wp: *mut win_T);
    fn magic_isset() -> bool;
    fn option_set_callback_func(
        optval: *mut ::core::ffi::c_char,
        optcb: *mut Callback,
    ) -> ::core::ffi::c_int;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn free_string_option(p: *mut ::core::ffi::c_char);
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_breakcheck();
    fn line_breakcheck();
    fn fast_breakcheck();
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn path_has_wildcard(p: *const ::core::ffi::c_char) -> bool;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn simplify_filename(filename: *mut ::core::ffi::c_char) -> size_t;
    fn vim_isAbsName(name: *const ::core::ffi::c_char) -> bool;
    fn set_errorlist(
        wp: *mut win_T,
        list: *mut list_T,
        action: ::core::ffi::c_int,
        title: *mut ::core::ffi::c_char,
        what: *mut dict_T,
    ) -> ::core::ffi::c_int;
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
    fn do_in_runtimepath(
        name: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        callback: DoInRuntimepathCB,
        cookie: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn ignorecase(pat: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ignorecase_opt(
        pat: *mut ::core::ffi::c_char,
        ic_in: ::core::ffi::c_int,
        scs: ::core::ffi::c_int,
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
    fn ui_has(ext: UIExtension) -> bool;
    fn check_can_set_curbuf_forceit(forceit: ::core::ffi::c_int) -> bool;
    fn swbuf_goto_win_with_buf(buf: *mut buf_T) -> *mut win_T;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn win_enter(wp: *mut win_T, undo_sync: bool);
}
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
pub type off_t = __off_t;
pub type time_t = __time_t;
pub type off_T = off_t;
pub type ptrdiff_t = isize;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_16 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_16 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_16 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_16 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_16 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_16 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_16 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_16 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_16 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_16 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_16 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_16 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_16 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_16 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_16 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_16 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_16 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_16 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_16 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_16 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_16 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_16 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_16 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_16 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_16 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_16 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_16 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_16 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_16 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_16 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_16 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_16 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_16 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_16 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_16 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_16 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_16 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_16 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_16 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_16 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_16 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_16 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_16 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_16 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_16 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_16 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_16 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_16 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_16 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_16 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_16 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_16 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_16 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_16 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_16 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_16 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_16 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
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
    pub cs_pend: C2Rust_Unnamed_17,
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
pub union C2Rust_Unnamed_17 {
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
pub type getf_values = ::core::ffi::c_uint;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub type getf_retvalues = ::core::ffi::c_int;
pub const GETFILE_UNUSED: getf_retvalues = 8;
pub const GETFILE_OPEN_OTHER: getf_retvalues = -1;
pub const GETFILE_SAME_FILE: getf_retvalues = 0;
pub const GETFILE_NOT_WRITTEN: getf_retvalues = 2;
pub const GETFILE_ERROR: getf_retvalues = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_18 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_18 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_18 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_18 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_18 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_18 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_18 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_18 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_18 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_18 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptJopFlagClean: C2Rust_Unnamed_19 = 4;
pub const kOptJopFlagView: C2Rust_Unnamed_19 = 2;
pub const kOptJopFlagStack: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptSwbFlagUselast: C2Rust_Unnamed_20 = 32;
pub const kOptSwbFlagVsplit: C2Rust_Unnamed_20 = 16;
pub const kOptSwbFlagNewtab: C2Rust_Unnamed_20 = 8;
pub const kOptSwbFlagSplit: C2Rust_Unnamed_20 = 4;
pub const kOptSwbFlagUsetab: C2Rust_Unnamed_20 = 2;
pub const kOptSwbFlagUseopen: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const kOptTcFlagSmart: C2Rust_Unnamed_21 = 16;
pub const kOptTcFlagFollowscs: C2Rust_Unnamed_21 = 8;
pub const kOptTcFlagMatch: C2Rust_Unnamed_21 = 4;
pub const kOptTcFlagIgnore: C2Rust_Unnamed_21 = 2;
pub const kOptTcFlagFollowic: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_22 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_22 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_22 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_22 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_22 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_22 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_22 = 7;
pub const WILD_ALL: C2Rust_Unnamed_22 = 6;
pub const WILD_PREV: C2Rust_Unnamed_22 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_22 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_22 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_22 = 2;
pub const WILD_FREE: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_23 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_23 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_23 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_23 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_23 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_23 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_23 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_23 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_23 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_23 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_23 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_23 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_23 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_23 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_23 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_23 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_24 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_24 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_24 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_24 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_24 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_24 = 20;
pub const UPD_VALID: C2Rust_Unnamed_24 = 10;
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
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_25 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_25 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_25 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_25 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_25 = 1;
pub const CONV_NONE: C2Rust_Unnamed_25 = 0;
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
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const FINDFILE_BOTH: C2Rust_Unnamed_26 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_26 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_26 = 0;
pub type DoInRuntimepathCB = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut *mut ::core::ffi::c_char,
        bool,
        *mut ::core::ffi::c_void,
    ) -> bool,
>;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_27 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_27 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_27 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_27 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_27 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_27 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_27 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_27 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_27 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_27 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_27 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_27 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_27 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_27 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_27 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_27 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_27 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_27 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_27 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_28 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_28 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_28 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_28 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_28 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_28 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_28 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_28 = 1;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_29 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_29 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_29 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_29 = 32;
pub const DIP_OPT: C2Rust_Unnamed_29 = 16;
pub const DIP_START: C2Rust_Unnamed_29 = 8;
pub const DIP_ERR: C2Rust_Unnamed_29 = 4;
pub const DIP_DIR: C2Rust_Unnamed_29 = 2;
pub const DIP_ALL: C2Rust_Unnamed_29 = 1;
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
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_30 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_30 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_30 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_30 = 512;
pub const SEARCH_START: C2Rust_Unnamed_30 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_30 = 128;
pub const SEARCH_END: C2Rust_Unnamed_30 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_30 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_30 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_30 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_30 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_30 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_30 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchit_arg_T {
    pub sa_stop_lnum: linenr_T,
    pub sa_tm: *mut proftime_T,
    pub sa_timed_out: ::core::ffi::c_int,
    pub sa_wrapped: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const LSIZE: C2Rust_Unnamed_31 = 512;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const DT_FREE: C2Rust_Unnamed_32 = 99;
pub const DT_LTAG: C2Rust_Unnamed_32 = 11;
pub const DT_JUMP: C2Rust_Unnamed_32 = 9;
pub const DT_HELP: C2Rust_Unnamed_32 = 8;
pub const DT_SELECT: C2Rust_Unnamed_32 = 7;
pub const DT_LAST: C2Rust_Unnamed_32 = 6;
pub const DT_FIRST: C2Rust_Unnamed_32 = 5;
pub const DT_PREV: C2Rust_Unnamed_32 = 4;
pub const DT_NEXT: C2Rust_Unnamed_32 = 3;
pub const DT_POP: C2Rust_Unnamed_32 = 2;
pub const DT_TAG: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const TAG_MANY: C2Rust_Unnamed_33 = 300;
pub const TAG_NO_TAGFUNC: C2Rust_Unnamed_33 = 256;
pub const TAG_KEEP_LANG: C2Rust_Unnamed_33 = 128;
pub const TAG_INS_COMP: C2Rust_Unnamed_33 = 64;
pub const TAG_VERBOSE: C2Rust_Unnamed_33 = 32;
pub const TAG_NOIC: C2Rust_Unnamed_33 = 8;
pub const TAG_REGEXP: C2Rust_Unnamed_33 = 4;
pub const TAG_NAMES: C2Rust_Unnamed_33 = 2;
pub const TAG_HELP: C2Rust_Unnamed_33 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagname_T {
    pub tn_tags: *mut ::core::ffi::c_char,
    pub tn_np: *mut ::core::ffi::c_char,
    pub tn_did_filefind_init: ::core::ffi::c_int,
    pub tn_hf_idx: ::core::ffi::c_int,
    pub tn_search_ctx: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagptrs_T {
    pub tagname: *mut ::core::ffi::c_char,
    pub tagname_end: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub fname_end: *mut ::core::ffi::c_char,
    pub command: *mut ::core::ffi::c_char,
    pub command_end: *mut ::core::ffi::c_char,
    pub tag_fname: *mut ::core::ffi::c_char,
    pub tagkind: *mut ::core::ffi::c_char,
    pub tagkind_end: *mut ::core::ffi::c_char,
    pub user_data: *mut ::core::ffi::c_char,
    pub user_data_end: *mut ::core::ffi::c_char,
    pub tagline: linenr_T,
}
pub const WSP_VERT: C2Rust_Unnamed_34 = 2;
pub const MT_IC_OFF: C2Rust_Unnamed_35 = 4;
pub const MT_MASK: C2Rust_Unnamed_35 = 7;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct findtags_state_T {
    pub state: tagsearch_state_T,
    pub stop_searching: bool,
    pub orgpat: *mut pat_T,
    pub lbuf: *mut ::core::ffi::c_char,
    pub lbuf_size: ::core::ffi::c_int,
    pub tag_fname: *mut ::core::ffi::c_char,
    pub fp: *mut FILE,
    pub flags: ::core::ffi::c_int,
    pub tag_file_sorted: ::core::ffi::c_int,
    pub get_searchpat: bool,
    pub help_only: bool,
    pub did_open: bool,
    pub mincount: ::core::ffi::c_int,
    pub linear: bool,
    pub vimconv: vimconv_T,
    pub help_lang: [::core::ffi::c_char; 3],
    pub help_pri: ::core::ffi::c_int,
    pub help_lang_find: *mut ::core::ffi::c_char,
    pub is_txt: bool,
    pub match_count: ::core::ffi::c_int,
    pub ga_match: [garray_T; 16],
    pub ht_match: [hashtab_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pat_T {
    pub pat: *mut ::core::ffi::c_char,
    pub len: ::core::ffi::c_int,
    pub head: *mut ::core::ffi::c_char,
    pub headlen: ::core::ffi::c_int,
    pub regmatch: regmatch_T,
}
pub type tagsearch_state_T = ::core::ffi::c_uint;
pub const TS_STEP_FORWARD: tagsearch_state_T = 4;
pub const TS_SKIP_BACK: tagsearch_state_T = 3;
pub const TS_BINARY: tagsearch_state_T = 2;
pub const TS_LINEAR: tagsearch_state_T = 1;
pub const TS_START: tagsearch_state_T = 0;
pub const MT_COUNT: C2Rust_Unnamed_35 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct findtags_match_args_T {
    pub matchoff: ::core::ffi::c_int,
    pub match_re: bool,
    pub match_no_ic: bool,
    pub has_re: bool,
    pub sortic: bool,
    pub sort_error: bool,
}
pub const MT_GL_OTH: C2Rust_Unnamed_35 = 2;
pub const MT_GL_CUR: C2Rust_Unnamed_35 = 1;
pub const MT_ST_OTH: C2Rust_Unnamed_35 = 3;
pub const MT_ST_CUR: C2Rust_Unnamed_35 = 0;
pub const MT_RE_OFF: C2Rust_Unnamed_35 = 8;
pub const TAG_MATCH_FAIL: tagmatch_status_T = 2;
pub type tags_read_status_T = ::core::ffi::c_uint;
pub const TAGS_READ_IGNORE: tags_read_status_T = 3;
pub const TAGS_READ_EOF: tags_read_status_T = 2;
pub const TAGS_READ_SUCCESS: tags_read_status_T = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagsearch_info_T {
    pub low_offset: off_T,
    pub high_offset: off_T,
    pub curr_offset: off_T,
    pub curr_offset_used: off_T,
    pub match_offset: off_T,
    pub low_char: ::core::ffi::c_int,
    pub high_char: ::core::ffi::c_int,
}
pub const TAG_MATCH_STOP: tagmatch_status_T = 3;
pub const TAG_MATCH_NEXT: tagmatch_status_T = 4;
pub type tagmatch_status_T = ::core::ffi::c_uint;
pub const TAG_MATCH_SUCCESS: tagmatch_status_T = 1;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_34 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_34 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_34 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_34 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_34 = 64;
pub const WSP_HELP: C2Rust_Unnamed_34 = 32;
pub const WSP_BOT: C2Rust_Unnamed_34 = 16;
pub const WSP_TOP: C2Rust_Unnamed_34 = 8;
pub const WSP_HOR: C2Rust_Unnamed_34 = 4;
pub const WSP_ROOM: C2Rust_Unnamed_34 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const CMDBUFFSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
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
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TAGSTACKSIZE: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_TAGPAT: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static mt_names: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"FSC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"F C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"F  \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"FS \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b" SC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"  C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"   \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b" S \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
pub const NOTAGFILE: ::core::ffi::c_int = 99 as ::core::ffi::c_int;
static nofile_fname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static e_tag_stack_empty: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E73: Tag stack empty\0")
});
static e_tag_not_found_str: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E426: Tag not found: %s\0")
});
static e_at_bottom_of_tag_stack: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E555: At bottom of tag stack\0",
    )
});
static e_at_top_of_tag_stack: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E556: At top of tag stack\0")
});
static e_cannot_modify_tag_stack_within_tagfunc: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E986: Cannot modify the tag stack within tagfunc\0",
        )
    });
static e_invalid_return_value_from_tagfunc: GlobalCell<[::core::ffi::c_char; 40]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
            *b"E987: Invalid return value from tagfunc\0",
        )
    });
static e_window_unexpectedly_close_while_searching_for_tags: GlobalCell<[::core::ffi::c_char; 59]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 59], [::core::ffi::c_char; 59]>(
            *b"E1299: Window unexpectedly closed while searching for tags\0",
        )
    });
static tagmatchname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static ptag_entry: GlobalCell<taggy_T> = GlobalCell::new(taggy_T {
    tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    fmark: fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    cur_match: 0 as ::core::ffi::c_int,
    cur_fnum: 0 as ::core::ffi::c_int,
    user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
});
static tfu_in_use: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static tfu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_5 {
        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
    type_0: kCallbackNone,
});
pub const TAG_SEP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn did_set_tagfunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: ::core::ffi::c_int = 0;
    if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
        retval = option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_tfu_cb);
    } else {
        retval = option_set_callback_func((*args).os_newval.string.data, tfu_cb.ptr());
        if retval == OK && (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
            set_buflocal_tfu_callback(buf);
        }
    }
    return if retval == FAIL {
        &raw const e_invarg as *const ::core::ffi::c_char
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_ref_in_tagfunc(mut copyID: ::core::ffi::c_int) -> bool {
    return set_ref_in_callback(
        tfu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn set_buflocal_tfu_callback(mut buf: *mut buf_T) {
    callback_free(&raw mut (*buf).b_tfu_cb);
    if (*tfu_cb.ptr()).type_0 as ::core::ffi::c_uint
        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        callback_copy(&raw mut (*buf).b_tfu_cb, tfu_cb.ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_tag(
    mut tag: *mut ::core::ffi::c_char,
    mut type_0: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut verbose: bool,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin).w_tagstackidx;
    let mut tagstacklen: ::core::ffi::c_int = (*curwin).w_tagstacklen;
    let mut cur_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur_fnum: ::core::ffi::c_int = (*curbuf).handle as ::core::ffi::c_int;
    let mut oldtagstackidx: ::core::ffi::c_int = tagstackidx;
    let mut prevtagstackidx: ::core::ffi::c_int = tagstackidx;
    let mut new_tag: bool = false_0 != 0;
    let mut no_regexp: bool = false_0 != 0;
    let mut error_cur_match: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_pos: bool = false_0 != 0;
    let mut saved_fmark: fmark_T = fmark_T {
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
    };
    let mut new_num_matches: ::core::ffi::c_int = 0;
    let mut new_matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut use_tagstack: bool = false;
    let mut skip_msg: bool = false_0 != 0;
    let mut buf_ffname: *mut ::core::ffi::c_char = (*curbuf).b_ffname;
    let mut use_tfu: bool = true_0 != 0;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static num_matches: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static max_num_matches: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static matches: GlobalCell<*mut *mut ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
    static flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    if tfu_in_use.get() {
        emsg(gettext(
            (e_cannot_modify_tag_stack_within_tagfunc.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    if postponed_split == 0 as ::core::ffi::c_int && !check_can_set_curbuf_forceit(forceit) {
        return;
    }
    if type_0 == DT_HELP as ::core::ffi::c_int {
        type_0 = DT_TAG as ::core::ffi::c_int;
        no_regexp = true_0 != 0;
        use_tfu = false_0 != 0;
    }
    let mut prev_num_matches: ::core::ffi::c_int = num_matches.get();
    free_string_option(nofile_fname.get());
    nofile_fname.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    clearpos(&raw mut saved_fmark.mark);
    saved_fmark.fnum = 0 as ::core::ffi::c_int;
    saved_fmark.view = fmarkv_T {
        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
        skipcol: 0 as colnr_T,
    };
    '_c2rust_label: {
        if !tag.is_null() {
        } else {
            __assert_fail(
                b"tag != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tag.rs\0".as_ptr() as *const ::core::ffi::c_char,
                349 as ::core::ffi::c_uint,
                b"void do_tag(char *, int, int, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_end_do_tag: {
        if p_tgst == 0 && *tag as ::core::ffi::c_int != NUL {
            use_tagstack = false_0 != 0;
            new_tag = true_0 != 0;
            if g_do_tagpreview != 0 as ::core::ffi::c_int {
                tagstack_clear_entry(ptag_entry.ptr());
                (*ptag_entry.ptr()).tagname = xstrdup(tag);
            }
        } else {
            if g_do_tagpreview != 0 as ::core::ffi::c_int {
                use_tagstack = false_0 != 0;
            } else {
                use_tagstack = true_0 != 0;
            }
            if *tag as ::core::ffi::c_int != NUL
                && (type_0 == DT_TAG as ::core::ffi::c_int
                    || type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                    || type_0 == DT_LTAG as ::core::ffi::c_int)
            {
                if g_do_tagpreview != 0 as ::core::ffi::c_int {
                    if !(*ptag_entry.ptr()).tagname.is_null()
                        && strcmp((*ptag_entry.ptr()).tagname, tag) == 0 as ::core::ffi::c_int
                    {
                        cur_match = (*ptag_entry.ptr()).cur_match;
                        cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                    } else {
                        tagstack_clear_entry(ptag_entry.ptr());
                        (*ptag_entry.ptr()).tagname = xstrdup(tag);
                    }
                } else {
                    while tagstackidx < tagstacklen {
                        tagstacklen -= 1;
                        tagstack_clear_entry(tagstack.offset(tagstacklen as isize));
                    }
                    tagstacklen += 1;
                    if tagstacklen > TAGSTACKSIZE {
                        tagstacklen = TAGSTACKSIZE;
                        tagstack_clear_entry(tagstack.offset(0 as ::core::ffi::c_int as isize));
                        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        while i < tagstacklen {
                            *tagstack.offset((i - 1 as ::core::ffi::c_int) as isize) =
                                *tagstack.offset(i as isize);
                            i += 1;
                        }
                        tagstackidx -= 1;
                        let c2rust_lvalue_ptr =
                            &raw mut (*tagstack.offset(tagstackidx as isize)).user_data;
                        *c2rust_lvalue_ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    (*tagstack.offset(tagstackidx as isize)).tagname = xstrdup(tag);
                    (*curwin).w_tagstacklen = tagstacklen;
                    save_pos = true_0 != 0;
                }
                new_tag = true_0 != 0;
            } else if if g_do_tagpreview != 0 as ::core::ffi::c_int {
                (*ptag_entry.ptr()).tagname.is_null() as ::core::ffi::c_int
            } else {
                (tagstacklen == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            } != 0
            {
                emsg(gettext(
                    (e_tag_stack_empty.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                break '_end_do_tag;
            } else if type_0 == DT_POP as ::core::ffi::c_int {
                let old_KeyTyped: bool = KeyTyped;
                tagstackidx -= count;
                if tagstackidx < 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        (e_at_bottom_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    if tagstackidx + count == 0 as ::core::ffi::c_int {
                        tagstackidx = 0 as ::core::ffi::c_int;
                        break '_end_do_tag;
                    } else {
                        tagstackidx = 0 as ::core::ffi::c_int;
                    }
                } else if tagstackidx >= tagstacklen {
                    emsg(gettext(
                        (e_at_top_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    break '_end_do_tag;
                }
                saved_fmark = (*tagstack.offset(tagstackidx as isize)).fmark;
                if saved_fmark.fnum != (*curbuf).handle {
                    if buflist_getfile(
                        saved_fmark.fnum,
                        saved_fmark.mark.lnum,
                        GETF_SETMARK as ::core::ffi::c_int,
                        forceit,
                    ) == FAIL
                    {
                        tagstackidx = oldtagstackidx;
                        break '_end_do_tag;
                    } else {
                        (*curwin).w_cursor.lnum = saved_fmark.mark.lnum;
                    }
                } else {
                    setpcmark();
                    (*curwin).w_cursor.lnum = saved_fmark.mark.lnum;
                }
                (*curwin).w_cursor.col = saved_fmark.mark.col;
                (*curwin).w_set_curswant = true_0;
                if jop_flags & kOptJopFlagView as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
                    mark_view_restore(&raw mut saved_fmark);
                }
                check_cursor(curwin);
                if fdo_flags & kOptFdoFlagTag as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                    && old_KeyTyped as ::core::ffi::c_int != 0
                {
                    foldOpenCursor();
                }
                FreeWild(num_matches.get(), matches.get());
                num_matches.set(0 as ::core::ffi::c_int);
                tag_freematch();
                break '_end_do_tag;
            } else if type_0 == DT_TAG as ::core::ffi::c_int
                || type_0 == DT_LTAG as ::core::ffi::c_int
            {
                if g_do_tagpreview != 0 as ::core::ffi::c_int {
                    cur_match = (*ptag_entry.ptr()).cur_match;
                    cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                } else {
                    save_pos = true_0 != 0;
                    tagstackidx += count - 1 as ::core::ffi::c_int;
                    if tagstackidx >= tagstacklen {
                        tagstackidx = tagstacklen - 1 as ::core::ffi::c_int;
                        emsg(gettext(
                            (e_at_top_of_tag_stack.ptr() as *const _) as *const ::core::ffi::c_char,
                        ));
                        save_pos = false_0 != 0;
                    } else if tagstackidx < 0 as ::core::ffi::c_int {
                        emsg(gettext(
                            (e_at_bottom_of_tag_stack.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        tagstackidx = 0 as ::core::ffi::c_int;
                        break '_end_do_tag;
                    }
                    cur_match = (*tagstack.offset(tagstackidx as isize)).cur_match;
                    cur_fnum = (*tagstack.offset(tagstackidx as isize)).cur_fnum;
                }
                new_tag = true_0 != 0;
            } else {
                prevtagstackidx = tagstackidx;
                if g_do_tagpreview != 0 as ::core::ffi::c_int {
                    cur_match = (*ptag_entry.ptr()).cur_match;
                    cur_fnum = (*ptag_entry.ptr()).cur_fnum;
                } else {
                    tagstackidx -= 1;
                    if tagstackidx < 0 as ::core::ffi::c_int {
                        tagstackidx = 0 as ::core::ffi::c_int;
                    }
                    cur_match = (*tagstack.offset(tagstackidx as isize)).cur_match;
                    cur_fnum = (*tagstack.offset(tagstackidx as isize)).cur_fnum;
                }
                match type_0 {
                    5 => {
                        cur_match = count - 1 as ::core::ffi::c_int;
                    }
                    7 | 9 | 6 => {
                        cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                    }
                    3 => {
                        cur_match += count;
                    }
                    4 => {
                        cur_match -= count;
                    }
                    _ => {}
                }
                if cur_match >= MAXCOL as ::core::ffi::c_int {
                    cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                } else if cur_match < 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        b"E425: Cannot go before first matching tag\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    skip_msg = true_0 != 0;
                    cur_match = 0 as ::core::ffi::c_int;
                    cur_fnum = (*curbuf).handle as ::core::ffi::c_int;
                }
            }
            if g_do_tagpreview != 0 as ::core::ffi::c_int {
                if type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                {
                    (*ptag_entry.ptr()).cur_match = cur_match;
                    (*ptag_entry.ptr()).cur_fnum = cur_fnum;
                }
            } else {
                saved_fmark = (*tagstack.offset(tagstackidx as isize)).fmark;
                if save_pos {
                    (*tagstack.offset(tagstackidx as isize)).fmark.mark = (*curwin).w_cursor;
                    (*tagstack.offset(tagstackidx as isize)).fmark.fnum =
                        (*curbuf).handle as ::core::ffi::c_int;
                    (*tagstack.offset(tagstackidx as isize)).fmark.view =
                        mark_view_make(curwin, (*curwin).w_cursor);
                }
                (*curwin).w_tagstackidx = tagstackidx;
                if type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                {
                    (*curwin).w_tagstack[tagstackidx as usize].cur_match = cur_match;
                    (*curwin).w_tagstack[tagstackidx as usize].cur_fnum = cur_fnum;
                }
            }
        }
        if cur_fnum != (*curbuf).handle {
            let mut buf: *mut buf_T = buflist_findnr(cur_fnum);
            if !buf.is_null() {
                buf_ffname = (*buf).b_ffname;
            }
        }
        loop {
            let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if use_tagstack {
                name = xstrdup((*tagstack.offset(tagstackidx as isize)).tagname);
                xfree(tofree as *mut ::core::ffi::c_void);
                tofree = name;
            } else if g_do_tagpreview != 0 as ::core::ffi::c_int {
                name = (*ptag_entry.ptr()).tagname;
            } else {
                name = tag;
            }
            let mut other_name: bool = (*tagmatchname.ptr()).is_null()
                || strcmp(tagmatchname.get(), name) != 0 as ::core::ffi::c_int;
            if new_tag as ::core::ffi::c_int != 0
                || cur_match >= num_matches.get()
                    && max_num_matches.get() != MAXCOL as ::core::ffi::c_int
                || other_name as ::core::ffi::c_int != 0
            {
                if other_name {
                    xfree(tagmatchname.get() as *mut ::core::ffi::c_void);
                    tagmatchname.set(xstrdup(name));
                }
                if type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                    || type_0 == DT_LTAG as ::core::ffi::c_int
                {
                    cur_match = MAXCOL as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                }
                max_num_matches.set(if type_0 == DT_TAG as ::core::ffi::c_int {
                    MAXCOL as ::core::ffi::c_int
                } else {
                    cur_match + 1 as ::core::ffi::c_int
                });
                if !no_regexp && *name as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                    flags.set(TAG_REGEXP as ::core::ffi::c_int);
                    name = name.offset(1);
                } else {
                    flags.set(TAG_NOIC as ::core::ffi::c_int);
                }
                (*flags.ptr()) |= if verbose as ::core::ffi::c_int != 0 {
                    TAG_VERBOSE as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                (*flags.ptr()) |= if !use_tfu {
                    TAG_NO_TAGFUNC as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if find_tags(
                    name,
                    &raw mut new_num_matches,
                    &raw mut new_matches,
                    flags.get(),
                    max_num_matches.get(),
                    buf_ffname,
                ) == OK
                    && new_num_matches < max_num_matches.get()
                {
                    max_num_matches.set(MAXCOL as ::core::ffi::c_int);
                }
                if tagstack != &raw mut (*curwin).w_tagstack as *mut taggy_T {
                    emsg(gettext(
                        (e_window_unexpectedly_close_while_searching_for_tags.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    FreeWild(new_num_matches, new_matches);
                    break '_end_do_tag;
                } else {
                    if !new_tag && !other_name {
                        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut tagp: tagptrs_T = tagptrs_T {
                            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagline: 0,
                        };
                        let mut tagp2: tagptrs_T = tagptrs_T {
                            tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            tagline: 0,
                        };
                        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while j < num_matches.get() {
                            parse_match(*(*matches.ptr()).offset(j as isize), &raw mut tagp);
                            let mut i_0: ::core::ffi::c_int = idx;
                            while i_0 < new_num_matches {
                                parse_match(*new_matches.offset(i_0 as isize), &raw mut tagp2);
                                if strcmp(tagp.tagname, tagp2.tagname) == 0 as ::core::ffi::c_int {
                                    let mut p: *mut ::core::ffi::c_char =
                                        *new_matches.offset(i_0 as isize);
                                    let mut k: ::core::ffi::c_int = i_0;
                                    while k > idx {
                                        *new_matches.offset(k as isize) = *new_matches
                                            .offset((k - 1 as ::core::ffi::c_int) as isize);
                                        k -= 1;
                                    }
                                    let c2rust_fresh0 = idx;
                                    idx = idx + 1;
                                    let c2rust_lvalue_ptr_0 =
                                        &raw mut *new_matches.offset(c2rust_fresh0 as isize);
                                    *c2rust_lvalue_ptr_0 = p;
                                    break;
                                } else {
                                    i_0 += 1;
                                }
                            }
                            j += 1;
                        }
                    }
                    FreeWild(num_matches.get(), matches.get());
                    num_matches.set(new_num_matches);
                    matches.set(new_matches);
                }
            }
            if num_matches.get() <= 0 as ::core::ffi::c_int {
                if verbose {
                    semsg(
                        gettext(
                            (e_tag_not_found_str.ptr() as *const _) as *const ::core::ffi::c_char,
                        ),
                        name,
                    );
                }
                g_do_tagpreview = 0 as ::core::ffi::c_int;
                break '_end_do_tag;
            } else {
                let mut ask_for_selection: bool = false_0 != 0;
                if type_0 == DT_TAG as ::core::ffi::c_int && *tag as ::core::ffi::c_int != NUL {
                    cur_match = if count > 0 as ::core::ffi::c_int {
                        count - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                } else if type_0 == DT_SELECT as ::core::ffi::c_int
                    || type_0 == DT_JUMP as ::core::ffi::c_int
                        && num_matches.get() > 1 as ::core::ffi::c_int
                {
                    print_tag_list(new_tag, use_tagstack, num_matches.get(), matches.get());
                    ask_for_selection = true_0 != 0;
                } else if type_0 == DT_LTAG as ::core::ffi::c_int {
                    if add_llist_tags(tag, num_matches.get(), matches.get()) == FAIL {
                        break '_end_do_tag;
                    }
                    cur_match = 0 as ::core::ffi::c_int;
                }
                if ask_for_selection {
                    let mut i_1: ::core::ffi::c_int = prompt_for_input(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if i_1 <= 0 as ::core::ffi::c_int
                        || i_1 > num_matches.get()
                        || got_int as ::core::ffi::c_int != 0
                    {
                        if use_tagstack {
                            (*tagstack.offset(tagstackidx as isize)).fmark = saved_fmark;
                            tagstackidx = prevtagstackidx;
                        }
                        break '_end_do_tag;
                    } else {
                        cur_match = i_1 - 1 as ::core::ffi::c_int;
                    }
                }
                if cur_match >= num_matches.get() {
                    if (type_0 == DT_NEXT as ::core::ffi::c_int
                        || type_0 == DT_FIRST as ::core::ffi::c_int)
                        && (*nofile_fname.ptr()).is_null()
                    {
                        if num_matches.get() == 1 as ::core::ffi::c_int {
                            emsg(gettext(b"E427: There is only one matching tag\0".as_ptr()
                                as *const ::core::ffi::c_char));
                        } else {
                            emsg(gettext(
                                b"E428: Cannot go beyond last matching tag\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        skip_msg = true_0 != 0;
                    }
                    cur_match = num_matches.get() - 1 as ::core::ffi::c_int;
                }
                if use_tagstack {
                    let mut tagp2_0: tagptrs_T = tagptrs_T {
                        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        tagline: 0,
                    };
                    (*tagstack.offset(tagstackidx as isize)).cur_match = cur_match;
                    (*tagstack.offset(tagstackidx as isize)).cur_fnum = cur_fnum;
                    if use_tfu as ::core::ffi::c_int != 0
                        && parse_match(
                            *(*matches.ptr()).offset(cur_match as isize),
                            &raw mut tagp2_0,
                        ) == OK
                        && !tagp2_0.user_data.is_null()
                    {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut (*tagstack.offset(tagstackidx as isize)).user_data
                                as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        *ptr_;
                        (*tagstack.offset(tagstackidx as isize)).user_data = xmemdupz(
                            tagp2_0.user_data as *const ::core::ffi::c_void,
                            tagp2_0.user_data_end.offset_from(tagp2_0.user_data) as size_t,
                        )
                            as *mut ::core::ffi::c_char;
                    }
                    tagstackidx += 1;
                } else if g_do_tagpreview != 0 as ::core::ffi::c_int {
                    (*ptag_entry.ptr()).cur_match = cur_match;
                    (*ptag_entry.ptr()).cur_fnum = cur_fnum;
                }
                if !(*nofile_fname.ptr()).is_null() && error_cur_match != cur_match {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"File \"%s\" does not exist\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        nofile_fname.get(),
                    );
                }
                let mut ic: bool = *(*(*matches.ptr()).offset(cur_match as isize))
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & MT_IC_OFF as ::core::ffi::c_int
                    != 0;
                if type_0 != DT_TAG as ::core::ffi::c_int
                    && type_0 != DT_SELECT as ::core::ffi::c_int
                    && type_0 != DT_JUMP as ::core::ffi::c_int
                    && (num_matches.get() > 1 as ::core::ffi::c_int
                        || ic as ::core::ffi::c_int != 0)
                    && !skip_msg
                {
                    snprintf(
                        &raw mut IObuff as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
                        gettext(b"tag %d of %d%s\0".as_ptr() as *const ::core::ffi::c_char),
                        cur_match + 1 as ::core::ffi::c_int,
                        num_matches.get(),
                        if max_num_matches.get() != MAXCOL as ::core::ffi::c_int {
                            gettext(b" or more\0".as_ptr() as *const ::core::ffi::c_char)
                                as *const ::core::ffi::c_char
                        } else {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    if ic {
                        xstrlcat(
                            &raw mut IObuff as *mut ::core::ffi::c_char,
                            gettext(b"  Using tag with different case!\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            IOSIZE as size_t,
                        );
                    }
                    if (num_matches.get() > prev_num_matches || new_tag as ::core::ffi::c_int != 0)
                        && num_matches.get() > 1 as ::core::ffi::c_int
                    {
                        msg(
                            &raw mut IObuff as *mut ::core::ffi::c_char,
                            if ic as ::core::ffi::c_int != 0 {
                                HLF_W as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            },
                        );
                        msg_scroll = true_0;
                    } else {
                        give_warning(&raw mut IObuff as *mut ::core::ffi::c_char, ic, true_0 != 0);
                    }
                    if ic as ::core::ffi::c_int != 0
                        && msg_scrolled == 0
                        && msg_silent == 0 as ::core::ffi::c_int
                    {
                        msg_delay(1007 as uint64_t, true_0 != 0);
                    }
                }
                let mut IObufflen: size_t = vim_snprintf_safelen(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b":ta %s\r\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                set_vim_var_string(
                    VV_SWAPCOMMAND,
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    IObufflen as ptrdiff_t,
                );
                let mut i_2: ::core::ffi::c_int = jumpto_tag(
                    *(*matches.ptr()).offset(cur_match as isize),
                    forceit,
                    true_0 != 0,
                );
                set_vim_var_string(
                    VV_SWAPCOMMAND,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    -1 as ptrdiff_t,
                );
                if i_2 == NOTAGFILE {
                    if type_0 == DT_PREV as ::core::ffi::c_int
                        && cur_match > 0 as ::core::ffi::c_int
                        || (type_0 == DT_TAG as ::core::ffi::c_int
                            || type_0 == DT_NEXT as ::core::ffi::c_int
                            || type_0 == DT_FIRST as ::core::ffi::c_int)
                            && (max_num_matches.get() != MAXCOL as ::core::ffi::c_int
                                || cur_match < num_matches.get() - 1 as ::core::ffi::c_int)
                    {
                        error_cur_match = cur_match;
                        if use_tagstack {
                            tagstackidx -= 1;
                        }
                        if type_0 == DT_PREV as ::core::ffi::c_int {
                            cur_match -= 1;
                        } else {
                            type_0 = DT_NEXT as ::core::ffi::c_int;
                            cur_match += 1;
                        }
                    } else {
                        semsg(
                            gettext(b"E429: File \"%s\" does not exist\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            nofile_fname.get(),
                        );
                        break '_end_do_tag;
                    }
                } else {
                    if use_tagstack as ::core::ffi::c_int != 0
                        && tagstackidx > (*curwin).w_tagstacklen
                    {
                        tagstackidx = (*curwin).w_tagstackidx;
                    }
                    break '_end_do_tag;
                }
            }
        }
    }
    if use_tagstack as ::core::ffi::c_int != 0 && tagstackidx <= (*curwin).w_tagstacklen {
        (*curwin).w_tagstackidx = tagstackidx;
    }
    postponed_split = 0 as ::core::ffi::c_int;
    g_do_tagpreview = 0 as ::core::ffi::c_int;
    xfree(tofree as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn print_tag_list(
    mut new_tag: bool,
    mut use_tagstack: bool,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin).w_tagstackidx;
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    parse_match(
        *matches.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tagp,
    );
    let mut taglen: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
        + 2 as isize) as ::core::ffi::c_int
        > 18 as ::core::ffi::c_int
    {
        (tagp.tagname_end.offset_from(tagp.tagname) + 2 as isize) as ::core::ffi::c_int
    } else {
        18 as ::core::ffi::c_int
    };
    if taglen > Columns - 25 as ::core::ffi::c_int {
        taglen = MAXCOL as ::core::ffi::c_int;
    }
    if msg_col == 0 as ::core::ffi::c_int {
        msg_didout = false_0 != 0;
    }
    msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    msg_puts_hl(
        gettext(b"  # pri kind tag\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_T as ::core::ffi::c_int,
        false_0 != 0,
    );
    msg_clr_eos();
    taglen_advance(taglen);
    msg_puts_hl(
        gettext(b"file\n\0".as_ptr() as *const ::core::ffi::c_char),
        HLF_T as ::core::ffi::c_int,
        false_0 != 0,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches && !got_int {
        parse_match(*matches.offset(i as isize), &raw mut tagp);
        if !new_tag
            && (g_do_tagpreview != 0 as ::core::ffi::c_int && i == (*ptag_entry.ptr()).cur_match
                || use_tagstack as ::core::ffi::c_int != 0
                    && i == (*tagstack.offset(tagstackidx as isize)).cur_match)
        {
            *(&raw mut IObuff as *mut ::core::ffi::c_char) = '>' as ::core::ffi::c_char;
        } else {
            *(&raw mut IObuff as *mut ::core::ffi::c_char) = ' ' as ::core::ffi::c_char;
        }
        vim_snprintf(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
            (IOSIZE - 1 as ::core::ffi::c_int) as size_t,
            b"%2d %s \0".as_ptr() as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
            (*mt_names.ptr())[(*(*matches.offset(i as isize))
                .offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & MT_MASK as ::core::ffi::c_int) as usize],
        );
        msg_puts(&raw mut IObuff as *mut ::core::ffi::c_char);
        if !tagp.tagkind.is_null() {
            msg_outtrans_len(
                tagp.tagkind,
                tagp.tagkind_end.offset_from(tagp.tagkind) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        msg_advance(13 as ::core::ffi::c_int);
        msg_outtrans_len(
            tagp.tagname,
            tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int,
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        msg_putchar(' ' as ::core::ffi::c_int);
        taglen_advance(taglen);
        let mut p: *const ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
        if !p.is_null() {
            msg_outtrans(p, HLF_D as ::core::ffi::c_int, false_0 != 0);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut p as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            *ptr_;
        }
        if msg_col > 0 as ::core::ffi::c_int {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if got_int {
            break;
        }
        msg_advance(15 as ::core::ffi::c_int);
        let mut command_end: *const ::core::ffi::c_char = tagp.command_end;
        if !command_end.is_null() {
            p = command_end.offset(3 as ::core::ffi::c_int as isize);
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                while *p as ::core::ffi::c_int == TAB {
                    p = p.offset(1);
                }
                if strncmp(
                    p,
                    b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && ascii_isspace(
                        *p.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                {
                    p = p.offset(5 as ::core::ffi::c_int as isize);
                } else if p == tagp.tagkind as *const ::core::ffi::c_char
                    || p.offset(5 as ::core::ffi::c_int as isize)
                        == tagp.tagkind as *const ::core::ffi::c_char
                        && strncmp(
                            p,
                            b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                {
                    p = tagp.tagkind_end;
                } else {
                    let mut hl_id: ::core::ffi::c_int = HLF_CM as ::core::ffi::c_int;
                    while *p as ::core::ffi::c_int != 0
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        if msg_col + ptr2cells(p) >= Columns {
                            msg_putchar('\n' as ::core::ffi::c_int);
                            if got_int {
                                break;
                            }
                            msg_advance(15 as ::core::ffi::c_int);
                        }
                        p = msg_outtrans_one(p, hl_id, false_0 != 0);
                        if *p as ::core::ffi::c_int == TAB {
                            msg_puts_hl(
                                b" \0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            break;
                        } else if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
                            hl_id = 0 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            if msg_col > 15 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int {
                    break;
                }
                msg_advance(15 as ::core::ffi::c_int);
            }
        } else {
            p = tagp.command;
            while *p as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            command_end = p;
        }
        p = tagp.command;
        if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int
        {
            p = p.offset(1);
            if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
        while p != command_end && ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            p = p.offset(1);
        }
        while p != command_end {
            if msg_col
                + (if *p as ::core::ffi::c_int == TAB {
                    1 as ::core::ffi::c_int
                } else {
                    ptr2cells(p)
                })
                > Columns
            {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            if got_int {
                break;
            }
            msg_advance(15 as ::core::ffi::c_int);
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *tagp.command as ::core::ffi::c_int
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == TAB {
                msg_putchar(' ' as ::core::ffi::c_int);
                p = p.offset(1);
            } else {
                p = msg_outtrans_one(p, 0 as ::core::ffi::c_int, false_0 != 0);
            }
            if p == command_end.offset(-(2 as ::core::ffi::c_int as isize))
                && *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *tagp.command as ::core::ffi::c_int
            {
                break;
            }
            if p == command_end.offset(-(1 as ::core::ffi::c_int as isize))
                && *p as ::core::ffi::c_int == *tagp.command as ::core::ffi::c_int
                && (*p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '?' as ::core::ffi::c_int)
            {
                break;
            }
        }
        if msg_col != 0 && (!ui_has(kUIMessages) || i < num_matches - 1 as ::core::ffi::c_int) {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        os_breakcheck();
        i += 1;
    }
    if got_int {
        got_int = false_0 != 0;
    }
}
unsafe extern "C" fn add_llist_tags(
    mut tag: *mut ::core::ffi::c_char,
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tag_name: [::core::ffi::c_char; 129] = [0; 129];
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut fname: *mut ::core::ffi::c_char =
        xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    let mut cmd: *mut ::core::ffi::c_char =
        xmalloc((CMDBUFFSIZE + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    let mut list: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches {
        let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        parse_match(*matches.offset(i as isize), &raw mut tagp);
        let mut len: ::core::ffi::c_int = if (tagp.tagname_end.offset_from(tagp.tagname)
            as ::core::ffi::c_int)
            < 128 as ::core::ffi::c_int
        {
            tagp.tagname_end.offset_from(tagp.tagname) as ::core::ffi::c_int
        } else {
            128 as ::core::ffi::c_int
        };
        xmemcpyz(
            &raw mut tag_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            tagp.tagname as *const ::core::ffi::c_void,
            len as size_t,
        );
        tag_name[len as usize] = NUL as ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = tag_full_fname(&raw mut tagp);
        if !p.is_null() {
            xstrlcpy(fname, p, MAXPATHL as size_t);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut p as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            *ptr_;
            let mut lnum: linenr_T = 0 as linenr_T;
            if *(*__ctype_b_loc()).offset(*tagp.command as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                lnum = atoi(tagp.command) as linenr_T;
            } else {
                let mut cmd_start: *mut ::core::ffi::c_char = tagp.command;
                let mut cmd_end: *mut ::core::ffi::c_char = tagp.command_end;
                if cmd_end.is_null() {
                    p = tagp.command;
                    while *p as ::core::ffi::c_int != 0
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    cmd_end = p;
                }
                cmd_end = cmd_end.offset(-1);
                if *cmd_start as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *cmd_start as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                {
                    cmd_start = cmd_start.offset(1);
                }
                if *cmd_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                    || *cmd_end as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                {
                    cmd_end = cmd_end.offset(-1);
                }
                len = 0 as ::core::ffi::c_int;
                *cmd.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                if *cmd_start as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                    strcpy(
                        cmd,
                        b"^\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    cmd_start = cmd_start.offset(1);
                    len += 1;
                }
                strcat(cmd, b"\\V\0".as_ptr() as *const ::core::ffi::c_char);
                len += 2 as ::core::ffi::c_int;
                let mut cmd_len: ::core::ffi::c_int =
                    if ((cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int)
                        < 1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                    {
                        (cmd_end.offset_from(cmd_start) + 1 as isize) as ::core::ffi::c_int
                    } else {
                        1024 as ::core::ffi::c_int - 5 as ::core::ffi::c_int
                    };
                snprintf(
                    cmd.offset(len as isize),
                    (CMDBUFFSIZE + 1 as ::core::ffi::c_int - len) as size_t,
                    b"%.*s\0".as_ptr() as *const ::core::ffi::c_char,
                    cmd_len,
                    cmd_start,
                );
                len += cmd_len;
                if *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    == '$' as ::core::ffi::c_int
                {
                    *cmd.offset((len - 1 as ::core::ffi::c_int) as isize) =
                        '\\' as ::core::ffi::c_char;
                    *cmd.offset(len as isize) = '$' as ::core::ffi::c_char;
                    len += 1;
                }
                *cmd.offset(len as isize) = NUL as ::core::ffi::c_char;
            }
            dict = tv_dict_alloc();
            tv_list_append_dict(list, dict);
            tv_dict_add_str(
                dict,
                b"text\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                &raw mut tag_name as *mut ::core::ffi::c_char,
            );
            tv_dict_add_str(
                dict,
                b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                fname,
            );
            tv_dict_add_nr(
                dict,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                lnum as varnumber_T,
            );
            if lnum == 0 as linenr_T {
                tv_dict_add_str(
                    dict,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    cmd,
                );
            }
        }
        i += 1;
    }
    vim_snprintf(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b"ltag %s\0".as_ptr() as *const ::core::ffi::c_char,
        tag,
    );
    set_errorlist(
        curwin,
        list,
        ' ' as ::core::ffi::c_int,
        &raw mut IObuff as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<dict_T>(),
    );
    tv_list_free(list);
    let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut fname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void = &raw mut cmd as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    *ptr__1;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tag_freematch() {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        tagmatchname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
}
unsafe extern "C" fn taglen_advance(mut l: ::core::ffi::c_int) {
    if l == MAXCOL as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        msg_advance(24 as ::core::ffi::c_int);
    } else {
        msg_advance(13 as ::core::ffi::c_int + l);
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_tags(mut _eap: *mut exarg_T) {
    let mut tagstack: *mut taggy_T = &raw mut (*curwin).w_tagstack as *mut taggy_T;
    let mut tagstackidx: ::core::ffi::c_int = (*curwin).w_tagstackidx;
    let mut tagstacklen: ::core::ffi::c_int = (*curwin).w_tagstacklen;
    msg_puts_title(gettext(
        b"\n  # TO tag         FROM line  in file/text\0".as_ptr() as *const ::core::ffi::c_char,
    ));
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < tagstacklen {
        if !(*tagstack.offset(i as isize)).tagname.is_null() {
            let mut name: *mut ::core::ffi::c_char = fm_getname(
                &raw mut (*tagstack.offset(i as isize)).fmark,
                30 as ::core::ffi::c_int,
            );
            if !name.is_null() {
                msg_putchar('\n' as ::core::ffi::c_int);
                vim_snprintf(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%c%2d %2d %-15s %5d  \0".as_ptr() as *const ::core::ffi::c_char,
                    if i == tagstackidx {
                        '>' as ::core::ffi::c_int
                    } else {
                        ' ' as ::core::ffi::c_int
                    },
                    i + 1 as ::core::ffi::c_int,
                    (*tagstack.offset(i as isize)).cur_match + 1 as ::core::ffi::c_int,
                    (*tagstack.offset(i as isize)).tagname,
                    (*tagstack.offset(i as isize)).fmark.mark.lnum,
                );
                msg_outtrans(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                msg_outtrans(
                    name,
                    if (*tagstack.offset(i as isize)).fmark.fnum == (*curbuf).handle {
                        HLF_D as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                    false_0 != 0,
                );
                xfree(name as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
    if tagstackidx == tagstacklen {
        msg_puts(b"\n>\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn tag_strnicmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    while len > 0 as size_t {
        let mut i: ::core::ffi::c_int =
            (if (*s1 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                || *s1 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
            {
                *s1 as uint8_t as ::core::ffi::c_int
            } else {
                *s1 as uint8_t as ::core::ffi::c_int
                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) - (if (*s2 as uint8_t as ::core::ffi::c_int) < 'a' as ::core::ffi::c_int
                || *s2 as uint8_t as ::core::ffi::c_int > 'z' as ::core::ffi::c_int
            {
                *s2 as uint8_t as ::core::ffi::c_int
            } else {
                *s2 as uint8_t as ::core::ffi::c_int
                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            });
        if i != 0 as ::core::ffi::c_int {
            return i;
        }
        if *s1 as ::core::ffi::c_int == NUL {
            break;
        }
        s1 = s1.offset(1);
        s2 = s2.offset(1);
        len = len.wrapping_sub(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn prepare_pats(mut pats: *mut pat_T, mut has_re: bool) {
    (*pats).head = (*pats).pat;
    (*pats).headlen = (*pats).len;
    if has_re {
        if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '^' as ::core::ffi::c_int
        {
            (*pats).head = (*pats).pat.offset(1 as ::core::ffi::c_int as isize);
        } else if *(*pats).pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *(*pats).pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '<' as ::core::ffi::c_int
        {
            (*pats).head = (*pats).pat.offset(2 as ::core::ffi::c_int as isize);
        }
        if (*pats).head == (*pats).pat {
            (*pats).headlen = 0 as ::core::ffi::c_int;
        } else {
            (*pats).headlen = 0 as ::core::ffi::c_int;
            while *(*pats).head.offset((*pats).headlen as isize) as ::core::ffi::c_int != NUL {
                if !vim_strchr(
                    if magic_isset() as ::core::ffi::c_int != 0 {
                        b".[~*\\$\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\\$\0".as_ptr() as *const ::core::ffi::c_char
                    },
                    *(*pats).head.offset((*pats).headlen as isize) as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                {
                    break;
                }
                (*pats).headlen += 1;
            }
        }
        if p_tl != 0 as OptInt && (*pats).headlen as OptInt > p_tl {
            (*pats).headlen = p_tl as ::core::ffi::c_int;
        }
    }
    if has_re {
        (*pats).regmatch.regprog = vim_regcomp(
            (*pats).pat,
            if magic_isset() as ::core::ffi::c_int != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
    } else {
        (*pats).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    };
}
unsafe extern "C" fn find_tagfunc_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut ga: *mut garray_T,
    mut match_count: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ntags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut args: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut flagString: [::core::ffi::c_char; 4] = [0; 4];
    let mut tag: *mut taggy_T = ::core::ptr::null_mut::<taggy_T>();
    if (*curwin).w_tagstacklen > 0 as ::core::ffi::c_int {
        if (*curwin).w_tagstackidx == (*curwin).w_tagstacklen {
            tag = (&raw mut (*curwin).w_tagstack as *mut taggy_T)
                .offset(((*curwin).w_tagstackidx - 1 as ::core::ffi::c_int) as isize);
        } else {
            tag = (&raw mut (*curwin).w_tagstack as *mut taggy_T)
                .offset((*curwin).w_tagstackidx as isize);
        }
    }
    if *(*curbuf).b_p_tfu as ::core::ffi::c_int == NUL
        || (*curbuf).b_tfu_cb.type_0 as ::core::ffi::c_uint
            == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
    args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    args[1 as ::core::ffi::c_int as usize].vval.v_string =
        &raw mut flagString as *mut ::core::ffi::c_char;
    let d: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
    if flags & TAG_INS_COMP as ::core::ffi::c_int == 0
        && !tag.is_null()
        && !(*tag).user_data.is_null()
    {
        tv_dict_add_str(
            d,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (*tag).user_data,
        );
    }
    if !buf_ffname.is_null() {
        tv_dict_add_str(
            d,
            b"buf_ffname\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
            buf_ffname,
        );
    }
    (*d).dv_refcount += 1;
    args[2 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
    args[2 as ::core::ffi::c_int as usize].vval.v_dict = d;
    args[3 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    vim_snprintf(
        &raw mut flagString as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>(),
        b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        if g_tag_at_cursor as ::core::ffi::c_int != 0 {
            b"c\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
            b"i\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if flags & TAG_REGEXP as ::core::ffi::c_int != 0 {
            b"r\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    let mut save_pos: pos_T = (*curwin).w_cursor;
    let mut result: ::core::ffi::c_int = callback_call(
        &raw mut (*curbuf).b_tfu_cb,
        3 as ::core::ffi::c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) as ::core::ffi::c_int;
    (*curwin).w_cursor = save_pos;
    check_cursor(curwin);
    (*d).dv_refcount -= 1;
    if result == FAIL {
        return FAIL;
    }
    if rettv.v_type as ::core::ffi::c_uint
        == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && rettv.vval.v_special as ::core::ffi::c_uint
            == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(&raw mut rettv);
        return NOTDONE;
    }
    if rettv.v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || rettv.vval.v_list.is_null()
    {
        tv_clear(&raw mut rettv);
        emsg(gettext(
            (e_invalid_return_value_from_tagfunc.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut taglist: *mut list_T = rettv.vval.v_list;
    let l_: *const list_T = taglist;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut res_name: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_fname: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_cmd: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut res_kind: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut has_extra: bool = false;
            let mut name_only: ::core::ffi::c_int = flags & TAG_NAMES as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                break;
            } else {
                let mut len: size_t = 2 as size_t;
                res_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                res_kind = ::core::ptr::null_mut::<::core::ffi::c_char>();
                let dihi_ht_: *mut hashtab_T = &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
                let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
                while dihi_todo_ != 0 {
                    if !((*dihi_).hi_key.is_null() || (*dihi_).hi_key == &raw mut hash_removed) {
                        dihi_todo_ = dihi_todo_.wrapping_sub(1);
                        let di: *mut dictitem_T = (*dihi_)
                            .hi_key
                            .offset(-(17 as ::core::ffi::c_ulong as isize))
                            as *mut dictitem_T;
                        let mut dict_key: *const ::core::ffi::c_char =
                            &raw mut (*di).di_key as *mut ::core::ffi::c_char;
                        let mut tv: *mut typval_T = &raw mut (*di).di_tv;
                        if !((*tv).v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*tv).vval.v_string.is_null())
                        {
                            len = len.wrapping_add(
                                strlen((*tv).vval.v_string).wrapping_add(1 as size_t),
                            );
                            if strcmp(dict_key, b"name\0".as_ptr() as *const ::core::ffi::c_char)
                                == 0
                            {
                                res_name = (*tv).vval.v_string;
                            } else if strcmp(
                                dict_key,
                                b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0
                            {
                                res_fname = (*tv).vval.v_string;
                            } else if strcmp(
                                dict_key,
                                b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                            ) == 0
                            {
                                res_cmd = (*tv).vval.v_string;
                            } else {
                                has_extra = true;
                                if strcmp(
                                    dict_key,
                                    b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    res_kind = (*tv).vval.v_string;
                                } else {
                                    len = len
                                        .wrapping_add(strlen(dict_key).wrapping_add(1 as size_t));
                                }
                            }
                        }
                    }
                    dihi_ = dihi_.offset(1);
                }
                if has_extra {
                    len = len.wrapping_add(2 as size_t);
                }
                if res_name.is_null() || res_fname.is_null() || res_cmd.is_null() {
                    emsg(gettext(
                        (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    break;
                } else {
                    let mfp: *mut ::core::ffi::c_char = (if name_only != 0 {
                        xstrdup(res_name) as *mut ::core::ffi::c_void
                    } else {
                        xmalloc(len.wrapping_add(2 as size_t))
                    })
                        as *mut ::core::ffi::c_char;
                    if name_only == 0 {
                        let mut p: *mut ::core::ffi::c_char = mfp;
                        let c2rust_fresh7 = p;
                        p = p.offset(1);
                        *c2rust_fresh7 = (MT_GL_OTH as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                            as ::core::ffi::c_char;
                        let c2rust_fresh8 = p;
                        p = p.offset(1);
                        *c2rust_fresh8 = 0x2 as ::core::ffi::c_char;
                        strcpy(p, res_name);
                        p = p.offset(strlen(p) as isize);
                        let c2rust_fresh9 = p;
                        p = p.offset(1);
                        *c2rust_fresh9 = '\t' as ::core::ffi::c_char;
                        strcpy(p, res_fname);
                        p = p.offset(strlen(p) as isize);
                        let c2rust_fresh10 = p;
                        p = p.offset(1);
                        *c2rust_fresh10 = '\t' as ::core::ffi::c_char;
                        strcpy(p, res_cmd);
                        p = p.offset(strlen(p) as isize);
                        if has_extra {
                            strcpy(
                                p,
                                b";\"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                            p = p.offset(strlen(p) as isize);
                            if !res_kind.is_null() {
                                let c2rust_fresh11 = p;
                                p = p.offset(1);
                                *c2rust_fresh11 = '\t' as ::core::ffi::c_char;
                                strcpy(p, res_kind);
                                p = p.offset(strlen(p) as isize);
                            }
                            let dihi_ht__0: *mut hashtab_T =
                                &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                            let mut dihi_todo__0: size_t = (*dihi_ht__0).ht_used;
                            let mut dihi__0: *mut hashitem_T = (*dihi_ht__0).ht_array;
                            while dihi_todo__0 != 0 {
                                if !((*dihi__0).hi_key.is_null()
                                    || (*dihi__0).hi_key == &raw mut hash_removed)
                                {
                                    dihi_todo__0 = dihi_todo__0.wrapping_sub(1);
                                    let di_0: *mut dictitem_T = (*dihi__0)
                                        .hi_key
                                        .offset(-(17 as ::core::ffi::c_ulong as isize))
                                        as *mut dictitem_T;
                                    let mut dict_key_0: *const ::core::ffi::c_char =
                                        &raw mut (*di_0).di_key as *mut ::core::ffi::c_char;
                                    let mut tv_0: *mut typval_T = &raw mut (*di_0).di_tv;
                                    if !((*tv_0).v_type as ::core::ffi::c_uint
                                        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                                        || (*tv_0).vval.v_string.is_null())
                                    {
                                        if strcmp(
                                            dict_key_0,
                                            b"name\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) != 0
                                        {
                                            if strcmp(
                                                dict_key_0,
                                                b"filename\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) != 0
                                            {
                                                if strcmp(
                                                    dict_key_0,
                                                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    if strcmp(
                                                        dict_key_0,
                                                        b"kind\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                    {
                                                        let c2rust_fresh12 = p;
                                                        p = p.offset(1);
                                                        *c2rust_fresh12 =
                                                            '\t' as ::core::ffi::c_char;
                                                        strcpy(
                                                            p,
                                                            dict_key_0 as *mut ::core::ffi::c_char,
                                                        );
                                                        p = p.offset(strlen(p) as isize);
                                                        strcpy(
                                                            p,
                                                            b":\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                        );
                                                        p = p.offset(strlen(p) as isize);
                                                        strcpy(p, (*tv_0).vval.v_string);
                                                        p = p.offset(strlen(p) as isize);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                dihi__0 = dihi__0.offset(1);
                            }
                        }
                    }
                    ga_grow(ga, 1 as ::core::ffi::c_int);
                    let c2rust_fresh13 = (*ga).ga_len;
                    (*ga).ga_len = (*ga).ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *((*ga).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh13 as isize);
                    *c2rust_lvalue_ptr = mfp;
                    ntags += 1;
                    result = 1 as ::core::ffi::c_int;
                    li = (*li).li_next;
                }
            }
        }
    }
    tv_clear(&raw mut rettv);
    *match_count = ntags;
    return result;
}
unsafe extern "C" fn findtags_state_init(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
) {
    (*st).tag_fname =
        xmalloc((MAXPATHL + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
    (*st).fp = ::core::ptr::null_mut::<FILE>();
    (*st).orgpat = xmalloc(::core::mem::size_of::<pat_T>()) as *mut pat_T;
    (*(*st).orgpat).pat = pat;
    (*(*st).orgpat).len = strlen(pat) as ::core::ffi::c_int;
    (*(*st).orgpat).regmatch.regprog = ::core::ptr::null_mut::<regprog_T>();
    (*st).flags = flags;
    (*st).tag_file_sorted = NUL;
    (*st).help_lang_find = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*st).is_txt = false_0 != 0;
    (*st).did_open = false_0 != 0;
    (*st).help_only = flags & TAG_HELP as ::core::ffi::c_int != 0;
    (*st).get_searchpat = false_0 != 0;
    (*st).help_lang[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    (*st).help_pri = 0 as ::core::ffi::c_int;
    (*st).mincount = mincount;
    (*st).lbuf_size = LSIZE as ::core::ffi::c_int;
    (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
    (*st).match_count = 0 as ::core::ffi::c_int;
    (*st).stop_searching = false_0 != 0;
    let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mtt < MT_COUNT as ::core::ffi::c_int {
        ga_init(
            (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            100 as ::core::ffi::c_int,
        );
        hash_init((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
        mtt += 1;
    }
}
unsafe extern "C" fn findtags_state_free(mut st: *mut findtags_state_T) {
    xfree((*st).tag_fname as *mut ::core::ffi::c_void);
    xfree((*st).lbuf as *mut ::core::ffi::c_void);
    vim_regfree((*(*st).orgpat).regmatch.regprog);
    xfree((*st).orgpat as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn findtags_in_help_init(mut st: *mut findtags_state_T) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    if (*st).is_txt {
        strcpy(
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    } else {
        i = strlen((*st).tag_fname) as ::core::ffi::c_int;
        if i > 3 as ::core::ffi::c_int
            && *(*st)
                .tag_fname
                .offset((i - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
        {
            xmemcpyz(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (*st)
                    .tag_fname
                    .offset(i as isize)
                    .offset(-(2 as ::core::ffi::c_int as isize))
                    as *const ::core::ffi::c_void,
                2 as size_t,
            );
        } else {
            strcpy(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
    }
    if !(*st).help_lang_find.is_null()
        && strcasecmp(
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            (*st).help_lang_find,
        ) != 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    if (*st).flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
        && (*st).help_lang_find.is_null()
        && !(*curbuf).b_fname.is_null()
        && {
            i = strlen((*curbuf).b_fname) as ::core::ffi::c_int;
            i > 4 as ::core::ffi::c_int
        }
        && *(*curbuf)
            .b_fname
            .offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == 'x' as ::core::ffi::c_int
        && *(*curbuf)
            .b_fname
            .offset((i - 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
        && strncasecmp(
            (*curbuf)
                .b_fname
                .offset(i as isize)
                .offset(-(3 as ::core::ffi::c_int as isize)),
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        (*st).help_pri = 0 as ::core::ffi::c_int;
    } else {
        (*st).help_pri = 1 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        s = p_hlg;
        while *s as ::core::ffi::c_int != NUL {
            if strncasecmp(
                s,
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            (*st).help_pri += 1;
            s = vim_strchr(s, ',' as ::core::ffi::c_int);
            if s.is_null() {
                break;
            }
            s = s.offset(1);
        }
        if s.is_null() || *s as ::core::ffi::c_int == NUL {
            (*st).help_pri += 1;
            if strcasecmp(
                &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
                b"en\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
                (*st).help_pri += 1;
            }
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn findtags_apply_tfu(
    mut st: *mut findtags_state_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let use_tfu: bool =
        (*st).flags & TAG_NO_TAGFUNC as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    if !use_tfu
        || tfu_in_use.get() as ::core::ffi::c_int != 0
        || *(*curbuf).b_p_tfu as ::core::ffi::c_int == NUL
    {
        return NOTDONE;
    }
    tfu_in_use.set(true_0 != 0);
    let mut retval: ::core::ffi::c_int = find_tagfunc_tags(
        pat,
        &raw mut (*st).ga_match as *mut garray_T,
        &raw mut (*st).match_count,
        (*st).flags,
        buf_ffname,
    );
    tfu_in_use.set(false_0 != 0);
    return retval;
}
unsafe extern "C" fn findtags_get_next_line(
    mut st: *mut findtags_state_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tags_read_status_T {
    let mut eof: bool = false;
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut offset: off_T =
            (*sinfo_p).low_offset + ((*sinfo_p).high_offset - (*sinfo_p).low_offset) / 2 as off_T;
        if offset == (*sinfo_p).curr_offset {
            return TAGS_READ_EOF;
        } else {
            (*sinfo_p).curr_offset = offset;
        }
    } else if (*st).state as ::core::ffi::c_uint
        == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sinfo_p).curr_offset -= ((*st).lbuf_size * 2 as ::core::ffi::c_int) as off_T;
        if (*sinfo_p).curr_offset < 0 as off_T {
            (*sinfo_p).curr_offset = 0 as off_T;
            fseek((*st).fp, 0 as ::core::ffi::c_long, SEEK_SET);
            (*st).state = TS_STEP_FORWARD;
        }
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*st).state as ::core::ffi::c_uint
            == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sinfo_p).curr_offset_used = (*sinfo_p).curr_offset;
        vim_ignored = fseeko((*st).fp, (*sinfo_p).curr_offset as __off_t, SEEK_SET);
        eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        if !eof && (*sinfo_p).curr_offset != 0 as off_T {
            (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
            if (*sinfo_p).curr_offset == (*sinfo_p).high_offset {
                vim_ignored = fseeko((*st).fp, (*sinfo_p).low_offset as __off_t, SEEK_SET);
                (*sinfo_p).curr_offset = (*sinfo_p).low_offset;
            }
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        }
        while !eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0 {
            (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
        }
        if eof {
            (*st).state = TS_SKIP_BACK;
            (*sinfo_p).match_offset = ftello((*st).fp) as off_T;
            (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
            return TAGS_READ_IGNORE;
        }
    } else {
        loop {
            eof = vim_fgets((*st).lbuf, (*st).lbuf_size, (*st).fp);
            if !(!eof && vim_isblankline((*st).lbuf) as ::core::ffi::c_int != 0) {
                break;
            }
        }
        if eof {
            return TAGS_READ_EOF;
        }
    }
    return TAGS_READ_SUCCESS;
}
unsafe extern "C" fn findtags_hdr_parse(mut st: *mut findtags_state_T) -> bool {
    if strncmp(
        (*st).lbuf,
        b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if strncmp(
        (*st).lbuf,
        b"!_TAG_FILE_SORTED\t\0".as_ptr() as *const ::core::ffi::c_char,
        18 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        (*st).tag_file_sorted =
            *(*st).lbuf.offset(18 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
    }
    if strncmp(
        (*st).lbuf,
        b"!_TAG_FILE_ENCODING\t\0".as_ptr() as *const ::core::ffi::c_char,
        20 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = (*st).lbuf.offset(20 as ::core::ffi::c_int as isize);
        while *p as ::core::ffi::c_int > ' ' as ::core::ffi::c_int
            && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        *p = NUL as ::core::ffi::c_char;
        convert_setup(
            &raw mut (*st).vimconv,
            (*st).lbuf.offset(20 as ::core::ffi::c_int as isize),
            p_enc,
        );
    }
    return false_0 != 0;
}
unsafe extern "C" fn findtags_start_state_handler(
    mut st: *mut findtags_state_T,
    mut sortic: *mut bool,
    mut sinfo_p: *mut tagsearch_info_T,
) -> bool {
    let noic: bool = (*st).flags & TAG_NOIC as ::core::ffi::c_int != 0;
    if strncmp(
        (*st).lbuf,
        b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) <= 0 as ::core::ffi::c_int
        || *(*st).lbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '!' as ::core::ffi::c_int
            && (*(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                >= 'a' as ::core::ffi::c_uint
                && *(*st).lbuf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                    <= 'z' as ::core::ffi::c_uint)
    {
        return findtags_hdr_parse(st);
    }
    if (*st).linear {
        (*st).state = TS_LINEAR;
    } else if (*st).tag_file_sorted == NUL {
        (*st).state = TS_BINARY;
    } else if (*st).tag_file_sorted == '1' as ::core::ffi::c_int {
        (*st).state = TS_BINARY;
    } else if (*st).tag_file_sorted == '2' as ::core::ffi::c_int {
        (*st).state = TS_BINARY;
        *sortic = true_0 != 0;
        (*(*st).orgpat).regmatch.rm_ic = p_ic != 0 || !noic;
    } else {
        (*st).state = TS_LINEAR;
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
        && !*sortic
    {
        (*st).linear = true_0 != 0;
        (*st).state = TS_LINEAR;
    }
    if (*st).state as ::core::ffi::c_uint == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if fseeko((*st).fp, 0 as __off_t, SEEK_END) != 0 as ::core::ffi::c_int {
            (*st).state = TS_LINEAR;
        } else {
            let filesize: off_T = ftello((*st).fp);
            vim_ignored = fseeko((*st).fp, 0 as __off_t, SEEK_SET);
            (*sinfo_p).low_offset = 0 as off_T;
            (*sinfo_p).low_char = 0 as ::core::ffi::c_int;
            (*sinfo_p).high_offset = filesize;
            (*sinfo_p).curr_offset = 0 as off_T;
            (*sinfo_p).high_char = 0xff as ::core::ffi::c_int;
        }
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn findtags_parse_line(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut sinfo_p: *mut tagsearch_info_T,
) -> tagmatch_status_T {
    let mut status: ::core::ffi::c_int = 0;
    if (*(*st).orgpat).headlen != 0 {
        memset(
            tagpp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tagptrs_T>(),
        );
        (*tagpp).tagname = (*st).lbuf;
        (*tagpp).tagname_end = vim_strchr((*st).lbuf, TAB);
        if (*tagpp).tagname_end.is_null() {
            return TAG_MATCH_FAIL;
        }
        let mut cmplen: ::core::ffi::c_int =
            (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
        if p_tl != 0 as OptInt && cmplen as OptInt > p_tl {
            cmplen = p_tl as ::core::ffi::c_int;
        }
        if (*st).flags & TAG_REGEXP as ::core::ffi::c_int != 0 && (*(*st).orgpat).headlen < cmplen {
            cmplen = (*(*st).orgpat).headlen;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*(*st).orgpat).headlen != cmplen
        {
            return TAG_MATCH_NEXT;
        }
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tagcmp: ::core::ffi::c_int = 0;
            let mut i: ::core::ffi::c_int =
                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                    as ::core::ffi::c_int;
            if (*margs).sortic {
                i = if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int)
                    < 'a' as ::core::ffi::c_int
                    || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        > 'z' as ::core::ffi::c_int
                {
                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else {
                    *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                };
            }
            if i < (*sinfo_p).low_char || i > (*sinfo_p).high_char {
                (*margs).sort_error = true_0 != 0;
            }
            if (*margs).sortic {
                tagcmp = tag_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
            } else {
                tagcmp = strncmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t);
            }
            if tagcmp == 0 as ::core::ffi::c_int {
                if cmplen < (*(*st).orgpat).headlen {
                    tagcmp = -1 as ::core::ffi::c_int;
                } else if cmplen > (*(*st).orgpat).headlen {
                    tagcmp = 1 as ::core::ffi::c_int;
                }
            }
            if tagcmp == 0 as ::core::ffi::c_int {
                (*st).state = TS_SKIP_BACK;
                (*sinfo_p).match_offset = (*sinfo_p).curr_offset;
                return TAG_MATCH_NEXT;
            }
            if tagcmp < 0 as ::core::ffi::c_int {
                (*sinfo_p).curr_offset = ftello((*st).fp) as off_T;
                if (*sinfo_p).curr_offset < (*sinfo_p).high_offset {
                    (*sinfo_p).low_offset = (*sinfo_p).curr_offset;
                    if (*margs).sortic {
                        (*sinfo_p).low_char =
                            if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                                < 'a' as ::core::ffi::c_int
                                || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    > 'z' as ::core::ffi::c_int
                            {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                            } else {
                                *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                            };
                    } else {
                        (*sinfo_p).low_char =
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int;
                    }
                    return TAG_MATCH_NEXT;
                }
            }
            if tagcmp > 0 as ::core::ffi::c_int && (*sinfo_p).curr_offset != (*sinfo_p).high_offset
            {
                (*sinfo_p).high_offset = (*sinfo_p).curr_offset;
                if (*margs).sortic {
                    (*sinfo_p).high_char =
                        if (*(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int)
                            < 'a' as ::core::ffi::c_int
                            || *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                > 'z' as ::core::ffi::c_int
                        {
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                        } else {
                            *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
                        };
                } else {
                    (*sinfo_p).high_char =
                        *(*tagpp).tagname.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int;
                }
                return TAG_MATCH_NEXT;
            }
            return TAG_MATCH_STOP;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            '_c2rust_label: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1797 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                (*st).state = TS_STEP_FORWARD;
            } else {
                (*sinfo_p).curr_offset = (*sinfo_p).curr_offset_used;
            }
            return TAG_MATCH_NEXT;
        } else if (*st).state as ::core::ffi::c_uint
            == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            '_c2rust_label_0: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1807 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                return (if ftello((*st).fp) > (*sinfo_p).match_offset {
                    TAG_MATCH_STOP as ::core::ffi::c_int
                } else {
                    TAG_MATCH_NEXT as ::core::ffi::c_int
                }) as tagmatch_status_T;
            }
        } else {
            '_c2rust_label_1: {
                if cmplen >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tag.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1815 as ::core::ffi::c_uint,
                        b"tagmatch_status_T findtags_parse_line(findtags_state_T *, tagptrs_T *, findtags_match_args_T *, tagsearch_info_T *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).head, cmplen as size_t)
                != 0 as ::core::ffi::c_int
            {
                return TAG_MATCH_NEXT;
            }
        }
        (*tagpp).fname = (*tagpp)
            .tagname_end
            .offset(1 as ::core::ffi::c_int as isize);
        (*tagpp).fname_end = vim_strchr((*tagpp).fname, TAB);
        if (*tagpp).fname_end.is_null() {
            status = FAIL;
        } else {
            (*tagpp).command = (*tagpp).fname_end.offset(1 as ::core::ffi::c_int as isize);
            status = OK;
        }
    } else {
        status = parse_tag_line((*st).lbuf, tagpp);
    }
    return (if status == FAIL {
        TAG_MATCH_FAIL as ::core::ffi::c_int
    } else {
        TAG_MATCH_SUCCESS as ::core::ffi::c_int
    }) as tagmatch_status_T;
}
unsafe extern "C" fn findtags_matchargs_init(
    mut margs: *mut findtags_match_args_T,
    mut flags: ::core::ffi::c_int,
) {
    (*margs).matchoff = 0 as ::core::ffi::c_int;
    (*margs).match_re = false_0 != 0;
    (*margs).match_no_ic = false_0 != 0;
    (*margs).has_re = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
    (*margs).sortic = false_0 != 0;
    (*margs).sort_error = false_0 != 0;
}
unsafe extern "C" fn findtags_match_tag(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
) -> bool {
    let mut match_0: bool = false_0 != 0;
    let mut cmplen: ::core::ffi::c_int =
        (*tagpp).tagname_end.offset_from((*tagpp).tagname) as ::core::ffi::c_int;
    if p_tl != 0 as OptInt && cmplen as OptInt > p_tl {
        cmplen = p_tl as ::core::ffi::c_int;
    }
    if (*(*st).orgpat).len != cmplen {
        match_0 = false_0 != 0;
    } else if (*(*st).orgpat).regmatch.rm_ic {
        '_c2rust_label: {
            if cmplen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"cmplen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tag.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1869 as ::core::ffi::c_uint,
                    b"_Bool findtags_match_tag(findtags_state_T *, tagptrs_T *, findtags_match_args_T *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        match_0 = mb_strnicmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
            == 0 as ::core::ffi::c_int;
        if match_0 {
            (*margs).match_no_ic = strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
                == 0 as ::core::ffi::c_int;
        }
    } else {
        match_0 = strncmp((*tagpp).tagname, (*(*st).orgpat).pat, cmplen as size_t)
            == 0 as ::core::ffi::c_int;
    }
    (*margs).match_re = false_0 != 0;
    if !match_0 && !(*(*st).orgpat).regmatch.regprog.is_null() {
        let mut cc: ::core::ffi::c_char = *(*tagpp).tagname_end;
        *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
        match_0 = vim_regexec(
            &raw mut (*(*st).orgpat).regmatch,
            (*tagpp).tagname,
            0 as colnr_T,
        );
        if match_0 {
            (*margs).matchoff = (*(*st).orgpat).regmatch.startp[0 as ::core::ffi::c_int as usize]
                .offset_from((*tagpp).tagname)
                as ::core::ffi::c_int;
            if (*(*st).orgpat).regmatch.rm_ic {
                (*(*st).orgpat).regmatch.rm_ic = false_0 != 0;
                (*margs).match_no_ic = vim_regexec(
                    &raw mut (*(*st).orgpat).regmatch,
                    (*tagpp).tagname,
                    0 as colnr_T,
                );
                (*(*st).orgpat).regmatch.rm_ic = true_0 != 0;
            }
        }
        *(*tagpp).tagname_end = cc;
        (*margs).match_re = true_0 != 0;
    }
    return match_0;
}
unsafe extern "C" fn findtags_string_convert(mut st: *mut findtags_state_T) {
    let mut conv_line: *mut ::core::ffi::c_char = string_convert(
        &raw mut (*st).vimconv,
        (*st).lbuf,
        ::core::ptr::null_mut::<size_t>(),
    );
    if conv_line.is_null() {
        return;
    }
    let mut len: ::core::ffi::c_int =
        strlen(conv_line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if len > (*st).lbuf_size {
        xfree((*st).lbuf as *mut ::core::ffi::c_void);
        (*st).lbuf = conv_line;
        (*st).lbuf_size = len;
    } else {
        strcpy((*st).lbuf, conv_line);
        xfree(conv_line as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn findtags_add_match(
    mut st: *mut findtags_state_T,
    mut tagpp: *mut tagptrs_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
    mut hash: *mut hash_T,
) {
    let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
    let mut len: size_t = 0 as size_t;
    let mut mfp_size: size_t = 0 as size_t;
    let mut mfp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_current: bool = test_for_current(
        (*tagpp).fname,
        (*tagpp).fname_end,
        (*st).tag_fname,
        buf_ffname,
    ) != 0;
    let mut is_static: bool = test_for_static(tagpp);
    let mut mtt: ::core::ffi::c_int = if is_static as ::core::ffi::c_int != 0 {
        if is_current as ::core::ffi::c_int != 0 {
            MT_ST_CUR as ::core::ffi::c_int
        } else {
            MT_ST_OTH as ::core::ffi::c_int
        }
    } else if is_current as ::core::ffi::c_int != 0 {
        MT_GL_CUR as ::core::ffi::c_int
    } else {
        MT_GL_OTH as ::core::ffi::c_int
    };
    if (*(*st).orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0 && !(*margs).match_no_ic {
        mtt += MT_IC_OFF as ::core::ffi::c_int;
    }
    if (*margs).match_re {
        mtt += MT_RE_OFF as ::core::ffi::c_int;
    }
    if (*st).help_only {
        *(*tagpp).tagname_end = NUL as ::core::ffi::c_char;
        len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
        mfp_size = ::core::mem::size_of::<::core::ffi::c_char>()
            .wrapping_add(len as usize)
            .wrapping_add(10 as usize)
            .wrapping_add(ML_EXTRA as usize)
            .wrapping_add(1 as usize) as size_t;
        mfp = xmalloc(mfp_size) as *mut ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = mfp;
        strcpy(p, (*tagpp).tagname);
        *p.offset(len as isize) = '@' as ::core::ffi::c_char;
        strcpy(
            p.offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize),
            &raw mut (*st).help_lang as *mut ::core::ffi::c_char,
        );
        snprintf(
            p.offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize)
                .offset(ML_EXTRA as isize),
            mfp_size.wrapping_sub(
                len.wrapping_add(1 as size_t)
                    .wrapping_add(ML_EXTRA as size_t),
            ),
            b"%06d\0".as_ptr() as *const ::core::ffi::c_char,
            help_heuristic(
                (*tagpp).tagname,
                if (*margs).match_re as ::core::ffi::c_int != 0 {
                    (*margs).matchoff
                } else {
                    0 as ::core::ffi::c_int
                },
                !(*margs).match_no_ic,
            ) + (*st).help_pri,
        );
        *(*tagpp).tagname_end = TAB as ::core::ffi::c_char;
    } else if name_only {
        if (*st).get_searchpat {
            let mut temp_end: *mut ::core::ffi::c_char = (*tagpp).command;
            if *temp_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
                while *temp_end as ::core::ffi::c_int != 0
                    && *temp_end as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    && *temp_end as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                    && *temp_end as ::core::ffi::c_int != '$' as ::core::ffi::c_int
                {
                    temp_end = temp_end.offset(1);
                }
            }
            if (*tagpp).command.offset(2 as ::core::ffi::c_int as isize) < temp_end {
                len = (temp_end.offset_from((*tagpp).command) - 2 as isize) as size_t;
                mfp = xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
                xmemcpyz(
                    mfp as *mut ::core::ffi::c_void,
                    (*tagpp).command.offset(2 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    len,
                );
            } else {
                mfp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            (*st).get_searchpat = false_0 != 0;
        } else {
            len = (*tagpp).tagname_end.offset_from((*tagpp).tagname) as size_t;
            mfp = xmalloc(
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_add(len)
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            xmemcpyz(
                mfp as *mut ::core::ffi::c_void,
                (*tagpp).tagname as *const ::core::ffi::c_void,
                len,
            );
            if State & MODE_INSERT as ::core::ffi::c_int != 0 {
                (*st).get_searchpat = p_sft != 0;
            }
        }
    } else {
        let mut tag_fname_len: size_t = strlen((*st).tag_fname);
        len = tag_fname_len
            .wrapping_add(strlen((*st).lbuf))
            .wrapping_add(3 as size_t);
        mfp = xmalloc(
            ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_add(len)
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        let mut p_0: *mut ::core::ffi::c_char = mfp;
        *p_0.offset(0 as ::core::ffi::c_int as isize) =
            (mtt + 1 as ::core::ffi::c_int) as ::core::ffi::c_char;
        strcpy(
            p_0.offset(1 as ::core::ffi::c_int as isize),
            (*st).tag_fname,
        );
        *p_0.offset(tag_fname_len.wrapping_add(1 as size_t) as isize) =
            TAG_SEP as ::core::ffi::c_char;
        let mut s: *mut ::core::ffi::c_char = p_0
            .offset(1 as ::core::ffi::c_int as isize)
            .offset(tag_fname_len as isize)
            .offset(1 as ::core::ffi::c_int as isize);
        strcpy(s, (*st).lbuf);
    }
    if !mfp.is_null() {
        let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
        *hash = hash_hash(mfp);
        hi = hash_lookup(
            (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
            mfp,
            strlen(mfp),
            *hash,
        );
        if (*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed {
            hash_add_item(
                (&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize),
                hi,
                mfp,
                *hash,
            );
            ga_grow(
                (&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize),
                1 as ::core::ffi::c_int,
            );
            *((*st).ga_match[mtt as usize].ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*st).ga_match[mtt as usize].ga_len as isize) = mfp;
            (*st).ga_match[mtt as usize].ga_len += 1;
            (*st).match_count += 1;
        } else {
            xfree(mfp as *mut ::core::ffi::c_void);
        }
    }
}
pub const ML_EXTRA: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
unsafe extern "C" fn findtags_get_all_tags(
    mut st: *mut findtags_state_T,
    mut margs: *mut findtags_match_args_T,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut search_info: tagsearch_info_T = tagsearch_info_T {
        low_offset: 0,
        high_offset: 0,
        curr_offset: 0,
        curr_offset_used: 0,
        match_offset: 0,
        low_char: 0,
        high_char: 0,
    };
    let mut hash: hash_T = 0 as hash_T;
    memset(
        &raw mut search_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<tagsearch_info_T>(),
    );
    let mut retval: ::core::ffi::c_int = 0;
    loop {
        if (*st).state as ::core::ffi::c_uint
            == TS_BINARY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*st).state as ::core::ffi::c_uint
                == TS_SKIP_BACK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            line_breakcheck();
        } else {
            fast_breakcheck();
        }
        if (*st).flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
            ins_compl_check_keys(30 as ::core::ffi::c_int, false_0 != 0);
        }
        if got_int as ::core::ffi::c_int != 0 || ins_compl_interrupted() as ::core::ffi::c_int != 0
        {
            (*st).stop_searching = true_0 != 0;
            break;
        } else if (*st).mincount == TAG_MANY as ::core::ffi::c_int
            && (*st).match_count >= TAG_MANY as ::core::ffi::c_int
        {
            (*st).stop_searching = true_0 != 0;
            break;
        } else {
            if !(*st).get_searchpat {
                retval = findtags_get_next_line(st, &raw mut search_info) as ::core::ffi::c_int;
                if retval == TAGS_READ_IGNORE as ::core::ffi::c_int {
                    continue;
                }
                if retval == TAGS_READ_EOF as ::core::ffi::c_int {
                    break;
                }
            }
            if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
                findtags_string_convert(st);
            }
            if (*st).state as ::core::ffi::c_uint
                == TS_START as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !findtags_start_state_handler(st, &raw mut (*margs).sortic, &raw mut search_info)
                {
                    continue;
                }
            }
            if *(*st)
                .lbuf
                .offset(((*st).lbuf_size - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                != NUL
            {
                (*st).lbuf_size *= 2 as ::core::ffi::c_int;
                xfree((*st).lbuf as *mut ::core::ffi::c_void);
                (*st).lbuf = xmalloc((*st).lbuf_size as size_t) as *mut ::core::ffi::c_char;
                if (*st).state as ::core::ffi::c_uint
                    == TS_STEP_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*st).state as ::core::ffi::c_uint
                        == TS_LINEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    vim_ignored = fseeko((*st).fp, search_info.curr_offset as __off_t, SEEK_SET);
                }
                search_info.curr_offset = 0 as off_T;
            } else {
                retval = findtags_parse_line(st, &raw mut tagp, margs, &raw mut search_info)
                    as ::core::ffi::c_int;
                if retval == TAG_MATCH_NEXT as ::core::ffi::c_int {
                    continue;
                }
                if retval == TAG_MATCH_STOP as ::core::ffi::c_int {
                    break;
                }
                if retval == TAG_MATCH_FAIL as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E431: Format error in tags file \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*st).tag_fname,
                    );
                    semsg(
                        gettext(b"Before byte %ld\0".as_ptr() as *const ::core::ffi::c_char),
                        ftello((*st).fp) as int64_t,
                    );
                    (*st).stop_searching = true_0 != 0;
                    return;
                }
                if findtags_match_tag(st, &raw mut tagp, margs) {
                    findtags_add_match(st, &raw mut tagp, margs, buf_ffname, &raw mut hash);
                }
            }
        }
    }
}
unsafe extern "C" fn findtags_in_file(
    mut st: *mut findtags_state_T,
    mut _flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) {
    let mut margs: findtags_match_args_T = findtags_match_args_T {
        matchoff: 0,
        match_re: false,
        match_no_ic: false,
        has_re: false,
        sortic: false,
        sort_error: false,
    };
    (*st).vimconv.vc_type = CONV_NONE as ::core::ffi::c_int;
    (*st).tag_file_sorted = NUL;
    (*st).fp = ::core::ptr::null_mut::<FILE>();
    findtags_matchargs_init(&raw mut margs, (*st).flags);
    if (*curbuf).b_help {
        if !findtags_in_help_init(st) {
            return;
        }
    }
    (*st).fp = os_fopen(
        (*st).tag_fname,
        b"r\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*st).fp.is_null() {
        return;
    }
    if p_verbose >= 5 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Searching tags file %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*st).tag_fname,
        );
        verbose_leave();
    }
    (*st).did_open = true_0 != 0;
    (*st).state = TS_START;
    findtags_get_all_tags(st, &raw mut margs, buf_ffname);
    if !(*st).fp.is_null() {
        fclose((*st).fp);
        (*st).fp = ::core::ptr::null_mut::<FILE>();
    }
    if (*st).vimconv.vc_type != CONV_NONE as ::core::ffi::c_int {
        convert_setup(
            &raw mut (*st).vimconv,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    }
    if margs.sort_error {
        semsg(
            gettext(b"E432: Tags file not sorted: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*st).tag_fname,
        );
    }
    if (*st).match_count >= (*st).mincount {
        (*st).stop_searching = true_0 != 0;
    }
}
unsafe extern "C" fn findtags_copy_matches(
    mut st: *mut findtags_state_T,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let name_only: bool = (*st).flags & TAG_NAMES as ::core::ffi::c_int != 0;
    let mut matches: *mut *mut ::core::ffi::c_char = (if (*st).match_count > 0 as ::core::ffi::c_int
    {
        xmalloc(
            ((*st).match_count as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        )
    } else {
        NULL_0
    }) as *mut *mut ::core::ffi::c_char;
    (*st).match_count = 0 as ::core::ffi::c_int;
    let mut mtt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while mtt < MT_COUNT as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*st).ga_match[mtt as usize].ga_len {
            let mut mfp: *mut ::core::ffi::c_char = *((*st).ga_match[mtt as usize].ga_data
                as *mut *mut ::core::ffi::c_char)
                .offset(i as isize);
            if matches.is_null() {
                xfree(mfp as *mut ::core::ffi::c_void);
            } else {
                if !name_only {
                    *mfp = (*mfp as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        as ::core::ffi::c_char;
                    let mut p: *mut ::core::ffi::c_char =
                        mfp.offset(1 as ::core::ffi::c_int as isize);
                    while *p as ::core::ffi::c_int != NUL {
                        if *p as ::core::ffi::c_int == TAG_SEP {
                            *p = NUL as ::core::ffi::c_char;
                        }
                        p = p.offset(1);
                    }
                }
                let c2rust_fresh4 = (*st).match_count;
                (*st).match_count = (*st).match_count + 1;
                let c2rust_lvalue_ptr = &raw mut *matches.offset(c2rust_fresh4 as isize);
                *c2rust_lvalue_ptr = mfp;
            }
            i += 1;
        }
        ga_clear((&raw mut (*st).ga_match as *mut garray_T).offset(mtt as isize));
        hash_clear((&raw mut (*st).ht_match as *mut hashtab_T).offset(mtt as isize));
        mtt += 1;
    }
    *matchesp = matches;
    return (*st).match_count;
}
#[no_mangle]
pub unsafe extern "C" fn find_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut num_matches: *mut ::core::ffi::c_int,
    mut matchesp: *mut *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mincount: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut st: findtags_state_T = findtags_state_T {
        state: TS_START,
        stop_searching: false,
        orgpat: ::core::ptr::null_mut::<pat_T>(),
        lbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lbuf_size: 0,
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fp: ::core::ptr::null_mut::<FILE>(),
        flags: 0,
        tag_file_sorted: 0,
        get_searchpat: false,
        help_only: false,
        did_open: false,
        mincount: 0,
        linear: false,
        vimconv: vimconv_T {
            vc_type: 0,
            vc_factor: 0,
            vc_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            vc_fail: false,
        },
        help_lang: [0; 3],
        help_pri: 0,
        help_lang_find: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        is_txt: false,
        match_count: 0,
        ga_match: [garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        }; 16],
        ht_match: [hashtab_T {
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
        }; 16],
    };
    let mut tn: tagname_T = tagname_T {
        tn_tags: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_np: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tn_did_filefind_init: 0,
        tn_hf_idx: 0,
        tn_search_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut first_file: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut i: ::core::ffi::c_int = 0;
    let mut saved_pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut findall: ::core::ffi::c_int = (mincount == MAXCOL as ::core::ffi::c_int
        || mincount == TAG_MANY as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    let mut has_re: bool = flags & TAG_REGEXP as ::core::ffi::c_int != 0;
    let mut noic: ::core::ffi::c_int = flags & TAG_NOIC as ::core::ffi::c_int;
    let mut verbose: ::core::ffi::c_int = flags & TAG_VERBOSE as ::core::ffi::c_int;
    let mut save_p_ic: ::core::ffi::c_int = p_ic;
    match if (*curbuf).b_tc_flags != 0 {
        (*curbuf).b_tc_flags
    } else {
        tc_flags
    } {
        1 => {}
        2 => {
            p_ic = true_0;
        }
        4 => {
            p_ic = false_0;
        }
        8 => {
            p_ic = ignorecase(pat);
        }
        16 => {
            p_ic = ignorecase_opt(pat, true_0, true_0);
        }
        _ => {
            abort();
        }
    }
    let mut help_save: ::core::ffi::c_int = (*curbuf).b_help as ::core::ffi::c_int;
    findtags_state_init(&raw mut st, pat, flags, mincount);
    if st.help_only {
        (*curbuf).b_help = true_0 != 0;
    }
    if (*curbuf).b_help {
        if (*st.orgpat).len > 3 as ::core::ffi::c_int
            && *pat.offset(((*st.orgpat).len - 3 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                == '@' as ::core::ffi::c_int
            && (*pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
            && (*pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_uint
                >= 'A' as ::core::ffi::c_uint
                && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    <= 'Z' as ::core::ffi::c_uint
                || *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_uint
                    >= 'a' as ::core::ffi::c_uint
                    && *pat.offset(((*st.orgpat).len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_uint
                        <= 'z' as ::core::ffi::c_uint)
        {
            saved_pat = xstrnsave(pat, ((*st.orgpat).len as size_t).wrapping_sub(3 as size_t));
            st.help_lang_find = pat.offset(((*st.orgpat).len - 2 as ::core::ffi::c_int) as isize);
            (*st.orgpat).pat = saved_pat;
            (*st.orgpat).len -= 3 as ::core::ffi::c_int;
        }
    }
    if p_tl != 0 as OptInt && (*st.orgpat).len as OptInt > p_tl {
        (*st.orgpat).len = p_tl as ::core::ffi::c_int;
    }
    let mut save_emsg_off: ::core::ffi::c_int = emsg_off;
    emsg_off = true_0;
    prepare_pats(st.orgpat, has_re);
    emsg_off = save_emsg_off;
    if !(has_re as ::core::ffi::c_int != 0 && (*st.orgpat).regmatch.regprog.is_null()) {
        retval = findtags_apply_tfu(&raw mut st, pat, buf_ffname);
        if retval == NOTDONE {
            retval = FAIL;
            if flags & TAG_KEEP_LANG as ::core::ffi::c_int != 0
                && st.help_lang_find.is_null()
                && !(*curbuf).b_fname.is_null()
                && {
                    i = strlen((*curbuf).b_fname) as ::core::ffi::c_int;
                    i > 4 as ::core::ffi::c_int
                }
                && strcasecmp(
                    (*curbuf)
                        .b_fname
                        .offset(i as isize)
                        .offset(-(4 as ::core::ffi::c_int as isize)),
                    b".txt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                st.is_txt = true_0 != 0;
            }
            (*st.orgpat).regmatch.rm_ic = (p_ic != 0 || noic == 0)
                && (findall != 0 || (*st.orgpat).headlen == 0 as ::core::ffi::c_int || p_tbs == 0);
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                st.linear = (*st.orgpat).headlen == 0 as ::core::ffi::c_int
                    || p_tbs == 0
                    || round == 2 as ::core::ffi::c_int;
                first_file = true_0;
                while get_tagfname(&raw mut tn, first_file, st.tag_fname) == OK {
                    findtags_in_file(&raw mut st, flags, buf_ffname);
                    if st.stop_searching {
                        retval = OK;
                        break;
                    } else {
                        first_file = false_0;
                    }
                }
                tagname_free(&raw mut tn);
                if st.stop_searching as ::core::ffi::c_int != 0
                    || st.linear as ::core::ffi::c_int != 0
                    || p_ic == 0 && noic != 0
                    || (*st.orgpat).regmatch.rm_ic as ::core::ffi::c_int != 0
                {
                    break;
                }
                (*st.orgpat).regmatch.rm_ic = true_0 != 0;
                round += 1;
            }
            if !st.stop_searching {
                if !st.did_open && verbose != 0 {
                    emsg(gettext(
                        b"E433: No tags file\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                retval = OK;
            }
        }
    }
    findtags_state_free(&raw mut st);
    if retval == FAIL {
        st.match_count = 0 as ::core::ffi::c_int;
    }
    *num_matches = findtags_copy_matches(&raw mut st, matchesp);
    (*curbuf).b_help = help_save != 0;
    xfree(saved_pat as *mut ::core::ffi::c_void);
    p_ic = save_p_ic;
    return retval;
}
static tag_fnames: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
unsafe extern "C" fn found_tagfile_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut _cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        let tag_fname: *mut ::core::ffi::c_char = xstrdup(*fnames.offset(i as isize));
        simplify_filename(tag_fname);
        ga_grow(tag_fnames.ptr(), 1 as ::core::ffi::c_int);
        *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
            .offset((*tag_fnames.ptr()).ga_len as isize) = tag_fname;
        (*tag_fnames.ptr()).ga_len += 1;
        if !all {
            break;
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn get_tagfname(
    mut tnp: *mut tagname_T,
    mut first: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if first != 0 {
        memset(
            tnp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<tagname_T>(),
        );
    }
    if (*curbuf).b_help {
        if first != 0 {
            ga_clear_strings(tag_fnames.ptr());
            ga_init(
                tag_fnames.ptr(),
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                10 as ::core::ffi::c_int,
            );
            do_in_runtimepath(
                b"doc/tags doc/tags-??\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                DIP_ALL as ::core::ffi::c_int,
                Some(
                    found_tagfile_cb
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
        if (*tnp).tn_hf_idx >= (*tag_fnames.ptr()).ga_len {
            if (*tnp).tn_hf_idx > (*tag_fnames.ptr()).ga_len || *p_hf as ::core::ffi::c_int == NUL {
                return FAIL;
            }
            (*tnp).tn_hf_idx += 1;
            xstrlcpy(
                buf,
                p_hf,
                (MAXPATHL as size_t).wrapping_sub(
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                ),
            );
            strcpy(
                path_tail(buf),
                b"tags\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            simplify_filename(buf);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*tag_fnames.ptr()).ga_len {
                if strcmp(
                    buf,
                    *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset(i as isize),
                ) == 0 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                i += 1;
            }
        } else {
            let c2rust_fresh5 = (*tnp).tn_hf_idx;
            (*tnp).tn_hf_idx = (*tnp).tn_hf_idx + 1;
            xstrlcpy(
                buf,
                *((*tag_fnames.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh5 as isize),
                MAXPATHL as size_t,
            );
        }
        return OK;
    }
    if first != 0 {
        (*tnp).tn_tags = xstrdup(if *(*curbuf).b_p_tags as ::core::ffi::c_int != NUL {
            (*curbuf).b_p_tags
        } else {
            p_tags
        });
        (*tnp).tn_np = (*tnp).tn_tags;
    }
    loop {
        if (*tnp).tn_did_filefind_init != 0 {
            fname = vim_findfile((*tnp).tn_search_ctx);
            if !fname.is_null() {
                break;
            }
            (*tnp).tn_did_filefind_init = false_0;
        } else {
            let mut filename: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if *(*tnp).tn_np as ::core::ffi::c_int == NUL {
                vim_findfile_cleanup((*tnp).tn_search_ctx);
                (*tnp).tn_search_ctx = NULL_0;
                return FAIL;
            }
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            copy_option_part(
                &raw mut (*tnp).tn_np,
                buf,
                (MAXPATHL - 1 as ::core::ffi::c_int) as size_t,
                b" ,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut r_ptr: *mut ::core::ffi::c_char = vim_findfile_stopdir(buf);
            filename = path_tail(buf);
            if !r_ptr.is_null() {
                memmove(
                    r_ptr.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    r_ptr as *const ::core::ffi::c_void,
                    strlen(r_ptr).wrapping_add(1 as size_t),
                );
                r_ptr = r_ptr.offset(1);
            }
            memmove(
                filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                filename as *const ::core::ffi::c_void,
                strlen(filename).wrapping_add(1 as size_t),
            );
            let c2rust_fresh6 = filename;
            filename = filename.offset(1);
            *c2rust_fresh6 = NUL as ::core::ffi::c_char;
            (*tnp).tn_search_ctx = vim_findfile_init(
                buf,
                filename,
                strlen(filename),
                r_ptr,
                100 as ::core::ffi::c_int,
                false_0,
                FINDFILE_FILE as ::core::ffi::c_int,
                (*tnp).tn_search_ctx,
                true_0,
                (*curbuf).b_ffname,
            );
            if !(*tnp).tn_search_ctx.is_null() {
                (*tnp).tn_did_filefind_init = true_0;
            }
        }
    }
    strcpy(buf, fname);
    xfree(fname as *mut ::core::ffi::c_void);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tagname_free(mut tnp: *mut tagname_T) {
    xfree((*tnp).tn_tags as *mut ::core::ffi::c_void);
    vim_findfile_cleanup((*tnp).tn_search_ctx);
    (*tnp).tn_search_ctx = NULL_0;
    ga_clear_strings(tag_fnames.ptr());
}
unsafe extern "C" fn parse_tag_line(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    (*tagp).tagname = lbuf;
    let mut p: *mut ::core::ffi::c_char = vim_strchr(lbuf, TAB);
    if p.is_null() {
        return FAIL;
    }
    (*tagp).tagname_end = p;
    if *p as ::core::ffi::c_int != NUL {
        p = p.offset(1);
    }
    (*tagp).fname = p;
    p = vim_strchr(p, TAB);
    if p.is_null() {
        return FAIL;
    }
    (*tagp).fname_end = p;
    if *p as ::core::ffi::c_int != NUL {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    (*tagp).command = p;
    return OK;
}
unsafe extern "C" fn test_for_static(mut tagp: *mut tagptrs_T) -> bool {
    let mut p: *mut ::core::ffi::c_char = (*tagp).command;
    loop {
        p = vim_strchr(p, '\t' as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        p = p.offset(1);
        if strncmp(
            p,
            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
unsafe extern "C" fn matching_line_len(lbuf: *const ::core::ffi::c_char) -> size_t {
    let mut p: *const ::core::ffi::c_char = lbuf.offset(1 as ::core::ffi::c_int as isize);
    p = p.offset(strlen(p).wrapping_add(1 as size_t) as isize);
    return (p.offset_from(lbuf) as size_t).wrapping_add(strlen(p));
}
unsafe extern "C" fn parse_match(
    mut lbuf: *mut ::core::ffi::c_char,
    mut tagp: *mut tagptrs_T,
) -> ::core::ffi::c_int {
    (*tagp).tag_fname = lbuf.offset(1 as ::core::ffi::c_int as isize);
    lbuf = lbuf.offset(strlen((*tagp).tag_fname).wrapping_add(2 as size_t) as isize);
    let mut retval: ::core::ffi::c_int = parse_tag_line(lbuf, tagp);
    (*tagp).tagkind = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tagp).user_data = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tagp).tagline = 0 as ::core::ffi::c_int as linenr_T;
    (*tagp).command_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if retval != OK {
        return retval;
    }
    let mut p: *mut ::core::ffi::c_char = (*tagp).command;
    if find_extra(&raw mut p) == OK {
        (*tagp).command_end = p;
        if p > (*tagp).command
            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '|' as ::core::ffi::c_int
        {
            (*tagp).command_end = p.offset(-(1 as ::core::ffi::c_int as isize));
        }
        p = p.offset(2 as ::core::ffi::c_int as isize);
        let c2rust_fresh3 = p;
        p = p.offset(1);
        if *c2rust_fresh3 as ::core::ffi::c_int == TAB {
            while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || utfc_ptr2len(p) > 1 as ::core::ffi::c_int
            {
                if strncmp(
                    p,
                    b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).tagkind = p.offset(5 as ::core::ffi::c_int as isize);
                } else if strncmp(
                    p,
                    b"user_data:\0".as_ptr() as *const ::core::ffi::c_char,
                    10 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).user_data = p.offset(10 as ::core::ffi::c_int as isize);
                } else if strncmp(
                    p,
                    b"line:\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*tagp).tagline = atoi(p.offset(5 as ::core::ffi::c_int as isize)) as linenr_T;
                }
                if !(*tagp).tagkind.is_null() && !(*tagp).user_data.is_null() {
                    break;
                }
                let mut pc: *mut ::core::ffi::c_char = vim_strchr(p, ':' as ::core::ffi::c_int);
                let mut pt: *mut ::core::ffi::c_char = vim_strchr(p, '\t' as ::core::ffi::c_int);
                if pc.is_null() || !pt.is_null() && pc > pt {
                    (*tagp).tagkind = p;
                }
                if pt.is_null() {
                    break;
                }
                p = pt;
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        }
    }
    if !(*tagp).tagkind.is_null() {
        p = (*tagp).tagkind;
        while *p as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        (*tagp).tagkind_end = p;
    }
    if !(*tagp).user_data.is_null() {
        p = (*tagp).user_data;
        while *p as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != '\t' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        (*tagp).user_data_end = p;
    }
    return retval;
}
unsafe extern "C" fn tag_full_fname(mut tagp: *mut tagptrs_T) -> *mut ::core::ffi::c_char {
    let mut c: ::core::ffi::c_char = *(*tagp).fname_end;
    *(*tagp).fname_end = NUL as ::core::ffi::c_char;
    let mut fullname: *mut ::core::ffi::c_char =
        expand_tag_fname((*tagp).fname, (*tagp).tag_fname, false_0 != 0);
    *(*tagp).fname_end = c;
    return fullname;
}
unsafe extern "C" fn jumpto_tag(
    mut lbuf_arg: *const ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut keep_help: bool,
) -> ::core::ffi::c_int {
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if postponed_split == 0 as ::core::ffi::c_int && !check_can_set_curbuf_forceit(forceit) {
        return FAIL;
    }
    let mut pbuf_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tagp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut getfile_result: ::core::ffi::c_int = GETFILE_UNUSED as ::core::ffi::c_int;
    let mut search_options: ::core::ffi::c_int = 0;
    let mut curwin_save: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut full_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let old_KeyTyped: bool = KeyTyped;
    let l_g_do_tagpreview: ::core::ffi::c_int = g_do_tagpreview;
    let len: size_t = matching_line_len(lbuf_arg).wrapping_add(1 as size_t);
    let mut lbuf: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    memmove(
        lbuf as *mut ::core::ffi::c_void,
        lbuf_arg as *const ::core::ffi::c_void,
        len,
    );
    let mut pbuf: *mut ::core::ffi::c_char =
        xmalloc(LSIZE as ::core::ffi::c_int as size_t) as *mut ::core::ffi::c_char;
    '_erret: {
        if parse_match(lbuf, &raw mut tagp) == FAIL {
            tagp.fname_end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            *tagp.fname_end = NUL as ::core::ffi::c_char;
            fname = tagp.fname;
            str = tagp.command;
            pbuf_end = pbuf;
            while *str as ::core::ffi::c_int != 0
                && *str as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                && *str as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            {
                let c2rust_fresh1 = str;
                str = str.offset(1);
                let c2rust_fresh2 = pbuf_end;
                pbuf_end = pbuf_end.offset(1);
                *c2rust_fresh2 = *c2rust_fresh1;
                if pbuf_end.offset_from(pbuf) + 1 as isize >= LSIZE as ::core::ffi::c_int as isize {
                    break;
                }
            }
            *pbuf_end = NUL as ::core::ffi::c_char;
            str = pbuf;
            if find_extra(&raw mut str) == OK {
                pbuf_end = str;
                *pbuf_end = NUL as ::core::ffi::c_char;
            }
            fname = expand_tag_fname(fname, tagp.tag_fname, true_0 != 0);
            tofree_fname = fname;
            if !os_path_exists(fname)
                && !has_autocmd(EVENT_BUFREADCMD, fname, ::core::ptr::null_mut::<buf_T>())
            {
                retval = NOTAGFILE;
                xfree(nofile_fname.get() as *mut ::core::ffi::c_void);
                nofile_fname.set(xstrdup(fname));
            } else {
                RedrawingDisabled += 1;
                if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                    postponed_split = 0 as ::core::ffi::c_int;
                    curwin_save = curwin;
                    if (*curwin).w_onebuf_opt.wo_pvw == 0 {
                        full_fname = FullName_save(fname, false_0 != 0);
                        fname = full_fname;
                        prepare_tagpreview(true_0 != 0);
                    }
                }
                if postponed_split != 0
                    && swb_flags
                        & (kOptSwbFlagUseopen as ::core::ffi::c_int
                            | kOptSwbFlagUsetab as ::core::ffi::c_int)
                            as ::core::ffi::c_uint
                        != 0
                {
                    let existing_buf: *mut buf_T = buflist_findname_exp(fname);
                    if !existing_buf.is_null() {
                        if !swbuf_goto_win_with_buf(existing_buf).is_null() {
                            getfile_result = GETFILE_SAME_FILE as ::core::ffi::c_int;
                        }
                    }
                }
                if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int
                    && (postponed_split != 0 || cmdmod.cmod_tab != 0 as ::core::ffi::c_int)
                {
                    if swb_flags & kOptSwbFlagVsplit as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        cmdmod.cmod_split |= WSP_VERT as ::core::ffi::c_int;
                    }
                    if swb_flags & kOptSwbFlagNewtab as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                        && cmdmod.cmod_tab == 0 as ::core::ffi::c_int
                    {
                        cmdmod.cmod_tab = tabpage_index(curtab) + 1 as ::core::ffi::c_int;
                    }
                    if win_split(
                        if postponed_split > 0 as ::core::ffi::c_int {
                            postponed_split
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        postponed_split_flags,
                    ) == FAIL
                    {
                        RedrawingDisabled -= 1;
                        break '_erret;
                    } else {
                        (*curwin).w_onebuf_opt.wo_scb = false_0;
                        (*curwin).w_onebuf_opt.wo_crb = false_0;
                    }
                }
                if keep_help {
                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int {
                        keep_help_flag = bt_help((*curwin_save).w_buffer);
                    } else {
                        keep_help_flag = (*curbuf).b_help;
                    }
                }
                if getfile_result == GETFILE_UNUSED as ::core::ffi::c_int {
                    getfile_result = getfile(
                        0 as ::core::ffi::c_int,
                        fname,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        true_0 != 0,
                        0 as linenr_T,
                        forceit != 0,
                    );
                }
                keep_help_flag = false_0 != 0;
                if getfile_result <= 0 as ::core::ffi::c_int {
                    (*curwin).w_set_curswant = true_0;
                    postponed_split = 0 as ::core::ffi::c_int;
                    let save_magic_overruled: optmagic_T = magic_overruled;
                    magic_overruled = OPTION_MAGIC_OFF;
                    let save_no_hlsearch: bool = no_hlsearch;
                    if !vim_strchr(p_cpo, CPO_TAGPAT).is_null() {
                        search_options = 0 as ::core::ffi::c_int;
                    } else {
                        search_options = SEARCH_KEEP as ::core::ffi::c_int;
                    }
                    str = pbuf;
                    if *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                        || *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '?' as ::core::ffi::c_int
                    {
                        str = skip_regexp(
                            pbuf.offset(1 as ::core::ffi::c_int as isize),
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            false_0,
                        )
                        .offset(1 as ::core::ffi::c_int as isize);
                    }
                    if str > pbuf_end.offset(-(1 as ::core::ffi::c_int as isize)) {
                        let mut pbuflen: size_t = pbuf_end.offset_from(pbuf) as size_t;
                        let mut save_p_ws: bool = p_ws != 0;
                        let mut save_p_ic: ::core::ffi::c_int = p_ic;
                        let mut save_p_scs: ::core::ffi::c_int = p_scs;
                        p_ws = true_0;
                        p_ic = false_0;
                        p_scs = false_0;
                        let mut save_lnum: linenr_T = (*curwin).w_cursor.lnum;
                        (*curwin).w_cursor.lnum = if tagp.tagline > 0 as linenr_T {
                            tagp.tagline - 1 as linenr_T
                        } else {
                            0 as linenr_T
                        };
                        if do_search(
                            ::core::ptr::null_mut::<oparg_T>(),
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            *pbuf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                            pbuf.offset(1 as ::core::ffi::c_int as isize),
                            pbuflen.wrapping_sub(1 as size_t),
                            1 as ::core::ffi::c_int,
                            search_options,
                            ::core::ptr::null_mut::<searchit_arg_T>(),
                        ) != 0
                        {
                            retval = OK;
                        } else {
                            let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            p_ic = true_0;
                            if do_search(
                                ::core::ptr::null_mut::<oparg_T>(),
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                *pbuf.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int,
                                pbuf.offset(1 as ::core::ffi::c_int as isize),
                                pbuflen.wrapping_sub(1 as size_t),
                                1 as ::core::ffi::c_int,
                                search_options,
                                ::core::ptr::null_mut::<searchit_arg_T>(),
                            ) == 0
                            {
                                found = 2 as ::core::ffi::c_int;
                                test_for_static(&raw mut tagp);
                                let mut cc: ::core::ffi::c_char = *tagp.tagname_end;
                                *tagp.tagname_end = NUL as ::core::ffi::c_char;
                                pbuflen = snprintf(
                                    pbuf,
                                    LSIZE as ::core::ffi::c_int as size_t,
                                    b"^%s\\s\\*(\0".as_ptr() as *const ::core::ffi::c_char,
                                    tagp.tagname,
                                ) as size_t;
                                if do_search(
                                    ::core::ptr::null_mut::<oparg_T>(),
                                    '/' as ::core::ffi::c_int,
                                    '/' as ::core::ffi::c_int,
                                    pbuf,
                                    pbuflen,
                                    1 as ::core::ffi::c_int,
                                    search_options,
                                    ::core::ptr::null_mut::<searchit_arg_T>(),
                                ) == 0
                                {
                                    pbuflen = snprintf(
                                        pbuf,
                                        LSIZE as ::core::ffi::c_int as size_t,
                                        b"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        tagp.tagname,
                                    ) as size_t;
                                    if do_search(
                                        ::core::ptr::null_mut::<oparg_T>(),
                                        '/' as ::core::ffi::c_int,
                                        '/' as ::core::ffi::c_int,
                                        pbuf,
                                        pbuflen,
                                        1 as ::core::ffi::c_int,
                                        search_options,
                                        ::core::ptr::null_mut::<searchit_arg_T>(),
                                    ) == 0
                                    {
                                        found = 0 as ::core::ffi::c_int;
                                    }
                                }
                                *tagp.tagname_end = cc;
                            }
                            if found == 0 as ::core::ffi::c_int {
                                emsg(gettext(b"E434: Can't find tag pattern\0".as_ptr()
                                    as *const ::core::ffi::c_char));
                                (*curwin).w_cursor.lnum = save_lnum;
                            } else {
                                if found == 2 as ::core::ffi::c_int || save_p_ic == 0 {
                                    msg(
                                        gettext(
                                            b"E435: Couldn't find tag, just guessing!\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        0 as ::core::ffi::c_int,
                                    );
                                    if msg_scrolled == 0 && msg_silent == 0 as ::core::ffi::c_int {
                                        msg_delay(1010 as uint64_t, true_0 != 0);
                                    }
                                }
                                retval = OK;
                            }
                        }
                        p_ws = save_p_ws as ::core::ffi::c_int;
                        p_ic = save_p_ic;
                        p_scs = save_p_scs;
                        check_cursor(curwin);
                    } else {
                        let save_secure: ::core::ffi::c_int = secure;
                        secure = 1 as ::core::ffi::c_int;
                        sandbox += 1;
                        (*curwin).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                        (*curwin).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                        (*curwin).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        do_cmdline_cmd(pbuf);
                        retval = OK;
                        if secure == 2 as ::core::ffi::c_int {
                            wait_return(true_0);
                        }
                        secure = save_secure;
                        sandbox -= 1;
                    }
                    magic_overruled = save_magic_overruled;
                    if search_options != 0 {
                        set_no_hlsearch(save_no_hlsearch);
                    }
                    if getfile_result == GETFILE_OPEN_OTHER as ::core::ffi::c_int {
                        retval = OK;
                    }
                    if retval == OK {
                        if (*curbuf).b_help {
                            set_topline(curwin, (*curwin).w_cursor.lnum);
                        }
                        if fdo_flags & kOptFdoFlagTag as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0
                            && old_KeyTyped as ::core::ffi::c_int != 0
                        {
                            foldOpenCursor();
                        }
                    }
                    if l_g_do_tagpreview != 0 as ::core::ffi::c_int
                        && curwin != curwin_save
                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                    {
                        validate_cursor(curwin);
                        redraw_later(curwin, UPD_VALID as ::core::ffi::c_int);
                        win_enter(curwin_save, true_0 != 0);
                    }
                    RedrawingDisabled -= 1;
                } else {
                    RedrawingDisabled -= 1;
                    if postponed_split != 0 {
                        win_close(curwin, false_0 != 0, false_0 != 0);
                        postponed_split = 0 as ::core::ffi::c_int;
                    }
                }
            }
        }
    }
    g_do_tagpreview = 0 as ::core::ffi::c_int;
    xfree(lbuf as *mut ::core::ffi::c_void);
    xfree(pbuf as *mut ::core::ffi::c_void);
    xfree(tofree_fname as *mut ::core::ffi::c_void);
    xfree(full_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn expand_tag_fname(
    mut fname: *mut ::core::ffi::c_char,
    tag_fname: *mut ::core::ffi::c_char,
    expand: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded_fname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
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
    if expand as ::core::ffi::c_int != 0
        && path_has_wildcard(fname) as ::core::ffi::c_int != 0
        && vim_strchr(fname, '`' as ::core::ffi::c_int).is_null()
    {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
        expanded_fname = ExpandOne(
            &raw mut xpc,
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            WILD_LIST_NOTFOUND as ::core::ffi::c_int | WILD_SILENT as ::core::ffi::c_int,
            WILD_EXPAND_FREE as ::core::ffi::c_int,
        );
        if !expanded_fname.is_null() {
            fname = expanded_fname;
        }
    }
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (p_tr != 0 || (*curbuf).b_help as ::core::ffi::c_int != 0) && !vim_isAbsName(fname) && {
        p = path_tail(tag_fname);
        p != tag_fname
    } {
        retval = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        strcpy(retval, tag_fname);
        xstrlcpy(
            retval.offset(p.offset_from(tag_fname) as isize),
            fname,
            (MAXPATHL as isize - p.offset_from(tag_fname)) as size_t,
        );
        simplify_filename(retval);
    } else {
        retval = xstrdup(fname);
    }
    xfree(expanded_fname as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn test_for_current(
    mut fname: *mut ::core::ffi::c_char,
    mut fname_end: *mut ::core::ffi::c_char,
    mut tag_fname: *mut ::core::ffi::c_char,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = false_0;
    if !buf_ffname.is_null() {
        let mut c: ::core::ffi::c_char = 0;
        c = *fname_end;
        *fname_end = NUL as ::core::ffi::c_char;
        let mut fullname: *mut ::core::ffi::c_char =
            expand_tag_fname(fname, tag_fname, true_0 != 0);
        retval = (path_full_compare(fullname, buf_ffname, true_0 != 0, true_0 != 0)
            as ::core::ffi::c_uint
            & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int;
        xfree(fullname as *mut ::core::ffi::c_void);
        *fname_end = c;
    }
    return retval;
}
unsafe extern "C" fn find_extra(mut pp: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut str: *mut ::core::ffi::c_char = *pp;
    let mut first_char: ::core::ffi::c_char = **pp;
    loop {
        if ascii_isdigit(*str as ::core::ffi::c_int) {
            str = skipdigits(str.offset(1 as ::core::ffi::c_int as isize));
        } else if *str as ::core::ffi::c_int == '/' as ::core::ffi::c_int
            || *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int
        {
            str = skip_regexp(
                str.offset(1 as ::core::ffi::c_int as isize),
                *str as ::core::ffi::c_int,
                false_0,
            );
            if *str as ::core::ffi::c_int != first_char as ::core::ffi::c_int {
                str = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                str = str.offset(1);
            }
        } else {
            str = strstr(str, b"|;\"\0".as_ptr() as *const ::core::ffi::c_char);
            if !str.is_null() {
                str = str.offset(1);
                break;
            }
        }
        if str.is_null()
            || *str as ::core::ffi::c_int != ';' as ::core::ffi::c_int
            || !(ascii_isdigit(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                || *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int)
        {
            break;
        }
        str = str.offset(1);
        first_char = *str;
    }
    if !str.is_null()
        && strncmp(
            str,
            b";\"\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        *pp = str;
        return OK;
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn tagstack_clear_entry(mut item: *mut taggy_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*item).tagname as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*item).user_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
}
#[no_mangle]
pub unsafe extern "C" fn expand_tags(
    mut tagnames: bool,
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut name_buf_size: size_t = 100 as size_t;
    let mut ret: ::core::ffi::c_int = 0;
    let mut name_buf: *mut ::core::ffi::c_char = xmalloc(name_buf_size) as *mut ::core::ffi::c_char;
    let mut extra_flag: ::core::ffi::c_int = if tagnames as ::core::ffi::c_int != 0 {
        TAG_NAMES as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '/' as ::core::ffi::c_int
    {
        ret = find_tags(
            pat.offset(1 as ::core::ffi::c_int as isize),
            num_file,
            file,
            TAG_REGEXP as ::core::ffi::c_int
                | extra_flag
                | TAG_VERBOSE as ::core::ffi::c_int
                | TAG_NO_TAGFUNC as ::core::ffi::c_int,
            TAG_MANY as ::core::ffi::c_int,
            (*curbuf).b_ffname,
        );
    } else {
        ret = find_tags(
            pat,
            num_file,
            file,
            TAG_REGEXP as ::core::ffi::c_int
                | extra_flag
                | TAG_VERBOSE as ::core::ffi::c_int
                | TAG_NO_TAGFUNC as ::core::ffi::c_int
                | TAG_NOIC as ::core::ffi::c_int,
            TAG_MANY as ::core::ffi::c_int,
            (*curbuf).b_ffname,
        );
    }
    if ret == OK && !tagnames {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < *num_file {
            let mut t_p: tagptrs_T = tagptrs_T {
                tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                tagline: 0,
            };
            parse_match(*(*file).offset(i as isize), &raw mut t_p);
            let mut len: size_t = t_p.tagname_end.offset_from(t_p.tagname) as size_t;
            if len > name_buf_size.wrapping_sub(3 as size_t) {
                name_buf_size = len.wrapping_add(3 as size_t);
                let mut buf: *mut ::core::ffi::c_char =
                    xrealloc(name_buf as *mut ::core::ffi::c_void, name_buf_size)
                        as *mut ::core::ffi::c_char;
                name_buf = buf;
            }
            memmove(
                name_buf as *mut ::core::ffi::c_void,
                t_p.tagname as *const ::core::ffi::c_void,
                len,
            );
            let c2rust_fresh14 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh14 as isize) = 0 as ::core::ffi::c_char;
            let c2rust_fresh15 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh15 as isize) =
                (if !t_p.tagkind.is_null() && *t_p.tagkind as ::core::ffi::c_int != 0 {
                    *t_p.tagkind as ::core::ffi::c_int
                } else {
                    'f' as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            let c2rust_fresh16 = len;
            len = len.wrapping_add(1);
            *name_buf.offset(c2rust_fresh16 as isize) = 0 as ::core::ffi::c_char;
            memmove(
                (*(*file).offset(i as isize)).offset(len as isize) as *mut ::core::ffi::c_void,
                t_p.fname as *const ::core::ffi::c_void,
                t_p.fname_end.offset_from(t_p.fname) as size_t,
            );
            *(*(*file).offset(i as isize)).offset(
                len.wrapping_add(t_p.fname_end.offset_from(t_p.fname) as size_t) as isize,
            ) = 0 as ::core::ffi::c_char;
            memmove(
                *(*file).offset(i as isize) as *mut ::core::ffi::c_void,
                name_buf as *const ::core::ffi::c_void,
                len,
            );
            i += 1;
        }
    }
    xfree(name_buf as *mut ::core::ffi::c_void);
    return ret;
}
unsafe extern "C" fn add_tag_field(
    mut dict: *mut dict_T,
    mut field_name: *const ::core::ffi::c_char,
    mut start: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if !tv_dict_find(dict, field_name, -1 as ptrdiff_t).is_null() {
        if p_verbose > 0 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Duplicate field name: %s\0".as_ptr() as *const ::core::ffi::c_char),
                field_name,
            );
            verbose_leave();
        }
        return FAIL;
    }
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if !start.is_null() {
        if end.is_null() {
            end = start.offset(strlen(start) as isize);
            while end > start
                && (*end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\r' as ::core::ffi::c_int
                    || *end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int)
            {
                end = end.offset(-1);
            }
        }
        len = if (end.offset_from(start) as ::core::ffi::c_int)
            < 4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            end.offset_from(start) as ::core::ffi::c_int
        } else {
            4096 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        };
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            start as *const ::core::ffi::c_void,
            len as size_t,
        );
    }
    *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
    let mut retval: ::core::ffi::c_int = tv_dict_add_str(dict, field_name, strlen(field_name), buf);
    xfree(buf as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn get_tags(
    mut list: *mut list_T,
    mut pat: *mut ::core::ffi::c_char,
    mut buf_fname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut num_matches: ::core::ffi::c_int = 0;
    let mut matches: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tp: tagptrs_T = tagptrs_T {
        tagname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        command_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tag_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagkind_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        user_data_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tagline: 0,
    };
    let mut ret: ::core::ffi::c_int = find_tags(
        pat,
        &raw mut num_matches,
        &raw mut matches,
        TAG_REGEXP as ::core::ffi::c_int | TAG_NOIC as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
        buf_fname,
    );
    if ret != OK || num_matches <= 0 as ::core::ffi::c_int {
        return ret;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_matches {
        if parse_match(*matches.offset(i as isize), &raw mut tp) == FAIL {
            xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
        } else {
            let mut is_static: bool = test_for_static(&raw mut tp);
            if strncmp(
                tp.tagname,
                b"!_TAG_\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
            } else {
                let mut dict: *mut dict_T = tv_dict_alloc();
                tv_list_append_dict(list, dict);
                let mut full_fname: *mut ::core::ffi::c_char = tag_full_fname(&raw mut tp);
                if add_tag_field(
                    dict,
                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                    tp.tagname,
                    tp.tagname_end,
                ) == FAIL
                    || add_tag_field(
                        dict,
                        b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                        full_fname,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    ) == FAIL
                    || add_tag_field(
                        dict,
                        b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                        tp.command,
                        tp.command_end,
                    ) == FAIL
                    || add_tag_field(
                        dict,
                        b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                        tp.tagkind,
                        if !tp.tagkind.is_null() {
                            tp.tagkind_end
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        },
                    ) == FAIL
                    || tv_dict_add_nr(
                        dict,
                        b"static\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                        is_static as varnumber_T,
                    ) == FAIL
                {
                    ret = FAIL;
                }
                xfree(full_fname as *mut ::core::ffi::c_void);
                if !tp.command_end.is_null() {
                    let mut p: *mut ::core::ffi::c_char =
                        tp.command_end.offset(3 as ::core::ffi::c_int as isize);
                    while *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
                    {
                        if p == tp.tagkind
                            || p.offset(5 as ::core::ffi::c_int as isize) == tp.tagkind
                                && strncmp(
                                    p,
                                    b"kind:\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as size_t,
                                ) == 0 as ::core::ffi::c_int
                        {
                            p = tp.tagkind_end.offset(-(1 as ::core::ffi::c_int as isize));
                        } else if strncmp(
                            p,
                            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                            5 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        {
                            p = p.offset(4 as ::core::ffi::c_int as isize);
                        } else if !ascii_iswhite(*p as ::core::ffi::c_int) {
                            let mut len: ::core::ffi::c_int = 0;
                            let mut n: *mut ::core::ffi::c_char = p;
                            while *p as ::core::ffi::c_int != NUL
                                && *p as ::core::ffi::c_int >= ' ' as ::core::ffi::c_int
                                && (*p as ::core::ffi::c_int) < 127 as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                            }
                            len = p.offset_from(n) as ::core::ffi::c_int;
                            if *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                                && len > 0 as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                                let mut s: *mut ::core::ffi::c_char = p;
                                while *p as ::core::ffi::c_int != NUL
                                    && *p as uint8_t as ::core::ffi::c_int
                                        >= ' ' as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                }
                                *n.offset(len as isize) = NUL as ::core::ffi::c_char;
                                if add_tag_field(dict, n, s, p) == FAIL {
                                    ret = FAIL;
                                }
                                *n.offset(len as isize) = ':' as ::core::ffi::c_char;
                            } else {
                                while *p as ::core::ffi::c_int != NUL
                                    && *p as uint8_t as ::core::ffi::c_int
                                        >= ' ' as ::core::ffi::c_int
                                {
                                    p = p.offset(1);
                                }
                            }
                            if *p as ::core::ffi::c_int == NUL {
                                break;
                            }
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                }
                xfree(*matches.offset(i as isize) as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
    xfree(matches as *mut ::core::ffi::c_void);
    return ret;
}
unsafe extern "C" fn get_tag_details(mut tag: *mut taggy_T, mut retdict: *mut dict_T) {
    tv_dict_add_str(
        retdict,
        b"tagname\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*tag).tagname,
    );
    tv_dict_add_nr(
        retdict,
        b"matchnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ((*tag).cur_match + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        retdict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*tag).cur_fnum as varnumber_T,
    );
    if !(*tag).user_data.is_null() {
        tv_dict_add_str(
            retdict,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            (*tag).user_data,
        );
    }
    let mut pos: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
    tv_dict_add_list(
        retdict,
        b"from\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        pos,
    );
    let mut fmark: *mut fmark_T = &raw mut (*tag).fmark;
    tv_list_append_number(
        pos,
        (if (*fmark).fnum != -1 as ::core::ffi::c_int {
            (*fmark).fnum
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(pos, (*fmark).mark.lnum as varnumber_T);
    tv_list_append_number(
        pos,
        (if (*fmark).mark.col == MAXCOL as ::core::ffi::c_int {
            MAXCOL as ::core::ffi::c_int
        } else {
            (*fmark).mark.col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(pos, (*fmark).mark.coladd as varnumber_T);
}
#[no_mangle]
pub unsafe extern "C" fn get_tagstack(mut wp: *mut win_T, mut retdict: *mut dict_T) {
    tv_dict_add_nr(
        retdict,
        b"length\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_tagstacklen as varnumber_T,
    );
    tv_dict_add_nr(
        retdict,
        b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_tagstackidx + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    let mut l: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
    tv_dict_add_list(
        retdict,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        let mut d: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(l, d);
        get_tag_details(
            (&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize),
            d,
        );
        i += 1;
    }
}
unsafe extern "C" fn tagstack_clear(mut wp: *mut win_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
        i += 1;
    }
    (*wp).w_tagstacklen = 0 as ::core::ffi::c_int;
    (*wp).w_tagstackidx = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tagstack_shift(mut wp: *mut win_T) {
    let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
    tagstack_clear_entry(tagstack.offset(0 as ::core::ffi::c_int as isize));
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < (*wp).w_tagstacklen {
        *tagstack.offset((i - 1 as ::core::ffi::c_int) as isize) = *tagstack.offset(i as isize);
        i += 1;
    }
    (*wp).w_tagstacklen -= 1;
}
unsafe extern "C" fn tagstack_push_item(
    mut wp: *mut win_T,
    mut tagname: *mut ::core::ffi::c_char,
    mut cur_fnum: ::core::ffi::c_int,
    mut cur_match: ::core::ffi::c_int,
    mut mark: pos_T,
    mut fnum: ::core::ffi::c_int,
    mut user_data: *mut ::core::ffi::c_char,
) {
    let mut tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
    let mut idx: ::core::ffi::c_int = (*wp).w_tagstacklen;
    if idx >= TAGSTACKSIZE {
        tagstack_shift(wp);
        idx = TAGSTACKSIZE - 1 as ::core::ffi::c_int;
    }
    (*wp).w_tagstacklen += 1;
    (*tagstack.offset(idx as isize)).tagname = tagname;
    (*tagstack.offset(idx as isize)).cur_fnum = cur_fnum;
    (*tagstack.offset(idx as isize)).cur_match = cur_match;
    (*tagstack.offset(idx as isize)).cur_match =
        if (*tagstack.offset(idx as isize)).cur_match > 0 as ::core::ffi::c_int {
            (*tagstack.offset(idx as isize)).cur_match
        } else {
            0 as ::core::ffi::c_int
        };
    (*tagstack.offset(idx as isize)).fmark.mark = mark;
    (*tagstack.offset(idx as isize)).fmark.fnum = fnum;
    (*tagstack.offset(idx as isize)).fmark.view = fmarkv_T {
        topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
        skipcol: 0 as colnr_T,
    };
    (*tagstack.offset(idx as isize)).user_data = user_data;
}
unsafe extern "C" fn tagstack_push_items(mut wp: *mut win_T, mut l: *mut list_T) {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut tagname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mark: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut fnum: ::core::ffi::c_int = 0;
    let mut li: *mut listitem_T = tv_list_first(l);
    while !li.is_null() {
        if !((*li).li_tv.v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*li).li_tv.vval.v_dict.is_null())
        {
            let mut itemdict: *mut dict_T = (*li).li_tv.vval.v_dict;
            di = tv_dict_find(
                itemdict,
                b"from\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                if list2fpos(
                    &raw mut (*di).di_tv,
                    &raw mut mark,
                    &raw mut fnum,
                    ::core::ptr::null_mut::<colnr_T>(),
                    false_0 != 0,
                ) == OK
                {
                    tagname = tv_dict_get_string(
                        itemdict,
                        b"tagname\0".as_ptr() as *const ::core::ffi::c_char,
                        true_0 != 0,
                    );
                    if !tagname.is_null() {
                        if mark.col > 0 as ::core::ffi::c_int {
                            mark.col -= 1;
                        }
                        tagstack_push_item(
                            wp,
                            tagname,
                            tv_dict_get_number(
                                itemdict,
                                b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                            ) as ::core::ffi::c_int,
                            tv_dict_get_number(
                                itemdict,
                                b"matchnr\0".as_ptr() as *const ::core::ffi::c_char,
                            ) as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int,
                            mark,
                            fnum,
                            tv_dict_get_string(
                                itemdict,
                                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                                true_0 != 0,
                            ),
                        );
                    }
                }
            }
        }
        li = (*li).li_next;
    }
}
unsafe extern "C" fn tagstack_set_curidx(mut wp: *mut win_T, mut curidx: ::core::ffi::c_int) {
    (*wp).w_tagstackidx = curidx;
    (*wp).w_tagstackidx = if (if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
        (*wp).w_tagstackidx
    } else {
        0 as ::core::ffi::c_int
    }) < (*wp).w_tagstacklen
    {
        if (*wp).w_tagstackidx > 0 as ::core::ffi::c_int {
            (*wp).w_tagstackidx
        } else {
            0 as ::core::ffi::c_int
        }
    } else {
        (*wp).w_tagstacklen
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_tagstack(
    mut wp: *mut win_T,
    mut d: *const dict_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if tfu_in_use.get() {
        emsg(gettext(
            (e_cannot_modify_tag_stack_within_tagfunc.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    di = tv_dict_find(
        d,
        b"items\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return FAIL;
        }
        l = (*di).di_tv.vval.v_list;
    }
    di = tv_dict_find(
        d,
        b"curidx\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        tagstack_set_curidx(
            wp,
            tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        );
    }
    if action == 't' as ::core::ffi::c_int {
        let tagstack: *mut taggy_T = &raw mut (*wp).w_tagstack as *mut taggy_T;
        let tagstackidx: ::core::ffi::c_int = (*wp).w_tagstackidx;
        let mut tagstacklen: ::core::ffi::c_int = (*wp).w_tagstacklen;
        while tagstackidx < tagstacklen {
            tagstacklen -= 1;
            tagstack_clear_entry(tagstack.offset(tagstacklen as isize));
        }
        (*wp).w_tagstacklen = tagstacklen;
    }
    if !l.is_null() {
        if action == 'r' as ::core::ffi::c_int {
            tagstack_clear(wp);
        }
        tagstack_push_items(wp, l);
        (*wp).w_tagstackidx = (*wp).w_tagstacklen;
    }
    return OK;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
