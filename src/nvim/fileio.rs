use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type __dirstream;
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
    fn iconv_close(__cd: iconv_t) -> ::core::ffi::c_int;
    fn iconv(
        __cd: iconv_t,
        __inbuf: *mut *mut ::core::ffi::c_char,
        __inbytesleft: *mut size_t,
        __outbuf: *mut *mut ::core::ffi::c_char,
        __outbytesleft: *mut size_t,
    ) -> size_t;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fgets(
        __s: *mut ::core::ffi::c_char,
        __n: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> *mut ::core::ffi::c_char;
    fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn feof(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn umask(__mask: __mode_t) -> __mode_t;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn closedir(__dirp: *mut DIR) -> ::core::ffi::c_int;
    fn opendir(__name: *const ::core::ffi::c_char) -> *mut DIR;
    fn dirfd(__dirp: *mut DIR) -> ::core::ffi::c_int;
    fn lseek(__fd: ::core::ffi::c_int, __offset: __off_t, __whence: ::core::ffi::c_int) -> __off_t;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn read(__fd: ::core::ffi::c_int, __buf: *mut ::core::ffi::c_void, __nbytes: size_t)
        -> ssize_t;
    fn write(__fd: ::core::ffi::c_int, __buf: *const ::core::ffi::c_void, __n: size_t) -> ssize_t;
    fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn symlink(
        __from: *const ::core::ffi::c_char,
        __to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn readlink(
        __path: *const ::core::ffi::c_char,
        __buf: *mut ::core::ffi::c_char,
        __len: size_t,
    ) -> ssize_t;
    fn verbose_try_malloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn memchrsub(
        data: *mut ::core::ffi::c_void,
        c: ::core::ffi::c_char,
        x: ::core::ffi::c_char,
        len: size_t,
    );
    fn xstrlcat(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn time_to_bytes(time_: time_t, buf: *mut uint8_t);
    static mut autocmd_busy: bool;
    fn augroup_exists(name: *const ::core::ffi::c_char) -> bool;
    fn do_doautocmd(
        arg_start: *mut ::core::ffi::c_char,
        do_msg: bool,
        did_something: *mut bool,
    ) -> ::core::ffi::c_int;
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn apply_autocmds_exarg(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
        eap: *mut exarg_T,
    ) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn ngettext(
        __msgid1: *const ::core::ffi::c_char,
        __msgid2: *const ::core::ffi::c_char,
        __n: ::core::ffi::c_ulong,
    ) -> *mut ::core::ffi::c_char;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
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
    fn bt_normal(buf: *const buf_T) -> bool;
    fn bt_nofilename(buf: *const buf_T) -> bool;
    fn bt_dontwrite(buf: *const buf_T) -> bool;
    fn buf_contents_changed(buf: *mut buf_T) -> bool;
    fn wipe_buffer(buf: *mut buf_T, aucmd: bool);
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn buf_updates_unload(buf: *mut buf_T, can_reload: bool);
    fn appended_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn unchanged(buf: *mut buf_T, ff: bool, always_inc_changedtick: bool);
    fn save_file_ff(buf: *mut buf_T);
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn diff_invalidate(buf: *mut buf_T);
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn redraw_curbuf_later(type_0: ::core::ffi::c_int);
    fn status_redraw_all();
    fn beginline(flags: ::core::ffi::c_int);
    static e_interr: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    fn eval_charconvert(
        enc_from: *const ::core::ffi::c_char,
        enc_to: *const ::core::ffi::c_char,
        fname_from: *const ::core::ffi::c_char,
        fname_to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn aborting() -> bool;
    fn foldmethodIsManual(wp: *mut win_T) -> bool;
    fn foldUpdateAll(win: *mut win_T);
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn stuff_empty() -> bool;
    fn typebuf_typed() -> ::core::ffi::c_int;
    fn enc_canon_props(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_byte2len(b: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2len_len(
        p: *const ::core::ffi::c_char,
        size: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn enc_canonize(enc: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn my_iconv_open(
        to: *mut ::core::ffi::c_char,
        from: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    static mut redraw_cmdline: bool;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_scrolled: ::core::ffi::c_int;
    static mut msg_scrolled_ign: bool;
    static mut keep_msg: *mut ::core::ffi::c_char;
    static mut need_fileinfo: bool;
    static mut msg_scroll: ::core::ffi::c_int;
    static mut no_wait_return: ::core::ffi::c_int;
    static mut need_wait_return: bool;
    static mut need_check_timestamps: bool;
    static mut did_check_timestamps: bool;
    static mut no_check_timestamps: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut redraw_tabline: bool;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut exiting: bool;
    static mut stdin_fd: ::core::ffi::c_int;
    static mut allbuf_lock: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut exmode_active: bool;
    static mut ex_no_reprint: bool;
    static mut restart_edit: ::core::ffi::c_int;
    static mut cmdmod: cmdmod_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut emsg_silent: ::core::ffi::c_int;
    static mut in_assert_fails: bool;
    static mut swap_exists_action: ::core::ffi::c_int;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut readonlymode: bool;
    static mut recoverymode: bool;
    static mut got_int: bool;
    static mut global_busy: ::core::ffi::c_int;
    static mut vim_ignored: ::core::ffi::c_int;
    fn mf_fullname(mfp: *mut memfile_T);
    static mut msg_listdo_overwrite: ::core::ffi::c_int;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_schedule_semsg(fmt: *const ::core::ffi::c_char, ...);
    fn msg_trunc(
        s: *mut ::core::ffi::c_char,
        force: bool,
        hl_id: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn msg_may_trunc(force: bool, s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn msg_progress(
        s: *mut ::core::ffi::c_char,
        id: *mut ::core::ffi::c_char,
        status: *mut ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
        trunc: bool,
    ) -> *mut ::core::ffi::c_char;
    fn set_keep_msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_clr_eos();
    fn msg_end() -> bool;
    fn do_dialog(
        type_0: ::core::ffi::c_int,
        title: *const ::core::ffi::c_char,
        message: *const ::core::ffi::c_char,
        buttons: *const ::core::ffi::c_char,
        dfltbutton: ::core::ffi::c_int,
        textfield: *const ::core::ffi::c_char,
        ex_cmd: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn msg_delay(ms: uint64_t, ignoreinput: bool);
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn update_topline(wp: *mut win_T);
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
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
    fn shortmess(x: ::core::ffi::c_int) -> bool;
    fn get_fileformat(buf: *const buf_T) -> ::core::ffi::c_int;
    fn get_fileformat_force(buf: *const buf_T, eap: *const exarg_T) -> ::core::ffi::c_int;
    fn default_fileformat() -> ::core::ffi::c_int;
    fn set_fileformat(eol_style: ::core::ffi::c_int, opt_flags: ::core::ffi::c_int);
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    static mut p_ar: ::core::ffi::c_int;
    static mut p_enc: *mut ::core::ffi::c_char;
    static mut p_ccv: *mut ::core::ffi::c_char;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_fencs: *mut ::core::ffi::c_char;
    static mut p_ffs: *mut ::core::ffi::c_char;
    static mut p_fic: ::core::ffi::c_int;
    static mut p_ur: OptInt;
    static mut p_verbose: OptInt;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isrealdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_open(
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_copy(
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_getperm(name: *const ::core::ffi::c_char) -> int32_t;
    fn os_setperm(name: *const ::core::ffi::c_char, perm: ::core::ffi::c_int)
        -> ::core::ffi::c_int;
    fn os_get_acl(fname: *const ::core::ffi::c_char) -> vim_acl_T;
    fn os_set_acl(fname: *const ::core::ffi::c_char, aclent: vim_acl_T);
    fn os_free_acl(aclent: vim_acl_T);
    fn os_file_owned(fname: *const ::core::ffi::c_char) -> bool;
    fn os_fchown(fd: ::core::ffi::c_int, owner: uv_uid_t, group: uv_gid_t) -> ::core::ffi::c_int;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_file_is_writable(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_rename(
        path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn os_mkdir(path: *const ::core::ffi::c_char, mode: int32_t) -> ::core::ffi::c_int;
    fn os_mkdtemp(
        templ: *const ::core::ffi::c_char,
        path: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn os_rmdir(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_scandir(dir: *mut Directory, path: *const ::core::ffi::c_char) -> bool;
    fn os_scandir_next(dir: *mut Directory) -> *const ::core::ffi::c_char;
    fn os_closedir(dir: *mut Directory);
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_fileinfo(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_link(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_id_equal(file_info_1: *const FileInfo, file_info_2: *const FileInfo) -> bool;
    fn os_fileinfo_size(file_info: *const FileInfo) -> uint64_t;
    fn os_breakcheck();
    fn os_env_exists(name: *const ::core::ffi::c_char, nonempty: bool) -> bool;
    fn expand_env(
        src: *mut ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
    ) -> size_t;
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
    fn os_get_username(s: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn dir_of_file_exists(fname: *mut ::core::ffi::c_char) -> bool;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn path_with_url(fname: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_FullName(
        fname: *const ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        len: size_t,
        force: bool,
    ) -> ::core::ffi::c_int;
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn path_shorten_fname(
        full_path: *mut ::core::ffi::c_char,
        dir_name: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn sha256_start(ctx: *mut context_sha256_T);
    fn sha256_update(ctx: *mut context_sha256_T, input: *const uint8_t, length: size_t);
    fn sha256_finish(ctx: *mut context_sha256_T, digest: *mut uint8_t);
    fn check_marks_read();
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn sort_strings(files: *mut *mut ::core::ffi::c_char, count: ::core::ffi::c_int);
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn u_savecommon(
        buf: *mut buf_T,
        top: linenr_T,
        bot: linenr_T,
        newbot: linenr_T,
        reload: bool,
    ) -> ::core::ffi::c_int;
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
    fn u_sync(force: bool);
    fn u_unchanged(buf: *mut buf_T);
    fn u_find_first_changed();
    fn u_clearallandblockfree(buf: *mut buf_T);
    fn u_clearline(buf: *mut buf_T);
    fn bufIsChanged(buf: *mut buf_T) -> bool;
    fn flock(__fd: ::core::ffi::c_int, __operation: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ml_open(buf: *mut buf_T) -> ::core::ffi::c_int;
    fn check_need_swap(newfile: bool);
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
    fn ml_delete(lnum: linenr_T) -> ::core::ffi::c_int;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __mode_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type size_t = usize;
pub type mode_t = __mode_t;
pub type off_t = __off_t;
pub type time_t = __time_t;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type uintmax_t = ::libc::uintmax_t;
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
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
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
pub type DIR = __dirstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__work {
    pub work: Option<unsafe extern "C" fn(*mut uv__work) -> ()>,
    pub done: Option<unsafe extern "C" fn(*mut uv__work, ::core::ffi::c_int) -> ()>,
    pub loop_0: *mut uv_loop_s,
    pub wq: uv__queue,
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
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed_5 = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed_5 = -8;
pub const UV_EUNATCH: C2Rust_Unnamed_5 = -49;
pub const UV_ENODATA: C2Rust_Unnamed_5 = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed_5 = -94;
pub const UV_EILSEQ: C2Rust_Unnamed_5 = -84;
pub const UV_EFTYPE: C2Rust_Unnamed_5 = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed_5 = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed_5 = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed_5 = -112;
pub const UV_EMLINK: C2Rust_Unnamed_5 = -31;
pub const UV_ENXIO: C2Rust_Unnamed_5 = -6;
pub const UV_EOF: C2Rust_Unnamed_5 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_5 = -4094;
pub const UV_EXDEV: C2Rust_Unnamed_5 = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed_5 = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed_5 = -110;
pub const UV_ESRCH: C2Rust_Unnamed_5 = -3;
pub const UV_ESPIPE: C2Rust_Unnamed_5 = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed_5 = -108;
pub const UV_EROFS: C2Rust_Unnamed_5 = -30;
pub const UV_ERANGE: C2Rust_Unnamed_5 = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed_5 = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed_5 = -93;
pub const UV_EPROTO: C2Rust_Unnamed_5 = -71;
pub const UV_EPIPE: C2Rust_Unnamed_5 = -32;
pub const UV_EPERM: C2Rust_Unnamed_5 = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed_5 = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed_5 = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed_5 = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed_5 = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed_5 = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed_5 = -107;
pub const UV_ENOSYS: C2Rust_Unnamed_5 = -38;
pub const UV_ENOSPC: C2Rust_Unnamed_5 = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed_5 = -92;
pub const UV_ENONET: C2Rust_Unnamed_5 = -64;
pub const UV_ENOMEM: C2Rust_Unnamed_5 = -12;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_ENODEV: C2Rust_Unnamed_5 = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed_5 = -105;
pub const UV_ENFILE: C2Rust_Unnamed_5 = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed_5 = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed_5 = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed_5 = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed_5 = -90;
pub const UV_EMFILE: C2Rust_Unnamed_5 = -24;
pub const UV_ELOOP: C2Rust_Unnamed_5 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_5 = -21;
pub const UV_EISCONN: C2Rust_Unnamed_5 = -106;
pub const UV_EIO: C2Rust_Unnamed_5 = -5;
pub const UV_EINVAL: C2Rust_Unnamed_5 = -22;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed_5 = -113;
pub const UV_EFBIG: C2Rust_Unnamed_5 = -27;
pub const UV_EFAULT: C2Rust_Unnamed_5 = -14;
pub const UV_EEXIST: C2Rust_Unnamed_5 = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed_5 = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed_5 = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed_5 = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed_5 = -103;
pub const UV_ECHARSET: C2Rust_Unnamed_5 = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed_5 = -125;
pub const UV_EBUSY: C2Rust_Unnamed_5 = -16;
pub const UV_EBADF: C2Rust_Unnamed_5 = -9;
pub const UV_EALREADY: C2Rust_Unnamed_5 = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed_5 = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed_5 = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed_5 = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed_5 = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed_5 = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed_5 = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed_5 = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed_5 = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed_5 = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed_5 = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed_5 = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed_5 = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed_5 = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed_5 = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed_5 = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed_5 = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed_5 = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed_5 = -98;
pub const UV_EACCES: C2Rust_Unnamed_5 = -13;
pub const UV_E2BIG: C2Rust_Unnamed_5 = -7;
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
pub type uv_dirent_t = uv_dirent_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_dirent_s {
    pub name: *const ::core::ffi::c_char,
    pub type_0: uv_dirent_type_t,
}
pub type uv_dirent_type_t = ::core::ffi::c_uint;
pub const UV_DIRENT_BLOCK: uv_dirent_type_t = 7;
pub const UV_DIRENT_CHAR: uv_dirent_type_t = 6;
pub const UV_DIRENT_SOCKET: uv_dirent_type_t = 5;
pub const UV_DIRENT_FIFO: uv_dirent_type_t = 4;
pub const UV_DIRENT_LINK: uv_dirent_type_t = 3;
pub const UV_DIRENT_DIR: uv_dirent_type_t = 2;
pub const UV_DIRENT_FILE: uv_dirent_type_t = 1;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_fs_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub fs_type: uv_fs_type,
    pub loop_0: *mut uv_loop_t,
    pub cb: uv_fs_cb,
    pub result: ssize_t,
    pub ptr: *mut ::core::ffi::c_void,
    pub path: *const ::core::ffi::c_char,
    pub statbuf: uv_stat_t,
    pub new_path: *const ::core::ffi::c_char,
    pub file: uv_file,
    pub flags: ::core::ffi::c_int,
    pub mode: mode_t,
    pub nbufs: ::core::ffi::c_uint,
    pub bufs: *mut uv_buf_t,
    pub off: off_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
    pub atime: ::core::ffi::c_double,
    pub mtime: ::core::ffi::c_double,
    pub work_req: uv__work,
    pub bufsml: [uv_buf_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec_t {
    pub tv_sec: ::core::ffi::c_long,
    pub tv_nsec: ::core::ffi::c_long,
}
pub type uv_fs_cb = Option<unsafe extern "C" fn(*mut uv_fs_t) -> ()>;
pub type uv_fs_t = uv_fs_s;
pub type uv_fs_type = ::core::ffi::c_int;
pub const UV_FS_LUTIME: uv_fs_type = 36;
pub const UV_FS_MKSTEMP: uv_fs_type = 35;
pub const UV_FS_STATFS: uv_fs_type = 34;
pub const UV_FS_CLOSEDIR: uv_fs_type = 33;
pub const UV_FS_READDIR: uv_fs_type = 32;
pub const UV_FS_OPENDIR: uv_fs_type = 31;
pub const UV_FS_LCHOWN: uv_fs_type = 30;
pub const UV_FS_COPYFILE: uv_fs_type = 29;
pub const UV_FS_REALPATH: uv_fs_type = 28;
pub const UV_FS_FCHOWN: uv_fs_type = 27;
pub const UV_FS_CHOWN: uv_fs_type = 26;
pub const UV_FS_READLINK: uv_fs_type = 25;
pub const UV_FS_SYMLINK: uv_fs_type = 24;
pub const UV_FS_LINK: uv_fs_type = 23;
pub const UV_FS_SCANDIR: uv_fs_type = 22;
pub const UV_FS_RENAME: uv_fs_type = 21;
pub const UV_FS_MKDTEMP: uv_fs_type = 20;
pub const UV_FS_MKDIR: uv_fs_type = 19;
pub const UV_FS_RMDIR: uv_fs_type = 18;
pub const UV_FS_UNLINK: uv_fs_type = 17;
pub const UV_FS_FDATASYNC: uv_fs_type = 16;
pub const UV_FS_FSYNC: uv_fs_type = 15;
pub const UV_FS_FCHMOD: uv_fs_type = 14;
pub const UV_FS_CHMOD: uv_fs_type = 13;
pub const UV_FS_ACCESS: uv_fs_type = 12;
pub const UV_FS_FUTIME: uv_fs_type = 11;
pub const UV_FS_UTIME: uv_fs_type = 10;
pub const UV_FS_FTRUNCATE: uv_fs_type = 9;
pub const UV_FS_FSTAT: uv_fs_type = 8;
pub const UV_FS_LSTAT: uv_fs_type = 7;
pub const UV_FS_STAT: uv_fs_type = 6;
pub const UV_FS_SENDFILE: uv_fs_type = 5;
pub const UV_FS_WRITE: uv_fs_type = 4;
pub const UV_FS_READ: uv_fs_type = 3;
pub const UV_FS_CLOSE: uv_fs_type = 2;
pub const UV_FS_OPEN: uv_fs_type = 1;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const UV_FS_UNKNOWN: uv_fs_type = -1;
pub type off_T = off_t;
pub type vim_acl_T = *mut ::core::ffi::c_void;
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
    pub b_wininfo: C2Rust_Unnamed_17,
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
    pub b_signcols: C2Rust_Unnamed_9,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_7,
    pub update_callbacks: C2Rust_Unnamed_6,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_6 {
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
pub struct C2Rust_Unnamed_7 {
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
    pub data: C2Rust_Unnamed_8,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
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
pub struct C2Rust_Unnamed_9 {
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
    pub sst_union: C2Rust_Unnamed_10,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
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
    pub data: C2Rust_Unnamed_11,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
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
    pub fc_fixvar: [C2Rust_Unnamed_12; 12],
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
pub struct C2Rust_Unnamed_12 {
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
    pub uh_next: C2Rust_Unnamed_16,
    pub uh_prev: C2Rust_Unnamed_15,
    pub uh_alt_next: C2Rust_Unnamed_14,
    pub uh_alt_prev: C2Rust_Unnamed_13,
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
pub union C2Rust_Unnamed_13 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_15 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_16 {
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
pub struct C2Rust_Unnamed_17 {
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
    pub type_0: C2Rust_Unnamed_18,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_18 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_18 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_18 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_18 = 0;
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_19 = 2147483647;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_20 = 2147483647;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_21 = 76;
pub const HLF_PRE: C2Rust_Unnamed_21 = 75;
pub const HLF_OK: C2Rust_Unnamed_21 = 74;
pub const HLF_SO: C2Rust_Unnamed_21 = 73;
pub const HLF_SE: C2Rust_Unnamed_21 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_21 = 71;
pub const HLF_TS: C2Rust_Unnamed_21 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_21 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_21 = 68;
pub const HLF_CU: C2Rust_Unnamed_21 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_21 = 66;
pub const HLF_WBR: C2Rust_Unnamed_21 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_21 = 64;
pub const HLF_MSG: C2Rust_Unnamed_21 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_21 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_21 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_21 = 60;
pub const HLF_0: C2Rust_Unnamed_21 = 59;
pub const HLF_QFL: C2Rust_Unnamed_21 = 58;
pub const HLF_MC: C2Rust_Unnamed_21 = 57;
pub const HLF_CUL: C2Rust_Unnamed_21 = 56;
pub const HLF_CUC: C2Rust_Unnamed_21 = 55;
pub const HLF_TPF: C2Rust_Unnamed_21 = 54;
pub const HLF_TPS: C2Rust_Unnamed_21 = 53;
pub const HLF_TP: C2Rust_Unnamed_21 = 52;
pub const HLF_PBR: C2Rust_Unnamed_21 = 51;
pub const HLF_PST: C2Rust_Unnamed_21 = 50;
pub const HLF_PSB: C2Rust_Unnamed_21 = 49;
pub const HLF_PSX: C2Rust_Unnamed_21 = 48;
pub const HLF_PNX: C2Rust_Unnamed_21 = 47;
pub const HLF_PSK: C2Rust_Unnamed_21 = 46;
pub const HLF_PNK: C2Rust_Unnamed_21 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_21 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_21 = 43;
pub const HLF_PSI: C2Rust_Unnamed_21 = 42;
pub const HLF_PNI: C2Rust_Unnamed_21 = 41;
pub const HLF_SPL: C2Rust_Unnamed_21 = 40;
pub const HLF_SPR: C2Rust_Unnamed_21 = 39;
pub const HLF_SPC: C2Rust_Unnamed_21 = 38;
pub const HLF_SPB: C2Rust_Unnamed_21 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_21 = 36;
pub const HLF_SC: C2Rust_Unnamed_21 = 35;
pub const HLF_TXA: C2Rust_Unnamed_21 = 34;
pub const HLF_TXD: C2Rust_Unnamed_21 = 33;
pub const HLF_DED: C2Rust_Unnamed_21 = 32;
pub const HLF_CHD: C2Rust_Unnamed_21 = 31;
pub const HLF_ADD: C2Rust_Unnamed_21 = 30;
pub const HLF_FC: C2Rust_Unnamed_21 = 29;
pub const HLF_FL: C2Rust_Unnamed_21 = 28;
pub const HLF_WM: C2Rust_Unnamed_21 = 27;
pub const HLF_W: C2Rust_Unnamed_21 = 26;
pub const HLF_VNC: C2Rust_Unnamed_21 = 25;
pub const HLF_V: C2Rust_Unnamed_21 = 24;
pub const HLF_T: C2Rust_Unnamed_21 = 23;
pub const HLF_VSP: C2Rust_Unnamed_21 = 22;
pub const HLF_C: C2Rust_Unnamed_21 = 21;
pub const HLF_SNC: C2Rust_Unnamed_21 = 20;
pub const HLF_S: C2Rust_Unnamed_21 = 19;
pub const HLF_R: C2Rust_Unnamed_21 = 18;
pub const HLF_CLF: C2Rust_Unnamed_21 = 17;
pub const HLF_CLS: C2Rust_Unnamed_21 = 16;
pub const HLF_CLN: C2Rust_Unnamed_21 = 15;
pub const HLF_LNB: C2Rust_Unnamed_21 = 14;
pub const HLF_LNA: C2Rust_Unnamed_21 = 13;
pub const HLF_N: C2Rust_Unnamed_21 = 12;
pub const HLF_CM: C2Rust_Unnamed_21 = 11;
pub const HLF_M: C2Rust_Unnamed_21 = 10;
pub const HLF_LC: C2Rust_Unnamed_21 = 9;
pub const HLF_L: C2Rust_Unnamed_21 = 8;
pub const HLF_I: C2Rust_Unnamed_21 = 7;
pub const HLF_E: C2Rust_Unnamed_21 = 6;
pub const HLF_D: C2Rust_Unnamed_21 = 5;
pub const HLF_AT: C2Rust_Unnamed_21 = 4;
pub const HLF_TERM: C2Rust_Unnamed_21 = 3;
pub const HLF_EOB: C2Rust_Unnamed_21 = 2;
pub const HLF_8: C2Rust_Unnamed_21 = 1;
pub const HLF_NONE: C2Rust_Unnamed_21 = 0;
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
pub struct FileInfo {
    pub stat: uv_stat_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Directory {
    pub request: uv_fs_t,
    pub ent: uv_dirent_t,
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
    pub cs_pend: C2Rust_Unnamed_22,
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
pub union C2Rust_Unnamed_22 {
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
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_23 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_23 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_23 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_23 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_23 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_23 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_23 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_23 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_23 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_23 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_23 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_23 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_23 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_23 = 1;
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
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
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
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const ENC_MACROMAN: C2Rust_Unnamed_26 = 2048;
pub const ENC_LATIN9: C2Rust_Unnamed_26 = 1024;
pub const ENC_LATIN1: C2Rust_Unnamed_26 = 512;
pub const ENC_2WORD: C2Rust_Unnamed_26 = 256;
pub const ENC_4BYTE: C2Rust_Unnamed_26 = 128;
pub const ENC_2BYTE: C2Rust_Unnamed_26 = 64;
pub const ENC_ENDIAN_L: C2Rust_Unnamed_26 = 32;
pub const ENC_ENDIAN_B: C2Rust_Unnamed_26 = 16;
pub const ENC_UNICODE: C2Rust_Unnamed_26 = 4;
pub const ENC_DBCS: C2Rust_Unnamed_26 = 2;
pub const ENC_8BIT: C2Rust_Unnamed_26 = 1;
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
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_27 = 256;
pub const READ_NOWINENTER: C2Rust_Unnamed_27 = 128;
pub const READ_FIFO: C2Rust_Unnamed_27 = 64;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_27 = 32;
pub const READ_DUMMY: C2Rust_Unnamed_27 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_27 = 8;
pub const READ_STDIN: C2Rust_Unnamed_27 = 4;
pub const READ_FILTER: C2Rust_Unnamed_27 = 2;
pub const READ_NEW: C2Rust_Unnamed_27 = 1;
pub type CheckItem = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *const ::core::ffi::c_char) -> varnumber_T,
>;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const FIO_ALL: C2Rust_Unnamed_28 = -1;
pub const FIO_UCSBOM: C2Rust_Unnamed_28 = 16384;
pub const FIO_NOCONVERT: C2Rust_Unnamed_28 = 8192;
pub const FIO_ENDIAN_L: C2Rust_Unnamed_28 = 128;
pub const FIO_UTF16: C2Rust_Unnamed_28 = 16;
pub const FIO_UCS4: C2Rust_Unnamed_28 = 8;
pub const FIO_UCS2: C2Rust_Unnamed_28 = 4;
pub const FIO_UTF8: C2Rust_Unnamed_28 = 2;
pub const FIO_LATIN1: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const CONV_RESTLEN: C2Rust_Unnamed_29 = 30;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const ICONV_MULT: C2Rust_Unnamed_30 = 8;
pub const SHM_OVERALL: C2Rust_Unnamed_35 = 79;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct context_sha256_T {
    pub total: [uint32_t; 2],
    pub state: [uint32_t; 8],
    pub buffer: [uint8_t; 64],
}
pub const SHM_LINES: C2Rust_Unnamed_35 = 108;
pub const SHM_RO: C2Rust_Unnamed_35 = 114;
pub const OPT_LOCAL: C2Rust_Unnamed_34 = 2;
pub const SHM_OVER: C2Rust_Unnamed_35 = 111;
pub const RELOAD_DETECT: C2Rust_Unnamed_31 = 2;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const RELOAD_NORMAL: C2Rust_Unnamed_31 = 1;
pub const RELOAD_NONE: C2Rust_Unnamed_31 = 0;
pub const SHM_FILEINFO: C2Rust_Unnamed_35 = 70;
pub const MODE_CMDLINE: C2Rust_Unnamed_32 = 8;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_32 = 4097;
pub const VIM_WARNING: C2Rust_Unnamed_33 = 2;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_32 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_32 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_32 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_32 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_32 = 8193;
pub const MODE_LREPLACE: C2Rust_Unnamed_32 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_32 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_32 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_32 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_32 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_32 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_32 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_32 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_32 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_32 = 16;
pub const MODE_OP_PENDING: C2Rust_Unnamed_32 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_32 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_32 = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const VIM_LAST_TYPE: C2Rust_Unnamed_33 = 4;
pub const VIM_QUESTION: C2Rust_Unnamed_33 = 4;
pub const VIM_INFO: C2Rust_Unnamed_33 = 3;
pub const VIM_ERROR: C2Rust_Unnamed_33 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_34 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_34 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_34 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_34 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_34 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_34 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_34 = 1;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_35 = 83;
pub const SHM_RECORDING: C2Rust_Unnamed_35 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_35 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_35 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_35 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_35 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_35 = 115;
pub const SHM_TRUNCALL: C2Rust_Unnamed_35 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_35 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_35 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_35 = 97;
pub const SHM_WRI: C2Rust_Unnamed_35 = 119;
pub const SHM_MOD: C2Rust_Unnamed_35 = 109;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UV_FS_COPYFILE_EXCL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BACKSLASH_IN_FILENAME_BOOL: ::core::ffi::c_int = false_0;
pub const BASENAMELEN: ::core::ffi::c_int = NAME_MAX - 5 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_NEW_W: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_auchangedbuf: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E812: Autocommands changed buffer or buffer name\0".as_ptr() as *const ::core::ffi::c_char,
);
pub const NONASCII_MASK: uint64_t = (-1 as ::core::ffi::c_int as uint64_t)
    .wrapping_div(0xff as uint64_t)
    .wrapping_mul(0x80 as uint64_t);
#[no_mangle]
pub unsafe extern "C" fn filemess(
    mut buf: *mut buf_T,
    mut name: *mut ::core::ffi::c_char,
    mut s: *mut ::core::ffi::c_char,
) {
    let mut prev_msg_col: ::core::ffi::c_int = msg_col;
    if msg_silent != 0 as ::core::ffi::c_int {
        return;
    }
    add_quoted_fname(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        (IOSIZE - 100 as ::core::ffi::c_int) as size_t,
        buf,
        name,
    );
    xstrlcat(
        &raw mut IObuff as *mut ::core::ffi::c_char,
        s,
        IOSIZE as size_t,
    );
    let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll;
    if shortmess(SHM_OVERALL as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        && msg_listdo_overwrite == 0
        && !exiting
        && p_verbose == 0 as OptInt
    {
        msg_scroll = false_0;
    }
    if msg_scroll == 0 {
        msg_check_for_delay(false_0 != 0);
    }
    msg_start();
    if prev_msg_col != 0 as ::core::ffi::c_int && msg_col == 0 as ::core::ffi::c_int {
        msg_putchar('\r' as ::core::ffi::c_int);
    }
    msg_scroll = msg_scroll_save;
    msg_scrolled_ign = true_0 != 0;
    if *s as ::core::ffi::c_int == NUL {
        msg_progress(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            b"bufwrite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            b"running\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            false_0 != 0,
            true_0 != 0,
        );
    } else {
        msg_outtrans(
            msg_may_trunc(false_0 != 0, &raw mut IObuff as *mut ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    msg_clr_eos();
    msg_scrolled_ign = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn readfile(
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut from: linenr_T,
    mut lines_to_skip: linenr_T,
    mut lines_to_read: linenr_T,
    mut eap: *mut exarg_T,
    mut flags: ::core::ffi::c_int,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut incomplete_tail: bool = false;
    let mut can_retry: bool = false;
    let mut check_readonly: bool = false;
    let mut file_readonly: bool = false;
    let mut try_mac: ::core::ffi::c_int = 0;
    let mut try_dos: ::core::ffi::c_int = 0;
    let mut try_unix: ::core::ffi::c_int = 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut fd: ::core::ffi::c_int = if stdin_fd >= 0 as ::core::ffi::c_int {
        stdin_fd
    } else {
        0 as ::core::ffi::c_int
    };
    let mut newfile: bool = flags & READ_NEW as ::core::ffi::c_int != 0;
    let mut filtering: bool = flags & READ_FILTER as ::core::ffi::c_int != 0;
    let mut read_stdin: bool = flags & READ_STDIN as ::core::ffi::c_int != 0;
    let mut read_buffer: bool = flags & READ_BUFFER as ::core::ffi::c_int != 0;
    let mut read_fifo: bool = flags & READ_FIFO as ::core::ffi::c_int != 0;
    let mut set_options: bool = newfile as ::core::ffi::c_int != 0
        || read_buffer as ::core::ffi::c_int != 0
        || !eap.is_null() && (*eap).read_edit != 0;
    let mut read_buf_lnum: linenr_T = 1 as linenr_T;
    let mut read_buf_col: colnr_T = 0 as colnr_T;
    let mut c: ::core::ffi::c_char = 0;
    let mut lnum: linenr_T = from;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line_start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wasempty: ::core::ffi::c_int = 0;
    let mut len: colnr_T = 0;
    let mut size: ptrdiff_t = 0 as ptrdiff_t;
    let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut filesize: off_T = 0 as off_T;
    let mut skip_read: bool = false_0 != 0;
    let mut sha_ctx: context_sha256_T = context_sha256_T {
        total: [0; 2],
        state: [0; 8],
        buffer: [0; 64],
    };
    let mut read_undo_file: bool = false_0 != 0;
    let mut split: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut linecnt: linenr_T = 0;
    let mut error: bool = false_0 != 0;
    let mut ff_error: ::core::ffi::c_int = EOL_UNKNOWN;
    let mut linerest: ptrdiff_t = 0 as ptrdiff_t;
    let mut perm: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut swap_mode: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut fileformat: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keep_fileformat: bool = false_0 != 0;
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut skip_count: linenr_T = 0 as linenr_T;
    let mut read_count: linenr_T = 0 as linenr_T;
    let mut msg_save: ::core::ffi::c_int = msg_scroll;
    let mut read_no_eol_lnum: linenr_T = 0 as linenr_T;
    let mut file_rewind: bool = false_0 != 0;
    let mut conv_error: linenr_T = 0 as linenr_T;
    let mut illegal_byte: linenr_T = 0 as linenr_T;
    let mut keep_dest_enc: bool = false_0 != 0;
    let mut bad_char_behavior: ::core::ffi::c_int = BAD_REPLACE;
    let mut tmpname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fio_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fenc_alloced: bool = false;
    let mut fenc_next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut advance_fenc: bool = false_0 != 0;
    let mut real_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut iconv_fd: iconv_t = ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
        -1 as ::core::ffi::c_int as usize,
    );
    let mut did_iconv: bool = false_0 != 0;
    let mut converted: bool = false_0 != 0;
    let mut notconverted: bool = false_0 != 0;
    let mut conv_rest: [::core::ffi::c_char; 30] = [0; 30];
    let mut conv_restlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut orig_start: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut old_curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut old_b_ffname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_b_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut using_b_ffname: ::core::ffi::c_int = 0;
    let mut using_b_fname: ::core::ffi::c_int = 0;
    static msg_is_a_directory: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
        b"is a directory\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    (*curbuf).b_au_did_filetype = false_0 != 0;
    (*curbuf).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
    '_theend: {
        if (*curbuf).b_ffname.is_null()
            && !filtering
            && !fname.is_null()
            && !vim_strchr(p_cpo, CPO_FNAMER).is_null()
            && flags & READ_DUMMY as ::core::ffi::c_int == 0
        {
            if set_rw_fname(fname, sfname) == FAIL {
                break '_theend;
            }
        }
        old_curbuf = curbuf;
        old_b_ffname = (*curbuf).b_ffname;
        old_b_fname = (*curbuf).b_fname;
        using_b_ffname =
            (fname == (*curbuf).b_ffname || sfname == (*curbuf).b_ffname) as ::core::ffi::c_int;
        using_b_fname =
            (fname == (*curbuf).b_fname || sfname == (*curbuf).b_fname) as ::core::ffi::c_int;
        ex_no_reprint = true_0 != 0;
        need_fileinfo = false_0 != 0;
        if sfname.is_null() {
            sfname = fname;
        }
        fname = sfname;
        if !filtering && !read_stdin && !read_buffer {
            orig_start = (*curbuf).b_op_start;
            (*curbuf).b_op_start.lnum = if from == 0 as linenr_T {
                1 as linenr_T
            } else {
                from
            };
            (*curbuf).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
            if newfile {
                if apply_autocmds_exarg(
                    EVENT_BUFREADCMD,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    sfname,
                    false_0 != 0,
                    curbuf,
                    eap,
                ) {
                    retval = OK;
                    if aborting() {
                        retval = FAIL;
                    }
                    if retval == OK {
                        (*curbuf).b_flags &= !BF_NOTEDITED;
                    }
                    break '_theend;
                }
            } else if apply_autocmds_exarg(
                EVENT_FILEREADCMD,
                sfname,
                sfname,
                false_0 != 0,
                ::core::ptr::null_mut::<buf_T>(),
                eap,
            ) {
                retval = if aborting() as ::core::ffi::c_int != 0 {
                    FAIL
                } else {
                    OK
                };
                break '_theend;
            }
            (*curbuf).b_op_start = orig_start;
            if flags & READ_NOFILE as ::core::ffi::c_int != 0 {
                retval = NOTDONE;
                break '_theend;
            }
        }
        if (shortmess(SHM_OVER as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            && msg_listdo_overwrite == 0
            || (*curbuf).b_help as ::core::ffi::c_int != 0)
            && p_verbose == 0 as OptInt
        {
            msg_scroll = false_0;
        } else {
            msg_scroll = true_0;
        }
        if !fname.is_null() && *fname as ::core::ffi::c_int != NUL {
            let mut fnamelen: size_t = strlen(fname);
            if fnamelen >= MAXPATHL as size_t {
                filemess(
                    curbuf,
                    fname,
                    gettext(b"Illegal file name\0".as_ptr() as *const ::core::ffi::c_char),
                );
                msg_end();
                msg_scroll = msg_save;
                break '_theend;
            } else if after_pathsep(fname, fname.offset(fnamelen as isize)) != 0 {
                if !silent {
                    filemess(curbuf, fname, gettext(msg_is_a_directory.get()));
                }
                msg_end();
                msg_scroll = msg_save;
                retval = NOTDONE;
                break '_theend;
            }
        }
        if !read_stdin && !fname.is_null() {
            perm = os_getperm(fname) as ::core::ffi::c_int;
        }
        if !read_stdin && !read_buffer && !read_fifo {
            if perm >= 0 as ::core::ffi::c_int
                && !(perm & __S_IFMT == 0o100000 as ::core::ffi::c_int)
                && !(perm & __S_IFMT == 0o10000 as ::core::ffi::c_int)
                && !(perm & __S_IFMT == 0o140000 as ::core::ffi::c_int)
                && true
            {
                if perm & __S_IFMT == 0o40000 as ::core::ffi::c_int {
                    if !silent {
                        filemess(curbuf, fname, gettext(msg_is_a_directory.get()));
                    }
                    retval = NOTDONE;
                } else {
                    filemess(
                        curbuf,
                        fname,
                        gettext(b"is not a file\0".as_ptr() as *const ::core::ffi::c_char),
                    );
                }
                msg_end();
                msg_scroll = msg_save;
                break '_theend;
            }
        }
        set_file_options(set_options, eap);
        check_readonly = newfile as ::core::ffi::c_int != 0 && (*curbuf).b_flags & BF_CHECK_RO != 0;
        if check_readonly as ::core::ffi::c_int != 0 && !readonlymode {
            (*curbuf).b_p_ro = false_0;
        }
        if newfile as ::core::ffi::c_int != 0 && !read_stdin && !read_buffer && !read_fifo {
            if os_fileinfo(fname, &raw mut file_info) {
                buf_store_file_info(curbuf, &raw mut file_info);
                (*curbuf).b_mtime_read = (*curbuf).b_mtime;
                (*curbuf).b_mtime_read_ns = (*curbuf).b_mtime_ns;
                swap_mode = file_info.stat.st_mode as ::core::ffi::c_int
                    & 0o644 as ::core::ffi::c_int
                    | 0o600 as ::core::ffi::c_int;
            } else {
                (*curbuf).b_mtime = 0 as int64_t;
                (*curbuf).b_mtime_ns = 0 as int64_t;
                (*curbuf).b_mtime_read = 0 as int64_t;
                (*curbuf).b_mtime_read_ns = 0 as int64_t;
                (*curbuf).b_orig_size = 0 as uint64_t;
                (*curbuf).b_orig_mode = 0 as ::core::ffi::c_int;
            }
            (*curbuf).b_flags &= !(BF_NEW | BF_NEW_W);
        }
        file_readonly = false_0 != 0;
        if !read_buffer && !read_stdin {
            if !newfile
                || readonlymode as ::core::ffi::c_int != 0
                || perm & 0o222 as ::core::ffi::c_int == 0
                || os_file_is_writable(fname) == 0
            {
                file_readonly = true_0 != 0;
            }
            fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
        }
        if fd < 0 as ::core::ffi::c_int {
            msg_scroll = msg_save;
            if newfile {
                if perm == UV_ENOENT as ::core::ffi::c_int {
                    (*curbuf).b_flags |= BF_NEW;
                    if !bt_dontwrite(curbuf) {
                        check_need_swap(newfile);
                        if curbuf != old_curbuf
                            || using_b_ffname != 0 && old_b_ffname != (*curbuf).b_ffname
                            || using_b_fname != 0 && old_b_fname != (*curbuf).b_fname
                        {
                            emsg(gettext(e_auchangedbuf.get()));
                            break '_theend;
                        }
                    }
                    if !silent {
                        if dir_of_file_exists(fname) {
                            filemess(
                                curbuf,
                                sfname,
                                gettext(b"[New]\0".as_ptr() as *const ::core::ffi::c_char),
                            );
                        } else {
                            filemess(
                                curbuf,
                                sfname,
                                gettext(b"[New DIRECTORY]\0".as_ptr() as *const ::core::ffi::c_char),
                            );
                        }
                    }
                    check_marks_read();
                    if !eap.is_null() {
                        set_forced_fenc(eap);
                    }
                    apply_autocmds_exarg(
                        EVENT_BUFNEWFILE,
                        sfname,
                        sfname,
                        false_0 != 0,
                        curbuf,
                        eap,
                    );
                    save_file_ff(curbuf);
                    if !aborting() {
                        retval = OK;
                    }
                } else {
                    filemess(
                        curbuf,
                        sfname,
                        if fd == UV_EFBIG as ::core::ffi::c_int {
                            gettext(b"[File too big]\0".as_ptr() as *const ::core::ffi::c_char)
                        } else if fd == -EOVERFLOW {
                            gettext(b"[File too big]\0".as_ptr() as *const ::core::ffi::c_char)
                        } else {
                            gettext(b"[Permission Denied]\0".as_ptr() as *const ::core::ffi::c_char)
                        },
                    );
                    (*curbuf).b_p_ro = true_0;
                }
            }
        } else {
            if check_readonly as ::core::ffi::c_int != 0 && file_readonly as ::core::ffi::c_int != 0
                || (*curbuf).b_help as ::core::ffi::c_int != 0
            {
                (*curbuf).b_p_ro = true_0;
            }
            if set_options {
                if !read_buffer {
                    (*curbuf).b_p_eof = false_0;
                    (*curbuf).b_start_eof = false_0;
                    (*curbuf).b_p_eol = true_0;
                    (*curbuf).b_start_eol = true_0;
                }
                (*curbuf).b_p_bomb = false_0;
                (*curbuf).b_start_bomb = false_0;
            }
            if !bt_dontwrite(curbuf) {
                check_need_swap(newfile);
                if !read_stdin
                    && (curbuf != old_curbuf
                        || using_b_ffname != 0 && old_b_ffname != (*curbuf).b_ffname
                        || using_b_fname != 0 && old_b_fname != (*curbuf).b_fname)
                {
                    emsg(gettext(e_auchangedbuf.get()));
                    if !read_buffer {
                        close(fd);
                    }
                    break '_theend;
                } else if swap_mode > 0 as ::core::ffi::c_int
                    && !(*curbuf).b_ml.ml_mfp.is_null()
                    && !(*(*curbuf).b_ml.ml_mfp).mf_fname.is_null()
                {
                    let mut swap_fname: *const ::core::ffi::c_char =
                        (*(*curbuf).b_ml.ml_mfp).mf_fname;
                    if swap_mode & 0o44 as ::core::ffi::c_int == 0o40 as ::core::ffi::c_int {
                        let mut swap_info: FileInfo = FileInfo {
                            stat: uv_stat_t {
                                st_dev: 0,
                                st_mode: 0,
                                st_nlink: 0,
                                st_uid: 0,
                                st_gid: 0,
                                st_rdev: 0,
                                st_ino: 0,
                                st_size: 0,
                                st_blksize: 0,
                                st_blocks: 0,
                                st_flags: 0,
                                st_gen: 0,
                                st_atim: uv_timespec_t {
                                    tv_sec: 0,
                                    tv_nsec: 0,
                                },
                                st_mtim: uv_timespec_t {
                                    tv_sec: 0,
                                    tv_nsec: 0,
                                },
                                st_ctim: uv_timespec_t {
                                    tv_sec: 0,
                                    tv_nsec: 0,
                                },
                                st_birthtim: uv_timespec_t {
                                    tv_sec: 0,
                                    tv_nsec: 0,
                                },
                            },
                        };
                        if os_fileinfo(swap_fname, &raw mut swap_info) as ::core::ffi::c_int != 0
                            && file_info.stat.st_gid != swap_info.stat.st_gid
                            && os_fchown(
                                (*(*curbuf).b_ml.ml_mfp).mf_fd,
                                -1 as ::core::ffi::c_int as uv_uid_t,
                                file_info.stat.st_gid as uv_gid_t,
                            ) == -1 as ::core::ffi::c_int
                        {
                            swap_mode &= 0o600 as ::core::ffi::c_int;
                        }
                    }
                    os_setperm(swap_fname, swap_mode);
                }
            }
            if swap_exists_action == SEA_QUIT {
                if !read_buffer && !read_stdin {
                    close(fd);
                }
            } else {
                no_wait_return += 1;
                orig_start = (*curbuf).b_op_start;
                (*curbuf).b_op_start.lnum = if from == 0 as linenr_T {
                    1 as linenr_T
                } else {
                    from
                };
                (*curbuf).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                try_mac =
                    !vim_strchr(p_ffs, 'm' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
                try_dos =
                    !vim_strchr(p_ffs, 'd' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
                try_unix =
                    !vim_strchr(p_ffs, 'x' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
                if !read_buffer {
                    let mut m: ::core::ffi::c_int = msg_scroll;
                    let mut n: ::core::ffi::c_int = msg_scrolled;
                    if !read_stdin {
                        close(fd);
                    }
                    msg_scroll = true_0;
                    if filtering {
                        apply_autocmds_exarg(
                            EVENT_FILTERREADPRE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            sfname,
                            false_0 != 0,
                            curbuf,
                            eap,
                        );
                    } else if read_stdin {
                        apply_autocmds_exarg(
                            EVENT_STDINREADPRE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            sfname,
                            false_0 != 0,
                            curbuf,
                            eap,
                        );
                    } else if newfile {
                        apply_autocmds_exarg(
                            EVENT_BUFREADPRE,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            sfname,
                            false_0 != 0,
                            curbuf,
                            eap,
                        );
                    } else {
                        apply_autocmds_exarg(
                            EVENT_FILEREADPRE,
                            sfname,
                            sfname,
                            false_0 != 0,
                            ::core::ptr::null_mut::<buf_T>(),
                            eap,
                        );
                    }
                    try_mac = !vim_strchr(p_ffs, 'm' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    try_dos = !vim_strchr(p_ffs, 'd' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    try_unix = !vim_strchr(p_ffs, 'x' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    (*curbuf).b_op_start = orig_start;
                    if msg_scrolled == n {
                        msg_scroll = m;
                    }
                    if aborting() {
                        no_wait_return -= 1;
                        msg_scroll = msg_save;
                        (*curbuf).b_p_ro = true_0;
                        break '_theend;
                    } else if !read_stdin
                        && (curbuf != old_curbuf
                            || using_b_ffname != 0 && old_b_ffname != (*curbuf).b_ffname
                            || using_b_fname != 0 && old_b_fname != (*curbuf).b_fname
                            || {
                                fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
                                fd < 0 as ::core::ffi::c_int
                            })
                    {
                        no_wait_return -= 1;
                        msg_scroll = msg_save;
                        if fd < 0 as ::core::ffi::c_int {
                            emsg(gettext(
                                b"E200: *ReadPre autocommands made the file unreadable\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        } else {
                            emsg(gettext(
                                b"E201: *ReadPre autocommands must not change current buffer\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        (*curbuf).b_p_ro = true_0;
                        break '_theend;
                    }
                }
                wasempty = (*curbuf).b_ml.ml_flags & ML_EMPTY;
                if !recoverymode
                    && !filtering
                    && flags & READ_DUMMY as ::core::ffi::c_int == 0
                    && !silent
                {
                    if !read_stdin && !read_buffer {
                        filemess(
                            curbuf,
                            sfname,
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                    }
                }
                msg_scroll = false_0;
                linecnt = (*curbuf).b_ml.ml_line_count;
                if !eap.is_null() && (*eap).bad_char != 0 as ::core::ffi::c_int {
                    bad_char_behavior = (*eap).bad_char;
                    if set_options {
                        (*curbuf).b_bad_char = (*eap).bad_char;
                    }
                } else {
                    (*curbuf).b_bad_char = 0 as ::core::ffi::c_int;
                }
                if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                    fenc = enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
                    fenc_alloced = true_0 != 0;
                    keep_dest_enc = true_0 != 0;
                } else if (*curbuf).b_p_bin != 0 {
                    fenc = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                    fenc_alloced = false_0 != 0;
                } else if (*curbuf).b_help {
                    fenc_next = b"latin1\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    fenc = b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    fenc_alloced = false_0 != 0;
                } else if *p_fencs as ::core::ffi::c_int == NUL {
                    fenc = (*curbuf).b_p_fenc;
                    fenc_alloced = false_0 != 0;
                } else {
                    fenc_next = p_fencs;
                    fenc = next_fenc(&raw mut fenc_next, &raw mut fenc_alloced);
                }
                '_failed: loop {
                    if file_rewind {
                        if read_buffer {
                            read_buf_lnum = 1 as ::core::ffi::c_int as linenr_T;
                            read_buf_col = 0 as ::core::ffi::c_int as colnr_T;
                        } else if read_stdin as ::core::ffi::c_int != 0
                            || lseek(fd, 0 as __off_t, SEEK_SET) != 0 as __off_t
                        {
                            error = true_0 != 0;
                            break;
                        }
                        while lnum > from {
                            let c2rust_fresh0 = lnum;
                            lnum = lnum - 1;
                            ml_delete(c2rust_fresh0);
                        }
                        file_rewind = false_0 != 0;
                        if set_options {
                            (*curbuf).b_p_bomb = false_0;
                            (*curbuf).b_start_bomb = false_0;
                        }
                        conv_error = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    if keep_fileformat {
                        keep_fileformat = false_0 != 0;
                    } else if !eap.is_null() && (*eap).force_ff != 0 as ::core::ffi::c_int {
                        fileformat = get_fileformat_force(curbuf, eap);
                        try_mac = false_0;
                        try_dos = try_mac;
                        try_unix = try_dos;
                    } else if (*curbuf).b_p_bin != 0 {
                        fileformat = EOL_UNIX;
                    } else if *p_ffs as ::core::ffi::c_int == NUL {
                        fileformat = get_fileformat(curbuf);
                    } else {
                        fileformat = EOL_UNKNOWN;
                    }
                    if iconv_fd
                        != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                            -1 as ::core::ffi::c_int as usize,
                        )
                    {
                        iconv_close(iconv_fd);
                        iconv_fd = ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                            -1 as ::core::ffi::c_int as usize,
                        );
                    }
                    if advance_fenc {
                        advance_fenc = false_0 != 0;
                        if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                            notconverted = true_0 != 0;
                            conv_error = 0 as ::core::ffi::c_int as linenr_T;
                            if fenc_alloced {
                                xfree(fenc as *mut ::core::ffi::c_void);
                            }
                            fenc = b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                            fenc_alloced = false_0 != 0;
                        } else {
                            if fenc_alloced {
                                xfree(fenc as *mut ::core::ffi::c_void);
                            }
                            if !fenc_next.is_null() {
                                fenc = next_fenc(&raw mut fenc_next, &raw mut fenc_alloced);
                            } else {
                                fenc = b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                                fenc_alloced = false_0 != 0;
                            }
                        }
                        if !tmpname.is_null() {
                            os_remove(tmpname);
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut tmpname as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            *ptr_;
                        }
                    }
                    fio_flags = 0 as ::core::ffi::c_int;
                    converted = need_conversion(fenc);
                    if converted {
                        if strcmp(fenc, ENC_UCSBOM.as_ptr()) == 0 as ::core::ffi::c_int {
                            fio_flags = FIO_UCSBOM as ::core::ffi::c_int;
                        } else {
                            fio_flags = get_fio_flags(fenc);
                        }
                        if fio_flags == 0 as ::core::ffi::c_int && !did_iconv {
                            iconv_fd = my_iconv_open(
                                b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                fenc,
                            );
                        }
                        if fio_flags == 0 as ::core::ffi::c_int
                            && !read_stdin
                            && !read_buffer
                            && *p_ccv as ::core::ffi::c_int != NUL
                            && !read_fifo
                            && iconv_fd
                                == ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                    -1 as ::core::ffi::c_int as usize,
                                )
                        {
                            did_iconv = false_0 != 0;
                            if tmpname.is_null() {
                                tmpname = readfile_charconvert(fname, fenc, &raw mut fd);
                                if tmpname.is_null() {
                                    advance_fenc = true_0 != 0;
                                    if fd >= 0 as ::core::ffi::c_int {
                                        continue;
                                    }
                                    emsg(gettext(
                                        b"E202: Conversion made file unreadable!\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ));
                                    error = true_0 != 0;
                                    break;
                                }
                            }
                        } else if fio_flags == 0 as ::core::ffi::c_int
                            && iconv_fd
                                == ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                    -1 as ::core::ffi::c_int as usize,
                                )
                        {
                            advance_fenc = true_0 != 0;
                            continue;
                        }
                    }
                    can_retry = *fenc as ::core::ffi::c_int != NUL
                        && !read_stdin
                        && !keep_dest_enc
                        && !read_fifo;
                    if !skip_read {
                        linerest = 0 as ptrdiff_t;
                        filesize = 0 as off_T;
                        skip_count = lines_to_skip;
                        read_count = lines_to_read;
                        conv_restlen = 0 as ::core::ffi::c_int;
                        read_undo_file = newfile as ::core::ffi::c_int != 0
                            && flags & READ_KEEP_UNDO as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            && !(*curbuf).b_ffname.is_null()
                            && (*curbuf).b_p_udf != 0
                            && !filtering
                            && !read_fifo
                            && !read_stdin
                            && !read_buffer;
                        if read_undo_file {
                            sha256_start(&raw mut sha_ctx);
                        }
                    }
                    's_1469: loop {
                        if !(!error && !got_int) {
                            break '_failed;
                        }
                        if !skip_read {
                            size = if 0x10000 as ::core::ffi::c_int as ptrdiff_t + linerest
                                < 0x100000 as ::core::ffi::c_int as ptrdiff_t
                            {
                                0x10000 as ::core::ffi::c_int as ptrdiff_t + linerest
                            } else {
                                0x100000 as ::core::ffi::c_int as ptrdiff_t
                            };
                        }
                        '_rewind_retry: {
                            if size < 0 as ptrdiff_t
                                || (size + linerest + 1 as ptrdiff_t) < 0 as ptrdiff_t
                                || linerest >= MAXCOL as ::core::ffi::c_int as ptrdiff_t - size
                            {
                                split += 1;
                                *ptr = NL as ::core::ffi::c_char;
                                size = 1 as ptrdiff_t;
                            } else if !skip_read {
                                while size >= 10 as ptrdiff_t {
                                    new_buffer = verbose_try_malloc(
                                        (size as size_t)
                                            .wrapping_add(linerest as size_t)
                                            .wrapping_add(1 as size_t),
                                    )
                                        as *mut ::core::ffi::c_char;
                                    if !new_buffer.is_null() {
                                        break;
                                    }
                                    size /= 2 as ptrdiff_t;
                                }
                                if new_buffer.is_null() {
                                    error = true_0 != 0;
                                    break '_failed;
                                } else {
                                    if linerest != 0 {
                                        memmove(
                                            new_buffer as *mut ::core::ffi::c_void,
                                            ptr.offset(-(linerest as isize))
                                                as *const ::core::ffi::c_void,
                                            linerest as size_t,
                                        );
                                    }
                                    xfree(buffer as *mut ::core::ffi::c_void);
                                    buffer = new_buffer;
                                    ptr = buffer.offset(linerest as isize);
                                    line_start = buffer;
                                    real_size = size as ::core::ffi::c_int;
                                    if iconv_fd
                                        != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                            -1 as ::core::ffi::c_int as usize,
                                        )
                                    {
                                        size = size / ICONV_MULT as ::core::ffi::c_int as ptrdiff_t;
                                    } else if fio_flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
                                        size = size / 2 as ptrdiff_t;
                                    } else if fio_flags
                                        & (FIO_UCS2 as ::core::ffi::c_int
                                            | FIO_UTF16 as ::core::ffi::c_int)
                                        != 0
                                    {
                                        size = size * 2 as ptrdiff_t / 3 as ptrdiff_t
                                            & !(1 as ::core::ffi::c_int) as ptrdiff_t;
                                    } else if fio_flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                                        size = size * 2 as ptrdiff_t / 3 as ptrdiff_t
                                            & !(3 as ::core::ffi::c_int) as ptrdiff_t;
                                    } else if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                        size = size / ICONV_MULT as ::core::ffi::c_int as ptrdiff_t;
                                    }
                                    if conv_restlen > 0 as ::core::ffi::c_int {
                                        memmove(
                                            ptr as *mut ::core::ffi::c_void,
                                            &raw mut conv_rest as *mut ::core::ffi::c_char
                                                as *const ::core::ffi::c_void,
                                            conv_restlen as size_t,
                                        );
                                        ptr = ptr.offset(conv_restlen as isize);
                                        size -= conv_restlen as ptrdiff_t;
                                    }
                                    if read_buffer {
                                        if read_buf_lnum > from {
                                            size = 0 as ptrdiff_t;
                                        } else {
                                            let mut ni: ::core::ffi::c_int = 0;
                                            let mut tlen: ::core::ffi::c_int =
                                                0 as ::core::ffi::c_int;
                                            loop {
                                                p = (ml_get(read_buf_lnum) as *mut uint8_t)
                                                    .offset(read_buf_col as isize);
                                                let mut n_0: ::core::ffi::c_int =
                                                    ml_get_len(read_buf_lnum)
                                                        - read_buf_col as ::core::ffi::c_int;
                                                if (tlen + n_0 + 1 as ::core::ffi::c_int)
                                                    as ptrdiff_t
                                                    > size
                                                {
                                                    n_0 = (size - tlen as ptrdiff_t)
                                                        as ::core::ffi::c_int;
                                                    ni = 0 as ::core::ffi::c_int;
                                                    while ni < n_0 {
                                                        if *p.offset(ni as isize)
                                                            as ::core::ffi::c_int
                                                            == NL
                                                        {
                                                            let c2rust_fresh1 = tlen;
                                                            tlen = tlen + 1;
                                                            *ptr.offset(c2rust_fresh1 as isize) =
                                                                NUL as ::core::ffi::c_char;
                                                        } else {
                                                            let c2rust_fresh2 = tlen;
                                                            tlen = tlen + 1;
                                                            *ptr.offset(c2rust_fresh2 as isize) = *p
                                                                .offset(ni as isize)
                                                                as ::core::ffi::c_char;
                                                        }
                                                        ni += 1;
                                                    }
                                                    read_buf_col += n_0;
                                                    break;
                                                } else {
                                                    ni = 0 as ::core::ffi::c_int;
                                                    while ni < n_0 {
                                                        if *p.offset(ni as isize)
                                                            as ::core::ffi::c_int
                                                            == NL
                                                        {
                                                            let c2rust_fresh3 = tlen;
                                                            tlen = tlen + 1;
                                                            *ptr.offset(c2rust_fresh3 as isize) =
                                                                NUL as ::core::ffi::c_char;
                                                        } else {
                                                            let c2rust_fresh4 = tlen;
                                                            tlen = tlen + 1;
                                                            *ptr.offset(c2rust_fresh4 as isize) = *p
                                                                .offset(ni as isize)
                                                                as ::core::ffi::c_char;
                                                        }
                                                        ni += 1;
                                                    }
                                                    let c2rust_fresh5 = tlen;
                                                    tlen = tlen + 1;
                                                    *ptr.offset(c2rust_fresh5 as isize) =
                                                        NL as ::core::ffi::c_char;
                                                    read_buf_col =
                                                        0 as ::core::ffi::c_int as colnr_T;
                                                    read_buf_lnum += 1;
                                                    if read_buf_lnum <= from {
                                                        continue;
                                                    }
                                                    if (*curbuf).b_p_eol == 0 {
                                                        tlen -= 1;
                                                    }
                                                    size = tlen as ptrdiff_t;
                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        let mut read_size: size_t = size as size_t;
                                        size = read_eintr(
                                            fd,
                                            ptr as *mut ::core::ffi::c_void,
                                            read_size,
                                        )
                                            as ptrdiff_t;
                                    }
                                    if size <= 0 as ptrdiff_t {
                                        if size < 0 as ptrdiff_t {
                                            error = true_0 != 0;
                                        } else if conv_restlen > 0 as ::core::ffi::c_int {
                                            if fio_flags != 0 as ::core::ffi::c_int
                                                || iconv_fd
                                                    != ::core::ptr::from_exposed_addr_mut::<
                                                        ::core::ffi::c_void,
                                                    >(
                                                        -1 as ::core::ffi::c_int as usize
                                                    )
                                            {
                                                if can_retry {
                                                    break '_rewind_retry;
                                                } else if conv_error == 0 as linenr_T {
                                                    conv_error = (*curbuf).b_ml.ml_line_count
                                                        - linecnt
                                                        + 1 as linenr_T;
                                                }
                                            } else if illegal_byte == 0 as linenr_T {
                                                illegal_byte = (*curbuf).b_ml.ml_line_count
                                                    - linecnt
                                                    + 1 as linenr_T;
                                            }
                                            if bad_char_behavior == BAD_DROP {
                                                *ptr.offset(-(conv_restlen as isize)) =
                                                    NUL as ::core::ffi::c_char;
                                                conv_restlen = 0 as ::core::ffi::c_int;
                                            } else {
                                                if bad_char_behavior != BAD_KEEP
                                                    && (fio_flags != 0 as ::core::ffi::c_int
                                                        || iconv_fd
                                                            != ::core::ptr::from_exposed_addr_mut::<
                                                                ::core::ffi::c_void,
                                                            >(
                                                                -1 as ::core::ffi::c_int as usize
                                                            ))
                                                {
                                                    while conv_restlen > 0 as ::core::ffi::c_int {
                                                        ptr = ptr.offset(-1);
                                                        *ptr = bad_char_behavior
                                                            as ::core::ffi::c_char;
                                                        conv_restlen -= 1;
                                                    }
                                                }
                                                fio_flags = 0 as ::core::ffi::c_int;
                                                if iconv_fd
                                                    != ::core::ptr::from_exposed_addr_mut::<
                                                        ::core::ffi::c_void,
                                                    >(
                                                        -1 as ::core::ffi::c_int as usize
                                                    )
                                                {
                                                    iconv_close(iconv_fd);
                                                    iconv_fd = ::core::ptr::from_exposed_addr_mut::<
                                                        ::core::ffi::c_void,
                                                    >(
                                                        -1 as ::core::ffi::c_int as usize
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            skip_read = false_0 != 0;
                            if filesize == 0 as off_T
                                && (fio_flags == FIO_UCSBOM as ::core::ffi::c_int
                                    || (*curbuf).b_p_bomb == 0
                                        && tmpname.is_null()
                                        && (*fenc as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                            || *fenc as ::core::ffi::c_int == NUL))
                            {
                                let mut ccname: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut blen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                if size < 2 as ptrdiff_t || (*curbuf).b_p_bin != 0 {
                                    ccname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                } else {
                                    ccname = check_for_bom(
                                        ptr,
                                        size as ::core::ffi::c_int,
                                        &raw mut blen,
                                        if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                            FIO_ALL as ::core::ffi::c_int
                                        } else {
                                            get_fio_flags(fenc)
                                        },
                                    );
                                }
                                if !ccname.is_null() {
                                    filesize += blen as off_T;
                                    size -= blen as ptrdiff_t;
                                    memmove(
                                        ptr as *mut ::core::ffi::c_void,
                                        ptr.offset(blen as isize) as *const ::core::ffi::c_void,
                                        size as size_t,
                                    );
                                    if set_options {
                                        (*curbuf).b_p_bomb = true_0;
                                        (*curbuf).b_start_bomb = true_0;
                                    }
                                }
                                if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                    if ccname.is_null() {
                                        advance_fenc = true_0 != 0;
                                    } else {
                                        if fenc_alloced {
                                            xfree(fenc as *mut ::core::ffi::c_void);
                                        }
                                        fenc = ccname;
                                        fenc_alloced = false_0 != 0;
                                    }
                                    skip_read = true_0 != 0;
                                    break 's_1469;
                                }
                            }
                            ptr = ptr.offset(-(conv_restlen as isize));
                            size += conv_restlen as ptrdiff_t;
                            conv_restlen = 0 as ::core::ffi::c_int;
                            if size <= 0 as ptrdiff_t {
                                break '_failed;
                            }
                            if iconv_fd
                                != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                    -1 as ::core::ffi::c_int as usize,
                                )
                            {
                                let mut fromp: *const ::core::ffi::c_char = ptr;
                                let mut from_size: size_t = size as size_t;
                                ptr = ptr.offset(size as isize);
                                let mut top: *mut ::core::ffi::c_char = ptr;
                                let mut to_size: size_t = (real_size as ptrdiff_t - size) as size_t;
                                while iconv(
                                    iconv_fd,
                                    &raw mut fromp as *mut ::core::ffi::c_void
                                        as *mut *mut ::core::ffi::c_char,
                                    &raw mut from_size,
                                    &raw mut top,
                                    &raw mut to_size,
                                ) == -1 as ::core::ffi::c_int as size_t
                                    && *__errno_location() != ICONV_EINVAL
                                    || from_size > CONV_RESTLEN as ::core::ffi::c_int as size_t
                                {
                                    if can_retry {
                                        break '_rewind_retry;
                                    }
                                    if conv_error == 0 as linenr_T {
                                        conv_error = readfile_linenr(linecnt, ptr, top);
                                    }
                                    fromp = fromp.offset(1);
                                    from_size = from_size.wrapping_sub(1);
                                    if bad_char_behavior == BAD_KEEP {
                                        let c2rust_fresh6 = top;
                                        top = top.offset(1);
                                        *c2rust_fresh6 =
                                            *fromp.offset(-(1 as ::core::ffi::c_int as isize));
                                        to_size = to_size.wrapping_sub(1);
                                    } else if bad_char_behavior != BAD_DROP {
                                        let c2rust_fresh7 = top;
                                        top = top.offset(1);
                                        *c2rust_fresh7 = bad_char_behavior as ::core::ffi::c_char;
                                        to_size = to_size.wrapping_sub(1);
                                    }
                                }
                                if from_size > 0 as size_t {
                                    memmove(
                                        &raw mut conv_rest as *mut ::core::ffi::c_char
                                            as *mut ::core::ffi::c_void,
                                        fromp as *const ::core::ffi::c_void,
                                        from_size,
                                    );
                                    conv_restlen = from_size as ::core::ffi::c_int;
                                }
                                line_start = ptr.offset(-(linerest as isize));
                                memmove(
                                    line_start as *mut ::core::ffi::c_void,
                                    buffer as *const ::core::ffi::c_void,
                                    linerest as size_t,
                                );
                                size = top.offset_from(ptr) as ptrdiff_t;
                            }
                            if fio_flags != 0 as ::core::ffi::c_int {
                                let mut u8c: ::core::ffi::c_uint = 0;
                                let mut tail: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                let mut dest: *mut ::core::ffi::c_char =
                                    ptr.offset(real_size as isize);
                                if fio_flags == FIO_LATIN1 as ::core::ffi::c_int
                                    || fio_flags == FIO_UTF8 as ::core::ffi::c_int
                                {
                                    p = (ptr as *mut uint8_t).offset(size as isize);
                                    if fio_flags == FIO_UTF8 as ::core::ffi::c_int {
                                        tail = ptr
                                            .offset(size as isize)
                                            .offset(-(1 as ::core::ffi::c_int as isize));
                                        while tail > ptr
                                            && *tail as ::core::ffi::c_int
                                                & 0xc0 as ::core::ffi::c_int
                                                == 0x80 as ::core::ffi::c_int
                                        {
                                            tail = tail.offset(-1);
                                        }
                                        if tail.offset(
                                            utf_byte2len(*tail as ::core::ffi::c_int) as isize
                                        ) <= ptr.offset(size as isize)
                                        {
                                            tail = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        } else {
                                            p = tail as *mut uint8_t;
                                        }
                                    }
                                } else if fio_flags
                                    & (FIO_UCS2 as ::core::ffi::c_int
                                        | FIO_UTF16 as ::core::ffi::c_int)
                                    != 0
                                {
                                    p = (ptr as *mut uint8_t).offset(
                                        (size & !(1 as ::core::ffi::c_int) as ptrdiff_t) as isize,
                                    );
                                    if size & 1 as ptrdiff_t != 0 {
                                        tail = p as *mut ::core::ffi::c_char;
                                    }
                                    if fio_flags & FIO_UTF16 as ::core::ffi::c_int != 0
                                        && p > ptr as *mut uint8_t
                                    {
                                        if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                            p = p.offset(-1);
                                            u8c = (*p as ::core::ffi::c_uint)
                                                << 8 as ::core::ffi::c_int;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                        } else {
                                            p = p.offset(-1);
                                            u8c = *p as ::core::ffi::c_uint;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int,
                                            );
                                        }
                                        if u8c >= 0xd800 as ::core::ffi::c_uint
                                            && u8c <= 0xdbff as ::core::ffi::c_uint
                                        {
                                            tail = p as *mut ::core::ffi::c_char;
                                        } else {
                                            p = p.offset(2 as ::core::ffi::c_int as isize);
                                        }
                                    }
                                } else {
                                    p = (ptr as *mut uint8_t).offset(
                                        (size & !(3 as ::core::ffi::c_int) as ptrdiff_t) as isize,
                                    );
                                    if size & 3 as ptrdiff_t != 0 {
                                        tail = p as *mut ::core::ffi::c_char;
                                    }
                                }
                                if !tail.is_null() {
                                    conv_restlen = ptr.offset(size as isize).offset_from(tail)
                                        as ::core::ffi::c_int;
                                    memmove(
                                        &raw mut conv_rest as *mut ::core::ffi::c_char
                                            as *mut ::core::ffi::c_void,
                                        tail as *const ::core::ffi::c_void,
                                        conv_restlen as size_t,
                                    );
                                    size -= conv_restlen as ptrdiff_t;
                                }
                                while p > ptr as *mut uint8_t {
                                    if fio_flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
                                        p = p.offset(-1);
                                        u8c = *p as ::core::ffi::c_uint;
                                    } else if fio_flags
                                        & (FIO_UCS2 as ::core::ffi::c_int
                                            | FIO_UTF16 as ::core::ffi::c_int)
                                        != 0
                                    {
                                        if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                            p = p.offset(-1);
                                            u8c = (*p as ::core::ffi::c_uint)
                                                << 8 as ::core::ffi::c_int;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                        } else {
                                            p = p.offset(-1);
                                            u8c = *p as ::core::ffi::c_uint;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int,
                                            );
                                        }
                                        if fio_flags & FIO_UTF16 as ::core::ffi::c_int != 0
                                            && u8c >= 0xdc00 as ::core::ffi::c_uint
                                            && u8c <= 0xdfff as ::core::ffi::c_uint
                                        {
                                            let mut u16c: ::core::ffi::c_int = 0;
                                            if p == ptr as *mut uint8_t {
                                                if can_retry {
                                                    break '_rewind_retry;
                                                }
                                                if conv_error == 0 as linenr_T {
                                                    conv_error = readfile_linenr(
                                                        linecnt,
                                                        ptr,
                                                        p as *mut ::core::ffi::c_char,
                                                    );
                                                }
                                                if bad_char_behavior == BAD_DROP {
                                                    continue;
                                                }
                                                if bad_char_behavior != BAD_KEEP {
                                                    u8c = bad_char_behavior as ::core::ffi::c_uint;
                                                }
                                            }
                                            if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                                p = p.offset(-1);
                                                u16c = (*p as ::core::ffi::c_int)
                                                    << 8 as ::core::ffi::c_int;
                                                p = p.offset(-1);
                                                u16c += *p as ::core::ffi::c_int;
                                            } else {
                                                p = p.offset(-1);
                                                u16c = *p as ::core::ffi::c_int;
                                                p = p.offset(-1);
                                                u16c += (*p as ::core::ffi::c_int)
                                                    << 8 as ::core::ffi::c_int;
                                            }
                                            u8c = (0x10000 as ::core::ffi::c_int
                                                as ::core::ffi::c_uint)
                                                .wrapping_add(
                                                    (u16c as ::core::ffi::c_uint
                                                        & 0x3ff as ::core::ffi::c_uint)
                                                        << 10 as ::core::ffi::c_int,
                                                )
                                                .wrapping_add(u8c & 0x3ff as ::core::ffi::c_uint);
                                            if u16c < 0xd800 as ::core::ffi::c_int
                                                || u16c > 0xdbff as ::core::ffi::c_int
                                            {
                                                if can_retry {
                                                    break '_rewind_retry;
                                                }
                                                if conv_error == 0 as linenr_T {
                                                    conv_error = readfile_linenr(
                                                        linecnt,
                                                        ptr,
                                                        p as *mut ::core::ffi::c_char,
                                                    );
                                                }
                                                if bad_char_behavior == BAD_DROP {
                                                    continue;
                                                }
                                                if bad_char_behavior != BAD_KEEP {
                                                    u8c = bad_char_behavior as ::core::ffi::c_uint;
                                                }
                                            }
                                        }
                                    } else if fio_flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                                        if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                            p = p.offset(-1);
                                            u8c = (*p as ::core::ffi::c_uint)
                                                << 24 as ::core::ffi::c_int;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 16 as ::core::ffi::c_int,
                                            );
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int,
                                            );
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                        } else {
                                            p = p.offset(-1);
                                            u8c = *p as ::core::ffi::c_uint;
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int,
                                            );
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 16 as ::core::ffi::c_int,
                                            );
                                            p = p.offset(-1);
                                            u8c = u8c.wrapping_add(
                                                (*p as ::core::ffi::c_uint)
                                                    << 24 as ::core::ffi::c_int,
                                            );
                                        }
                                        if u8c > INT_MAX as ::core::ffi::c_uint {
                                            u8c = 0xfffd as ::core::ffi::c_uint;
                                        }
                                    } else {
                                        p = p.offset(-1);
                                        if (*p as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
                                            u8c = *p as ::core::ffi::c_uint;
                                        } else {
                                            len = utf_head_off(ptr, p as *mut ::core::ffi::c_char)
                                                as colnr_T;
                                            p = p.offset(-(len as isize));
                                            u8c = utf_ptr2char(p as *mut ::core::ffi::c_char)
                                                as ::core::ffi::c_uint;
                                            if len == 0 as ::core::ffi::c_int {
                                                if can_retry {
                                                    break '_rewind_retry;
                                                }
                                                if conv_error == 0 as linenr_T {
                                                    conv_error = readfile_linenr(
                                                        linecnt,
                                                        ptr,
                                                        p as *mut ::core::ffi::c_char,
                                                    );
                                                }
                                                if bad_char_behavior == BAD_DROP {
                                                    continue;
                                                }
                                                if bad_char_behavior != BAD_KEEP {
                                                    u8c = bad_char_behavior as ::core::ffi::c_uint;
                                                }
                                            }
                                        }
                                    }
                                    '_c2rust_label: {
                                        if u8c
                                            <= 2147483647 as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                        } else {
                                            __assert_fail(
                                                b"u8c <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/fileio.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1328 as ::core::ffi::c_uint,
                                                b"int readfile(char *, char *, linenr_T, linenr_T, linenr_T, exarg_T *, int, _Bool)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    dest = dest.offset(
                                        -(utf_char2len(u8c as ::core::ffi::c_int) as isize),
                                    );
                                    utf_char2bytes(u8c as ::core::ffi::c_int, dest);
                                }
                                line_start = dest.offset(-(linerest as isize));
                                memmove(
                                    line_start as *mut ::core::ffi::c_void,
                                    buffer as *const ::core::ffi::c_void,
                                    linerest as size_t,
                                );
                                size =
                                    ptr.offset(real_size as isize).offset_from(dest) as ptrdiff_t;
                                ptr = dest;
                            } else if (*curbuf).b_p_bin == 0 {
                                incomplete_tail = false_0 != 0;
                                p = ptr as *mut uint8_t;
                                loop {
                                    let mut ascii_end: *mut uint8_t =
                                        (ptr as *mut uint8_t).offset(size as isize);
                                    while ascii_end.offset_from(p)
                                        >= ::core::mem::size_of::<uint64_t>() as ptrdiff_t
                                    {
                                        let mut word: uint64_t = 0;
                                        memcpy(
                                            &raw mut word as *mut ::core::ffi::c_void,
                                            p as *const ::core::ffi::c_void,
                                            ::core::mem::size_of::<uint64_t>(),
                                        );
                                        if word & NONASCII_MASK != 0 {
                                            break;
                                        }
                                        p = p.offset(::core::mem::size_of::<uint64_t>() as isize);
                                    }
                                    while p < ascii_end
                                        && (*p as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
                                    {
                                        p = p.offset(1);
                                    }
                                    let mut todo: ::core::ffi::c_int =
                                        (ptr as *mut uint8_t).offset(size as isize).offset_from(p)
                                            as ::core::ffi::c_int;
                                    if todo <= 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                    if (*p as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
                                        continue;
                                    }
                                    let mut l: ::core::ffi::c_int =
                                        utf_ptr2len_len(p as *mut ::core::ffi::c_char, todo);
                                    if l > todo && !incomplete_tail {
                                        if p > ptr as *mut uint8_t || filesize > 0 as off_T {
                                            incomplete_tail = true_0 != 0;
                                        }
                                        if p > ptr as *mut uint8_t {
                                            conv_restlen = todo;
                                            memmove(
                                                &raw mut conv_rest as *mut ::core::ffi::c_char
                                                    as *mut ::core::ffi::c_void,
                                                p as *const ::core::ffi::c_void,
                                                conv_restlen as size_t,
                                            );
                                            size -= conv_restlen as ptrdiff_t;
                                            break;
                                        }
                                    }
                                    if l == 1 as ::core::ffi::c_int || l > todo {
                                        if can_retry as ::core::ffi::c_int != 0 && !incomplete_tail
                                        {
                                            break;
                                        }
                                        if iconv_fd
                                            != ::core::ptr::from_exposed_addr_mut::<
                                                ::core::ffi::c_void,
                                            >(
                                                -1 as ::core::ffi::c_int as usize
                                            )
                                            && conv_error == 0 as linenr_T
                                        {
                                            conv_error = readfile_linenr(
                                                linecnt,
                                                ptr,
                                                p as *mut ::core::ffi::c_char,
                                            );
                                        }
                                        if conv_error == 0 as linenr_T
                                            && illegal_byte == 0 as linenr_T
                                        {
                                            illegal_byte = readfile_linenr(
                                                linecnt,
                                                ptr,
                                                p as *mut ::core::ffi::c_char,
                                            );
                                        }
                                        if bad_char_behavior == BAD_DROP {
                                            memmove(
                                                p as *mut ::core::ffi::c_void,
                                                p.offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                (todo - 1 as ::core::ffi::c_int) as size_t,
                                            );
                                            size -= 1;
                                        } else {
                                            if bad_char_behavior != BAD_KEEP {
                                                *p = bad_char_behavior as uint8_t;
                                            }
                                            p = p.offset(1);
                                        }
                                    } else {
                                        p = p.offset(l as isize);
                                    }
                                }
                                if p < (ptr as *mut uint8_t).offset(size as isize)
                                    && !incomplete_tail
                                {
                                    break '_rewind_retry;
                                }
                            }
                            filesize += size as ::core::ffi::c_long;
                            if fileformat == EOL_UNKNOWN {
                                if try_dos != 0 || try_unix != 0 {
                                    if try_mac != 0 {
                                        try_mac = 1 as ::core::ffi::c_int;
                                    }
                                    p = ptr as *mut uint8_t;
                                    while p < (ptr as *mut uint8_t).offset(size as isize) {
                                        if *p as ::core::ffi::c_int == NL {
                                            if try_unix == 0
                                                || try_dos != 0
                                                    && p > ptr as *mut uint8_t
                                                    && *p.offset(-1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == CAR
                                            {
                                                fileformat = EOL_DOS;
                                            } else {
                                                fileformat = EOL_UNIX;
                                            }
                                            break;
                                        } else {
                                            if *p as ::core::ffi::c_int == CAR && try_mac != 0 {
                                                try_mac += 1;
                                            }
                                            p = p.offset(1);
                                        }
                                    }
                                    if fileformat == EOL_UNIX && try_mac != 0 {
                                        try_mac = 1 as ::core::ffi::c_int;
                                        try_unix = 1 as ::core::ffi::c_int;
                                        while p >= ptr as *mut uint8_t
                                            && *p as ::core::ffi::c_int != CAR
                                        {
                                            p = p.offset(-1);
                                        }
                                        if p >= ptr as *mut uint8_t {
                                            p = ptr as *mut uint8_t;
                                            while p < (ptr as *mut uint8_t).offset(size as isize) {
                                                if *p as ::core::ffi::c_int == NL {
                                                    try_unix += 1;
                                                } else if *p as ::core::ffi::c_int == CAR {
                                                    try_mac += 1;
                                                }
                                                p = p.offset(1);
                                            }
                                            if try_mac > try_unix {
                                                fileformat = EOL_MAC;
                                            }
                                        }
                                    } else if fileformat == EOL_UNKNOWN
                                        && try_mac == 1 as ::core::ffi::c_int
                                    {
                                        fileformat = default_fileformat();
                                    }
                                }
                                if fileformat == EOL_UNKNOWN && try_mac != 0 {
                                    fileformat = EOL_MAC;
                                }
                                if fileformat == EOL_UNKNOWN {
                                    fileformat = default_fileformat();
                                }
                                if set_options {
                                    set_fileformat(fileformat, OPT_LOCAL as ::core::ffi::c_int);
                                }
                            }
                            if fileformat == EOL_MAC {
                                ptr = ptr.offset(-1);
                                loop {
                                    ptr = ptr.offset(1);
                                    size -= 1;
                                    if size < 0 as ptrdiff_t {
                                        break;
                                    }
                                    c = *ptr;
                                    if c as ::core::ffi::c_int != NUL
                                        && c as ::core::ffi::c_int != CAR
                                        && c as ::core::ffi::c_int != NL
                                    {
                                        continue;
                                    }
                                    if c as ::core::ffi::c_int == NUL {
                                        *ptr = NL as ::core::ffi::c_char;
                                    } else if c as ::core::ffi::c_int == NL {
                                        *ptr = CAR as ::core::ffi::c_char;
                                    } else {
                                        if skip_count == 0 as linenr_T {
                                            *ptr = NUL as ::core::ffi::c_char;
                                            len = (ptr.offset_from(line_start) + 1 as isize)
                                                as colnr_T;
                                            if ml_append(lnum, line_start, len, newfile) == FAIL {
                                                error = true_0 != 0;
                                                break;
                                            } else {
                                                if read_undo_file {
                                                    sha256_update(
                                                        &raw mut sha_ctx,
                                                        line_start as *mut uint8_t,
                                                        len as size_t,
                                                    );
                                                }
                                                lnum += 1;
                                                read_count -= 1;
                                                if read_count == 0 as linenr_T {
                                                    error = true_0 != 0;
                                                    line_start = ptr;
                                                    break;
                                                }
                                            }
                                        } else {
                                            skip_count -= 1;
                                        }
                                        line_start = ptr.offset(1 as ::core::ffi::c_int as isize);
                                    }
                                }
                            } else {
                                let mut end: *mut ::core::ffi::c_char = ptr.offset(size as isize);
                                while ptr < end {
                                    let mut nl: *mut ::core::ffi::c_char = memchr(
                                        ptr as *const ::core::ffi::c_void,
                                        NL,
                                        end.offset_from(ptr) as size_t,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    let mut nul_scan: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    if nl.is_null() {
                                        loop {
                                            nul_scan = memchr(
                                                ptr as *const ::core::ffi::c_void,
                                                NUL,
                                                end.offset_from(ptr) as size_t,
                                            )
                                                as *mut ::core::ffi::c_char;
                                            if nul_scan.is_null() {
                                                break;
                                            }
                                            *nul_scan = NL as ::core::ffi::c_char;
                                            ptr = nul_scan.offset(1 as ::core::ffi::c_int as isize);
                                        }
                                        ptr = end;
                                        break;
                                    } else {
                                        let mut scan: *mut ::core::ffi::c_char = ptr;
                                        loop {
                                            nul_scan = memchr(
                                                scan as *const ::core::ffi::c_void,
                                                NUL,
                                                nl.offset_from(scan) as size_t,
                                            )
                                                as *mut ::core::ffi::c_char;
                                            if nul_scan.is_null() {
                                                break;
                                            }
                                            *nul_scan = NL as ::core::ffi::c_char;
                                            scan =
                                                nul_scan.offset(1 as ::core::ffi::c_int as isize);
                                        }
                                        ptr = nl;
                                        if skip_count == 0 as linenr_T {
                                            *ptr = NUL as ::core::ffi::c_char;
                                            len = (ptr.offset_from(line_start) + 1 as isize)
                                                as colnr_T;
                                            if fileformat == EOL_DOS {
                                                if ptr > line_start
                                                    && *ptr
                                                        .offset(-1 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int
                                                        == CAR
                                                {
                                                    *ptr.offset(
                                                        -1 as ::core::ffi::c_int as isize,
                                                    ) = NUL as ::core::ffi::c_char;
                                                    len -= 1;
                                                } else if ff_error != EOL_DOS {
                                                    if try_unix != 0
                                                        && !read_stdin
                                                        && (read_buffer as ::core::ffi::c_int != 0
                                                            || lseek(fd, 0 as __off_t, SEEK_SET)
                                                                == 0 as __off_t)
                                                    {
                                                        fileformat = EOL_UNIX;
                                                        if set_options {
                                                            set_fileformat(
                                                                EOL_UNIX,
                                                                OPT_LOCAL as ::core::ffi::c_int,
                                                            );
                                                        }
                                                        file_rewind = true_0 != 0;
                                                        keep_fileformat = true_0 != 0;
                                                        continue '_failed;
                                                    } else {
                                                        ff_error = EOL_DOS;
                                                    }
                                                }
                                            }
                                            if ml_append(lnum, line_start, len, newfile) == FAIL {
                                                error = true_0 != 0;
                                                break;
                                            } else {
                                                if read_undo_file {
                                                    sha256_update(
                                                        &raw mut sha_ctx,
                                                        line_start as *mut uint8_t,
                                                        len as size_t,
                                                    );
                                                }
                                                lnum += 1;
                                                read_count -= 1;
                                                if read_count == 0 as linenr_T {
                                                    error = true_0 != 0;
                                                    line_start = ptr;
                                                    break;
                                                }
                                            }
                                        } else {
                                            skip_count -= 1;
                                        }
                                        line_start = ptr.offset(1 as ::core::ffi::c_int as isize);
                                        ptr = ptr.offset(1);
                                    }
                                }
                                size = -1 as ptrdiff_t;
                            }
                            linerest = ptr.offset_from(line_start) as ptrdiff_t;
                            os_breakcheck();
                            continue 's_1469;
                        }
                        if *p_ccv as ::core::ffi::c_int != NUL
                            && iconv_fd
                                != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                    -1 as ::core::ffi::c_int as usize,
                                )
                        {
                            did_iconv = true_0 != 0;
                        } else {
                            advance_fenc = true_0 != 0;
                        }
                        file_rewind = true_0 != 0;
                        break;
                    }
                }
                if error as ::core::ffi::c_int != 0 && read_count == 0 as linenr_T {
                    error = false_0 != 0;
                }
                if linerest != 0 as ptrdiff_t
                    && (*curbuf).b_p_bin == 0
                    && fileformat == EOL_DOS
                    && *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == Ctrl_Z
                {
                    ptr = ptr.offset(-1);
                    linerest -= 1;
                    if set_options {
                        (*curbuf).b_p_eof = true_0;
                    }
                }
                if !error && !got_int && linerest != 0 as ptrdiff_t {
                    if set_options {
                        (*curbuf).b_p_eol = false_0;
                    }
                    *ptr = NUL as ::core::ffi::c_char;
                    len = (ptr.offset_from(line_start) + 1 as isize) as colnr_T;
                    if ml_append(lnum, line_start, len, newfile) == FAIL {
                        error = true_0 != 0;
                    } else {
                        if read_undo_file {
                            sha256_update(
                                &raw mut sha_ctx,
                                line_start as *mut uint8_t,
                                len as size_t,
                            );
                        }
                        lnum += 1;
                        read_no_eol_lnum = lnum;
                    }
                }
                if set_options {
                    save_file_ff(curbuf);
                    set_option_direct(
                        kOptFileencoding,
                        OptVal {
                            type_0: kOptValTypeString,
                            data: OptValData {
                                string: cstr_as_string(fenc),
                            },
                        },
                        OPT_LOCAL as ::core::ffi::c_int,
                        0 as scid_T,
                    );
                }
                if fenc_alloced {
                    xfree(fenc as *mut ::core::ffi::c_void);
                }
                if iconv_fd
                    != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        -1 as ::core::ffi::c_int as usize,
                    )
                {
                    iconv_close(iconv_fd);
                }
                if !read_buffer && !read_stdin {
                    close(fd);
                } else {
                    os_set_cloexec(fd);
                }
                xfree(buffer as *mut ::core::ffi::c_void);
                if read_stdin {
                    close(fd);
                    if stdin_fd < 0 as ::core::ffi::c_int {
                        vim_ignored = dup(2 as ::core::ffi::c_int);
                    }
                }
                if !tmpname.is_null() {
                    os_remove(tmpname);
                    xfree(tmpname as *mut ::core::ffi::c_void);
                }
                no_wait_return -= 1;
                if !recoverymode {
                    if newfile as ::core::ffi::c_int != 0
                        && wasempty != 0
                        && (*curbuf).b_ml.ml_flags & ML_EMPTY == 0
                    {
                        ml_delete((*curbuf).b_ml.ml_line_count);
                        linecnt -= 1;
                    }
                    (*curbuf).deleted_bytes = 0 as size_t;
                    (*curbuf).deleted_bytes2 = 0 as size_t;
                    (*curbuf).deleted_codepoints = 0 as size_t;
                    (*curbuf).deleted_codeunits = 0 as size_t;
                    linecnt = (*curbuf).b_ml.ml_line_count - linecnt;
                    if filesize == 0 as off_T {
                        linecnt = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    if newfile as ::core::ffi::c_int != 0 || read_buffer as ::core::ffi::c_int != 0
                    {
                        redraw_curbuf_later(UPD_NOT_VALID as ::core::ffi::c_int);
                        diff_invalidate(curbuf);
                        foldUpdateAll(curwin);
                    } else if linecnt != 0 {
                        appended_lines_mark(from, linecnt as ::core::ffi::c_int);
                    }
                    if got_int {
                        if flags & READ_DUMMY as ::core::ffi::c_int == 0 {
                            filemess(
                                curbuf,
                                sfname,
                                gettext(&raw const e_interr as *const ::core::ffi::c_char),
                            );
                            if newfile {
                                (*curbuf).b_p_ro = true_0;
                            }
                        }
                        msg_scroll = msg_save;
                        check_marks_read();
                        retval = OK;
                        break '_theend;
                    } else {
                        if !filtering && flags & READ_DUMMY as ::core::ffi::c_int == 0 && !silent {
                            add_quoted_fname(
                                &raw mut IObuff as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                curbuf,
                                sfname,
                            );
                            c = false_0 as ::core::ffi::c_char;
                            let mut buflen: ::core::ffi::c_int =
                                strlen(&raw mut IObuff as *mut ::core::ffi::c_char)
                                    as ::core::ffi::c_int;
                            if perm & __S_IFMT == 0o10000 as ::core::ffi::c_int {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[fifo]\0".as_ptr() as *const ::core::ffi::c_char),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if perm & __S_IFMT == 0o140000 as ::core::ffi::c_int {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[socket]\0".as_ptr() as *const ::core::ffi::c_char),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if (*curbuf).b_p_ro != 0 {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    if shortmess(SHM_RO as ::core::ffi::c_int) as ::core::ffi::c_int
                                        != 0
                                    {
                                        gettext(b"[RO]\0".as_ptr() as *const ::core::ffi::c_char)
                                    } else {
                                        gettext(
                                            b"[readonly]\0".as_ptr() as *const ::core::ffi::c_char
                                        )
                                    },
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if read_no_eol_lnum != 0 {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[noeol]\0".as_ptr() as *const ::core::ffi::c_char),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if ff_error == EOL_DOS {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(
                                        b"[CR missing]\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if split != 0 {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[long lines split]\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if notconverted {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(
                                        b"[NOT converted]\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            } else if converted {
                                buflen += snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[converted]\0".as_ptr() as *const ::core::ffi::c_char),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if conv_error != 0 as linenr_T {
                                snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[CONVERSION ERROR in line %ld]\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    conv_error as int64_t,
                                );
                                c = true_0 as ::core::ffi::c_char;
                            } else if illegal_byte > 0 as linenr_T {
                                snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(b"[ILLEGAL BYTE in line %ld]\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    illegal_byte as int64_t,
                                );
                                c = true_0 as ::core::ffi::c_char;
                            } else if error {
                                snprintf(
                                    (&raw mut IObuff as *mut ::core::ffi::c_char)
                                        .offset(buflen as isize),
                                    (IOSIZE - buflen) as size_t,
                                    gettext(
                                        b"[READ ERRORS]\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                                c = true_0 as ::core::ffi::c_char;
                            }
                            if msg_add_fileformat(fileformat) {
                                c = true_0 as ::core::ffi::c_char;
                            }
                            msg_add_lines(c as ::core::ffi::c_int, linecnt, filesize);
                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                &raw mut keep_msg as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__0);
                            *ptr__0 = NULL;
                            *ptr__0;
                            p = ::core::ptr::null_mut::<uint8_t>();
                            msg_scrolled_ign = true_0 != 0;
                            if !read_stdin && !read_buffer {
                                if msg_col > 0 as ::core::ffi::c_int {
                                    msg_putchar('\r' as ::core::ffi::c_int);
                                }
                                p = msg_trunc(
                                    &raw mut IObuff as *mut ::core::ffi::c_char,
                                    false_0 != 0,
                                    0 as ::core::ffi::c_int,
                                ) as *mut uint8_t;
                            }
                            if read_stdin as ::core::ffi::c_int != 0
                                || read_buffer as ::core::ffi::c_int != 0
                                || restart_edit != 0 as ::core::ffi::c_int
                                || msg_scrolled != 0 as ::core::ffi::c_int && !need_wait_return
                            {
                                set_keep_msg(
                                    p as *mut ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                );
                            }
                            msg_scrolled_ign = false_0 != 0;
                        }
                        if newfile as ::core::ffi::c_int != 0
                            && (error as ::core::ffi::c_int != 0
                                || conv_error != 0 as linenr_T
                                || illegal_byte > 0 as linenr_T && bad_char_behavior != BAD_KEEP)
                        {
                            (*curbuf).b_p_ro = true_0;
                        }
                        u_clearline(curbuf);
                        if exmode_active {
                            (*curwin).w_cursor.lnum = from + linecnt;
                        } else {
                            (*curwin).w_cursor.lnum = from + 1 as linenr_T;
                        }
                        check_cursor_lnum(curwin);
                        beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
                        if cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_int
                        {
                            (*curbuf).b_op_start.lnum = from + 1 as linenr_T;
                            (*curbuf).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                            (*curbuf).b_op_end.lnum = from + linecnt;
                            (*curbuf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                    }
                }
                msg_scroll = msg_save;
                check_marks_read();
                (*curbuf).b_no_eol_lnum = read_no_eol_lnum;
                if flags & READ_KEEP_UNDO as ::core::ffi::c_int != 0 {
                    u_find_first_changed();
                }
                if read_undo_file {
                    let mut hash: [uint8_t; 32] = [0; 32];
                    sha256_finish(&raw mut sha_ctx, &raw mut hash as *mut uint8_t);
                    u_read_undo(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut hash as *mut uint8_t,
                        fname,
                    );
                }
                if !read_stdin && !read_fifo && (!read_buffer || !sfname.is_null()) {
                    let mut m_0: ::core::ffi::c_int = msg_scroll;
                    let mut n_1: ::core::ffi::c_int = msg_scrolled;
                    if set_options {
                        save_file_ff(curbuf);
                    }
                    msg_scroll = true_0;
                    if filtering {
                        apply_autocmds_exarg(
                            EVENT_FILTERREADPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            sfname,
                            false_0 != 0,
                            curbuf,
                            eap,
                        );
                    } else if newfile as ::core::ffi::c_int != 0
                        || read_buffer as ::core::ffi::c_int != 0 && !sfname.is_null()
                    {
                        apply_autocmds_exarg(
                            EVENT_BUFREADPOST,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            sfname,
                            false_0 != 0,
                            curbuf,
                            eap,
                        );
                        if !(*curbuf).b_au_did_filetype
                            && *(*curbuf).b_p_ft as ::core::ffi::c_int != NUL
                        {
                            apply_autocmds(
                                EVENT_FILETYPE,
                                (*curbuf).b_p_ft,
                                (*curbuf).b_fname,
                                true_0 != 0,
                                curbuf,
                            );
                        }
                    } else {
                        apply_autocmds_exarg(
                            EVENT_FILEREADPOST,
                            sfname,
                            sfname,
                            false_0 != 0,
                            ::core::ptr::null_mut::<buf_T>(),
                            eap,
                        );
                    }
                    if msg_scrolled == n_1 {
                        msg_scroll = m_0;
                    }
                    if aborting() {
                        return FAIL;
                    }
                }
                if !(recoverymode as ::core::ffi::c_int != 0 && error as ::core::ffi::c_int != 0) {
                    retval = OK;
                }
            }
        }
    }
    if !(*curbuf).b_ml.ml_mfp.is_null()
        && (*(*curbuf).b_ml.ml_mfp).mf_dirty as ::core::ffi::c_uint
            == MF_DIRTY_YES_NOSYNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*(*curbuf).b_ml.ml_mfp).mf_dirty = MF_DIRTY_YES;
    }
    return retval;
}
unsafe extern "C" fn readfile_linenr(
    mut linecnt: linenr_T,
    mut p: *mut ::core::ffi::c_char,
    mut endp: *const ::core::ffi::c_char,
) -> linenr_T {
    let mut lnum: linenr_T = (*curbuf).b_ml.ml_line_count - linecnt + 1 as linenr_T;
    let mut s: *mut ::core::ffi::c_char = p;
    while s < endp as *mut ::core::ffi::c_char {
        if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            lnum += 1;
        }
        s = s.offset(1);
    }
    return lnum;
}
#[no_mangle]
pub unsafe extern "C" fn prep_exarg(mut eap: *mut exarg_T, mut buf: *const buf_T) {
    let cmd_len: size_t = (15 as size_t).wrapping_add(strlen((*buf).b_p_fenc));
    (*eap).cmd = xmalloc(cmd_len) as *mut ::core::ffi::c_char;
    snprintf(
        (*eap).cmd,
        cmd_len,
        b"e ++enc=%s\0".as_ptr() as *const ::core::ffi::c_char,
        (*buf).b_p_fenc,
    );
    (*eap).force_enc = 8 as ::core::ffi::c_int;
    (*eap).bad_char = (*buf).b_bad_char;
    (*eap).force_ff = *(*buf).b_p_ff as ::core::ffi::c_uchar as ::core::ffi::c_int;
    (*eap).force_bin = if (*buf).b_p_bin != 0 {
        FORCE_BIN
    } else {
        FORCE_NOBIN
    };
    (*eap).read_edit = false_0;
    (*eap).forceit = false_0;
}
#[no_mangle]
pub unsafe extern "C" fn set_file_options(mut set_options: bool, mut eap: *mut exarg_T) {
    if set_options {
        if !eap.is_null() && (*eap).force_ff != 0 as ::core::ffi::c_int {
            set_fileformat(
                get_fileformat_force(curbuf, eap),
                OPT_LOCAL as ::core::ffi::c_int,
            );
        } else if *p_ffs as ::core::ffi::c_int != NUL {
            set_fileformat(default_fileformat(), OPT_LOCAL as ::core::ffi::c_int);
        }
    }
    if !eap.is_null() && (*eap).force_bin != 0 as ::core::ffi::c_int {
        let mut oldval: ::core::ffi::c_int = (*curbuf).b_p_bin;
        (*curbuf).b_p_bin = ((*eap).force_bin == FORCE_BIN) as ::core::ffi::c_int;
        set_options_bin(oldval, (*curbuf).b_p_bin, OPT_LOCAL as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_forced_fenc(mut eap: *mut exarg_T) {
    if (*eap).force_enc == 0 as ::core::ffi::c_int {
        return;
    }
    let mut fenc: *mut ::core::ffi::c_char =
        enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
    set_option_direct(
        kOptFileencoding,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(fenc),
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
    );
    xfree(fenc as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn next_fenc(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut alloced: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut r: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *alloced = false_0 != 0;
    if **pp as ::core::ffi::c_int == NUL {
        *pp = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    let mut p: *mut ::core::ffi::c_char = vim_strchr(*pp, ',' as ::core::ffi::c_int);
    if p.is_null() {
        r = enc_canonize(*pp);
        *pp = (*pp).offset(strlen(*pp) as isize);
    } else {
        r = xmemdupz(
            *pp as *const ::core::ffi::c_void,
            p.offset_from(*pp) as size_t,
        ) as *mut ::core::ffi::c_char;
        *pp = p.offset(1 as ::core::ffi::c_int as isize);
        p = enc_canonize(r);
        xfree(r as *mut ::core::ffi::c_void);
        r = p;
    }
    *alloced = true_0 != 0;
    return r;
}
unsafe extern "C" fn readfile_charconvert(
    mut fname: *mut ::core::ffi::c_char,
    mut fenc: *mut ::core::ffi::c_char,
    mut fdp: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut errmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmpname: *mut ::core::ffi::c_char = vim_tempname();
    if tmpname.is_null() {
        errmsg = gettext(
            b"Can't find temp file for conversion\0".as_ptr() as *const ::core::ffi::c_char
        );
    } else {
        close(*fdp);
        *fdp = -1 as ::core::ffi::c_int;
        if eval_charconvert(
            fenc,
            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
            fname,
            tmpname,
        ) == FAIL
        {
            errmsg =
                gettext(b"Conversion with 'charconvert' failed\0".as_ptr()
                    as *const ::core::ffi::c_char);
        }
        if errmsg.is_null() && {
            *fdp = os_open(tmpname, O_RDONLY, 0 as ::core::ffi::c_int);
            *fdp < 0 as ::core::ffi::c_int
        } {
            errmsg = gettext(
                b"can't read output of 'charconvert'\0".as_ptr() as *const ::core::ffi::c_char
            );
        }
    }
    if !errmsg.is_null() {
        msg(errmsg, 0 as ::core::ffi::c_int);
        if !tmpname.is_null() {
            os_remove(tmpname);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut tmpname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        }
    }
    if *fdp < 0 as ::core::ffi::c_int {
        *fdp = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
    }
    return tmpname;
}
#[no_mangle]
pub unsafe extern "C" fn set_rw_fname(
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = curbuf;
    if (*curbuf).b_p_bl != 0 {
        apply_autocmds(
            EVENT_BUFDELETE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
    }
    apply_autocmds(
        EVENT_BUFWIPEOUT,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if aborting() {
        return FAIL;
    }
    if curbuf != buf {
        emsg(gettext(e_auchangedbuf.get()));
        return FAIL;
    }
    if setfname(curbuf, fname, sfname, false_0 != 0) == OK {
        (*curbuf).b_flags |= BF_NOTEDITED;
    }
    apply_autocmds(
        EVENT_BUFNEW,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if (*curbuf).b_p_bl != 0 {
        apply_autocmds(
            EVENT_BUFADD,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf,
        );
    }
    if aborting() {
        return FAIL;
    }
    if *(*curbuf).b_p_ft as ::core::ffi::c_int == NUL {
        if augroup_exists(b"filetypedetect\0".as_ptr() as *const ::core::ffi::c_char) {
            do_doautocmd(
                b"filetypedetect BufRead\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
        }
        do_modelines(0 as ::core::ffi::c_int);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn add_quoted_fname(
    ret_buf: *mut ::core::ffi::c_char,
    buf_len: size_t,
    buf: *const buf_T,
    mut fname: *const ::core::ffi::c_char,
) {
    if fname.is_null() {
        fname = b"-stdin-\0".as_ptr() as *const ::core::ffi::c_char;
    }
    *ret_buf.offset(0 as ::core::ffi::c_int as isize) = '"' as ::core::ffi::c_char;
    home_replace(
        buf,
        fname,
        ret_buf.offset(1 as ::core::ffi::c_int as isize),
        buf_len.wrapping_sub(4 as size_t),
        true_0 != 0,
    );
    xstrlcat(
        ret_buf,
        b"\" \0".as_ptr() as *const ::core::ffi::c_char,
        buf_len,
    );
}
#[no_mangle]
pub unsafe extern "C" fn msg_add_fileformat(mut eol_type: ::core::ffi::c_int) -> bool {
    if eol_type == EOL_DOS {
        xstrlcat(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            gettext(b"[dos]\0".as_ptr() as *const ::core::ffi::c_char),
            IOSIZE as size_t,
        );
        return true_0 != 0;
    }
    if eol_type == EOL_MAC {
        xstrlcat(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            gettext(b"[mac]\0".as_ptr() as *const ::core::ffi::c_char),
            IOSIZE as size_t,
        );
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn msg_add_lines(
    mut insert_space: ::core::ffi::c_int,
    mut lnum: linenr_T,
    mut nchars: off_T,
) {
    let mut len: size_t = strlen(&raw mut IObuff as *mut ::core::ffi::c_char);
    if shortmess(SHM_LINES as ::core::ffi::c_int) {
        snprintf(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(len as isize),
            (IOSIZE as size_t).wrapping_sub(len),
            gettext(b"%s%ldL, %ldB\0".as_ptr() as *const ::core::ffi::c_char),
            if insert_space != 0 {
                b" \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            lnum as int64_t,
            nchars as int64_t,
        );
    } else {
        len = len.wrapping_add(snprintf(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(len as isize),
            (IOSIZE as size_t).wrapping_sub(len),
            ngettext(
                b"%s%ld line, \0".as_ptr() as *const ::core::ffi::c_char,
                b"%s%ld lines, \0".as_ptr() as *const ::core::ffi::c_char,
                lnum as ::core::ffi::c_ulong,
            ),
            if insert_space != 0 {
                b" \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            lnum as int64_t,
        ) as size_t);
        snprintf(
            (&raw mut IObuff as *mut ::core::ffi::c_char).offset(len as isize),
            (IOSIZE as size_t).wrapping_sub(len),
            ngettext(
                b"%ld byte\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld bytes\0".as_ptr() as *const ::core::ffi::c_char,
                nchars as ::core::ffi::c_ulong,
            ),
            nchars as int64_t,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn time_differs(
    mut file_info: *const FileInfo,
    mut mtime: int64_t,
    mut mtime_ns: int64_t,
) -> bool {
    return (*file_info).stat.st_mtim.tv_nsec as int64_t != mtime_ns
        || (*file_info).stat.st_mtim.tv_sec as int64_t - mtime > 1 as int64_t
        || mtime - (*file_info).stat.st_mtim.tv_sec as int64_t > 1 as int64_t;
}
#[no_mangle]
pub unsafe extern "C" fn need_conversion(mut fenc: *const ::core::ffi::c_char) -> bool {
    let mut same_encoding: bool = false;
    let mut fenc_flags: ::core::ffi::c_int = 0;
    if *fenc as ::core::ffi::c_int == NUL || strcmp(p_enc, fenc) == 0 as ::core::ffi::c_int {
        same_encoding = true_0 != 0;
        fenc_flags = 0 as ::core::ffi::c_int;
    } else {
        let mut enc_flags: ::core::ffi::c_int = get_fio_flags(p_enc);
        fenc_flags = get_fio_flags(fenc);
        same_encoding = enc_flags != 0 as ::core::ffi::c_int && fenc_flags == enc_flags;
    }
    if same_encoding {
        return false_0 != 0;
    }
    return !(fenc_flags == FIO_UTF8 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn get_fio_flags(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *name as ::core::ffi::c_int == NUL {
        name = p_enc;
    }
    let mut prop: ::core::ffi::c_int = enc_canon_props(name);
    if prop & ENC_UNICODE as ::core::ffi::c_int != 0 {
        if prop & ENC_2BYTE as ::core::ffi::c_int != 0 {
            if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                return FIO_UCS2 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
            }
            return FIO_UCS2 as ::core::ffi::c_int;
        }
        if prop & ENC_4BYTE as ::core::ffi::c_int != 0 {
            if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                return FIO_UCS4 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
            }
            return FIO_UCS4 as ::core::ffi::c_int;
        }
        if prop & ENC_2WORD as ::core::ffi::c_int != 0 {
            if prop & ENC_ENDIAN_L as ::core::ffi::c_int != 0 {
                return FIO_UTF16 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int;
            }
            return FIO_UTF16 as ::core::ffi::c_int;
        }
        return FIO_UTF8 as ::core::ffi::c_int;
    }
    if prop & ENC_LATIN1 as ::core::ffi::c_int != 0 {
        return FIO_LATIN1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn check_for_bom(
    mut p_in: *const ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
    mut lenp: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *const uint8_t = p_in as *const uint8_t;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 0xef as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xbb as ::core::ffi::c_int
        && size >= 3 as ::core::ffi::c_int
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xbf as ::core::ffi::c_int
        && (flags == FIO_ALL as ::core::ffi::c_int
            || flags == FIO_UTF8 as ::core::ffi::c_int
            || flags == 0 as ::core::ffi::c_int)
    {
        name = b"utf-8\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        len = 3 as ::core::ffi::c_int;
    } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 0xff as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xfe as ::core::ffi::c_int
    {
        if size >= 4 as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            && (flags == FIO_ALL as ::core::ffi::c_int
                || flags == FIO_UCS4 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int)
        {
            name = b"ucs-4le\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            len = 4 as ::core::ffi::c_int;
        } else if flags == FIO_UCS2 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int {
            name = b"ucs-2le\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if flags == FIO_ALL as ::core::ffi::c_int
            || flags == FIO_UTF16 as ::core::ffi::c_int | FIO_ENDIAN_L as ::core::ffi::c_int
        {
            name = b"utf-16le\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 0xfe as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xff as ::core::ffi::c_int
        && (flags == FIO_ALL as ::core::ffi::c_int
            || flags == FIO_UCS2 as ::core::ffi::c_int
            || flags == FIO_UTF16 as ::core::ffi::c_int)
    {
        if flags == FIO_UCS2 as ::core::ffi::c_int {
            name = b"ucs-2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            name = b"utf-16\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    } else if size >= 4 as ::core::ffi::c_int
        && *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xfe as ::core::ffi::c_int
        && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 0xff as ::core::ffi::c_int
        && (flags == FIO_ALL as ::core::ffi::c_int || flags == FIO_UCS4 as ::core::ffi::c_int)
    {
        name = b"ucs-4\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        len = 4 as ::core::ffi::c_int;
    }
    *lenp = len;
    return name;
}
#[no_mangle]
pub unsafe extern "C" fn shorten_buf_fname(
    mut buf: *mut buf_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut force: ::core::ffi::c_int,
) {
    if !(*buf).b_fname.is_null()
        && !bt_nofilename(buf)
        && path_with_url((*buf).b_fname) == 0
        && (force != 0
            || (*buf).b_sfname.is_null()
            || path_is_absolute((*buf).b_sfname) as ::core::ffi::c_int != 0)
    {
        if (*buf).b_sfname != (*buf).b_ffname {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        }
        let mut p: *mut ::core::ffi::c_char = path_shorten_fname((*buf).b_ffname, dirname);
        if !p.is_null() {
            (*buf).b_sfname = xstrdup(p);
            (*buf).b_fname = (*buf).b_sfname;
        }
        if p.is_null() {
            (*buf).b_fname = (*buf).b_ffname;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn shorten_fnames(mut force: ::core::ffi::c_int) {
    let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
    os_dirname(
        &raw mut dirname as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    );
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        shorten_buf_fname(buf, &raw mut dirname as *mut ::core::ffi::c_char, force);
        mf_fullname((*buf).b_ml.ml_mfp);
        buf = (*buf).b_next;
    }
    status_redraw_all();
    redraw_tabline = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn modname(
    mut fname: *const ::core::ffi::c_char,
    mut ext: *const ::core::ffi::c_char,
    mut prepend_dot: bool,
) -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fnamelen: size_t = 0;
    let mut extlen: size_t = strlen(ext);
    if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
        retval = xmalloc(
            (MAXPATHL as size_t)
                .wrapping_add(extlen)
                .wrapping_add(3 as size_t),
        ) as *mut ::core::ffi::c_char;
        if os_dirname(retval, MAXPATHL as size_t) == FAIL || strlen(retval) == 0 as size_t {
            xfree(retval as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        add_pathsep(retval);
        fnamelen = strlen(retval);
        prepend_dot = false_0 != 0;
    } else {
        fnamelen = strlen(fname);
        retval = xmalloc(fnamelen.wrapping_add(extlen).wrapping_add(3 as size_t))
            as *mut ::core::ffi::c_char;
        strcpy(retval, fname);
    }
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ptr = retval.offset(fnamelen as isize);
    while ptr > retval {
        if vim_ispathsep(*ptr as ::core::ffi::c_int) {
            ptr = ptr.offset(1);
            break;
        } else {
            ptr = ptr.offset(
                -((utf_head_off(retval, ptr.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
    }
    let mut ptrlen: size_t = fnamelen.wrapping_sub(ptr.offset_from(retval) as size_t);
    if ptrlen > BASENAMELEN as ::core::ffi::c_uint as size_t {
        ptrlen = BASENAMELEN as size_t;
        *ptr.offset(ptrlen as isize) = NUL as ::core::ffi::c_char;
    }
    let mut s: *mut ::core::ffi::c_char = ptr.offset(ptrlen as isize);
    strcpy(s, ext);
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if prepend_dot as ::core::ffi::c_int != 0 && {
        e = path_tail(retval);
        *e as ::core::ffi::c_int != '.' as ::core::ffi::c_int
    } {
        memmove(
            e.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            e as *const ::core::ffi::c_void,
            fnamelen
                .wrapping_add(extlen)
                .wrapping_sub(e.offset_from(retval) as size_t)
                .wrapping_add(1 as size_t),
        );
        *e = '.' as ::core::ffi::c_char;
    }
    if !fname.is_null() && strcmp(fname, retval) == 0 as ::core::ffi::c_int {
        loop {
            s = s.offset(-1);
            if s < ptr {
                break;
            }
            if *s as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                continue;
            }
            *s = '_' as ::core::ffi::c_char;
            break;
        }
        if s < ptr {
            *ptr = 'v' as ::core::ffi::c_char;
        }
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn vim_fgets(
    mut buf: *mut ::core::ffi::c_char,
    mut size: ::core::ffi::c_int,
    mut fp: *mut FILE,
) -> bool {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if size > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2504 as ::core::ffi::c_uint,
                b"_Bool vim_fgets(char *, int, FILE *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    *buf.offset((size - 2 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
    loop {
        *__errno_location() = 0 as ::core::ffi::c_int;
        retval = fgets(buf, size, fp);
        if !(retval.is_null() && *__errno_location() == EINTR && ferror(fp) != 0) {
            break;
        }
    }
    if *buf.offset((size - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int != NUL
        && *buf.offset((size - 2 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            != '\n' as ::core::ffi::c_int
    {
        let mut tbuf: [::core::ffi::c_char; 200] = [0; 200];
        *buf.offset((size - 1 as ::core::ffi::c_int) as isize) = NUL as ::core::ffi::c_char;
        loop {
            tbuf[::core::mem::size_of::<[::core::ffi::c_char; 200]>().wrapping_sub(2 as usize)
                as usize] = NUL as ::core::ffi::c_char;
            *__errno_location() = 0 as ::core::ffi::c_int;
            retval = fgets(
                &raw mut tbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 200]>() as ::core::ffi::c_int,
                fp,
            );
            if retval.is_null() && (feof(fp) != 0 || *__errno_location() != EINTR) {
                break;
            }
            if !(tbuf[::core::mem::size_of::<[::core::ffi::c_char; 200]>().wrapping_sub(2 as usize)
                as usize] as ::core::ffi::c_int
                != NUL
                && tbuf[::core::mem::size_of::<[::core::ffi::c_char; 200]>()
                    .wrapping_sub(2 as usize) as usize] as ::core::ffi::c_int
                    != '\n' as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
    return retval.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn get2c(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let n: ::core::ffi::c_int = getc(fd);
    if n == EOF {
        return -1 as ::core::ffi::c_int;
    }
    let c: ::core::ffi::c_int = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    return (n << 8 as ::core::ffi::c_int) + c;
}
#[no_mangle]
pub unsafe extern "C" fn get3c(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = getc(fd);
    if n == EOF {
        return -1 as ::core::ffi::c_int;
    }
    let mut c: ::core::ffi::c_int = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    n = (n << 8 as ::core::ffi::c_int) + c;
    c = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    return (n << 8 as ::core::ffi::c_int) + c;
}
#[no_mangle]
pub unsafe extern "C" fn get4c(mut fd: *mut FILE) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_uint = 0;
    let mut c: ::core::ffi::c_int = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    n = c as ::core::ffi::c_uint;
    c = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    n = (n << 8 as ::core::ffi::c_int).wrapping_add(c as ::core::ffi::c_uint);
    c = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    n = (n << 8 as ::core::ffi::c_int).wrapping_add(c as ::core::ffi::c_uint);
    c = getc(fd);
    if c == EOF {
        return -1 as ::core::ffi::c_int;
    }
    n = (n << 8 as ::core::ffi::c_int).wrapping_add(c as ::core::ffi::c_uint);
    return n as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn get8ctime(mut fd: *mut FILE) -> time_t {
    let mut n: time_t = 0 as time_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 8 as ::core::ffi::c_int {
        let c: ::core::ffi::c_int = getc(fd);
        if c == EOF {
            return -1 as time_t;
        }
        n = (n << 8 as ::core::ffi::c_int) + c as time_t;
        i += 1;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn read_string(
    mut fd: *mut FILE,
    mut cnt: size_t,
) -> *mut ::core::ffi::c_char {
    let mut str: *mut ::core::ffi::c_char = xmallocz(cnt) as *mut ::core::ffi::c_char;
    let mut i: size_t = 0 as size_t;
    while i < cnt {
        let mut c: ::core::ffi::c_int = getc(fd);
        if c == EOF {
            xfree(str as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        *str.offset(i as isize) = c as ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn put_bytes(
    mut fd: *mut FILE,
    mut number: uintmax_t,
    mut len: size_t,
) -> bool {
    '_c2rust_label: {
        if len > 0 as size_t {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2638 as ::core::ffi::c_uint,
                b"_Bool put_bytes(FILE *, uintmax_t, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut i: size_t = len.wrapping_sub(1 as size_t);
    while i < len {
        if putc(
            (number >> i.wrapping_mul(8 as size_t)) as ::core::ffi::c_int,
            fd,
        ) == EOF
        {
            return false_0 != 0;
        }
        i = i.wrapping_sub(1);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn put_time(mut fd: *mut FILE, mut time_: time_t) -> ::core::ffi::c_int {
    let mut buf: [uint8_t; 8] = [0; 8];
    time_to_bytes(time_, &raw mut buf as *mut uint8_t);
    return if fwrite(
        &raw mut buf as *mut uint8_t as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint8_t>(),
        ::core::mem::size_of::<[uint8_t; 8]>()
            .wrapping_div(::core::mem::size_of::<uint8_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint8_t; 8]>()
                    .wrapping_rem(::core::mem::size_of::<uint8_t>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        fd,
    ) == 1 as ::core::ffi::c_ulong
    {
        OK
    } else {
        FAIL
    };
}
unsafe extern "C" fn rename_with_tmp(
    from: *const ::core::ffi::c_char,
    to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if strlen(from) >= (MAXPATHL - 5 as ::core::ffi::c_int) as size_t {
        return -1 as ::core::ffi::c_int;
    }
    let mut tempname: [::core::ffi::c_char; 4097] = [0; 4097];
    strcpy(
        &raw mut tempname as *mut ::core::ffi::c_char,
        from as *mut ::core::ffi::c_char,
    );
    let mut n: ::core::ffi::c_int = 123 as ::core::ffi::c_int;
    while n < 99999 as ::core::ffi::c_int {
        let mut tail: *mut ::core::ffi::c_char =
            path_tail(&raw mut tempname as *mut ::core::ffi::c_char);
        snprintf(
            tail,
            ((MAXPATHL + 1 as ::core::ffi::c_int) as isize
                - tail.offset_from(&raw mut tempname as *mut ::core::ffi::c_char))
                as size_t,
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            n,
        );
        if !os_path_exists(&raw mut tempname as *mut ::core::ffi::c_char) {
            if os_rename(from, &raw mut tempname as *mut ::core::ffi::c_char) == OK {
                if os_rename(&raw mut tempname as *mut ::core::ffi::c_char, to) == OK {
                    return 0 as ::core::ffi::c_int;
                }
                os_rename(&raw mut tempname as *mut ::core::ffi::c_char, from);
                return -1 as ::core::ffi::c_int;
            }
            return -1 as ::core::ffi::c_int;
        }
        n += 1;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_rename(
    mut from: *const ::core::ffi::c_char,
    mut to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut use_tmp_file: bool = false_0 != 0;
    if path_fnamecmp(from, to) == 0 as ::core::ffi::c_int {
        if p_fic != 0 && strcmp(path_tail(from), path_tail(to)) != 0 as ::core::ffi::c_int {
            use_tmp_file = true_0 != 0;
        } else {
            return 0 as ::core::ffi::c_int;
        }
    }
    let mut from_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    if !os_fileinfo(from, &raw mut from_info) {
        return -1 as ::core::ffi::c_int;
    }
    let mut to_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    if os_fileinfo(to, &raw mut to_info) as ::core::ffi::c_int != 0
        && os_fileinfo_id_equal(&raw mut from_info, &raw mut to_info) as ::core::ffi::c_int != 0
    {
        use_tmp_file = true_0 != 0;
    }
    if use_tmp_file {
        return rename_with_tmp(from, to);
    }
    os_remove(to);
    if os_rename(from, to) == OK {
        return 0 as ::core::ffi::c_int;
    }
    let mut ret: ::core::ffi::c_int = vim_copyfile(from, to);
    if ret != OK {
        return -1 as ::core::ffi::c_int;
    }
    if os_fileinfo(from, &raw mut from_info) {
        os_remove(from);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn vim_copyfile(
    mut from: *const ::core::ffi::c_char,
    mut to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut from_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    if os_fileinfo_link(from, &raw mut from_info) as ::core::ffi::c_int != 0
        && from_info.stat.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t
    {
        let mut ret: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut linkbuf: [::core::ffi::c_char; 4097] = [0; 4097];
        let mut len: ssize_t = readlink(
            from,
            &raw mut linkbuf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        );
        if len > 0 as ssize_t {
            linkbuf[len as usize] = NUL as ::core::ffi::c_char;
            ret = symlink(&raw mut linkbuf as *mut ::core::ffi::c_char, to);
        }
        return if ret == 0 as ::core::ffi::c_int {
            OK
        } else {
            FAIL
        };
    }
    let mut acl: vim_acl_T = os_get_acl(from);
    if os_copy(from, to, UV_FS_COPYFILE_EXCL) != 0 as ::core::ffi::c_int {
        os_free_acl(acl);
        return FAIL;
    }
    os_set_acl(to, acl);
    os_free_acl(acl);
    if !errmsg.is_null() {
        semsg(errmsg, to);
        return FAIL;
    }
    return OK;
}
static already_warned: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
#[no_mangle]
pub unsafe extern "C" fn check_timestamps(mut focus: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if no_check_timestamps > 0 as ::core::ffi::c_int {
        return false_0;
    }
    if focus != 0 && did_check_timestamps as ::core::ffi::c_int != 0 {
        need_check_timestamps = true_0 != 0;
        return false_0;
    }
    let mut didit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !stuff_empty()
        || global_busy != 0
        || typebuf_typed() == 0
        || autocmd_busy as ::core::ffi::c_int != 0
        || (*curbuf).b_ro_locked > 0 as ::core::ffi::c_int
        || allbuf_lock > 0 as ::core::ffi::c_int
    {
        need_check_timestamps = true_0 != 0;
    } else {
        no_wait_return += 1;
        did_check_timestamps = true_0 != 0;
        already_warned.set(false_0 != 0);
        let mut buf: *mut buf_T = firstbuf;
        while !buf.is_null() {
            if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
                let mut bufref: bufref_T = bufref_T {
                    br_buf: ::core::ptr::null_mut::<buf_T>(),
                    br_fnum: 0,
                    br_buf_free_count: 0,
                };
                set_bufref(&raw mut bufref, buf);
                let n: ::core::ffi::c_int = buf_check_timestamp(buf);
                didit = if didit > n { didit } else { n };
                if n > 0 as ::core::ffi::c_int && !bufref_valid(&raw mut bufref) {
                    buf = firstbuf;
                }
            }
            buf = (*buf).b_next;
        }
        no_wait_return -= 1;
        need_check_timestamps = false_0 != 0;
        if need_wait_return as ::core::ffi::c_int != 0 && didit == 2 as ::core::ffi::c_int {
            msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
            ui_flush();
        }
    }
    return didit;
}
unsafe extern "C" fn move_lines(
    mut frombuf: *mut buf_T,
    mut tobuf: *mut buf_T,
) -> ::core::ffi::c_int {
    let mut tbuf: *mut buf_T = curbuf;
    let mut retval: ::core::ffi::c_int = OK;
    curbuf = tobuf;
    let mut lnum: linenr_T = 1 as linenr_T;
    while lnum <= (*frombuf).b_ml.ml_line_count {
        let mut p: *mut ::core::ffi::c_char = xmemdupz(
            ml_get_buf(frombuf, lnum) as *const ::core::ffi::c_void,
            ml_get_buf_len(frombuf, lnum) as size_t,
        ) as *mut ::core::ffi::c_char;
        if ml_append(lnum - 1 as linenr_T, p, 0 as colnr_T, false_0 != 0) == FAIL {
            xfree(p as *mut ::core::ffi::c_void);
            retval = FAIL;
            break;
        } else {
            xfree(p as *mut ::core::ffi::c_void);
            lnum += 1;
        }
    }
    if retval != FAIL {
        curbuf = frombuf;
        let mut lnum_0: linenr_T = (*curbuf).b_ml.ml_line_count;
        while lnum_0 > 0 as linenr_T {
            if ml_delete(lnum_0) == FAIL {
                retval = FAIL;
                break;
            } else {
                lnum_0 -= 1;
            }
        }
    }
    curbuf = tbuf;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn buf_check_timestamp(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mesg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mesg2: *mut ::core::ffi::c_char =
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    let mut helpmesg: bool = false_0 != 0;
    let mut reload: C2Rust_Unnamed_31 = RELOAD_NONE;
    let mut can_reload: bool = false_0 != 0;
    let mut orig_size: uint64_t = (*buf).b_orig_size;
    let mut orig_mode: ::core::ffi::c_int = (*buf).b_orig_mode;
    static busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, buf);
    if !(*buf).terminal.is_null()
        || (*buf).b_ffname.is_null()
        || (*buf).b_ml.ml_mfp.is_null()
        || !bt_normal(buf)
        || (*buf).b_saving as ::core::ffi::c_int != 0
        || busy.get() as ::core::ffi::c_int != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut file_info_ok: bool = false;
    if (*buf).b_flags & BF_NOTEDITED == 0 && (*buf).b_mtime != 0 as int64_t && {
        file_info_ok = os_fileinfo((*buf).b_ffname, &raw mut file_info);
        !file_info_ok
            || time_differs(&raw mut file_info, (*buf).b_mtime, (*buf).b_mtime_ns)
                as ::core::ffi::c_int
                != 0
            || file_info.stat.st_mode as ::core::ffi::c_int != (*buf).b_orig_mode
    } {
        let prev_b_mtime: int64_t = (*buf).b_mtime;
        retval = 1 as ::core::ffi::c_int;
        if !file_info_ok {
            (*buf).b_mtime = -1 as int64_t;
            (*buf).b_orig_size = 0 as uint64_t;
            (*buf).b_orig_mode = 0 as ::core::ffi::c_int;
        } else {
            buf_store_file_info(buf, &raw mut file_info);
        }
        if !os_isdir((*buf).b_fname) {
            if (if (*buf).b_p_ar >= 0 as ::core::ffi::c_int {
                (*buf).b_p_ar
            } else {
                p_ar
            }) != 0
                && !bufIsChanged(buf)
                && file_info_ok as ::core::ffi::c_int != 0
            {
                reload = RELOAD_NORMAL;
            } else {
                let mut reason: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut reasonlen: size_t = 0;
                if !file_info_ok {
                    reason = b"deleted\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                        .wrapping_sub(1 as usize) as size_t;
                } else if bufIsChanged(buf) {
                    reason = b"conflict\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as usize) as size_t;
                } else if orig_size != (*buf).b_orig_size
                    || buf_contents_changed(buf) as ::core::ffi::c_int != 0
                {
                    reason = b"changed\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                        .wrapping_sub(1 as usize) as size_t;
                } else if orig_mode != (*buf).b_orig_mode {
                    reason = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as usize) as size_t;
                } else {
                    reason = b"time\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                    reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as usize) as size_t;
                }
                busy.set(true_0 != 0);
                set_vim_var_string(
                    VV_FCS_REASON,
                    reason,
                    reasonlen as ::core::ffi::c_int as ptrdiff_t,
                );
                set_vim_var_string(
                    VV_FCS_CHOICE,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as ptrdiff_t,
                );
                allbuf_lock += 1;
                let mut n: bool = apply_autocmds(
                    EVENT_FILECHANGEDSHELL,
                    (*buf).b_fname,
                    (*buf).b_fname,
                    false_0 != 0,
                    buf,
                );
                allbuf_lock -= 1;
                busy.set(false_0 != 0);
                if n {
                    if !bufref_valid(&raw mut bufref) {
                        emsg(gettext(
                            b"E246: FileChangedShell autocommand deleted buffer\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                    }
                    let mut s: *mut ::core::ffi::c_char = get_vim_var_str(VV_FCS_CHOICE);
                    if strcmp(s, b"reload\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                        && *reason as ::core::ffi::c_int != 'd' as ::core::ffi::c_int
                    {
                        reload = RELOAD_NORMAL;
                    } else if strcmp(s, b"edit\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        reload = RELOAD_DETECT;
                    } else if strcmp(s, b"ask\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        n = false_0 != 0;
                    } else {
                        return 2 as ::core::ffi::c_int;
                    }
                }
                if !n {
                    if *reason as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                        if prev_b_mtime != -1 as int64_t {
                            mesg = gettext(b"E211: File \"%s\" no longer available\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        }
                    } else {
                        helpmesg = true_0 != 0;
                        can_reload = true_0 != 0;
                        if *reason.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'n' as ::core::ffi::c_int
                        {
                            mesg = gettext(
                                b"W12: Warning: File \"%s\" has changed and the buffer was changed in Vim as well\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            mesg2 = gettext(b"See \":help W12\" for more info.\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        } else if *reason.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'h' as ::core::ffi::c_int
                        {
                            mesg = gettext(
                                b"W11: Warning: File \"%s\" has changed since editing started\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            mesg2 = gettext(b"See \":help W11\" for more info.\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        } else if *reason as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
                            mesg = gettext(
                                b"W16: Warning: Mode of file \"%s\" has changed since editing started\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            mesg2 = gettext(b"See \":help W16\" for more info.\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        } else {
                            (*buf).b_mtime_read = (*buf).b_mtime;
                            (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
                        }
                    }
                }
            }
        }
    } else if (*buf).b_flags & BF_NEW != 0
        && (*buf).b_flags & BF_NEW_W == 0
        && os_path_exists((*buf).b_ffname) as ::core::ffi::c_int != 0
    {
        retval = 1 as ::core::ffi::c_int;
        mesg = gettext(
            b"W13: Warning: File \"%s\" has been created after editing started\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        (*buf).b_flags |= BF_NEW_W;
        can_reload = true_0 != 0;
    }
    if !mesg.is_null() {
        let mut path: *mut ::core::ffi::c_char = home_replace_save(buf, (*buf).b_fname);
        if !helpmesg {
            mesg2 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        let tbufsize: size_t = strlen(path)
            .wrapping_add(strlen(mesg))
            .wrapping_add(strlen(mesg2))
            .wrapping_add(3 as size_t);
        let tbuf: *mut ::core::ffi::c_char = xmalloc(tbufsize) as *mut ::core::ffi::c_char;
        let mut tbuflen: ::core::ffi::c_int = snprintf(tbuf, tbufsize, mesg, path);
        set_vim_var_string(VV_WARNINGMSG, tbuf, tbuflen as ptrdiff_t);
        if can_reload {
            if *mesg2 as ::core::ffi::c_int != NUL {
                snprintf(
                    tbuf.offset(tbuflen as isize),
                    tbufsize.wrapping_sub(tbuflen as size_t),
                    b"\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                    mesg2,
                );
            }
            match do_dialog(
                VIM_WARNING as ::core::ffi::c_int,
                gettext(b"Warning\0".as_ptr() as *const ::core::ffi::c_char),
                tbuf,
                gettext(b"&OK\n&Load File\nLoad File &and Options\0".as_ptr()
                    as *const ::core::ffi::c_char),
                1 as ::core::ffi::c_int,
                ::core::ptr::null::<::core::ffi::c_char>(),
                true_0,
            ) {
                2 => {
                    reload = RELOAD_NORMAL;
                }
                3 => {
                    reload = RELOAD_DETECT;
                }
                _ => {}
            }
        } else if State > MODE_NORMAL_BUSY as ::core::ffi::c_int
            || State & MODE_CMDLINE as ::core::ffi::c_int != 0
            || already_warned.get() as ::core::ffi::c_int != 0
        {
            if *mesg2 as ::core::ffi::c_int != NUL {
                snprintf(
                    tbuf.offset(tbuflen as isize),
                    tbufsize.wrapping_sub(tbuflen as size_t),
                    b"; %s\0".as_ptr() as *const ::core::ffi::c_char,
                    mesg2,
                );
            }
            emsg(tbuf);
            retval = 2 as ::core::ffi::c_int;
        } else {
            if !autocmd_busy {
                msg_start();
                msg_puts_hl(tbuf, HLF_E as ::core::ffi::c_int, true_0 != 0);
                if *mesg2 as ::core::ffi::c_int != NUL {
                    msg_puts_hl(mesg2, HLF_W as ::core::ffi::c_int, true_0 != 0);
                }
                msg_clr_eos();
                msg_end();
                if emsg_silent == 0 as ::core::ffi::c_int
                    && !in_assert_fails
                    && !ui_has(kUIMessages)
                {
                    msg_delay(1004 as uint64_t, true_0 != 0);
                    redraw_cmdline = false_0 != 0;
                }
            }
            already_warned.set(true_0 != 0);
        }
        xfree(tbuf as *mut ::core::ffi::c_void);
        xfree(path as *mut ::core::ffi::c_void);
    }
    if reload as ::core::ffi::c_uint != RELOAD_NONE as ::core::ffi::c_int as ::core::ffi::c_uint {
        buf_reload(
            buf,
            orig_mode,
            reload as ::core::ffi::c_uint
                == RELOAD_DETECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        );
        if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
            && (*buf).b_p_udf != 0
            && !(*buf).b_ffname.is_null()
        {
            let mut hash: [uint8_t; 32] = [0; 32];
            u_compute_hash(buf, &raw mut hash as *mut uint8_t);
            u_write_undo(
                ::core::ptr::null::<::core::ffi::c_char>(),
                false_0 != 0,
                buf,
                &raw mut hash as *mut uint8_t,
            );
        }
    }
    if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0 && retval != 0 as ::core::ffi::c_int
    {
        apply_autocmds(
            EVENT_FILECHANGEDSHELLPOST,
            (*buf).b_fname,
            (*buf).b_fname,
            false_0 != 0,
            buf,
        );
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn buf_reload(
    mut buf: *mut buf_T,
    mut orig_mode: ::core::ffi::c_int,
    mut reload_options: bool,
) {
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
    let mut old_ro: ::core::ffi::c_int = (*buf).b_p_ro;
    let mut savebuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut saved: ::core::ffi::c_int = OK;
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
    let mut flags: ::core::ffi::c_int = READ_NEW as ::core::ffi::c_int;
    aucmd_prepbuf(&raw mut aco, buf);
    if reload_options {
        memset(
            &raw mut ea as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<exarg_T>(),
        );
    } else {
        prep_exarg(&raw mut ea, buf);
    }
    let mut old_cursor: pos_T = (*curwin).w_cursor;
    let mut old_topline: linenr_T = (*curwin).w_topline;
    if p_ur < 0 as OptInt || (*curbuf).b_ml.ml_line_count as OptInt <= p_ur {
        u_sync(false_0 != 0);
        saved = u_savecommon(
            curbuf,
            0 as linenr_T,
            (*curbuf).b_ml.ml_line_count + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        flags |= READ_KEEP_UNDO as ::core::ffi::c_int;
    }
    if buf_is_empty(curbuf) as ::core::ffi::c_int != 0 || saved == FAIL {
        savebuf = ::core::ptr::null_mut::<buf_T>();
    } else {
        savebuf = buflist_new(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            BLN_DUMMY as ::core::ffi::c_int,
        );
        set_bufref(&raw mut bufref, savebuf);
        if !savebuf.is_null() && buf == curbuf {
            curbuf = savebuf;
            (*curwin).w_buffer = savebuf;
            saved = ml_open(curbuf);
            curbuf = buf;
            (*curwin).w_buffer = buf;
        }
        if savebuf.is_null() || saved == FAIL || buf != curbuf || move_lines(buf, savebuf) == FAIL {
            semsg(
                gettext(b"E462: Could not prepare for reloading \"%s\"\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*buf).b_fname,
            );
            saved = FAIL;
        }
    }
    if saved == OK {
        (*curbuf).b_flags |= BF_CHECK_RO;
        (*curbuf).b_keep_filetype = true_0 != 0;
        if readfile(
            (*buf).b_ffname,
            (*buf).b_fname,
            0 as linenr_T,
            0 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            &raw mut ea,
            flags,
            shortmess(SHM_FILEINFO as ::core::ffi::c_int),
        ) != OK
        {
            if !aborting() {
                semsg(
                    gettext(
                        b"E321: Could not reload \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    (*buf).b_fname,
                );
            }
            if !savebuf.is_null()
                && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && buf == curbuf
            {
                while !buf_is_empty(curbuf) {
                    if ml_delete((*buf).b_ml.ml_line_count) == FAIL {
                        break;
                    }
                }
                move_lines(savebuf, buf);
            }
        } else if buf == curbuf {
            unchanged(buf, true_0 != 0, true_0 != 0);
            if flags & READ_KEEP_UNDO as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                u_clearallandblockfree(buf);
            } else {
                u_unchanged(curbuf);
            }
            buf_updates_unload(curbuf, true_0 != 0);
            (*curbuf).b_mod_set = true_0 != 0;
        }
    }
    xfree(ea.cmd as *mut ::core::ffi::c_void);
    if !savebuf.is_null() && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0 {
        wipe_buffer(savebuf, false_0 != 0);
    }
    diff_invalidate(curbuf);
    (*curwin).w_topline = if old_topline < (*curbuf).b_ml.ml_line_count {
        old_topline
    } else {
        (*curbuf).b_ml.ml_line_count
    };
    (*curwin).w_cursor = old_cursor;
    check_cursor(curwin);
    update_topline(curwin);
    (*curbuf).b_keep_filetype = false_0 != 0;
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == (*curwin).w_buffer && !foldmethodIsManual(wp) {
                foldUpdateAll(wp);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    if orig_mode == (*curbuf).b_orig_mode {
        (*curbuf).b_p_ro |= old_ro;
    }
    do_modelines(0 as ::core::ffi::c_int);
    aucmd_restbuf(&raw mut aco);
}
#[no_mangle]
pub unsafe extern "C" fn buf_store_file_info(mut buf: *mut buf_T, mut file_info: *mut FileInfo) {
    (*buf).b_mtime = (*file_info).stat.st_mtim.tv_sec as int64_t;
    (*buf).b_mtime_ns = (*file_info).stat.st_mtim.tv_nsec as int64_t;
    (*buf).b_orig_size = os_fileinfo_size(file_info);
    (*buf).b_orig_mode = (*file_info).stat.st_mode as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn write_lnum_adjust(mut offset: linenr_T) {
    if (*curbuf).b_no_eol_lnum != 0 as linenr_T {
        (*curbuf).b_no_eol_lnum += offset;
    }
}
static vim_tempdir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static vim_tempdir_dp: GlobalCell<*mut DIR> = GlobalCell::new(::core::ptr::null_mut::<DIR>());
unsafe extern "C" fn vim_mktempdir() {
    static temp_dirs: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new(TEMP_DIR_NAMES);
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut path: [::core::ffi::c_char; 256] = [0; 256];
    let mut user: [::core::ffi::c_char; 40] = [
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
    ];
    os_get_username(
        &raw mut user as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
    );
    memchrsub(
        &raw mut user as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        '/' as ::core::ffi::c_char,
        '_' as ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
    );
    memchrsub(
        &raw mut user as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        '\\' as ::core::ffi::c_char,
        '_' as ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
    );
    let mut umask_save: mode_t = umask(0o77 as __mode_t);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
                .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut tmplen: size_t = expand_env(
            (*temp_dirs.ptr())[i as usize] as *mut ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
            TEMP_FILE_PATH_MAXLEN - 64 as ::core::ffi::c_int,
        );
        if !os_isdir(&raw mut tmp as *mut ::core::ffi::c_char) {
            if strequal(
                b"$TMPDIR\0".as_ptr() as *const ::core::ffi::c_char,
                (*temp_dirs.ptr())[i as usize],
            ) {
                if !os_env_exists(
                    b"TMPDIR\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0 != 0,
                ) {
                    logmsg(
                        LOGLVL_DBG,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3323 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"$TMPDIR is unset\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    logmsg(
                        LOGLVL_WRN,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3325 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"$TMPDIR tempdir not a directory (or does not exist): \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        &raw mut tmp as *mut ::core::ffi::c_char,
                    );
                }
            }
        } else {
            if after_pathsep(
                &raw mut tmp as *mut ::core::ffi::c_char,
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
            ) == 0
            {
                tmplen = tmplen.wrapping_add(vim_snprintf(
                    (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                    PATHSEPSTR.as_ptr(),
                ) as size_t);
                '_c2rust_label: {
                    if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                    } else {
                        __assert_fail(
                            b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3334 as ::core::ffi::c_uint,
                            b"void vim_mktempdir(void)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
            }
            tmplen = tmplen.wrapping_add(vim_snprintf(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                b"nvim.%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut user as *mut ::core::ffi::c_char,
            ) as size_t);
            '_c2rust_label_0: {
                if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                } else {
                    __assert_fail(
                        b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3338 as ::core::ffi::c_uint,
                        b"void vim_mktempdir(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            os_mkdir(&raw mut tmp as *mut ::core::ffi::c_char, 0o700 as int32_t);
            let mut owned: bool = os_file_owned(&raw mut tmp as *mut ::core::ffi::c_char);
            let mut isdir: bool = os_isdir(&raw mut tmp as *mut ::core::ffi::c_char);
            let mut perm: ::core::ffi::c_int =
                os_getperm(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            let mut valid: bool = isdir as ::core::ffi::c_int != 0
                && owned as ::core::ffi::c_int != 0
                && 0o700 as ::core::ffi::c_int == perm & 0o777 as ::core::ffi::c_int;
            if valid {
                if after_pathsep(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                ) == 0
                {
                    tmplen = tmplen.wrapping_add(vim_snprintf(
                        (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                        ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                        PATHSEPSTR.as_ptr(),
                    ) as size_t);
                    '_c2rust_label_1: {
                        if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                        } else {
                            __assert_fail(
                                b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                3351 as ::core::ffi::c_uint,
                                b"void vim_mktempdir(void)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            } else {
                if !owned {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3355 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"tempdir root not owned by current user (%s): %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        &raw mut user as *mut ::core::ffi::c_char,
                        &raw mut tmp as *mut ::core::ffi::c_char,
                    );
                } else if !isdir {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3357 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"tempdir root not a directory: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        &raw mut tmp as *mut ::core::ffi::c_char,
                    );
                }
                if 0o700 as ::core::ffi::c_int != perm & 0o777 as ::core::ffi::c_int {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3361 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"tempdir root has invalid permissions (%o): %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        perm,
                        &raw mut tmp as *mut ::core::ffi::c_char,
                    );
                }
                tmplen = tmplen.wrapping_sub(strlen(&raw mut user as *mut ::core::ffi::c_char));
                tmp[tmplen as usize] = NUL as ::core::ffi::c_char;
            }
            tmplen = tmplen.wrapping_add(vim_snprintf(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                b"XXXXXX\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t);
            '_c2rust_label_2: {
                if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                } else {
                    __assert_fail(
                        b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3373 as ::core::ffi::c_uint,
                        b"void vim_mktempdir(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut r: ::core::ffi::c_int = os_mkdtemp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                &raw mut path as *mut ::core::ffi::c_char,
            );
            if r != 0 as ::core::ffi::c_int {
                logmsg(
                    LOGLVL_WRN,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                    3377 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"tempdir create failed: %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    uv_strerror(r),
                    &raw mut tmp as *mut ::core::ffi::c_char,
                );
            } else {
                if vim_settempdir(&raw mut path as *mut ::core::ffi::c_char) {
                    break;
                }
                os_rmdir(&raw mut path as *mut ::core::ffi::c_char);
            }
        }
        i = i.wrapping_add(1);
    }
    umask(umask_save as __mode_t);
}
#[no_mangle]
pub unsafe extern "C" fn readdir_core(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut context: *mut ::core::ffi::c_void,
    mut checkitem: CheckItem,
) -> ::core::ffi::c_int {
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    let mut dir: Directory = Directory {
        request: uv_fs_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            fs_type: UV_FS_CUSTOM,
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            cb: None,
            result: 0,
            ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            path: ::core::ptr::null::<::core::ffi::c_char>(),
            statbuf: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
            new_path: ::core::ptr::null::<::core::ffi::c_char>(),
            file: 0,
            flags: 0,
            mode: 0,
            nbufs: 0,
            bufs: ::core::ptr::null_mut::<uv_buf_t>(),
            off: 0,
            uid: 0,
            gid: 0,
            atime: 0.,
            mtime: 0.,
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            bufsml: [uv_buf_t {
                base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                len: 0,
            }; 4],
        },
        ent: uv_dirent_t {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            type_0: UV_DIRENT_UNKNOWN,
        },
    };
    if !os_scandir(&raw mut dir, path) {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            path,
        );
        return FAIL;
    }
    loop {
        let mut p: *const ::core::ffi::c_char = os_scandir_next(&raw mut dir);
        if p.is_null() {
            break;
        }
        let mut ignore: bool = *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL);
        if !ignore && checkitem.is_some() {
            let mut r: varnumber_T = checkitem.expect("non-null function pointer")(context, p);
            if r < 0 as varnumber_T {
                break;
            }
            if r == 0 as varnumber_T {
                ignore = true_0 != 0;
            }
        }
        if !ignore {
            ga_grow(gap, 1 as ::core::ffi::c_int);
            let c2rust_fresh9 = (*gap).ga_len;
            (*gap).ga_len = (*gap).ga_len + 1;
            let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                .offset(c2rust_fresh9 as isize);
            *c2rust_lvalue_ptr = xstrdup(p);
        }
    }
    os_closedir(&raw mut dir);
    if (*gap).ga_len > 0 as ::core::ffi::c_int {
        sort_strings(
            (*gap).ga_data as *mut *mut ::core::ffi::c_char,
            (*gap).ga_len,
        );
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn delete_recursive(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if os_isrealdir(name) {
        let mut exp: *mut ::core::ffi::c_char = xstrdup(name);
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        if readdir_core(&raw mut ga, exp, NULL, None) == OK {
            let mut len: ::core::ffi::c_int = snprintf(
                &raw mut NameBuff as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                b"%s/\0".as_ptr() as *const ::core::ffi::c_char,
                exp,
            );
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ga.ga_len {
                snprintf(
                    (&raw mut NameBuff as *mut ::core::ffi::c_char).offset(len as isize),
                    (MAXPATHL as size_t).wrapping_sub(len as size_t),
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                );
                if delete_recursive(&raw mut NameBuff as *mut ::core::ffi::c_char)
                    != 0 as ::core::ffi::c_int
                {
                    result = -1 as ::core::ffi::c_int;
                }
                i += 1;
            }
            ga_clear_strings(&raw mut ga);
            if os_rmdir(exp) != 0 as ::core::ffi::c_int {
                result = -1 as ::core::ffi::c_int;
            }
        } else {
            result = -1 as ::core::ffi::c_int;
        }
        xfree(exp as *mut ::core::ffi::c_void);
    } else {
        result = if os_remove(name) == 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return result;
}
unsafe extern "C" fn vim_opentempdir() {
    if !(*vim_tempdir_dp.ptr()).is_null() {
        return;
    }
    let mut dp: *mut DIR = opendir(vim_tempdir.get());
    if dp.is_null() {
        return;
    }
    vim_tempdir_dp.set(dp);
    flock(dirfd(vim_tempdir_dp.get()), LOCK_SH);
}
unsafe extern "C" fn vim_closetempdir() {
    if (*vim_tempdir_dp.ptr()).is_null() {
        return;
    }
    closedir(vim_tempdir_dp.get());
    vim_tempdir_dp.set(::core::ptr::null_mut::<DIR>());
}
#[no_mangle]
pub unsafe extern "C" fn vim_deltempdir() {
    if (*vim_tempdir.ptr()).is_null() {
        return;
    }
    vim_closetempdir();
    *path_tail(vim_tempdir.get()).offset(-1 as ::core::ffi::c_int as isize) =
        NUL as ::core::ffi::c_char;
    delete_recursive(vim_tempdir.get());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        vim_tempdir.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
}
#[no_mangle]
pub unsafe extern "C" fn vim_gettempdir() -> *mut ::core::ffi::c_char {
    static notfound: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if (*vim_tempdir.ptr()).is_null() || !os_isdir(vim_tempdir.get()) {
        if !(*vim_tempdir.ptr()).is_null() {
            (*notfound.ptr()) += 1;
            if notfound.get() == 1 as ::core::ffi::c_int {
                logmsg(
                    LOGLVL_ERR,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"vim_gettempdir\0".as_ptr() as *const ::core::ffi::c_char,
                    3534 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"tempdir disappeared (antivirus or broken cleanup job?): %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    vim_tempdir.get(),
                );
            }
            if notfound.get() > 1 as ::core::ffi::c_int {
                msg_schedule_semsg(
                    b"E5431: tempdir disappeared (%d times)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    notfound.get(),
                );
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                vim_tempdir.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            *ptr_;
        }
        vim_mktempdir();
    }
    return vim_tempdir.get();
}
unsafe extern "C" fn vim_settempdir(mut tempdir: *mut ::core::ffi::c_char) -> bool {
    let mut buf: *mut ::core::ffi::c_char =
        verbose_try_malloc((MAXPATHL + 2 as ::core::ffi::c_int) as size_t)
            as *mut ::core::ffi::c_char;
    if buf.is_null() {
        return false_0 != 0;
    }
    vim_FullName(tempdir, buf, MAXPATHL as size_t, false_0 != 0);
    let mut buflen: size_t = strlen(buf);
    if after_pathsep(buf, buf.offset(buflen as isize)) == 0 {
        strcpy(buf.offset(buflen as isize), PATHSEPSTR.as_ptr());
        buflen = (buflen as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    vim_tempdir
        .set(xmemdupz(buf as *const ::core::ffi::c_void, buflen) as *mut ::core::ffi::c_char);
    vim_opentempdir();
    xfree(buf as *mut ::core::ffi::c_void);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_tempname() -> *mut ::core::ffi::c_char {
    static temp_count: GlobalCell<uint64_t> = GlobalCell::new(0);
    let mut tempdir: *mut ::core::ffi::c_char = vim_gettempdir();
    if tempdir.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut templ: [::core::ffi::c_char; 256] = [0; 256];
    let c2rust_fresh8 = temp_count.get();
    temp_count.set((*temp_count.ptr()).wrapping_add(1));
    let mut itmplen: ::core::ffi::c_int = snprintf(
        &raw mut templ as *mut ::core::ffi::c_char,
        TEMP_FILE_PATH_MAXLEN as size_t,
        b"%s%lu\0".as_ptr() as *const ::core::ffi::c_char,
        tempdir,
        c2rust_fresh8,
    );
    return xmemdupz(
        &raw mut templ as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        itmplen as size_t,
    ) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn match_file_pat(
    mut pattern: *mut ::core::ffi::c_char,
    mut prog: *mut *mut regprog_T,
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut tail: *mut ::core::ffi::c_char,
    mut allow_dirs: ::core::ffi::c_int,
) -> bool {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut result: bool = false_0 != 0;
    regmatch.rm_ic = p_fic != 0;
    regmatch.regprog = if !prog.is_null() {
        *prog
    } else {
        vim_regcomp(pattern, RE_MAGIC)
    };
    if !regmatch.regprog.is_null()
        && (allow_dirs != 0
            && (vim_regexec(&raw mut regmatch, fname, 0 as colnr_T) as ::core::ffi::c_int != 0
                || !sfname.is_null()
                    && vim_regexec(&raw mut regmatch, sfname, 0 as colnr_T) as ::core::ffi::c_int
                        != 0)
            || allow_dirs == 0
                && vim_regexec(&raw mut regmatch, tail, 0 as colnr_T) as ::core::ffi::c_int != 0)
    {
        result = true_0 != 0;
    }
    if !prog.is_null() {
        *prog = regmatch.regprog;
    } else {
        vim_regfree(regmatch.regprog);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn match_file_list(
    mut list: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut ffname: *mut ::core::ffi::c_char,
) -> bool {
    let mut tail: *mut ::core::ffi::c_char = path_tail(sfname);
    let mut p: *mut ::core::ffi::c_char = list;
    while *p != 0 {
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut allow_dirs: ::core::ffi::c_char = 0;
        let mut regpat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            &raw mut allow_dirs,
            false_0,
        );
        if regpat.is_null() {
            break;
        }
        let mut match_0: bool = match_file_pat(
            regpat,
            ::core::ptr::null_mut::<*mut regprog_T>(),
            ffname,
            sfname,
            tail,
            allow_dirs as ::core::ffi::c_int,
        );
        xfree(regpat as *mut ::core::ffi::c_void);
        if match_0 {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn file_pat_to_reg_pat(
    mut pat: *const ::core::ffi::c_char,
    mut pat_end: *const ::core::ffi::c_char,
    mut allow_dirs: *mut ::core::ffi::c_char,
    mut no_bslash: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if !allow_dirs.is_null() {
        *allow_dirs = false_0 as ::core::ffi::c_char;
    }
    if pat_end.is_null() {
        pat_end = pat.offset(strlen(pat) as isize);
    }
    if pat_end == pat {
        return xstrdup(b"^$\0".as_ptr() as *const ::core::ffi::c_char);
    }
    let mut size: size_t = 2 as size_t;
    let mut p: *const ::core::ffi::c_char = pat;
    while p < pat_end {
        match *p as ::core::ffi::c_int {
            42 | 46 | 44 | 123 | 125 | 126 => {
                size = size.wrapping_add(2 as size_t);
            }
            _ => {
                size = size.wrapping_add(1);
            }
        }
        p = p.offset(1);
    }
    let mut reg_pat: *mut ::core::ffi::c_char =
        xmalloc(size.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut i: size_t = 0 as size_t;
    if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '*' as ::core::ffi::c_int
    {
        while *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
            && pat < pat_end.offset(-(1 as ::core::ffi::c_int as isize))
        {
            pat = pat.offset(1);
        }
    } else {
        let c2rust_fresh10 = i;
        i = i.wrapping_add(1);
        *reg_pat.offset(c2rust_fresh10 as isize) = '^' as ::core::ffi::c_char;
    }
    let mut endp: *const ::core::ffi::c_char = pat_end.offset(-(1 as ::core::ffi::c_int as isize));
    let mut add_dollar: bool = true_0 != 0;
    if endp >= pat && *endp as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        while endp.offset_from(pat) > 0 as isize
            && *endp as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        {
            endp = endp.offset(-1);
        }
        add_dollar = false_0 != 0;
    }
    let mut nested: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p_0: *const ::core::ffi::c_char = pat;
    while *p_0 as ::core::ffi::c_int != 0 && nested >= 0 as ::core::ffi::c_int && p_0 <= endp {
        match *p_0 as ::core::ffi::c_int {
            42 => {
                let c2rust_fresh11 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh11 as isize) = '.' as ::core::ffi::c_char;
                let c2rust_fresh12 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh12 as isize) = '*' as ::core::ffi::c_char;
                while *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                {
                    p_0 = p_0.offset(1);
                }
            }
            46 | 126 => {
                let c2rust_fresh13 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh13 as isize) = '\\' as ::core::ffi::c_char;
                let c2rust_fresh14 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh14 as isize) = *p_0;
            }
            63 => {
                let c2rust_fresh15 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh15 as isize) = '.' as ::core::ffi::c_char;
            }
            92 => {
                if *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    p_0 = p_0.offset(1);
                    if *p_0 as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                        && (BACKSLASH_IN_FILENAME_BOOL == 0 || no_bslash != 0)
                    {
                        let c2rust_fresh16 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh16 as isize) = '?' as ::core::ffi::c_char;
                    } else if *p_0 as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                        || *p_0 as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                        || *p_0 as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                        || ascii_isspace(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || *p_0 as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        || *p_0 as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                    {
                        let c2rust_fresh17 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh17 as isize) = *p_0;
                    } else if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        && *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p_0.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '{' as ::core::ffi::c_int
                    {
                        let c2rust_fresh18 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh18 as isize) = '\\' as ::core::ffi::c_char;
                        let c2rust_fresh19 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh19 as isize) = '{' as ::core::ffi::c_char;
                        p_0 = p_0.offset(2 as ::core::ffi::c_int as isize);
                    } else {
                        if !allow_dirs.is_null()
                            && vim_ispathsep(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            && (BACKSLASH_IN_FILENAME_BOOL == 0
                                || (no_bslash == 0
                                    || *p_0 as ::core::ffi::c_int != '\\' as ::core::ffi::c_int))
                        {
                            *allow_dirs = true_0 as ::core::ffi::c_char;
                        }
                        let c2rust_fresh20 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh20 as isize) = '\\' as ::core::ffi::c_char;
                        let c2rust_fresh21 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh21 as isize) = *p_0;
                    }
                }
            }
            123 => {
                let c2rust_fresh22 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh22 as isize) = '\\' as ::core::ffi::c_char;
                let c2rust_fresh23 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh23 as isize) = '(' as ::core::ffi::c_char;
                nested += 1;
            }
            125 => {
                let c2rust_fresh24 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh24 as isize) = '\\' as ::core::ffi::c_char;
                let c2rust_fresh25 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh25 as isize) = ')' as ::core::ffi::c_char;
                nested -= 1;
            }
            44 => {
                if nested != 0 {
                    let c2rust_fresh26 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh26 as isize) = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh27 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh27 as isize) = '|' as ::core::ffi::c_char;
                } else {
                    let c2rust_fresh28 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh28 as isize) = ',' as ::core::ffi::c_char;
                }
            }
            _ => {
                if !allow_dirs.is_null()
                    && vim_ispathsep(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    *allow_dirs = true_0 as ::core::ffi::c_char;
                }
                let c2rust_fresh29 = i;
                i = i.wrapping_add(1);
                *reg_pat.offset(c2rust_fresh29 as isize) = *p_0;
            }
        }
        p_0 = p_0.offset(1);
    }
    if add_dollar {
        let c2rust_fresh30 = i;
        i = i.wrapping_add(1);
        *reg_pat.offset(c2rust_fresh30 as isize) = '$' as ::core::ffi::c_char;
    }
    *reg_pat.offset(i as isize) = NUL as ::core::ffi::c_char;
    if nested != 0 as ::core::ffi::c_int {
        if nested < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E219: Missing {.\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(
                b"E220: Missing }.\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut reg_pat as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
    }
    return reg_pat;
}
#[no_mangle]
pub unsafe extern "C" fn read_eintr(
    mut fd: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_void,
    mut bufsize: size_t,
) -> ssize_t {
    let mut ret: ssize_t = 0;
    loop {
        ret = read(fd, buf, bufsize);
        if ret >= 0 as ssize_t || *__errno_location() != EINTR {
            break;
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn write_eintr(
    mut fd: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_void,
    mut bufsize: size_t,
) -> ssize_t {
    let mut ret: ssize_t = 0 as ssize_t;
    while (ret as size_t) < bufsize {
        let mut wlen: ssize_t = write(
            fd,
            (buf as *mut ::core::ffi::c_char).offset(ret as isize) as *const ::core::ffi::c_void,
            bufsize.wrapping_sub(ret as size_t),
        );
        if wlen < 0 as ssize_t {
            if *__errno_location() != EINTR {
                break;
            }
        } else {
            ret += wlen;
        }
    }
    return ret;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BAD_REPLACE: ::core::ffi::c_int = '?' as ::core::ffi::c_int;
pub const BAD_KEEP: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const BAD_DROP: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FORCE_NOBIN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENC_UCSBOM: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"ucs-bom\0") };
pub const EOL_UNKNOWN: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_FNAMER: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
pub const LOCK_SH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const NAME_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TEMP_DIR_NAMES: [*const ::core::ffi::c_char; 4] = [
    b"$TMPDIR\0".as_ptr() as *const ::core::ffi::c_char,
    b"/tmp\0".as_ptr() as *const ::core::ffi::c_char,
    b".\0".as_ptr() as *const ::core::ffi::c_char,
    b"~\0".as_ptr() as *const ::core::ffi::c_char,
];
pub const TEMP_FILE_PATH_MAXLEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
