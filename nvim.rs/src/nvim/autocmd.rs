extern "C" {
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
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn abort() -> !;
    fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
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
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(
        data: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn object_to_vim(obj: Object, tv: *mut typval_T, err: *mut Error);
    fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t;
    fn mh_get_String(set: *mut Set_String, key: String_0) -> uint32_t;
    fn map_del_int_ptr_t(
        map: *mut Map_int_ptr_t,
        key: ::core::ffi::c_int,
        key_alloc: *mut ::core::ffi::c_int,
    ) -> ptr_t;
    fn map_put_ref_int_ptr_t(
        map: *mut Map_int_ptr_t,
        key: ::core::ffi::c_int,
        key_alloc: *mut *mut ::core::ffi::c_int,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn map_del_String_int(
        map: *mut Map_String_int,
        key: String_0,
        key_alloc: *mut String_0,
    ) -> ::core::ffi::c_int;
    fn map_put_ref_String_int(
        map: *mut Map_String_int,
        key: String_0,
        key_alloc: *mut *mut String_0,
        new_item: *mut bool,
    ) -> *mut ::core::ffi::c_int;
    fn map_del_int_String(
        map: *mut Map_int_String,
        key: ::core::ffi::c_int,
        key_alloc: *mut ::core::ffi::c_int,
    ) -> String_0;
    fn map_put_ref_int_String(
        map: *mut Map_int_String,
        key: ::core::ffi::c_int,
        key_alloc: *mut *mut ::core::ffi::c_int,
        new_item: *mut bool,
    ) -> *mut String_0;
    static mut last_cursormoved_win: *mut win_T;
    static mut last_cursormoved: pos_T;
    static mut autocmd_busy: bool;
    static mut autocmd_no_enter: ::core::ffi::c_int;
    static mut autocmd_no_leave: ::core::ffi::c_int;
    static mut au_pending_free_buf: *mut buf_T;
    static mut au_pending_free_win: *mut win_T;
    static mut autocmd_fname: *mut ::core::ffi::c_char;
    static mut autocmd_fname_full: bool;
    static mut autocmd_bufnr: ::core::ffi::c_int;
    static mut autocmd_match: *mut ::core::ffi::c_char;
    static mut did_cursorhold: bool;
    static mut aucmd_win_vec: C2Rust_Unnamed_30;
    static mut deferred_events: *mut MultiQueue;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn do_modelines(flags: ::core::ffi::c_int);
    static mut p_acd: ::core::ffi::c_int;
    static mut p_ei: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
    fn xstrnsave(
        string: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strnicmp_asc(
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skipdigits(q: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn check_pos(buf: *mut buf_T, pos: *mut pos_T);
    fn check_cursor(wp: *mut win_T);
    static e_argreq: [::core::ffi::c_char; 0];
    static e_duparg2: [::core::ffi::c_char; 0];
    static e_cannot_define_autocommands_for_all_events: [::core::ffi::c_char; 0];
    fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T;
    fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T);
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn last_set_msg(script_ctx: sctx_T);
    fn hash_init(ht: *mut hashtab_T);
    fn smsg(
        hl_id: ::core::ffi::c_int,
        s: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
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
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn msg_clr_eos();
    fn msg_end() -> bool;
    fn verbose_enter();
    fn verbose_leave();
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn give_warning(message: *const ::core::ffi::c_char, hl: bool, hist: bool);
    fn msg_advance(col: ::core::ffi::c_int);
    fn callback_free(callback: *mut Callback);
    fn callback_copy(dest: *mut Callback, src: *mut Callback);
    fn callback_to_string(
        cb: *mut Callback,
        arena: *mut Arena,
    ) -> *mut ::core::ffi::c_char;
    fn tv_dict_add_tv(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        tv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_set_keys_readonly(dict: *mut dict_T);
    fn tv_clear(tv: *mut typval_T);
    fn save_funccal(entry: *mut funccal_entry_T);
    fn restore_funccal();
    fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T;
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T);
    fn set_cmdarg(
        eap: *mut exarg_T,
        oldarg: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vars_clear(ht: *mut hashtab_T);
    fn os_now() -> uint64_t;
    fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue;
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn expand_sfile(arg: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_pressedreturn() -> bool;
    fn set_pressedreturn(val: bool);
    fn aborting() -> bool;
    fn should_abort(retcode: ::core::ffi::c_int) -> bool;
    fn check_timestamps(focus: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn match_file_pat(
        pattern: *mut ::core::ffi::c_char,
        prog: *mut *mut regprog_T,
        fname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        tail: *mut ::core::ffi::c_char,
        allow_dirs: ::core::ffi::c_int,
    ) -> bool;
    fn file_pat_to_reg_pat(
        pat: *const ::core::ffi::c_char,
        pat_end: *const ::core::ffi::c_char,
        allow_dirs: *mut ::core::ffi::c_char,
        no_bslash: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn saveRedobuff(save_redo: *mut save_redo_T);
    fn restoreRedobuff(save_redo: *mut save_redo_T);
    static mut msg_col: ::core::ffi::c_int;
    static mut did_emsg: ::core::ffi::c_int;
    static mut need_maketitle: bool;
    static mut do_profiling: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut firstwin: *mut win_T;
    static mut lastwin: *mut win_T;
    static mut prevwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut starting: ::core::ffi::c_int;
    static mut secure: ::core::ffi::c_int;
    static mut VIsual: pos_T;
    static mut VIsual_active: bool;
    static mut reg_recording: ::core::ffi::c_int;
    static mut RedrawingDisabled: ::core::ffi::c_int;
    static mut typebuf: typebuf_T;
    static mut KeyTyped: bool;
    static mut got_int: bool;
    static mut last_mode: [::core::ffi::c_char; 4];
    static mut globaldir: *mut ::core::ffi::c_char;
    fn grid_free(grid: *mut ScreenGrid);
    fn ins_compl_active() -> bool;
    static mut window_handles: Map_int_ptr_t;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn api_free_string(value: String_0);
    fn api_free_object(value: Object);
    fn api_clear_error(value: *mut Error);
    fn copy_object(obj: Object, arena: *mut Arena) -> Object;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_set_sctx(current: *mut sctx_T);
    static mut main_loop: Loop;
    fn set_option_direct(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
        set_sid: scid_T,
    );
    fn line_breakcheck();
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn prof_child_enter(tm: *mut proftime_T);
    fn prof_child_exit(tm: *mut proftime_T);
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn FullName_save(
        fname: *const ::core::ffi::c_char,
        force: bool,
    ) -> *mut ::core::ffi::c_char;
    static mut exestack: garray_T;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn estack_push(
        type_0: etype_T,
        name: *mut ::core::ffi::c_char,
        lnum: linenr_T,
    ) -> *mut estack_T;
    fn estack_pop();
    fn save_search_patterns();
    fn restore_search_patterns();
    fn get_real_state() -> ::core::ffi::c_int;
    fn get_mode(buf: *mut ::core::ffi::c_char);
    fn ui_call_win_hide(grid: Integer);
    fn win_find_by_handle(handle: handle_T) -> *mut win_T;
    fn entering_window(win: *mut win_T);
    fn win_init_empty(wp: *mut win_T);
    fn unuse_tabpage(tp: *mut tabpage_T);
    fn use_tabpage(tp: *mut tabpage_T);
    fn win_alloc_aucmd_win(idx: ::core::ffi::c_int);
    fn valid_tabpage_win(tpc: *mut tabpage_T) -> ::core::ffi::c_int;
    fn close_tabpage(tab: *mut tabpage_T);
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn win_goto(wp: *mut win_T);
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn win_fix_current_dir();
    fn win_append(after: *mut win_T, wp: *mut win_T, tp: *mut tabpage_T);
    fn win_remove(wp: *mut win_T, tp: *mut tabpage_T);
    fn snapshot_windows_scroll_size();
    fn check_lnums(do_curwin: bool);
    fn check_lnums_nested(do_curwin: bool);
    fn reset_lnums();
    fn win_config_float(wp: *mut win_T, fconfig: WinConfig);
    fn ui_comp_remove_grid(grid: *mut ScreenGrid);
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
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
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_21,
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
    pub u: C2Rust_Unnamed_18,
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
    pub u: C2Rust_Unnamed_13,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
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
    pub active_reqs: C2Rust_Unnamed_17,
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
    pub timer_heap: C2Rust_Unnamed_16,
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
pub type uv__io_cb = Option<
    unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> (),
>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_15,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_14,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<
    unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_15 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_16 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
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
    pub u: C2Rust_Unnamed_20,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_19,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_19 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_20 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
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
pub type proc_state_cb = Option<
    unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> (),
>;
pub type proc_exit_cb = Option<
    unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
>;
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
    pub uv: C2Rust_Unnamed_23,
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
pub type stream_write_cb = Option<
    unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> (),
>;
pub type stream_close_cb = Option<
    unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> (),
>;
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
    pub u: C2Rust_Unnamed_22,
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
pub type uv_connection_cb = Option<
    unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> (),
>;
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
pub type uv_shutdown_cb = Option<
    unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> (),
>;
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
pub type uv_connect_cb = Option<
    unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> (),
>;
pub type uv_read_cb = Option<
    unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> (),
>;
pub type uv_alloc_cb = Option<
    unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_23 {
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
    pub u: C2Rust_Unnamed_24,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
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
    pub u: C2Rust_Unnamed_25,
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
pub union C2Rust_Unnamed_25 {
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
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_26 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type Buffer = handle_T;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_27 = 76;
pub const HLF_PRE: C2Rust_Unnamed_27 = 75;
pub const HLF_OK: C2Rust_Unnamed_27 = 74;
pub const HLF_SO: C2Rust_Unnamed_27 = 73;
pub const HLF_SE: C2Rust_Unnamed_27 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_27 = 71;
pub const HLF_TS: C2Rust_Unnamed_27 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_27 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_27 = 68;
pub const HLF_CU: C2Rust_Unnamed_27 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_27 = 66;
pub const HLF_WBR: C2Rust_Unnamed_27 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_27 = 64;
pub const HLF_MSG: C2Rust_Unnamed_27 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_27 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_27 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_27 = 60;
pub const HLF_0: C2Rust_Unnamed_27 = 59;
pub const HLF_QFL: C2Rust_Unnamed_27 = 58;
pub const HLF_MC: C2Rust_Unnamed_27 = 57;
pub const HLF_CUL: C2Rust_Unnamed_27 = 56;
pub const HLF_CUC: C2Rust_Unnamed_27 = 55;
pub const HLF_TPF: C2Rust_Unnamed_27 = 54;
pub const HLF_TPS: C2Rust_Unnamed_27 = 53;
pub const HLF_TP: C2Rust_Unnamed_27 = 52;
pub const HLF_PBR: C2Rust_Unnamed_27 = 51;
pub const HLF_PST: C2Rust_Unnamed_27 = 50;
pub const HLF_PSB: C2Rust_Unnamed_27 = 49;
pub const HLF_PSX: C2Rust_Unnamed_27 = 48;
pub const HLF_PNX: C2Rust_Unnamed_27 = 47;
pub const HLF_PSK: C2Rust_Unnamed_27 = 46;
pub const HLF_PNK: C2Rust_Unnamed_27 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_27 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_27 = 43;
pub const HLF_PSI: C2Rust_Unnamed_27 = 42;
pub const HLF_PNI: C2Rust_Unnamed_27 = 41;
pub const HLF_SPL: C2Rust_Unnamed_27 = 40;
pub const HLF_SPR: C2Rust_Unnamed_27 = 39;
pub const HLF_SPC: C2Rust_Unnamed_27 = 38;
pub const HLF_SPB: C2Rust_Unnamed_27 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_27 = 36;
pub const HLF_SC: C2Rust_Unnamed_27 = 35;
pub const HLF_TXA: C2Rust_Unnamed_27 = 34;
pub const HLF_TXD: C2Rust_Unnamed_27 = 33;
pub const HLF_DED: C2Rust_Unnamed_27 = 32;
pub const HLF_CHD: C2Rust_Unnamed_27 = 31;
pub const HLF_ADD: C2Rust_Unnamed_27 = 30;
pub const HLF_FC: C2Rust_Unnamed_27 = 29;
pub const HLF_FL: C2Rust_Unnamed_27 = 28;
pub const HLF_WM: C2Rust_Unnamed_27 = 27;
pub const HLF_W: C2Rust_Unnamed_27 = 26;
pub const HLF_VNC: C2Rust_Unnamed_27 = 25;
pub const HLF_V: C2Rust_Unnamed_27 = 24;
pub const HLF_T: C2Rust_Unnamed_27 = 23;
pub const HLF_VSP: C2Rust_Unnamed_27 = 22;
pub const HLF_C: C2Rust_Unnamed_27 = 21;
pub const HLF_SNC: C2Rust_Unnamed_27 = 20;
pub const HLF_S: C2Rust_Unnamed_27 = 19;
pub const HLF_R: C2Rust_Unnamed_27 = 18;
pub const HLF_CLF: C2Rust_Unnamed_27 = 17;
pub const HLF_CLS: C2Rust_Unnamed_27 = 16;
pub const HLF_CLN: C2Rust_Unnamed_27 = 15;
pub const HLF_LNB: C2Rust_Unnamed_27 = 14;
pub const HLF_LNA: C2Rust_Unnamed_27 = 13;
pub const HLF_N: C2Rust_Unnamed_27 = 12;
pub const HLF_CM: C2Rust_Unnamed_27 = 11;
pub const HLF_M: C2Rust_Unnamed_27 = 10;
pub const HLF_LC: C2Rust_Unnamed_27 = 9;
pub const HLF_L: C2Rust_Unnamed_27 = 8;
pub const HLF_I: C2Rust_Unnamed_27 = 7;
pub const HLF_E: C2Rust_Unnamed_27 = 6;
pub const HLF_D: C2Rust_Unnamed_27 = 5;
pub const HLF_AT: C2Rust_Unnamed_27 = 4;
pub const HLF_TERM: C2Rust_Unnamed_27 = 3;
pub const HLF_EOB: C2Rust_Unnamed_27 = 2;
pub const HLF_8: C2Rust_Unnamed_27 = 1;
pub const HLF_NONE: C2Rust_Unnamed_27 = 0;
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
pub struct Map_int_String {
    pub set: Set_int,
    pub values: *mut String_0,
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
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_28 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_28 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_28 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_28 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_28 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_28 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_28 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_28 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_28 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_28 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_28 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_28 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_28 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_28 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_28 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_28 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_28 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_28 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_28 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_28 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_28 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_28 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_28 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_28 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_28 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_28 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_28 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_28 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_28 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_28 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_28 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_28 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_28 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_28 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_28 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_28 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_28 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_28 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_28 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_28 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_28 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_28 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_28 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_28 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_28 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_28 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_28 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_28 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_28 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_28 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_28 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_28 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_28 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_28 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_28 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_28 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_28 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_28 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_28 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_28 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_28 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_28 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_28 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_28 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_28 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_28 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_28 = -2;
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
pub struct AutoCmd {
    pub pat: *mut AutoPat,
    pub id: int64_t,
    pub desc: *mut ::core::ffi::c_char,
    pub handler_cmd: *mut ::core::ffi::c_char,
    pub handler_fn: Callback,
    pub script_ctx: sctx_T,
    pub once: bool,
    pub nested: bool,
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
pub struct AutoCmdVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut AutoCmd,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoCmdEvent {
    pub event: event_T,
    pub fname: *mut ::core::ffi::c_char,
    pub fname_io: *mut ::core::ffi::c_char,
    pub buf: Buffer,
    pub group: ::core::ffi::c_int,
    pub eap: *mut exarg_T,
    pub data: *mut Object,
}
pub type argv_callback = Option<
    unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aucmdwin_T {
    pub auc_win: *mut win_T,
    pub auc_win_used: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_30 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub type C2Rust_Unnamed_31 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_31 = -4;
pub const AUGROUP_ALL: C2Rust_Unnamed_31 = -3;
pub const AUGROUP_ERROR: C2Rust_Unnamed_31 = -2;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_31 = -1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const BUFLOCAL_PAT_LEN: C2Rust_Unnamed_32 = 25;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_name {
    pub len: size_t,
    pub name: *mut ::core::ffi::c_char,
    pub event: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_33 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_redo_T {
    pub sr_redobuff: buffheader_T,
    pub sr_old_redobuff: buffheader_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffheader_T {
    pub bh_first: buffblock_T,
    pub bh_curr: *mut buffblock_T,
    pub bh_index: size_t,
    pub bh_space: size_t,
    pub bh_create_newblock: bool,
}
pub type buffblock_T = buffblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffblock {
    pub b_next: *mut buffblock,
    pub b_strlen: size_t,
    pub b_str: [::core::ffi::c_char; 1],
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
pub const DOCMD_REPEAT: C2Rust_Unnamed_34 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_34 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_34 = 2;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub type funccal_entry_T = funccal_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccal_entry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut funccal_entry_T,
}
pub const OPT_NOWIN: C2Rust_Unnamed_36 = 16;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
pub const MODE_INSERT: C2Rust_Unnamed_35 = 16;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_35 = 4097;
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
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_34 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_34 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_34 = 8;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_35 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_35 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_35 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_35 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_35 = 8193;
pub const MODE_LREPLACE: C2Rust_Unnamed_35 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_35 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_35 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_35 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_35 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_35 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_35 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_35 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_35 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_35 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_35 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_35 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_35 = 1;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_36 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_36 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_36 = 32;
pub const OPT_WINONLY: C2Rust_Unnamed_36 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_36 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_36 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_36 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
static mut value_init_int: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut value_init_String: String_0 = STRING_INIT;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
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
#[inline]
unsafe extern "C" fn map_put_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut value: String_0,
) {
    let mut val: *mut String_0 = map_put_ref_int_String(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
) -> String_0 {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_String
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut e_autocommand_nesting_too_deep: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<
        [u8; 35],
        [::core::ffi::c_char; 35],
    >(*b"E218: Autocommand nesting too deep\0")
};
static mut active_apc_list: *mut AutoPatCmd = ::core::ptr::null_mut::<AutoPatCmd>();
static mut next_augroup_id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut deleted_augroup: *const ::core::ffi::c_char = ::core::ptr::null::<
    ::core::ffi::c_char,
>();
static mut current_augroup: ::core::ffi::c_int = AUGROUP_DEFAULT as ::core::ffi::c_int;
static mut au_need_clean: bool = false_0 != 0;
static mut autocmd_blocked: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut autocmd_nested: bool = false_0 != 0;
static mut autocmd_include_groups: bool = false_0 != 0;
static mut termresponse_changed: bool = false_0 != 0;
static mut map_augroup_name_to_id: Map_String_int = Map_String_int {
    set: Set_String {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<String_0>(),
    },
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
static mut map_augroup_id_to_name: Map_int_String = Map_int_String {
    set: Set_int {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<String_0>(),
};
#[no_mangle]
pub unsafe extern "C" fn autocmd_init() {
    deferred_events = multiqueue_new_child(main_loop.events);
}
unsafe extern "C" fn augroup_map_del(
    mut id: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
) {
    if !name.is_null() {
        let mut key: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        map_del_String_int(
            &raw mut map_augroup_name_to_id,
            cstr_as_string(name),
            &raw mut key,
        );
        api_free_string(key);
    }
    if id > 0 as ::core::ffi::c_int {
        let mut mapped: String_0 = map_del_int_String(
            &raw mut map_augroup_id_to_name,
            id,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        api_free_string(mapped);
    }
}
#[inline(always)]
unsafe extern "C" fn get_deleted_augroup() -> *const ::core::ffi::c_char {
    if deleted_augroup.is_null() {
        deleted_augroup = gettext(
            b"--Deleted--\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return deleted_augroup;
}
unsafe extern "C" fn au_show_for_all_events(
    mut group: ::core::ffi::c_int,
    mut pat: *const ::core::ffi::c_char,
) {
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        au_show_for_event(group, event, pat);
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
}
unsafe extern "C" fn au_show_for_event(
    mut group: ::core::ffi::c_int,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
) {
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset(event as ::core::ffi::c_int as isize);
    if (*acs).size == 0 as size_t {
        return;
    }
    let mut patlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *pat as ::core::ffi::c_int != NUL {
        patlen = aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
        if patlen == 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    let mut last_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
    let mut last_group_name: *const ::core::ffi::c_char = ::core::ptr::null::<
        ::core::ffi::c_char,
    >();
    loop {
        let mut last_ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
        let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
        if aupat_is_buflocal(pat, patlen) {
            aupat_normalize_buflocal_pat(
                &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                pat,
                patlen,
                aupat_get_buflocal_nr(pat, patlen),
            );
            pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
            patlen = strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int;
        }
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if !(*ac).pat.is_null() {
                if !(group != AUGROUP_ALL as ::core::ffi::c_int
                    && (*(*ac).pat).group != group
                    || patlen != 0
                        && ((*(*ac).pat).patlen != patlen
                            || strncmp(pat, (*(*ac).pat).pat, patlen as size_t)
                                != 0 as ::core::ffi::c_int))
                {
                    if (*(*ac).pat).group != last_group {
                        last_group = (*(*ac).pat).group;
                        last_group_name = augroup_name((*(*ac).pat).group);
                        if got_int {
                            return;
                        }
                        msg_putchar('\n' as ::core::ffi::c_int);
                        if got_int {
                            return;
                        }
                        if (*(*ac).pat).group != AUGROUP_DEFAULT as ::core::ffi::c_int {
                            if last_group_name.is_null() {
                                msg_puts_hl(
                                    get_deleted_augroup(),
                                    HLF_E as ::core::ffi::c_int,
                                    false_0 != 0,
                                );
                            } else {
                                msg_puts_hl(
                                    last_group_name,
                                    HLF_T as ::core::ffi::c_int,
                                    false_0 != 0,
                                );
                            }
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        }
                        msg_puts_hl(
                            event_nr2name(event),
                            HLF_T as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                    }
                    if last_ap != (*ac).pat {
                        last_ap = (*ac).pat;
                        msg_putchar('\n' as ::core::ffi::c_int);
                        if got_int {
                            return;
                        }
                        msg_advance(4 as ::core::ffi::c_int);
                        msg_outtrans(
                            (*(*ac).pat).pat,
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                    }
                    if got_int {
                        return;
                    }
                    if msg_col >= 14 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    msg_advance(14 as ::core::ffi::c_int);
                    if got_int {
                        return;
                    }
                    let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(
                        ac,
                    );
                    if !(*ac).desc.is_null() {
                        let mut msglen: size_t = 100 as size_t;
                        let mut msg: *mut ::core::ffi::c_char = xmallocz(msglen)
                            as *mut ::core::ffi::c_char;
                        if !(*ac).handler_cmd.is_null() {
                            snprintf(
                                msg,
                                msglen,
                                b"%s [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                handler_str,
                                (*ac).desc,
                            );
                        } else {
                            msg_puts_hl(
                                handler_str,
                                HLF_8 as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                            snprintf(
                                msg,
                                msglen,
                                b" [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                (*ac).desc,
                            );
                        }
                        msg_outtrans(msg, 0 as ::core::ffi::c_int, false_0 != 0);
                        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut msg
                            as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        *ptr_;
                    } else if !(*ac).handler_cmd.is_null() {
                        msg_outtrans(handler_str, 0 as ::core::ffi::c_int, false_0 != 0);
                    } else {
                        msg_puts_hl(
                            handler_str,
                            HLF_8 as ::core::ffi::c_int,
                            false_0 != 0,
                        );
                    }
                    let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut handler_str
                        as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL_0;
                    *ptr__0;
                    if p_verbose > 0 as OptInt {
                        last_set_msg((*ac).script_ctx);
                    }
                    if got_int {
                        return;
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
        if patlen == 0 {
            break;
        }
    };
}
unsafe extern "C" fn aucmd_del(mut ac: *mut AutoCmd) {
    if !(*ac).pat.is_null()
        && {
            (*(*ac).pat).refcount = (*(*ac).pat).refcount.wrapping_sub(1);
            (*(*ac).pat).refcount == 0 as size_t
        }
    {
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(*ac).pat).pat
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        vim_regfree((*(*ac).pat).reg_prog);
        xfree((*ac).pat as *mut ::core::ffi::c_void);
    }
    (*ac).pat = ::core::ptr::null_mut::<AutoPat>();
    if !(*ac).handler_cmd.is_null() {
        let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut (*ac).handler_cmd
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        *ptr__0;
    } else {
        callback_free(&raw mut (*ac).handler_fn);
    }
    let mut ptr__1: *mut *mut ::core::ffi::c_void = &raw mut (*ac).desc
        as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    *ptr__1;
    au_need_clean = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_del_for_event_and_group(
    mut event: event_T,
    mut group: ::core::ffi::c_int,
) {
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset(event as ::core::ffi::c_int as isize);
    let mut i: size_t = 0 as size_t;
    while i < (*acs).size {
        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
        if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
            aucmd_del(ac);
        }
        i = i.wrapping_add(1);
    }
    au_cleanup();
}
unsafe extern "C" fn au_cleanup() {
    if autocmd_busy as ::core::ffi::c_int != 0 || !au_need_clean {
        return;
    }
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
            .offset(event as ::core::ffi::c_int as isize);
        let mut nsize: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if nsize != i {
                *(*acs).items.offset(nsize as isize) = *ac;
            }
            if !(*ac).pat.is_null() {
                nsize = nsize.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if nsize == 0 as size_t {
            xfree((*acs).items as *mut ::core::ffi::c_void);
            (*acs).capacity = 0 as size_t;
            (*acs).size = (*acs).capacity;
            (*acs).items = ::core::ptr::null_mut::<AutoCmd>();
        } else {
            (*acs).size = nsize;
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    au_need_clean = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn au_get_autocmds_for_event(
    mut event: event_T,
) -> *mut AutoCmdVec {
    return (&raw mut autocmds as *mut AutoCmdVec)
        .offset(event as ::core::ffi::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn aubuflocal_remove(mut buf: *mut buf_T) {
    let mut apc: *mut AutoPatCmd = active_apc_list;
    while !apc.is_null() {
        if (*buf).handle == (*apc).arg_bufnr {
            (*apc).arg_bufnr = 0 as ::core::ffi::c_int;
        }
        apc = (*apc).next;
    }
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
            .offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if !((*ac).pat.is_null() || (*(*ac).pat).buflocal_nr != (*buf).handle) {
                aucmd_del(ac);
                if p_verbose >= 6 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(
                            b"auto-removing autocommand: %s <buffer=%d>\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        event_nr2name(event),
                        (*buf).handle,
                    );
                    verbose_leave();
                }
            }
            i = i.wrapping_add(1);
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    au_cleanup();
}
#[no_mangle]
pub unsafe extern "C" fn augroup_add(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
        {} else {
            __assert_fail(
                b"STRICMP(name, \"end\") != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                400 as ::core::ffi::c_uint,
                b"int augroup_add(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut existing_id: ::core::ffi::c_int = augroup_find(name);
    if existing_id > 0 as ::core::ffi::c_int {
        '_c2rust_label_0: {
            if existing_id != AUGROUP_DELETED as ::core::ffi::c_int {} else {
                __assert_fail(
                    b"existing_id != AUGROUP_DELETED\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    404 as ::core::ffi::c_uint,
                    b"int augroup_add(const char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return existing_id;
    }
    if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
        augroup_map_del(existing_id, name);
    }
    let c2rust_fresh0 = next_augroup_id;
    next_augroup_id = next_augroup_id + 1;
    let mut next_id: ::core::ffi::c_int = c2rust_fresh0;
    let mut name_key: String_0 = cstr_to_string(name);
    let mut name_val: String_0 = cstr_to_string(name);
    map_put_String_int(&raw mut map_augroup_name_to_id, name_key, next_id);
    map_put_int_String(&raw mut map_augroup_id_to_name, next_id, name_val);
    return next_id;
}
#[no_mangle]
pub unsafe extern "C" fn augroup_del(
    mut name: *mut ::core::ffi::c_char,
    mut stupid_legacy_mode: bool,
) {
    let mut group: ::core::ffi::c_int = augroup_find(name);
    if group == AUGROUP_ERROR as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E367: No such group: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            ),
            name,
        );
        return;
    } else if group == current_augroup {
        emsg(
            gettext(
                b"E936: Cannot delete the current group\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return;
    }
    if stupid_legacy_mode {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
                .offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                if !ap.is_null() && (*ap).group == group {
                    give_warning(
                        gettext(
                            b"W19: Deleting augroup that is still in use\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        true_0 != 0,
                        true_0 != 0,
                    );
                    map_put_String_int(
                        &raw mut map_augroup_name_to_id,
                        cstr_as_string(name),
                        AUGROUP_DELETED as ::core::ffi::c_int,
                    );
                    augroup_map_del(
                        (*ap).group,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                i = i.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    } else {
        let mut event_0: event_T = EVENT_BUFADD;
        while (event_0 as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs_0: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
                .offset(event_0 as ::core::ffi::c_int as isize);
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*acs_0).size {
                let ac: *mut AutoCmd = (*acs_0).items.offset(i_0 as isize);
                if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
                    aucmd_del(ac);
                }
                i_0 = i_0.wrapping_add(1);
            }
            event_0 = (event_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                as event_T;
        }
    }
    augroup_map_del(group, name);
    au_cleanup();
}
#[no_mangle]
pub unsafe extern "C" fn augroup_find(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut existing_id: ::core::ffi::c_int = map_get_String_int(
        &raw mut map_augroup_name_to_id,
        cstr_as_string(name),
    );
    if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
        return existing_id;
    }
    if existing_id > 0 as ::core::ffi::c_int {
        return existing_id;
    }
    return AUGROUP_ERROR as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn augroup_name(
    mut group: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if group != 0 as ::core::ffi::c_int {} else {
            __assert_fail(
                b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                496 as ::core::ffi::c_uint,
                b"char *augroup_name(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if group == AUGROUP_DELETED as ::core::ffi::c_int {
        return get_deleted_augroup() as *mut ::core::ffi::c_char;
    }
    if group == AUGROUP_ALL as ::core::ffi::c_int {
        group = current_augroup;
    }
    if group == next_augroup_id {
        return b"END\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char;
    }
    if group > next_augroup_id {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut key: String_0 = map_get_int_String(&raw mut map_augroup_id_to_name, group);
    if !key.data.is_null() {
        return key.data;
    }
    return get_deleted_augroup() as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn augroup_exists(mut name: *const ::core::ffi::c_char) -> bool {
    return augroup_find(name) > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn do_augroup(
    mut arg: *mut ::core::ffi::c_char,
    mut del_group: bool,
) {
    if del_group {
        if *arg as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        } else {
            augroup_del(arg, true_0 != 0);
        }
    } else if strcasecmp(
        arg,
        b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        current_augroup = AUGROUP_DEFAULT as ::core::ffi::c_int;
    } else if *arg != 0 {
        current_augroup = augroup_add(arg);
    } else {
        msg_start();
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut name: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        let mut value: ::core::ffi::c_int = 0;
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < map_augroup_name_to_id.set.h.n_keys {
            name = *map_augroup_name_to_id.set.keys.offset(__i as isize);
            value = *map_augroup_name_to_id.values.offset(__i as isize);
            if value > 0 as ::core::ffi::c_int {
                msg_puts(name.data);
            } else {
                msg_puts(augroup_name(value));
            }
            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
            __i = __i.wrapping_add(1);
        }
        msg_clr_eos();
        msg_end();
    };
}
#[no_mangle]
pub unsafe extern "C" fn is_aucmd_win(mut win: *mut win_T) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < aucmd_win_vec.size as ::core::ffi::c_int {
        if (*aucmd_win_vec.items.offset(i as isize)).auc_win_used as ::core::ffi::c_int
            != 0 && (*aucmd_win_vec.items.offset(i as isize)).auc_win == win
        {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn event_name2nr(
    mut start: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
) -> event_T {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = start;
    while *p as ::core::ffi::c_int != 0 && !ascii_iswhite(*p as ::core::ffi::c_int)
        && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    let mut hash_idx: ::core::ffi::c_int = event_name2nr_hash(
        start,
        p.offset_from(start) as size_t,
    );
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    *end = p as *mut ::core::ffi::c_char;
    if hash_idx < 0 as ::core::ffi::c_int {
        return NUM_EVENTS;
    }
    return abs(event_names[event_hash[hash_idx as usize] as usize].event) as event_T;
}
#[no_mangle]
pub unsafe extern "C" fn event_name2nr_str(mut str: String_0) -> event_T {
    let mut hash_idx: ::core::ffi::c_int = event_name2nr_hash(str.data, str.size);
    if hash_idx < 0 as ::core::ffi::c_int {
        return NUM_EVENTS;
    }
    return abs(event_names[event_hash[hash_idx as usize] as usize].event) as event_T;
}
#[no_mangle]
pub unsafe extern "C" fn event_nr2name(
    mut event: event_T,
) -> *const ::core::ffi::c_char {
    return if event as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
        && (event as ::core::ffi::c_uint)
            < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        event_names[event as usize].name as *const ::core::ffi::c_char
    } else {
        b"Unknown\0".as_ptr() as *const ::core::ffi::c_char
    };
}
#[no_mangle]
pub unsafe extern "C" fn event_ignored(
    mut event: event_T,
    mut ei: *mut ::core::ffi::c_char,
) -> bool {
    let mut ignored: bool = false_0 != 0;
    while *ei as ::core::ffi::c_int != NUL {
        let mut unignore: bool = *ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
        ei = ei.offset(unignore as ::core::ffi::c_int as isize);
        if strncasecmp(
            ei,
            b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            3 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int)
        {
            ignored = ei == p_ei
                || event_names[event as usize].event <= 0 as ::core::ffi::c_int;
            ei = ei
                .offset(
                    (3 as ::core::ffi::c_int
                        + (*ei.offset(3 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                            as ::core::ffi::c_int) as isize,
                );
        } else if event_name2nr(ei, &raw mut ei) as ::core::ffi::c_uint
            == event as ::core::ffi::c_uint
        {
            if unignore {
                return false_0 != 0;
            }
            ignored = true_0 != 0;
        }
    }
    return ignored;
}
#[no_mangle]
pub unsafe extern "C" fn check_ei(
    mut ei: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut win: bool = ei != p_ei;
    while *ei != 0 {
        if strncasecmp(
            ei,
            b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            3 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int)
        {
            ei = ei
                .offset(
                    (3 as ::core::ffi::c_int
                        + (*ei.offset(3 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                            as ::core::ffi::c_int) as isize,
                );
        } else {
            ei = ei
                .offset(
                    (*ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int)
                        as ::core::ffi::c_int as isize,
                );
            let mut event: event_T = event_name2nr(ei, &raw mut ei);
            if event as ::core::ffi::c_uint
                == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                || win as ::core::ffi::c_int != 0
                    && event_names[event as usize].event > 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn au_event_disable(
    mut what: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut p_ei_len: size_t = strlen(p_ei);
    let mut save_ei: *mut ::core::ffi::c_char = xmemdupz(
        p_ei as *const ::core::ffi::c_void,
        p_ei_len,
    ) as *mut ::core::ffi::c_char;
    let mut new_ei: *mut ::core::ffi::c_char = xstrnsave(
        p_ei,
        p_ei_len.wrapping_add(strlen(what)),
    );
    if *what as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        && *p_ei as ::core::ffi::c_int == NUL
    {
        strcpy(new_ei, what.offset(1 as ::core::ffi::c_int as isize));
    } else {
        strcpy(new_ei.offset(p_ei_len as isize), what);
    }
    set_option_direct(
        kOptEventignore,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(new_ei),
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    xfree(new_ei as *mut ::core::ffi::c_void);
    return save_ei;
}
#[no_mangle]
pub unsafe extern "C" fn au_event_restore(mut old_ei: *mut ::core::ffi::c_char) {
    if !old_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(old_ei),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        xfree(old_ei as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd(
    mut eap: *mut exarg_T,
    mut arg_in: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
) {
    let mut arg: *mut ::core::ffi::c_char = arg_in;
    let mut envpat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut need_free: bool = false_0 != 0;
    let mut nested: bool = false_0 != 0;
    let mut once: bool = false_0 != 0;
    let mut group: ::core::ffi::c_int = 0;
    if *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
        (*eap).nextcmd = arg.offset(1 as ::core::ffi::c_int as isize);
        arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        group = AUGROUP_ALL as ::core::ffi::c_int;
    } else {
        group = arg_augroup_get(&raw mut arg);
    }
    let mut pat: *mut ::core::ffi::c_char = arg_event_skip(
        arg,
        group != AUGROUP_ALL as ::core::ffi::c_int,
    );
    if pat.is_null() {
        return;
    }
    pat = skipwhite(pat);
    if *pat as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
        (*eap).nextcmd = pat.offset(1 as ::core::ffi::c_int as isize);
        pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        cmd = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        cmd = pat;
        while *cmd as ::core::ffi::c_int != 0
            && (!ascii_iswhite(*cmd as ::core::ffi::c_int)
                || *cmd.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
        {
            cmd = cmd.offset(1);
        }
        if *cmd != 0 {
            let c2rust_fresh1 = cmd;
            cmd = cmd.offset(1);
            *c2rust_fresh1 = NUL as ::core::ffi::c_char;
        }
        if !vim_strchr(pat, '$' as ::core::ffi::c_int).is_null()
            || !vim_strchr(pat, '~' as ::core::ffi::c_int).is_null()
        {
            envpat = expand_env_save(pat);
            if !envpat.is_null() {
                pat = envpat;
            }
        }
        cmd = skipwhite(cmd);
        let mut invalid_flags: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < 2 as size_t {
            if *cmd as ::core::ffi::c_int != NUL {
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut once,
                        &raw mut cmd,
                        b"++once\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        6 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int != 0;
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut nested,
                        &raw mut cmd,
                        b"++nested\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        8 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int != 0;
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut nested,
                        &raw mut cmd,
                        b"nested\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        6 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int != 0;
            }
            i = i.wrapping_add(1);
        }
        if invalid_flags {
            return;
        }
        if *cmd as ::core::ffi::c_int != NUL {
            cmd = expand_sfile(cmd);
            if cmd.is_null() {
                return;
            }
            need_free = true_0 != 0;
        }
    }
    let is_showing: bool = forceit == 0 && *cmd as ::core::ffi::c_int == NUL;
    if is_showing {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts_title(
            gettext(b"\n--- Autocommands ---\0".as_ptr() as *const ::core::ffi::c_char),
        );
        if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == NUL
        {
            au_show_for_all_events(group, pat);
        } else {
            let mut event: event_T = event_name2nr(arg, &raw mut arg);
            '_c2rust_label: {
                if (event as ::core::ffi::c_uint)
                    < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                {} else {
                    __assert_fail(
                        b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        860 as ::core::ffi::c_uint,
                        b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            au_show_for_event(group, event, pat);
        }
    } else if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        || *arg as ::core::ffi::c_int == NUL
        || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
    {
        if *cmd as ::core::ffi::c_int != NUL {
            emsg(
                gettext(
                    &raw const e_cannot_define_autocommands_for_all_events
                        as *const ::core::ffi::c_char,
                ),
            );
        } else {
            do_all_autocmd_events(
                pat,
                once,
                nested as ::core::ffi::c_int,
                cmd,
                forceit != 0,
                group,
            );
        }
    } else {
        while *arg as ::core::ffi::c_int != 0
            && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
            && !ascii_iswhite(*arg as ::core::ffi::c_int)
        {
            let mut event_0: event_T = event_name2nr(arg, &raw mut arg);
            '_c2rust_label_0: {
                if (event_0 as ::core::ffi::c_uint)
                    < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                {} else {
                    __assert_fail(
                        b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        873 as ::core::ffi::c_uint,
                        b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if do_autocmd_event(
                event_0,
                pat,
                once,
                nested as ::core::ffi::c_int,
                cmd,
                forceit != 0,
                group,
            ) == FAIL
            {
                break;
            }
        }
    }
    if need_free {
        xfree(cmd as *mut ::core::ffi::c_void);
    }
    xfree(envpat as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn do_all_autocmd_events(
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *mut ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) {
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        if do_autocmd_event(event, pat, once, nested, cmd, del, group) == FAIL {
            return;
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_event(
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *const ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
            || del as ::core::ffi::c_int != 0
        {} else {
            __assert_fail(
                b"*pat != NUL || del\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                908 as ::core::ffi::c_uint,
                b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    let mut is_adding_cmd: bool = *cmd as ::core::ffi::c_int != NUL;
    let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
        current_augroup
    } else {
        group
    };
    if *pat as ::core::ffi::c_int == NUL && del as ::core::ffi::c_int != 0 {
        aucmd_del_for_event_and_group(event, findgroup);
        return OK;
    }
    let mut patlen: ::core::ffi::c_int = aucmd_span_pattern(pat, &raw mut pat)
        as ::core::ffi::c_int;
    while patlen != 0 {
        let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
        let mut is_buflocal: bool = aupat_is_buflocal(pat, patlen);
        if is_buflocal {
            let buflocal_nr: ::core::ffi::c_int = aupat_get_buflocal_nr(pat, patlen);
            aupat_normalize_buflocal_pat(
                &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                pat,
                patlen,
                buflocal_nr,
            );
            pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
            patlen = strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int;
        }
        if del {
            '_c2rust_label_0: {
                if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {} else {
                    __assert_fail(
                        b"*pat != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        939 as ::core::ffi::c_uint,
                        b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
                .offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                let ap: *mut AutoPat = (*ac).pat;
                if !ap.is_null() && (*ap).group == findgroup && (*ap).patlen == patlen
                    && strncmp(pat, (*ap).pat, patlen as size_t)
                        == 0 as ::core::ffi::c_int
                {
                    aucmd_del(ac);
                }
                i = i.wrapping_add(1);
            }
        }
        if is_adding_cmd {
            let mut handler_fn: Callback = Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            };
            autocmd_register(
                0 as int64_t,
                event,
                pat,
                patlen,
                group,
                once,
                nested != 0,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd,
                &raw mut handler_fn,
            );
        }
        patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
    }
    au_cleanup();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn autocmd_register(
    mut id: int64_t,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut group: ::core::ffi::c_int,
    mut once: bool,
    mut nested: bool,
    mut desc: *mut ::core::ffi::c_char,
    mut handler_cmd: *const ::core::ffi::c_char,
    mut handler_fn: *mut Callback,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if group != 0 as ::core::ffi::c_int {} else {
            __assert_fail(
                b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                984 as ::core::ffi::c_uint,
                b"int autocmd_register(int64_t, event_T, const char *, int, int, _Bool, _Bool, char *, const char *, Callback *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if patlen > strlen(pat) as ::core::ffi::c_int {
        return FAIL;
    }
    let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
        current_augroup
    } else {
        group
    };
    let is_buflocal: bool = aupat_is_buflocal(pat, patlen);
    let mut buflocal_nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    if is_buflocal {
        buflocal_nr = aupat_get_buflocal_nr(pat, patlen);
        aupat_normalize_buflocal_pat(
            &raw mut buflocal_pat as *mut ::core::ffi::c_char,
            pat,
            patlen,
            buflocal_nr,
        );
        pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
        patlen = strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int;
    }
    let mut ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset(event as ::core::ffi::c_int as isize);
    let mut i: ptrdiff_t = (*acs).size as ptrdiff_t - 1 as ptrdiff_t;
    while i >= 0 as ptrdiff_t {
        ap = (*(*acs).items.offset(i as isize)).pat;
        if ap.is_null() {
            i -= 1;
        } else {
            if (*ap).group != findgroup || (*ap).patlen != patlen
                || strncmp(pat, (*ap).pat, patlen as size_t) != 0 as ::core::ffi::c_int
            {
                ap = ::core::ptr::null_mut::<AutoPat>();
            }
            break;
        }
    }
    if ap.is_null() {
        if is_buflocal as ::core::ffi::c_int != 0
            && (buflocal_nr == 0 as ::core::ffi::c_int
                || buflist_findnr(buflocal_nr).is_null())
        {
            semsg(
                gettext(
                    b"E680: <buffer=%d>: invalid buffer number \0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                buflocal_nr,
            );
            return FAIL;
        }
        ap = xmalloc(::core::mem::size_of::<AutoPat>()) as *mut AutoPat;
        if is_buflocal {
            (*ap).buflocal_nr = buflocal_nr;
            (*ap).reg_prog = ::core::ptr::null_mut::<regprog_T>();
        } else {
            (*ap).buflocal_nr = 0 as ::core::ffi::c_int;
            let mut reg_pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
                pat,
                pat.offset(patlen as isize),
                &raw mut (*ap).allow_dirs,
                true_0,
            );
            if !reg_pat.is_null() {
                (*ap).reg_prog = vim_regcomp(reg_pat, RE_MAGIC);
            }
            xfree(reg_pat as *mut ::core::ffi::c_void);
            if reg_pat.is_null() || (*ap).reg_prog.is_null() {
                xfree(ap as *mut ::core::ffi::c_void);
                return FAIL;
            }
        }
        (*ap).refcount = 0 as size_t;
        (*ap).pat = xmemdupz(pat as *const ::core::ffi::c_void, patlen as size_t)
            as *mut ::core::ffi::c_char;
        (*ap).patlen = patlen;
        if event as ::core::ffi::c_uint
            == EVENT_MODECHANGED as ::core::ffi::c_int as ::core::ffi::c_uint
            && !has_event(EVENT_MODECHANGED)
        {
            get_mode(&raw mut last_mode as *mut ::core::ffi::c_char);
        }
        if event as ::core::ffi::c_uint
            == EVENT_CURSORMOVED as ::core::ffi::c_int as ::core::ffi::c_uint
            && !has_event(EVENT_CURSORMOVED)
            || event as ::core::ffi::c_uint
                == EVENT_CURSORMOVEDI as ::core::ffi::c_int as ::core::ffi::c_uint
                && !has_event(EVENT_CURSORMOVEDI)
        {
            last_cursormoved_win = curwin;
            last_cursormoved = (*curwin).w_cursor;
        }
        if (event as ::core::ffi::c_uint
            == EVENT_WINSCROLLED as ::core::ffi::c_int as ::core::ffi::c_uint
            || event as ::core::ffi::c_uint
                == EVENT_WINRESIZED as ::core::ffi::c_int as ::core::ffi::c_uint)
            && !(has_event(EVENT_WINSCROLLED) as ::core::ffi::c_int != 0
                || has_event(EVENT_WINRESIZED) as ::core::ffi::c_int != 0)
        {
            let mut save_curtab: *mut tabpage_T = curtab;
            let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
            while !tp.is_null() {
                unuse_tabpage(curtab);
                use_tabpage(tp as *mut tabpage_T);
                snapshot_windows_scroll_size();
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            unuse_tabpage(curtab);
            use_tabpage(save_curtab);
        }
        (*ap).group = if group == AUGROUP_ALL as ::core::ffi::c_int {
            current_augroup
        } else {
            group
        };
    }
    (*ap).refcount = (*ap).refcount.wrapping_add(1);
    if autocmds[event as ::core::ffi::c_int as usize].size
        == autocmds[event as ::core::ffi::c_int as usize].capacity
    {
        autocmds[event as ::core::ffi::c_int as usize].capacity = (if autocmds[event
                as ::core::ffi::c_int as usize]
            .capacity != 0
        {
            autocmds[event as ::core::ffi::c_int as usize].capacity
                << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        });
        autocmds[event as ::core::ffi::c_int as usize].items = xrealloc(
            autocmds[event as ::core::ffi::c_int as usize].items
                as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<AutoCmd>()
                .wrapping_mul(autocmds[event as ::core::ffi::c_int as usize].capacity),
        ) as *mut AutoCmd;
    } else {};
    let c2rust_fresh2 = autocmds[event as ::core::ffi::c_int as usize].size;
    autocmds[event as ::core::ffi::c_int as usize].size = autocmds[event
            as ::core::ffi::c_int as usize]
        .size
        .wrapping_add(1);
    let mut ac: *mut AutoCmd = autocmds[event as ::core::ffi::c_int as usize]
        .items
        .offset(c2rust_fresh2 as isize);
    (*ac).pat = ap;
    (*ac).id = id;
    if !handler_cmd.is_null() {
        (*ac).handler_cmd = xstrdup(handler_cmd);
    } else {
        (*ac).handler_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
        callback_copy(&raw mut (*ac).handler_fn, handler_fn);
    }
    (*ac).script_ctx = current_sctx;
    (*ac).script_ctx.sc_lnum
        += (*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum;
    nlua_set_sctx(&raw mut (*ac).script_ctx);
    (*ac).once = once;
    (*ac).nested = nested;
    (*ac).desc = if desc.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(desc)
    };
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_span_pattern(
    mut pat: *const ::core::ffi::c_char,
    mut start: *mut *const ::core::ffi::c_char,
) -> size_t {
    while *pat as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        pat = pat.offset(1);
    }
    let mut p: *const ::core::ffi::c_char = pat;
    let mut brace_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != 0
        && (*p as ::core::ffi::c_int != ',' as ::core::ffi::c_int || brace_level != 0
            || p > pat
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
    {
        if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            brace_level += 1;
        } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
            brace_level -= 1;
        }
        p = p.offset(1);
    }
    *start = pat;
    return p.offset_from(pat) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn do_doautocmd(
    mut arg_start: *mut ::core::ffi::c_char,
    mut do_msg: bool,
    mut did_something: *mut bool,
) -> ::core::ffi::c_int {
    let mut arg: *mut ::core::ffi::c_char = arg_start;
    let mut nothing_done: ::core::ffi::c_int = true_0;
    if !did_something.is_null() {
        *did_something = false_0 != 0;
    }
    let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        emsg(
            gettext(
                b"E217: Can't execute autocommands for ALL events\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    let mut fname: *mut ::core::ffi::c_char = arg_event_skip(
        arg,
        group != AUGROUP_ALL as ::core::ffi::c_int,
    );
    if fname.is_null() {
        return FAIL;
    }
    fname = skipwhite(fname);
    while *arg as ::core::ffi::c_int != 0 && ends_excmd(*arg as ::core::ffi::c_int) == 0
        && !ascii_iswhite(*arg as ::core::ffi::c_int)
    {
        if apply_autocmds_group(
            event_name2nr(arg, &raw mut arg),
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            group,
            curbuf,
            ::core::ptr::null_mut::<exarg_T>(),
            ::core::ptr::null_mut::<Object>(),
        ) {
            nothing_done = false_0;
        }
    }
    if nothing_done != 0 && do_msg as ::core::ffi::c_int != 0 && !aborting() {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(
                b"No matching autocommands: %s\0".as_ptr() as *const ::core::ffi::c_char,
            ),
            arg_start,
        );
    }
    if !did_something.is_null() {
        *did_something = nothing_done == 0;
    }
    return if aborting() as ::core::ffi::c_int != 0 { FAIL } else { OK };
}
#[no_mangle]
pub unsafe extern "C" fn ex_doautoall(mut eap: *mut exarg_T) {
    let mut retval: ::core::ffi::c_int = OK;
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
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut call_do_modelines: ::core::ffi::c_int = check_nomodeline(&raw mut arg)
        as ::core::ffi::c_int;
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut did_aucmd: bool = false;
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        if !((*buf).b_ml.ml_mfp.is_null() || buf == curbuf) {
            aucmd_prepbuf(&raw mut aco, buf);
            set_bufref(&raw mut bufref, buf);
            retval = do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
            if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
                do_modelines(
                    if is_aucmd_win(curwin) as ::core::ffi::c_int != 0 {
                        OPT_NOWIN as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
            }
            aucmd_restbuf(&raw mut aco);
            if retval == FAIL || !bufref_valid(&raw mut bufref) {
                retval = FAIL;
                break;
            }
        }
        buf = (*buf).b_next;
    }
    if retval == OK {
        do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
        if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
            do_modelines(0 as ::core::ffi::c_int);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_nomodeline(
    mut argp: *mut *mut ::core::ffi::c_char,
) -> bool {
    if strncmp(
        *argp,
        b"<nomodeline>\0".as_ptr() as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite((*argp).offset(12 as ::core::ffi::c_int as isize));
        return false_0 != 0;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_prepbuf(mut aco: *mut aco_save_T, mut buf: *mut buf_T) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut need_append: bool = true_0 != 0;
    let same_buffer: bool = buf == curbuf;
    if same_buffer {
        win = curwin;
    } else {
        win = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = if curtab == curtab {
            firstwin
        } else {
            (*curtab).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                win = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
    }
    let mut auc_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut auc_idx: ::core::ffi::c_int = aucmd_win_vec.size as ::core::ffi::c_int;
    if win.is_null() {
        auc_idx = 0 as ::core::ffi::c_int;
        while auc_idx < aucmd_win_vec.size as ::core::ffi::c_int {
            if !(*aucmd_win_vec.items.offset(auc_idx as isize)).auc_win_used {
                break;
            }
            auc_idx += 1;
        }
        if auc_idx == aucmd_win_vec.size as ::core::ffi::c_int {
            if aucmd_win_vec.size == aucmd_win_vec.capacity {
                aucmd_win_vec.capacity = (if aucmd_win_vec.capacity != 0 {
                    aucmd_win_vec.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                });
                aucmd_win_vec.items = xrealloc(
                    aucmd_win_vec.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<aucmdwin_T>()
                        .wrapping_mul(aucmd_win_vec.capacity),
                ) as *mut aucmdwin_T;
            } else {};
            let c2rust_fresh12 = aucmd_win_vec.size;
            aucmd_win_vec.size = aucmd_win_vec.size.wrapping_add(1);
            *aucmd_win_vec.items.offset(c2rust_fresh12 as isize) = aucmdwin_T {
                auc_win: ::core::ptr::null_mut::<win_T>(),
                auc_win_used: false,
            };
        }
        if (*aucmd_win_vec.items.offset(auc_idx as isize)).auc_win.is_null() {
            win_alloc_aucmd_win(auc_idx);
            need_append = false_0 != 0;
        }
        auc_win = (*aucmd_win_vec.items.offset(auc_idx as isize)).auc_win;
        (*aucmd_win_vec.items.offset(auc_idx as isize)).auc_win_used = true_0 != 0;
    }
    (*aco).save_curwin_handle = (*curwin).handle;
    (*aco).save_prevwin_handle = (if prevwin.is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*prevwin).handle as ::core::ffi::c_int
    }) as handle_T;
    if bt_prompt(curbuf) {
        (*aco).save_prompt_insert = (*curbuf).b_prompt_insert;
    }
    if !win.is_null() {
        (*aco).use_aucmd_win_idx = -1 as ::core::ffi::c_int;
        curwin = win;
    } else {
        (*aco).use_aucmd_win_idx = auc_idx;
        (*auc_win).w_buffer = buf;
        (*auc_win).w_s = &raw mut (*buf).b_s;
        (*buf).b_nwindows += 1;
        win_init_empty(auc_win);
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*auc_win).w_localdir
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        (*aco).tp_localdir = (*curtab).tp_localdir;
        (*curtab).tp_localdir = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*aco).globaldir = globaldir;
        globaldir = ::core::ptr::null_mut::<::core::ffi::c_char>();
        block_autocmds();
        if need_append {
            win_append(lastwin, auc_win, ::core::ptr::null_mut::<tabpage_T>());
            map_put_int_ptr_t(
                &raw mut window_handles,
                (*auc_win).handle as ::core::ffi::c_int,
                auc_win as ptr_t,
            );
            win_config_float(auc_win, (*auc_win).w_config);
        }
        let save_acd: ::core::ffi::c_int = p_acd;
        p_acd = false_0;
        RedrawingDisabled += 1;
        win_enter(auc_win, false_0 != 0);
        RedrawingDisabled -= 1;
        p_acd = save_acd;
        unblock_autocmds();
        curwin = auc_win;
    }
    curbuf = buf;
    (*aco).new_curwin_handle = (*curwin).handle;
    set_bufref(&raw mut (*aco).new_curbuf, curbuf);
    (*aco).save_VIsual_active = VIsual_active;
    if !same_buffer {
        VIsual_active = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_restbuf(mut aco: *mut aco_save_T) {
    if (*aco).use_aucmd_win_idx >= 0 as ::core::ffi::c_int {
        let mut awp: *mut win_T = (*aucmd_win_vec
            .items
            .offset((*aco).use_aucmd_win_idx as isize))
            .auc_win;
        block_autocmds();
        '_win_found: {
            if curwin != awp {
                let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
                loop {
                    if tp.is_null() {
                        break '_win_found;
                    }
                    let mut wp: *mut win_T = if tp == curtab {
                        firstwin
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !wp.is_null() {
                        if wp == awp {
                            if tp != curtab {
                                goto_tabpage_tp(
                                    tp as *mut tabpage_T,
                                    true_0 != 0,
                                    true_0 != 0,
                                );
                            }
                            win_goto(awp);
                            break '_win_found;
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
        }
        (*curbuf).b_nwindows -= 1;
        win_remove(curwin, ::core::ptr::null_mut::<tabpage_T>());
        map_del_int_ptr_t(
            &raw mut window_handles,
            (*curwin).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if !(*curwin).w_grid_alloc.chars.is_null() {
            ui_comp_remove_grid(&raw mut (*curwin).w_grid_alloc);
            ui_call_win_hide((*curwin).w_grid_alloc.handle as Integer);
            grid_free(&raw mut (*curwin).w_grid_alloc);
        }
        (*aucmd_win_vec.items.offset((*aco).use_aucmd_win_idx as isize)).auc_win_used = false_0
            != 0;
        if valid_tabpage_win(curtab) == 0 {
            close_tabpage(curtab);
        }
        unblock_autocmds();
        let save_curwin: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
        if !save_curwin.is_null() {
            curwin = save_curwin;
        } else {
            curwin = firstwin;
        }
        curbuf = (*curwin).w_buffer;
        entering_window(curwin);
        if bt_prompt(curbuf) {
            (*curbuf).b_prompt_insert = (*aco).save_prompt_insert;
        }
        prevwin = win_find_by_handle((*aco).save_prevwin_handle);
        vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab);
        hash_init(&raw mut (*(*awp).w_vars).dv_hashtab);
        if !(*awp).w_localdir.is_null() {
            win_fix_current_dir();
        }
        xfree((*curtab).tp_localdir as *mut ::core::ffi::c_void);
        (*curtab).tp_localdir = (*aco).tp_localdir;
        xfree(globaldir as *mut ::core::ffi::c_void);
        globaldir = (*aco).globaldir;
        VIsual_active = (*aco).save_VIsual_active;
        check_cursor(curwin);
        if (*curwin).w_topline > (*curbuf).b_ml.ml_line_count {
            (*curwin).w_topline = (*curbuf).b_ml.ml_line_count;
            (*curwin).w_topfill = 0 as ::core::ffi::c_int;
        }
    } else {
        let save_curwin_0: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
        if !save_curwin_0.is_null() {
            if (*curwin).handle == (*aco).new_curwin_handle
                && curbuf != (*aco).new_curbuf.br_buf
                && bufref_valid(&raw mut (*aco).new_curbuf) as ::core::ffi::c_int != 0
                && !(*(*aco).new_curbuf.br_buf).b_ml.ml_mfp.is_null()
            {
                if (*curwin).w_s == &raw mut (*curbuf).b_s {
                    (*curwin).w_s = &raw mut (*(*aco).new_curbuf.br_buf).b_s;
                }
                (*curbuf).b_nwindows -= 1;
                curbuf = (*aco).new_curbuf.br_buf;
                (*curwin).w_buffer = curbuf;
                (*curbuf).b_nwindows += 1;
            }
            curwin = save_curwin_0;
            curbuf = (*curwin).w_buffer;
            prevwin = win_find_by_handle((*aco).save_prevwin_handle);
            VIsual_active = (*aco).save_VIsual_active;
            check_cursor(curwin);
        }
    }
    VIsual_active = (*aco).save_VIsual_active;
    check_cursor(curwin);
    if VIsual_active {
        check_pos(curbuf, &raw mut VIsual);
    }
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_defer(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) {
    let mut evdata: *mut AutoCmdEvent = xmalloc(::core::mem::size_of::<AutoCmdEvent>())
        as *mut AutoCmdEvent;
    (*evdata).event = event;
    (*evdata).fname = if !fname.is_null() {
        xstrdup(fname)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*evdata).fname_io = if !fname_io.is_null() {
        xstrdup(fname_io)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*evdata).group = group;
    (*evdata).buf = (*buf).handle as Buffer;
    (*evdata).eap = eap;
    if !data.is_null() {
        (*evdata).data = xmalloc(::core::mem::size_of::<Object>()) as *mut Object;
        *(*evdata).data = copy_object(*data, ::core::ptr::null_mut::<Arena>());
    } else {
        (*evdata).data = ::core::ptr::null_mut::<Object>();
    }
    multiqueue_put_event(
        deferred_events,
        Event {
            handler: Some(
                deferred_event
                    as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                evdata as *mut ::core::ffi::c_void,
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
unsafe extern "C" fn deferred_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut e: *mut AutoCmdEvent = *argv.offset(0 as ::core::ffi::c_int as isize)
        as *mut AutoCmdEvent;
    let mut event: event_T = (*e).event;
    let mut fname: *mut ::core::ffi::c_char = (*e).fname;
    let mut fname_io: *mut ::core::ffi::c_char = (*e).fname_io;
    let mut group: ::core::ffi::c_int = (*e).group;
    let mut eap: *mut exarg_T = (*e).eap;
    let mut data: *mut Object = (*e).data;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut buf: *mut buf_T = find_buffer_by_handle((*e).buf, &raw mut err);
    if !buf.is_null() {
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
        let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
        if !data.is_null()
            && (*data).type_0 as ::core::ffi::c_uint
                == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut i: size_t = 0 as size_t;
            while i < (*data).data.dict.size {
                let mut item: KeyValuePair = *(*data).data.dict.items.offset(i as isize);
                let mut tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                object_to_vim(item.value, &raw mut tv, &raw mut err);
                if err.type_0 as ::core::ffi::c_int
                    != kErrorTypeNone as ::core::ffi::c_int
                {
                    api_clear_error(&raw mut err);
                } else {
                    tv_dict_add_tv(v_event, item.key.data, item.key.size, &raw mut tv);
                    tv_clear(&raw mut tv);
                }
                i = i.wrapping_add(1);
            }
        }
        tv_dict_set_keys_readonly(v_event);
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
        apply_autocmds_group(
            event,
            fname,
            fname_io,
            false_0 != 0,
            group,
            buf,
            eap,
            data,
        );
        aucmd_restbuf(&raw mut aco);
        restore_v_event(v_event, &raw mut save_v_event);
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(fname_io as *mut ::core::ffi::c_void);
    if !data.is_null() {
        api_free_object(*data);
        xfree(data as *mut ::core::ffi::c_void);
    }
    xfree(e as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn apply_autocmds(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
) -> bool {
    return apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        ::core::ptr::null_mut::<Object>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn apply_autocmds_exarg(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
) -> bool {
    return apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        eap,
        ::core::ptr::null_mut::<Object>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn apply_autocmds_retval(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut retval: *mut ::core::ffi::c_int,
) -> bool {
    if should_abort(*retval) {
        return false_0 != 0;
    }
    let mut did_cmd: bool = apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        ::core::ptr::null_mut::<Object>(),
    );
    if did_cmd as ::core::ffi::c_int != 0 && aborting() as ::core::ffi::c_int != 0 {
        *retval = FAIL;
    }
    return did_cmd;
}
#[no_mangle]
pub unsafe extern "C" fn has_event(mut event: event_T) -> bool {
    return autocmds[event as ::core::ffi::c_int as usize].size != 0 as size_t;
}
unsafe extern "C" fn has_cursorhold() -> bool {
    return has_event(
        (if get_real_state() == MODE_NORMAL_BUSY as ::core::ffi::c_int {
            EVENT_CURSORHOLD as ::core::ffi::c_int
        } else {
            EVENT_CURSORHOLDI as ::core::ffi::c_int
        }) as event_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn trigger_cursorhold() -> bool {
    if !did_cursorhold && has_cursorhold() as ::core::ffi::c_int != 0
        && reg_recording == 0 as ::core::ffi::c_int
        && typebuf.tb_len == 0 as ::core::ffi::c_int && !ins_compl_active()
    {
        let mut state: ::core::ffi::c_int = get_real_state();
        if state == MODE_NORMAL_BUSY as ::core::ffi::c_int
            || state & MODE_INSERT as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn apply_autocmds_group(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) -> bool {
    let mut win_ignore: bool = false;
    let mut save_autocmd_fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut save_autocmd_fname_full: bool = false;
    let mut save_autocmd_bufnr: ::core::ffi::c_int = 0;
    let mut save_autocmd_match: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut save_autocmd_busy: ::core::ffi::c_int = 0;
    let mut save_autocmd_nested: ::core::ffi::c_int = 0;
    let mut save_changed: bool = false;
    let mut old_curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut afile_orig: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut patcmd: AutoPatCmd = AutoPatCmd {
        lastpat: ::core::ptr::null_mut::<AutoPat>(),
        auidx: 0,
        ausize: 0,
        afile_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tail: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        group: 0,
        event: EVENT_BUFADD,
        script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
        arg_bufnr: 0,
        data: ::core::ptr::null_mut::<Object>(),
        next: ::core::ptr::null_mut::<AutoPatCmd>(),
    };
    let mut sfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut retval: bool = false_0 != 0;
    static mut nesting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_cmdarg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    static mut filechangeshell_busy: bool = false_0 != 0;
    let mut wait_time: proftime_T = 0;
    let mut did_save_redobuff: bool = false_0 != 0;
    let mut save_redo: save_redo_T = save_redo_T {
        sr_redobuff: buffheader_T {
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
        sr_old_redobuff: buffheader_T {
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
    };
    let save_KeyTyped: bool = KeyTyped;
    if !(event as ::core::ffi::c_uint
        == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
        || autocmds[event as ::core::ffi::c_int as usize].size == 0 as size_t
        || is_autocmd_blocked() as ::core::ffi::c_int != 0)
    {
        if !(autocmd_busy as ::core::ffi::c_int != 0
            && !(force as ::core::ffi::c_int != 0
                || autocmd_nested as ::core::ffi::c_int != 0))
        {
            if !aborting() {
                if !(filechangeshell_busy as ::core::ffi::c_int != 0
                    && (event as ::core::ffi::c_uint
                        == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                        || event as ::core::ffi::c_uint
                            == EVENT_FILECHANGEDSHELLPOST as ::core::ffi::c_int
                                as ::core::ffi::c_uint))
                {
                    if !event_ignored(event, p_ei) {
                        win_ignore = false_0 != 0;
                        if buf == curbuf
                            && event_names[event as usize].event
                                <= 0 as ::core::ffi::c_int
                        {
                            win_ignore = event_ignored(
                                event,
                                (*curwin).w_onebuf_opt.wo_eiw,
                            );
                        } else if !buf.is_null()
                            && event_names[event as usize].event
                                <= 0 as ::core::ffi::c_int
                            && (*buf).b_nwindows > 0 as ::core::ffi::c_int
                        {
                            win_ignore = true_0 != 0;
                            let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
                            while !tp.is_null() {
                                let mut wp: *mut win_T = if tp == curtab {
                                    firstwin
                                } else {
                                    (*tp).tp_firstwin
                                };
                                while !wp.is_null() {
                                    if (*wp).w_buffer == buf
                                        && !event_ignored(event, (*wp).w_onebuf_opt.wo_eiw)
                                    {
                                        win_ignore = false_0 != 0;
                                        break;
                                    } else {
                                        wp = (*wp).w_next;
                                    }
                                }
                                tp = (*tp).tp_next as *mut tabpage_T;
                            }
                        }
                        if !win_ignore {
                            if nesting == 10 as ::core::ffi::c_int {
                                emsg(
                                    gettext(
                                        &raw const e_autocommand_nesting_too_deep
                                            as *const ::core::ffi::c_char,
                                    ),
                                );
                            } else if !(autocmd_no_enter != 0
                                && (event as ::core::ffi::c_uint
                                    == EVENT_WINENTER as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                    || event as ::core::ffi::c_uint
                                        == EVENT_BUFENTER as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                                || autocmd_no_leave != 0
                                    && (event as ::core::ffi::c_uint
                                        == EVENT_WINLEAVE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_BUFLEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint))
                            {
                                save_autocmd_fname = autocmd_fname;
                                save_autocmd_fname_full = autocmd_fname_full;
                                save_autocmd_bufnr = autocmd_bufnr;
                                save_autocmd_match = autocmd_match;
                                save_autocmd_busy = autocmd_busy as ::core::ffi::c_int;
                                save_autocmd_nested = autocmd_nested as ::core::ffi::c_int;
                                save_changed = (*curbuf).b_changed != 0;
                                old_curbuf = curbuf;
                                if fname_io.is_null() {
                                    if event as ::core::ffi::c_uint
                                        == EVENT_COLORSCHEME as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_OPTIONSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MODECHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MARKSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        autocmd_fname = ::core::ptr::null_mut::<
                                            ::core::ffi::c_char,
                                        >();
                                    } else if !fname.is_null()
                                        && ends_excmd(*fname as ::core::ffi::c_int) == 0
                                    {
                                        autocmd_fname = fname;
                                    } else if !buf.is_null() {
                                        autocmd_fname = (*buf).b_ffname;
                                    } else {
                                        autocmd_fname = ::core::ptr::null_mut::<
                                            ::core::ffi::c_char,
                                        >();
                                    }
                                } else {
                                    autocmd_fname = fname_io;
                                }
                                afile_orig = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if !autocmd_fname.is_null() {
                                    afile_orig = xstrdup(autocmd_fname);
                                    autocmd_fname = xstrnsave(
                                        autocmd_fname,
                                        MAXPATHL as size_t,
                                    );
                                }
                                autocmd_fname_full = false_0 != 0;
                                autocmd_bufnr = if buf.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*buf).handle as ::core::ffi::c_int
                                };
                                if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
                                    if buf.is_null() {
                                        fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    } else if event as ::core::ffi::c_uint
                                        == EVENT_SYNTAX as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        fname = (*buf).b_p_syn;
                                    } else if event as ::core::ffi::c_uint
                                        == EVENT_FILETYPE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        fname = (*buf).b_p_ft;
                                    } else {
                                        if !(*buf).b_sfname.is_null() {
                                            sfname = xstrdup((*buf).b_sfname);
                                        }
                                        fname = (*buf).b_ffname;
                                    }
                                    if fname.is_null() {
                                        fname = b"\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char;
                                    }
                                    fname = xstrdup(fname);
                                } else {
                                    sfname = xstrdup(fname);
                                    if event as ::core::ffi::c_uint
                                        == EVENT_CMDLINECHANGED as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINEENTER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINELEAVEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINELEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDUNDEFINED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CURSORMOVEDC as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDWINENTER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDWINLEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEME as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_DIRCHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_FILETYPE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_FUNCUNDEFINED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MARKSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MENUPOPUP as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MODECHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_OPTIONSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_PROGRESS as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_QUICKFIXCMDPOST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_QUICKFIXCMDPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_REMOTEREPLY as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SIGNAL as ::core::ffi::c_int as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SPELLFILEMISSING as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SYNTAX as ::core::ffi::c_int as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_TABCLOSED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_USER as ::core::ffi::c_int as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINCLOSED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINRESIZED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINSCROLLED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        fname = xstrdup(fname);
                                        autocmd_fname_full = true_0 != 0;
                                    } else {
                                        fname = FullName_save(fname, false_0 != 0);
                                    }
                                }
                                if fname.is_null() {
                                    xfree(sfname as *mut ::core::ffi::c_void);
                                    retval = false_0 != 0;
                                } else {
                                    autocmd_match = fname;
                                    RedrawingDisabled += 1;
                                    estack_push(
                                        ETYPE_AUCMD,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        0 as linenr_T,
                                    );
                                    save_current_sctx = current_sctx;
                                    if do_profiling == PROF_YES {
                                        prof_child_enter(&raw mut wait_time);
                                    }
                                    funccal_entry = funccal_entry_T {
                                        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                                        next: ::core::ptr::null_mut::<funccal_entry_T>(),
                                    };
                                    save_funccal(&raw mut funccal_entry);
                                    if !autocmd_busy {
                                        save_search_patterns();
                                        if !ins_compl_active() {
                                            saveRedobuff(&raw mut save_redo);
                                            did_save_redobuff = true_0 != 0;
                                        }
                                        (*curbuf).b_did_filetype = (*curbuf).b_keep_filetype;
                                    }
                                    autocmd_busy = true_0 != 0;
                                    filechangeshell_busy = event as ::core::ffi::c_uint
                                        == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int
                                            as ::core::ffi::c_uint;
                                    nesting += 1;
                                    if event as ::core::ffi::c_uint
                                        == EVENT_FILETYPE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*curbuf).b_did_filetype = true_0 != 0;
                                    }
                                    tail = path_tail(fname);
                                    patcmd = AutoPatCmd_S {
                                        lastpat: ::core::ptr::null_mut::<AutoPat>(),
                                        auidx: 0 as size_t,
                                        ausize: autocmds[event as ::core::ffi::c_int as usize].size,
                                        afile_orig: afile_orig,
                                        fname: fname,
                                        sfname: sfname,
                                        tail: tail,
                                        group: group,
                                        event: event,
                                        script_ctx: sctx_T {
                                            sc_sid: 0,
                                            sc_seq: 0,
                                            sc_lnum: 0,
                                            sc_chan: 0,
                                        },
                                        arg_bufnr: autocmd_bufnr,
                                        data: ::core::ptr::null_mut::<Object>(),
                                        next: ::core::ptr::null_mut::<AutoPatCmd>(),
                                    };
                                    aucmd_next(&raw mut patcmd);
                                    if !patcmd.lastpat.is_null() {
                                        patcmd.next = active_apc_list;
                                        active_apc_list = &raw mut patcmd;
                                        patcmd.data = data;
                                        let mut save_cmdbang: varnumber_T = get_vim_var_nr(
                                            VV_CMDBANG,
                                        );
                                        if !eap.is_null() {
                                            save_cmdarg = set_cmdarg(
                                                eap,
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            );
                                            set_vim_var_nr(VV_CMDBANG, (*eap).forceit as varnumber_T);
                                        } else {
                                            save_cmdarg = ::core::ptr::null_mut::<
                                                ::core::ffi::c_char,
                                            >();
                                        }
                                        retval = true_0 != 0;
                                        if nesting == 1 as ::core::ffi::c_int {
                                            check_lnums(true_0 != 0);
                                        } else {
                                            check_lnums_nested(true_0 != 0);
                                        }
                                        let save_did_emsg: ::core::ffi::c_int = did_emsg;
                                        let save_ex_pressedreturn: bool = get_pressedreturn();
                                        do_cmdline(
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            Some(
                                                getnextac
                                                    as unsafe extern "C" fn(
                                                        ::core::ffi::c_int,
                                                        *mut ::core::ffi::c_void,
                                                        ::core::ffi::c_int,
                                                        bool,
                                                    ) -> *mut ::core::ffi::c_char,
                                            ),
                                            &raw mut patcmd as *mut ::core::ffi::c_void,
                                            DOCMD_NOWAIT as ::core::ffi::c_int
                                                | DOCMD_VERBOSE as ::core::ffi::c_int
                                                | DOCMD_REPEAT as ::core::ffi::c_int,
                                        );
                                        did_emsg += save_did_emsg;
                                        set_pressedreturn(save_ex_pressedreturn);
                                        if nesting == 1 as ::core::ffi::c_int {
                                            reset_lnums();
                                        }
                                        if !eap.is_null() {
                                            set_cmdarg(::core::ptr::null_mut::<exarg_T>(), save_cmdarg);
                                            set_vim_var_nr(VV_CMDBANG, save_cmdbang);
                                        }
                                        if active_apc_list == &raw mut patcmd {
                                            active_apc_list = patcmd.next;
                                        }
                                    }
                                    RedrawingDisabled -= 1;
                                    autocmd_busy = save_autocmd_busy != 0;
                                    filechangeshell_busy = false_0 != 0;
                                    autocmd_nested = save_autocmd_nested != 0;
                                    xfree(
                                        (*(exestack.ga_data as *mut estack_T)
                                            .offset(
                                                (exestack.ga_len - 1 as ::core::ffi::c_int) as isize,
                                            ))
                                            .es_name as *mut ::core::ffi::c_void,
                                    );
                                    estack_pop();
                                    xfree(afile_orig as *mut ::core::ffi::c_void);
                                    xfree(autocmd_fname as *mut ::core::ffi::c_void);
                                    autocmd_fname = save_autocmd_fname;
                                    autocmd_fname_full = save_autocmd_fname_full;
                                    autocmd_bufnr = save_autocmd_bufnr;
                                    autocmd_match = save_autocmd_match;
                                    current_sctx = save_current_sctx;
                                    restore_funccal();
                                    if do_profiling == PROF_YES {
                                        prof_child_exit(&raw mut wait_time);
                                    }
                                    KeyTyped = save_KeyTyped;
                                    xfree(fname as *mut ::core::ffi::c_void);
                                    xfree(sfname as *mut ::core::ffi::c_void);
                                    nesting -= 1;
                                    if !autocmd_busy {
                                        restore_search_patterns();
                                        if did_save_redobuff {
                                            restoreRedobuff(&raw mut save_redo);
                                        }
                                        (*curbuf).b_did_filetype = false_0 != 0;
                                        while !au_pending_free_buf.is_null() {
                                            let mut b: *mut buf_T = (*au_pending_free_buf).b_next;
                                            xfree(au_pending_free_buf as *mut ::core::ffi::c_void);
                                            au_pending_free_buf = b;
                                        }
                                        while !au_pending_free_win.is_null() {
                                            let mut w: *mut win_T = (*au_pending_free_win).w_next;
                                            xfree(au_pending_free_win as *mut ::core::ffi::c_void);
                                            au_pending_free_win = w;
                                        }
                                    }
                                    if curbuf == old_curbuf
                                        && (event as ::core::ffi::c_uint
                                            == EVENT_BUFREADPOST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_BUFWRITEPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_FILEAPPENDPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_VIMLEAVE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_VIMLEAVEPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                    {
                                        if (*curbuf).b_changed != save_changed as ::core::ffi::c_int
                                        {
                                            need_maketitle = true_0 != 0;
                                        }
                                        (*curbuf).b_changed = save_changed as ::core::ffi::c_int;
                                    }
                                    au_cleanup();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if event as ::core::ffi::c_uint
        == EVENT_BUFWIPEOUT as ::core::ffi::c_int as ::core::ffi::c_uint
        && !buf.is_null()
    {
        aubuflocal_remove(buf);
    }
    if retval as ::core::ffi::c_int == OK
        && event as ::core::ffi::c_uint
            == EVENT_FILETYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*curbuf).b_au_did_filetype = true_0 != 0;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn do_termresponse_autocmd(sequence: String_0) {
    let mut data: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut data__items: [KeyValuePair; 1] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 1];
    data.capacity = 1 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let c2rust_fresh11 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"sequence\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: sequence },
        },
    };
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: data },
    };
    apply_autocmds_group(
        EVENT_TERMRESPONSE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        AUGROUP_ALL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<buf_T>(),
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
    termresponse_changed = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn block_autocmds() {
    if !is_autocmd_blocked() {
        termresponse_changed = false_0 != 0;
    }
    autocmd_blocked += 1;
}
#[no_mangle]
pub unsafe extern "C" fn unblock_autocmds() {
    autocmd_blocked -= 1;
    if !is_autocmd_blocked() && termresponse_changed as ::core::ffi::c_int != 0
        && has_event(EVENT_TERMRESPONSE) as ::core::ffi::c_int != 0
    {
        let sequence: String_0 = cstr_to_string(get_vim_var_str(VV_TERMRESPONSE));
        do_termresponse_autocmd(sequence);
        api_free_string(sequence);
    }
}
#[no_mangle]
pub unsafe extern "C" fn is_autocmd_blocked() -> bool {
    return autocmd_blocked != 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn aucmd_next(mut apc: *mut AutoPatCmd) {
    let entry: *mut estack_T = (exestack.ga_data as *mut estack_T)
        .offset(exestack.ga_len as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset((*apc).event as ::core::ffi::c_int as isize);
    '_c2rust_label: {
        if (*apc).ausize <= (*acs).size {} else {
            __assert_fail(
                b"apc->ausize <= kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2077 as ::core::ffi::c_uint,
                b"void aucmd_next(AutoPatCmd *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut i: size_t = (*apc).auidx;
    while i < (*apc).ausize && !got_int {
        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
        let ap: *mut AutoPat = (*ac).pat;
        's_11: {
            if !ap.is_null() {
                if ap != (*apc).lastpat {
                    if (*apc).group != AUGROUP_ALL as ::core::ffi::c_int
                        && (*apc).group != (*ap).group
                    {
                        break 's_11;
                    } else if if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                        !match_file_pat(
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut (*ap).reg_prog,
                            (*apc).fname,
                            (*apc).sfname,
                            (*apc).tail,
                            (*ap).allow_dirs as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                    } else {
                        ((*ap).buflocal_nr != (*apc).arg_bufnr) as ::core::ffi::c_int
                    } != 0
                    {
                        break 's_11;
                    } else {
                        let name: *const ::core::ffi::c_char = event_nr2name(
                            (*apc).event,
                        );
                        let s: *const ::core::ffi::c_char = gettext(
                            b"%s Autocommands for \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        let sourcing_name_len: size_t = strlen(s)
                            .wrapping_add(strlen(name))
                            .wrapping_add((*ap).patlen as size_t)
                            .wrapping_add(1 as size_t);
                        let namep: *mut ::core::ffi::c_char = xmalloc(sourcing_name_len)
                            as *mut ::core::ffi::c_char;
                        snprintf(namep, sourcing_name_len, s, name, (*ap).pat);
                        if p_verbose >= 8 as OptInt {
                            verbose_enter();
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(
                                    b"Executing %s\0".as_ptr() as *const ::core::ffi::c_char,
                                ),
                                namep,
                            );
                            verbose_leave();
                        }
                        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*entry)
                            .es_name as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        *ptr_;
                        (*entry).es_name = namep;
                        (*entry).es_info.aucmd = apc;
                    }
                }
                (*apc).lastpat = ap;
                (*apc).auidx = i;
                line_breakcheck();
                return;
            }
        }
        i = i.wrapping_add(1);
    }
    let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut (*entry).es_name
        as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    *ptr__0;
    (*entry).es_info.aucmd = ::core::ptr::null_mut::<AutoPatCmd>();
    (*apc).lastpat = ::core::ptr::null_mut::<AutoPat>();
    (*apc).auidx = SIZE_MAX as size_t;
}
unsafe extern "C" fn au_callback(
    mut ac: *const AutoCmd,
    mut apc: *const AutoPatCmd,
) -> bool {
    let mut callback: Callback = (*ac).handler_fn;
    if callback.type_0 as ::core::ffi::c_uint
        == kCallbackLua as ::core::ffi::c_int as ::core::ffi::c_uint
    {
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
                data: C2Rust_Unnamed { boolean: false },
            },
        }; 7];
        data.capacity = 7 as size_t;
        data.items = &raw mut data__items as *mut KeyValuePair;
        let c2rust_fresh3 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*ac).id,
                },
            },
        };
        let c2rust_fresh4 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"event\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(event_nr2name((*apc).event)),
                },
            },
        };
        let c2rust_fresh5 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*apc).afile_orig),
                },
            },
        };
        let c2rust_fresh6 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"match\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(autocmd_match),
                },
            },
        };
        let c2rust_fresh7 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: autocmd_bufnr as Integer,
                },
            },
        };
        if !(*apc).data.is_null() {
            let c2rust_fresh8 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
                value: *(*apc).data,
            };
        }
        let mut group: ::core::ffi::c_int = (*(*ac).pat).group;
        match group {
            -2 => {
                abort();
            }
            -1 | -3 | -4 => {}
            _ => {
                let c2rust_fresh9 = data.size;
                data.size = data.size.wrapping_add(1);
                *data.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                    key: cstr_as_string(
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ),
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: group as Integer,
                        },
                    },
                };
            }
        }
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh10 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh10 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: data },
        };
        let mut result: Object = nlua_call_ref(
            callback.data.luaref,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            ::core::ptr::null_mut::<Error>(),
        );
        return result.type_0 as ::core::ffi::c_uint
            == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            && result.data.boolean as ::core::ffi::c_int == true_0;
    } else {
        let mut argsin: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        callback_call(
            &raw mut callback,
            0 as ::core::ffi::c_int,
            &raw mut argsin,
            &raw mut rettv,
        );
        return false_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn getnextac(
    mut c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut indent: ::core::ffi::c_int,
    mut do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let apc: *mut AutoPatCmd = cookie as *mut AutoPatCmd;
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset((*apc).event as ::core::ffi::c_int as isize);
    aucmd_next(apc);
    if (*apc).lastpat.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    '_c2rust_label: {
        if (*apc).auidx < (*acs).size {} else {
            __assert_fail(
                b"apc->auidx < kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2193 as ::core::ffi::c_uint,
                b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let ac: *mut AutoCmd = (*acs).items.offset((*apc).auidx as isize);
    '_c2rust_label_0: {
        if !(*ac).pat.is_null() {} else {
            __assert_fail(
                b"ac->pat != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2195 as ::core::ffi::c_uint,
                b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut oneshot: bool = (*ac).once;
    if p_verbose >= 9 as OptInt {
        verbose_enter_scroll();
        let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(ac);
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"autocommand %s\0".as_ptr() as *const ::core::ffi::c_char),
            handler_str,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut handler_str
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        verbose_leave_scroll();
    }
    autocmd_nested = (*ac).nested;
    current_sctx = (*ac).script_ctx;
    (*apc).script_ctx = current_sctx;
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if !(*ac).handler_cmd.is_null() {
        retval = xstrdup((*ac).handler_cmd);
    } else {
        let mut ac_copy: AutoCmd = *ac;
        (*ac).pat = if oneshot as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<AutoPat>()
        } else {
            (*ac).pat
        };
        let mut rv: bool = au_callback(&raw mut ac_copy, apc);
        if oneshot {
            (*(*acs).items.offset((*apc).auidx as isize)).pat = ac_copy.pat;
        }
        oneshot = oneshot as ::core::ffi::c_int != 0 || rv as ::core::ffi::c_int != 0;
        retval = xcalloc(1 as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
    }
    if oneshot {
        aucmd_del((*acs).items.offset((*apc).auidx as isize));
    }
    if (*apc).auidx < (*apc).ausize {
        (*apc).auidx = (*apc).auidx.wrapping_add(1);
    } else {
        (*apc).auidx = SIZE_MAX as size_t;
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn has_autocmd(
    mut event: event_T,
    mut sfname: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) -> bool {
    let mut tail: *mut ::core::ffi::c_char = path_tail(sfname);
    let mut retval: bool = false_0 != 0;
    let mut fname: *mut ::core::ffi::c_char = FullName_save(sfname, false_0 != 0);
    if fname.is_null() {
        return false_0 != 0;
    }
    let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
        .offset(event as ::core::ffi::c_int as isize);
    let mut i: size_t = 0 as size_t;
    while i < (*acs).size {
        let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
        if !ap.is_null()
            && (if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                match_file_pat(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut (*ap).reg_prog,
                    fname,
                    sfname,
                    tail,
                    (*ap).allow_dirs as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
            } else {
                (!buf.is_null() && (*ap).buflocal_nr == (*buf).handle)
                    as ::core::ffi::c_int
            }) != 0
        {
            retval = true_0 != 0;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn expand_get_augroup_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return augroup_name(idx + 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_autocmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
    mut doautocmd: bool,
) -> *mut ::core::ffi::c_char {
    autocmd_include_groups = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = arg;
    let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
    if *arg as ::core::ffi::c_int == NUL && group != AUGROUP_ALL as ::core::ffi::c_int
        && !ascii_iswhite(
            *arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        )
    {
        arg = p;
        group = AUGROUP_ALL as ::core::ffi::c_int;
    }
    p = arg;
    while *p as ::core::ffi::c_int != NUL && !ascii_iswhite(*p as ::core::ffi::c_int) {
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            arg = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        if group == AUGROUP_ALL as ::core::ffi::c_int {
            autocmd_include_groups = true_0 != 0;
        }
        (*xp).xp_context = EXPAND_EVENTS as ::core::ffi::c_int;
        (*xp).xp_pattern = arg;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    arg = skipwhite(p);
    while *arg as ::core::ffi::c_int != 0
        && (!ascii_iswhite(*arg as ::core::ffi::c_int)
            || *arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int)
    {
        arg = arg.offset(1);
    }
    if *arg != 0 {
        return arg;
    }
    if doautocmd {
        (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_get_event_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = augroup_name(idx + 1 as ::core::ffi::c_int);
    if !name.is_null() {
        if !autocmd_include_groups
            || name == get_deleted_augroup() as *mut ::core::ffi::c_char
        {
            return b"\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        return name;
    }
    let mut i: ::core::ffi::c_int = idx - next_augroup_id;
    if i < 0 as ::core::ffi::c_int || i >= NUM_EVENTS as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return event_names[i as usize].name;
}
#[no_mangle]
pub unsafe extern "C" fn get_event_name_no_group(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut win: bool,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int || idx >= NUM_EVENTS as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !win {
        return event_names[idx as usize].name;
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NUM_EVENTS as ::core::ffi::c_int {
        j
            += (event_names[i as usize].event <= 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
        if j == idx + 1 as ::core::ffi::c_int {
            return event_names[i as usize].name;
        }
        i += 1;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn autocmd_supported(event: *const ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return event_name2nr(event, &raw mut p) as ::core::ffi::c_uint
        != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn au_exists(arg: *const ::core::ffi::c_char) -> bool {
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut event: event_T = EVENT_BUFADD;
    let mut acs: *mut AutoCmdVec = ::core::ptr::null_mut::<AutoCmdVec>();
    let mut buflocal_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut retval: bool = false_0 != 0;
    let arg_save: *mut ::core::ffi::c_char = xstrdup(arg);
    let mut p: *mut ::core::ffi::c_char = strchr(arg_save, '#' as ::core::ffi::c_int);
    if !p.is_null() {
        let c2rust_fresh13 = p;
        p = p.offset(1);
        *c2rust_fresh13 = NUL as ::core::ffi::c_char;
    }
    let mut group: ::core::ffi::c_int = augroup_find(arg_save);
    let mut event_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    '_theend: {
        if group == AUGROUP_ERROR as ::core::ffi::c_int {
            group = AUGROUP_ALL as ::core::ffi::c_int;
            event_name = arg_save;
        } else if p.is_null() {
            retval = true_0 != 0;
            break '_theend;
        } else {
            event_name = p;
            p = strchr(event_name, '#' as ::core::ffi::c_int);
            if !p.is_null() {
                let c2rust_fresh14 = p;
                p = p.offset(1);
                *c2rust_fresh14 = NUL as ::core::ffi::c_char;
            }
        }
        pattern = p;
        event = event_name2nr(event_name, &raw mut p);
        if event as ::core::ffi::c_uint
            != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            acs = (&raw mut autocmds as *mut AutoCmdVec)
                .offset(event as ::core::ffi::c_int as isize);
            if (*acs).size != 0 as size_t {
                if !pattern.is_null()
                    && strcasecmp(
                        pattern,
                        b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    buflocal_buf = curbuf;
                }
                let mut i: size_t = 0 as size_t;
                while i < (*acs).size {
                    let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                    if !ap.is_null()
                        && (group == AUGROUP_ALL as ::core::ffi::c_int
                            || (*ap).group == group)
                        && (pattern.is_null()
                            || (if buflocal_buf.is_null() {
                                (path_fnamecmp((*ap).pat, pattern)
                                    == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                            } else {
                                ((*ap).buflocal_nr == (*buflocal_buf).handle)
                                    as ::core::ffi::c_int
                            }) != 0)
                    {
                        retval = true_0 != 0;
                        break;
                    } else {
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }
    xfree(arg_save as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn aupat_is_buflocal(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> bool {
    return patlen >= 8 as ::core::ffi::c_int
        && strncmp(pat, b"<buffer\0".as_ptr() as *const ::core::ffi::c_char, 7 as size_t)
            == 0 as ::core::ffi::c_int
        && *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '>' as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn aupat_get_buflocal_nr(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if aupat_is_buflocal(pat, patlen) {} else {
            __assert_fail(
                b"aupat_is_buflocal(pat, patlen)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2514 as ::core::ffi::c_uint,
                b"int aupat_get_buflocal_nr(const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if patlen == 8 as ::core::ffi::c_int {
        return (*curbuf).handle as ::core::ffi::c_int;
    }
    if patlen > 9 as ::core::ffi::c_int
        && *pat.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        if patlen == 13 as ::core::ffi::c_int
            && strncasecmp(
                pat as *mut ::core::ffi::c_char,
                b"<buffer=abuf>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                13 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            return autocmd_bufnr;
        }
        if skipdigits(pat.offset(8 as ::core::ffi::c_int as isize))
            == pat.offset(patlen as isize).offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char
        {
            return atoi(pat.offset(8 as ::core::ffi::c_int as isize));
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn aupat_normalize_buflocal_pat(
    mut dest: *mut ::core::ffi::c_char,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut buflocal_nr: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if aupat_is_buflocal(pat, patlen) {} else {
            __assert_fail(
                b"aupat_is_buflocal(pat, patlen)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2539 as ::core::ffi::c_uint,
                b"void aupat_normalize_buflocal_pat(char *, const char *, int, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if buflocal_nr == 0 as ::core::ffi::c_int {
        buflocal_nr = (*curbuf).handle as ::core::ffi::c_int;
    }
    snprintf(
        dest,
        BUFLOCAL_PAT_LEN as ::core::ffi::c_int as size_t,
        b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
        buflocal_nr,
    );
}
#[no_mangle]
pub unsafe extern "C" fn autocmd_delete_event(
    mut group: ::core::ffi::c_int,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return do_autocmd_event(
        event,
        pat,
        false_0 != 0,
        false_0,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
        group,
    );
}
#[no_mangle]
pub unsafe extern "C" fn autocmd_delete_id(mut id: int64_t) -> bool {
    '_c2rust_label: {
        if id > 0 as int64_t {} else {
            __assert_fail(
                b"id > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2560 as ::core::ffi::c_uint,
                b"_Bool autocmd_delete_id(int64_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut success: bool = false_0 != 0;
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec = (&raw mut autocmds as *mut AutoCmdVec)
            .offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if (*ac).id == id {
                aucmd_del(ac);
                success = true_0 != 0;
            }
            i = i.wrapping_add(1);
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    return success;
}
#[no_mangle]
pub unsafe extern "C" fn aucmd_handler_to_string(
    mut ac: *mut AutoCmd,
) -> *mut ::core::ffi::c_char {
    if !(*ac).handler_cmd.is_null() {
        return xstrdup((*ac).handler_cmd);
    }
    return callback_to_string(
        &raw mut (*ac).handler_fn,
        ::core::ptr::null_mut::<Arena>(),
    );
}
unsafe extern "C" fn arg_event_skip(
    mut arg: *mut ::core::ffi::c_char,
    mut have_group: bool,
) -> *mut ::core::ffi::c_char {
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && !ascii_iswhite(
                *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
        {
            semsg(
                gettext(
                    b"E215: Illegal character after *: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                arg,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        pat = arg.offset(1 as ::core::ffi::c_int as isize);
    } else {
        pat = arg;
        while *pat as ::core::ffi::c_int != 0
            && *pat as ::core::ffi::c_int != '|' as ::core::ffi::c_int
            && !ascii_iswhite(*pat as ::core::ffi::c_int)
        {
            if event_name2nr(pat, &raw mut p) as ::core::ffi::c_int
                >= NUM_EVENTS as ::core::ffi::c_int
            {
                if have_group {
                    semsg(
                        gettext(
                            b"E216: No such event: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        pat,
                    );
                } else {
                    semsg(
                        gettext(
                            b"E216: No such group or event: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        pat,
                    );
                }
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            pat = p;
        }
    }
    return pat;
}
unsafe extern "C" fn arg_augroup_get(
    mut argp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = *argp;
    p = arg;
    while *p as ::core::ffi::c_int != 0 && !ascii_iswhite(*p as ::core::ffi::c_int)
        && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if p <= arg {
        return AUGROUP_ALL as ::core::ffi::c_int;
    }
    let mut group_name: *mut ::core::ffi::c_char = xmemdupz(
        arg as *const ::core::ffi::c_void,
        p.offset_from(arg) as size_t,
    ) as *mut ::core::ffi::c_char;
    let mut group: ::core::ffi::c_int = augroup_find(group_name);
    if group == AUGROUP_ERROR as ::core::ffi::c_int {
        group = AUGROUP_ALL as ::core::ffi::c_int;
    } else {
        *argp = skipwhite(p);
    }
    xfree(group_name as *mut ::core::ffi::c_void);
    return group;
}
unsafe extern "C" fn arg_autocmd_flag_get(
    mut flag: *mut bool,
    mut cmd_ptr: *mut *mut ::core::ffi::c_char,
    mut pattern: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    if strncmp(*cmd_ptr, pattern, len as size_t) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*(*cmd_ptr).offset(len as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int != 0
    {
        if *flag {
            semsg(gettext(&raw const e_duparg2 as *const ::core::ffi::c_char), pattern);
            return true_0 != 0;
        }
        *flag = true_0 != 0;
        *cmd_ptr = skipwhite((*cmd_ptr).offset(len as isize));
    }
    return false_0 != 0;
}
static mut pending_vimresume: TriState = kFalse;
unsafe extern "C" fn vimresume_event(mut argv: *mut *mut ::core::ffi::c_void) {
    apply_autocmds(
        EVENT_VIMRESUME,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    pending_vimresume = kFalse;
}
#[no_mangle]
pub unsafe extern "C" fn may_trigger_vim_suspend_resume(mut suspend: bool) {
    if suspend as ::core::ffi::c_int != 0
        && pending_vimresume as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
    {
        pending_vimresume = kNone;
        apply_autocmds(
            EVENT_VIMSUSPEND,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        pending_vimresume = kTrue;
    } else if !suspend
        && pending_vimresume as ::core::ffi::c_int == kTrue as ::core::ffi::c_int
    {
        pending_vimresume = kNone;
        multiqueue_put_event(
            main_loop.events,
            Event {
                handler: Some(
                    vimresume_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_uienter(mut chanid: uint64_t, mut attached: bool) {
    static mut recursive: bool = false_0 != 0;
    if starting == NO_SCREEN {
        return;
    }
    if recursive {
        return;
    }
    recursive = true_0 != 0;
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
    '_c2rust_label: {
        if chanid < 9223372036854775807 as uint64_t {} else {
            __assert_fail(
                b"chanid < VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/autocmd.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2697 as ::core::ffi::c_uint,
                b"void do_autocmd_uienter(uint64_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tv_dict_add_nr(
        dict,
        b"chan\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        chanid as varnumber_T,
    );
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(
        (if attached as ::core::ffi::c_int != 0 {
            EVENT_UIENTER as ::core::ffi::c_int
        } else {
            EVENT_UILEAVE as ::core::ffi::c_int
        }) as event_T,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    restore_v_event(dict, &raw mut save_v_event);
    recursive = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_focusgained(mut gained: bool) {
    static mut recursive: bool = false_0 != 0;
    static mut last_time: Timestamp = 0 as Timestamp;
    if recursive {
        return;
    }
    recursive = true_0 != 0;
    apply_autocmds(
        (if gained as ::core::ffi::c_int != 0 {
            EVENT_FOCUSGAINED as ::core::ffi::c_int
        } else {
            EVENT_FOCUSLOST as ::core::ffi::c_int
        }) as event_T,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf,
    );
    if gained as ::core::ffi::c_int != 0
        && last_time.wrapping_add(2000 as ::core::ffi::c_int as Timestamp) < os_now()
    {
        check_timestamps(true_0);
        last_time = os_now() as Timestamp;
    }
    recursive = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn do_filetype_autocmd(
    mut buf: *mut buf_T,
    mut force: bool,
) -> bool {
    static mut ft_recursive: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ft_recursive > 0 as ::core::ffi::c_int && !force {
        return false_0 != 0;
    }
    let mut secure_save: ::core::ffi::c_int = secure;
    secure = 0 as ::core::ffi::c_int;
    ft_recursive += 1;
    (*buf).b_did_filetype = true_0 != 0;
    let mut ret: bool = apply_autocmds(
        EVENT_FILETYPE,
        (*buf).b_p_ft,
        (*buf).b_fname,
        force as ::core::ffi::c_int != 0 || ft_recursive == 1 as ::core::ffi::c_int,
        buf,
    );
    ft_recursive -= 1;
    secure = secure_save;
    return ret;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
static mut event_names: [event_name; 145] = [
    event_name {
        len: 6 as size_t,
        name: b"BufAdd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFADD as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufCreate\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFADD as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufDelete\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFDELETE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufFilePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFFILEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufFilePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFFILEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufHidden\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFHIDDEN as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"BufModifiedSet\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFMODIFIEDSET as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"BufNew\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFNEW as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufNewFile\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFNEWFILE as ::core::ffi::c_int),
    },
    event_name {
        len: 7 as size_t,
        name: b"BufRead\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufReadCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufReadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufReadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufUnload\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFUNLOAD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWinEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWINENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWinLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWINLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufWipeout\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWIPEOUT as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufWrite\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWriteCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITECMD as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"BufWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWritePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"ChanInfo\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CHANINFO as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"ChanOpen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CHANOPEN as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"CmdlineChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdlineEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINEENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdlineLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINELEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CmdlineLeavePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINELEAVEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdUndefined\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDUNDEFINED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"CmdwinEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDWINENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"CmdwinLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDWINLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"ColorScheme\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COLORSCHEME as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"ColorSchemePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COLORSCHEMEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CompleteChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CompleteDone\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETEDONE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CompleteDonePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETEDONEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"CursorHold\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORHOLD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"CursorHoldI\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORHOLDI as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"CursorMoved\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVED as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"CursorMovedC\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVEDC as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"CursorMovedI\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVEDI as ::core::ffi::c_int),
    },
    event_name {
        len: 17 as size_t,
        name: b"DiagnosticChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_DIAGNOSTICCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"DiffUpdated\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_DIFFUPDATED as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"DirChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_DIRCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"DirChangedPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_DIRCHANGEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"EncodingChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_ENCODINGCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"ExitPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_EXITPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"FileAppendCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FileAppendPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileAppendPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileChangedRO\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDRO as ::core::ffi::c_int),
    },
    event_name {
        len: 16 as size_t,
        name: b"FileChangedShell\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDSHELL as ::core::ffi::c_int),
    },
    event_name {
        len: 20 as size_t,
        name: b"FileChangedShellPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDSHELLPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileEncoding\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_ENCODINGCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"FileReadCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileReadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"FileReadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"FileType\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILETYPE as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileWriteCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITECMD as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileWritePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FilterReadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FilterReadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 15 as size_t,
        name: b"FilterWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FilterWritePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"FocusGained\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_FOCUSGAINED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"FocusLost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_FOCUSLOST as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"FuncUndefined\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_FUNCUNDEFINED as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"GUIEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_GUIENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"GUIFailed\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_GUIFAILED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"InsertChange\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTCHANGE as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"InsertCharPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTCHARPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"InsertEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"InsertLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"InsertLeavePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTLEAVEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"LspAttach\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPATTACH as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"LspDetach\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPDETACH as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"LspNotify\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPNOTIFY as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"LspProgress\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPPROGRESS as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"LspRequest\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPREQUEST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"LspTokenUpdate\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPTOKENUPDATE as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"MarkSet\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_MARKSET as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"MenuPopup\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_MENUPOPUP as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"ModeChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_MODECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"OptionSet\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_OPTIONSET as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"PackChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_PACKCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"PackChangedPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_PACKCHANGEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"Progress\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_PROGRESS as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"QuickFixCmdPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_QUICKFIXCMDPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"QuickFixCmdPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_QUICKFIXCMDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"QuitPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_QUITPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"RecordingEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_RECORDINGENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"RecordingLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_RECORDINGLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"RemoteReply\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_REMOTEREPLY as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SafeState\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SAFESTATE as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"SearchWrapped\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_SEARCHWRAPPED as ::core::ffi::c_int),
    },
    event_name {
        len: 15 as size_t,
        name: b"SessionLoadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONLOADPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"SessionLoadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONLOADPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 16 as size_t,
        name: b"SessionWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONWRITEPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"ShellCmdPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SHELLCMDPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"ShellFilterPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_SHELLFILTERPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"Signal\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SIGNAL as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SourceCmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SOURCECMD as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"SourcePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SOURCEPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SourcePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SOURCEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 16 as size_t,
        name: b"SpellFileMissing\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SPELLFILEMISSING as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"StdinReadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_STDINREADPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"StdinReadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_STDINREADPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"SwapExists\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SWAPEXISTS as ::core::ffi::c_int,
    },
    event_name {
        len: 6 as size_t,
        name: b"Syntax\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SYNTAX as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TabClosed\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABCLOSED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"TabClosedPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABCLOSEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TabEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TabLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 6 as size_t,
        name: b"TabNew\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABNEW as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"TabNewEntered\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TABNEWENTERED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TermChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermClose\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMCLOSE as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TermOpen\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMOPEN as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TermRequest\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMREQUEST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"TermResponse\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_TERMRESPONSE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TextChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGED as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedI\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDI as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedP\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDP as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedT\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDT as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextYankPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTYANKPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 7 as size_t,
        name: b"UIEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_UIENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"UILeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_UILEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 4 as size_t,
        name: b"User\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_USER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"VimEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"VimLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"VimLeavePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMLEAVEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"VimResized\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMRESIZED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"VimResume\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMRESUME as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"VimSuspend\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_VIMSUSPEND as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"WinClosed\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_WINCLOSED as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"WinEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_WINENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"WinLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_WINLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"WinNew\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_WINNEW as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"WinNewPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_WINNEWPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"WinResized\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_WINRESIZED as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"WinScrolled\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_WINSCROLLED as ::core::ffi::c_int),
    },
];
static mut autocmds: [AutoCmdVec; 145] = [
    AutoCmdVec {
        size: 0 as size_t,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
];
static mut event_hash: [event_T; 145] = [
    EVENT_USER,
    EVENT_BUFADD,
    EVENT_BUFNEW,
    EVENT_SIGNAL,
    EVENT_SYNTAX,
    EVENT_TABNEW,
    EVENT_WINNEW,
    EVENT_BUFREAD,
    EVENT_EXITPRE,
    EVENT_MARKSET,
    EVENT_QUITPRE,
    EVENT_UIENTER,
    EVENT_UILEAVE,
    EVENT_BUFENTER,
    EVENT_BUFLEAVE,
    EVENT_BUFWRITE,
    EVENT_CHANINFO,
    EVENT_CHANOPEN,
    EVENT_FILETYPE,
    EVENT_GUIENTER,
    EVENT_PROGRESS,
    EVENT_TABENTER,
    EVENT_TABLEAVE,
    EVENT_TERMOPEN,
    EVENT_VIMENTER,
    EVENT_VIMLEAVE,
    EVENT_WINENTER,
    EVENT_WINLEAVE,
    EVENT_LSPATTACH,
    EVENT_BUFCREATE,
    EVENT_TABCLOSED,
    EVENT_WINCLOSED,
    EVENT_BUFDELETE,
    EVENT_LSPDETACH,
    EVENT_SAFESTATE,
    EVENT_GUIFAILED,
    EVENT_BUFHIDDEN,
    EVENT_OPTIONSET,
    EVENT_TERMCLOSE,
    EVENT_TERMENTER,
    EVENT_TERMLEAVE,
    EVENT_LSPNOTIFY,
    EVENT_WINNEWPRE,
    EVENT_SOURCECMD,
    EVENT_SOURCEPRE,
    EVENT_VIMRESUME,
    EVENT_BUFUNLOAD,
    EVENT_FOCUSLOST,
    EVENT_MENUPOPUP,
    EVENT_BUFREADCMD,
    EVENT_BUFREADPRE,
    EVENT_DIRCHANGED,
    EVENT_SOURCEPOST,
    EVENT_BUFFILEPRE,
    EVENT_BUFWIPEOUT,
    EVENT_LSPREQUEST,
    EVENT_CURSORHOLD,
    EVENT_VIMRESIZED,
    EVENT_VIMSUSPEND,
    EVENT_WINRESIZED,
    EVENT_BUFNEWFILE,
    EVENT_SWAPEXISTS,
    EVENT_BUFREADPOST,
    EVENT_VIMLEAVEPRE,
    EVENT_FILEREADCMD,
    EVENT_FILEREADPRE,
    EVENT_REMOTEREPLY,
    EVENT_TERMREQUEST,
    EVENT_FOCUSGAINED,
    EVENT_MODECHANGED,
    EVENT_PACKCHANGED,
    EVENT_TERMCHANGED,
    EVENT_TEXTCHANGED,
    EVENT_BUFWRITECMD,
    EVENT_BUFWRITEPRE,
    EVENT_BUFFILEPOST,
    EVENT_BUFWINENTER,
    EVENT_BUFWINLEAVE,
    EVENT_CMDWINENTER,
    EVENT_CMDWINLEAVE,
    EVENT_LSPPROGRESS,
    EVENT_DIFFUPDATED,
    EVENT_CURSORHOLDI,
    EVENT_CURSORMOVED,
    EVENT_WINSCROLLED,
    EVENT_COLORSCHEME,
    EVENT_INSERTENTER,
    EVENT_INSERTLEAVE,
    EVENT_TABCLOSEDPRE,
    EVENT_CMDLINEENTER,
    EVENT_CMDLINELEAVE,
    EVENT_CMDUNDEFINED,
    EVENT_STDINREADPRE,
    EVENT_SHELLCMDPOST,
    EVENT_BUFWRITEPOST,
    EVENT_FILEENCODING,
    EVENT_FILEREADPOST,
    EVENT_FILEWRITECMD,
    EVENT_FILEWRITEPRE,
    EVENT_COMPLETEDONE,
    EVENT_CURSORMOVEDC,
    EVENT_CURSORMOVEDI,
    EVENT_TERMRESPONSE,
    EVENT_INSERTCHANGE,
    EVENT_TEXTCHANGEDI,
    EVENT_TEXTCHANGEDP,
    EVENT_TEXTCHANGEDT,
    EVENT_TEXTYANKPOST,
    EVENT_FILEAPPENDCMD,
    EVENT_FILEAPPENDPRE,
    EVENT_FILECHANGEDRO,
    EVENT_SEARCHWRAPPED,
    EVENT_FILTERREADPRE,
    EVENT_TABNEWENTERED,
    EVENT_DIRCHANGEDPRE,
    EVENT_STDINREADPOST,
    EVENT_INSERTCHARPRE,
    EVENT_FUNCUNDEFINED,
    EVENT_FILEWRITEPOST,
    EVENT_BUFMODIFIEDSET,
    EVENT_CMDLINECHANGED,
    EVENT_COLORSCHEMEPRE,
    EVENT_FILEAPPENDPOST,
    EVENT_FILTERREADPOST,
    EVENT_FILTERWRITEPRE,
    EVENT_INSERTLEAVEPRE,
    EVENT_LSPTOKENUPDATE,
    EVENT_PACKCHANGEDPRE,
    EVENT_QUICKFIXCMDPRE,
    EVENT_RECORDINGENTER,
    EVENT_RECORDINGLEAVE,
    EVENT_SESSIONLOADPRE,
    EVENT_SESSIONLOADPOST,
    EVENT_SHELLFILTERPOST,
    EVENT_FILTERWRITEPOST,
    EVENT_CMDLINELEAVEPRE,
    EVENT_ENCODINGCHANGED,
    EVENT_COMPLETECHANGED,
    EVENT_COMPLETEDONEPRE,
    EVENT_QUICKFIXCMDPOST,
    EVENT_SESSIONWRITEPOST,
    EVENT_FILECHANGEDSHELL,
    EVENT_SPELLFILEMISSING,
    EVENT_DIAGNOSTICCHANGED,
    EVENT_FILECHANGEDSHELLPOST,
];
unsafe extern "C" fn event_name2nr_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
            high = 1 as ::core::ffi::c_int;
        }
        6 => {
            match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                66 | 98 => {
                    low = 1 as ::core::ffi::c_int;
                    high = 3 as ::core::ffi::c_int;
                }
                83 | 115 => {
                    low = 3 as ::core::ffi::c_int;
                    high = 5 as ::core::ffi::c_int;
                }
                84 | 116 => {
                    low = 5 as ::core::ffi::c_int;
                    high = 6 as ::core::ffi::c_int;
                }
                87 | 119 => {
                    low = 6 as ::core::ffi::c_int;
                    high = 7 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        7 => {
            match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                66 | 98 => {
                    low = 7 as ::core::ffi::c_int;
                    high = 8 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 8 as ::core::ffi::c_int;
                    high = 9 as ::core::ffi::c_int;
                }
                77 | 109 => {
                    low = 9 as ::core::ffi::c_int;
                    high = 10 as ::core::ffi::c_int;
                }
                81 | 113 => {
                    low = 10 as ::core::ffi::c_int;
                    high = 11 as ::core::ffi::c_int;
                }
                85 | 117 => {
                    low = 11 as ::core::ffi::c_int;
                    high = 13 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        8 => {
            match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                66 | 98 => {
                    low = 13 as ::core::ffi::c_int;
                    high = 16 as ::core::ffi::c_int;
                }
                67 | 99 => {
                    low = 16 as ::core::ffi::c_int;
                    high = 18 as ::core::ffi::c_int;
                }
                70 | 102 => {
                    low = 18 as ::core::ffi::c_int;
                    high = 19 as ::core::ffi::c_int;
                }
                71 | 103 => {
                    low = 19 as ::core::ffi::c_int;
                    high = 20 as ::core::ffi::c_int;
                }
                80 | 112 => {
                    low = 20 as ::core::ffi::c_int;
                    high = 21 as ::core::ffi::c_int;
                }
                84 | 116 => {
                    low = 21 as ::core::ffi::c_int;
                    high = 24 as ::core::ffi::c_int;
                }
                86 | 118 => {
                    low = 24 as ::core::ffi::c_int;
                    high = 26 as ::core::ffi::c_int;
                }
                87 | 119 => {
                    low = 26 as ::core::ffi::c_int;
                    high = 28 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        9 => {
            match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                65 | 97 => {
                    low = 28 as ::core::ffi::c_int;
                    high = 29 as ::core::ffi::c_int;
                }
                67 | 99 => {
                    low = 29 as ::core::ffi::c_int;
                    high = 32 as ::core::ffi::c_int;
                }
                68 | 100 => {
                    low = 32 as ::core::ffi::c_int;
                    high = 34 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 34 as ::core::ffi::c_int;
                    high = 35 as ::core::ffi::c_int;
                }
                70 | 102 => {
                    low = 35 as ::core::ffi::c_int;
                    high = 36 as ::core::ffi::c_int;
                }
                72 | 104 => {
                    low = 36 as ::core::ffi::c_int;
                    high = 37 as ::core::ffi::c_int;
                }
                73 | 105 => {
                    low = 37 as ::core::ffi::c_int;
                    high = 38 as ::core::ffi::c_int;
                }
                77 | 109 => {
                    low = 38 as ::core::ffi::c_int;
                    high = 41 as ::core::ffi::c_int;
                }
                78 | 110 => {
                    low = 41 as ::core::ffi::c_int;
                    high = 43 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 43 as ::core::ffi::c_int;
                    high = 46 as ::core::ffi::c_int;
                }
                85 | 117 => {
                    low = 46 as ::core::ffi::c_int;
                    high = 49 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        10 => {
            match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                65 | 97 => {
                    low = 49 as ::core::ffi::c_int;
                    high = 52 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 52 as ::core::ffi::c_int;
                    high = 53 as ::core::ffi::c_int;
                }
                76 | 108 => {
                    low = 53 as ::core::ffi::c_int;
                    high = 54 as ::core::ffi::c_int;
                }
                80 | 112 => {
                    low = 54 as ::core::ffi::c_int;
                    high = 55 as ::core::ffi::c_int;
                }
                81 | 113 => {
                    low = 55 as ::core::ffi::c_int;
                    high = 56 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 56 as ::core::ffi::c_int;
                    high = 57 as ::core::ffi::c_int;
                }
                83 | 115 => {
                    low = 57 as ::core::ffi::c_int;
                    high = 60 as ::core::ffi::c_int;
                }
                87 | 119 => {
                    low = 60 as ::core::ffi::c_int;
                    high = 61 as ::core::ffi::c_int;
                }
                88 | 120 => {
                    low = 61 as ::core::ffi::c_int;
                    high = 62 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        11 => {
            match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                65 | 97 => {
                    low = 62 as ::core::ffi::c_int;
                    high = 64 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 64 as ::core::ffi::c_int;
                    high = 68 as ::core::ffi::c_int;
                }
                71 | 103 => {
                    low = 68 as ::core::ffi::c_int;
                    high = 69 as ::core::ffi::c_int;
                }
                72 | 104 => {
                    low = 69 as ::core::ffi::c_int;
                    high = 73 as ::core::ffi::c_int;
                }
                73 | 105 => {
                    low = 73 as ::core::ffi::c_int;
                    high = 75 as ::core::ffi::c_int;
                }
                76 | 108 => {
                    low = 75 as ::core::ffi::c_int;
                    high = 76 as ::core::ffi::c_int;
                }
                78 | 110 => {
                    low = 76 as ::core::ffi::c_int;
                    high = 80 as ::core::ffi::c_int;
                }
                79 | 111 => {
                    low = 80 as ::core::ffi::c_int;
                    high = 81 as ::core::ffi::c_int;
                }
                80 | 112 => {
                    low = 81 as ::core::ffi::c_int;
                    high = 82 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 82 as ::core::ffi::c_int;
                    high = 85 as ::core::ffi::c_int;
                }
                83 | 115 => {
                    low = 85 as ::core::ffi::c_int;
                    high = 86 as ::core::ffi::c_int;
                }
                84 | 116 => {
                    low = 86 as ::core::ffi::c_int;
                    high = 88 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        12 => {
            match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                66 | 98 => {
                    low = 88 as ::core::ffi::c_int;
                    high = 89 as ::core::ffi::c_int;
                }
                68 | 100 => {
                    low = 89 as ::core::ffi::c_int;
                    high = 93 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 93 as ::core::ffi::c_int;
                    high = 94 as ::core::ffi::c_int;
                }
                70 | 102 => {
                    low = 94 as ::core::ffi::c_int;
                    high = 95 as ::core::ffi::c_int;
                }
                76 | 108 => {
                    low = 95 as ::core::ffi::c_int;
                    high = 99 as ::core::ffi::c_int;
                }
                77 | 109 => {
                    low = 99 as ::core::ffi::c_int;
                    high = 100 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 100 as ::core::ffi::c_int;
                    high = 103 as ::core::ffi::c_int;
                }
                83 | 115 => {
                    low = 103 as ::core::ffi::c_int;
                    high = 104 as ::core::ffi::c_int;
                }
                88 | 120 => {
                    low = 104 as ::core::ffi::c_int;
                    high = 108 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        13 => {
            match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                65 | 97 => {
                    low = 108 as ::core::ffi::c_int;
                    high = 110 as ::core::ffi::c_int;
                }
                67 | 99 => {
                    low = 110 as ::core::ffi::c_int;
                    high = 112 as ::core::ffi::c_int;
                }
                69 | 101 => {
                    low = 112 as ::core::ffi::c_int;
                    high = 114 as ::core::ffi::c_int;
                }
                72 | 104 => {
                    low = 114 as ::core::ffi::c_int;
                    high = 115 as ::core::ffi::c_int;
                }
                78 | 110 => {
                    low = 115 as ::core::ffi::c_int;
                    high = 116 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 116 as ::core::ffi::c_int;
                    high = 117 as ::core::ffi::c_int;
                }
                85 | 117 => {
                    low = 117 as ::core::ffi::c_int;
                    high = 118 as ::core::ffi::c_int;
                }
                87 | 119 => {
                    low = 118 as ::core::ffi::c_int;
                    high = 119 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        14 => {
            match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                66 | 98 => {
                    low = 119 as ::core::ffi::c_int;
                    high = 120 as ::core::ffi::c_int;
                }
                67 | 99 => {
                    low = 120 as ::core::ffi::c_int;
                    high = 122 as ::core::ffi::c_int;
                }
                70 | 102 => {
                    low = 122 as ::core::ffi::c_int;
                    high = 125 as ::core::ffi::c_int;
                }
                73 | 105 => {
                    low = 125 as ::core::ffi::c_int;
                    high = 126 as ::core::ffi::c_int;
                }
                76 | 108 => {
                    low = 126 as ::core::ffi::c_int;
                    high = 127 as ::core::ffi::c_int;
                }
                80 | 112 => {
                    low = 127 as ::core::ffi::c_int;
                    high = 128 as ::core::ffi::c_int;
                }
                81 | 113 => {
                    low = 128 as ::core::ffi::c_int;
                    high = 129 as ::core::ffi::c_int;
                }
                82 | 114 => {
                    low = 129 as ::core::ffi::c_int;
                    high = 131 as ::core::ffi::c_int;
                }
                83 | 115 => {
                    low = 131 as ::core::ffi::c_int;
                    high = 132 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        15 => {
            match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                69 | 101 => {
                    low = 132 as ::core::ffi::c_int;
                    high = 133 as ::core::ffi::c_int;
                }
                72 | 104 => {
                    low = 133 as ::core::ffi::c_int;
                    high = 134 as ::core::ffi::c_int;
                }
                73 | 105 => {
                    low = 134 as ::core::ffi::c_int;
                    high = 135 as ::core::ffi::c_int;
                }
                77 | 109 => {
                    low = 135 as ::core::ffi::c_int;
                    high = 136 as ::core::ffi::c_int;
                }
                78 | 110 => {
                    low = 136 as ::core::ffi::c_int;
                    high = 137 as ::core::ffi::c_int;
                }
                79 | 111 => {
                    low = 137 as ::core::ffi::c_int;
                    high = 139 as ::core::ffi::c_int;
                }
                85 | 117 => {
                    low = 139 as ::core::ffi::c_int;
                    high = 140 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        16 => {
            match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                69 | 101 => {
                    low = 140 as ::core::ffi::c_int;
                    high = 141 as ::core::ffi::c_int;
                }
                73 | 105 => {
                    low = 141 as ::core::ffi::c_int;
                    high = 142 as ::core::ffi::c_int;
                }
                80 | 112 => {
                    low = 142 as ::core::ffi::c_int;
                    high = 143 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        17 => {
            low = 143 as ::core::ffi::c_int;
            high = 144 as ::core::ffi::c_int;
        }
        20 => {
            low = 144 as ::core::ffi::c_int;
            high = 145 as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if vim_strnicmp_asc(str, event_names[event_hash[i as usize] as usize].name, len)
            == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
