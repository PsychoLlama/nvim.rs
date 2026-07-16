extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type terminal;
    pub type multiqueue;
    pub type Unpacker;
    pub type TUIData;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn abort() -> !;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
    fn api_free_array(value: Array);
    fn api_metadata() -> Object;
    fn copy_array(array: Array, arena: *mut Arena) -> Array;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_dict_to_keydict(
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        dict: Dict,
        err: *mut Error,
    ) -> bool;
    fn KeyDict_highlight_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
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
    fn channel_connect(
        tcp: bool,
        address: *const ::core::ffi::c_char,
        rpc: bool,
        on_output: CallbackReader,
        timeout: ::core::ffi::c_int,
        error: *mut *const ::core::ffi::c_char,
    ) -> uint64_t;
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn socket_address_tcp_host_end(address: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    static mut t_colors: ::core::ffi::c_int;
    static mut stdin_isatty: bool;
    static mut stdout_isatty: bool;
    static mut stderr_isatty: bool;
    static mut time_fd: *mut FILE;
    fn dict2hlattrs(
        dict: *mut KeyDict_highlight,
        use_rgb: bool,
        link_id: *mut ::core::ffi::c_int,
        base: *mut HlAttrs,
        err: *mut Error,
    ) -> HlAttrs;
    static mut main_loop: Loop;
    fn os_exit(r: ::core::ffi::c_int) -> !;
    fn rpc_send_event(id: uint64_t, name: *const ::core::ffi::c_char, args: Array) -> bool;
    fn os_env_exists(name: *const ::core::ffi::c_char, nonempty: bool) -> bool;
    fn os_get_pid() -> int64_t;
    fn time_msg(mesg: *const ::core::ffi::c_char, start: *const proftime_T);
    fn time_finish();
    fn tui_start(
        tui_p: *mut *mut TUIData,
        width: *mut ::core::ffi::c_int,
        height: *mut ::core::ffi::c_int,
        term: *mut *mut ::core::ffi::c_char,
        rgb: *mut bool,
    );
    fn tui_stop(tui_0: *mut TUIData);
    fn tui_is_stopped(tui_0: *mut TUIData) -> bool;
    fn tui_grid_resize(tui_0: *mut TUIData, g: Integer, width: Integer, height: Integer);
    fn tui_grid_clear(tui_0: *mut TUIData, g: Integer);
    fn tui_grid_cursor_goto(tui_0: *mut TUIData, grid: Integer, row: Integer, col: Integer);
    fn tui_mode_info_set(tui_0: *mut TUIData, guicursor_enabled: bool, args: Array);
    fn tui_update_menu(tui_0: *mut TUIData);
    fn tui_busy_start(tui_0: *mut TUIData);
    fn tui_busy_stop(tui_0: *mut TUIData);
    fn tui_mouse_on(tui_0: *mut TUIData);
    fn tui_mouse_off(tui_0: *mut TUIData);
    fn tui_mode_change(tui_0: *mut TUIData, mode: String_0, mode_idx: Integer);
    fn tui_grid_scroll(
        tui_0: *mut TUIData,
        g: Integer,
        startrow: Integer,
        endrow: Integer,
        startcol: Integer,
        endcol: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn tui_add_url(tui_0: *mut TUIData, url: *const ::core::ffi::c_char) -> int32_t;
    fn tui_hl_attr_define(
        tui_0: *mut TUIData,
        id: Integer,
        attrs: HlAttrs,
        cterm_attrs: HlAttrs,
        info: Array,
    );
    fn tui_bell(tui_0: *mut TUIData);
    fn tui_visual_bell(tui_0: *mut TUIData);
    fn tui_default_colors_set(
        tui_0: *mut TUIData,
        rgb_fg: Integer,
        rgb_bg: Integer,
        rgb_sp: Integer,
        cterm_fg: Integer,
        cterm_bg: Integer,
    );
    fn tui_ui_send(tui_0: *mut TUIData, content: String_0);
    fn tui_flush(tui_0: *mut TUIData);
    fn tui_suspend(tui_0: *mut TUIData);
    fn tui_set_title(tui_0: *mut TUIData, title: String_0);
    fn tui_set_icon(tui_0: *mut TUIData, icon: String_0);
    fn tui_screenshot(tui_0: *mut TUIData, path: String_0);
    fn tui_option_set(tui_0: *mut TUIData, name: String_0, value: Object);
    fn tui_chdir(tui_0: *mut TUIData, path: String_0);
    fn tui_raw_line(
        tui_0: *mut TUIData,
        g: Integer,
        linerow: Integer,
        startcol: Integer,
        endcol: Integer,
        clearcol: Integer,
        clearattr: Integer,
        flags: LineFlags,
        chunk: *const schar_T,
        attrs: *const sattr_T,
    );
    static mut grid_line_buf_size: size_t;
    static mut grid_line_buf_char: *mut schar_T;
    static mut grid_line_buf_attr: *mut sattr_T;
    static mut ui_client_channel_id: uint64_t;
    static mut ui_client_error_exit: ::core::ffi::c_int;
    static mut ui_client_exit_status: ::core::ffi::c_int;
    static mut ui_client_attached: bool;
    static mut ui_client_forward_stdin: bool;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type gid_t = __gid_t;
pub type uid_t = __uid_t;
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
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
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
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type linenr_T = int32_t;
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
pub struct Callback {
    pub data: C2Rust_Unnamed_0,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
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
    pub fc_fixvar: [C2Rust_Unnamed_1; 12],
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
pub struct C2Rust_Unnamed_1 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_10,
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
    pub u: C2Rust_Unnamed_7,
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
    pub u: C2Rust_Unnamed_2,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
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
    pub active_reqs: C2Rust_Unnamed_6,
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
    pub timer_heap: C2Rust_Unnamed_5,
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
    pub u: C2Rust_Unnamed_4,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed_3,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_5 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_10 {
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
    pub uv: C2Rust_Unnamed_12,
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
    pub u: C2Rust_Unnamed_11,
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
pub union C2Rust_Unnamed_11 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
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
    pub u: C2Rust_Unnamed_13,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
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
    pub u: C2Rust_Unnamed_14,
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
pub union C2Rust_Unnamed_14 {
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
    pub u: C2Rust_Unnamed_15,
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
pub union C2Rust_Unnamed_15 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type OptionalKeys = uint64_t;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type FieldHashfn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub is_set__highlight_: OptionalKeys,
    pub altfont: Boolean,
    pub blink: Boolean,
    pub bold: Boolean,
    pub conceal: Boolean,
    pub dim: Boolean,
    pub italic: Boolean,
    pub nocombine: Boolean,
    pub overline: Boolean,
    pub reverse: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub undercurl: Boolean,
    pub underdashed: Boolean,
    pub underdotted: Boolean,
    pub underdouble: Boolean,
    pub underline: Boolean,
    pub default_: Boolean,
    pub cterm: Dict,
    pub foreground: Object,
    pub fg: Object,
    pub background: Object,
    pub bg: Object,
    pub ctermfg: Object,
    pub ctermbg: Object,
    pub special: Object,
    pub sp: Object,
    pub link: HLGroupID,
    pub link_global: HLGroupID,
    pub fallback: Boolean,
    pub blend: Integer,
    pub fg_indexed: Boolean,
    pub bg_indexed: Boolean,
    pub force: Boolean,
    pub update: Boolean,
    pub url: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridLineEvent {
    pub args: [::core::ffi::c_int; 3],
    pub icell: ::core::ffi::c_int,
    pub ncells: ::core::ffi::c_int,
    pub coloff: ::core::ffi::c_int,
    pub cur_attr: ::core::ffi::c_int,
    pub clear_width: ::core::ffi::c_int,
    pub wrap: bool,
}
pub type RgbValue = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_16,
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
pub union C2Rust_Unnamed_16 {
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
    pub data: C2Rust_Unnamed_17,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_17 {
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
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_18 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_18 = 1;
pub type LineFlags = ::core::ffi::c_int;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIClientHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: Option<unsafe extern "C" fn(Array) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_20,
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
    pub call_stack: C2Rust_Unnamed_19,
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
pub struct C2Rust_Unnamed_19 {
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
pub union C2Rust_Unnamed_20 {
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"void ui_client_attach(int, int, char *, _Bool)\0",
    )
};
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub const KEYDICT_INIT: KeyDict_highlight = KeyDict_highlight {
    is_set__highlight_: 0 as OptionalKeys,
    altfont: false,
    blink: false,
    bold: false,
    conceal: false,
    dim: false,
    italic: false,
    nocombine: false,
    overline: false,
    reverse: false,
    standout: false,
    strikethrough: false,
    undercurl: false,
    underdashed: false,
    underdotted: false,
    underdouble: false,
    underline: false,
    default_: false,
    cterm: Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    },
    foreground: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    fg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    background: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    bg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    ctermfg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    ctermbg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    special: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    sp: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    link: 0,
    link_global: 0,
    fallback: false,
    blend: 0,
    fg_indexed: false,
    bg_indexed: false,
    force: false,
    update: false,
    url: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
};
pub const KEYSET_OPTIDX_highlight__url: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
static mut tui: *mut TUIData = ::core::ptr::null_mut::<TUIData>();
static mut tui_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut tui_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut tui_term: *mut ::core::ffi::c_char =
    b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
static mut tui_rgb: bool = false_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn ui_client_start_server(
    mut exepath: *const ::core::ffi::c_char,
    mut argc: size_t,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> uint64_t {
    let mut args: *mut *mut ::core::ffi::c_char = xmalloc(
        (2 as size_t)
            .wrapping_add(argc)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut args_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let c2rust_fresh0 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr = &raw mut *args.offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = xstrdup(*argv.offset(0 as ::core::ffi::c_int as isize));
    let c2rust_fresh1 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr_0 = &raw mut *args.offset(c2rust_fresh1 as isize);
    *c2rust_lvalue_ptr_0 = xstrdup(b"--embed\0".as_ptr() as *const ::core::ffi::c_char);
    let mut i: size_t = 1 as size_t;
    while i < argc {
        let c2rust_fresh2 = args_idx;
        args_idx = args_idx + 1;
        let c2rust_lvalue_ptr_1 = &raw mut *args.offset(c2rust_fresh2 as isize);
        *c2rust_lvalue_ptr_1 = xstrdup(*argv.offset(i as isize));
        i = i.wrapping_add(1);
    }
    let c2rust_fresh3 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr_2 = &raw mut *args.offset(c2rust_fresh3 as isize);
    *c2rust_lvalue_ptr_2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut on_err: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_0 {
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
        args,
        exepath,
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_0 {
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
            data: C2Rust_Unnamed_0 {
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
        return 0 as uint64_t;
    }
    if ui_client_forward_stdin {
        close(0 as ::core::ffi::c_int);
        dup(if stderr_isatty as ::core::ffi::c_int != 0 {
            STDERR_FILENO
        } else {
            STDOUT_FILENO
        });
    }
    return (*channel).id;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_attach(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut term: *mut ::core::ffi::c_char,
    mut rgb: bool,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh4 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: width as Integer,
        },
    };
    let c2rust_fresh5 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: height as Integer,
        },
    };
    let mut opts: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut opts__items: [KeyValuePair; 9] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 9];
    opts.capacity = 9 as size_t;
    opts.items = &raw mut opts__items as *mut KeyValuePair;
    let c2rust_fresh6 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: rgb },
        },
    };
    let c2rust_fresh7 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh7 as isize) = key_value_pair {
        key: cstr_as_string(b"ext_linegrid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: true },
        },
    };
    let c2rust_fresh8 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh8 as isize) = key_value_pair {
        key: cstr_as_string(b"ext_termcolors\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: true },
        },
    };
    if !term.is_null() {
        let c2rust_fresh9 = opts.size;
        opts.size = opts.size.wrapping_add(1);
        *opts.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"term_name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(term),
                },
            },
        };
    }
    let c2rust_fresh10 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh10 as isize) = key_value_pair {
        key: cstr_as_string(b"term_colors\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: t_colors as Integer,
            },
        },
    };
    let c2rust_fresh11 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: stdin_isatty,
            },
        },
    };
    let c2rust_fresh12 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh12 as isize) = key_value_pair {
        key: cstr_as_string(b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: stdout_isatty,
            },
        },
    };
    if ui_client_forward_stdin {
        let c2rust_fresh13 = opts.size;
        opts.size = opts.size.wrapping_add(1);
        *opts.items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 3 as Integer,
                },
            },
        };
        ui_client_forward_stdin = false_0 != 0;
    }
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: opts },
    };
    rpc_send_event(
        ui_client_channel_id,
        b"nvim_ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
    ui_client_attached = true_0 != 0;
    if !time_fd.is_null() {
        time_msg(
            b"nvim_ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut args2: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args2__items: [Object; 5] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 5];
    args2.capacity = 5 as size_t;
    args2.items = &raw mut args2__items as *mut Object;
    let c2rust_fresh15 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(b"nvim-tui\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let mut m: Object = api_metadata();
    let mut version: Dict = Dict {
        size: 0 as size_t,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    '_c2rust_label: {
        if m.data.dict.size > 0 as size_t {
        } else {
            __assert_fail(
                b"m.data.dict.size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/ui_client.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                123 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut i: size_t = 0 as size_t;
    while i < m.data.dict.size {
        if strequal(
            (*m.data.dict.items.offset(i as isize)).key.data,
            b"version\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            version = (*m.data.dict.items.offset(i as isize)).value.data.dict;
            break;
        } else {
            if i.wrapping_add(1 as size_t) == m.data.dict.size {
                abort();
            }
            i = i.wrapping_add(1);
        }
    }
    let c2rust_fresh16 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh16 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: version },
    };
    let c2rust_fresh17 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh17 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(b"ui\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let c2rust_fresh18 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh18 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        },
    };
    let mut info: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut info__items: [KeyValuePair; 9] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 9];
    info.capacity = 9 as size_t;
    info.items = &raw mut info__items as *mut KeyValuePair;
    let c2rust_fresh19 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"website\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(
                    b"https://neovim.io\0".as_ptr() as *const ::core::ffi::c_char
                ),
            },
        },
    };
    let c2rust_fresh20 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"license\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(b"Apache 2\0".as_ptr() as *const ::core::ffi::c_char),
            },
        },
    };
    let c2rust_fresh21 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"pid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: os_get_pid(),
            },
        },
    };
    let c2rust_fresh22 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: info },
    };
    rpc_send_event(
        ui_client_channel_id,
        b"nvim_set_client_info\0".as_ptr() as *const ::core::ffi::c_char,
        args2,
    );
    if !time_fd.is_null() {
        time_msg(
            b"nvim_set_client_info\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_detach() {
    rpc_send_event(
        ui_client_channel_id,
        b"nvim_ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
        Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        },
    );
    ui_client_attached = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_run() -> ! {
    tui_start(
        &raw mut tui,
        &raw mut tui_width,
        &raw mut tui_height,
        &raw mut tui_term,
        &raw mut tui_rgb,
    );
    ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);
    if os_env_exists(
        b"__NVIM_TEST_LOG\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    ) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_run\0".as_ptr() as *const ::core::ffi::c_char,
            163 as ::core::ffi::c_int,
            true_0 != 0,
            b"test log message\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    time_finish();
    loop {
        if !main_loop.events.is_null() && !multiqueue_empty(main_loop.events) {
            multiqueue_process_events(main_loop.events);
        } else {
            loop_poll_events(&raw mut main_loop, -1 as int64_t);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_stop() {
    ui_client_attached = false_0 != 0;
    if !tui_is_stopped(tui) {
        tui_stop(tui);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_set_size(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    if ui_client_attached {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 2] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        args.capacity = 2 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh23 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh23 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: width as Integer,
            },
        };
        let c2rust_fresh24 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh24 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: height as Integer,
            },
        };
        rpc_send_event(
            ui_client_channel_id,
            b"nvim_ui_try_resize\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    }
    tui_width = width;
    tui_height = height;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_get_redraw_handler(
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
    mut error: *mut Error,
) -> UIClientHandler {
    let mut hash: ::core::ffi::c_int = ui_client_handler_hash(name, name_len);
    if hash < 0 as ::core::ffi::c_int {
        return UIClientHandler {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_0: None,
        };
    }
    return event_handlers[hash as usize];
}
#[no_mangle]
pub unsafe extern "C" fn handle_ui_client_redraw(
    mut channel_id: uint64_t,
    mut args: Array,
    mut arena: *mut Arena,
    mut error: *mut Error,
) -> Object {
    api_set_error(
        error,
        kErrorTypeValidation,
        b"'redraw' cannot be sent as a request\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
}
unsafe extern "C" fn ui_client_dict2hlattrs(mut d: Dict, mut rgb: bool) -> HlAttrs {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut dict: KeyDict_highlight = KEYDICT_INIT;
    if !api_dict_to_keydict(
        &raw mut dict as *mut ::core::ffi::c_void,
        Some(
            KeyDict_highlight_get_field
                as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
        ),
        d,
        &raw mut err,
    ) {
        return HLATTRS_INIT;
    }
    let mut attrs: HlAttrs = dict2hlattrs(
        &raw mut dict,
        rgb,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<HlAttrs>(),
        &raw mut err,
    );
    if dict.is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__url
        != 0 as ::core::ffi::c_ulonglong
    {
        attrs.url = tui_add_url(tui, dict.url.data);
    }
    return attrs;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_grid_resize(mut args: Array) {
    if args.size < 3 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_resize\0".as_ptr() as *const ::core::ffi::c_char,
            241 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_resize'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut grid: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut width: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut height: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_resize(tui, grid, width, height);
    if grid_line_buf_size < width as size_t {
        xfree(grid_line_buf_char as *mut ::core::ffi::c_void);
        xfree(grid_line_buf_attr as *mut ::core::ffi::c_void);
        grid_line_buf_size = width as size_t;
        grid_line_buf_char =
            xmalloc(grid_line_buf_size.wrapping_mul(::core::mem::size_of::<schar_T>()))
                as *mut schar_T;
        grid_line_buf_attr =
            xmalloc(grid_line_buf_size.wrapping_mul(::core::mem::size_of::<sattr_T>()))
                as *mut sattr_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_grid_line(mut args: Array) -> ! {
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_raw_line(mut g: *mut GridLineEvent) {
    let mut grid: ::core::ffi::c_int = (*g).args[0 as ::core::ffi::c_int as usize];
    let mut row: ::core::ffi::c_int = (*g).args[1 as ::core::ffi::c_int as usize];
    let mut startcol: ::core::ffi::c_int = (*g).args[2 as ::core::ffi::c_int as usize];
    let mut endcol: Integer = (startcol + (*g).coloff) as Integer;
    let mut clearcol: Integer = endcol + (*g).clear_width as Integer;
    let mut lineflags: LineFlags = if (*g).wrap as ::core::ffi::c_int != 0 {
        kLineFlagWrap as ::core::ffi::c_int
    } else {
        0 as LineFlags
    };
    tui_raw_line(
        tui,
        grid as Integer,
        row as Integer,
        startcol as Integer,
        endcol,
        clearcol,
        (*g).cur_attr as Integer,
        lineflags,
        grid_line_buf_char as *const schar_T,
        grid_line_buf_attr,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_connect(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_connect\0".as_ptr() as *const ::core::ffi::c_char,
            282 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling UI event 'connect'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut s: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut server_addr: *mut ::core::ffi::c_char =
        xmemdupz(s.data as *const ::core::ffi::c_void, s.size) as *mut ::core::ffi::c_char;
    multiqueue_put_event(
        main_loop.fast_events,
        Event {
            handler: Some(
                channel_connect_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                server_addr as *mut ::core::ffi::c_void,
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
    ui_client_channel_id = UINT64_MAX as uint64_t;
}
unsafe extern "C" fn channel_connect_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut server_addr: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    let mut err: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut is_tcp: bool = !socket_address_tcp_host_end(server_addr).is_null();
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_0 {
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
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        50 as ::core::ffi::c_int,
        &raw mut err,
    );
    if !strequal(err, b"\0".as_ptr() as *const ::core::ffi::c_char) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"channel_connect_event\0".as_ptr() as *const ::core::ffi::c_char,
            303 as ::core::ffi::c_int,
            true_0 != 0,
            b"Cannot connect to server %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            server_addr,
            err,
        );
        xfree(server_addr as *mut ::core::ffi::c_void);
        ui_client_exit_status = 1 as ::core::ffi::c_int;
        os_exit(1 as ::core::ffi::c_int);
    }
    ui_client_channel_id = chan;
    ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"channel_connect_event\0".as_ptr() as *const ::core::ffi::c_char,
        312 as ::core::ffi::c_int,
        true_0 != 0,
        b"Connected to server %s on channel %ld\0".as_ptr() as *const ::core::ffi::c_char,
        server_addr,
        chan,
    );
    xfree(server_addr as *mut ::core::ffi::c_void);
}
static mut restart_args: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
static mut restart_pending: bool = false_0 != 0;
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_restart(mut args: Array) {
    api_free_array(restart_args);
    restart_args = copy_array(args, ::core::ptr::null_mut::<Arena>());
    restart_pending = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_attach_to_restarted_server() {
    let mut listen_addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_tcp: bool = false;
    let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut chan_id: uint64_t = 0;
    if !restart_pending {
        return;
    }
    restart_pending = false_0 != 0;
    if restart_args.size < 1 as size_t
        || (*restart_args.items.offset(0 as ::core::ffi::c_int as isize)).type_0
            as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
            343 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'restart'\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        listen_addr = (*restart_args.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .string
            .data;
        is_tcp = !socket_address_tcp_host_end(listen_addr).is_null();
        err = b"\0".as_ptr() as *const ::core::ffi::c_char;
        chan_id = channel_connect(
            is_tcp,
            listen_addr,
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: C2Rust_Unnamed_0 {
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
            50 as ::core::ffi::c_int,
            &raw mut err,
        );
        if !strequal(err, b"\0".as_ptr() as *const ::core::ffi::c_char) {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
                353 as ::core::ffi::c_int,
                true_0 != 0,
                b"cannot connect to server %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                listen_addr,
                err,
            );
        } else {
            ui_client_channel_id = chan_id;
            ui_client_attach(tui_width, tui_height, tui_term, tui_rgb);
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
                361 as ::core::ffi::c_int,
                true_0 != 0,
                b"restarted server address=%s id=%ld\0".as_ptr() as *const ::core::ffi::c_char,
                listen_addr,
                chan_id,
            );
        }
    }
    api_free_array(restart_args);
    restart_args = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_error_exit(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            372 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'error_exit'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    ui_client_error_exit = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_mode_info_set(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
            6 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'mode_info_set'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Boolean = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .boolean;
    let mut arg_2: Array = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .array;
    tui_mode_info_set(tui, arg_1 as bool, arg_2);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_update_menu(mut args: Array) {
    tui_update_menu(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_busy_start(mut args: Array) {
    tui_busy_start(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_busy_stop(mut args: Array) {
    tui_busy_stop(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_mouse_on(mut args: Array) {
    tui_mouse_on(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_mouse_off(mut args: Array) {
    tui_mouse_off(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_mode_change(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_mode_change\0".as_ptr() as *const ::core::ffi::c_char,
            44 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'mode_change'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_mode_change(tui, arg_1, arg_2);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_bell(mut args: Array) {
    tui_bell(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_visual_bell(mut args: Array) {
    tui_visual_bell(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_flush(mut args: Array) {
    tui_flush(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_suspend(mut args: Array) {
    tui_suspend(tui);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_set_title(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_set_title\0".as_ptr() as *const ::core::ffi::c_char,
            76 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'set_title'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_set_title(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_set_icon(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_set_icon\0".as_ptr() as *const ::core::ffi::c_char,
            87 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'set_icon'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_set_icon(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_screenshot(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_screenshot\0".as_ptr() as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'screenshot'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_screenshot(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_option_set(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_option_set\0".as_ptr() as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'option_set'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut arg_2: Object = *args.items.offset(1 as ::core::ffi::c_int as isize);
    tui_option_set(tui, arg_1, arg_2);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_chdir(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_chdir\0".as_ptr() as *const ::core::ffi::c_char,
            121 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'chdir'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_chdir(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_ui_send(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            132 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'ui_send'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_ui_send(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_default_colors_set(mut args: Array) {
    if args.size < 5 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(4 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
            147 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'default_colors_set'\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_4: Integer = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_5: Integer = (*args.items.offset(4 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_default_colors_set(tui, arg_1, arg_2, arg_3, arg_4, arg_5);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_hl_attr_define(mut args: Array) {
    if args.size < 4 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
            165 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'hl_attr_define'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: HlAttrs = ui_client_dict2hlattrs(
        (*args.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .dict,
        true_0 != 0,
    );
    let mut arg_3: HlAttrs = ui_client_dict2hlattrs(
        (*args.items.offset(2 as ::core::ffi::c_int as isize))
            .data
            .dict,
        false_0 != 0,
    );
    let mut arg_4: Array = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .array;
    tui_hl_attr_define(tui, arg_1, arg_2, arg_3, arg_4);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_grid_clear(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_clear\0".as_ptr() as *const ::core::ffi::c_char,
            179 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_clear'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_clear(tui, arg_1);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_grid_cursor_goto(mut args: Array) {
    if args.size < 3 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            192 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_cursor_goto'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_cursor_goto(tui, arg_1, arg_2, arg_3);
}
#[no_mangle]
pub unsafe extern "C" fn ui_client_event_grid_scroll(mut args: Array) {
    if args.size < 7 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(4 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(5 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(6 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            211 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_scroll'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_4: Integer = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_5: Integer = (*args.items.offset(4 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_6: Integer = (*args.items.offset(5 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_7: Integer = (*args.items.offset(6 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_scroll(tui, arg_1, arg_2, arg_3, arg_4, arg_5, arg_6, arg_7);
}
static mut event_handlers: [UIClientHandler; 27] = unsafe {
    [
        UIClientHandler {
            name: b"bell\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_bell as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"chdir\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_chdir as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"flush\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_flush as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"connect\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_connect as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"restart\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_restart as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_suspend as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_ui_send as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mouse_on\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mouse_on as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"set_icon\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_set_icon as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"busy_stop\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_busy_stop as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_line\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(Array) -> !>,
                Option<unsafe extern "C" fn(Array) -> ()>,
            >(Some(
                ui_client_event_grid_line as unsafe extern "C" fn(Array) -> !,
            )),
        },
        UIClientHandler {
            name: b"mouse_off\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mouse_off as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"set_title\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_set_title as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"busy_start\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_busy_start as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_error_exit as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_clear\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_clear as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"option_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_option_set as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"screenshot\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_screenshot as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mode_change\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mode_change as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"update_menu\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_update_menu as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"visual_bell\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_visual_bell as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_resize as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_scroll as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mode_info_set as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_hl_attr_define as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_cursor_goto as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_default_colors_set as unsafe extern "C" fn(Array) -> ()),
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn ui_client_handler_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
        }
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 1 as ::core::ffi::c_int;
            }
            102 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 3 as ::core::ffi::c_int;
            }
            114 => {
                low = 4 as ::core::ffi::c_int;
            }
            115 => {
                low = 5 as ::core::ffi::c_int;
            }
            117 => {
                low = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            109 => {
                low = 7 as ::core::ffi::c_int;
            }
            115 => {
                low = 8 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 9 as ::core::ffi::c_int;
            }
            103 => {
                low = 10 as ::core::ffi::c_int;
            }
            109 => {
                low = 11 as ::core::ffi::c_int;
            }
            115 => {
                low = 12 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 13 as ::core::ffi::c_int;
            }
            101 => {
                low = 14 as ::core::ffi::c_int;
            }
            103 => {
                low = 15 as ::core::ffi::c_int;
            }
            111 => {
                low = 16 as ::core::ffi::c_int;
            }
            115 => {
                low = 17 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 18 as ::core::ffi::c_int;
            }
            101 => {
                low = 19 as ::core::ffi::c_int;
            }
            108 => {
                low = 20 as ::core::ffi::c_int;
            }
            114 => {
                low = 21 as ::core::ffi::c_int;
            }
            115 => {
                low = 22 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => {
            low = 23 as ::core::ffi::c_int;
        }
        14 => {
            low = 24 as ::core::ffi::c_int;
        }
        16 => {
            low = 25 as ::core::ffi::c_int;
        }
        18 => {
            low = 26 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            event_handlers[low as usize].name as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
