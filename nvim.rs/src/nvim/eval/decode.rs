extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn abort() -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn mpack_unpack_boolean(t: mpack_token_t) -> bool;
    fn mpack_unpack_uint(t: mpack_token_t) -> mpack_uintmax_t;
    fn mpack_unpack_sint(t: mpack_token_t) -> mpack_sintmax_t;
    fn mpack_unpack_float_fast(t: mpack_token_t) -> ::core::ffi::c_double;
    fn mpack_parser_init(p: *mut mpack_parser_t, c: mpack_uint32_t);
    fn mpack_parse(
        parser: *mut mpack_parser_t,
        b: *mut *const ::core::ffi::c_char,
        bl: *mut size_t,
        enter_cb: mpack_walk_cb,
        exit_cb: mpack_walk_cb,
    ) -> ::core::ffi::c_int;
    fn vim_str2nr(
        start: *const ::core::ffi::c_char,
        prep: *mut ::core::ffi::c_int,
        len: *mut ::core::ffi::c_int,
        what: ::core::ffi::c_int,
        nptr: *mut varnumber_T,
        unptr: *mut uvarnumber_T,
        maxlen: ::core::ffi::c_int,
        strict: bool,
        overflow: *mut bool,
    );
    fn string2float(text: *const ::core::ffi::c_char, ret_value: *mut float_T) -> size_t;
    fn encode_list_write(
        data: *mut ::core::ffi::c_void,
        buf: *const ::core::ffi::c_char,
        len: size_t,
    );
    static mut hash_removed: ::core::ffi::c_char;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_owned_tv(l: *mut list_T, tv: typval_T) -> *mut typval_T;
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_dict_item_alloc_len(key: *const ::core::ffi::c_char, key_len: size_t) -> *mut dictitem_T;
    fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T;
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int;
    fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T;
    fn tv_clear(tv: *mut typval_T);
    static mut eval_msgpack_type_lists: [*const list_T; 8];
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_ptr2len(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2len(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type mpack_uint32_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_value_s {
    pub lo: mpack_uint32_t,
    pub hi: mpack_uint32_t,
}
pub type mpack_value_t = mpack_value_s;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed = 2;
pub const MPACK_EOF: C2Rust_Unnamed = 1;
pub const MPACK_OK: C2Rust_Unnamed = 0;
pub type mpack_token_type_t = ::core::ffi::c_uint;
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_token_s {
    pub type_0: mpack_token_type_t,
    pub length: mpack_uint32_t,
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub value: mpack_value_t,
    pub chunk_ptr: *const ::core::ffi::c_char,
    pub ext_type: ::core::ffi::c_int,
}
pub type mpack_token_t = mpack_token_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_tokbuf_s {
    pub pending: [::core::ffi::c_char; 9],
    pub pending_tok: mpack_token_t,
    pub ppos: size_t,
    pub plen: size_t,
    pub passthrough: mpack_uint32_t,
}
pub type mpack_tokbuf_t = mpack_tokbuf_s;
pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
#[derive(Copy, Clone)]
#[repr(C)]
pub union mpack_data_t {
    pub p: *mut ::core::ffi::c_void,
    pub u: mpack_uintmax_t,
    pub i: mpack_sintmax_t,
    pub d: ::core::ffi::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_node_s {
    pub tok: mpack_token_t,
    pub pos: size_t,
    pub key_visited: ::core::ffi::c_int,
    pub data: [mpack_data_t; 2],
}
pub type mpack_node_t = mpack_node_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 33],
}
pub type mpack_walk_cb = Option<unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()>;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
pub type proftime_T = uint64_t;
pub type linenr_T = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
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
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
pub type varnumber_T = int64_t;
pub type uvarnumber_T = uint64_t;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
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
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub type list_T = listvar_S;
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
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
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
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
pub type SpecialVarValue = ::core::ffi::c_uint;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type BoolVarValue = ::core::ffi::c_uint;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
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
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_2 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_2 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_2 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_2 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_2 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_2 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_2 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_2 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_2 = 0;
pub type MessagePackType = ::core::ffi::c_uint;
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPArray: MessagePackType = 5;
pub const kMPString: MessagePackType = 4;
pub const kMPFloat: MessagePackType = 3;
pub const kMPInteger: MessagePackType = 2;
pub const kMPBoolean: MessagePackType = 1;
pub const kMPNil: MessagePackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerStackItem {
    pub stack_index: size_t,
    pub special_val: *mut list_T,
    pub s: *const ::core::ffi::c_char,
    pub container: typval_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContainerStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ContainerStackItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ValuesStackItem {
    pub is_special_string: bool,
    pub didcomma: bool,
    pub didcolon: bool,
    pub val: typval_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ValuesStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ValuesStackItem,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = '\u{8}' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
pub const FF: ::core::ffi::c_int = '\u{c}' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = 13;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isxdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int
        || c >= 'a' as ::core::ffi::c_int && c <= 'f' as ::core::ffi::c_int
        || c >= 'A' as ::core::ffi::c_int && c <= 'F' as ::core::ffi::c_int;
}
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn create_special_dict(
    rettv: *mut typval_T,
    type_0: MessagePackType,
    mut val: typval_T,
) {
    let dict: *mut dict_T = tv_dict_alloc();
    let type_di: *mut dictitem_T = tv_dict_item_alloc_len(
        b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    (*type_di).di_tv.v_type = VAR_LIST;
    (*type_di).di_tv.v_lock = VAR_UNLOCKED;
    (*type_di).di_tv.vval.v_list = eval_msgpack_type_lists[type_0 as usize] as *mut list_T;
    tv_list_ref((*type_di).di_tv.vval.v_list);
    tv_dict_add(dict, type_di);
    let val_di: *mut dictitem_T = tv_dict_item_alloc_len(
        b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
    );
    (*val_di).di_tv = val;
    tv_dict_add(dict, val_di);
    (*dict).dv_refcount += 1;
    *rettv = typval_T {
        v_type: VAR_DICT,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_dict: dict },
    };
}
#[inline]
unsafe extern "C" fn json_decoder_pop(
    mut obj: ValuesStackItem,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    pp: *mut *const ::core::ffi::c_char,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    if (*container_stack).size == 0 as size_t {
        if (*stack).size == (*stack).capacity {
            (*stack).capacity = (if (*stack).capacity != 0 {
                (*stack).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            });
            (*stack).items = xrealloc(
                (*stack).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ValuesStackItem>().wrapping_mul((*stack).capacity),
            ) as *mut ValuesStackItem;
        } else {
        };
        let c2rust_fresh4 = (*stack).size;
        (*stack).size = (*stack).size.wrapping_add(1);
        *(*stack).items.offset(c2rust_fresh4 as isize) = obj;
        return OK;
    }
    let mut last_container: ContainerStackItem = *(*container_stack).items.offset(
        (*container_stack)
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    let mut val_location: *const ::core::ffi::c_char = *pp;
    if obj.val.v_type as ::core::ffi::c_uint
        == last_container.container.v_type as ::core::ffi::c_uint
        && obj.val.vval.v_list as *mut ::core::ffi::c_void
            == last_container.container.vval.v_list as *mut ::core::ffi::c_void
    {
        (*container_stack).size = (*container_stack).size.wrapping_sub(1);
        val_location = last_container.s;
        last_container = *(*container_stack).items.offset(
            (*container_stack)
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
    }
    if last_container.container.v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_list_len(last_container.container.vval.v_list) != 0 as ::core::ffi::c_int
            && !obj.didcomma
        {
            semsg(
                gettext(b"E474: Expected comma before list item: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                val_location,
            );
            tv_clear(&raw mut obj.val);
            return FAIL;
        }
        '_c2rust_label: {
            if last_container.special_val.is_null() {
            } else {
                __assert_fail(
                    b"last_container.special_val == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    133 as ::core::ffi::c_uint,
                    b"int json_decoder_pop(ValuesStackItem, ValuesStack *const, ContainerStack *const, const char **const, _Bool *const, _Bool *const, _Bool *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        tv_list_append_owned_tv(last_container.container.vval.v_list, obj.val);
    } else if last_container.stack_index == (*stack).size.wrapping_sub(2 as size_t) {
        if !obj.didcolon {
            semsg(
                gettext(
                    b"E474: Expected colon before dictionary value: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                val_location,
            );
            tv_clear(&raw mut obj.val);
            return FAIL;
        }
        (*stack).size = (*stack).size.wrapping_sub(1);
        let mut key: ValuesStackItem = *(*stack).items.offset((*stack).size as isize);
        if last_container.special_val.is_null() {
            '_c2rust_label_0: {
                if !(key.is_special_string as ::core::ffi::c_int != 0
                    || key.val.vval.v_string.is_null())
                {
                } else {
                    __assert_fail(
                        b"!(key.is_special_string || key.val.vval.v_string == NULL)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        145 as ::core::ffi::c_uint,
                        b"int json_decoder_pop(ValuesStackItem, ValuesStack *const, ContainerStack *const, const char **const, _Bool *const, _Bool *const, _Bool *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let obj_di: *mut dictitem_T = tv_dict_item_alloc(key.val.vval.v_string);
            tv_clear(&raw mut key.val);
            if tv_dict_add(last_container.container.vval.v_dict, obj_di) == FAIL {
                abort();
            }
            (*obj_di).di_tv = obj.val;
        } else {
            let kv_pair: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
            tv_list_append_list(last_container.special_val, kv_pair);
            tv_list_append_owned_tv(kv_pair, key.val);
            tv_list_append_owned_tv(kv_pair, obj.val);
        }
    } else {
        if !obj.is_special_string
            && obj.val.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(b"E474: Expected string key: %s\0".as_ptr() as *const ::core::ffi::c_char),
                *pp,
            );
            tv_clear(&raw mut obj.val);
            return FAIL;
        } else if !obj.didcomma
            && (last_container.special_val.is_null()
                && (*last_container.container.vval.v_dict).dv_hashtab.ht_used != 0 as size_t)
        {
            semsg(
                gettext(b"E474: Expected comma before dictionary key: %s\0".as_ptr()
                    as *const ::core::ffi::c_char),
                val_location,
            );
            tv_clear(&raw mut obj.val);
            return FAIL;
        }
        if last_container.special_val.is_null()
            && (obj.is_special_string as ::core::ffi::c_int != 0
                || obj.val.vval.v_string.is_null()
                || !tv_dict_find(
                    last_container.container.vval.v_dict,
                    obj.val.vval.v_string,
                    -1 as ptrdiff_t,
                )
                .is_null())
        {
            tv_clear(&raw mut obj.val);
            (*container_stack).size = (*container_stack).size.wrapping_sub(1);
            let mut last_container_val: ValuesStackItem =
                *(*stack).items.offset(last_container.stack_index as isize);
            while (*stack).size > last_container.stack_index {
                (*stack).size = (*stack).size.wrapping_sub(1);
                tv_clear(&raw mut (*(*stack).items.offset((*stack).size as isize)).val);
            }
            *pp = last_container.s;
            *didcomma = last_container_val.didcomma;
            *didcolon = last_container_val.didcolon;
            *next_map_special = true_0 != 0;
            return OK;
        }
        if (*stack).size == (*stack).capacity {
            (*stack).capacity = (if (*stack).capacity != 0 {
                (*stack).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            });
            (*stack).items = xrealloc(
                (*stack).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ValuesStackItem>().wrapping_mul((*stack).capacity),
            ) as *mut ValuesStackItem;
        } else {
        };
        let c2rust_fresh5 = (*stack).size;
        (*stack).size = (*stack).size.wrapping_add(1);
        *(*stack).items.offset(c2rust_fresh5 as isize) = obj;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn decode_create_map_special_dict(
    ret_tv: *mut typval_T,
    len: ptrdiff_t,
) -> *mut list_T {
    let list: *mut list_T = tv_list_alloc(len);
    tv_list_ref(list);
    create_special_dict(
        ret_tv,
        kMPMap,
        typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: list },
        },
    );
    return list;
}
#[no_mangle]
pub unsafe extern "C" fn decode_string(
    s: *const ::core::ffi::c_char,
    len: size_t,
    mut force_blob: bool,
    s_allocated: bool,
) -> typval_T {
    '_c2rust_label: {
        if !s.is_null() || len == 0 as size_t {
        } else {
            __assert_fail(
                b"s != NULL || len == 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                261 as ::core::ffi::c_uint,
                b"typval_T decode_string(const char *const, const size_t, _Bool, const _Bool)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let use_blob: bool = force_blob as ::core::ffi::c_int != 0
        || !s.is_null() && !memchr(s as *const ::core::ffi::c_void, NUL, len).is_null();
    if use_blob {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_lock = VAR_UNLOCKED;
        let mut b: *mut blob_T = tv_blob_alloc_ret(&raw mut tv);
        if s_allocated {
            (*b).bv_ga.ga_data = s as *mut ::core::ffi::c_void;
            (*b).bv_ga.ga_len = len as ::core::ffi::c_int;
            (*b).bv_ga.ga_maxlen = len as ::core::ffi::c_int;
        } else {
            ga_concat_len(&raw mut (*b).bv_ga, s, len);
        }
        return tv;
    }
    return typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union {
            v_string: (if s.is_null() || s_allocated as ::core::ffi::c_int != 0 {
                s as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void
            } else {
                xmemdupz(s as *const ::core::ffi::c_void, len)
            }) as *mut ::core::ffi::c_char,
        },
    };
}
#[inline(always)]
unsafe extern "C" fn parse_json_string(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    pp: *mut *const ::core::ffi::c_char,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fst_in_pair: ::core::ffi::c_int = 0;
    let mut str_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut obj: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
    let mut p: *const ::core::ffi::c_char = *pp;
    let mut len: size_t = 0 as size_t;
    p = p.offset(1);
    let s: *const ::core::ffi::c_char = p;
    let mut ret: ::core::ffi::c_int = OK;
    '_parse_json_string_ret: {
        '_parse_json_string_fail: {
            while p < e && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    p = p.offset(1);
                    if p == e {
                        semsg(
                            gettext(b"E474: Unfinished escape sequence: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            buf_len as ::core::ffi::c_int,
                            buf,
                        );
                        break '_parse_json_string_fail;
                    } else {
                        match *p as ::core::ffi::c_int {
                            117 => {
                                if p.offset(4 as ::core::ffi::c_int as isize) >= e {
                                    semsg(
                                        gettext(
                                            b"E474: Unfinished unicode escape sequence: %.*s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        buf_len as ::core::ffi::c_int,
                                        buf,
                                    );
                                    break '_parse_json_string_fail;
                                } else if !ascii_isxdigit(
                                    *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                ) || !ascii_isxdigit(
                                    *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                ) || !ascii_isxdigit(
                                    *p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                ) || !ascii_isxdigit(
                                    *p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int,
                                ) {
                                    semsg(
                                        gettext(
                                            b"E474: Expected four hex digits after \\u: %.*s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        e.offset_from(p.offset(-(1 as ::core::ffi::c_int as isize)))
                                            as ::core::ffi::c_int,
                                        p.offset(-(1 as ::core::ffi::c_int as isize)),
                                    );
                                    break '_parse_json_string_fail;
                                } else {
                                    len = len.wrapping_add(3 as size_t);
                                    p = p.offset(5 as ::core::ffi::c_int as isize);
                                }
                            }
                            92 | 47 | 34 | 116 | 98 | 110 | 114 | 102 => {
                                len = len.wrapping_add(1);
                                p = p.offset(1);
                            }
                            _ => {
                                semsg(
                                    gettext(b"E474: Unknown escape sequence: %.*s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    e.offset_from(p.offset(-(1 as ::core::ffi::c_int as isize)))
                                        as ::core::ffi::c_int,
                                    p.offset(-(1 as ::core::ffi::c_int as isize)),
                                );
                                break '_parse_json_string_fail;
                            }
                        }
                    }
                } else {
                    let mut p_byte: uint8_t = *p as uint8_t;
                    if (p_byte as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                b"E474: ASCII control characters cannot be present inside string: %.*s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            e.offset_from(p) as ::core::ffi::c_int,
                            p,
                        );
                        break '_parse_json_string_fail;
                    } else {
                        let ch: ::core::ffi::c_int = utf_ptr2char(p);
                        if ch >= 0x80 as ::core::ffi::c_int
                            && p_byte as ::core::ffi::c_int == ch
                            && !(ch == 0xc3 as ::core::ffi::c_int
                                && p.offset(1 as ::core::ffi::c_int as isize) < e
                                && *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    == 0x83 as ::core::ffi::c_int)
                        {
                            semsg(
                                gettext(b"E474: Only UTF-8 strings allowed: %.*s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                e.offset_from(p) as ::core::ffi::c_int,
                                p,
                            );
                            break '_parse_json_string_fail;
                        } else if ch > 0x10ffff as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    b"E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: %.*s\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                ),
                                e.offset_from(p) as ::core::ffi::c_int,
                                p,
                            );
                            break '_parse_json_string_fail;
                        } else {
                            let ch_len: size_t = utf_char2len(ch) as size_t;
                            '_c2rust_label: {
                                if ch_len
                                    == (if ch != 0 {
                                        utf_ptr2len(p)
                                    } else {
                                        1 as ::core::ffi::c_int
                                    }) as size_t
                                {
                                } else {
                                    __assert_fail(
                                        b"ch_len == (size_t)(ch ? utf_ptr2len(p) : 1)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        380 as ::core::ffi::c_uint,
                                        b"int parse_json_string(const char *const, const size_t, const char **const, ValuesStack *const, ContainerStack *const, _Bool *const, _Bool *const, _Bool *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            len = len.wrapping_add(ch_len);
                            p = p.offset(ch_len as isize);
                        }
                    }
                }
            }
            if p == e || *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E474: Expected string end: %.*s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    buf_len as ::core::ffi::c_int,
                    buf,
                );
            } else {
                str = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
                fst_in_pair = 0 as ::core::ffi::c_int;
                str_end = str;
                let mut t: *const ::core::ffi::c_char = s;
                while t < p {
                    if *t.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\\' as ::core::ffi::c_int
                        || *t.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != 'u' as ::core::ffi::c_int
                    {
                        if fst_in_pair != 0 as ::core::ffi::c_int {
                            str_end = str_end.offset(utf_char2bytes(fst_in_pair, str_end) as isize);
                            fst_in_pair = 0 as ::core::ffi::c_int;
                        }
                    }
                    if *t as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                        t = t.offset(1);
                        match *t as ::core::ffi::c_int {
                            117 => {
                                let ubuf: [::core::ffi::c_char; 4] = [
                                    *t.offset(1 as ::core::ffi::c_int as isize),
                                    *t.offset(2 as ::core::ffi::c_int as isize),
                                    *t.offset(3 as ::core::ffi::c_int as isize),
                                    *t.offset(4 as ::core::ffi::c_int as isize),
                                ];
                                t = t.offset(4 as ::core::ffi::c_int as isize);
                                let mut ch_0: uvarnumber_T = 0;
                                vim_str2nr(
                                    &raw const ubuf as *const ::core::ffi::c_char,
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    STR2NR_HEX as ::core::ffi::c_int
                                        | STR2NR_FORCE as ::core::ffi::c_int,
                                    ::core::ptr::null_mut::<varnumber_T>(),
                                    &raw mut ch_0,
                                    4 as ::core::ffi::c_int,
                                    true_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                );
                                if SURROGATE_HI_START as uvarnumber_T <= ch_0
                                    && ch_0 <= SURROGATE_HI_END as uvarnumber_T
                                {
                                    if fst_in_pair != 0 as ::core::ffi::c_int {
                                        str_end =
                                            str_end.offset(
                                                utf_char2bytes(fst_in_pair, str_end) as isize
                                            );
                                        fst_in_pair = 0 as ::core::ffi::c_int;
                                    }
                                    fst_in_pair = ch_0 as ::core::ffi::c_int;
                                } else if SURROGATE_LO_START as uvarnumber_T <= ch_0
                                    && ch_0 <= SURROGATE_LO_END as uvarnumber_T
                                    && fst_in_pair != 0 as ::core::ffi::c_int
                                {
                                    let full_char: ::core::ffi::c_int = ch_0
                                        .wrapping_sub(SURROGATE_LO_START as uvarnumber_T)
                                        as ::core::ffi::c_int
                                        + (fst_in_pair - SURROGATE_HI_START
                                            << 10 as ::core::ffi::c_int)
                                        + SURROGATE_FIRST_CHAR;
                                    str_end =
                                        str_end.offset(utf_char2bytes(full_char, str_end) as isize);
                                    fst_in_pair = 0 as ::core::ffi::c_int;
                                } else {
                                    if fst_in_pair != 0 as ::core::ffi::c_int {
                                        str_end =
                                            str_end.offset(
                                                utf_char2bytes(fst_in_pair, str_end) as isize
                                            );
                                        fst_in_pair = 0 as ::core::ffi::c_int;
                                    }
                                    str_end = str_end.offset(utf_char2bytes(
                                        ch_0 as ::core::ffi::c_int,
                                        str_end,
                                    )
                                        as isize);
                                }
                            }
                            92 | 47 | 34 | 116 | 98 | 110 | 114 | 102 => {
                                static mut escapes: [::core::ffi::c_char; 117] = [
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    '"' as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    '/' as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    '\\' as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    BS as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    FF as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    0,
                                    NL as ::core::ffi::c_char,
                                    0,
                                    0,
                                    0,
                                    CAR as ::core::ffi::c_char,
                                    0,
                                    TAB as ::core::ffi::c_char,
                                ];
                                let c2rust_fresh6 = str_end;
                                str_end = str_end.offset(1);
                                *c2rust_fresh6 = escapes[*t as ::core::ffi::c_int as usize];
                            }
                            _ => {
                                abort();
                            }
                        }
                    } else {
                        let c2rust_fresh7 = str_end;
                        str_end = str_end.offset(1);
                        *c2rust_fresh7 = *t;
                    }
                    t = t.offset(1);
                }
                if fst_in_pair != 0 as ::core::ffi::c_int {
                    str_end = str_end.offset(utf_char2bytes(fst_in_pair, str_end) as isize);
                    fst_in_pair = 0 as ::core::ffi::c_int;
                }
                *str_end = NUL as ::core::ffi::c_char;
                obj = decode_string(
                    str,
                    str_end.offset_from(str) as size_t,
                    false_0 != 0,
                    true_0 != 0,
                );
                if json_decoder_pop(
                    ValuesStackItem {
                        is_special_string: obj.v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint,
                        didcomma: *didcomma,
                        didcolon: *didcolon,
                        val: obj,
                    },
                    stack,
                    container_stack,
                    &raw mut p,
                    next_map_special,
                    didcomma,
                    didcolon,
                ) != FAIL
                {
                    if *next_map_special {
                        break '_parse_json_string_ret;
                    } else {
                        break '_parse_json_string_ret;
                    }
                }
            }
        }
        ret = FAIL;
    }
    *pp = p;
    return ret;
}
#[inline(always)]
unsafe extern "C" fn parse_json_number(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    pp: *mut *const ::core::ffi::c_char,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut exp_num_len: size_t = 0;
    let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
    let mut p: *const ::core::ffi::c_char = *pp;
    let mut ret: ::core::ffi::c_int = OK;
    let s: *const ::core::ffi::c_char = p;
    let mut ints: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fracs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut exps: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut exps_s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    ints = p;
    '_parse_json_number_ret: {
        '_parse_json_number_fail: {
            '_parse_json_number_check: {
                if p < e {
                    while p < e
                        && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        p = p.offset(1);
                    }
                    if p != ints.offset(1 as ::core::ffi::c_int as isize)
                        && *ints as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                    {
                        semsg(
                            gettext(b"E474: Leading zeroes are not allowed: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            e.offset_from(s) as ::core::ffi::c_int,
                            s,
                        );
                        break '_parse_json_number_fail;
                    } else if !(p >= e || p == ints) {
                        if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                            p = p.offset(1);
                            fracs = p;
                            while p < e
                                && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(1);
                            }
                            if p >= e || p == fracs {
                                break '_parse_json_number_check;
                            }
                        }
                        if *p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                            exps_s = p;
                            if p < e
                                && (*p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                                    || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int)
                            {
                                p = p.offset(1);
                            }
                            exps = p;
                            while p < e
                                && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(1);
                            }
                        }
                    }
                }
            }
            if p == ints {
                semsg(
                    gettext(b"E474: Missing number after minus sign: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    e.offset_from(s) as ::core::ffi::c_int,
                    s,
                );
            } else if p == fracs
                || !fracs.is_null() && exps_s == fracs.offset(1 as ::core::ffi::c_int as isize)
            {
                semsg(
                    gettext(b"E474: Missing number after decimal dot: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    e.offset_from(s) as ::core::ffi::c_int,
                    s,
                );
            } else if p == exps {
                semsg(
                    gettext(
                        b"E474: Missing exponent: %.*s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    e.offset_from(s) as ::core::ffi::c_int,
                    s,
                );
            } else {
                tv = typval_T {
                    v_type: VAR_NUMBER,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                exp_num_len = p.offset_from(s) as size_t;
                if !fracs.is_null() || !exps.is_null() {
                    let num_len: size_t = string2float(s, &raw mut tv.vval.v_float);
                    if exp_num_len != num_len {
                        semsg(
                            gettext(
                                b"E685: internal error: while converting number \"%.*s\" to float string2float consumed %zu bytes in place of %zu\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            exp_num_len as ::core::ffi::c_int,
                            s,
                            num_len,
                            exp_num_len,
                        );
                    }
                    tv.v_type = VAR_FLOAT;
                } else {
                    let mut nr: varnumber_T = 0;
                    let mut num_len_0: ::core::ffi::c_int = 0;
                    vim_str2nr(
                        s,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        &raw mut num_len_0,
                        0 as ::core::ffi::c_int,
                        &raw mut nr,
                        ::core::ptr::null_mut::<uvarnumber_T>(),
                        p.offset_from(s) as ::core::ffi::c_int,
                        true_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if exp_num_len as ::core::ffi::c_int != num_len_0 {
                        semsg(
                            gettext(
                                b"E685: internal error: while converting number \"%.*s\" to integer vim_str2nr consumed %i bytes in place of %zu\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            exp_num_len as ::core::ffi::c_int,
                            s,
                            num_len_0,
                            exp_num_len,
                        );
                    }
                    tv.vval.v_number = nr;
                }
                if json_decoder_pop(
                    ValuesStackItem {
                        is_special_string: false,
                        didcomma: *didcomma,
                        didcolon: *didcolon,
                        val: tv,
                    },
                    stack,
                    container_stack,
                    &raw mut p,
                    next_map_special,
                    didcomma,
                    didcolon,
                ) != FAIL
                {
                    if *next_map_special {
                        break '_parse_json_number_ret;
                    } else {
                        p = p.offset(-1);
                        break '_parse_json_number_ret;
                    }
                }
            }
        }
        ret = FAIL;
    }
    *pp = p;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn json_decode_string(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = buf;
    let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
    while p < e
        && (*p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == TAB
            || *p as ::core::ffi::c_int == NL
            || *p as ::core::ffi::c_int == CAR)
    {
        p = p.offset(1);
    }
    if p == e {
        emsg(gettext(
            b"E474: Attempt to decode a blank string\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut ret: ::core::ffi::c_int = OK;
    let mut stack: ValuesStack = ValuesStack {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<ValuesStackItem>(),
    };
    let mut container_stack: ContainerStack = ContainerStack {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<ContainerStackItem>(),
    };
    (*rettv).v_type = VAR_UNKNOWN;
    let mut didcomma: bool = false_0 != 0;
    let mut didcolon: bool = false_0 != 0;
    let mut next_map_special: bool = false_0 != 0;
    '_json_decode_string_ret: {
        '_json_decode_string_fail: {
            's_559: while p < e {
                's_49: {
                    loop {
                        '_c2rust_label: {
                            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                                || next_map_special as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                            {
                            } else {
                                __assert_fail(
                                    b"*p == '{' || next_map_special == false\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    640 as ::core::ffi::c_uint,
                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        match *p as ::core::ffi::c_int {
                            125 | 93 => {
                                if container_stack.size == 0 as size_t {
                                    semsg(
                                        gettext(b"E474: No container to close: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    let mut last_container: ContainerStackItem =
                                        *container_stack.items.offset(
                                            container_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        );
                                    if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                                        && last_container.container.v_type as ::core::ffi::c_uint
                                            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        semsg(
                                            gettext(
                                                b"E474: Closing list with curly bracket: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                                        && last_container.container.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        semsg(
                                            gettext(
                                                b"E474: Closing dictionary with square bracket: %.*s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if didcomma {
                                        semsg(
                                            gettext(b"E474: Trailing comma: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if didcolon {
                                        semsg(
                                            gettext(
                                                b"E474: Expected value after colon: %.*s\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if last_container.stack_index
                                        != stack.size.wrapping_sub(1 as size_t)
                                    {
                                        '_c2rust_label_0: {
                                            if last_container.stack_index
                                                < stack.size.wrapping_sub(1 as size_t)
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"last_container.stack_index < kv_size(stack) - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    663 as ::core::ffi::c_uint,
                                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        semsg(
                                            gettext(b"E474: Expected value: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if stack.size == 1 as size_t {
                                        p = p.offset(1);
                                        container_stack.size = container_stack.size.wrapping_sub(1);
                                        break 's_559;
                                    } else {
                                        stack.size = stack.size.wrapping_sub(1);
                                        if json_decoder_pop(
                                            *stack.items.offset(stack.size as isize),
                                            &raw mut stack,
                                            &raw mut container_stack,
                                            &raw mut p,
                                            &raw mut next_map_special,
                                            &raw mut didcomma,
                                            &raw mut didcolon,
                                        ) == FAIL
                                        {
                                            break '_json_decode_string_fail;
                                        }
                                        '_c2rust_label_1: {
                                            if !next_map_special {
                                            } else {
                                                __assert_fail(
                                                    b"!next_map_special\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    677 as ::core::ffi::c_uint,
                                                    b"int json_decode_string(const char *const, const size_t, typval_T *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        break;
                                    }
                                }
                            }
                            44 => {
                                if container_stack.size == 0 as size_t {
                                    semsg(
                                        gettext(
                                            b"E474: Comma not inside container: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    let mut last_container_0: ContainerStackItem =
                                        *container_stack.items.offset(
                                            container_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        );
                                    if didcomma {
                                        semsg(
                                            gettext(b"E474: Duplicate comma: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if didcolon {
                                        semsg(
                                            gettext(b"E474: Comma after colon: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if last_container_0.container.v_type
                                        as ::core::ffi::c_uint
                                        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                        && last_container_0.stack_index
                                            != stack.size.wrapping_sub(1 as size_t)
                                    {
                                        semsg(
                                            gettext(
                                                b"E474: Using comma in place of colon: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if if last_container_0.special_val.is_null() {
                                        if last_container_0.container.v_type as ::core::ffi::c_uint
                                            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            ((*last_container_0.container.vval.v_dict)
                                                .dv_hashtab
                                                .ht_used
                                                == 0 as size_t)
                                                as ::core::ffi::c_int
                                        } else {
                                            (tv_list_len(last_container_0.container.vval.v_list)
                                                == 0 as ::core::ffi::c_int)
                                                as ::core::ffi::c_int
                                        }
                                    } else {
                                        (tv_list_len(last_container_0.special_val)
                                            == 0 as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } != 0
                                    {
                                        semsg(
                                            gettext(b"E474: Leading comma: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        didcomma = true_0 != 0;
                                        break 's_49;
                                    }
                                }
                            }
                            58 => {
                                if container_stack.size == 0 as size_t {
                                    semsg(
                                        gettext(
                                            b"E474: Colon not inside container: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    let mut last_container_1: ContainerStackItem =
                                        *container_stack.items.offset(
                                            container_stack
                                                .size
                                                .wrapping_sub(0 as size_t)
                                                .wrapping_sub(1 as size_t)
                                                as isize,
                                        );
                                    if last_container_1.container.v_type as ::core::ffi::c_uint
                                        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        semsg(
                                            gettext(
                                                b"E474: Using colon not in dictionary: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if last_container_1.stack_index
                                        != stack.size.wrapping_sub(2 as size_t)
                                    {
                                        semsg(
                                            gettext(b"E474: Unexpected colon: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if didcomma {
                                        semsg(
                                            gettext(b"E474: Colon after comma: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else if didcolon {
                                        semsg(
                                            gettext(b"E474: Duplicate colon: %.*s\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            e.offset_from(p) as ::core::ffi::c_int,
                                            p,
                                        );
                                        break '_json_decode_string_fail;
                                    } else {
                                        didcolon = true_0 != 0;
                                        break 's_49;
                                    }
                                }
                            }
                            32 | TAB | NL | CAR => {
                                break 's_49;
                            }
                            110 => {
                                if p.offset(3 as ::core::ffi::c_int as isize) >= e
                                    || strncmp(
                                        p.offset(1 as ::core::ffi::c_int as isize),
                                        b"ull\0".as_ptr() as *const ::core::ffi::c_char,
                                        3 as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    semsg(
                                        gettext(b"E474: Expected null: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    p = p.offset(3 as ::core::ffi::c_int as isize);
                                    if json_decoder_pop(
                                        ValuesStackItem {
                                            is_special_string: false,
                                            didcomma: didcomma,
                                            didcolon: didcolon,
                                            val: typval_T {
                                                v_type: VAR_SPECIAL,
                                                v_lock: VAR_UNLOCKED,
                                                vval: typval_vval_union {
                                                    v_special: kSpecialVarNull,
                                                },
                                            },
                                        },
                                        &raw mut stack,
                                        &raw mut container_stack,
                                        &raw mut p,
                                        &raw mut next_map_special,
                                        &raw mut didcomma,
                                        &raw mut didcolon,
                                    ) == FAIL
                                    {
                                        break '_json_decode_string_fail;
                                    }
                                    if !next_map_special {
                                        break;
                                    }
                                }
                            }
                            116 => {
                                if p.offset(3 as ::core::ffi::c_int as isize) >= e
                                    || strncmp(
                                        p.offset(1 as ::core::ffi::c_int as isize),
                                        b"rue\0".as_ptr() as *const ::core::ffi::c_char,
                                        3 as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    semsg(
                                        gettext(b"E474: Expected true: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    p = p.offset(3 as ::core::ffi::c_int as isize);
                                    if json_decoder_pop(
                                        ValuesStackItem {
                                            is_special_string: false,
                                            didcomma: didcomma,
                                            didcolon: didcolon,
                                            val: typval_T {
                                                v_type: VAR_BOOL,
                                                v_lock: VAR_UNLOCKED,
                                                vval: typval_vval_union {
                                                    v_bool: kBoolVarTrue,
                                                },
                                            },
                                        },
                                        &raw mut stack,
                                        &raw mut container_stack,
                                        &raw mut p,
                                        &raw mut next_map_special,
                                        &raw mut didcomma,
                                        &raw mut didcolon,
                                    ) == FAIL
                                    {
                                        break '_json_decode_string_fail;
                                    }
                                    if !next_map_special {
                                        break;
                                    }
                                }
                            }
                            102 => {
                                if p.offset(4 as ::core::ffi::c_int as isize) >= e
                                    || strncmp(
                                        p.offset(1 as ::core::ffi::c_int as isize),
                                        b"alse\0".as_ptr() as *const ::core::ffi::c_char,
                                        4 as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    semsg(
                                        gettext(b"E474: Expected false: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p) as ::core::ffi::c_int,
                                        p,
                                    );
                                    break '_json_decode_string_fail;
                                } else {
                                    p = p.offset(4 as ::core::ffi::c_int as isize);
                                    if json_decoder_pop(
                                        ValuesStackItem {
                                            is_special_string: false,
                                            didcomma: didcomma,
                                            didcolon: didcolon,
                                            val: typval_T {
                                                v_type: VAR_BOOL,
                                                v_lock: VAR_UNLOCKED,
                                                vval: typval_vval_union {
                                                    v_bool: kBoolVarFalse,
                                                },
                                            },
                                        },
                                        &raw mut stack,
                                        &raw mut container_stack,
                                        &raw mut p,
                                        &raw mut next_map_special,
                                        &raw mut didcomma,
                                        &raw mut didcolon,
                                    ) == FAIL
                                    {
                                        break '_json_decode_string_fail;
                                    }
                                    if !next_map_special {
                                        break;
                                    }
                                }
                            }
                            34 => {
                                if parse_json_string(
                                    buf,
                                    buf_len,
                                    &raw mut p,
                                    &raw mut stack,
                                    &raw mut container_stack,
                                    &raw mut next_map_special,
                                    &raw mut didcomma,
                                    &raw mut didcolon,
                                ) == FAIL
                                {
                                    break '_json_decode_string_fail;
                                } else if !next_map_special {
                                    break;
                                }
                            }
                            45 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                                if parse_json_number(
                                    buf,
                                    buf_len,
                                    &raw mut p,
                                    &raw mut stack,
                                    &raw mut container_stack,
                                    &raw mut next_map_special,
                                    &raw mut didcomma,
                                    &raw mut didcolon,
                                ) == FAIL
                                {
                                    break '_json_decode_string_fail;
                                }
                                if !next_map_special {
                                    break;
                                }
                            }
                            91 => {
                                let mut list: *mut list_T = tv_list_alloc(
                                    kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t,
                                );
                                tv_list_ref(list);
                                let mut tv: typval_T = typval_T {
                                    v_type: VAR_LIST,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_list: list },
                                };
                                if container_stack.size == container_stack.capacity {
                                    container_stack.capacity = (if container_stack.capacity != 0 {
                                        container_stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    });
                                    container_stack.items = xrealloc(
                                        container_stack.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<ContainerStackItem>()
                                            .wrapping_mul(container_stack.capacity),
                                    )
                                        as *mut ContainerStackItem;
                                } else {
                                };
                                let c2rust_fresh0 = container_stack.size;
                                container_stack.size = container_stack.size.wrapping_add(1);
                                *container_stack.items.offset(c2rust_fresh0 as isize) =
                                    ContainerStackItem {
                                        stack_index: stack.size,
                                        special_val: ::core::ptr::null_mut::<list_T>(),
                                        s: p,
                                        container: tv,
                                    };
                                if stack.size == stack.capacity {
                                    stack.capacity = (if stack.capacity != 0 {
                                        stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    });
                                    stack.items = xrealloc(
                                        stack.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<ValuesStackItem>()
                                            .wrapping_mul(stack.capacity),
                                    )
                                        as *mut ValuesStackItem;
                                } else {
                                };
                                let c2rust_fresh1 = stack.size;
                                stack.size = stack.size.wrapping_add(1);
                                *stack.items.offset(c2rust_fresh1 as isize) = ValuesStackItem {
                                    is_special_string: false,
                                    didcomma: didcomma,
                                    didcolon: didcolon,
                                    val: tv,
                                };
                                break;
                            }
                            123 => {
                                let mut tv_0: typval_T = typval_T {
                                    v_type: VAR_UNKNOWN,
                                    v_lock: VAR_UNLOCKED,
                                    vval: typval_vval_union { v_number: 0 },
                                };
                                let mut val_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
                                if next_map_special {
                                    next_map_special = false_0 != 0;
                                    val_list = decode_create_map_special_dict(
                                        &raw mut tv_0,
                                        kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t,
                                    );
                                } else {
                                    let mut dict: *mut dict_T = tv_dict_alloc();
                                    (*dict).dv_refcount += 1;
                                    tv_0 = typval_T {
                                        v_type: VAR_DICT,
                                        v_lock: VAR_UNLOCKED,
                                        vval: typval_vval_union { v_dict: dict },
                                    };
                                }
                                if container_stack.size == container_stack.capacity {
                                    container_stack.capacity = (if container_stack.capacity != 0 {
                                        container_stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    });
                                    container_stack.items = xrealloc(
                                        container_stack.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<ContainerStackItem>()
                                            .wrapping_mul(container_stack.capacity),
                                    )
                                        as *mut ContainerStackItem;
                                } else {
                                };
                                let c2rust_fresh2 = container_stack.size;
                                container_stack.size = container_stack.size.wrapping_add(1);
                                *container_stack.items.offset(c2rust_fresh2 as isize) =
                                    ContainerStackItem {
                                        stack_index: stack.size,
                                        special_val: val_list,
                                        s: p,
                                        container: tv_0,
                                    };
                                if stack.size == stack.capacity {
                                    stack.capacity = (if stack.capacity != 0 {
                                        stack.capacity << 1 as ::core::ffi::c_int
                                    } else {
                                        8 as size_t
                                    });
                                    stack.items = xrealloc(
                                        stack.items as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<ValuesStackItem>()
                                            .wrapping_mul(stack.capacity),
                                    )
                                        as *mut ValuesStackItem;
                                } else {
                                };
                                let c2rust_fresh3 = stack.size;
                                stack.size = stack.size.wrapping_add(1);
                                *stack.items.offset(c2rust_fresh3 as isize) = ValuesStackItem {
                                    is_special_string: false,
                                    didcomma: didcomma,
                                    didcolon: didcolon,
                                    val: tv_0,
                                };
                                break;
                            }
                            _ => {
                                semsg(
                                    gettext(b"E474: Unidentified byte: %.*s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    e.offset_from(p) as ::core::ffi::c_int,
                                    p,
                                );
                                break '_json_decode_string_fail;
                            }
                        }
                    }
                    didcomma = false_0 != 0;
                    didcolon = false_0 != 0;
                    if container_stack.size == 0 as size_t {
                        p = p.offset(1);
                        break 's_559;
                    }
                }
                p = p.offset(1);
            }
            while p < e {
                match *p as ::core::ffi::c_int {
                    NL | 32 | TAB | CAR => {
                        p = p.offset(1);
                    }
                    _ => {
                        semsg(
                            gettext(b"E474: Trailing characters: %.*s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            e.offset_from(p) as ::core::ffi::c_int,
                            p,
                        );
                        break '_json_decode_string_fail;
                    }
                }
            }
            if stack.size == 1 as size_t && container_stack.size == 0 as size_t {
                stack.size = stack.size.wrapping_sub(1);
                *rettv = (*stack.items.offset(stack.size as isize)).val;
                break '_json_decode_string_ret;
            } else {
                semsg(
                    gettext(b"E474: Unexpected end of input: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    buf_len as ::core::ffi::c_int,
                    buf,
                );
            }
        }
        ret = FAIL;
        while stack.size != 0 {
            stack.size = stack.size.wrapping_sub(1);
            tv_clear(&raw mut (*stack.items.offset(stack.size as isize)).val);
        }
    }
    xfree(stack.items as *mut ::core::ffi::c_void);
    stack.capacity = 0 as size_t;
    stack.size = stack.capacity;
    stack.items = ::core::ptr::null_mut::<ValuesStackItem>();
    xfree(container_stack.items as *mut ::core::ffi::c_void);
    container_stack.capacity = 0 as size_t;
    container_stack.size = container_stack.capacity;
    container_stack.items = ::core::ptr::null_mut::<ContainerStackItem>();
    return ret;
}
unsafe extern "C" fn positive_integer_to_special_typval(
    mut rettv: *mut typval_T,
    mut val: uint64_t,
) {
    if val <= VARNUMBER_MAX as uint64_t {
        *rettv = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: val as varnumber_T,
            },
        };
    } else {
        let list: *mut list_T = tv_list_alloc(4 as ptrdiff_t);
        tv_list_ref(list);
        create_special_dict(
            rettv,
            kMPInteger,
            typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: list },
            },
        );
        tv_list_append_number(list, 1 as varnumber_T);
        tv_list_append_number(
            list,
            (val >> 62 as ::core::ffi::c_int & 0x3 as uint64_t) as varnumber_T,
        );
        tv_list_append_number(
            list,
            (val >> 31 as ::core::ffi::c_int & 0x7fffffff as uint64_t) as varnumber_T,
        );
        tv_list_append_number(list, (val & 0x7fffffff as uint64_t) as varnumber_T);
    };
}
unsafe extern "C" fn typval_parse_enter(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut result: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    let mut parent: *mut mpack_node_t = if (*node.offset(-(1 as ::core::ffi::c_int as isize))).pos
        == -1 as ::core::ffi::c_int as size_t
    {
        ::core::ptr::null_mut::<mpack_node_t>()
    } else {
        node.offset(-(1 as ::core::ffi::c_int as isize))
    };
    if !parent.is_null() {
        match (*parent).tok.type_0 as ::core::ffi::c_uint {
            7 => {
                let mut list: *mut list_T =
                    (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut list_T;
                result = tv_list_append_owned_tv(
                    list,
                    typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    },
                );
            }
            8 => {
                let mut items: *mut [typval_T; 2] =
                    (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut [typval_T; 2];
                result = (&raw mut *items.offset((*parent).pos as isize) as *mut typval_T)
                    .offset((*parent).key_visited as isize);
            }
            10 | 9 | 11 => {
                '_c2rust_label: {
                    if (*node).tok.type_0 as ::core::ffi::c_uint
                        == MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                    } else {
                        __assert_fail(
                            b"node->tok.type == MPACK_TOKEN_CHUNK\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/decode.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            932 as ::core::ffi::c_uint,
                            b"void typval_parse_enter(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
            }
            _ => {
                abort();
            }
        }
    } else {
        result = (*parser).data.p as *mut typval_T;
    }
    (*node).data[0 as ::core::ffi::c_int as usize].p = result as *mut ::core::ffi::c_void;
    (*node).data[1 as ::core::ffi::c_int as usize].p = NULL;
    match (*node).tok.type_0 as ::core::ffi::c_uint {
        1 => {
            *result = typval_T {
                v_type: VAR_SPECIAL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_special: kSpecialVarNull,
                },
            };
        }
        2 => {
            *result = typval_T {
                v_type: VAR_BOOL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_bool: (if mpack_unpack_boolean((*node).tok) as ::core::ffi::c_int != 0 {
                        kBoolVarTrue as ::core::ffi::c_int
                    } else {
                        kBoolVarFalse as ::core::ffi::c_int
                    }) as BoolVarValue,
                },
            };
        }
        4 => {
            *result = typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_number: mpack_unpack_sint((*node).tok) as varnumber_T,
                },
            };
        }
        3 => {
            positive_integer_to_special_typval(result, mpack_unpack_uint((*node).tok) as uint64_t);
        }
        5 => {
            *result = typval_T {
                v_type: VAR_FLOAT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_float: mpack_unpack_float_fast((*node).tok),
                },
            };
        }
        9 | 10 | 11 => {
            (*node).data[1 as ::core::ffi::c_int as usize].p =
                xmallocz((*node).tok.length as size_t);
        }
        6 => {
            let mut data: *mut ::core::ffi::c_char =
                (*parent).data[1 as ::core::ffi::c_int as usize].p as *mut ::core::ffi::c_char;
            memcpy(
                data.offset((*parent).pos as isize) as *mut ::core::ffi::c_void,
                (*node).tok.data.chunk_ptr as *const ::core::ffi::c_void,
                (*node).tok.length as size_t,
            );
        }
        7 => {
            let list_0: *mut list_T = tv_list_alloc((*node).tok.length as ptrdiff_t);
            tv_list_ref(list_0);
            *result = typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: list_0 },
            };
            (*node).data[1 as ::core::ffi::c_int as usize].p = list_0 as *mut ::core::ffi::c_void;
        }
        8 => {
            (*node).data[1 as ::core::ffi::c_int as usize].p = xmallocz(
                ((*node).tok.length.wrapping_mul(2 as mpack_uint32_t) as size_t)
                    .wrapping_mul(::core::mem::size_of::<typval_T>()),
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn typval_parser_error_free(mut parser: *mut mpack_parser_t) {
    let mut i: uint32_t = 0 as uint32_t;
    while i < (*parser).size as uint32_t {
        let mut node: *mut mpack_node_t =
            (&raw mut (*parser).items as *mut mpack_node_t).offset(i as isize);
        match (*node).tok.type_0 as ::core::ffi::c_uint {
            9 | 10 | 11 | 8 => {
                let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*node).data
                    as *mut mpack_data_t)
                    .offset(1 as ::core::ffi::c_int as isize))
                .p;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn typval_parse_exit(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut result: *mut typval_T =
        (*node).data[0 as ::core::ffi::c_int as usize].p as *mut typval_T;
    's_308: {
        match (*node).tok.type_0 as ::core::ffi::c_uint {
            9 | 10 => {
                *result = decode_string(
                    (*node).data[1 as ::core::ffi::c_int as usize].p as *const ::core::ffi::c_char,
                    (*node).tok.length as size_t,
                    false_0 != 0,
                    true_0 != 0,
                );
                (*node).data[1 as ::core::ffi::c_int as usize].p = NULL;
            }
            11 => {
                let list: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                tv_list_ref(list);
                tv_list_append_number(list, (*node).tok.data.ext_type as varnumber_T);
                let ext_val_list: *mut list_T =
                    tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
                tv_list_append_list(list, ext_val_list);
                create_special_dict(
                    result,
                    kMPExt,
                    typval_T {
                        v_type: VAR_LIST,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_list: list },
                    },
                );
                encode_list_write(
                    ext_val_list as *mut ::core::ffi::c_void,
                    (*node).data[1 as ::core::ffi::c_int as usize].p as *const ::core::ffi::c_char,
                    (*node).tok.length as size_t,
                );
                let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*node).data
                    as *mut mpack_data_t)
                    .offset(1 as ::core::ffi::c_int as isize))
                .p;
                xfree(*ptr_);
                *ptr_ = NULL;
                *ptr_;
            }
            8 => {
                let mut items: *mut [typval_T; 2] =
                    (*node).data[1 as ::core::ffi::c_int as usize].p as *mut [typval_T; 2];
                let mut i: size_t = 0 as size_t;
                's_251: {
                    while i < (*node).tok.length as size_t {
                        let mut key: *mut typval_T = (&raw mut *items.offset(i as isize)
                            as *mut typval_T)
                            .offset(0 as ::core::ffi::c_int as isize);
                        if (*key).v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*key).vval.v_string.is_null()
                            || *(*key)
                                .vval
                                .v_string
                                .offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == NUL
                        {
                            break 's_251;
                        }
                        i = i.wrapping_add(1);
                    }
                    dict = tv_dict_alloc();
                    (*dict).dv_refcount += 1;
                    *result = typval_T {
                        v_type: VAR_DICT,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_dict: dict },
                    };
                    let mut i_0: size_t = 0 as size_t;
                    while i_0 < (*node).tok.length as size_t {
                        let mut key_0: *mut ::core::ffi::c_char = (*items.offset(i_0 as isize))
                            [0 as ::core::ffi::c_int as usize]
                            .vval
                            .v_string;
                        let mut keylen: size_t = strlen(key_0);
                        let di: *mut dictitem_T =
                            xmallocz((17 as size_t).wrapping_add(keylen)) as *mut dictitem_T;
                        memcpy(
                            (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                .offset(0 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            key_0 as *const ::core::ffi::c_void,
                            keylen,
                        );
                        (*di).di_tv.v_type = VAR_UNKNOWN;
                        if tv_dict_add(dict, di) == FAIL {
                            let dhi_ht_: *mut hashtab_T = &raw mut (*dict).dv_hashtab;
                            let mut dhi_todo_: size_t = (*dhi_ht_).ht_used;
                            let mut dhi_: *mut hashitem_T = (*dhi_ht_).ht_array;
                            while dhi_todo_ != 0 {
                                if !((*dhi_).hi_key.is_null()
                                    || (*dhi_).hi_key == &raw mut hash_removed)
                                {
                                    dhi_todo_ = dhi_todo_.wrapping_sub(1);
                                    let d: *mut dictitem_T = (*dhi_)
                                        .hi_key
                                        .offset(-(17 as ::core::ffi::c_ulong as isize))
                                        as *mut dictitem_T;
                                    (*d).di_tv.v_type = VAR_SPECIAL;
                                    (*d).di_tv.vval.v_special = kSpecialVarNull;
                                }
                                dhi_ = dhi_.offset(1);
                            }
                            tv_clear(result);
                            xfree(di as *mut ::core::ffi::c_void);
                            break 's_251;
                        } else {
                            (*di).di_tv =
                                (*items.offset(i_0 as isize))[1 as ::core::ffi::c_int as usize];
                            i_0 = i_0.wrapping_add(1);
                        }
                    }
                    let mut i_1: size_t = 0 as size_t;
                    while i_1 < (*node).tok.length as size_t {
                        xfree(
                            (*items.offset(i_1 as isize))[0 as ::core::ffi::c_int as usize]
                                .vval
                                .v_string as *mut ::core::ffi::c_void,
                        );
                        i_1 = i_1.wrapping_add(1);
                    }
                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                        &raw mut (*(&raw mut (*node).data as *mut mpack_data_t)
                            .offset(1 as ::core::ffi::c_int as isize))
                        .p;
                    xfree(*ptr__0);
                    *ptr__0 = NULL;
                    *ptr__0;
                    break 's_308;
                }
                let list_0: *mut list_T =
                    decode_create_map_special_dict(result, (*node).tok.length as ptrdiff_t);
                let mut i_2: size_t = 0 as size_t;
                while i_2 < (*node).tok.length as size_t {
                    let kv_pair: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                    tv_list_append_list(list_0, kv_pair);
                    tv_list_append_owned_tv(
                        kv_pair,
                        (*items.offset(i_2 as isize))[0 as ::core::ffi::c_int as usize],
                    );
                    tv_list_append_owned_tv(
                        kv_pair,
                        (*items.offset(i_2 as isize))[1 as ::core::ffi::c_int as usize],
                    );
                    i_2 = i_2.wrapping_add(1);
                }
                let mut ptr__1: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*node).data
                    as *mut mpack_data_t)
                    .offset(1 as ::core::ffi::c_int as isize))
                .p;
                xfree(*ptr__1);
                *ptr__1 = NULL;
                *ptr__1;
            }
            _ => {}
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn mpack_parse_typval(
    mut parser: *mut mpack_parser_t,
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    return mpack_parse(
        parser,
        data,
        size,
        Some(
            typval_parse_enter
                as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
        ),
        Some(
            typval_parse_exit as unsafe extern "C" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn unpack_typval(
    mut data: *mut *const ::core::ffi::c_char,
    mut size: *mut size_t,
    mut ret: *mut typval_T,
) -> ::core::ffi::c_int {
    (*ret).v_type = VAR_UNKNOWN;
    let mut parser: mpack_parser_t = mpack_parser_t {
        data: mpack_data_t {
            p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        size: 0,
        capacity: 0,
        status: 0,
        exiting: 0,
        tokbuf: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_0 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        items: [mpack_node_t {
            tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed_0 {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            pos: 0,
            key_visited: 0,
            data: [mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 2],
        }; 33],
    };
    mpack_parser_init(&raw mut parser, 0 as mpack_uint32_t);
    parser.data.p = ret as *mut ::core::ffi::c_void;
    let mut status: ::core::ffi::c_int = mpack_parse_typval(&raw mut parser, data, size);
    if status != MPACK_OK as ::core::ffi::c_int {
        typval_parser_error_free(&raw mut parser);
        tv_clear(ret);
    }
    return status;
}
pub const SURROGATE_HI_START: ::core::ffi::c_int = 0xd800 as ::core::ffi::c_int;
pub const SURROGATE_HI_END: ::core::ffi::c_int = 0xdbff as ::core::ffi::c_int;
pub const SURROGATE_LO_START: ::core::ffi::c_int = 0xdc00 as ::core::ffi::c_int;
pub const SURROGATE_LO_END: ::core::ffi::c_int = 0xdfff as ::core::ffi::c_int;
pub const SURROGATE_FIRST_CHAR: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
