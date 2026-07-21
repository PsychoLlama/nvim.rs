use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
extern "C" {
    pub type terminal;
    pub type multiqueue;
    pub type Unpacker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn alloc_block() -> *mut ::core::ffi::c_void;
    fn free_block(block: *mut ::core::ffi::c_void);
    fn arena_mem_free(mem: ArenaMem);
    fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t;
    fn map_put_ref_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut *mut uint64_t,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn map_del_uint64_t_ptr_t(
        map: *mut Map_uint64_t_ptr_t,
        key: uint64_t,
        key_alloc: *mut uint64_t,
    ) -> ptr_t;
    fn api_err_invalid(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        val_s: *const ::core::ffi::c_char,
        val_n: int64_t,
        quote_val: bool,
    );
    fn api_err_exp(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        expected: *const ::core::ffi::c_char,
        actual: *const ::core::ffi::c_char,
    );
    fn string_to_cstr(str: String_0) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_typename(t: ObjectType) -> *mut ::core::ffi::c_char;
    static ui_ext_names: GlobalCell<[*const ::core::ffi::c_char; 0]>;
    fn may_trigger_vim_suspend_resume(suspend: bool);
    fn do_autocmd_focusgained(gained: bool);
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    fn os_hrtime() -> uint64_t;
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn wstream_new_buffer(
        data: *mut ::core::ffi::c_char,
        size: size_t,
        refcount: size_t,
        cb: wbuffer_data_finalizer,
    ) -> *mut WBuffer;
    static Columns: GlobalCell<::core::ffi::c_int>;
    static current_ui: GlobalCell<uint64_t>;
    static t_colors: GlobalCell<::core::ffi::c_int>;
    static starting: GlobalCell<::core::ffi::c_int>;
    static stdin_isatty: GlobalCell<bool>;
    static stdout_isatty: GlobalCell<bool>;
    static stdin_fd: GlobalCell<::core::ffi::c_int>;
    fn schar_get(buf_out: *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    fn schar_get_adv(buf_out: *mut *mut ::core::ffi::c_char, sc: schar_T) -> size_t;
    static p_bg: GlobalCell<*mut ::core::ffi::c_char>;
    fn hl_get_url(index: uint32_t) -> *const ::core::ffi::c_char;
    fn syn_attr2entry(attr: ::core::ffi::c_int) -> HlAttrs;
    fn hlattrs2dict(
        hl: *mut Dict,
        hl_attrs: *mut Dict,
        ae: HlAttrs,
        use_rgb: bool,
        short_keys: bool,
    );
    static main_loop: SharedCell<Loop>;
    fn utf_ambiguous_width(p: *const ::core::ffi::c_char) -> bool;
    fn rpc_write_raw(id: uint64_t, buffer: *mut WBuffer) -> bool;
    fn mpack_object_array(arr: Array, packer: *mut PackerBuffer);
    fn set_tty_option(name: *const ::core::ffi::c_char, value: *mut ::core::ffi::c_char) -> bool;
    fn ui_call_ui_send(content: String_0);
    fn ui_active() -> size_t;
    fn ui_refresh();
    fn ui_can_attach_more() -> bool;
    fn ui_attach_impl(ui: *mut RemoteUI, chanid: uint64_t);
    fn ui_detach_impl(ui: *mut RemoteUI, chanid: uint64_t);
    fn ui_set_ext_option(ui: *mut RemoteUI, ext: UIExtension, active: bool);
    fn ui_grid_resize(
        grid_handle: handle_T,
        width: ::core::ffi::c_int,
        height: ::core::ffi::c_int,
        err: *mut Error,
    );
    static noargs: GlobalCell<Array>;
    static channels: GlobalCell<Map_uint64_t_ptr_t>;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
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
pub type Window = handle_T;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLATTRS_DICT_SIZE: C2Rust_Unnamed_16 = 24;
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
    pub u: C2Rust_Unnamed_17,
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
pub union C2Rust_Unnamed_17 {
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
    pub data: C2Rust_Unnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_19 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_19 = 1;
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
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_20,
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
pub struct C2Rust_Unnamed_20 {
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
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_21,
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
pub union C2Rust_Unnamed_21 {
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
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
pub type ChannelStreamType = ::core::ffi::c_uint;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub type WBuffer = wbuffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbuffer {
    pub size: size_t,
    pub refcount: size_t,
    pub data: *mut ::core::ffi::c_char,
    pub cb: wbuffer_data_finalizer,
}
pub type wbuffer_data_finalizer = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint64_t = Set_uint64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint64_t>(),
};
pub const MAP_INIT: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint64_t(mut set: *mut Set_uint64_t, mut key: uint64_t) -> bool {
    return mh_get_uint64_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
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
pub const UI_BUF_SIZE: ::core::ffi::c_int = ARENA_BLOCK_SIZE;
pub const EVENT_BUF_SIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
static connected_uis: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(MAP_INIT);
unsafe extern "C" fn get_ui_or_err(mut chan_id: uint64_t, mut err: *mut Error) -> *mut RemoteUI {
    let mut ui: *mut RemoteUI =
        map_get_uint64_t_ptr_t(connected_uis.ptr(), chan_id) as *mut RemoteUI;
    if ui.is_null() && !err.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"UI not attached to channel: %ld\0".as_ptr() as *const ::core::ffi::c_char,
            chan_id,
        );
    }
    return ui;
}
unsafe extern "C" fn mpack_array_dyn16(
    mut buf: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let c2rust_fresh4 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh4 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut pos: *mut ::core::ffi::c_char = *buf;
    mpack_w2(buf, 0xffef as uint32_t);
    return pos;
}
unsafe extern "C" fn mpack_str_small(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    '_c2rust_label: {
        if len < 0x20 as size_t {
        } else {
            __assert_fail(
                b"len < 0x20\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                71 as ::core::ffi::c_uint,
                b"void mpack_str_small(char **, const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh3 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh3 = (0xa0 as size_t | len) as ::core::ffi::c_char;
    memcpy(
        *buf as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    *buf = (*buf).offset(len as isize);
}
unsafe extern "C" fn remote_ui_destroy(mut ui: *mut RemoteUI) {
    xfree((*ui).packer.startptr as *mut ::core::ffi::c_void);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*ui).term_name as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree(ui as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_disconnect(
    mut channel_id: uint64_t,
    mut err: *mut Error,
    mut send_error_exit: bool,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if send_error_exit {
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
        let c2rust_fresh0 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        push_call(
            ui,
            b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        ui_flush_buf(ui, false_0 != 0);
    }
    map_del_uint64_t_ptr_t(
        connected_uis.ptr(),
        channel_id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    ui_detach_impl(ui, channel_id);
    let mut chan: *mut Channel = find_channel(channel_id);
    if !chan.is_null() && (*chan).rpc.ui == ui {
        (*chan).rpc.ui = ::core::ptr::null_mut::<RemoteUI>();
    }
    remote_ui_destroy(ui);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_wait_for_attach() {
    let mut remaining: int64_t = -1 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while ui_active() == 0 {
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
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_attach(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut options: Dict,
    mut err: *mut Error,
) {
    if set_has_uint64_t(&raw mut (*connected_uis.ptr()).set, channel_id) {
        api_set_error(
            err,
            kErrorTypeException,
            b"UI already attached to channel: %ld\0".as_ptr() as *const ::core::ffi::c_char,
            channel_id,
        );
        return;
    }
    if !ui_can_attach_more() {
        api_set_error(
            err,
            kErrorTypeException,
            b"Maximum UI count reached\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if width <= 0 as Integer || height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0 and height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ui: *mut RemoteUI =
        xcalloc(1 as size_t, ::core::mem::size_of::<RemoteUI>()) as *mut RemoteUI;
    (*ui).channel_id = channel_id;
    (*ui).width = width as ::core::ffi::c_int;
    (*ui).height = height as ::core::ffi::c_int;
    (*ui).pum_row = -1.0f64;
    (*ui).pum_col = -1.0f64;
    (*ui).rgb = true_0 != 0;
    memset(
        &raw mut (*ui).ui_ext as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 10]>(),
    );
    let mut i: size_t = 0 as size_t;
    while i < options.size {
        ui_set_option(
            ui,
            true_0 != 0,
            (*options.items.offset(i as isize)).key,
            (*options.items.offset(i as isize)).value,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            xfree(ui as *mut ::core::ffi::c_void);
            return;
        }
        i = i.wrapping_add(1);
    }
    if (*ui).ui_ext[kUIHlState as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
        || (*ui).ui_ext[kUIMultigrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
    {
        (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] = true_0 != 0;
    }
    if (*ui).ui_ext[kUIMessages as ::core::ffi::c_int as usize] {
        (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] = true_0 != 0;
        (*ui).ui_ext[kUICmdline as ::core::ffi::c_int as usize] = true_0 != 0;
    }
    (*ui).cur_event = ::core::ptr::null::<::core::ffi::c_char>();
    (*ui).hl_id = 0 as ::core::ffi::c_int;
    (*ui).client_col = -1 as Integer;
    (*ui).nevents_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).nevents = 0 as uint32_t;
    (*ui).flushed_events = false_0 != 0;
    (*ui).incomplete_event = false_0 != 0;
    (*ui).ncalls_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).ncalls = 0 as uint32_t;
    (*ui).ncells_pending = 0 as size_t;
    (*ui).packer = packer_buffer_t {
        startptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        endptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        anydata: ui as *mut ::core::ffi::c_void,
        anyint: 0,
        packer_flush: Some(ui_flush_callback as unsafe extern "C" fn(*mut PackerBuffer) -> ()),
    };
    (*ui).wildmenu_active = false_0 != 0;
    map_put_uint64_t_ptr_t(connected_uis.ptr(), channel_id, ui as ptr_t);
    current_ui.set(channel_id);
    ui_attach_impl(ui, channel_id);
    let mut chan: *mut Channel = find_channel(channel_id);
    if !chan.is_null() {
        (*chan).rpc.ui = ui;
    }
    may_trigger_vim_suspend_resume(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ui_attach(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut enable_rgb: Boolean,
    mut err: *mut Error,
) {
    let mut opts: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut opts__items: [KeyValuePair; 1] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 1];
    opts.capacity = 1 as size_t;
    opts.items = &raw mut opts__items as *mut KeyValuePair;
    let c2rust_fresh17 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: enable_rgb,
            },
        },
    };
    nvim_ui_attach(channel_id, width, height, opts, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_set_focus(
    mut channel_id: uint64_t,
    mut gained: Boolean,
    mut error: *mut Error,
) {
    if get_ui_or_err(channel_id, error).is_null() {
        return;
    }
    if gained {
        current_ui.set(channel_id);
        may_trigger_vim_suspend_resume(false_0 != 0);
    }
    do_autocmd_focusgained(gained);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_detach(mut channel_id: uint64_t, mut err: *mut Error) {
    remote_ui_disconnect(channel_id, err, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_connect(
    mut channel_id: uint64_t,
    mut server_addr: *mut ::core::ffi::c_char,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
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
    let c2rust_fresh18 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh18 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(server_addr),
        },
    };
    push_call(
        ui,
        b"connect\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_stop(mut _ui: *mut RemoteUI) {}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_try_resize(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if width <= 0 as Integer || height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0 and height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).width = width as ::core::ffi::c_int;
    (*ui).height = height as ::core::ffi::c_int;
    ui_refresh();
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_set_option(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut error: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, error);
    if ui.is_null() {
        return;
    }
    ui_set_option(ui, false_0 != 0, name, value, error);
}
unsafe extern "C" fn ui_set_option(
    mut ui: *mut RemoteUI,
    mut init: bool,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    if strequal(
        name.data,
        b"override\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"override\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        (*ui).override_0 = value.data.boolean as bool;
        return;
    }
    if strequal(name.data, b"rgb\0".as_ptr() as *const ::core::ffi::c_char) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"rgb\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        (*ui).rgb = value.data.boolean as bool;
        if !init && !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            ui_refresh();
        }
        return;
    }
    if strequal(
        name.data,
        b"term_name\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"term_name\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename(value.type_0),
            );
            return;
        }
        set_tty_option(
            b"term\0".as_ptr() as *const ::core::ffi::c_char,
            string_to_cstr(value.data.string),
        );
        (*ui).term_name = string_to_cstr(value.data.string);
        return;
    }
    if strequal(
        name.data,
        b"term_colors\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"term_colors\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeInteger),
                api_typename(value.type_0),
            );
            return;
        }
        t_colors.set(value.data.integer as ::core::ffi::c_int);
        (*ui).term_colors = value.data.integer as ::core::ffi::c_int;
        return;
    }
    if strequal(
        name.data,
        b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeInteger),
                api_typename(value.type_0),
            );
            return;
        }
        if !(value.data.integer >= 0 as Integer) {
            api_err_invalid(
                err,
                b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                value.data.integer as int64_t,
                false_0 != 0,
            );
            return;
        }
        if !(starting.get() == 2 as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"stdin_fd can only be used with first attached UI\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return;
        }
        stdin_fd.set(value.data.integer as ::core::ffi::c_int);
        return;
    }
    if strequal(
        name.data,
        b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        if (*ui).channel_id == CHAN_STDIO as uint64_t {
            stdin_isatty.set(value.data.boolean as bool);
        }
        (*ui).stdin_tty = value.data.boolean as bool;
        return;
    }
    if strequal(
        name.data,
        b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        if (*ui).channel_id == CHAN_STDIO as uint64_t {
            stdout_isatty.set(value.data.boolean as bool);
        }
        (*ui).stdout_tty = value.data.boolean as bool;
        return;
    }
    let mut is_popupmenu: bool = strequal(
        name.data,
        b"popupmenu_external\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut i: UIExtension = kUICmdline;
    while (i as ::core::ffi::c_uint) < kUIExtCount as ::core::ffi::c_int as ::core::ffi::c_uint {
        if strequal(
            name.data,
            *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(i as isize),
        ) as ::core::ffi::c_int
            != 0
            || i as ::core::ffi::c_uint == kUIPopupmenu as ::core::ffi::c_int as ::core::ffi::c_uint
                && is_popupmenu as ::core::ffi::c_int != 0
        {
            if !(value.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_exp(
                    err,
                    name.data,
                    b"Boolean\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(value.type_0),
                );
                return;
            }
            let mut boolval: bool = value.data.boolean as bool;
            if !init
                && i as ::core::ffi::c_uint
                    == kUILinegrid as ::core::ffi::c_int as ::core::ffi::c_uint
                && boolval as ::core::ffi::c_int != (*ui).ui_ext[i as usize] as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"ext_linegrid option cannot be changed\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
            (*ui).ui_ext[i as usize] = boolval;
            if !init {
                ui_set_ext_option(ui, i, boolval);
            }
            return;
        }
        i += 1;
    }
    if true {
        api_err_invalid(
            err,
            b"UI option\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_try_resize_grid(
    mut channel_id: uint64_t,
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
    mut err: *mut Error,
) {
    if get_ui_or_err(channel_id, err).is_null() {
        return;
    }
    if grid == DEFAULT_GRID_HANDLE as Integer {
        nvim_ui_try_resize(channel_id, width, height, err);
    } else {
        ui_grid_resize(
            grid as handle_T,
            width as ::core::ffi::c_int,
            height as ::core::ffi::c_int,
            err,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_pum_set_height(
    mut channel_id: uint64_t,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected pum height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize] {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"UI must support the ext_popupmenu option\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).pum_nlines = height as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_pum_set_bounds(
    mut channel_id: uint64_t,
    mut width: Float,
    mut height: Float,
    mut row: Float,
    mut col: Float,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize] {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"UI must support the ext_popupmenu option\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if width <= 0 as ::core::ffi::c_int as Float {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    } else if height <= 0 as ::core::ffi::c_int as Float {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).pum_row = row;
    (*ui).pum_col = col;
    (*ui).pum_width = width;
    (*ui).pum_height = height;
    (*ui).pum_pos = true_0 != 0;
}
unsafe extern "C" fn flush_event(mut ui: *mut RemoteUI) {
    if !(*ui).cur_event.is_null() {
        mpack_w2(
            &raw mut (*ui).ncalls_pos,
            (1 as uint32_t).wrapping_add((*ui).ncalls),
        );
        (*ui).cur_event = ::core::ptr::null::<::core::ffi::c_char>();
        (*ui).ncalls_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*ui).ncalls = 0 as uint32_t;
    }
}
unsafe extern "C" fn ui_alloc_buf(mut ui: *mut RemoteUI) {
    (*ui).packer.startptr = alloc_block() as *mut ::core::ffi::c_char;
    (*ui).packer.ptr = (*ui).packer.startptr;
    (*ui).packer.endptr = (*ui).packer.startptr.offset(UI_BUF_SIZE as isize);
}
unsafe extern "C" fn prepare_call(mut ui: *mut RemoteUI, mut name: *const ::core::ffi::c_char) {
    if !(*ui).packer.startptr.is_null()
        && ((*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t
            > (UI_BUF_SIZE - EVENT_BUF_SIZE) as size_t
            || (*ui).ncells_pending >= 500 as size_t)
    {
        ui_flush_buf(ui, false_0 != 0);
    }
    if (*ui).packer.startptr.is_null() {
        ui_alloc_buf(ui);
    }
    if (*ui).cur_event.is_null() || !strequal((*ui).cur_event, name) {
        let mut buf: *mut *mut ::core::ffi::c_char = &raw mut (*ui).packer.ptr;
        if (*ui).nevents_pos.is_null() {
            mpack_array(buf, 3 as uint32_t);
            mpack_uint(buf, 2 as uint32_t);
            mpack_str_small(
                buf,
                b"redraw\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            );
            (*ui).nevents_pos = mpack_array_dyn16(buf);
            '_c2rust_label: {
                if (*ui).cur_event.is_null() {
                } else {
                    __assert_fail(
                        b"ui->cur_event == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        549 as ::core::ffi::c_uint,
                        b"void prepare_call(RemoteUI *, const char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        flush_event(ui);
        (*ui).cur_event = name;
        (*ui).ncalls_pos = mpack_array_dyn16(buf);
        mpack_str_small(buf, name, strlen(name));
        (*ui).nevents = (*ui).nevents.wrapping_add(1);
        (*ui).ncalls = 1 as uint32_t;
    } else {
        (*ui).ncalls = (*ui).ncalls.wrapping_add(1);
    };
}
unsafe extern "C" fn push_call(
    mut ui: *mut RemoteUI,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
) {
    prepare_call(ui, name);
    mpack_object_array(args, &raw mut (*ui).packer);
}
unsafe extern "C" fn ui_flush_callback(mut packer: *mut PackerBuffer) {
    let mut ui: *mut RemoteUI = (*packer).anydata as *mut RemoteUI;
    ui_flush_buf(ui, true_0 != 0);
    ui_alloc_buf(ui);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_grid_clear(mut ui: *mut RemoteUI, mut grid: Integer) {
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
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let c2rust_fresh19 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh19 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
    }
    let mut name: *const ::core::ffi::c_char =
        if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0 {
            b"grid_clear\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"clear\0".as_ptr() as *const ::core::ffi::c_char
        };
    push_call(ui, name, args);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_grid_resize(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
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
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let c2rust_fresh20 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh20 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
    } else {
        (*ui).client_col = -1 as Integer;
    }
    let c2rust_fresh21 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh21 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: width },
    };
    let c2rust_fresh22 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: height },
    };
    let mut name: *const ::core::ffi::c_char =
        if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0 {
            b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"resize\0".as_ptr() as *const ::core::ffi::c_char
        };
    push_call(ui, name, args);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_grid_scroll(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 7] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 7];
        args.capacity = 7 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh23 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh23 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
        let c2rust_fresh24 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh24 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: top },
        };
        let c2rust_fresh25 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh25 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: bot },
        };
        let c2rust_fresh26 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh26 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: left },
        };
        let c2rust_fresh27 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh27 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: right },
        };
        let c2rust_fresh28 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh28 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: rows },
        };
        let c2rust_fresh29 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: cols },
        };
        push_call(
            ui,
            b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    } else {
        let mut args_0: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items_0: [Object; 4] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 4];
        args_0.capacity = 4 as size_t;
        args_0.items = &raw mut args__items_0 as *mut Object;
        let c2rust_fresh30 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh30 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: top },
        };
        let c2rust_fresh31 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh31 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: bot - 1 as Integer,
            },
        };
        let c2rust_fresh32 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh32 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: left },
        };
        let c2rust_fresh33 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh33 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: right - 1 as Integer,
            },
        };
        push_call(
            ui,
            b"set_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
        args_0.size = 0 as size_t;
        let c2rust_fresh34 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh34 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: rows },
        };
        push_call(
            ui,
            b"scroll\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
        args_0.size = 0 as size_t;
        let c2rust_fresh35 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh35 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh36 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh36 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ((*ui).height - 1 as ::core::ffi::c_int) as Integer,
            },
        };
        let c2rust_fresh37 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh37 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh38 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh38 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ((*ui).width - 1 as ::core::ffi::c_int) as Integer,
            },
        };
        push_call(
            ui,
            b"set_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_default_colors_set(
    mut ui: *mut RemoteUI,
    mut rgb_fg: Integer,
    mut rgb_bg: Integer,
    mut rgb_sp: Integer,
    mut cterm_fg: Integer,
    mut cterm_bg: Integer,
) {
    if !(*ui).ui_ext[kUITermColors as ::core::ffi::c_int as usize] {
        let mut dark_: bool = *p_bg.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int;
        rgb_fg = if rgb_fg != -1 as Integer {
            rgb_fg
        } else {
            (if dark_ as ::core::ffi::c_int != 0 {
                0xffffff as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as Integer
        };
        rgb_bg = if rgb_bg != -1 as Integer {
            rgb_bg
        } else {
            (if dark_ as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                0xffffff as ::core::ffi::c_int
            }) as Integer
        };
        rgb_sp = if rgb_sp != -1 as Integer {
            rgb_sp
        } else {
            0xff0000 as Integer
        };
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 5] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 5];
    args.capacity = 5 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh39 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh39 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_fg },
    };
    let c2rust_fresh40 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh40 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_bg },
    };
    let c2rust_fresh41 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh41 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_sp },
    };
    let c2rust_fresh42 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh42 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: cterm_fg },
    };
    let c2rust_fresh43 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh43 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: cterm_bg },
    };
    push_call(
        ui,
        b"default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
    if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        args.size = 0 as size_t;
        let c2rust_fresh44 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh44 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_fg
                } else {
                    cterm_fg - 1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_fg\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        args.size = 0 as size_t;
        let c2rust_fresh45 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh45 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_bg
                } else {
                    cterm_bg - 1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_bg\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        args.size = 0 as size_t;
        let c2rust_fresh46 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh46 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_sp
                } else {
                    -1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_sp\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_hl_attr_define(
    mut ui: *mut RemoteUI,
    mut id: Integer,
    mut rgb_attrs: HlAttrs,
    mut _cterm_attrs: HlAttrs,
    mut info: Array,
) {
    if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 4];
    args.capacity = 4 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh47 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh47 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: id },
    };
    let mut rgb: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut rgb__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    rgb.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    rgb.items = &raw mut rgb__items as *mut KeyValuePair;
    let mut cterm: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut cterm__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    cterm.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    cterm.items = &raw mut cterm__items as *mut KeyValuePair;
    hlattrs2dict(
        &raw mut rgb,
        ::core::ptr::null_mut::<Dict>(),
        rgb_attrs,
        true_0 != 0,
        false_0 != 0,
    );
    hlattrs2dict(
        &raw mut cterm,
        ::core::ptr::null_mut::<Dict>(),
        rgb_attrs,
        false_0 != 0,
        false_0 != 0,
    );
    if rgb_attrs.url >= 0 as int32_t {
        let mut url: *const ::core::ffi::c_char = hl_get_url(rgb_attrs.url as uint32_t);
        let c2rust_fresh48 = rgb.size;
        rgb.size = rgb.size.wrapping_add(1);
        *rgb.items.offset(c2rust_fresh48 as isize) = key_value_pair {
            key: cstr_as_string(b"url\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(url),
                },
            },
        };
    }
    let c2rust_fresh49 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh49 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: rgb },
    };
    let c2rust_fresh50 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh50 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: cterm },
    };
    if (*ui).ui_ext[kUIHlState as ::core::ffi::c_int as usize] {
        let c2rust_fresh51 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh51 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: info },
        };
    } else {
        let c2rust_fresh52 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh52 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                },
            },
        };
    }
    push_call(
        ui,
        b"hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_highlight_set(
    mut ui: *mut RemoteUI,
    mut id: ::core::ffi::c_int,
) {
    if (*ui).hl_id == id {
        return;
    }
    (*ui).hl_id = id;
    let mut dict: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut dict__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    dict.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    dict.items = &raw mut dict__items as *mut KeyValuePair;
    hlattrs2dict(
        &raw mut dict,
        ::core::ptr::null_mut::<Dict>(),
        syn_attr2entry(id),
        (*ui).rgb,
        false_0 != 0,
    );
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
    let c2rust_fresh53 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh53 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: dict },
    };
    push_call(
        ui,
        b"highlight_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_grid_cursor_goto(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
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
        let c2rust_fresh54 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh54 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
        let c2rust_fresh55 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh55 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: row },
        };
        let c2rust_fresh56 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh56 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: col },
        };
        push_call(
            ui,
            b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    } else {
        (*ui).cursor_row = row;
        (*ui).cursor_col = col;
        remote_ui_cursor_goto(ui, row, col);
    };
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_cursor_goto(
    mut ui: *mut RemoteUI,
    mut row: Integer,
    mut col: Integer,
) {
    if (*ui).client_row == row && (*ui).client_col == col {
        return;
    }
    (*ui).client_row = row;
    (*ui).client_col = col;
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
    let c2rust_fresh57 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh57 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh58 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh58 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: col },
    };
    push_call(
        ui,
        b"cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_put(
    mut ui: *mut RemoteUI,
    mut cell: *const ::core::ffi::c_char,
) {
    (*ui).client_col += 1;
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
    let c2rust_fresh59 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh59 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(cell),
        },
    };
    push_call(ui, b"put\0".as_ptr() as *const ::core::ffi::c_char, args);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_raw_line(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        prepare_call(ui, b"grid_line\0".as_ptr() as *const ::core::ffi::c_char);
        let mut buf: *mut *mut ::core::ffi::c_char = &raw mut (*ui).packer.ptr;
        mpack_array(buf, 5 as uint32_t);
        mpack_uint(buf, grid as uint32_t);
        mpack_uint(buf, row as uint32_t);
        mpack_uint(buf, startcol as uint32_t);
        let mut lenpos: *mut ::core::ffi::c_char = mpack_array_dyn16(buf);
        let mut repeat: uint32_t = 0 as uint32_t;
        let mut ncells: size_t = (endcol - startcol) as size_t;
        let mut last_hl: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut nelem: uint32_t = 0 as uint32_t;
        let mut was_space: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < ncells {
            repeat = repeat.wrapping_add(1);
            if i == ncells.wrapping_sub(1 as size_t)
                || *attrs.offset(i as isize) != *attrs.offset(i.wrapping_add(1 as size_t) as isize)
                || *chunk.offset(i as isize) != *chunk.offset(i.wrapping_add(1 as size_t) as isize)
            {
                if (UI_BUF_SIZE as size_t)
                    .wrapping_sub((*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t)
                    < (2 as ::core::ffi::c_int
                        * (1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                            + MAX_SCHAR_SIZE
                            + 5 as ::core::ffi::c_int
                            + 5 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int) as size_t
                    || (*ui).ncells_pending >= 500 as size_t
                {
                    if was_space {
                        nelem = nelem.wrapping_add(1);
                        (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(1 as size_t);
                        mpack_array(buf, 3 as uint32_t);
                        mpack_str_small(
                            buf,
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        );
                        mpack_uint(buf, clearattr as uint32_t);
                        mpack_uint(buf, 0 as uint32_t);
                    }
                    mpack_w2(&raw mut lenpos, nelem);
                    mpack_bool(buf, false_0 != 0);
                    ui_flush_buf(ui, false_0 != 0);
                    prepare_call(ui, b"grid_line\0".as_ptr() as *const ::core::ffi::c_char);
                    mpack_array(buf, 5 as uint32_t);
                    mpack_uint(buf, grid as uint32_t);
                    mpack_uint(buf, row as uint32_t);
                    mpack_uint(
                        buf,
                        (startcol as uint32_t)
                            .wrapping_add(i as uint32_t)
                            .wrapping_sub(repeat)
                            .wrapping_add(1 as uint32_t),
                    );
                    lenpos = mpack_array_dyn16(buf);
                    nelem = 0 as uint32_t;
                    last_hl = -1 as ::core::ffi::c_int;
                }
                let mut csize: uint32_t = (if repeat > 1 as uint32_t {
                    3 as ::core::ffi::c_int
                } else if *attrs.offset(i as isize) != last_hl as sattr_T {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as uint32_t;
                nelem = nelem.wrapping_add(1);
                mpack_array(buf, csize);
                let c2rust_fresh60 = *buf;
                *buf = (*buf).offset(1);
                let mut size_byte: *mut ::core::ffi::c_char = c2rust_fresh60;
                let mut len: size_t = schar_get_adv(buf, *chunk.offset(i as isize));
                *size_byte = (0xa0 as size_t | len) as ::core::ffi::c_char;
                if csize >= 2 as uint32_t {
                    mpack_uint(buf, *attrs.offset(i as isize) as uint32_t);
                    if csize >= 3 as uint32_t {
                        mpack_uint(buf, repeat);
                    }
                }
                (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(
                    (if repeat < 2 as uint32_t {
                        repeat
                    } else {
                        2 as uint32_t
                    }) as size_t,
                );
                last_hl = *attrs.offset(i as isize) as ::core::ffi::c_int;
                repeat = 0 as uint32_t;
                was_space = *chunk.offset(i as isize) == ' ' as ::core::ffi::c_int as schar_T;
            }
            i = i.wrapping_add(1);
        }
        if endcol < clearcol || was_space as ::core::ffi::c_int != 0 {
            nelem = nelem.wrapping_add(1);
            (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(1 as size_t);
            mpack_array(buf, 3 as uint32_t);
            mpack_str_small(
                buf,
                b" \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
            mpack_uint(buf, clearattr as uint32_t);
            mpack_uint(buf, (clearcol - endcol) as uint32_t);
        }
        mpack_w2(&raw mut lenpos, nelem);
        mpack_bool(
            buf,
            flags as ::core::ffi::c_int & kLineFlagWrap as ::core::ffi::c_int != 0,
        );
    } else {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as Integer) < endcol - startcol {
            remote_ui_cursor_goto(ui, row, startcol + i_0 as Integer);
            remote_ui_highlight_set(ui, *attrs.offset(i_0 as isize) as ::core::ffi::c_int);
            let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
            schar_get(
                &raw mut sc_buf as *mut ::core::ffi::c_char,
                *chunk.offset(i_0 as isize),
            );
            remote_ui_put(ui, &raw mut sc_buf as *mut ::core::ffi::c_char);
            if utf_ambiguous_width(&raw mut sc_buf as *mut ::core::ffi::c_char) {
                (*ui).client_col = -1 as Integer;
            }
            i_0 += 1;
        }
        if endcol < clearcol {
            remote_ui_cursor_goto(ui, row, endcol);
            remote_ui_highlight_set(ui, clearattr as ::core::ffi::c_int);
            if clearattr == 0 as Integer && clearcol == Columns.get() as Integer {
                let mut args: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                push_call(
                    ui,
                    b"eol_clear\0".as_ptr() as *const ::core::ffi::c_char,
                    args,
                );
            } else {
                let mut c: Integer = endcol;
                while c < clearcol {
                    remote_ui_put(ui, b" \0".as_ptr() as *const ::core::ffi::c_char);
                    c += 1;
                }
            }
        }
    };
}
unsafe extern "C" fn ui_flush_buf(mut ui: *mut RemoteUI, mut incomplete_event: bool) {
    if (*ui).packer.startptr.is_null()
        || (*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t == 0
    {
        return;
    }
    (*ui).incomplete_event = incomplete_event;
    flush_event(ui);
    if !(*ui).nevents_pos.is_null() {
        mpack_w2(&raw mut (*ui).nevents_pos, (*ui).nevents);
        (*ui).nevents = 0 as uint32_t;
        (*ui).nevents_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut WBuffer = wstream_new_buffer(
        (*ui).packer.startptr,
        (*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t,
        1 as size_t,
        Some(free_block as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    rpc_write_raw((*ui).channel_id, buf);
    (*ui).packer.startptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).packer.ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).flushed_events = true_0 != 0;
    (*ui).ncells_pending = 0 as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_flush(mut ui: *mut RemoteUI) {
    if (*ui).nevents > 0 as uint32_t || (*ui).flushed_events as ::core::ffi::c_int != 0 {
        if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            remote_ui_cursor_goto(ui, (*ui).cursor_row, (*ui).cursor_col);
        }
        push_call(
            ui,
            b"flush\0".as_ptr() as *const ::core::ffi::c_char,
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        );
        ui_flush_buf(ui, false_0 != 0);
        (*ui).flushed_events = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_ui_send(mut ui: *mut RemoteUI, mut content: String_0) {
    if !(*ui).stdout_tty {
        return;
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
    let c2rust_fresh62 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh62 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: content },
    };
    push_call(
        ui,
        b"ui_send\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_flush_pending_data(mut ui: *mut RemoteUI) {
    ui_flush_buf(ui, false_0 != 0);
}
unsafe extern "C" fn translate_contents(
    mut ui: *mut RemoteUI,
    mut contents: Array,
    mut arena: *mut Arena,
) -> Array {
    let mut new_contents: Array = arena_array(arena, contents.size);
    let mut i: size_t = 0 as size_t;
    while i < contents.size {
        let mut item: Array = (*contents.items.offset(i as isize)).data.array;
        let mut new_item: Array = arena_array(arena, 2 as size_t);
        let mut attr: ::core::ffi::c_int = (*item.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as ::core::ffi::c_int;
        if attr != 0 {
            let mut rgb_attrs: Dict =
                arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
            hlattrs2dict(
                &raw mut rgb_attrs,
                ::core::ptr::null_mut::<Dict>(),
                syn_attr2entry(attr),
                (*ui).rgb,
                false_0 != 0,
            );
            let c2rust_fresh70 = new_item.size;
            new_item.size = new_item.size.wrapping_add(1);
            *new_item.items.offset(c2rust_fresh70 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: rgb_attrs },
            };
        } else {
            let c2rust_fresh71 = new_item.size;
            new_item.size = new_item.size.wrapping_add(1);
            *new_item.items.offset(c2rust_fresh71 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed {
                    dict: Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    },
                },
            };
        }
        let c2rust_fresh72 = new_item.size;
        new_item.size = new_item.size.wrapping_add(1);
        *new_item.items.offset(c2rust_fresh72 as isize) =
            *item.items.offset(1 as ::core::ffi::c_int as isize);
        let c2rust_fresh73 = new_contents.size;
        new_contents.size = new_contents.size.wrapping_add(1);
        *new_contents.items.offset(c2rust_fresh73 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: new_item },
        };
        i = i.wrapping_add(1);
    }
    return new_contents;
}
unsafe extern "C" fn translate_firstarg(
    mut ui: *mut RemoteUI,
    mut args: Array,
    mut arena: *mut Arena,
) -> Array {
    let mut new_args: Array = arena_array(arena, args.size);
    let mut contents: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .array;
    let c2rust_fresh68 = new_args.size;
    new_args.size = new_args.size.wrapping_add(1);
    *new_args.items.offset(c2rust_fresh68 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: translate_contents(ui, contents, arena),
        },
    };
    let mut i: size_t = 1 as size_t;
    while i < args.size {
        let c2rust_fresh69 = new_args.size;
        new_args.size = new_args.size.wrapping_add(1);
        *new_args.items.offset(c2rust_fresh69 as isize) = *args.items.offset(i as isize);
        i = i.wrapping_add(1);
    }
    return new_args;
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_event(
    mut ui: *mut RemoteUI,
    mut name: *mut ::core::ffi::c_char,
    mut args: Array,
) {
    let mut arena: Arena = ARENA_EMPTY;
    '_free_ret: {
        if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            if strequal(
                name,
                b"cmdline_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut new_args: Array = translate_firstarg(ui, args, &raw mut arena);
                push_call(ui, name, new_args);
                break '_free_ret;
            } else if strequal(
                name,
                b"cmdline_block_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut block: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .array;
                let mut new_block: Array = arena_array(&raw mut arena, block.size);
                let mut i: size_t = 0 as size_t;
                while i < block.size {
                    let c2rust_fresh63 = new_block.size;
                    new_block.size = new_block.size.wrapping_add(1);
                    *new_block.items.offset(c2rust_fresh63 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed {
                            array: translate_contents(
                                ui,
                                (*block.items.offset(i as isize)).data.array,
                                &raw mut arena,
                            ),
                        },
                    };
                    i = i.wrapping_add(1);
                }
                let mut new_args_0: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                let mut new_args__items: [Object; 1] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }; 1];
                new_args_0.capacity = 1 as size_t;
                new_args_0.items = &raw mut new_args__items as *mut Object;
                let c2rust_fresh64 = new_args_0.size;
                new_args_0.size = new_args_0.size.wrapping_add(1);
                *new_args_0.items.offset(c2rust_fresh64 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: new_block },
                };
                push_call(ui, name, new_args_0);
                break '_free_ret;
            } else if strequal(
                name,
                b"cmdline_block_append\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut new_args_1: Array = translate_firstarg(ui, args, &raw mut arena);
                push_call(ui, name, new_args_1);
                break '_free_ret;
            }
        }
        if (*ui).ui_ext[kUIWildmenu as ::core::ffi::c_int as usize] {
            if strequal(
                name,
                b"popupmenu_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                (*ui).wildmenu_active = (*args.items.offset(4 as ::core::ffi::c_int as isize))
                    .data
                    .integer
                    == -1 as Integer
                    || !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize];
                if (*ui).wildmenu_active {
                    let mut items: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .array;
                    let mut new_items: Array = arena_array(&raw mut arena, items.size);
                    let mut i_0: size_t = 0 as size_t;
                    while i_0 < items.size {
                        let c2rust_fresh65 = new_items.size;
                        new_items.size = new_items.size.wrapping_add(1);
                        *new_items.items.offset(c2rust_fresh65 as isize) =
                            *(*items.items.offset(i_0 as isize))
                                .data
                                .array
                                .items
                                .offset(0 as ::core::ffi::c_int as isize);
                        i_0 = i_0.wrapping_add(1);
                    }
                    let mut new_args_2: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut new_args__items_0: [Object; 1] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    }; 1];
                    new_args_2.capacity = 1 as size_t;
                    new_args_2.items = &raw mut new_args__items_0 as *mut Object;
                    let c2rust_fresh66 = new_args_2.size;
                    new_args_2.size = new_args_2.size.wrapping_add(1);
                    *new_args_2.items.offset(c2rust_fresh66 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: new_items },
                    };
                    push_call(
                        ui,
                        b"wildmenu_show\0".as_ptr() as *const ::core::ffi::c_char,
                        new_args_2,
                    );
                    if (*args.items.offset(1 as ::core::ffi::c_int as isize))
                        .data
                        .integer
                        != -1 as Integer
                    {
                        new_args_2.size = 0 as size_t;
                        let c2rust_fresh67 = new_args_2.size;
                        new_args_2.size = new_args_2.size.wrapping_add(1);
                        *new_args_2.items.offset(c2rust_fresh67 as isize) =
                            *args.items.offset(1 as ::core::ffi::c_int as isize);
                        push_call(
                            ui,
                            b"wildmenu_select\0".as_ptr() as *const ::core::ffi::c_char,
                            new_args_2,
                        );
                    }
                    break '_free_ret;
                }
            } else if strequal(
                name,
                b"popupmenu_select\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                if (*ui).wildmenu_active {
                    name = b"wildmenu_select\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
            } else if strequal(
                name,
                b"popupmenu_hide\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                if (*ui).wildmenu_active {
                    name = b"wildmenu_hide\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
            }
        }
        push_call(ui, name, args);
        return;
    }
    arena_mem_free(arena_finish(&raw mut arena));
}
#[no_mangle]
pub unsafe extern "C" fn nvim_ui_send(
    mut _channel_id: uint64_t,
    mut content: String_0,
    mut _err: *mut Error,
) {
    ui_call_ui_send(content);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_mode_info_set(
    mut ui: *mut RemoteUI,
    mut enabled: Boolean,
    mut cursor_styles: Array,
) {
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
    let c2rust_fresh74 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh74 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed { boolean: enabled },
    };
    let c2rust_fresh75 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh75 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: cursor_styles,
        },
    };
    push_call(
        ui,
        b"mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_update_menu(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"update_menu\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_busy_start(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"busy_start\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_busy_stop(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"busy_stop\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_mouse_on(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"mouse_on\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_mouse_off(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"mouse_off\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_mode_change(
    mut ui: *mut RemoteUI,
    mut mode: String_0,
    mut mode_idx: Integer,
) {
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
    let c2rust_fresh76 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh76 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: mode },
    };
    let c2rust_fresh77 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh77 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: mode_idx },
    };
    push_call(
        ui,
        b"mode_change\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_bell(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"bell\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_visual_bell(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"visual_bell\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_suspend(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_set_title(mut ui: *mut RemoteUI, mut title: String_0) {
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
    let c2rust_fresh78 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh78 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: title },
    };
    push_call(
        ui,
        b"set_title\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_set_icon(mut ui: *mut RemoteUI, mut icon: String_0) {
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
    let c2rust_fresh79 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh79 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: icon },
    };
    push_call(
        ui,
        b"set_icon\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_screenshot(mut ui: *mut RemoteUI, mut path: String_0) {
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
    let c2rust_fresh80 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh80 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: path },
    };
    push_call(
        ui,
        b"screenshot\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_option_set(
    mut ui: *mut RemoteUI,
    mut name: String_0,
    mut value: Object,
) {
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
    let c2rust_fresh81 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh81 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: name },
    };
    let c2rust_fresh82 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh82 as isize) = value;
    push_call(
        ui,
        b"option_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_chdir(mut ui: *mut RemoteUI, mut path: String_0) {
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
    let c2rust_fresh83 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh83 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: path },
    };
    push_call(ui, b"chdir\0".as_ptr() as *const ::core::ffi::c_char, args);
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_hl_group_set(
    mut ui: *mut RemoteUI,
    mut name: String_0,
    mut id: Integer,
) {
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
    let c2rust_fresh84 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh84 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: name },
    };
    let c2rust_fresh85 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh85 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: id },
    };
    push_call(
        ui,
        b"hl_group_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_msg_set_pos(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut scrolled: Boolean,
    mut sep_char: String_0,
    mut zindex: Integer,
    mut compindex: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh86 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh86 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh87 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh87 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh88 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh88 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed { boolean: scrolled },
    };
    let c2rust_fresh89 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh89 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: sep_char },
    };
    let c2rust_fresh90 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh90 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: zindex },
    };
    let c2rust_fresh91 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh91 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: compindex },
    };
    push_call(
        ui,
        b"msg_set_pos\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_win_viewport(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut win: Window,
    mut topline: Integer,
    mut botline: Integer,
    mut curline: Integer,
    mut curcol: Integer,
    mut line_count: Integer,
    mut scroll_delta: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 8] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 8];
    args.capacity = 8 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh92 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh92 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh93 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh93 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed {
            integer: win as Integer,
        },
    };
    let c2rust_fresh94 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh94 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: topline },
    };
    let c2rust_fresh95 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh95 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: botline },
    };
    let c2rust_fresh96 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh96 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: curline },
    };
    let c2rust_fresh97 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh97 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: curcol },
    };
    let c2rust_fresh98 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh98 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: line_count,
        },
    };
    let c2rust_fresh99 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh99 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: scroll_delta,
        },
    };
    push_call(
        ui,
        b"win_viewport\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_win_viewport_margins(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut win: Window,
    mut top: Integer,
    mut bottom: Integer,
    mut left: Integer,
    mut right: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh100 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh100 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh101 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh101 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed {
            integer: win as Integer,
        },
    };
    let c2rust_fresh102 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh102 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: top },
    };
    let c2rust_fresh103 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh103 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: bottom },
    };
    let c2rust_fresh104 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh104 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: left },
    };
    let c2rust_fresh105 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh105 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: right },
    };
    push_call(
        ui,
        b"win_viewport_margins\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
#[no_mangle]
pub unsafe extern "C" fn remote_ui_error_exit(mut ui: *mut RemoteUI, mut status: Integer) {
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
    let c2rust_fresh106 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh106 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: status },
    };
    push_call(
        ui,
        b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub const CHAN_STDIO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh1 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh1 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh2 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh2 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh10 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh10 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh11 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh11 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh12 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh12 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh13 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh13 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_uint(mut buf: *mut *mut ::core::ffi::c_char, mut val: uint32_t) {
    if val > 0xffff as uint32_t {
        let c2rust_fresh5 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh5 = 0xce as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, val);
    } else if val > 0xff as uint32_t {
        let c2rust_fresh6 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh6 = 0xcd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, val);
    } else if val > 0x7f as uint32_t {
        let c2rust_fresh7 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh7 = 0xcc as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh8 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh8 = val as ::core::ffi::c_char;
    } else {
        let c2rust_fresh9 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh9 = val as ::core::ffi::c_char;
    };
}
#[inline]
unsafe extern "C" fn mpack_bool(mut buf: *mut *mut ::core::ffi::c_char, mut val: bool) {
    let c2rust_fresh61 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh61 = (0xc2 as ::core::ffi::c_int
        | (if val as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh14 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh14 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh15 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh15 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh16 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh16 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
