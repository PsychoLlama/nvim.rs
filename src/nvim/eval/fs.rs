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
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseeko(
        __stream: *mut FILE,
        __off: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fileno(__stream: *mut FILE) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn readlink(
        __path: *const ::core::ffi::c_char,
        __buf: *mut ::core::ffi::c_char,
        __len: size_t,
    ) -> ssize_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
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
    fn ExpandOne(
        xp: *mut expand_T,
        str: *mut ::core::ffi::c_char,
        orig: *mut ::core::ffi::c_char,
        options: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ExpandInit(xp: *mut expand_T);
    fn ExpandCleanup(xp: *mut expand_T);
    fn globpath(
        path: *mut ::core::ffi::c_char,
        file: *mut ::core::ffi::c_char,
        ga: *mut garray_T,
        expand_options: ::core::ffi::c_int,
        dirs: bool,
    );
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_invexpr2: [::core::ffi::c_char; 0];
    static e_isadir2: [::core::ffi::c_char; 0];
    static e_mkdir: [::core::ffi::c_char; 0];
    static e_notopen: [::core::ffi::c_char; 0];
    static e_cant_read_file_str: [::core::ffi::c_char; 0];
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn eval_expr_typval(
        expr: *const typval_T,
        want_func: bool,
        argv: *mut typval_T,
        argc: ::core::ffi::c_int,
        rettv: *mut typval_T,
    ) -> ::core::ffi::c_int;
    fn do_string_sub(
        str: *mut ::core::ffi::c_char,
        len: size_t,
        pat: *mut ::core::ffi::c_char,
        sub: *mut ::core::ffi::c_char,
        expr: *mut typval_T,
        flags: *const ::core::ffi::c_char,
        ret_len: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_item_remove(l: *mut list_T, item: *mut listitem_T) -> *mut listitem_T;
    fn tv_list_append_owned_tv(l: *mut list_T, tv: typval_T) -> *mut typval_T;
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_blob_free(b: *mut blob_T);
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T;
    fn tv_clear(tv: *mut typval_T);
    fn tv_check_str_or_nr(tv: *const typval_T) -> bool;
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_string_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_check_for_nonempty_string_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string_buf_chk(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string_buf(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn can_add_defer() -> bool;
    fn add_defer(
        name: *mut ::core::ffi::c_char,
        argcount_arg: ::core::ffi::c_int,
        argvars: *mut typval_T,
    );
    fn prepare_vimvar(idx: ::core::ffi::c_int, save_tv: *mut typval_T);
    fn restore_vimvar(idx: ::core::ffi::c_int, save_tv: *mut typval_T);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn find_win_by_nr(vp: *mut typval_T, tp: *mut tabpage_T) -> *mut win_T;
    fn changedir_func(new_dir: *mut ::core::ffi::c_char, scope: CdScope) -> bool;
    fn vim_mkdir_emsg(
        name: *const ::core::ffi::c_char,
        prot: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_findfile_cleanup(ctx: *mut ::core::ffi::c_void);
    fn find_file_in_path_option(
        ptr: *mut ::core::ffi::c_char,
        len: size_t,
        options: ::core::ffi::c_int,
        first: ::core::ffi::c_int,
        path_option: *mut ::core::ffi::c_char,
        find_what: ::core::ffi::c_int,
        rel_fname: *mut ::core::ffi::c_char,
        suffixes: *mut ::core::ffi::c_char,
        file_to_find: *mut *mut ::core::ffi::c_char,
        search_ctx_arg: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_rename(
        from: *const ::core::ffi::c_char,
        to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn vim_copyfile(
        from: *const ::core::ffi::c_char,
        to: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn readdir_core(
        gap: *mut garray_T,
        path: *const ::core::ffi::c_char,
        context: *mut ::core::ffi::c_void,
        checkitem: CheckItem,
    ) -> ::core::ffi::c_int;
    fn delete_recursive(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_tempname() -> *mut ::core::ffi::c_char;
    fn file_pat_to_reg_pat(
        pat: *const ::core::ffi::c_char,
        pat_end: *const ::core::ffi::c_char,
        allow_dirs: *mut ::core::ffi::c_char,
        no_bslash: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat_strings(
        gap: *const garray_T,
        sep: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static current_sctx: GlobalCell<sctx_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static globaldir: GlobalCell<*mut ::core::ffi::c_char>;
    static p_fs: GlobalCell<::core::ffi::c_int>;
    static p_path: GlobalCell<*mut ::core::ffi::c_char>;
    static p_wic: GlobalCell<::core::ffi::c_int>;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_can_exe(
        name: *const ::core::ffi::c_char,
        abspath: *mut *mut ::core::ffi::c_char,
        use_path: bool,
    ) -> bool;
    fn os_fopen(path: *const ::core::ffi::c_char, flags: *const ::core::ffi::c_char) -> *mut FILE;
    fn os_getperm(name: *const ::core::ffi::c_char) -> int32_t;
    fn os_file_is_readable(name: *const ::core::ffi::c_char) -> bool;
    fn os_file_is_writable(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_mkdir_recurse(
        dir: *const ::core::ffi::c_char,
        mode: int32_t,
        failed_dir: *mut *mut ::core::ffi::c_char,
        created: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn os_rmdir(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_fileinfo(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_link(path: *const ::core::ffi::c_char, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_fd(file_descriptor: ::core::ffi::c_int, file_info: *mut FileInfo) -> bool;
    fn os_fileinfo_size(file_info: *const FileInfo) -> uint64_t;
    fn file_open(
        ret_fp: *mut FileDescriptor,
        fname: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> ::core::ffi::c_int;
    fn file_flush(fp: *mut FileDescriptor) -> ::core::ffi::c_int;
    fn file_write(
        fp: *mut FileDescriptor,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> ptrdiff_t;
    fn expand_env_save(src: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn home_replace(
        buf: *const buf_T,
        src: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: size_t,
        one: bool,
    ) -> size_t;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_next_component(fname: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char;
    fn get_past_head(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn shorten_dir_len(str: *mut ::core::ffi::c_char, trim_len: ::core::ffi::c_int);
    fn path_fnamencmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn add_pathsep(p: *mut ::core::ffi::c_char) -> bool;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn simplify_filename(filename: *mut ::core::ffi::c_char) -> size_t;
    fn vim_isAbsName(name: *const ::core::ffi::c_char) -> bool;
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn path_is_absolute(fname: *const ::core::ffi::c_char) -> bool;
    fn script_is_lua(sid: scid_T) -> bool;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strsave_shellescape(
        string: *const ::core::ffi::c_char,
        do_special: bool,
        do_newline: bool,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn find_tabpage(n: ::core::ffi::c_int) -> *mut tabpage_T;
    fn check_secure() -> bool;
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
pub type off_t = __off_t;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type off_T = off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
pub type linenr_T = int32_t;
pub type colnr_T = ::core::ffi::c_int;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed = 2147483647;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
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
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
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
    pub b_signcols: C2Rust_Unnamed_4,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_2,
    pub update_callbacks: C2Rust_Unnamed_1,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub data: C2Rust_Unnamed_3,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
pub struct C2Rust_Unnamed_4 {
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
pub type disptick_T = uint64_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_5,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_6,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub fc_fixvar: [C2Rust_Unnamed_7; 12],
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
pub struct C2Rust_Unnamed_7 {
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
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileInfo {
    pub stat: uv_stat_t,
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_15 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_15 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_15 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_15 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_15 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_15 = 7;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub const WILD_PREV: C2Rust_Unnamed_15 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_15 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_15 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_15 = 2;
pub const WILD_FREE: C2Rust_Unnamed_15 = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
pub const VALID_PATH: C2Rust_Unnamed_17 = 1;
pub const VALID_HEAD: C2Rust_Unnamed_17 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_18 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_18 = 0;
pub type CheckItem = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *const ::core::ffi::c_char) -> varnumber_T,
>;
pub const kFileCreate: C2Rust_Unnamed_19 = 2;
pub const kFileMkDir: C2Rust_Unnamed_19 = 256;
pub const kFileTruncate: C2Rust_Unnamed_19 = 32;
pub const kFileAppend: C2Rust_Unnamed_19 = 64;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const FINDFILE_BOTH: C2Rust_Unnamed_18 = 2;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kFileNonBlocking: C2Rust_Unnamed_19 = 128;
pub const kFileCreateOnly: C2Rust_Unnamed_19 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_19 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_19 = 4;
pub const kFileReadOnly: C2Rust_Unnamed_19 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_error_while_writing_str: GlobalCell<[::core::ffi::c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"E80: Error while writing: %s\0",
    )
});
#[no_mangle]
pub unsafe extern "C" fn modify_fname(
    mut src: *mut ::core::ffi::c_char,
    mut tilde_file: bool,
    mut usedlen: *mut size_t,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut fnamelen: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut valid: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut has_fullname: bool = false_0 != 0;
    let mut has_homerelative: bool = false_0 != 0;
    loop {
        if *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 'p' as ::core::ffi::c_int
        {
            has_fullname = true_0 != 0;
            valid |= VALID_PATH as ::core::ffi::c_int;
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            if *(*fnamep).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '~' as ::core::ffi::c_int
                && !(tilde_file as ::core::ffi::c_int != 0
                    && *(*fnamep).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == NUL)
            {
                *fnamep = expand_env_save(*fnamep);
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                if (*fnamep).is_null() {
                    return -1 as ::core::ffi::c_int;
                }
            }
            p = *fnamep;
            while *p as ::core::ffi::c_int != NUL {
                if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                    && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || vim_ispathsep(
                            *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            && (*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == NUL
                                || vim_ispathsep(*p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0))
                {
                    break;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            if *p as ::core::ffi::c_int != NUL || !vim_isAbsName(*fnamep) {
                *fnamep = FullName_save(*fnamep, *p as ::core::ffi::c_int != NUL);
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                if (*fnamep).is_null() {
                    return -1 as ::core::ffi::c_int;
                }
            }
            if os_isdir(*fnamep) {
                *fnamep = xstrnsave(*fnamep, strlen(*fnamep).wrapping_add(2 as size_t));
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = *fnamep;
                add_pathsep(*fnamep);
            }
        }
        c = 0;
        while *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
            c = *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as uint8_t
                as ::core::ffi::c_int;
            c == '.' as ::core::ffi::c_int
                || c == '~' as ::core::ffi::c_int
                || c == '8' as ::core::ffi::c_int
        } {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            if c == '8' as ::core::ffi::c_int {
                continue;
            }
            pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !has_fullname && !has_homerelative {
                if **fnamep as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                    pbuf = expand_env_save(*fnamep);
                    p = pbuf;
                } else {
                    pbuf = FullName_save(*fnamep, false_0 != 0);
                    p = pbuf;
                }
            } else {
                p = *fnamep;
            }
            has_fullname = false_0 != 0;
            if !p.is_null() {
                if c == '.' as ::core::ffi::c_int {
                    os_dirname(
                        &raw mut dirname as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                    );
                    if has_homerelative {
                        s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            s,
                            &raw mut dirname as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        xfree(s as *mut ::core::ffi::c_void);
                    }
                    let mut namelen: size_t = strlen(&raw mut dirname as *mut ::core::ffi::c_char);
                    if path_fnamencmp(p, &raw mut dirname as *mut ::core::ffi::c_char, namelen)
                        == 0 as ::core::ffi::c_int
                    {
                        p = p.offset(namelen as isize);
                        if vim_ispathsep(*p as ::core::ffi::c_int) {
                            while *p as ::core::ffi::c_int != 0
                                && vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(1);
                            }
                            *fnamep = p;
                            if !pbuf.is_null() {
                                xfree(*bufp as *mut ::core::ffi::c_void);
                                *bufp = pbuf;
                                pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                        }
                    }
                } else {
                    home_replace(
                        ::core::ptr::null::<buf_T>(),
                        p,
                        &raw mut dirname as *mut ::core::ffi::c_char,
                        MAXPATHL as size_t,
                        true_0 != 0,
                    );
                    if *(&raw mut dirname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                    {
                        s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                        '_c2rust_label: {
                            if !s.is_null() {
                            } else {
                                __assert_fail(
                                    b"s != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/eval/fs.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    195 as ::core::ffi::c_uint,
                                    b"int modify_fname(char *, _Bool, size_t *, char **, char **, size_t *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        *fnamep = s;
                        xfree(*bufp as *mut ::core::ffi::c_void);
                        *bufp = s;
                        has_homerelative = true_0 != 0;
                    }
                }
                xfree(pbuf as *mut ::core::ffi::c_void);
            }
        }
        tail = path_tail(*fnamep);
        *fnamelen = strlen(*fnamep);
        while *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 'h' as ::core::ffi::c_int
        {
            valid |= VALID_HEAD as ::core::ffi::c_int;
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            s = get_past_head(*fnamep);
            while tail > s && after_pathsep(s, tail) != 0 {
                tail = tail.offset(
                    -((utf_head_off(*fnamep, tail.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
            *fnamelen = tail.offset_from(*fnamep) as size_t;
            if *fnamelen == 0 as size_t {
                xfree(*bufp as *mut ::core::ffi::c_void);
                tail = xstrdup(b".\0".as_ptr() as *const ::core::ffi::c_char);
                *fnamep = tail;
                *bufp = *fnamep;
                *fnamelen = 1 as size_t;
            } else {
                while tail > s && after_pathsep(s, tail) == 0 {
                    tail = tail.offset(
                        -((utf_head_off(*fnamep, tail.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
            }
        }
        if *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == '8' as ::core::ffi::c_int
        {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
        }
        if *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 't' as ::core::ffi::c_int
        {
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
            *fnamelen = (*fnamelen).wrapping_sub(tail.offset_from(*fnamep) as size_t);
            *fnamep = tail;
        }
        while *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && (*src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
                || *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    == 'r' as ::core::ffi::c_int)
        {
            let is_second_e: bool = *fnamep > tail;
            if *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
                && is_second_e as ::core::ffi::c_int != 0
            {
                s = (*fnamep).offset(-(2 as ::core::ffi::c_int as isize));
            } else {
                s = (*fnamep)
                    .offset(*fnamelen as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize));
            }
            while s > tail {
                if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                {
                    break;
                }
                s = s.offset(-1);
            }
            if *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
            {
                if s > tail || false && is_second_e as ::core::ffi::c_int != 0 && s == tail {
                    let mut newstart: *mut ::core::ffi::c_char =
                        s.offset(1 as ::core::ffi::c_int as isize);
                    let mut distance_stepped_back: size_t =
                        (*fnamep).offset_from(newstart) as size_t;
                    *fnamelen = (*fnamelen).wrapping_add(distance_stepped_back);
                    *fnamep = newstart;
                } else if *fnamep <= tail {
                    *fnamelen = 0 as size_t;
                }
            } else if s > (if tail > *fnamep { tail } else { *fnamep }) {
                *fnamelen = s.offset_from(*fnamep) as size_t;
            }
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
        }
        if !(*src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && (*src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == 's' as ::core::ffi::c_int
                || *src.offset((*usedlen).wrapping_add(1 as size_t) as isize)
                    as ::core::ffi::c_int
                    == 'g' as ::core::ffi::c_int
                    && *src.offset((*usedlen).wrapping_add(2 as size_t) as isize)
                        as ::core::ffi::c_int
                        == 's' as ::core::ffi::c_int))
        {
            break;
        }
        let mut didit: bool = false_0 != 0;
        let mut flags: *mut ::core::ffi::c_char =
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        s = src
            .offset(*usedlen as isize)
            .offset(2 as ::core::ffi::c_int as isize);
        if *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
            == 'g' as ::core::ffi::c_int
        {
            flags = b"g\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            s = s.offset(1);
        }
        let c2rust_fresh0 = s;
        s = s.offset(1);
        let mut sep: ::core::ffi::c_int = *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
        if sep == 0 {
            break;
        }
        p = vim_strchr(s, sep);
        if !p.is_null() {
            let pat: *mut ::core::ffi::c_char =
                xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                    as *mut ::core::ffi::c_char;
            s = p.offset(1 as ::core::ffi::c_int as isize);
            p = vim_strchr(s, sep);
            if !p.is_null() {
                let sub: *mut ::core::ffi::c_char =
                    xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                        as *mut ::core::ffi::c_char;
                let str: *mut ::core::ffi::c_char =
                    xmemdupz(*fnamep as *const ::core::ffi::c_void, *fnamelen)
                        as *mut ::core::ffi::c_char;
                *usedlen = p.offset(1 as ::core::ffi::c_int as isize).offset_from(src) as size_t;
                let mut slen: size_t = 0;
                s = do_string_sub(
                    str,
                    *fnamelen,
                    pat,
                    sub,
                    ::core::ptr::null_mut::<typval_T>(),
                    flags,
                    &raw mut slen,
                );
                *fnamep = s;
                *fnamelen = slen;
                xfree(*bufp as *mut ::core::ffi::c_void);
                *bufp = s;
                didit = true_0 != 0;
                xfree(sub as *mut ::core::ffi::c_void);
                xfree(str as *mut ::core::ffi::c_void);
            }
            xfree(pat as *mut ::core::ffi::c_void);
        }
        if !didit {
            break;
        }
    }
    if *src.offset(*usedlen as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        && *src.offset((*usedlen).wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
            == 'S' as ::core::ffi::c_int
    {
        c = *(*fnamep).offset(*fnamelen as isize) as uint8_t as ::core::ffi::c_int;
        if c != NUL {
            *(*fnamep).offset(*fnamelen as isize) = NUL as ::core::ffi::c_char;
        }
        p = vim_strsave_shellescape(*fnamep, false_0 != 0, false_0 != 0);
        if c != NUL {
            *(*fnamep).offset(*fnamelen as isize) = c as ::core::ffi::c_char;
        }
        xfree(*bufp as *mut ::core::ffi::c_void);
        *fnamep = p;
        *bufp = *fnamep;
        *fnamelen = strlen(p);
        *usedlen = (*usedlen).wrapping_add(2 as size_t);
    }
    return valid;
}
#[no_mangle]
pub unsafe extern "C" fn f_chdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut cwd: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if os_dirname(cwd, MAXPATHL as size_t) != FAIL {
        (*rettv).vval.v_string = xstrdup(cwd);
    }
    xfree(cwd as *mut ::core::ffi::c_void);
    let mut scope: CdScope = kCdScopeGlobal;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut s: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        if strcmp(s, b"global\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        {
            scope = kCdScopeGlobal;
        } else if strcmp(s, b"tabpage\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            scope = kCdScopeTabpage;
        } else if strcmp(s, b"window\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            scope = kCdScopeWindow;
        } else {
            semsg(
                gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                b"scope\0".as_ptr() as *const ::core::ffi::c_char,
                s,
            );
            return;
        }
    } else if !(*curwin.get()).w_localdir.is_null() {
        scope = kCdScopeWindow;
    } else if !(*curtab.get()).tp_localdir.is_null() {
        scope = kCdScopeTabpage;
    }
    if !changedir_func(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_string,
        scope,
    ) {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*rettv).vval.v_string as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_delete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if check_secure() {
        return;
    }
    let name: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *name as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut flags: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags = tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut nbuf as *mut ::core::ffi::c_char,
        );
    } else {
        flags = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if *flags as ::core::ffi::c_int == NUL {
        (*rettv).vval.v_number = (if os_remove(name) == 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
    } else if strcmp(flags, b"d\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        (*rettv).vval.v_number = (if os_rmdir(name) == 0 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
    } else if strcmp(flags, b"rf\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        (*rettv).vval.v_number = delete_recursive(name) as varnumber_T;
    } else {
        semsg(
            gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
            flags,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_executable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    (*rettv).vval.v_number = os_can_exe(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        true_0 != 0,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_exepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonempty_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    os_can_exe(
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        &raw mut path,
        true_0 != 0,
    );
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = path;
}
#[no_mangle]
pub unsafe extern "C" fn f_filecopy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = false_0 as varnumber_T;
    if check_secure() as ::core::ffi::c_int != 0
        || tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut from: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
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
        && (from_info.stat.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t
            || from_info.stat.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t)
    {
        (*rettv).vval.v_number = (vim_copyfile(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        ) == OK) as ::core::ffi::c_int as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_filereadable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = (*p as ::core::ffi::c_int != 0
        && !os_isdir(p)
        && os_file_is_readable(p) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_filewritable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut filename: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number = os_file_is_writable(filename) as varnumber_T;
}
unsafe extern "C" fn findfilendir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut find_what: ::core::ffi::c_int,
) {
    let mut fresult: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: *mut ::core::ffi::c_char =
        if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        };
    let mut count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut first: bool = true_0 != 0;
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut pathbuf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut p: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut pathbuf as *mut ::core::ffi::c_char,
        );
        if p.is_null() {
            error = true_0 != 0;
        } else {
            if *p as ::core::ffi::c_int != NUL {
                path = p as *mut ::core::ffi::c_char;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                count = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
            }
        }
    }
    if count < 0 as ::core::ffi::c_int {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    }
    if *fname as ::core::ffi::c_int != NUL && !error {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        loop {
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xfree(fresult as *mut ::core::ffi::c_void);
            }
            fresult = find_file_in_path_option(
                if first as ::core::ffi::c_int != 0 {
                    fname as *mut ::core::ffi::c_char
                } else {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                },
                if first as ::core::ffi::c_int != 0 {
                    strlen(fname)
                } else {
                    0 as size_t
                },
                0 as ::core::ffi::c_int,
                first as ::core::ffi::c_int,
                path,
                find_what,
                (*curbuf.get()).b_ffname,
                (if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*curbuf.get()).b_p_sua as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
            first = false_0 != 0;
            if !fresult.is_null()
                && (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_list_append_string((*rettv).vval.v_list, fresult, -1 as ssize_t);
            }
            if !(((*rettv).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    count -= 1;
                    count > 0 as ::core::ffi::c_int
                })
                && !fresult.is_null())
            {
                break;
            }
        }
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    }
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_string = fresult;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_finddir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    findfilendir(argvars, rettv, FINDFILE_DIR as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn f_findfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    findfilendir(argvars, rettv, FINDFILE_FILE as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn f_fnamemodify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mods: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if mods.is_null() || fname.is_null() {
        fname = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        len = strlen(fname);
        if *mods as ::core::ffi::c_int != NUL {
            let mut usedlen: size_t = 0 as size_t;
            modify_fname(
                mods as *mut ::core::ffi::c_char,
                false_0 != 0,
                &raw mut usedlen,
                &raw mut fname as *mut *mut ::core::ffi::c_char,
                &raw mut fbuf,
                &raw mut len,
            );
        }
    }
    (*rettv).v_type = VAR_STRING;
    if fname.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string =
            xmemdupz(fname as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
    }
    xfree(fbuf as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn f_getcwd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut scope: CdScope = kCdScopeInvalid;
    let mut scope_number: [::core::ffi::c_int; 2] =
        [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int];
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut from: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tp: *mut tabpage_T = curtab.get();
    let mut win: *mut win_T = curwin.get();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = kCdScopeWindow as ::core::ffi::c_int;
    while i < kCdScopeGlobal as ::core::ffi::c_int {
        if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        scope_number[i as usize] =
            (*argvars.offset(i as isize)).vval.v_number as ::core::ffi::c_int;
        if scope_number[i as usize] < -1 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        if scope_number[i as usize] >= 0 as ::core::ffi::c_int
            && scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int
        {
            scope = i as CdScope;
        } else if scope_number[i as usize] < 0 as ::core::ffi::c_int {
            scope = (i + 1 as ::core::ffi::c_int) as CdScope;
        }
        i += 1;
    }
    if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
        tp = find_tabpage(scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]);
        if tp.is_null() {
            emsg(gettext(
                b"E5000: Cannot find tab number.\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
    }
    if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E5001: Higher scope cannot be -1 if lower scope is >= 0.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
            win = find_win_by_nr(argvars.offset(0 as ::core::ffi::c_int as isize), tp);
            if win.is_null() {
                emsg(gettext(
                    b"E5002: Cannot find window number.\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
        }
    }
    cwd = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    's_250: {
        'c_30008: {
            'c_30005: {
                match scope as ::core::ffi::c_int {
                    0 => {
                        '_c2rust_label: {
                            if !win.is_null() {
                            } else {
                                __assert_fail(
                                    b"win\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    701 as ::core::ffi::c_uint,
                                    b"void f_getcwd(typval_T *, typval_T *, EvalFuncData)\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        from = (*win).w_localdir;
                        if !from.is_null() {
                            break 's_250;
                        }
                    }
                    1 => {}
                    2 => {
                        break 'c_30005;
                    }
                    -1 => {
                        break 'c_30008;
                    }
                    _ => {
                        break 's_250;
                    }
                }
                '_c2rust_label_0: {
                    if !tp.is_null() {
                    } else {
                        __assert_fail(
                            b"tp\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            708 as ::core::ffi::c_uint,
                            b"void f_getcwd(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                from = (*tp).tp_localdir;
                if !from.is_null() {
                    break 's_250;
                }
            }
            if !(*globaldir.ptr()).is_null() {
                from = globaldir.get();
                break 's_250;
            }
        }
        if os_dirname(cwd, MAXPATHL as size_t) == FAIL {
            from = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    }
    if !from.is_null() {
        xstrlcpy(cwd, from, MAXPATHL as size_t);
    }
    (*rettv).vval.v_string = xstrdup(cwd);
    xfree(cwd as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn f_getfperm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut perm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"rwx\0");
    let mut filename: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut file_perm: int32_t = os_getperm(filename);
    if file_perm >= 0 as int32_t {
        perm = xstrdup(b"---------\0".as_ptr() as *const ::core::ffi::c_char);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 9 as ::core::ffi::c_int {
            if file_perm & (1 as int32_t) << 8 as ::core::ffi::c_int - i != 0 {
                *perm.offset(i as isize) = flags[(i % 3 as ::core::ffi::c_int) as usize];
            }
            i += 1;
        }
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = perm;
}
#[no_mangle]
pub unsafe extern "C" fn f_getfsize(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_NUMBER;
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
    if os_fileinfo(fname, &raw mut file_info) {
        let mut filesize: uint64_t = os_fileinfo_size(&raw mut file_info);
        if os_isdir(fname) {
            (*rettv).vval.v_number = 0 as varnumber_T;
        } else {
            (*rettv).vval.v_number = filesize as varnumber_T;
            if (*rettv).vval.v_number as uint64_t != filesize {
                (*rettv).vval.v_number = -2 as varnumber_T;
            }
        }
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_getftime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
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
    if os_fileinfo(fname, &raw mut file_info) {
        (*rettv).vval.v_number = file_info.stat.st_mtim.tv_sec as varnumber_T;
    } else {
        (*rettv).vval.v_number = -1 as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_getftype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut type_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_STRING;
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
    if os_fileinfo_link(fname, &raw mut file_info) {
        let mut mode: uint64_t = file_info.stat.st_mode;
        if mode & __S_IFMT as uint64_t == 0o100000 as uint64_t {
            t = b"file\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
            t = b"dir\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o120000 as uint64_t {
            t = b"link\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
            t = b"bdev\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o20000 as uint64_t {
            t = b"cdev\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o10000 as uint64_t {
            t = b"fifo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if mode & __S_IFMT as uint64_t == 0o140000 as uint64_t {
            t = b"socket\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            t = b"other\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        type_0 = xstrdup(t);
    }
    (*rettv).vval.v_string = type_0;
}
#[no_mangle]
pub unsafe extern "C" fn f_glob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut options: ::core::ffi::c_int =
        WILD_SILENT as ::core::ffi::c_int | WILD_USE_NL as ::core::ffi::c_int;
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
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        {
            options |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
            }
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
            {
                options |= WILD_ALLLINKS as ::core::ffi::c_int;
            }
        }
    }
    if !error {
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
        if p_wic.get() != 0 {
            options += WILD_ICASE as ::core::ffi::c_int;
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string = ExpandOne(
                &raw mut xpc,
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                options,
                WILD_ALL as ::core::ffi::c_int,
            );
        } else {
            ExpandOne(
                &raw mut xpc,
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char,
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
            ExpandCleanup(&raw mut xpc);
        }
    } else {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_globpath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut flags: ::core::ffi::c_int = WILD_IGNORE_COMPLETESLASH as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        {
            flags |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
            }
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_get_number_chk(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
            {
                flags |= WILD_ALLLINKS as ::core::ffi::c_int;
            }
        }
    }
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let file: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    if !file.is_null() && !error {
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
        globpath(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char,
            file as *mut ::core::ffi::c_char,
            &raw mut ga,
            flags,
            false_0 != 0,
        );
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string =
                ga_concat_strings(&raw mut ga, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            tv_list_alloc_ret(rettv, ga.ga_len as ptrdiff_t);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ga.ga_len {
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize),
                    -1 as ssize_t,
                );
                i += 1;
            }
        }
        ga_clear_strings(&raw mut ga);
    } else {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_glob2regpat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let pat: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = if pat.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        file_pat_to_reg_pat(
            pat,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        )
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_haslocaldir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut scope: CdScope = kCdScopeInvalid;
    let mut scope_number: [::core::ffi::c_int; 2] =
        [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int];
    let mut tp: *mut tabpage_T = curtab.get();
    let mut win: *mut win_T = curwin.get();
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut i: ::core::ffi::c_int = kCdScopeWindow as ::core::ffi::c_int;
    while i < kCdScopeGlobal as ::core::ffi::c_int {
        if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        scope_number[i as usize] =
            (*argvars.offset(i as isize)).vval.v_number as ::core::ffi::c_int;
        if scope_number[i as usize] < -1 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        if scope_number[i as usize] >= 0 as ::core::ffi::c_int
            && scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int
        {
            scope = i as CdScope;
        } else if scope_number[i as usize] < 0 as ::core::ffi::c_int {
            scope = (i + 1 as ::core::ffi::c_int) as CdScope;
        }
        i += 1;
    }
    if scope as ::core::ffi::c_int == kCdScopeInvalid as ::core::ffi::c_int {
        scope = kCdScopeWindow;
    }
    if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
        tp = find_tabpage(scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize]);
        if tp.is_null() {
            emsg(gettext(
                b"E5000: Cannot find tab number.\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
    }
    if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
        if scope_number[kCdScopeTabpage as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E5001: Higher scope cannot be -1 if lower scope is >= 0.\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
        if scope_number[kCdScopeWindow as ::core::ffi::c_int as usize] > 0 as ::core::ffi::c_int {
            win = find_win_by_nr(argvars.offset(0 as ::core::ffi::c_int as isize), tp);
            if win.is_null() {
                emsg(gettext(
                    b"E5002: Cannot find window number.\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
        }
    }
    match scope as ::core::ffi::c_int {
        0 => {
            '_c2rust_label: {
                if !win.is_null() {
                } else {
                    __assert_fail(
                        b"win\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1021 as ::core::ffi::c_uint,
                        b"void f_haslocaldir(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*rettv).vval.v_number = (if !(*win).w_localdir.is_null() {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as varnumber_T;
        }
        1 => {
            '_c2rust_label_0: {
                if !tp.is_null() {
                } else {
                    __assert_fail(
                        b"tp\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1025 as ::core::ffi::c_uint,
                        b"void f_haslocaldir(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            (*rettv).vval.v_number = (if !(*tp).tp_localdir.is_null() {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as varnumber_T;
        }
        -1 => {
            abort();
        }
        2 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_isabsolutepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = path_is_absolute(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_isdirectory(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = os_isdir(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    )) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_mkdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut prot: ::core::ffi::c_int = 0o755 as ::core::ffi::c_int;
    (*rettv).vval.v_number = FAIL as varnumber_T;
    if check_secure() {
        return;
    }
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let dir: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if *dir as ::core::ffi::c_int == NUL {
        return;
    }
    if *path_tail(dir) as ::core::ffi::c_int == NUL {
        *path_tail_with_sep(dir as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
    }
    let mut defer: bool = false_0 != 0;
    let mut defer_recurse: bool = false_0 != 0;
    let mut created: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prot = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as ::core::ffi::c_int;
            if prot == -1 as ::core::ffi::c_int {
                return;
            }
        }
        let mut arg2: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        defer = !vim_strchr(arg2, 'D' as ::core::ffi::c_int).is_null();
        defer_recurse = !vim_strchr(arg2, 'R' as ::core::ffi::c_int).is_null();
        if (defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0)
            && !can_add_defer()
        {
            return;
        }
        if !vim_strchr(arg2, 'p' as ::core::ffi::c_int).is_null() {
            let mut failed_dir: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut ret: ::core::ffi::c_int = os_mkdir_recurse(
                dir,
                prot as int32_t,
                &raw mut failed_dir,
                if defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0 {
                    &raw mut created
                } else {
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>()
                },
            );
            if ret != 0 as ::core::ffi::c_int {
                semsg(
                    gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
                    failed_dir,
                    uv_strerror(ret),
                );
                xfree(failed_dir as *mut ::core::ffi::c_void);
                (*rettv).vval.v_number = FAIL as varnumber_T;
                return;
            }
            (*rettv).vval.v_number = OK as varnumber_T;
        }
    }
    if (*rettv).vval.v_number == FAIL as varnumber_T {
        (*rettv).vval.v_number = vim_mkdir_emsg(dir, prot) as varnumber_T;
    }
    if (*rettv).vval.v_number == OK as varnumber_T
        && created.is_null()
        && (defer as ::core::ffi::c_int != 0 || defer_recurse as ::core::ffi::c_int != 0)
    {
        created = FullName_save(dir, false_0 != 0);
    }
    if !created.is_null() {
        let mut tv: [typval_T; 2] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 2];
        tv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        tv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        tv[0 as ::core::ffi::c_int as usize].vval.v_string = created;
        tv[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        tv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        tv[1 as ::core::ffi::c_int as usize].vval.v_string =
            xstrdup(if defer_recurse as ::core::ffi::c_int != 0 {
                b"rf\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"d\0".as_ptr() as *const ::core::ffi::c_char
            });
        add_defer(
            b"delete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            2 as ::core::ffi::c_int,
            &raw mut tv as *mut typval_T,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_pathshorten(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut trim_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        trim_len =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if trim_len < 1 as ::core::ffi::c_int {
            trim_len = 1 as ::core::ffi::c_int;
        }
    }
    (*rettv).v_type = VAR_STRING;
    let mut p: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if p.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = xstrdup(p);
        shorten_dir_len((*rettv).vval.v_string, trim_len);
    };
}
unsafe extern "C" fn readdir_checkitem(
    mut context: *mut ::core::ffi::c_void,
    mut name: *const ::core::ffi::c_char,
) -> varnumber_T {
    let mut expr: *mut typval_T = context as *mut typval_T;
    let mut argv: [typval_T; 2] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 2];
    let mut retval: varnumber_T = 0 as varnumber_T;
    let mut error: bool = false_0 != 0;
    if (*expr).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 1 as varnumber_T;
    }
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    set_vim_var_string(VV_VAL, name, -1 as ptrdiff_t);
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    argv[0 as ::core::ffi::c_int as usize].vval.v_string = name as *mut ::core::ffi::c_char;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv as *mut typval_T,
        1 as ::core::ffi::c_int,
        &raw mut rettv,
    ) != FAIL
    {
        retval = tv_get_number_chk(&raw mut rettv, &raw mut error);
        if error {
            retval = -1 as varnumber_T;
        }
        tv_clear(&raw mut rettv);
    }
    set_vim_var_string(
        VV_VAL,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn f_readdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut path: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut expr: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut ret: ::core::ffi::c_int = readdir_core(
        &raw mut ga,
        path,
        expr as *mut ::core::ffi::c_void,
        Some(
            readdir_checkitem
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                ) -> varnumber_T,
        ),
    );
    if ret == OK && ga.ga_len > 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut p: *const ::core::ffi::c_char =
                *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize);
            tv_list_append_string((*rettv).vval.v_list, p, -1 as ssize_t);
            i += 1;
        }
    }
    ga_clear_strings(&raw mut ga);
}
unsafe extern "C" fn read_blob(
    fd: *mut FILE,
    mut rettv: *mut typval_T,
    mut offset: off_T,
    mut size_arg: off_T,
) -> ::core::ffi::c_int {
    let blob: *mut blob_T = (*rettv).vval.v_blob;
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
    if !os_fileinfo_fd(fileno(fd), &raw mut file_info) {
        return FAIL;
    }
    let mut whence: ::core::ffi::c_int = 0;
    let mut size: off_T = size_arg;
    let file_size: off_T = os_fileinfo_size(&raw mut file_info) as off_T;
    if offset >= 0 as off_T {
        if size == -1 as off_T
            || size > file_size - offset
                && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
        {
            size = os_fileinfo_size(&raw mut file_info) as off_T - offset;
        }
        whence = SEEK_SET;
    } else {
        if -offset > file_size
            && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
        {
            offset = -file_size;
        }
        if size == -1 as off_T || size > -offset {
            size = -offset;
        }
        whence = SEEK_END;
    }
    if size <= 0 as off_T {
        return OK;
    }
    if offset != 0 as off_T && fseeko(fd, offset as __off_t, whence) != 0 as ::core::ffi::c_int {
        return OK;
    }
    ga_grow(&raw mut (*blob).bv_ga, size as ::core::ffi::c_int);
    (*blob).bv_ga.ga_len = size as ::core::ffi::c_int;
    if (fread(
        (*blob).bv_ga.ga_data,
        1 as size_t,
        (*blob).bv_ga.ga_len as size_t,
        fd,
    ) as size_t)
        < (*blob).bv_ga.ga_len as size_t
    {
        tv_blob_free((*rettv).vval.v_blob);
        (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn read_file_or_blob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut always_blob: bool,
) {
    let mut binary: bool = false_0 != 0;
    let mut blob: bool = always_blob;
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut io_size: ::core::ffi::c_int =
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as ::core::ffi::c_int;
    let mut prev: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prevlen: ptrdiff_t = 0 as ptrdiff_t;
    let mut prevsize: ptrdiff_t = 0 as ptrdiff_t;
    let mut maxline: int64_t = MAXLNUM as ::core::ffi::c_int as int64_t;
    let mut offset: off_T = 0 as off_T;
    let mut size: off_T = -1 as off_T;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if always_blob {
            offset = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as off_T;
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                size = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as off_T;
            }
        } else {
            if strcmp(
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                b"b\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                binary = true_0 != 0;
            } else if strcmp(
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                b"B\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                blob = true_0 != 0;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                maxline =
                    tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as int64_t;
            }
        }
    }
    if blob {
        tv_blob_alloc_ret(rettv);
    } else {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    }
    let fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if os_isdir(fname) {
        semsg(
            gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    if *fname as ::core::ffi::c_int == NUL || {
        fd = os_fopen(fname, READBIN.as_ptr());
        fd.is_null()
    } {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            if *fname as ::core::ffi::c_int == NUL {
                gettext(b"<empty>\0".as_ptr() as *const ::core::ffi::c_char)
                    as *const ::core::ffi::c_char
            } else {
                fname
            },
        );
        return;
    }
    if blob {
        if read_blob(fd, rettv, offset, size) == FAIL {
            semsg(
                gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                fname,
            );
        }
        fclose(fd);
        return;
    }
    let l: *mut list_T = (*rettv).vval.v_list;
    while maxline < 0 as int64_t || (tv_list_len(l) as int64_t) < maxline {
        let mut readlen: ::core::ffi::c_int = fread(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            1 as size_t,
            io_size as size_t,
            fd,
        ) as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = &raw mut buf as *mut ::core::ffi::c_char;
        start = &raw mut buf as *mut ::core::ffi::c_char;
        while p < (&raw mut buf as *mut ::core::ffi::c_char).offset(readlen as isize)
            || readlen <= 0 as ::core::ffi::c_int
                && (prevlen > 0 as ptrdiff_t || binary as ::core::ffi::c_int != 0)
        {
            if readlen <= 0 as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                let mut s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut len: size_t = p.offset_from(start) as size_t;
                if readlen > 0 as ::core::ffi::c_int && !binary {
                    while len > 0 as size_t
                        && *start.offset(len.wrapping_sub(1 as size_t) as isize)
                            as ::core::ffi::c_int
                            == '\r' as ::core::ffi::c_int
                    {
                        len = len.wrapping_sub(1);
                    }
                    if len == 0 as size_t {
                        while prevlen > 0 as ptrdiff_t
                            && *prev.offset((prevlen - 1 as ptrdiff_t) as isize)
                                as ::core::ffi::c_int
                                == '\r' as ::core::ffi::c_int
                        {
                            prevlen -= 1;
                        }
                    }
                }
                if prevlen == 0 as ptrdiff_t {
                    '_c2rust_label: {
                        if len < 2147483647 as ::core::ffi::c_int as size_t {
                        } else {
                            __assert_fail(
                                b"len < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1349 as ::core::ffi::c_uint,
                                b"void read_file_or_blob(typval_T *, typval_T *, _Bool)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    s = xmemdupz(start as *const ::core::ffi::c_void, len)
                        as *mut ::core::ffi::c_char;
                } else {
                    s = xrealloc(
                        prev as *mut ::core::ffi::c_void,
                        (prevlen as size_t)
                            .wrapping_add(len)
                            .wrapping_add(1 as size_t),
                    ) as *mut ::core::ffi::c_char;
                    memcpy(
                        s.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        len,
                    );
                    *s.offset((prevlen as size_t).wrapping_add(len) as isize) =
                        NUL as ::core::ffi::c_char;
                    prev = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    prevsize = 0 as ptrdiff_t;
                    prevlen = prevsize;
                }
                tv_list_append_owned_tv(
                    l,
                    typval_T {
                        v_type: VAR_STRING,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_string: s },
                    },
                );
                start = p.offset(1 as ::core::ffi::c_int as isize);
                if maxline < 0 as int64_t {
                    if tv_list_len(l) as int64_t > -maxline {
                        '_c2rust_label_0: {
                            if tv_list_len(l) as int64_t == 1 as int64_t + -maxline {
                            } else {
                                __assert_fail(
                                    b"tv_list_len(l) == 1 + (-maxline)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    1371 as ::core::ffi::c_uint,
                                    b"void read_file_or_blob(typval_T *, typval_T *, _Bool)\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        tv_list_item_remove(l, tv_list_first(l));
                    }
                } else if tv_list_len(l) as int64_t >= maxline {
                    '_c2rust_label_1: {
                        if tv_list_len(l) as int64_t == maxline {
                        } else {
                            __assert_fail(
                                b"tv_list_len(l) == maxline\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1375 as ::core::ffi::c_uint,
                                b"void read_file_or_blob(typval_T *, typval_T *, _Bool)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    break;
                }
                if readlen <= 0 as ::core::ffi::c_int {
                    break;
                }
            } else if *p as ::core::ffi::c_int == NUL {
                *p = '\n' as ::core::ffi::c_char;
            } else if *p as uint8_t as ::core::ffi::c_int == 0xbf as ::core::ffi::c_int && !binary {
                let mut back1: ::core::ffi::c_char = (if p
                    >= (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                {
                    *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else if prevlen >= 1 as ptrdiff_t {
                    *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else {
                    NUL
                }) as ::core::ffi::c_char;
                let mut back2: ::core::ffi::c_char = (if p
                    >= (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(2 as ::core::ffi::c_int as isize)
                {
                    *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                } else if p
                    == (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(1 as ::core::ffi::c_int as isize)
                    && prevlen >= 1 as ptrdiff_t
                {
                    *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else if prevlen >= 2 as ptrdiff_t {
                    *prev.offset((prevlen - 2 as ptrdiff_t) as isize) as ::core::ffi::c_int
                } else {
                    NUL
                }) as ::core::ffi::c_char;
                if back2 as uint8_t as ::core::ffi::c_int == 0xef as ::core::ffi::c_int
                    && back1 as uint8_t as ::core::ffi::c_int == 0xbb as ::core::ffi::c_int
                {
                    let mut dest: *mut ::core::ffi::c_char =
                        p.offset(-(2 as ::core::ffi::c_int as isize));
                    if start == dest {
                        start = p.offset(1 as ::core::ffi::c_int as isize);
                    } else {
                        let mut adjust_prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if dest < &raw mut buf as *mut ::core::ffi::c_char {
                            adjust_prevlen = (&raw mut buf as *mut ::core::ffi::c_char)
                                .offset_from(dest)
                                as ::core::ffi::c_int;
                            dest = &raw mut buf as *mut ::core::ffi::c_char;
                        }
                        if readlen as isize
                            > p.offset_from(&raw mut buf as *mut ::core::ffi::c_char) + 1 as isize
                        {
                            memmove(
                                dest as *mut ::core::ffi::c_void,
                                p.offset(1 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (readlen as size_t)
                                    .wrapping_sub(
                                        p.offset_from(&raw mut buf as *mut ::core::ffi::c_char)
                                            as size_t,
                                    )
                                    .wrapping_sub(1 as size_t),
                            );
                        }
                        readlen -= 3 as ::core::ffi::c_int - adjust_prevlen;
                        prevlen -= adjust_prevlen as ptrdiff_t;
                        p = dest.offset(-(1 as ::core::ffi::c_int as isize));
                    }
                }
            }
            p = p.offset(1);
        }
        if maxline >= 0 as int64_t && tv_list_len(l) as int64_t >= maxline
            || readlen <= 0 as ::core::ffi::c_int
        {
            break;
        }
        if start < p {
            if p.offset_from(start) + prevlen as isize >= prevsize {
                if prevsize == 0 as ptrdiff_t {
                    prevsize = p.offset_from(start) as ptrdiff_t;
                } else {
                    let mut grow50pc: ptrdiff_t = prevsize * 3 as ptrdiff_t / 2 as ptrdiff_t;
                    let mut growmin: ptrdiff_t = p.offset_from(start) * 2 as ptrdiff_t + prevlen;
                    prevsize = if grow50pc > growmin {
                        grow50pc
                    } else {
                        growmin
                    };
                }
                prev = xrealloc(prev as *mut ::core::ffi::c_void, prevsize as size_t)
                    as *mut ::core::ffi::c_char;
            }
            memmove(
                prev.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                start as *const ::core::ffi::c_void,
                p.offset_from(start) as size_t,
            );
            prevlen = (prevlen as ::core::ffi::c_long + p.offset_from(start) as ::core::ffi::c_long)
                as ptrdiff_t;
        }
    }
    xfree(prev as *mut ::core::ffi::c_void);
    fclose(fd);
}
#[no_mangle]
pub unsafe extern "C" fn f_readblob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    read_file_or_blob(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_readfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    read_file_or_blob(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_rename(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        (*rettv).vval.v_number = vim_rename(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut buf as *mut ::core::ffi::c_char,
            ),
        ) as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_resolve(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let mut fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut is_relative_to_current: bool = false_0 != 0;
    let mut has_trailing_pathsep: bool = false_0 != 0;
    let mut limit: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = xstrdup(fname);
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
    {
        is_relative_to_current = true_0 != 0;
    }
    let mut len: ptrdiff_t = strlen(p) as ptrdiff_t;
    if len > 1 as ptrdiff_t && after_pathsep(p, p.offset(len as isize)) != 0 {
        has_trailing_pathsep = true_0 != 0;
        *p.offset((len - 1 as ptrdiff_t) as isize) = NUL as ::core::ffi::c_char;
    }
    let mut q: *mut ::core::ffi::c_char = path_next_component(p) as *mut ::core::ffi::c_char;
    let mut remain: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *q as ::core::ffi::c_int != NUL {
        remain = xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)));
        *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    }
    let buf: *mut ::core::ffi::c_char = xmallocz(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    let mut cpy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        loop {
            len = readlink(p, buf, MAXPATHL as size_t) as ptrdiff_t;
            if len <= 0 as ptrdiff_t {
                break;
            }
            *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
            let c2rust_fresh1 = limit;
            limit = limit - 1;
            if c2rust_fresh1 == 0 as ::core::ffi::c_int {
                xfree(p as *mut ::core::ffi::c_void);
                xfree(remain as *mut ::core::ffi::c_void);
                emsg(gettext(
                    b"E655: Too many symbolic links (cycle?)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                xfree(buf as *mut ::core::ffi::c_void);
                return;
            }
            if remain.is_null() && has_trailing_pathsep as ::core::ffi::c_int != 0 {
                add_pathsep(buf);
            }
            q = path_next_component(
                if vim_ispathsep(*buf as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                    buf.offset(1 as ::core::ffi::c_int as isize)
                } else {
                    buf
                },
            ) as *mut ::core::ffi::c_char;
            if *q as ::core::ffi::c_int != NUL {
                cpy = remain;
                remain = if !remain.is_null() {
                    concat_str(q.offset(-(1 as ::core::ffi::c_int as isize)), remain)
                } else {
                    xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)))
                };
                xfree(cpy as *mut ::core::ffi::c_void);
                *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            }
            q = path_tail(p);
            if q > p && *q as ::core::ffi::c_int == NUL {
                *p.offset((q.offset_from(p) - 1 as isize) as isize) = NUL as ::core::ffi::c_char;
                q = path_tail(p);
            }
            if q > p && !path_is_absolute(buf) {
                let p_len: size_t = strlen(p);
                let buf_len: size_t = strlen(buf);
                p = xrealloc(
                    p as *mut ::core::ffi::c_void,
                    p_len.wrapping_add(buf_len).wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                memcpy(
                    path_tail(p) as *mut ::core::ffi::c_void,
                    buf as *const ::core::ffi::c_void,
                    buf_len.wrapping_add(1 as size_t),
                );
            } else {
                xfree(p as *mut ::core::ffi::c_void);
                p = xstrdup(buf);
            }
        }
        if remain.is_null() {
            break;
        }
        q = path_next_component(remain.offset(1 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
        len = (q.offset_from(remain)
            - (*q as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as isize)
            as ptrdiff_t;
        let p_len_0: size_t = strlen(p);
        cpy = xmallocz(p_len_0.wrapping_add(len as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            cpy as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            p_len_0.wrapping_add(1 as size_t),
        );
        xstrlcat(
            cpy.offset(p_len_0 as isize),
            remain,
            (len as size_t).wrapping_add(1 as size_t),
        );
        xfree(p as *mut ::core::ffi::c_void);
        p = cpy;
        if *q as ::core::ffi::c_int != NUL {
            memmove(
                remain as *mut ::core::ffi::c_void,
                q.offset(-(1 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void,
                strlen(q.offset(-(1 as ::core::ffi::c_int as isize))).wrapping_add(1 as size_t),
            );
        } else {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut remain as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
    }
    if !vim_ispathsep(*p as ::core::ffi::c_int) {
        if is_relative_to_current as ::core::ffi::c_int != 0
            && *p as ::core::ffi::c_int != NUL
            && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                            || vim_ispathsep(
                                *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0)))
        {
            cpy = concat_str(b"./\0".as_ptr() as *const ::core::ffi::c_char, p);
            xfree(p as *mut ::core::ffi::c_void);
            p = cpy;
        } else if !is_relative_to_current {
            q = p;
            while *q.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && vim_ispathsep(*q.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                q = q.offset(2 as ::core::ffi::c_int as isize);
            }
            if q > p {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(2 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
        }
    }
    if !has_trailing_pathsep {
        q = p.offset(strlen(p) as isize);
        if after_pathsep(p, q) != 0 {
            *path_tail_with_sep(p) = NUL as ::core::ffi::c_char;
        }
    }
    (*rettv).vval.v_string = p;
    xfree(buf as *mut ::core::ffi::c_void);
    simplify_filename((*rettv).vval.v_string);
}
#[no_mangle]
pub unsafe extern "C" fn f_simplify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_string = xstrdup(p);
    simplify_filename((*rettv).vval.v_string);
    (*rettv).v_type = VAR_STRING;
}
#[no_mangle]
pub unsafe extern "C" fn f_tempname(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = vim_tempname();
}
unsafe extern "C" fn write_list(
    fp: *mut FileDescriptor,
    list: *const list_T,
    binary: bool,
) -> bool {
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = list;
    '_write_list_error: {
        's_131: {
            if !l_.is_null() {
                let mut li: *const listitem_T = (*l_).lv_first;
                loop {
                    if li.is_null() {
                        break 's_131;
                    }
                    let s: *const ::core::ffi::c_char = tv_get_string_chk(&raw const (*li).li_tv);
                    if s.is_null() {
                        return false;
                    }
                    let mut hunk_start: *const ::core::ffi::c_char = s;
                    let mut p: *const ::core::ffi::c_char = hunk_start;
                    loop {
                        if *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                        {
                            if p != hunk_start {
                                let written: ptrdiff_t =
                                    file_write(fp, hunk_start, p.offset_from(hunk_start) as size_t);
                                if written < 0 as ptrdiff_t {
                                    error = written as ::core::ffi::c_int;
                                    break '_write_list_error;
                                }
                            }
                            if *p as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                                break;
                            }
                            hunk_start = p.offset(1 as ::core::ffi::c_int as isize);
                            let mut c2rust_lvalue: [::core::ffi::c_char; 1] =
                                ['\0' as ::core::ffi::c_char];
                            let written_0: ptrdiff_t = file_write(
                                fp,
                                &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                                1 as size_t,
                            );
                            if written_0 < 0 as ptrdiff_t {
                                error = written_0 as ::core::ffi::c_int;
                                break;
                            }
                        }
                        p = p.offset(1);
                    }
                    if !binary || !(*li).li_next.is_null() {
                        let written_1: ptrdiff_t = file_write(
                            fp,
                            b"\n\0".as_ptr() as *const ::core::ffi::c_char,
                            1 as size_t,
                        );
                        if written_1 < 0 as ptrdiff_t {
                            error = written_1 as ::core::ffi::c_int;
                            break '_write_list_error;
                        }
                    }
                    li = (*li).li_next;
                }
            }
        }
        error = file_flush(fp);
        if error == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
    }
    semsg(
        gettext((e_error_while_writing_str.ptr() as *const _) as *const ::core::ffi::c_char),
        uv_strerror(error),
    );
    return false_0 != 0;
}
unsafe extern "C" fn write_data(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
    len: size_t,
) -> bool {
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_write_blob_error: {
        if len > 0 as size_t {
            let written: ptrdiff_t = file_write(fp, data, len);
            if written < len as ptrdiff_t {
                error = written as ::core::ffi::c_int;
                break '_write_blob_error;
            }
        }
        error = file_flush(fp);
        if error == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
    }
    semsg(
        gettext((e_error_while_writing_str.ptr() as *const _) as *const ::core::ffi::c_char),
        uv_strerror(error),
    );
    return false_0 != 0;
}
unsafe extern "C" fn write_blob(fp: *mut FileDescriptor, blob: *const blob_T) -> bool {
    return write_data(
        fp,
        (*blob).bv_ga.ga_data as *const ::core::ffi::c_char,
        tv_blob_len(blob) as size_t,
    );
}
unsafe extern "C" fn write_string(
    fp: *mut FileDescriptor,
    data: *const ::core::ffi::c_char,
) -> bool {
    return write_data(fp, data, strlen(data));
}
#[no_mangle]
pub unsafe extern "C" fn f_writefile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if !tv_check_str_or_nr(&raw const (*li).li_tv) {
                    return;
                }
                li = (*li).li_next;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        && !((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && script_is_lua((*current_sctx.ptr()).sc_sid) as ::core::ffi::c_int != 0)
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            gettext(
                b"writefile() first argument must be a List or a Blob\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return;
    }
    let mut binary: bool = false_0 != 0;
    let mut append: bool = false_0 != 0;
    let mut defer: bool = false_0 != 0;
    let mut do_fsync: bool = p_fs.get() != 0;
    let mut mkdir_p: bool = false_0 != 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let flags: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(2 as ::core::ffi::c_int as isize));
        if flags.is_null() {
            return;
        }
        let mut p: *const ::core::ffi::c_char = flags;
        while *p != 0 {
            match *p as ::core::ffi::c_int {
                98 => {
                    binary = true_0 != 0;
                }
                97 => {
                    append = true_0 != 0;
                }
                68 => {
                    defer = true_0 != 0;
                }
                115 => {
                    do_fsync = true_0 != 0;
                }
                83 => {
                    do_fsync = false_0 != 0;
                }
                112 => {
                    mkdir_p = true_0 != 0;
                }
                _ => {
                    semsg(
                        gettext(b"E5060: Unknown flag: %s\0".as_ptr() as *const ::core::ffi::c_char),
                        p,
                    );
                    return;
                }
            }
            p = p.offset(1);
        }
    }
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let fname: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if fname.is_null() {
        return;
    }
    if defer as ::core::ffi::c_int != 0 && !can_add_defer() {
        return;
    }
    let mut fp: FileDescriptor = FileDescriptor {
        fd: 0,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        wr: false,
        eof: false,
        non_blocking: false,
        bytes_read: 0,
    };
    let mut error: ::core::ffi::c_int = 0;
    if *fname as ::core::ffi::c_int == NUL {
        emsg(gettext(
            b"E482: Can't open file with an empty name\0".as_ptr() as *const ::core::ffi::c_char,
        ));
    } else {
        error = file_open(
            &raw mut fp,
            fname,
            (if append as ::core::ffi::c_int != 0 {
                kFileAppend as ::core::ffi::c_int
            } else {
                kFileTruncate as ::core::ffi::c_int
            }) | (if mkdir_p as ::core::ffi::c_int != 0 {
                kFileMkDir as ::core::ffi::c_int
            } else {
                kFileCreate as ::core::ffi::c_int
            }) | kFileCreate as ::core::ffi::c_int,
            0o666 as ::core::ffi::c_int,
        );
        if error != 0 as ::core::ffi::c_int {
            semsg(
                gettext(b"E482: Can't open file %s for writing: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                fname,
                uv_strerror(error),
            );
        } else {
            if defer {
                let mut tv: typval_T = typval_T {
                    v_type: VAR_STRING,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_string: FullName_save(fname, false_0 != 0),
                    },
                };
                add_defer(
                    b"delete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    &raw mut tv,
                );
            }
            let mut write_ok: bool = false;
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                write_ok = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob
                    .is_null()
                    || write_blob(
                        &raw mut fp,
                        (*argvars.offset(0 as ::core::ffi::c_int as isize))
                            .vval
                            .v_blob,
                    ) as ::core::ffi::c_int
                        != 0;
            } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                write_ok = write_string(
                    &raw mut fp,
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_string,
                );
            } else {
                write_ok = write_list(
                    &raw mut fp,
                    (*argvars.offset(0 as ::core::ffi::c_int as isize))
                        .vval
                        .v_list,
                    binary,
                );
            }
            if write_ok {
                (*rettv).vval.v_number = 0 as varnumber_T;
            }
            error = file_close(&raw mut fp, do_fsync);
            if error != 0 as ::core::ffi::c_int {
                semsg(
                    gettext(b"E80: Error when closing file %s: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    fname,
                    uv_strerror(error),
                );
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_browse(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).v_type = VAR_STRING;
}
#[no_mangle]
pub unsafe extern "C" fn f_browsedir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    f_browse(argvars, rettv, fptr);
}
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
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
