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
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn putc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
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
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
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
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_alloc(arena: *mut Arena, size: size_t, align: bool) -> *mut ::core::ffi::c_void;
    fn arena_mem_free(mem: ArenaMem);
    fn object_to_vim_take_luaref(
        obj: *mut Object,
        tv: *mut typval_T,
        take_luaref: bool,
        err: *mut Error,
    );
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn string_to_cstr(str: String_0) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_take_arraybuilder(arena: *mut Arena, arr: *mut ArrayBuilder) -> Array;
    fn api_free_object(value: Object);
    fn api_clear_error(value: *mut Error);
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_set_sctx(channel_id: uint64_t) -> sctx_T;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_langmap: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
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
    fn transchar(c: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cmdline_fuzzy_complete(fuzzystr: *const ::core::ffi::c_char) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg: [::core::ffi::c_char; 0];
    static e_noabbr: [::core::ffi::c_char; 0];
    static e_nomap: [::core::ffi::c_char; 0];
    fn eval_to_string(
        arg: *mut ::core::ffi::c_char,
        join_list: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    fn last_set_msg(script_ctx: sctx_T);
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn semsg_multiline(
        kind: *const ::core::ffi::c_char,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn iemsg(s: *const ::core::ffi::c_char);
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_start();
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_outtrans_special(
        strstart: *const ::core::ffi::c_char,
        from: bool,
        maxlen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn str2special_save(
        str: *const ::core::ffi::c_char,
        replace_spaces: bool,
        replace_lt: bool,
    ) -> *mut ::core::ffi::c_char;
    fn str2special_arena(
        str: *const ::core::ffi::c_char,
        replace_spaces: bool,
        replace_lt: bool,
        arena: *mut Arena,
    ) -> *mut ::core::ffi::c_char;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn message_filtered(msg_0: *const ::core::ffi::c_char) -> bool;
    fn msg_clr_eos();
    fn swmsg(hl: bool, fmt: *const ::core::ffi::c_char, ...);
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
    fn tv_dict_get_bool(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        def: ::core::ffi::c_int,
    ) -> varnumber_T;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_bool(tv: *const typval_T) -> varnumber_T;
    fn tv_check_for_dict_arg(args: *const typval_T, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn tv_get_string_buf_chk(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string_buf(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn find_func(name: *const ::core::ffi::c_char) -> *mut ufunc_T;
    fn set_vim_var_char(c: ::core::ffi::c_int);
    fn put_eol(fd: *mut FILE) -> ::core::ffi::c_int;
    fn check_secure() -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
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
    fn noremap_keys() -> bool;
    fn ins_typebuf(
        str: *mut ::core::ffi::c_char,
        noremap: ::core::ffi::c_int,
        offset: ::core::ffi::c_int,
        nottyped: bool,
        silent: bool,
    ) -> ::core::ffi::c_int;
    static mut msg_col: ::core::ffi::c_int;
    static mut msg_row: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut secure: ::core::ffi::c_int;
    static mut State: ::core::ffi::c_int;
    static mut no_abbr: bool;
    static mut mapped_ctrl_c: ::core::ffi::c_int;
    static mut msg_silent: ::core::ffi::c_int;
    static mut typebuf: typebuf_T;
    static mut expr_map_lock: ::core::ffi::c_int;
    static mut got_int: bool;
    static mut langmap_mapchar: [uint8_t; 256];
    fn get_special_key_name(
        c: ::core::ffi::c_int,
        modifiers: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn replace_termcodes(
        from: *const ::core::ffi::c_char,
        from_len: size_t,
        bufp: *mut *mut ::core::ffi::c_char,
        sid_arg: scid_T,
        flags: ::core::ffi::c_int,
        did_simplify: *mut bool,
        cpo_val: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strsave_escape_ks(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_unescape_ks(p: *mut ::core::ffi::c_char);
    fn api_free_luaref(ref_0: LuaRef);
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn nlua_set_sctx(current: *mut sctx_T);
    fn nlua_funcref_str(ref_0: LuaRef, arena: *mut Arena) -> *mut ::core::ffi::c_char;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_prevptr(
        line: *mut ::core::ffi::c_char,
        p: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn mb_unescape(pp: *mut *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    static mut exestack: garray_T;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
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
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type time_t = __time_t;
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
pub type Buffer = handle_T;
pub type OptionalKeys = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_keymap {
    pub is_set__keymap_: OptionalKeys,
    pub noremap: Boolean,
    pub nowait: Boolean,
    pub silent: Boolean,
    pub script: Boolean,
    pub expr: Boolean,
    pub unique: Boolean,
    pub callback: LuaRef,
    pub desc: String_0,
    pub replace_keycodes: Boolean,
}
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXMAPLEN: C2Rust_Unnamed_13 = 50;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_14 = 76;
pub const HLF_PRE: C2Rust_Unnamed_14 = 75;
pub const HLF_OK: C2Rust_Unnamed_14 = 74;
pub const HLF_SO: C2Rust_Unnamed_14 = 73;
pub const HLF_SE: C2Rust_Unnamed_14 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_14 = 71;
pub const HLF_TS: C2Rust_Unnamed_14 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_14 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_14 = 68;
pub const HLF_CU: C2Rust_Unnamed_14 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_14 = 66;
pub const HLF_WBR: C2Rust_Unnamed_14 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_14 = 64;
pub const HLF_MSG: C2Rust_Unnamed_14 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_14 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_14 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_14 = 60;
pub const HLF_0: C2Rust_Unnamed_14 = 59;
pub const HLF_QFL: C2Rust_Unnamed_14 = 58;
pub const HLF_MC: C2Rust_Unnamed_14 = 57;
pub const HLF_CUL: C2Rust_Unnamed_14 = 56;
pub const HLF_CUC: C2Rust_Unnamed_14 = 55;
pub const HLF_TPF: C2Rust_Unnamed_14 = 54;
pub const HLF_TPS: C2Rust_Unnamed_14 = 53;
pub const HLF_TP: C2Rust_Unnamed_14 = 52;
pub const HLF_PBR: C2Rust_Unnamed_14 = 51;
pub const HLF_PST: C2Rust_Unnamed_14 = 50;
pub const HLF_PSB: C2Rust_Unnamed_14 = 49;
pub const HLF_PSX: C2Rust_Unnamed_14 = 48;
pub const HLF_PNX: C2Rust_Unnamed_14 = 47;
pub const HLF_PSK: C2Rust_Unnamed_14 = 46;
pub const HLF_PNK: C2Rust_Unnamed_14 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_14 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_14 = 43;
pub const HLF_PSI: C2Rust_Unnamed_14 = 42;
pub const HLF_PNI: C2Rust_Unnamed_14 = 41;
pub const HLF_SPL: C2Rust_Unnamed_14 = 40;
pub const HLF_SPR: C2Rust_Unnamed_14 = 39;
pub const HLF_SPC: C2Rust_Unnamed_14 = 38;
pub const HLF_SPB: C2Rust_Unnamed_14 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_14 = 36;
pub const HLF_SC: C2Rust_Unnamed_14 = 35;
pub const HLF_TXA: C2Rust_Unnamed_14 = 34;
pub const HLF_TXD: C2Rust_Unnamed_14 = 33;
pub const HLF_DED: C2Rust_Unnamed_14 = 32;
pub const HLF_CHD: C2Rust_Unnamed_14 = 31;
pub const HLF_ADD: C2Rust_Unnamed_14 = 30;
pub const HLF_FC: C2Rust_Unnamed_14 = 29;
pub const HLF_FL: C2Rust_Unnamed_14 = 28;
pub const HLF_WM: C2Rust_Unnamed_14 = 27;
pub const HLF_W: C2Rust_Unnamed_14 = 26;
pub const HLF_VNC: C2Rust_Unnamed_14 = 25;
pub const HLF_V: C2Rust_Unnamed_14 = 24;
pub const HLF_T: C2Rust_Unnamed_14 = 23;
pub const HLF_VSP: C2Rust_Unnamed_14 = 22;
pub const HLF_C: C2Rust_Unnamed_14 = 21;
pub const HLF_SNC: C2Rust_Unnamed_14 = 20;
pub const HLF_S: C2Rust_Unnamed_14 = 19;
pub const HLF_R: C2Rust_Unnamed_14 = 18;
pub const HLF_CLF: C2Rust_Unnamed_14 = 17;
pub const HLF_CLS: C2Rust_Unnamed_14 = 16;
pub const HLF_CLN: C2Rust_Unnamed_14 = 15;
pub const HLF_LNB: C2Rust_Unnamed_14 = 14;
pub const HLF_LNA: C2Rust_Unnamed_14 = 13;
pub const HLF_N: C2Rust_Unnamed_14 = 12;
pub const HLF_CM: C2Rust_Unnamed_14 = 11;
pub const HLF_M: C2Rust_Unnamed_14 = 10;
pub const HLF_LC: C2Rust_Unnamed_14 = 9;
pub const HLF_L: C2Rust_Unnamed_14 = 8;
pub const HLF_I: C2Rust_Unnamed_14 = 7;
pub const HLF_E: C2Rust_Unnamed_14 = 6;
pub const HLF_D: C2Rust_Unnamed_14 = 5;
pub const HLF_AT: C2Rust_Unnamed_14 = 4;
pub const HLF_TERM: C2Rust_Unnamed_14 = 3;
pub const HLF_EOB: C2Rust_Unnamed_14 = 2;
pub const HLF_8: C2Rust_Unnamed_14 = 1;
pub const HLF_NONE: C2Rust_Unnamed_14 = 0;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_15 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_15 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_15 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_15 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_15 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_15 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_15 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_15 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_15 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_15 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_15 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_15 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_15 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_15 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_15 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_15 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_15 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_15 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_15 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_15 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_15 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_15 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_15 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_15 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_15 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_15 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_15 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_15 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_15 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_15 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_15 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_15 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_15 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_15 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_15 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_15 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_15 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_15 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_15 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_15 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_15 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_15 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_15 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_15 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_15 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_15 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_15 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_15 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_15 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_15 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_15 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_15 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_15 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_15 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_15 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_15 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_15 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_15 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_15 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_15 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_15 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_15 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_15 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_15 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_15 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_15 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_15 = -2;
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
    pub cs_pend: C2Rust_Unnamed_16,
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
pub union C2Rust_Unnamed_16 {
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
pub struct ArrayBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
    pub init_array: [Object; 16],
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
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_17 = -2147483648;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzmatch_str_T {
    pub idx: ::core::ffi::c_int,
    pub str: *mut ::core::ffi::c_char,
    pub score: ::core::ffi::c_int,
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
    pub es_info: C2Rust_Unnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_19 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_19 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_19 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_19 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_19 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_19 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_19 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_19 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_19 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_19 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_19 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_19 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_19 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_19 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_19 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_19 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_19 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_19 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_19 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_19 = 1;
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
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_20 = 8;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_20 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_20 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_20 = 1;
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub type MapArguments = map_arguments;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct map_arguments {
    pub buffer: bool,
    pub expr: bool,
    pub noremap: bool,
    pub nowait: bool,
    pub script: bool,
    pub silent: bool,
    pub unique: bool,
    pub replace_keycodes: bool,
    pub lhs: [::core::ffi::c_char; 51],
    pub lhs_len: size_t,
    pub alt_lhs: [::core::ffi::c_char; 51],
    pub alt_lhs_len: size_t,
    pub rhs: *mut ::core::ffi::c_char,
    pub rhs_len: size_t,
    pub rhs_lua: LuaRef,
    pub rhs_is_noop: bool,
    pub orig_rhs: *mut ::core::ffi::c_char,
    pub orig_rhs_len: size_t,
    pub desc: *mut ::core::ffi::c_char,
}
pub const MAPTYPE_UNMAP: C2Rust_Unnamed_21 = 1;
pub const MAPTYPE_NOREMAP: C2Rust_Unnamed_21 = 2;
pub const MAPTYPE_UNMAP_LHS: C2Rust_Unnamed_21 = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct langmap_entry_T {
    pub from: ::core::ffi::c_int,
    pub to: ::core::ffi::c_int,
}
pub const MAPTYPE_MAP: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    return dest;
}
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const KEYSET_OPTIDX_keymap__desc: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_keymap__callback: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const Ctrl_J: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const CPO_BSLASH: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
pub const MAX_MAPHASH: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
static mut first_abbr: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
static mut maphash: [*mut mapblock_T; 256] = [
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
];
pub const MAP_ARGUMENTS_INIT: MapArguments = map_arguments {
    buffer: false_0 != 0,
    expr: false_0 != 0,
    noremap: false_0 != 0,
    nowait: false_0 != 0,
    script: false_0 != 0,
    silent: false_0 != 0,
    unique: false_0 != 0,
    replace_keycodes: false_0 != 0,
    lhs: [
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
    ],
    lhs_len: 0 as size_t,
    alt_lhs: [
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
    ],
    alt_lhs_len: 0 as size_t,
    rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    rhs_len: 0 as size_t,
    rhs_lua: LUA_NOREF,
    rhs_is_noop: false_0 != 0,
    orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    orig_rhs_len: 0 as size_t,
    desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
static mut e_global_abbreviation_already_exists_for_str: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E224: Global abbreviation already exists for %s\0",
    )
};
static mut e_global_mapping_already_exists_for_str: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"E225: Global mapping already exists for %s\0",
    )
};
static mut e_abbreviation_already_exists_for_str: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E226: Abbreviation already exists for %s\0",
    )
};
static mut e_mapping_already_exists_for_str: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"E227: Mapping already exists for %s\0",
    )
};
static mut e_entries_missing_in_mapset_dict_argument: [::core::ffi::c_char; 48] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
        *b"E460: Entries missing in mapset() dict argument\0",
    )
};
static mut e_illegal_map_mode_string_str: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"E1276: Illegal map mode string: '%s'\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn get_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    return maphash[(if state
        & (MODE_NORMAL as ::core::ffi::c_int
            | MODE_VISUAL as ::core::ffi::c_int
            | MODE_SELECT as ::core::ffi::c_int
            | MODE_OP_PENDING as ::core::ffi::c_int
            | MODE_TERMINAL as ::core::ffi::c_int)
        != 0
    {
        c
    } else {
        c ^ 0x80 as ::core::ffi::c_int
    }) as usize] as *mut mapblock_T;
}
#[no_mangle]
pub unsafe extern "C" fn get_buf_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    return (*curbuf).b_maphash[(if state
        & (MODE_NORMAL as ::core::ffi::c_int
            | MODE_VISUAL as ::core::ffi::c_int
            | MODE_SELECT as ::core::ffi::c_int
            | MODE_OP_PENDING as ::core::ffi::c_int
            | MODE_TERMINAL as ::core::ffi::c_int)
        != 0
    {
        c
    } else {
        c ^ 0x80 as ::core::ffi::c_int
    }) as usize] as *mut mapblock_T;
}
unsafe extern "C" fn mapblock_free(mut mpp: *mut *mut mapblock_T) {
    let mut mp: *mut mapblock_T = *mpp;
    xfree((*mp).m_keys as *mut ::core::ffi::c_void);
    if !(*mp).m_alt.is_null() {
        (*(*mp).m_alt).m_alt = ::core::ptr::null_mut::<mapblock_T>();
    } else {
        if (*mp).m_luaref != LUA_NOREF {
            api_free_luaref((*mp).m_luaref);
            (*mp).m_luaref = LUA_NOREF as LuaRef;
        }
        xfree((*mp).m_str as *mut ::core::ffi::c_void);
        xfree((*mp).m_orig_str as *mut ::core::ffi::c_void);
        xfree((*mp).m_desc as *mut ::core::ffi::c_void);
    }
    *mpp = (*mp).m_next;
    xfree(mp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn map_mode_to_chars(
    mut mode: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = buf;
    if mode & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int)
        == MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int
    {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        *c2rust_fresh0 = '!' as ::core::ffi::c_char;
    } else if mode & MODE_INSERT as ::core::ffi::c_int != 0 {
        let c2rust_fresh1 = p;
        p = p.offset(1);
        *c2rust_fresh1 = 'i' as ::core::ffi::c_char;
    } else if mode & MODE_LANGMAP as ::core::ffi::c_int != 0 {
        let c2rust_fresh2 = p;
        p = p.offset(1);
        *c2rust_fresh2 = 'l' as ::core::ffi::c_char;
    } else if mode & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        let c2rust_fresh3 = p;
        p = p.offset(1);
        *c2rust_fresh3 = 'c' as ::core::ffi::c_char;
    } else if mode
        & (MODE_NORMAL as ::core::ffi::c_int
            | MODE_VISUAL as ::core::ffi::c_int
            | MODE_SELECT as ::core::ffi::c_int
            | MODE_OP_PENDING as ::core::ffi::c_int)
        == MODE_NORMAL as ::core::ffi::c_int
            | MODE_VISUAL as ::core::ffi::c_int
            | MODE_SELECT as ::core::ffi::c_int
            | MODE_OP_PENDING as ::core::ffi::c_int
    {
        let c2rust_fresh4 = p;
        p = p.offset(1);
        *c2rust_fresh4 = ' ' as ::core::ffi::c_char;
    } else {
        if mode & MODE_NORMAL as ::core::ffi::c_int != 0 {
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = 'n' as ::core::ffi::c_char;
        }
        if mode & MODE_OP_PENDING as ::core::ffi::c_int != 0 {
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = 'o' as ::core::ffi::c_char;
        }
        if mode & MODE_TERMINAL as ::core::ffi::c_int != 0 {
            let c2rust_fresh7 = p;
            p = p.offset(1);
            *c2rust_fresh7 = 't' as ::core::ffi::c_char;
        }
        if mode & (MODE_VISUAL as ::core::ffi::c_int | MODE_SELECT as ::core::ffi::c_int)
            == MODE_VISUAL as ::core::ffi::c_int | MODE_SELECT as ::core::ffi::c_int
        {
            let c2rust_fresh8 = p;
            p = p.offset(1);
            *c2rust_fresh8 = 'v' as ::core::ffi::c_char;
        } else {
            if mode & MODE_VISUAL as ::core::ffi::c_int != 0 {
                let c2rust_fresh9 = p;
                p = p.offset(1);
                *c2rust_fresh9 = 'x' as ::core::ffi::c_char;
            }
            if mode & MODE_SELECT as ::core::ffi::c_int != 0 {
                let c2rust_fresh10 = p;
                p = p.offset(1);
                *c2rust_fresh10 = 's' as ::core::ffi::c_char;
            }
        }
    }
    *p = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn showmap(mut mp: *mut mapblock_T, mut local: bool) {
    if message_filtered((*mp).m_keys) as ::core::ffi::c_int != 0
        && message_filtered((*mp).m_str) as ::core::ffi::c_int != 0
        && ((*mp).m_desc.is_null() || message_filtered((*mp).m_desc) as ::core::ffi::c_int != 0)
    {
        return;
    }
    if msg_col > 0 as ::core::ffi::c_int || msg_silent != 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int {
            return;
        }
    }
    let mut mapchars: [::core::ffi::c_char; 7] = [0; 7];
    map_mode_to_chars((*mp).m_mode, &raw mut mapchars as *mut ::core::ffi::c_char);
    msg_puts(&raw mut mapchars as *mut ::core::ffi::c_char);
    let mut len: size_t = strlen(&raw mut mapchars as *mut ::core::ffi::c_char);
    loop {
        len = len.wrapping_add(1);
        if len > 3 as size_t {
            break;
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    len = msg_outtrans_special((*mp).m_keys, true_0 != 0, 0 as ::core::ffi::c_int) as size_t;
    loop {
        msg_putchar(' ' as ::core::ffi::c_int);
        len = len.wrapping_add(1);
        if len >= 12 as size_t {
            break;
        }
    }
    if (*mp).m_noremap == REMAP_NONE as ::core::ffi::c_int {
        msg_puts_hl(
            b"*\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8 as ::core::ffi::c_int,
            false_0 != 0,
        );
    } else if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
        msg_puts_hl(
            b"&\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8 as ::core::ffi::c_int,
            false_0 != 0,
        );
    } else {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    if local {
        msg_putchar('@' as ::core::ffi::c_int);
    } else {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    if (*mp).m_luaref != LUA_NOREF {
        let mut str: *mut ::core::ffi::c_char =
            nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
        msg_puts_hl(str, HLF_8 as ::core::ffi::c_int, false_0 != 0);
        xfree(str as *mut ::core::ffi::c_void);
    } else if *(*mp).m_str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        msg_puts_hl(
            b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8 as ::core::ffi::c_int,
            false_0 != 0,
        );
    } else {
        msg_outtrans_special((*mp).m_str, false_0 != 0, 0 as ::core::ffi::c_int);
    }
    if !(*mp).m_desc.is_null() {
        msg_puts(b"\n                 \0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts((*mp).m_desc);
    }
    if p_verbose > 0 as OptInt {
        last_set_msg((*mp).m_script_ctx);
    }
    msg_clr_eos();
}
unsafe extern "C" fn set_maparg_lhs_rhs(
    orig_lhs: *const ::core::ffi::c_char,
    orig_lhs_len: size_t,
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) -> bool {
    (*mapargs).rhs_lua = rhs_lua;
    let mut lhs_buf: [::core::ffi::c_char; 128] = [0; 128];
    let mut did_simplify: bool = false_0 != 0;
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let mut bufarg: *mut ::core::ffi::c_char = &raw mut lhs_buf as *mut ::core::ffi::c_char;
    let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
        orig_lhs,
        orig_lhs_len,
        &raw mut bufarg,
        0 as scid_T,
        flags,
        &raw mut did_simplify,
        cpo_val,
    );
    if replaced.is_null() {
        return false_0 != 0;
    }
    (*mapargs).lhs_len = strlen(replaced);
    xstrlcpy(
        &raw mut (*mapargs).lhs as *mut ::core::ffi::c_char,
        replaced,
        ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
    );
    if did_simplify {
        replaced = replace_termcodes(
            orig_lhs,
            orig_lhs_len,
            &raw mut bufarg,
            0 as scid_T,
            flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            cpo_val,
        );
        if replaced.is_null() {
            return false_0 != 0;
        }
        (*mapargs).alt_lhs_len = strlen(replaced);
        xstrlcpy(
            &raw mut (*mapargs).alt_lhs as *mut ::core::ffi::c_char,
            replaced,
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
        );
    } else {
        (*mapargs).alt_lhs_len = 0 as size_t;
    }
    set_maparg_rhs(
        orig_rhs,
        orig_rhs_len,
        rhs_lua,
        0 as scid_T,
        cpo_val,
        mapargs,
    );
    return true_0 != 0;
}
unsafe extern "C" fn set_maparg_rhs(
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    sid: scid_T,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) {
    (*mapargs).rhs_lua = rhs_lua;
    if rhs_lua == LUA_NOREF {
        (*mapargs).orig_rhs_len = orig_rhs_len;
        (*mapargs).orig_rhs = xcalloc(
            (*mapargs).orig_rhs_len.wrapping_add(1 as size_t),
            ::core::mem::size_of::<::core::ffi::c_char>(),
        ) as *mut ::core::ffi::c_char;
        xmemcpyz(
            (*mapargs).orig_rhs as *mut ::core::ffi::c_void,
            orig_rhs as *const ::core::ffi::c_void,
            (*mapargs).orig_rhs_len,
        );
        if strcasecmp(
            orig_rhs as *mut ::core::ffi::c_char,
            b"<nop>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*mapargs).rhs = xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
                as *mut ::core::ffi::c_char;
            (*mapargs).rhs_len = 0 as size_t;
            (*mapargs).rhs_is_noop = true_0 != 0;
        } else {
            let mut rhs_buf: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
                orig_rhs,
                orig_rhs_len,
                &raw mut rhs_buf,
                sid,
                REPTERM_DO_LT as ::core::ffi::c_int,
                ::core::ptr::null_mut::<bool>(),
                cpo_val,
            );
            (*mapargs).rhs_len = strlen(replaced);
            (*mapargs).rhs_is_noop =
                orig_rhs_len != 0 as size_t && (*mapargs).rhs_len == 0 as size_t;
            (*mapargs).rhs = replaced;
        }
    } else {
        let mut tmp_buf: [::core::ffi::c_char; 64] = [0; 64];
        (*mapargs).orig_rhs = xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
            as *mut ::core::ffi::c_char;
        (*mapargs).orig_rhs_len = 0 as size_t;
        (*mapargs).rhs_len = vim_snprintf(
            &raw mut tmp_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as size_t),
            b"%c%c%c%d\r\0".as_ptr() as *const ::core::ffi::c_char,
            K_SPECIAL,
            KS_EXTRA,
            KE_LUA as ::core::ffi::c_int,
            rhs_lua,
        ) as size_t;
        (*mapargs).rhs = xstrdup(&raw mut tmp_buf as *mut ::core::ffi::c_char);
    };
}
unsafe extern "C" fn str_to_mapargs(
    mut strargs: *const ::core::ffi::c_char,
    mut is_unmap: bool,
    mut mapargs: *mut MapArguments,
) -> ::core::ffi::c_int {
    let mut to_parse: *const ::core::ffi::c_char = strargs;
    to_parse = skipwhite(to_parse);
    memset(
        mapargs as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<MapArguments>(),
    );
    loop {
        if strncmp(
            to_parse,
            b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).buffer = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).nowait = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).silent = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(9 as ::core::ffi::c_int as isize));
        } else if strncmp(
            to_parse,
            b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).script = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(6 as ::core::ffi::c_int as isize));
            (*mapargs).expr = true_0 != 0;
        } else {
            if strncmp(
                to_parse,
                b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).unique = true_0 != 0;
        }
    }
    let mut lhs_end: *const ::core::ffi::c_char = to_parse;
    let mut do_backslash: bool = vim_strchr(p_cpo, CPO_BSLASH).is_null();
    while *lhs_end as ::core::ffi::c_int != 0
        && (is_unmap as ::core::ffi::c_int != 0 || !ascii_iswhite(*lhs_end as ::core::ffi::c_int))
    {
        if (*lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == Ctrl_V
            || do_backslash as ::core::ffi::c_int != 0
                && *lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
            && *lhs_end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            lhs_end = lhs_end.offset(1);
        }
        lhs_end = lhs_end.offset(1);
    }
    let mut rhs_start: *const ::core::ffi::c_char = skipwhite(lhs_end);
    let mut orig_lhs_len: size_t = lhs_end.offset_from(to_parse) as size_t;
    if orig_lhs_len >= 256 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    let mut lhs_to_replace: [::core::ffi::c_char; 256] = [0; 256];
    xmemcpyz(
        &raw mut lhs_to_replace as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        to_parse as *const ::core::ffi::c_void,
        orig_lhs_len,
    );
    let mut orig_rhs_len: size_t = strlen(rhs_start);
    if !set_maparg_lhs_rhs(
        &raw mut lhs_to_replace as *mut ::core::ffi::c_char,
        orig_lhs_len,
        rhs_start,
        orig_rhs_len,
        LUA_NOREF,
        p_cpo,
        mapargs,
    ) {
        return 1 as ::core::ffi::c_int;
    }
    if (*mapargs).lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn map_add(
    mut buf: *mut buf_T,
    mut map_table: *mut *mut mapblock_T,
    mut abbr_table: *mut *mut mapblock_T,
    mut keys: *const ::core::ffi::c_char,
    mut args: *mut MapArguments,
    mut noremap: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut is_abbr: bool,
    mut sid: scid_T,
    mut lnum: linenr_T,
    mut simplified: bool,
) -> *mut mapblock_T {
    let mut mp: *mut mapblock_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<mapblock_T>()) as *mut mapblock_T;
    if *keys as ::core::ffi::c_int == Ctrl_C {
        if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
            (*buf).b_mapped_ctrl_c |= mode;
        } else {
            mapped_ctrl_c |= mode;
        }
    }
    (*mp).m_keys = xstrdup(keys);
    (*mp).m_str = (*args).rhs;
    (*mp).m_orig_str = (*args).orig_rhs;
    (*mp).m_luaref = (*args).rhs_lua;
    (*mp).m_keylen = strlen((*mp).m_keys) as ::core::ffi::c_int;
    (*mp).m_noremap = noremap;
    (*mp).m_nowait = (*args).nowait as ::core::ffi::c_char;
    (*mp).m_silent = (*args).silent as ::core::ffi::c_char;
    (*mp).m_mode = mode;
    (*mp).m_simplified = simplified as ::core::ffi::c_int;
    (*mp).m_expr = (*args).expr as ::core::ffi::c_char;
    (*mp).m_replace_keycodes = (*args).replace_keycodes;
    if sid != 0 as ::core::ffi::c_int {
        (*mp).m_script_ctx.sc_sid = sid;
        (*mp).m_script_ctx.sc_lnum = lnum;
    } else {
        (*mp).m_script_ctx = current_sctx;
        (*mp).m_script_ctx.sc_lnum += (*(exestack.ga_data as *mut estack_T)
            .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*mp).m_script_ctx);
    }
    (*mp).m_desc = (*args).desc;
    if is_abbr {
        (*mp).m_next = *abbr_table;
        *abbr_table = mp;
    } else {
        let n: ::core::ffi::c_int = if (*mp).m_mode
            & (MODE_NORMAL as ::core::ffi::c_int
                | MODE_VISUAL as ::core::ffi::c_int
                | MODE_SELECT as ::core::ffi::c_int
                | MODE_OP_PENDING as ::core::ffi::c_int
                | MODE_TERMINAL as ::core::ffi::c_int)
            != 0
        {
            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        } else {
            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ^ 0x80 as ::core::ffi::c_int
        };
        (*mp).m_next = *map_table.offset(n as isize);
        *map_table.offset(n as isize) = mp;
    }
    return mp;
}
unsafe extern "C" fn buf_do_map(
    mut maptype: ::core::ffi::c_int,
    mut args: *mut MapArguments,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    let mut lhs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut did_simplify: bool = false;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut map_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
        &raw mut (*buf).b_maphash as *mut *mut mapblock_T
    } else {
        &raw mut maphash as *mut *mut mapblock_T
    };
    let mut abbr_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
        &raw mut (*buf).b_first_abbr
    } else {
        &raw mut first_abbr
    };
    let mut mp_result: [*mut mapblock_T; 2] = [
        ::core::ptr::null_mut::<mapblock_T>(),
        ::core::ptr::null_mut::<mapblock_T>(),
    ];
    let mut unmap_lhs_only: bool = false_0 != 0;
    if maptype == MAPTYPE_UNMAP_LHS as ::core::ffi::c_int {
        unmap_lhs_only = true_0 != 0;
        maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
    }
    let mut noremap: ::core::ffi::c_int = if (*args).script as ::core::ffi::c_int != 0 {
        REMAP_SCRIPT as ::core::ffi::c_int
    } else if maptype == MAPTYPE_NOREMAP as ::core::ffi::c_int {
        REMAP_NONE as ::core::ffi::c_int
    } else {
        REMAP_YES as ::core::ffi::c_int
    };
    let has_lhs: bool = (*args).lhs[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL;
    let has_rhs: bool = (*args).rhs_lua != LUA_NOREF
        || *(*args).rhs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        || (*args).rhs_is_noop as ::core::ffi::c_int != 0;
    let do_print: bool = !has_lhs || maptype != MAPTYPE_UNMAP as ::core::ffi::c_int && !has_rhs;
    if do_print {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    }
    '_theend: {
        if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !has_lhs {
            retval = 1 as ::core::ffi::c_int;
        } else {
            lhs = &raw mut (*args).lhs as *mut ::core::ffi::c_char;
            did_simplify = (*args).alt_lhs_len != 0 as size_t;
            let mut keyround: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while keyround <= 2 as ::core::ffi::c_int {
                let mut did_it: bool = false_0 != 0;
                let mut did_local: bool = false_0 != 0;
                let mut keyround1_simplified: bool =
                    keyround == 1 as ::core::ffi::c_int && did_simplify as ::core::ffi::c_int != 0;
                let mut len: ::core::ffi::c_int = (*args).lhs_len as ::core::ffi::c_int;
                if keyround == 2 as ::core::ffi::c_int {
                    if !did_simplify {
                        break;
                    }
                    lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                    len = (*args).alt_lhs_len as ::core::ffi::c_int;
                } else if did_simplify as ::core::ffi::c_int != 0
                    && do_print as ::core::ffi::c_int != 0
                {
                    lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                    len = (*args).alt_lhs_len as ::core::ffi::c_int;
                }
                's_209: {
                    if has_lhs {
                        if len > MAXMAPLEN as ::core::ffi::c_int {
                            retval = 1 as ::core::ffi::c_int;
                            break '_theend;
                        } else if is_abbrev as ::core::ffi::c_int != 0
                            && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                        {
                            let mut same: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let first: ::core::ffi::c_int = vim_iswordp(lhs) as ::core::ffi::c_int;
                            let mut last: ::core::ffi::c_int = first;
                            let mut p: *const ::core::ffi::c_char =
                                lhs.offset(utfc_ptr2len(lhs) as isize);
                            let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            while p < lhs.offset(len as isize) {
                                n += 1;
                                last = vim_iswordp(p) as ::core::ffi::c_int;
                                if same == -1 as ::core::ffi::c_int && last != first {
                                    same = n - 1 as ::core::ffi::c_int;
                                }
                                p = p.offset(utfc_ptr2len(p) as isize);
                            }
                            if last != 0
                                && n > 2 as ::core::ffi::c_int
                                && same >= 0 as ::core::ffi::c_int
                                && same < n - 1 as ::core::ffi::c_int
                            {
                                retval = 1 as ::core::ffi::c_int;
                                break '_theend;
                            } else {
                                n = 0 as ::core::ffi::c_int;
                                loop {
                                    if n >= len {
                                        break 's_209;
                                    }
                                    if ascii_iswhite(*lhs.offset(n as isize) as ::core::ffi::c_int)
                                    {
                                        retval = 1 as ::core::ffi::c_int;
                                        break '_theend;
                                    } else {
                                        n += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if has_lhs as ::core::ffi::c_int != 0
                    && has_rhs as ::core::ffi::c_int != 0
                    && is_abbrev as ::core::ffi::c_int != 0
                {
                    no_abbr = false_0 != 0;
                }
                if do_print {
                    msg_start();
                }
                's_299: {
                    if (*args).unique as ::core::ffi::c_int != 0
                        && map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                        && has_lhs as ::core::ffi::c_int != 0
                        && has_rhs as ::core::ffi::c_int != 0
                        && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                    {
                        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        loop {
                            if !(hash < 256 as ::core::ffi::c_int && !got_int) {
                                break 's_299;
                            }
                            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                            if is_abbrev {
                                if hash != 0 as ::core::ffi::c_int {
                                    break 's_299;
                                }
                                mp = first_abbr;
                            } else {
                                mp = maphash[hash as usize] as *mut mapblock_T;
                            }
                            while !mp.is_null() && !got_int {
                                if (*mp).m_mode & mode != 0 as ::core::ffi::c_int
                                    && (*mp).m_keylen == len
                                    && strncmp((*mp).m_keys, lhs, len as size_t)
                                        == 0 as ::core::ffi::c_int
                                {
                                    retval = 6 as ::core::ffi::c_int;
                                    break '_theend;
                                } else {
                                    mp = (*mp).m_next;
                                }
                            }
                            hash += 1;
                        }
                    }
                }
                if map_table != &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                    && !has_rhs
                    && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                {
                    let mut hash_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while hash_0 < 256 as ::core::ffi::c_int && !got_int {
                        let mut mp_0: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                        if is_abbrev {
                            if hash_0 != 0 as ::core::ffi::c_int {
                                break;
                            }
                            mp_0 = (*buf).b_first_abbr;
                        } else {
                            mp_0 = (*buf).b_maphash[hash_0 as usize] as *mut mapblock_T;
                        }
                        while !mp_0.is_null() && !got_int {
                            if (*mp_0).m_simplified == 0
                                && (*mp_0).m_mode & mode != 0 as ::core::ffi::c_int
                            {
                                if !has_lhs {
                                    showmap(mp_0, true_0 != 0);
                                    did_local = true_0 != 0;
                                } else {
                                    let mut n_0: ::core::ffi::c_int = (*mp_0).m_keylen;
                                    if strncmp(
                                        (*mp_0).m_keys,
                                        lhs,
                                        (if n_0 < len { n_0 } else { len }) as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        showmap(mp_0, true_0 != 0);
                                        did_local = true_0 != 0;
                                    }
                                }
                            }
                            mp_0 = (*mp_0).m_next;
                        }
                        hash_0 += 1;
                    }
                }
                let num_rounds: ::core::ffi::c_int =
                    if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !unmap_lhs_only {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while round < num_rounds && !did_it && !got_int {
                    let mut hash_start: ::core::ffi::c_int = 0;
                    let mut hash_end: ::core::ffi::c_int = 0;
                    if round == 0 as ::core::ffi::c_int && has_lhs as ::core::ffi::c_int != 0
                        || is_abbrev as ::core::ffi::c_int != 0
                    {
                        hash_start = if is_abbrev as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else if mode
                            & (MODE_NORMAL as ::core::ffi::c_int
                                | MODE_VISUAL as ::core::ffi::c_int
                                | MODE_SELECT as ::core::ffi::c_int
                                | MODE_OP_PENDING as ::core::ffi::c_int
                                | MODE_TERMINAL as ::core::ffi::c_int)
                            != 0
                        {
                            *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                        } else {
                            *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                ^ 0x80 as ::core::ffi::c_int
                        };
                        hash_end = hash_start + 1 as ::core::ffi::c_int;
                    } else {
                        hash_start = 0 as ::core::ffi::c_int;
                        hash_end = 256 as ::core::ffi::c_int;
                    }
                    let mut hash_1: ::core::ffi::c_int = hash_start;
                    while hash_1 < hash_end && !got_int {
                        let mut mpp: *mut *mut mapblock_T = if is_abbrev as ::core::ffi::c_int != 0
                        {
                            abbr_table
                        } else {
                            map_table.offset(hash_1 as isize)
                        };
                        let mut mp_1: *mut mapblock_T = *mpp;
                        's_448: while !mp_1.is_null() && !got_int {
                            's_458: {
                                if (*mp_1).m_mode & mode == 0 as ::core::ffi::c_int {
                                    mpp = &raw mut (*mp_1).m_next;
                                } else {
                                    if !has_lhs {
                                        if (*mp_1).m_simplified == 0 {
                                            showmap(
                                                mp_1,
                                                map_table
                                                    != &raw mut maphash as *mut *mut mapblock_T,
                                            );
                                            did_it = true_0 != 0;
                                        }
                                    } else {
                                        let mut n_1: ::core::ffi::c_int = 0;
                                        let mut p_0: *const ::core::ffi::c_char =
                                            ::core::ptr::null::<::core::ffi::c_char>();
                                        if round != 0 {
                                            n_1 = strlen((*mp_1).m_str) as ::core::ffi::c_int;
                                            p_0 = (*mp_1).m_str;
                                        } else {
                                            n_1 = (*mp_1).m_keylen;
                                            p_0 = (*mp_1).m_keys;
                                        }
                                        if strncmp(
                                            p_0,
                                            lhs,
                                            (if n_1 < len { n_1 } else { len }) as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                                                if n_1 != len
                                                    && (!is_abbrev
                                                        || round != 0
                                                        || n_1 > len
                                                        || *skipwhite(lhs.offset(n_1 as isize))
                                                            as ::core::ffi::c_int
                                                            != NUL)
                                                {
                                                    mpp = &raw mut (*mp_1).m_next;
                                                    break 's_458;
                                                } else {
                                                    if keyround1_simplified as ::core::ffi::c_int
                                                        != 0
                                                        && (*mp_1).m_simplified == 0
                                                    {
                                                        break 's_448;
                                                    }
                                                    (*mp_1).m_mode &= !mode;
                                                    did_it = true_0 != 0;
                                                }
                                            } else if !has_rhs {
                                                if (*mp_1).m_simplified == 0 {
                                                    showmap(
                                                        mp_1,
                                                        map_table
                                                            != &raw mut maphash
                                                                as *mut *mut mapblock_T,
                                                    );
                                                    did_it = true_0 != 0;
                                                }
                                            } else if n_1 != len {
                                                mpp = &raw mut (*mp_1).m_next;
                                                break 's_458;
                                            } else if keyround1_simplified as ::core::ffi::c_int
                                                != 0
                                                && (*mp_1).m_simplified == 0
                                            {
                                                did_it = true_0 != 0;
                                                break 's_448;
                                            } else if (*args).unique {
                                                retval = 5 as ::core::ffi::c_int;
                                                break '_theend;
                                            } else {
                                                (*mp_1).m_mode &= !mode;
                                                if (*mp_1).m_mode == 0 as ::core::ffi::c_int
                                                    && !did_it
                                                {
                                                    if !(*mp_1).m_alt.is_null() {
                                                        (*(*mp_1).m_alt).m_alt =
                                                            ::core::ptr::null_mut::<mapblock_T>();
                                                        (*mp_1).m_alt = (*(*mp_1).m_alt).m_alt;
                                                    } else {
                                                        if (*mp_1).m_luaref != LUA_NOREF {
                                                            api_free_luaref((*mp_1).m_luaref);
                                                            (*mp_1).m_luaref = LUA_NOREF as LuaRef;
                                                        }
                                                        xfree(
                                                            (*mp_1).m_str
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                        xfree(
                                                            (*mp_1).m_orig_str
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                        xfree(
                                                            (*mp_1).m_desc
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                    }
                                                    (*mp_1).m_str = (*args).rhs;
                                                    (*mp_1).m_orig_str = (*args).orig_rhs;
                                                    (*mp_1).m_luaref = (*args).rhs_lua;
                                                    (*mp_1).m_noremap = noremap;
                                                    (*mp_1).m_nowait =
                                                        (*args).nowait as ::core::ffi::c_char;
                                                    (*mp_1).m_silent =
                                                        (*args).silent as ::core::ffi::c_char;
                                                    (*mp_1).m_mode = mode;
                                                    (*mp_1).m_simplified =
                                                        keyround1_simplified as ::core::ffi::c_int;
                                                    (*mp_1).m_expr =
                                                        (*args).expr as ::core::ffi::c_char;
                                                    (*mp_1).m_replace_keycodes =
                                                        (*args).replace_keycodes;
                                                    (*mp_1).m_script_ctx = current_sctx;
                                                    (*mp_1).m_script_ctx.sc_lnum +=
                                                        (*(exestack.ga_data as *mut estack_T)
                                                            .offset(
                                                                (exestack.ga_len
                                                                    - 1 as ::core::ffi::c_int)
                                                                    as isize,
                                                            ))
                                                        .es_lnum;
                                                    nlua_set_sctx(&raw mut (*mp_1).m_script_ctx);
                                                    (*mp_1).m_desc = (*args).desc;
                                                    mp_result[(keyround - 1 as ::core::ffi::c_int)
                                                        as usize] = mp_1;
                                                    did_it = true_0 != 0;
                                                }
                                            }
                                            if (*mp_1).m_mode == 0 as ::core::ffi::c_int {
                                                mapblock_free(mpp);
                                                break 's_458;
                                            } else {
                                                let mut new_hash: ::core::ffi::c_int = if (*mp_1)
                                                    .m_mode
                                                    & (MODE_NORMAL as ::core::ffi::c_int
                                                        | MODE_VISUAL as ::core::ffi::c_int
                                                        | MODE_SELECT as ::core::ffi::c_int
                                                        | MODE_OP_PENDING as ::core::ffi::c_int
                                                        | MODE_TERMINAL as ::core::ffi::c_int)
                                                    != 0
                                                {
                                                    *(*mp_1)
                                                        .m_keys
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as uint8_t
                                                        as ::core::ffi::c_int
                                                } else {
                                                    *(*mp_1)
                                                        .m_keys
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as uint8_t
                                                        as ::core::ffi::c_int
                                                        ^ 0x80 as ::core::ffi::c_int
                                                };
                                                if !is_abbrev && new_hash != hash_1 {
                                                    *mpp = (*mp_1).m_next;
                                                    (*mp_1).m_next =
                                                        *map_table.offset(new_hash as isize);
                                                    *map_table.offset(new_hash as isize) = mp_1;
                                                    break 's_458;
                                                }
                                            }
                                        }
                                    }
                                    mpp = &raw mut (*mp_1).m_next;
                                }
                            }
                            mp_1 = *mpp;
                        }
                        hash_1 += 1;
                    }
                    round += 1;
                }
                if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                    if !did_it {
                        if !keyround1_simplified {
                            retval = 2 as ::core::ffi::c_int;
                        }
                    } else if *lhs as ::core::ffi::c_int == Ctrl_C {
                        if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
                            (*buf).b_mapped_ctrl_c &= !mode;
                        } else {
                            mapped_ctrl_c &= !mode;
                        }
                    }
                } else if !has_lhs || !has_rhs {
                    if !did_it && !did_local {
                        if is_abbrev {
                            msg(
                                gettext(b"No abbreviation found\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                0 as ::core::ffi::c_int,
                            );
                        } else {
                            msg(
                                gettext(
                                    b"No mapping found\0".as_ptr() as *const ::core::ffi::c_char
                                ),
                                0 as ::core::ffi::c_int,
                            );
                        }
                    }
                    break '_theend;
                } else if !did_it {
                    mp_result[(keyround - 1 as ::core::ffi::c_int) as usize] = map_add(
                        buf,
                        map_table,
                        abbr_table,
                        lhs,
                        args,
                        noremap,
                        mode,
                        is_abbrev,
                        0 as scid_T,
                        0 as linenr_T,
                        keyround1_simplified,
                    );
                }
                keyround += 1;
            }
            if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
                && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
            {
                (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
                    mp_result[1 as ::core::ffi::c_int as usize];
                (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
                    mp_result[0 as ::core::ffi::c_int as usize];
            }
        }
    }
    if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
        || !mp_result[1 as ::core::ffi::c_int as usize].is_null()
    {
        (*args).rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*args).orig_rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*args).rhs_lua = LUA_NOREF as LuaRef;
        (*args).desc = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn do_map(
    mut maptype: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
) -> ::core::ffi::c_int {
    let mut parsed_args: MapArguments = MapArguments {
        buffer: false,
        expr: false,
        noremap: false,
        nowait: false,
        script: false,
        silent: false,
        unique: false,
        replace_keycodes: false,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: ::core::ffi::c_int = str_to_mapargs(
        arg,
        maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
        &raw mut parsed_args,
    );
    match result {
        0 => {
            result = buf_do_map(maptype, &raw mut parsed_args, mode, is_abbrev, curbuf);
        }
        1 => {}
        _ => {
            '_c2rust_label: {
                if false
                    && !(b"Unknown return code from str_to_mapargs!\0".as_ptr()
                        as *const ::core::ffi::c_char)
                        .is_null()
                {
                } else {
                    __assert_fail(
                        b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        968 as ::core::ffi::c_uint,
                        b"int do_map(int, char *, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            result = -1 as ::core::ffi::c_int;
        }
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
    return result;
}
unsafe extern "C" fn get_map_mode(
    mut cmdp: *mut *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    let mut mode: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = *cmdp;
    let c2rust_fresh11 = p;
    p = p.offset(1);
    let mut modec: ::core::ffi::c_int = *c2rust_fresh11 as uint8_t as ::core::ffi::c_int;
    if modec == 'i' as ::core::ffi::c_int {
        mode = MODE_INSERT as ::core::ffi::c_int;
    } else if modec == 'l' as ::core::ffi::c_int {
        mode = MODE_LANGMAP as ::core::ffi::c_int;
    } else if modec == 'c' as ::core::ffi::c_int {
        mode = MODE_CMDLINE as ::core::ffi::c_int;
    } else if modec == 'n' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
    {
        mode = MODE_NORMAL as ::core::ffi::c_int;
    } else if modec == 'v' as ::core::ffi::c_int {
        mode = MODE_VISUAL as ::core::ffi::c_int | MODE_SELECT as ::core::ffi::c_int;
    } else if modec == 'x' as ::core::ffi::c_int {
        mode = MODE_VISUAL as ::core::ffi::c_int;
    } else if modec == 's' as ::core::ffi::c_int {
        mode = MODE_SELECT as ::core::ffi::c_int;
    } else if modec == 'o' as ::core::ffi::c_int {
        mode = MODE_OP_PENDING as ::core::ffi::c_int;
    } else if modec == 't' as ::core::ffi::c_int {
        mode = MODE_TERMINAL as ::core::ffi::c_int;
    } else {
        p = p.offset(-1);
        if forceit {
            mode = MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int;
        } else {
            mode = MODE_VISUAL as ::core::ffi::c_int
                | MODE_SELECT as ::core::ffi::c_int
                | MODE_NORMAL as ::core::ffi::c_int
                | MODE_OP_PENDING as ::core::ffi::c_int;
        }
    }
    *cmdp = p;
    return mode;
}
unsafe extern "C" fn do_mapclear(
    mut cmdp: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
) {
    let mut local: bool = strcmp(arg, b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int;
    if !local && *arg as ::core::ffi::c_int != NUL {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut mode: ::core::ffi::c_int = get_map_mode(&raw mut cmdp, forceit != 0);
    map_clear_mode(curbuf, mode, local, abbr != 0);
}
#[no_mangle]
pub unsafe extern "C" fn map_clear_mode(
    mut buf: *mut buf_T,
    mut mode: ::core::ffi::c_int,
    mut local: bool,
    mut abbr: bool,
) {
    let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while hash < 256 as ::core::ffi::c_int {
        let mut mpp: *mut *mut mapblock_T = ::core::ptr::null_mut::<*mut mapblock_T>();
        if abbr {
            if hash > 0 as ::core::ffi::c_int {
                break;
            }
            if local {
                mpp = &raw mut (*buf).b_first_abbr;
            } else {
                mpp = &raw mut first_abbr;
            }
        } else if local {
            mpp = (&raw mut (*buf).b_maphash as *mut *mut mapblock_T).offset(hash as isize)
                as *mut *mut mapblock_T;
        } else {
            mpp = (&raw mut maphash as *mut *mut mapblock_T).offset(hash as isize)
                as *mut *mut mapblock_T;
        }
        while !(*mpp).is_null() {
            let mut mp: *mut mapblock_T = *mpp;
            if (*mp).m_mode & mode != 0 {
                (*mp).m_mode &= !mode;
                if (*mp).m_mode == 0 as ::core::ffi::c_int {
                    mapblock_free(mpp);
                    continue;
                } else {
                    let mut new_hash: ::core::ffi::c_int = if (*mp).m_mode
                        & (MODE_NORMAL as ::core::ffi::c_int
                            | MODE_VISUAL as ::core::ffi::c_int
                            | MODE_SELECT as ::core::ffi::c_int
                            | MODE_OP_PENDING as ::core::ffi::c_int
                            | MODE_TERMINAL as ::core::ffi::c_int)
                        != 0
                    {
                        *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                    } else {
                        *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            ^ 0x80 as ::core::ffi::c_int
                    };
                    if !abbr && new_hash != hash {
                        *mpp = (*mp).m_next;
                        if local {
                            (*mp).m_next = (*buf).b_maphash[new_hash as usize] as *mut mapblock_T;
                            (*buf).b_maphash[new_hash as usize] = mp as *mut mapblock_T;
                        } else {
                            (*mp).m_next = maphash[new_hash as usize] as *mut mapblock_T;
                            maphash[new_hash as usize] = mp as *mut mapblock_T;
                        }
                        continue;
                    }
                }
            }
            mpp = &raw mut (*mp).m_next;
        }
        hash += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn map_to_exists(
    str: *const ::core::ffi::c_char,
    modechars: *const ::core::ffi::c_char,
    abbr: bool,
) -> bool {
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let rhs: *const ::core::ffi::c_char = replace_termcodes(
        str,
        strlen(str),
        &raw mut buf,
        0 as scid_T,
        REPTERM_DO_LT as ::core::ffi::c_int,
        ::core::ptr::null_mut::<bool>(),
        p_cpo,
    );
    if !strchr(modechars, 'n' as ::core::ffi::c_int).is_null() {
        mode |= MODE_NORMAL as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'v' as ::core::ffi::c_int).is_null() {
        mode |= MODE_VISUAL as ::core::ffi::c_int | MODE_SELECT as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'x' as ::core::ffi::c_int).is_null() {
        mode |= MODE_VISUAL as ::core::ffi::c_int;
    }
    if !strchr(modechars, 's' as ::core::ffi::c_int).is_null() {
        mode |= MODE_SELECT as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'o' as ::core::ffi::c_int).is_null() {
        mode |= MODE_OP_PENDING as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'i' as ::core::ffi::c_int).is_null() {
        mode |= MODE_INSERT as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'l' as ::core::ffi::c_int).is_null() {
        mode |= MODE_LANGMAP as ::core::ffi::c_int;
    }
    if !strchr(modechars, 'c' as ::core::ffi::c_int).is_null() {
        mode |= MODE_CMDLINE as ::core::ffi::c_int;
    }
    let mut retval: bool = map_to_exists_mode(rhs, mode, abbr);
    xfree(buf as *mut ::core::ffi::c_void);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn map_to_exists_mode(
    rhs: *const ::core::ffi::c_char,
    mode: ::core::ffi::c_int,
    abbr: bool,
) -> bool {
    let mut exp_buffer: bool = false_0 != 0;
    loop {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if exp_buffer {
                    mp = (*curbuf).b_first_abbr;
                } else {
                    mp = first_abbr;
                }
            } else if exp_buffer {
                mp = (*curbuf).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = maphash[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_mode & mode != 0 && !strstr((*mp).m_str, rhs).is_null() {
                    return true_0 != 0;
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        if exp_buffer {
            break;
        }
        exp_buffer = true_0 != 0;
    }
    return false_0 != 0;
}
static mut expand_mapmodes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut expand_isabbrev: bool = false_0 != 0;
static mut expand_buffer: bool = false_0 != 0;
unsafe extern "C" fn translate_mapping(
    str_in: *const ::core::ffi::c_char,
    cpo_val: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut str: *const uint8_t = str_in as *const uint8_t;
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
    let cpo_bslash: bool = !vim_strchr(cpo_val, CPO_BSLASH).is_null();
    while *str != 0 {
        let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
        's_13: {
            if c == K_SPECIAL
                && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == KS_MODIFIER
                {
                    str = str.offset(1);
                    str = str.offset(1);
                    modifiers = *str as ::core::ffi::c_int;
                    str = str.offset(1);
                    c = *str as ::core::ffi::c_int;
                }
                if c == K_SPECIAL
                    && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    c = if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_SPECIAL
                    {
                        K_SPECIAL
                    } else if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_ZERO
                    {
                        K_ZERO
                    } else {
                        -(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int))
                    };
                    if c == K_ZERO {
                        c = NUL;
                    }
                    str = str.offset(2 as ::core::ffi::c_int as isize);
                }
                if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                    ga_concat(&raw mut ga, get_special_key_name(c, modifiers));
                    break 's_13;
                }
            }
            if c == ' ' as ::core::ffi::c_int
                || c == '\t' as ::core::ffi::c_int
                || c == Ctrl_J
                || c == Ctrl_V
                || c == '<' as ::core::ffi::c_int
                || c == '\\' as ::core::ffi::c_int && !cpo_bslash
            {
                ga_append(
                    &raw mut ga,
                    (if cpo_bslash as ::core::ffi::c_int != 0 {
                        Ctrl_V
                    } else {
                        '\\' as ::core::ffi::c_int
                    }) as uint8_t,
                );
            }
            if c != 0 {
                ga_append(&raw mut ga, c as uint8_t);
            }
        }
        str = str.offset(1);
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_map_cmd(
    mut xp: *mut expand_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
    mut isabbrev: bool,
    mut isunmap: bool,
    mut cmdidx: cmdidx_T,
) -> *mut ::core::ffi::c_char {
    if forceit as ::core::ffi::c_int != 0
        && cmdidx as ::core::ffi::c_int != CMD_map as ::core::ffi::c_int
        && cmdidx as ::core::ffi::c_int != CMD_unmap as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    } else {
        if isunmap {
            expand_mapmodes = get_map_mode(
                &raw mut cmd,
                forceit as ::core::ffi::c_int != 0 || isabbrev as ::core::ffi::c_int != 0,
            );
        } else {
            expand_mapmodes =
                MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int;
            if !isabbrev {
                expand_mapmodes |= MODE_VISUAL as ::core::ffi::c_int
                    | MODE_SELECT as ::core::ffi::c_int
                    | MODE_NORMAL as ::core::ffi::c_int
                    | MODE_OP_PENDING as ::core::ffi::c_int;
            }
        }
        expand_isabbrev = isabbrev;
        (*xp).xp_context = EXPAND_MAPPINGS as ::core::ffi::c_int;
        expand_buffer = false_0 != 0;
        loop {
            if strncmp(
                arg,
                b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                expand_buffer = true_0 != 0;
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else {
                if strncmp(
                    arg,
                    b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
                arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
            }
        }
        (*xp).xp_pattern = arg;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn ExpandMappings(
    mut pat: *mut ::core::ffi::c_char,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let fuzzy: bool = cmdline_fuzzy_complete(pat);
    *numMatches = 0 as ::core::ffi::c_int;
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
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
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 7 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        's_34: {
            if i == 0 as ::core::ffi::c_int {
                p = b"<silent>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 1 as ::core::ffi::c_int {
                p = b"<unique>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 2 as ::core::ffi::c_int {
                p = b"<script>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 3 as ::core::ffi::c_int {
                p = b"<expr>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if i == 4 as ::core::ffi::c_int && !expand_buffer {
                p = b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 5 as ::core::ffi::c_int {
                p = b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 6 as ::core::ffi::c_int {
                p = b"<special>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                break 's_34;
            }
            let mut match_0: bool = false;
            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if !fuzzy {
                match_0 = vim_regexec(regmatch, p, 0 as colnr_T);
            } else {
                score = fuzzy_match_str(p, pat);
                match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
            }
            if match_0 {
                if fuzzy {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                        fuzmatch_str_T {
                            idx: ga.ga_len,
                            str: xstrdup(p),
                            score: score,
                        };
                    ga.ga_len += 1;
                } else {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                        xstrdup(p);
                    ga.ga_len += 1;
                }
            }
        }
        i += 1;
    }
    let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while hash < 256 as ::core::ffi::c_int {
        let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
        if expand_isabbrev {
            if hash > 0 as ::core::ffi::c_int {
                break;
            } else {
                mp = first_abbr;
            }
        } else if expand_buffer {
            mp = (*curbuf).b_maphash[hash as usize] as *mut mapblock_T;
        } else {
            mp = maphash[hash as usize] as *mut mapblock_T;
        }
        while !mp.is_null() {
            if !((*mp).m_simplified != 0 || (*mp).m_mode & expand_mapmodes == 0) {
                let mut p_0: *mut ::core::ffi::c_char = translate_mapping((*mp).m_keys, p_cpo);
                if !p_0.is_null() {
                    let mut match_1: bool = false;
                    let mut score_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if !fuzzy {
                        match_1 = vim_regexec(regmatch, p_0, 0 as colnr_T);
                    } else {
                        score_0 = fuzzy_match_str(p_0, pat);
                        match_1 = score_0 != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                    }
                    if !match_1 {
                        xfree(p_0 as *mut ::core::ffi::c_void);
                    } else if fuzzy {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                            fuzmatch_str_T {
                                idx: ga.ga_len,
                                str: p_0,
                                score: score_0,
                            };
                        ga.ga_len += 1;
                    } else {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                            p_0;
                        ga.ga_len += 1;
                    }
                }
            }
            mp = (*mp).m_next;
        }
        hash += 1;
    }
    if ga.ga_len == 0 as ::core::ffi::c_int {
        return FAIL;
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
    let mut count: ::core::ffi::c_int = *numMatches;
    if count > 1 as ::core::ffi::c_int {
        if !fuzzy {
            sort_strings(*matches, count);
        }
        let mut ptr1: *mut *mut ::core::ffi::c_char = *matches;
        let mut ptr2: *mut *mut ::core::ffi::c_char = ptr1.offset(1 as ::core::ffi::c_int as isize);
        let mut ptr3: *mut *mut ::core::ffi::c_char = ptr1.offset(count as isize);
        while ptr2 < ptr3 {
            if strcmp(*ptr1, *ptr2) != 0 as ::core::ffi::c_int {
                let c2rust_fresh12 = ptr2;
                ptr2 = ptr2.offset(1);
                ptr1 = ptr1.offset(1);
                let c2rust_lvalue_ptr = &raw mut *ptr1;
                *c2rust_lvalue_ptr = *c2rust_fresh12;
            } else {
                let c2rust_fresh13 = ptr2;
                ptr2 = ptr2.offset(1);
                xfree(*c2rust_fresh13 as *mut ::core::ffi::c_void);
                count -= 1;
            }
        }
    }
    *numMatches = count;
    return if count == 0 as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    };
}
#[no_mangle]
pub unsafe extern "C" fn check_abbr(
    mut c: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut mincol: ::core::ffi::c_int,
) -> bool {
    let mut tb: [uint8_t; 25] = [0; 25];
    let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if typebuf.tb_no_abbr_cnt != 0 {
        return false_0 != 0;
    }
    if noremap_keys() as ::core::ffi::c_int != 0 && c != Ctrl_RSB {
        return false_0 != 0;
    }
    if col == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut scol: ::core::ffi::c_int = 0;
    let mut is_id: bool = true_0 != 0;
    let mut vim_abbr: bool = false;
    let mut p: *mut ::core::ffi::c_char = mb_prevptr(ptr, ptr.offset(col as isize));
    if !vim_iswordp(p) {
        vim_abbr = true_0 != 0;
    } else {
        vim_abbr = false_0 != 0;
        if p > ptr {
            is_id = vim_iswordp(mb_prevptr(ptr, p));
        }
    }
    clen = 1 as ::core::ffi::c_int;
    while p > ptr.offset(mincol as isize) {
        p = mb_prevptr(ptr, p);
        if ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || !vim_abbr && is_id as ::core::ffi::c_int != vim_iswordp(p) as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
            break;
        } else {
            clen += 1;
        }
    }
    scol = p.offset_from(ptr) as ::core::ffi::c_int;
    if scol < mincol {
        scol = mincol;
    }
    if scol < col {
        ptr = ptr.offset(scol as isize);
        let mut len: ::core::ffi::c_int = col - scol;
        let mut mp: *mut mapblock_T = (*curbuf).b_first_abbr;
        let mut mp2: *mut mapblock_T = first_abbr;
        if mp.is_null() {
            mp = mp2;
            mp2 = ::core::ptr::null_mut::<mapblock_T>();
        }
        while !mp.is_null() {
            let mut qlen: ::core::ffi::c_int = (*mp).m_keylen;
            let mut q: *mut ::core::ffi::c_char = (*mp).m_keys;
            if !strchr((*mp).m_keys, K_SPECIAL).is_null() {
                q = xstrdup((*mp).m_keys);
                vim_unescape_ks(q);
                qlen = strlen(q) as ::core::ffi::c_int;
            }
            let mut match_0: ::core::ffi::c_int =
                ((*mp).m_mode & State != 0 && qlen == len && strncmp(q, ptr, len as size_t) == 0)
                    as ::core::ffi::c_int;
            if q != (*mp).m_keys {
                xfree(q as *mut ::core::ffi::c_void);
            }
            if match_0 != 0 {
                break;
            }
            if (*mp).m_next.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            } else {
                mp = (*mp).m_next;
            };
        }
        if !mp.is_null() {
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if c != Ctrl_RSB {
                if c < 0 as ::core::ffi::c_int || c == K_SPECIAL {
                    let c2rust_fresh14 = j;
                    j = j + 1;
                    tb[c2rust_fresh14 as usize] = K_SPECIAL as uint8_t;
                    let c2rust_fresh15 = j;
                    j = j + 1;
                    tb[c2rust_fresh15 as usize] = (if c == K_SPECIAL {
                        KS_SPECIAL
                    } else if c == NUL {
                        KS_ZERO
                    } else {
                        -c & 0xff as ::core::ffi::c_int
                    }) as uint8_t;
                    let c2rust_fresh16 = j;
                    j = j + 1;
                    tb[c2rust_fresh16 as usize] = (if c == K_SPECIAL || c == NUL {
                        KE_FILLER as ::core::ffi::c_uint
                    } else {
                        -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_uint
                    }) as uint8_t;
                } else {
                    if c < ABBR_OFF
                        && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
                    {
                        let c2rust_fresh17 = j;
                        j = j + 1;
                        tb[c2rust_fresh17 as usize] = Ctrl_V as uint8_t;
                    }
                    if c >= ABBR_OFF {
                        c -= ABBR_OFF;
                    }
                    let mut newlen: ::core::ffi::c_int = utf_char2bytes(
                        c,
                        (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                            .offset(j as isize),
                    );
                    tb[(j + newlen) as usize] = NUL as uint8_t;
                    let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(
                        (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                            .offset(j as isize),
                    );
                    if !escaped.is_null() {
                        newlen = strlen(escaped) as ::core::ffi::c_int;
                        memmove(
                            (&raw mut tb as *mut uint8_t).offset(j as isize)
                                as *mut ::core::ffi::c_void,
                            escaped as *const ::core::ffi::c_void,
                            newlen as size_t,
                        );
                        j += newlen;
                        xfree(escaped as *mut ::core::ffi::c_void);
                    }
                }
                tb[j as usize] = NUL as uint8_t;
                ins_typebuf(
                    &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    (*mp).m_silent != 0,
                );
            }
            let noremap: ::core::ffi::c_int = (*mp).m_noremap;
            let silent: bool = (*mp).m_silent != 0;
            let expr: bool = (*mp).m_expr != 0;
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if expr {
                s = eval_map_expr(mp, c);
            } else {
                s = (*mp).m_str;
            }
            if !s.is_null() {
                ins_typebuf(s, noremap, 0 as ::core::ffi::c_int, true_0 != 0, silent);
                typebuf.tb_no_abbr_cnt +=
                    strlen(s) as ::core::ffi::c_int + j + 1 as ::core::ffi::c_int;
                if expr {
                    xfree(s as *mut ::core::ffi::c_void);
                }
            }
            tb[0 as ::core::ffi::c_int as usize] = Ctrl_H as uint8_t;
            tb[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
            len = clen;
            loop {
                let c2rust_fresh18 = len;
                len = len - 1;
                if c2rust_fresh18 <= 0 as ::core::ffi::c_int {
                    break;
                }
                ins_typebuf(
                    &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent,
                );
            }
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn eval_map_expr(
    mut mp: *mut mapblock_T,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*mp).m_luaref == LUA_NOREF {
        expr = xstrdup((*mp).m_str);
        vim_unescape_ks(expr);
    }
    let replace_keycodes: bool = (*mp).m_replace_keycodes;
    expr_map_lock += 1;
    set_vim_var_char(c);
    let save_cursor: pos_T = (*curwin).w_cursor;
    let save_msg_col: ::core::ffi::c_int = msg_col;
    let save_msg_row: ::core::ffi::c_int = msg_row;
    if (*mp).m_luaref != LUA_NOREF {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut args: Array = ARRAY_DICT_INIT;
        let mut ret: Object = nlua_call_ref(
            (*mp).m_luaref,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            p = string_to_cstr(ret.data.string);
        }
        api_free_object(ret);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            semsg_multiline(
                b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
                b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
        }
    } else {
        p = eval_to_string(expr, false_0 != 0, false_0 != 0);
        xfree(expr as *mut ::core::ffi::c_void);
    }
    expr_map_lock -= 1;
    (*curwin).w_cursor = save_cursor;
    msg_col = save_msg_col;
    msg_row = save_msg_row;
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if replace_keycodes {
        replace_termcodes(
            p,
            strlen(p),
            &raw mut res,
            0 as scid_T,
            REPTERM_DO_LT as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            p_cpo,
        );
    } else {
        res = vim_strsave_escape_ks(p);
    }
    xfree(p as *mut ::core::ffi::c_void);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn makemap(mut fd: *mut FILE, mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut did_cpo: bool = false_0 != 0;
    let mut abbr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while abbr < 2 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr != 0 {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if !buf.is_null() {
                    mp = (*buf).b_first_abbr;
                } else {
                    mp = first_abbr;
                }
            } else if !buf.is_null() {
                mp = (*buf).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = maphash[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_noremap != REMAP_SCRIPT as ::core::ffi::c_int {
                    if (*mp).m_luaref == LUA_NOREF {
                        let mut p: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        p = (*mp).m_str;
                        while *p as ::core::ffi::c_int != NUL {
                            if *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                == K_SPECIAL
                                && *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    == KS_EXTRA
                                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    == KE_SNR as ::core::ffi::c_int
                            {
                                break;
                            }
                            p = p.offset(1);
                        }
                        if *p as ::core::ffi::c_int == NUL {
                            let mut c1: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut c2: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut c3: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut cmd: *mut ::core::ffi::c_char = (if abbr != 0 {
                                b"abbr\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"map\0".as_ptr() as *const ::core::ffi::c_char
                            })
                                as *mut ::core::ffi::c_char;
                            match (*mp).m_mode {
                                71 => {}
                                1 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                }
                                2 => {
                                    c1 = 'x' as ::core::ffi::c_char;
                                }
                                64 => {
                                    c1 = 's' as ::core::ffi::c_char;
                                }
                                4 => {
                                    c1 = 'o' as ::core::ffi::c_char;
                                }
                                3 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'x' as ::core::ffi::c_char;
                                }
                                65 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 's' as ::core::ffi::c_char;
                                }
                                5 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                66 => {
                                    c1 = 'v' as ::core::ffi::c_char;
                                }
                                6 => {
                                    c1 = 'x' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                68 => {
                                    c1 = 's' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                67 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'v' as ::core::ffi::c_char;
                                }
                                7 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'x' as ::core::ffi::c_char;
                                    c3 = 'o' as ::core::ffi::c_char;
                                }
                                69 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 's' as ::core::ffi::c_char;
                                    c3 = 'o' as ::core::ffi::c_char;
                                }
                                70 => {
                                    c1 = 'v' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                24 => {
                                    if abbr == 0 {
                                        cmd = b"map!\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char;
                                    }
                                }
                                8 => {
                                    c1 = 'c' as ::core::ffi::c_char;
                                }
                                16 => {
                                    c1 = 'i' as ::core::ffi::c_char;
                                }
                                32 => {
                                    c1 = 'l' as ::core::ffi::c_char;
                                }
                                128 => {
                                    c1 = 't' as ::core::ffi::c_char;
                                }
                                _ => {
                                    iemsg(gettext(b"E228: makemap: Illegal mode\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                    return FAIL;
                                }
                            }
                            loop {
                                if !did_cpo {
                                    if *(*mp).m_str as ::core::ffi::c_int == NUL {
                                        did_cpo = true_0 != 0;
                                    } else {
                                        let specials: [::core::ffi::c_char; 3] = [
                                            K_SPECIAL as uint8_t as ::core::ffi::c_char,
                                            NL as ::core::ffi::c_char,
                                            NUL as ::core::ffi::c_char,
                                        ];
                                        if !strpbrk(
                                            (*mp).m_str,
                                            &raw const specials as *const ::core::ffi::c_char,
                                        )
                                        .is_null()
                                            || !strpbrk(
                                                (*mp).m_keys,
                                                &raw const specials as *const ::core::ffi::c_char,
                                            )
                                            .is_null()
                                        {
                                            did_cpo = true_0 != 0;
                                        }
                                    }
                                    if did_cpo {
                                        if fprintf(
                                            fd,
                                            b"let s:cpo_save=&cpo\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ) < 0 as ::core::ffi::c_int
                                            || put_eol(fd) < 0 as ::core::ffi::c_int
                                            || fprintf(
                                                fd,
                                                b"set cpo&vim\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) < 0 as ::core::ffi::c_int
                                            || put_eol(fd) < 0 as ::core::ffi::c_int
                                        {
                                            return FAIL;
                                        }
                                    }
                                }
                                if c1 as ::core::ffi::c_int != 0
                                    && putc(c1 as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_noremap != REMAP_YES as ::core::ffi::c_int
                                    && fprintf(fd, b"nore\0".as_ptr() as *const ::core::ffi::c_char)
                                        < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if fputs(cmd, fd) < 0 as ::core::ffi::c_int {
                                    return FAIL;
                                }
                                if !buf.is_null()
                                    && fputs(
                                        b" <buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_nowait as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_silent as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <silent>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_expr as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <expr>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if putc(' ' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                    || put_escstr(fd, (*mp).m_keys, 0 as ::core::ffi::c_int) == FAIL
                                    || putc(' ' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                    || put_escstr(fd, (*mp).m_str, 1 as ::core::ffi::c_int) == FAIL
                                    || put_eol(fd) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                c1 = c2;
                                c2 = c3;
                                c3 = NUL as ::core::ffi::c_char;
                                if c1 as ::core::ffi::c_int == NUL {
                                    break;
                                }
                            }
                        }
                    }
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        abbr += 1;
    }
    if did_cpo {
        if fprintf(
            fd,
            b"let &cpo=s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
        ) < 0 as ::core::ffi::c_int
            || put_eol(fd) < 0 as ::core::ffi::c_int
            || fprintf(
                fd,
                b"unlet s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            || put_eol(fd) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn put_escstr(
    mut fd: *mut FILE,
    mut strstart: *const ::core::ffi::c_char,
    mut what: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut str: *mut uint8_t = strstart as *mut uint8_t;
    if *str as ::core::ffi::c_int == NUL && what == 1 as ::core::ffi::c_int {
        if fprintf(fd, b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        return OK;
    }
    while *str as ::core::ffi::c_int != NUL {
        let mut p: *const ::core::ffi::c_char =
            mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
        's_26: {
            if !p.is_null() {
                while *p as ::core::ffi::c_int != NUL {
                    let c2rust_fresh19 = p;
                    p = p.offset(1);
                    if fputc(*c2rust_fresh19 as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
                str = str.offset(-1);
            } else {
                let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
                if c == K_SPECIAL && what != 2 as ::core::ffi::c_int {
                    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_MODIFIER
                    {
                        modifiers =
                            *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                        str = str.offset(3 as ::core::ffi::c_int as isize);
                        p = mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
                        if p.is_null() {
                            c = *str as ::core::ffi::c_int;
                        } else {
                            c = utf_ptr2char(p);
                            str = str.offset(-1);
                        }
                    }
                    if c == K_SPECIAL {
                        c = if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == KS_SPECIAL
                        {
                            K_SPECIAL
                        } else if *str.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == KS_ZERO
                        {
                            K_ZERO
                        } else {
                            -(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int))
                        };
                        str = str.offset(2 as ::core::ffi::c_int as isize);
                    }
                    if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                        if fputs(get_special_key_name(c, modifiers), fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                        break 's_26;
                    }
                }
                if c == NL {
                    if what == 2 as ::core::ffi::c_int {
                        if fprintf(fd, b"\\\x16\n\0".as_ptr() as *const ::core::ffi::c_char)
                            < 0 as ::core::ffi::c_int
                        {
                            return FAIL;
                        }
                    } else if fprintf(fd, b"<NL>\0".as_ptr() as *const ::core::ffi::c_char)
                        < 0 as ::core::ffi::c_int
                    {
                        return FAIL;
                    }
                } else {
                    if what == 2 as ::core::ffi::c_int
                        && (ascii_iswhite(c) as ::core::ffi::c_int != 0
                            || c == '"' as ::core::ffi::c_int
                            || c == '\\' as ::core::ffi::c_int)
                    {
                        if putc('\\' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                    } else if c < ' ' as ::core::ffi::c_int
                        || c > '~' as ::core::ffi::c_int
                        || c == '|' as ::core::ffi::c_int
                        || what == 0 as ::core::ffi::c_int && c == ' ' as ::core::ffi::c_int
                        || what == 1 as ::core::ffi::c_int
                            && str == strstart as *mut uint8_t
                            && c == ' ' as ::core::ffi::c_int
                        || what != 2 as ::core::ffi::c_int && c == '<' as ::core::ffi::c_int
                    {
                        if putc(Ctrl_V, fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                    }
                    if putc(c, fd) < 0 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
            }
        }
        str = str.offset(1);
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_map(
    mut keys: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut exact: ::core::ffi::c_int,
    mut ign_mod: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
    mut mp_ptr: *mut *mut mapblock_T,
    mut local_ptr: *mut ::core::ffi::c_int,
    mut rhs_lua: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    *rhs_lua = LUA_NOREF;
    let mut len: ::core::ffi::c_int = strlen(keys) as ::core::ffi::c_int;
    let mut local: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while local >= 0 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr != 0 {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if local != 0 {
                    mp = (*curbuf).b_first_abbr;
                } else {
                    mp = first_abbr;
                }
            } else if local != 0 {
                mp = (*curbuf).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = maphash[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_mode & mode != 0 && (exact == 0 || (*mp).m_keylen == len) {
                    let mut s: *mut ::core::ffi::c_char = (*mp).m_keys;
                    let mut keylen: ::core::ffi::c_int = (*mp).m_keylen;
                    if ign_mod != 0
                        && keylen >= 3 as ::core::ffi::c_int
                        && *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            == K_SPECIAL
                        && *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            == KS_MODIFIER
                    {
                        s = s.offset(3 as ::core::ffi::c_int as isize);
                        keylen -= 3 as ::core::ffi::c_int;
                    }
                    let mut minlen: ::core::ffi::c_int = if keylen < len { keylen } else { len };
                    if strncmp(s, keys, minlen as size_t) == 0 as ::core::ffi::c_int {
                        if !mp_ptr.is_null() {
                            *mp_ptr = mp;
                        }
                        if !local_ptr.is_null() {
                            *local_ptr = local;
                        }
                        *rhs_lua = (*mp).m_luaref as ::core::ffi::c_int;
                        return if (*mp).m_luaref == LUA_NOREF {
                            (*mp).m_str
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        };
                    }
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        local -= 1;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn f_hasmapto(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let name: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut abbr: bool = false_0 != 0;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        mode = b"nvo\0".as_ptr() as *const ::core::ffi::c_char;
    } else {
        mode = tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
        }
    }
    (*rettv).vval.v_number = map_to_exists(name, mode, abbr) as varnumber_T;
}
unsafe extern "C" fn mapblock_fill_dict(
    mp: *const mapblock_T,
    mut lhsrawalt: *const ::core::ffi::c_char,
    buffer_value: ::core::ffi::c_int,
    abbr: bool,
    compatible: bool,
    mut arena: *mut Arena,
) -> Dict {
    let mut dict: Dict = arena_dict(arena, 20 as size_t);
    let lhs: *mut ::core::ffi::c_char =
        str2special_arena((*mp).m_keys, compatible, !compatible, arena);
    let mut mapmode: *mut ::core::ffi::c_char =
        arena_alloc(arena, 7 as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    map_mode_to_chars((*mp).m_mode, mapmode);
    let mut noremap_value: ::core::ffi::c_int = 0;
    if compatible {
        noremap_value = ((*mp).m_noremap != 0) as ::core::ffi::c_int;
    } else {
        noremap_value = if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            ((*mp).m_noremap != 0) as ::core::ffi::c_int
        };
    }
    if (*mp).m_luaref != LUA_NOREF {
        let c2rust_fresh21 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"callback\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeLuaRef,
                data: C2Rust_Unnamed {
                    luaref: api_new_luaref((*mp).m_luaref),
                },
            },
        };
    } else {
        let mut rhs: String_0 = cstr_as_string(if compatible as ::core::ffi::c_int != 0 {
            (*mp).m_orig_str
        } else {
            str2special_arena((*mp).m_str, false_0 != 0, true_0 != 0, arena)
        });
        let c2rust_fresh22 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"rhs\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: rhs },
            },
        };
    }
    if !(*mp).m_desc.is_null() {
        let c2rust_fresh23 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"desc\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*mp).m_desc),
                },
            },
        };
    }
    let c2rust_fresh24 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"lhs\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(lhs),
            },
        },
    };
    let c2rust_fresh25 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string((*mp).m_keys),
            },
        },
    };
    if !lhsrawalt.is_null() {
        let c2rust_fresh26 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(lhsrawalt),
                },
            },
        };
    }
    let c2rust_fresh27 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh27 as isize) = key_value_pair {
        key: cstr_as_string(b"noremap\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: noremap_value as Integer,
            },
        },
    };
    let c2rust_fresh28 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh28 as isize) = key_value_pair {
        key: cstr_as_string(b"script\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh29 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh29 as isize) = key_value_pair {
        key: cstr_as_string(b"expr\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_expr as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh30 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh30 as isize) = key_value_pair {
        key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_silent as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh31 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh31 as isize) = key_value_pair {
        key: cstr_as_string(b"sid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_script_ctx.sc_sid as Integer,
            },
        },
    };
    let c2rust_fresh32 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh32 as isize) = key_value_pair {
        key: cstr_as_string(b"scriptversion\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 1 as Integer,
            },
        },
    };
    let c2rust_fresh33 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh33 as isize) = key_value_pair {
        key: cstr_as_string(b"lnum\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_script_ctx.sc_lnum as Integer,
            },
        },
    };
    let c2rust_fresh34 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh34 as isize) = key_value_pair {
        key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: buffer_value as Integer,
            },
        },
    };
    if !compatible {
        let c2rust_fresh35 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh35 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buffer_value as Integer,
                },
            },
        };
    }
    let c2rust_fresh36 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh36 as isize) = key_value_pair {
        key: cstr_as_string(b"nowait\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_nowait as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh37 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh37 as isize) = key_value_pair {
        key: cstr_as_string(b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_replace_keycodes as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh38 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh38 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(mapmode),
            },
        },
    };
    let c2rust_fresh39 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh39 as isize) = key_value_pair {
        key: cstr_as_string(b"abbr\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if abbr as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh40 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh40 as isize) = key_value_pair {
        key: cstr_as_string(b"mode_bits\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_mode as Integer,
            },
        },
    };
    return dict;
}
unsafe extern "C" fn get_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut exact: ::core::ffi::c_int,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keys: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    if *keys as ::core::ffi::c_int == NUL {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut abbr: bool = false_0 != 0;
    let mut get_dict: bool = false_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        which = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                get_dict = tv_get_number(argvars.offset(3 as ::core::ffi::c_int as isize)) != 0;
            }
        }
    } else {
        which = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if which.is_null() {
        return;
    }
    let mut keys_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut alt_keys_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut did_simplify: bool = false_0 != 0;
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let mode: ::core::ffi::c_int =
        get_map_mode(&raw mut which as *mut *mut ::core::ffi::c_char, false);
    let mut keys_simplified: *mut ::core::ffi::c_char = replace_termcodes(
        keys,
        strlen(keys),
        &raw mut keys_buf,
        0 as scid_T,
        flags,
        &raw mut did_simplify,
        p_cpo,
    );
    let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut buffer_local: ::core::ffi::c_int = 0;
    let mut rhs_lua: LuaRef = 0;
    let mut rhs: *mut ::core::ffi::c_char = check_map(
        keys_simplified,
        mode,
        exact,
        false_0,
        abbr as ::core::ffi::c_int,
        &raw mut mp,
        &raw mut buffer_local,
        &raw mut rhs_lua,
    );
    if did_simplify {
        replace_termcodes(
            keys,
            strlen(keys),
            &raw mut alt_keys_buf,
            0 as scid_T,
            flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            p_cpo,
        );
        rhs = check_map(
            alt_keys_buf,
            mode,
            exact,
            false_0,
            abbr as ::core::ffi::c_int,
            &raw mut mp,
            &raw mut buffer_local,
            &raw mut rhs_lua,
        );
    }
    if !get_dict {
        if !rhs.is_null() {
            if *rhs as ::core::ffi::c_int == NUL {
                (*rettv).vval.v_string = xstrdup(b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                (*rettv).vval.v_string = str2special_save(rhs, false_0 != 0, false_0 != 0);
            }
        } else if rhs_lua != LUA_NOREF {
            (*rettv).vval.v_string =
                nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
        }
    } else if !mp.is_null() && (!rhs.is_null() || rhs_lua != LUA_NOREF) {
        let mut arena: Arena = ARENA_EMPTY;
        let mut dict: Dict = mapblock_fill_dict(
            mp,
            if did_simplify as ::core::ffi::c_int != 0 {
                keys_simplified
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            },
            buffer_local,
            abbr,
            true_0 != 0,
            &raw mut arena,
        );
        let mut c2rust_lvalue: Object = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: dict },
        };
        object_to_vim_take_luaref(
            &raw mut c2rust_lvalue,
            rettv,
            true_0 != 0,
            ::core::ptr::null_mut::<Error>(),
        );
        arena_mem_free(arena_finish(&raw mut arena));
    } else {
        tv_dict_alloc_ret(rettv);
    }
    xfree(keys_buf as *mut ::core::ffi::c_void);
    xfree(alt_keys_buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_map_mode_string(
    mode_string: *const ::core::ffi::c_char,
    abbr: bool,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = mode_string;
    let MASK_V: ::core::ffi::c_int =
        MODE_VISUAL as ::core::ffi::c_int | MODE_SELECT as ::core::ffi::c_int;
    let MASK_MAP: ::core::ffi::c_int = MODE_VISUAL as ::core::ffi::c_int
        | MODE_SELECT as ::core::ffi::c_int
        | MODE_NORMAL as ::core::ffi::c_int
        | MODE_OP_PENDING as ::core::ffi::c_int;
    let MASK_BANG: ::core::ffi::c_int =
        MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == NUL {
        p = b" \0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut modec: ::core::ffi::c_int = 0;
    loop {
        let c2rust_fresh20 = p;
        p = p.offset(1);
        modec = *c2rust_fresh20 as uint8_t as ::core::ffi::c_int;
        if modec == 0 {
            break;
        }
        let mut tmode: ::core::ffi::c_int = 0;
        match modec {
            105 => {
                tmode = MODE_INSERT as ::core::ffi::c_int;
            }
            108 => {
                tmode = MODE_LANGMAP as ::core::ffi::c_int;
            }
            99 => {
                tmode = MODE_CMDLINE as ::core::ffi::c_int;
            }
            110 => {
                tmode = MODE_NORMAL as ::core::ffi::c_int;
            }
            120 => {
                tmode = MODE_VISUAL as ::core::ffi::c_int;
            }
            115 => {
                tmode = MODE_SELECT as ::core::ffi::c_int;
            }
            111 => {
                tmode = MODE_OP_PENDING as ::core::ffi::c_int;
            }
            116 => {
                tmode = MODE_TERMINAL as ::core::ffi::c_int;
            }
            118 => {
                tmode = MASK_V;
            }
            33 => {
                tmode = MASK_BANG;
            }
            32 => {
                tmode = MASK_MAP;
            }
            _ => return 0 as ::core::ffi::c_int,
        }
        mode |= tmode;
    }
    if abbr as ::core::ffi::c_int != 0 && mode & !MASK_BANG != 0 as ::core::ffi::c_int
        || !abbr
            && mode & mode - 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && !(mode & MASK_BANG != 0 as ::core::ffi::c_int
                && mode & !MASK_BANG == 0 as ::core::ffi::c_int
                || mode & MASK_MAP != 0 as ::core::ffi::c_int
                    && mode & !MASK_MAP == 0 as ::core::ffi::c_int)
    {
        return 0 as ::core::ffi::c_int;
    }
    return mode;
}
#[no_mangle]
pub unsafe extern "C" fn f_mapset(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut is_abbr: ::core::ffi::c_int = 0;
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let dict_only: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint;
    if dict_only {
        d = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        which = tv_dict_get_string(
            d,
            b"mode\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        is_abbr = tv_dict_get_bool(
            d,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
        if which.is_null() || is_abbr < 0 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_entries_missing_in_mapset_dict_argument as *const ::core::ffi::c_char,
            ));
            return;
        }
    } else {
        which = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if which.is_null() {
            return;
        }
        is_abbr =
            tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        d = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    let mode: ::core::ffi::c_int = get_map_mode_string(which, is_abbr != 0);
    if mode == 0 as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_illegal_map_mode_string_str as *const ::core::ffi::c_char),
            which,
        );
        return;
    }
    let mut lhs: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhs\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut lhsraw: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut lhsrawalt: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut orig_rhs: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut rhs_lua: LuaRef = LUA_NOREF;
    let mut callback_di: *mut dictitem_T = tv_dict_find(
        d,
        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !callback_di.is_null() {
        if (*callback_di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut fp: *mut ufunc_T = find_func((*callback_di).di_tv.vval.v_string);
            if !fp.is_null() && (*fp).uf_flags & FC_LUAREF != 0 {
                rhs_lua = api_new_luaref((*fp).uf_luaref);
                orig_rhs = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
    }
    if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
        emsg(gettext(
            &raw const e_entries_missing_in_mapset_dict_argument as *const ::core::ffi::c_char,
        ));
        api_free_luaref(rhs_lua);
        return;
    }
    let mut noremap: ::core::ffi::c_int =
        if tv_dict_get_number(d, b"noremap\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T
        {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    if tv_dict_get_number(d, b"script\0".as_ptr() as *const ::core::ffi::c_char) != 0 as varnumber_T
    {
        noremap = REMAP_SCRIPT as ::core::ffi::c_int;
    }
    let mut args: MapArguments = map_arguments {
        buffer: false,
        expr: tv_dict_get_number(d, b"expr\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        noremap: false,
        nowait: tv_dict_get_number(d, b"nowait\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        script: false,
        silent: tv_dict_get_number(d, b"silent\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        unique: false,
        replace_keycodes: tv_dict_get_number(
            d,
            b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: tv_dict_get_string(
            d,
            b"desc\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ),
    };
    let mut sid: scid_T =
        tv_dict_get_number(d, b"sid\0".as_ptr() as *const ::core::ffi::c_char) as scid_T;
    let mut lnum: linenr_T =
        tv_dict_get_number(d, b"lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
    let mut buffer: bool =
        tv_dict_get_number(d, b"buffer\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
    set_maparg_rhs(
        orig_rhs,
        strlen(orig_rhs),
        rhs_lua,
        sid,
        p_cpo,
        &raw mut args,
    );
    let mut map_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
        &raw mut (*curbuf).b_maphash as *mut *mut mapblock_T
    } else {
        &raw mut maphash as *mut *mut mapblock_T
    };
    let mut abbr_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
        &raw mut (*curbuf).b_first_abbr
    } else {
        &raw mut first_abbr
    };
    let mut unmap_args: MapArguments = MAP_ARGUMENTS_INIT;
    set_maparg_lhs_rhs(
        lhs,
        strlen(lhs),
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        0 as size_t,
        LUA_NOREF,
        p_cpo,
        &raw mut unmap_args,
    );
    unmap_args.buffer = buffer;
    buf_do_map(
        MAPTYPE_UNMAP_LHS as ::core::ffi::c_int,
        &raw mut unmap_args,
        mode,
        is_abbr != 0,
        curbuf,
    );
    xfree(unmap_args.rhs as *mut ::core::ffi::c_void);
    xfree(unmap_args.orig_rhs as *mut ::core::ffi::c_void);
    let mut mp_result: [*mut mapblock_T; 2] = [
        ::core::ptr::null_mut::<mapblock_T>(),
        ::core::ptr::null_mut::<mapblock_T>(),
    ];
    mp_result[0 as ::core::ffi::c_int as usize] = map_add(
        curbuf,
        map_table,
        abbr_table,
        lhsraw,
        &raw mut args,
        noremap,
        mode,
        is_abbr != 0,
        sid,
        lnum,
        false_0 != 0,
    );
    if !lhsrawalt.is_null() {
        mp_result[1 as ::core::ffi::c_int as usize] = map_add(
            curbuf,
            map_table,
            abbr_table,
            lhsrawalt,
            &raw mut args,
            noremap,
            mode,
            is_abbr != 0,
            sid,
            lnum,
            true_0 != 0,
        );
    }
    if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
        && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
    {
        (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
            mp_result[1 as ::core::ffi::c_int as usize];
        (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
            mp_result[0 as ::core::ffi::c_int as usize];
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_maplist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let abbr: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_bool(argvars.offset(0 as ::core::ffi::c_int as isize)) != 0;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut buffer_local: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while buffer_local <= 1 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if buffer_local != 0 {
                    mp = (*curbuf).b_first_abbr;
                } else {
                    mp = first_abbr;
                }
            } else if buffer_local != 0 {
                mp = (*curbuf).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = maphash[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_simplified == 0 {
                    let mut keys_buf: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut did_simplify: bool = false_0 != 0;
                    let mut arena: Arena = ARENA_EMPTY;
                    let mut lhs: *mut ::core::ffi::c_char =
                        str2special_arena((*mp).m_keys, true_0 != 0, false_0 != 0, &raw mut arena);
                    replace_termcodes(
                        lhs,
                        strlen(lhs),
                        &raw mut keys_buf,
                        0 as scid_T,
                        flags,
                        &raw mut did_simplify,
                        p_cpo,
                    );
                    let mut dict: Dict = mapblock_fill_dict(
                        mp,
                        if did_simplify as ::core::ffi::c_int != 0 {
                            keys_buf
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        },
                        buffer_local,
                        abbr,
                        true_0 != 0,
                        &raw mut arena,
                    );
                    let mut d: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    let mut c2rust_lvalue: Object = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed { dict: dict },
                    };
                    object_to_vim_take_luaref(
                        &raw mut c2rust_lvalue,
                        &raw mut d,
                        true_0 != 0,
                        ::core::ptr::null_mut::<Error>(),
                    );
                    '_c2rust_label: {
                        if d.v_type as ::core::ffi::c_uint
                            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"d.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                2431 as ::core::ffi::c_uint,
                                b"void f_maplist(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    tv_list_append_dict((*rettv).vval.v_list, d.vval.v_dict);
                    arena_mem_free(arena_finish(&raw mut arena));
                    xfree(keys_buf as *mut ::core::ffi::c_void);
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        buffer_local += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_maparg(argvars, rettv, true_0);
}
#[no_mangle]
pub unsafe extern "C" fn f_mapcheck(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_maparg(argvars, rettv, false_0);
}
#[no_mangle]
pub unsafe extern "C" fn add_map(
    mut lhs: *mut ::core::ffi::c_char,
    mut rhs: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut buffer: bool,
) {
    let mut args: MapArguments = MAP_ARGUMENTS_INIT;
    set_maparg_lhs_rhs(
        lhs,
        strlen(lhs),
        rhs,
        strlen(rhs),
        LUA_NOREF,
        p_cpo,
        &raw mut args,
    );
    args.buffer = buffer;
    buf_do_map(
        MAPTYPE_NOREMAP as ::core::ffi::c_int,
        &raw mut args,
        mode,
        false_0 != 0,
        curbuf,
    );
    xfree(args.rhs as *mut ::core::ffi::c_void);
    xfree(args.orig_rhs as *mut ::core::ffi::c_void);
}
static mut langmap_mapga: garray_T = GA_EMPTY_INIT_VALUE;
unsafe extern "C" fn langmap_set_entry(mut from: ::core::ffi::c_int, mut to: ::core::ffi::c_int) {
    let mut entries: *mut langmap_entry_T = langmap_mapga.ga_data as *mut langmap_entry_T;
    let mut a: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    '_c2rust_label: {
        if langmap_mapga.ga_len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"langmap_mapga.ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                2496 as ::core::ffi::c_uint,
                b"void langmap_set_entry(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut b: ::core::ffi::c_uint = langmap_mapga.ga_len as ::core::ffi::c_uint;
    while a != b {
        let mut i: ::core::ffi::c_uint = a.wrapping_add(b).wrapping_div(2 as ::core::ffi::c_uint);
        let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - from;
        if d == 0 as ::core::ffi::c_int {
            (*entries.offset(i as isize)).to = to;
            return;
        }
        if d < 0 as ::core::ffi::c_int {
            a = i.wrapping_add(1 as ::core::ffi::c_uint);
        } else {
            b = i;
        }
    }
    ga_grow(&raw mut langmap_mapga, 1 as ::core::ffi::c_int);
    entries = (langmap_mapga.ga_data as *mut langmap_entry_T).offset(a as isize);
    memmove(
        entries.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        entries as *const ::core::ffi::c_void,
        ((langmap_mapga.ga_len as ::core::ffi::c_uint).wrapping_sub(a) as size_t)
            .wrapping_mul(::core::mem::size_of::<langmap_entry_T>()),
    );
    langmap_mapga.ga_len += 1;
    (*entries.offset(0 as ::core::ffi::c_int as isize)).from = from;
    (*entries.offset(0 as ::core::ffi::c_int as isize)).to = to;
}
#[no_mangle]
pub unsafe extern "C" fn langmap_adjust_mb(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut entries: *mut langmap_entry_T = langmap_mapga.ga_data as *mut langmap_entry_T;
    let mut a: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = langmap_mapga.ga_len;
    while a != b {
        let mut i: ::core::ffi::c_int = (a + b) / 2 as ::core::ffi::c_int;
        let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - c;
        if d == 0 as ::core::ffi::c_int {
            return (*entries.offset(i as isize)).to;
        }
        if d < 0 as ::core::ffi::c_int {
            a = i + 1 as ::core::ffi::c_int;
        } else {
            b = i;
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn langmap_init() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        langmap_mapchar[i as usize] = i as uint8_t;
        i += 1;
    }
    ga_init(
        &raw mut langmap_mapga,
        ::core::mem::size_of::<langmap_entry_T>() as ::core::ffi::c_int,
        8 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn did_set_langmap(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    ga_clear(&raw mut langmap_mapga);
    langmap_init();
    let mut p: *mut ::core::ffi::c_char = p_langmap;
    while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p2 = p;
        while *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ',' as ::core::ffi::c_int
            && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ';' as ::core::ffi::c_int
        {
            if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p2 = p2.offset(1);
            }
            p2 = p2.offset(utfc_ptr2len(p2) as isize);
        }
        if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ';' as ::core::ffi::c_int
        {
            p2 = p2.offset(1);
        } else {
            p2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
                break;
            } else {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                }
                let mut from: ::core::ffi::c_int = utf_ptr2char(p);
                let from_ptr: *const ::core::ffi::c_char = p;
                let mut to: ::core::ffi::c_int = NUL;
                let mut to_ptr: *const ::core::ffi::c_char =
                    b"\0".as_ptr() as *const ::core::ffi::c_char;
                if p2.is_null() {
                    p = p.offset(utfc_ptr2len(p) as isize);
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ',' as ::core::ffi::c_int
                    {
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                        }
                        to_ptr = p;
                        to = utf_ptr2char(to_ptr);
                    }
                } else if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int
                {
                    if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    {
                        p2 = p2.offset(1);
                    }
                    to_ptr = p2;
                    to = utf_ptr2char(to_ptr);
                }
                if to == NUL {
                    snprintf(
                        (*args).os_errbuf,
                        (*args).os_errbuflen,
                        gettext(
                            b"E357: 'langmap': Matching character missing for %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        transchar(from),
                    );
                    return (*args).os_errbuf;
                }
                if from >= 256 as ::core::ffi::c_int {
                    langmap_set_entry(from, to);
                } else {
                    if to > UCHAR_MAX {
                        swmsg(
                            true_0 != 0,
                            b"'langmap': Mapping from %.*s to %.*s will not work properly\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            utf_ptr2len(from_ptr),
                            from_ptr,
                            utf_ptr2len(to_ptr),
                            to_ptr,
                        );
                    }
                    langmap_mapchar[(from & 255 as ::core::ffi::c_int) as usize] = to as uint8_t;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
                if p2.is_null() {
                    continue;
                }
                p2 = p2.offset(utfc_ptr2len(p2) as isize);
                if *p as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                    continue;
                }
                p = p2;
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ',' as ::core::ffi::c_int
                    {
                        snprintf(
                            (*args).os_errbuf,
                            (*args).os_errbuflen,
                            gettext(
                                b"E358: 'langmap': Extra characters after semicolon: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            p,
                        );
                        return (*args).os_errbuf;
                    }
                    p = p.offset(1);
                }
                break;
            }
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn do_exmap(mut eap: *mut exarg_T, mut isabbrev: ::core::ffi::c_int) {
    let mut cmdp: *mut ::core::ffi::c_char = (*eap).cmd;
    let mut mode: ::core::ffi::c_int =
        get_map_mode(&raw mut cmdp, (*eap).forceit != 0 || isabbrev != 0);
    let mut maptype: ::core::ffi::c_int = 0;
    if *cmdp as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
        maptype = MAPTYPE_NOREMAP as ::core::ffi::c_int;
    } else if *cmdp as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
        maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
    } else {
        maptype = MAPTYPE_MAP as ::core::ffi::c_int;
    }
    let mut parsed_args: MapArguments = MapArguments {
        buffer: false,
        expr: false,
        noremap: false,
        nowait: false,
        script: false,
        silent: false,
        unique: false,
        replace_keycodes: false,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: ::core::ffi::c_int = str_to_mapargs(
        (*eap).arg,
        maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
        &raw mut parsed_args,
    );
    match result {
        0 => match buf_do_map(maptype, &raw mut parsed_args, mode, isabbrev != 0, curbuf) {
            1 => {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            }
            2 => {
                emsg(if isabbrev != 0 {
                    gettext(&raw const e_noabbr as *const ::core::ffi::c_char)
                } else {
                    gettext(&raw const e_nomap as *const ::core::ffi::c_char)
                });
            }
            5 => {
                semsg(
                    if isabbrev != 0 {
                        gettext(
                            &raw const e_abbreviation_already_exists_for_str
                                as *const ::core::ffi::c_char,
                        )
                    } else {
                        gettext(
                            &raw const e_mapping_already_exists_for_str
                                as *const ::core::ffi::c_char,
                        )
                    },
                    &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                );
            }
            6 => {
                semsg(
                    if isabbrev != 0 {
                        gettext(
                            &raw const e_global_abbreviation_already_exists_for_str
                                as *const ::core::ffi::c_char,
                        )
                    } else {
                        gettext(
                            &raw const e_global_mapping_already_exists_for_str
                                as *const ::core::ffi::c_char,
                        )
                    },
                    &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                );
            }
            _ => {}
        },
        1 => {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        }
        _ => {
            '_c2rust_label: {
                if false
                    && !(b"Unknown return code from str_to_mapargs!\0".as_ptr()
                        as *const ::core::ffi::c_char)
                        .is_null()
                {
                } else {
                    __assert_fail(
                        b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        2669 as ::core::ffi::c_uint,
                        b"void do_exmap(exarg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ex_abbreviate(mut eap: *mut exarg_T) {
    do_exmap(eap, true_0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_map(mut eap: *mut exarg_T) {
    if secure != 0 {
        secure = 2 as ::core::ffi::c_int;
        msg_outtrans((*eap).cmd, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    do_exmap(eap, false_0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_unmap(mut eap: *mut exarg_T) {
    do_exmap(eap, false_0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_mapclear(mut eap: *mut exarg_T) {
    do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit, false_0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_abclear(mut eap: *mut exarg_T) {
    do_mapclear((*eap).cmd, (*eap).arg, true_0, true_0);
}
#[no_mangle]
pub unsafe extern "C" fn modify_keymap(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut is_unmap: bool,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut forceit: bool = false;
    let mut mode_val: ::core::ffi::c_int = 0;
    let mut is_abbrev: bool = false;
    let mut is_noremap: bool = false;
    let mut maptype_val: ::core::ffi::c_int = 0;
    let mut lua_funcref: LuaRef = LUA_NOREF;
    let mut global: bool = buffer == -1 as ::core::ffi::c_int;
    if global {
        buffer = 0 as ::core::ffi::c_int as Buffer;
    }
    let mut target_buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if target_buf.is_null() {
        return;
    }
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    let mut parsed_args: MapArguments = MAP_ARGUMENTS_INIT;
    if !opts.is_null() {
        parsed_args.nowait = (*opts).nowait as bool;
        parsed_args.noremap = (*opts).noremap as bool;
        parsed_args.silent = (*opts).silent as bool;
        parsed_args.script = (*opts).script as bool;
        parsed_args.expr = (*opts).expr as bool;
        parsed_args.unique = (*opts).unique as bool;
        parsed_args.replace_keycodes = (*opts).replace_keycodes as bool;
        if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__callback
            != 0 as ::core::ffi::c_ulonglong
        {
            lua_funcref = (*opts).callback;
            (*opts).callback = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__desc
            != 0 as ::core::ffi::c_ulonglong
        {
            parsed_args.desc = string_to_cstr((*opts).desc);
        }
    }
    parsed_args.buffer = !global;
    '_fail_and_free: {
        if parsed_args.replace_keycodes as ::core::ffi::c_int != 0 && !parsed_args.expr {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"\"replace_keycodes\" requires \"expr\"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if !set_maparg_lhs_rhs(
            lhs.data,
            lhs.size,
            rhs.data,
            rhs.size,
            lua_funcref,
            p_cpo,
            &raw mut parsed_args,
        ) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                lhs.data,
            );
        } else if parsed_args.lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
            || parsed_args.alt_lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                lhs.data,
            );
        } else {
            p = (if mode.size > 0 as size_t {
                mode.data as *const ::core::ffi::c_char
            } else {
                b"m\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            forceit = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
            mode_val = get_map_mode(&raw mut p, forceit);
            if forceit {
                '_c2rust_label: {
                    if p == mode.data {
                    } else {
                        __assert_fail(
                            b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2794 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                p = p.offset(1);
            }
            is_abbrev = mode_val
                & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
                && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
            if is_abbrev {
                p = p.offset(1);
            }
            if mode.size > 0 as size_t && p.offset_from(mode.data) as size_t != mode.size {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid mode shortname: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                    mode.data,
                );
            } else if parsed_args.lhs_len == 0 as size_t {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid (empty) LHS\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                is_noremap = parsed_args.noremap;
                '_c2rust_label_0: {
                    if !(is_unmap as ::core::ffi::c_int != 0
                        && is_noremap as ::core::ffi::c_int != 0)
                    {
                    } else {
                        __assert_fail(
                            b"!(is_unmap && is_noremap)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2812 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if !is_unmap
                    && lua_funcref == LUA_NOREF
                    && (parsed_args.rhs_len == 0 as size_t && !parsed_args.rhs_is_noop)
                {
                    if rhs.size == 0 as size_t {
                        parsed_args.rhs_is_noop = true_0 != 0;
                    } else {
                        abort();
                    }
                } else if is_unmap as ::core::ffi::c_int != 0
                    && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF)
                {
                    if parsed_args.rhs_len != 0 {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Gave nonempty RHS in unmap command: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            parsed_args.rhs,
                        );
                    } else {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Gave nonempty RHS for unmap\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break '_fail_and_free;
                }
                maptype_val = MAPTYPE_MAP as ::core::ffi::c_int;
                if is_unmap {
                    maptype_val = MAPTYPE_UNMAP as ::core::ffi::c_int;
                } else if is_noremap {
                    maptype_val = MAPTYPE_NOREMAP as ::core::ffi::c_int;
                }
                match buf_do_map(
                    maptype_val,
                    &raw mut parsed_args,
                    mode_val,
                    is_abbrev,
                    target_buf,
                ) {
                    0 => {}
                    1 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            &raw const e_invarg as *const ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    2 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            &raw const e_nomap as *const ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    5 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            if is_abbrev as ::core::ffi::c_int != 0 {
                                &raw const e_abbreviation_already_exists_for_str
                                    as *const ::core::ffi::c_char
                            } else {
                                &raw const e_mapping_already_exists_for_str
                                    as *const ::core::ffi::c_char
                            },
                            lhs.data,
                        );
                    }
                    6 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            if is_abbrev as ::core::ffi::c_int != 0 {
                                &raw const e_global_abbreviation_already_exists_for_str
                                    as *const ::core::ffi::c_char
                            } else {
                                &raw const e_global_mapping_already_exists_for_str
                                    as *const ::core::ffi::c_char
                            },
                            lhs.data,
                        );
                    }
                    _ => {
                        '_c2rust_label_1: {
                            if false
                                && !(b"Unrecognized return code!\0".as_ptr()
                                    as *const ::core::ffi::c_char)
                                    .is_null()
                            {
                            } else {
                                __assert_fail(
                                    b"false && \"Unrecognized return code!\"\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2860 as ::core::ffi::c_uint,
                                    b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                }
            }
        }
    }
    current_sctx = save_current_sctx;
    if parsed_args.rhs_lua != LUA_NOREF {
        api_free_luaref(parsed_args.rhs_lua);
        parsed_args.rhs_lua = LUA_NOREF as LuaRef;
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.desc as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn keymap_array(
    mut mode: String_0,
    mut buf: *mut buf_T,
    mut arena: *mut Arena,
) -> Array {
    let mut mappings: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
    };
    mappings.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mappings.size = 0 as size_t;
    mappings.items = &raw mut mappings.init_array as *mut Object;
    let mut p: *mut ::core::ffi::c_char = (if mode.size > 0 as size_t {
        mode.data as *const ::core::ffi::c_char
    } else {
        b"m\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    let mut forceit: bool = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
    let mut int_mode: ::core::ffi::c_int = get_map_mode(&raw mut p, forceit);
    if forceit {
        '_c2rust_label: {
            if p == mode.data {
            } else {
                __assert_fail(
                    b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/mapping.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    2888 as ::core::ffi::c_uint,
                    b"Array keymap_array(String, buf_T *, Arena *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        p = p.offset(1);
    }
    let mut is_abbrev: bool = int_mode
        & (MODE_INSERT as ::core::ffi::c_int | MODE_CMDLINE as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
        && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
    let mut buffer_value: ::core::ffi::c_int = if buf.is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*buf).handle as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i
        < (if is_abbrev as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            MAX_MAPHASH
        })
    {
        let mut current_maphash: *const mapblock_T = if is_abbrev as ::core::ffi::c_int != 0 {
            if !buf.is_null() {
                (*buf).b_first_abbr
            } else {
                first_abbr
            }
        } else if !buf.is_null() {
            (*buf).b_maphash[i as usize] as *mut mapblock_T
        } else {
            maphash[i as usize] as *mut mapblock_T
        };
        while !current_maphash.is_null() {
            if (*current_maphash).m_simplified == 0 {
                if int_mode & (*current_maphash).m_mode != 0 {
                    if mappings.size == mappings.capacity {
                        mappings.capacity = if mappings.capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            mappings.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        mappings.items = (if mappings.capacity
                            == ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if mappings.items == &raw mut mappings.init_array as *mut Object {
                                mappings.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut mappings.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    mappings.items as *mut ::core::ffi::c_void,
                                    mappings.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if mappings.items == &raw mut mappings.init_array as *mut Object {
                                memcpy(
                                    xmalloc(
                                        mappings
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    mappings.items as *const ::core::ffi::c_void,
                                    mappings.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    mappings.items as *mut ::core::ffi::c_void,
                                    mappings
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh41 = mappings.size;
                    mappings.size = mappings.size.wrapping_add(1);
                    *mappings.items.offset(c2rust_fresh41 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed {
                            dict: mapblock_fill_dict(
                                current_maphash,
                                if !(*current_maphash).m_alt.is_null() {
                                    (*(*current_maphash).m_alt).m_keys
                                } else {
                                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                                },
                                buffer_value,
                                is_abbrev,
                                false,
                                arena,
                            ),
                        },
                    };
                }
            }
            current_maphash = (*current_maphash).m_next;
        }
        i += 1;
    }
    return arena_take_arraybuilder(arena, &raw mut mappings);
}
pub const SCHAR_MAX: ::core::ffi::c_int = __SCHAR_MAX__;
pub const UCHAR_MAX: ::core::ffi::c_int =
    SCHAR_MAX * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __SCHAR_MAX__: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
