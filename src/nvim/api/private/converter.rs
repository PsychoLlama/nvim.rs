extern "C" {
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn decode_string(
        s: *const ::core::ffi::c_char,
        len: size_t,
        force_blob: bool,
        s_allocated: bool,
    ) -> typval_T;
    static mut hash_removed: ::core::ffi::c_char;
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_owned_tv(l: *mut list_T, tv: typval_T) -> *mut typval_T;
    fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T;
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int;
    fn find_func(name: *const ::core::ffi::c_char) -> *mut ufunc_T;
    fn register_luafunc(ref_0: LuaRef) -> *mut ::core::ffi::c_char;
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;
    fn partial_name(pt: *mut partial_T) -> *mut ::core::ffi::c_char;
    fn get_copyID() -> ::core::ffi::c_int;
    fn encode_vim_list_to_buf(
        list: *const list_T,
        ret_len: *mut size_t,
        ret_buf: *mut *mut ::core::ffi::c_char,
    ) -> bool;
    static mut eval_msgpack_type_lists: [*const list_T; 8];
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
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
    pub fc_fixvar: [C2Rust_Unnamed_0; 12],
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
pub struct C2Rust_Unnamed_0 {
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
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
    pub init_array: [Object; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct EncodedData {
    pub stack: C2Rust_Unnamed_1,
    pub arena: *mut Arena,
    pub reuse_strdata: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal {
    pub type_0: MPConvStackValType,
    pub tv: *mut typval_T,
    pub saved_copyID: ::core::ffi::c_int,
    pub data: C2Rust_Unnamed_2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub d: C2Rust_Unnamed_6,
    pub l: C2Rust_Unnamed_5,
    pub p: C2Rust_Unnamed_4,
    pub a: C2Rust_Unnamed_3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub arg: *mut typval_T,
    pub argv: *mut typval_T,
    pub todo: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_4 {
    pub stage: MPConvPartialStage,
    pub pt: *mut partial_T,
}
pub type MPConvPartialStage = ::core::ffi::c_uint;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_5 {
    pub list: *mut list_T,
    pub li: *mut listitem_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_6 {
    pub dict: *mut dict_T,
    pub dictp: *mut *mut dict_T,
    pub hi: *mut hashitem_T,
    pub todo: size_t,
}
pub type MPConvStackValType = ::core::ffi::c_uint;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MPConvStackVal,
    pub init_array: [MPConvStackVal; 8],
}
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPArray: MessagePackType = 5;
pub const kMPString: MessagePackType = 4;
pub const kMPFloat: MessagePackType = 3;
pub const kMPInteger: MessagePackType = 2;
pub const kMPBoolean: MessagePackType = 1;
pub const kMPNil: MessagePackType = 0;
pub type MessagePackType = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
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
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    (*l).lv_copyID = copyid;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    return (*l).lv_copyID;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_strlen(tv: *const typval_T) -> size_t {
    '_c2rust_label: {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_STRING\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.h\0".as_ptr()
                    as *const ::core::ffi::c_char,
                77 as ::core::ffi::c_uint,
                b"size_t tv_strlen(const typval_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return if (*tv).vval.v_string.is_null() {
        0 as size_t
    } else {
        strlen((*tv).vval.v_string)
    };
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
unsafe extern "C" fn typval_cbuf_to_obj(
    mut edata: *mut EncodedData,
    mut data: *const ::core::ffi::c_char,
    mut len: size_t,
) -> Object {
    if (*edata).reuse_strdata {
        return object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: (if len != 0 {
                        data
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    }) as *mut ::core::ffi::c_char,
                    size: len,
                },
            },
        };
    } else {
        return object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string(
                    (*edata).arena,
                    String_0 {
                        data: data as *mut ::core::ffi::c_char,
                        size: len,
                    },
                ),
            },
        };
    };
}
#[inline(always)]
unsafe extern "C" fn typval_encode_list_start(edata: *mut EncodedData, len: size_t) {
    if (*edata).stack.size == (*edata).stack.capacity {
        (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*edata).stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*edata).stack.items = (if (*edata).stack.capacity
            == ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                (*edata).stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*edata).stack.init_array as *mut Object as *mut ::core::ffi::c_void,
                    (*edata).stack.items as *mut ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            }
        } else {
            if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                memcpy(
                    xmalloc(
                        (*edata)
                            .stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    ),
                    (*edata).stack.items as *const ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            } else {
                xrealloc(
                    (*edata).stack.items as *mut ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            }
        }) as *mut Object;
    } else {
    };
    let c2rust_fresh32 = (*edata).stack.size;
    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
    *(*edata).stack.items.offset(c2rust_fresh32 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: arena_array((*edata).arena, len),
        },
    };
}
#[inline(always)]
unsafe extern "C" fn typval_encode_between_list_items(edata: *mut EncodedData) {
    (*edata).stack.size = (*edata).stack.size.wrapping_sub(1);
    let mut item: Object = *(*edata).stack.items.offset((*edata).stack.size as isize);
    let list: *mut Object = (*edata).stack.items.offset(
        (*edata)
            .stack
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    '_c2rust_label: {
        if (*list).type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"list->type == kObjectTypeArray\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                115 as ::core::ffi::c_uint,
                b"void typval_encode_between_list_items(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*list).data.array.size < (*list).data.array.capacity {
        } else {
            __assert_fail(
                b"list->data.array.size < list->data.array.capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                116 as ::core::ffi::c_uint,
                b"void typval_encode_between_list_items(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh33 = (*list).data.array.size;
    (*list).data.array.size = (*list).data.array.size.wrapping_add(1);
    *(*list).data.array.items.offset(c2rust_fresh33 as isize) = item;
}
#[inline(always)]
unsafe extern "C" fn typval_encode_list_end(edata: *mut EncodedData) {
    typval_encode_between_list_items(edata);
    let list: *const Object = (*edata).stack.items.offset(
        (*edata)
            .stack
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    '_c2rust_label: {
        if (*list).data.array.size == (*list).data.array.capacity {
        } else {
            __assert_fail(
                b"list->data.array.size == list->data.array.capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                129 as ::core::ffi::c_uint,
                b"void typval_encode_list_end(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
}
#[inline(always)]
unsafe extern "C" fn typval_encode_dict_start(edata: *mut EncodedData, len: size_t) {
    if (*edata).stack.size == (*edata).stack.capacity {
        (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*edata).stack.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*edata).stack.items = (if (*edata).stack.capacity
            == ::core::mem::size_of::<[Object; 2]>()
                .wrapping_div(::core::mem::size_of::<Object>())
                .wrapping_div(
                    (::core::mem::size_of::<[Object; 2]>()
                        .wrapping_rem(::core::mem::size_of::<Object>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                (*edata).stack.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*edata).stack.init_array as *mut Object as *mut ::core::ffi::c_void,
                    (*edata).stack.items as *mut ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            }
        } else {
            if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                memcpy(
                    xmalloc(
                        (*edata)
                            .stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    ),
                    (*edata).stack.items as *const ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .size
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            } else {
                xrealloc(
                    (*edata).stack.items as *mut ::core::ffi::c_void,
                    (*edata)
                        .stack
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<Object>()),
                )
            }
        }) as *mut Object;
    } else {
    };
    let c2rust_fresh30 = (*edata).stack.size;
    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
    *(*edata).stack.items.offset(c2rust_fresh30 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed {
            dict: arena_dict((*edata).arena, len),
        },
    };
}
#[inline(always)]
unsafe extern "C" fn typval_encode_after_key(edata: *mut EncodedData) {
    (*edata).stack.size = (*edata).stack.size.wrapping_sub(1);
    let mut key: Object = *(*edata).stack.items.offset((*edata).stack.size as isize);
    let dict: *mut Object = (*edata).stack.items.offset(
        (*edata)
            .stack
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    '_c2rust_label: {
        if (*dict).type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"dict->type == kObjectTypeDict\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                154 as ::core::ffi::c_uint,
                b"void typval_encode_after_key(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*dict).data.dict.size < (*dict).data.dict.capacity {
        } else {
            __assert_fail(
                b"dict->data.dict.size < dict->data.dict.capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                155 as ::core::ffi::c_uint,
                b"void typval_encode_after_key(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if key.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*(*dict)
            .data
            .dict
            .items
            .offset((*dict).data.dict.size as isize))
        .key = key.data.string;
    } else {
        (*(*dict)
            .data
            .dict
            .items
            .offset((*dict).data.dict.size as isize))
        .key = String_0 {
            data: b"__INVALID_KEY__\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 16]>().wrapping_sub(1 as size_t),
        };
    };
}
#[inline(always)]
unsafe extern "C" fn typval_encode_between_dict_items(edata: *mut EncodedData) {
    (*edata).stack.size = (*edata).stack.size.wrapping_sub(1);
    let mut val: Object = *(*edata).stack.items.offset((*edata).stack.size as isize);
    let dict: *mut Object = (*edata).stack.items.offset(
        (*edata)
            .stack
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    '_c2rust_label: {
        if (*dict).type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"dict->type == kObjectTypeDict\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                173 as ::core::ffi::c_uint,
                b"void typval_encode_between_dict_items(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*dict).data.dict.size < (*dict).data.dict.capacity {
        } else {
            __assert_fail(
                b"dict->data.dict.size < dict->data.dict.capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                174 as ::core::ffi::c_uint,
                b"void typval_encode_between_dict_items(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh34 = (*dict).data.dict.size;
    (*dict).data.dict.size = (*dict).data.dict.size.wrapping_add(1);
    (*(*dict).data.dict.items.offset(c2rust_fresh34 as isize)).value = val;
}
#[inline(always)]
unsafe extern "C" fn typval_encode_dict_end(edata: *mut EncodedData) {
    typval_encode_between_dict_items(edata);
    let dict: *const Object = (*edata).stack.items.offset(
        (*edata)
            .stack
            .size
            .wrapping_sub(0 as size_t)
            .wrapping_sub(1 as size_t) as isize,
    );
    '_c2rust_label: {
        if (*dict).data.dict.size == (*dict).data.dict.capacity {
        } else {
            __assert_fail(
                b"dict->data.dict.size == dict->data.dict.capacity\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                187 as ::core::ffi::c_uint,
                b"void typval_encode_dict_end(EncodedData *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
}
#[no_mangle]
pub static mut _typval_encode_object_nodict_var: *const dict_T = ::core::ptr::null::<dict_T>();
#[inline(always)]
unsafe extern "C" fn _typval_encode_object_check_self_reference(
    edata: *mut EncodedData,
    _val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    _mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    _conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        if (*edata).stack.size == (*edata).stack.capacity {
            (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 2]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 2]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*edata).stack.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 2]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 2]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*edata).stack.items = (if (*edata).stack.capacity
                == ::core::mem::size_of::<[Object; 2]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 2]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                    (*edata).stack.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*edata).stack.init_array as *mut Object
                            as *mut ::core::ffi::c_void,
                        (*edata).stack.items as *mut ::core::ffi::c_void,
                        (*edata)
                            .stack
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if (*edata).stack.items == &raw mut (*edata).stack.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            (*edata)
                                .stack
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        (*edata).stack.items as *const ::core::ffi::c_void,
                        (*edata)
                            .stack
                            .size
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        (*edata).stack.items as *mut ::core::ffi::c_void,
                        (*edata)
                            .stack
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh31 = (*edata).stack.size;
        (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
        *(*edata).stack.items.offset(c2rust_fresh31 as isize) = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_object_convert_one_value(
    edata: *mut EncodedData,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                let len_: size_t = tv_strlen(tv);
                let str_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                '_c2rust_label: {
                    if len_ == 0 as size_t || !str_.is_null() {
                    } else {
                        __assert_fail(
                            b"len_ == 0 || str_ != NULL\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            335 as ::core::ffi::c_uint,
                            b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if (*edata).stack.size == (*edata).stack.capacity {
                    (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*edata).stack.items = (if (*edata).stack.capacity
                        == ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            (*edata).stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*edata).stack.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            memcpy(
                                xmalloc(
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                (*edata).stack.items as *const ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh5 = (*edata).stack.size;
                (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                *(*edata).stack.items.offset(c2rust_fresh5 as isize) =
                    typval_cbuf_to_obj(edata, str_, len_);
            }
            1 => {
                if (*edata).stack.size == (*edata).stack.capacity {
                    (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*edata).stack.items = (if (*edata).stack.capacity
                        == ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            (*edata).stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*edata).stack.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            memcpy(
                                xmalloc(
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                (*edata).stack.items as *const ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh6 = (*edata).stack.size;
                (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                *(*edata).stack.items.offset(c2rust_fresh6 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*tv).vval.v_number,
                    },
                };
            }
            6 => {
                if (*edata).stack.size == (*edata).stack.capacity {
                    (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*edata).stack.items = (if (*edata).stack.capacity
                        == ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            (*edata).stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*edata).stack.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            memcpy(
                                xmalloc(
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                (*edata).stack.items as *const ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh7 = (*edata).stack.size;
                (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                *(*edata).stack.items.offset(c2rust_fresh7 as isize) = object {
                    type_0: kObjectTypeFloat,
                    data: C2Rust_Unnamed {
                        floating: (*tv).vval.v_float,
                    },
                };
            }
            10 => {
                let len__0: size_t = tv_blob_len((*tv).vval.v_blob) as size_t;
                let blob_: *const blob_T = (*tv).vval.v_blob;
                if (*edata).stack.size == (*edata).stack.capacity {
                    (*edata).stack.capacity = if (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*edata).stack.items = (if (*edata).stack.capacity
                        == ::core::mem::size_of::<[Object; 2]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 2]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            (*edata).stack.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*edata).stack.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if (*edata).stack.items
                            == &raw mut (*edata).stack.init_array as *mut Object
                        {
                            memcpy(
                                xmalloc(
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                (*edata).stack.items as *const ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                (*edata)
                                    .stack
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh8 = (*edata).stack.size;
                (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                *(*edata).stack.items.offset(c2rust_fresh8 as isize) = typval_cbuf_to_obj(
                    edata,
                    (if len__0 != 0 {
                        (*blob_).bv_ga.ga_data
                    } else {
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void
                    }) as *const ::core::ffi::c_char,
                    len__0,
                );
            }
            3 => {
                let fun_: *const ::core::ffi::c_char = (*tv).vval.v_string;
                let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
                if !fun_.is_null()
                    && {
                        fp = find_func(fun_);
                        !fp.is_null()
                    }
                    && (*fp).uf_flags & FC_LUAREF != 0
                {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh9 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh9 as isize) = object {
                        type_0: kObjectTypeLuaRef,
                        data: C2Rust_Unnamed {
                            luaref: api_new_luaref((*fp).uf_luaref),
                        },
                    };
                } else {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh10 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh10 as isize) = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                }
            }
            9 => {
                let pt: *mut partial_T = (*tv).vval.v_partial;
                let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    partial_name(pt)
                };
                let _prefix: *const ::core::ffi::c_char = if !fun.is_null()
                    && !pt.is_null()
                    && (*pt).pt_name.is_null()
                    && (*fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint)
                {
                    b"g:\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                };
                let fun__0: *const ::core::ffi::c_char = fun;
                let mut fp_0: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
                if !fun__0.is_null()
                    && {
                        fp_0 = find_func(fun__0);
                        !fp_0.is_null()
                    }
                    && (*fp_0).uf_flags & FC_LUAREF != 0
                {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh11 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh11 as isize) = object {
                        type_0: kObjectTypeLuaRef,
                        data: C2Rust_Unnamed {
                            luaref: api_new_luaref((*fp_0).uf_luaref),
                        },
                    };
                } else {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh12 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh12 as isize) = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                }
            }
            4 => {
                if (*tv).vval.v_list.is_null()
                    || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh14 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh14 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed {
                            array: Array {
                                size: 0 as size_t,
                                capacity: 0 as size_t,
                                items: ::core::ptr::null_mut::<Object>(),
                            },
                        },
                    };
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int = _typval_encode_object_check_self_reference(
                        edata,
                        (*tv).vval.v_list as *mut ::core::ffi::c_void,
                        &raw mut (*(*tv).vval.v_list).lv_copyID,
                        mpstack,
                        copyID,
                        kMPConvList,
                        objname,
                    );
                    if te_csr_ret != NOTDONE {
                        return te_csr_ret;
                    }
                    typval_encode_list_start(edata, tv_list_len((*tv).vval.v_list) as size_t);
                    '_c2rust_label_0: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh15 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh15 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_2 {
                            l: C2Rust_Unnamed_5 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh16 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh16 as isize) = object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed {
                            boolean: (*tv).vval.v_bool as ::core::ffi::c_uint
                                == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint,
                        },
                    };
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh17 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh17 as isize) = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    if (*edata).stack.size == (*edata).stack.capacity {
                        (*edata).stack.capacity = if (*edata).stack.capacity
                            << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*edata).stack.items = (if (*edata).stack.capacity
                            == ::core::mem::size_of::<[Object; 2]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                (*edata).stack.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*edata).stack.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if (*edata).stack.items
                                == &raw mut (*edata).stack.init_array as *mut Object
                            {
                                memcpy(
                                    xmalloc(
                                        (*edata)
                                            .stack
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    (*edata).stack.items as *const ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    (*edata).stack.items as *mut ::core::ffi::c_void,
                                    (*edata)
                                        .stack
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh18 = (*edata).stack.size;
                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                    *(*edata).stack.items.offset(c2rust_fresh18 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed {
                            dict: Dict {
                                size: 0 as size_t,
                                capacity: 0 as size_t,
                                items: ::core::ptr::null_mut::<KeyValuePair>(),
                            },
                        },
                    };
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_647: {
                        if TYPVAL_ENCODE_ALLOW_SPECIALS != 0
                            && (*(*tv).vval.v_dict).dv_hashtab.ht_used == 2 as size_t
                            && {
                                type_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !type_di.is_null()
                            }
                            && (*type_di).di_tv.v_type as ::core::ffi::c_uint
                                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                            && {
                                val_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !val_di.is_null()
                            }
                        {
                            let mut i: size_t = 0;
                            i = 0 as size_t;
                            while i < ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                if (*type_di).di_tv.vval.v_list
                                    == eval_msgpack_type_lists[i as usize] as *mut list_T
                                {
                                    break;
                                }
                                i = i.wrapping_add(1);
                            }
                            if i != ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                match i as MessagePackType as ::core::ffi::c_uint {
                                    0 => {
                                        if (*edata).stack.size == (*edata).stack.capacity {
                                            (*edata).stack.capacity = if (*edata).stack.capacity
                                                << 1 as ::core::ffi::c_int
                                                > ::core::mem::size_of::<[Object; 2]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 2]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                (*edata).stack.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                ::core::mem::size_of::<[Object; 2]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 2]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                            };
                                            (*edata).stack.items = (if (*edata).stack.capacity
                                                == ::core::mem::size_of::<[Object; 2]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 2]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                if (*edata).stack.items
                                                    == &raw mut (*edata).stack.init_array
                                                        as *mut Object
                                                {
                                                    (*edata).stack.items as *mut ::core::ffi::c_void
                                                } else {
                                                    _memcpy_free(
                                                        &raw mut (*edata).stack.init_array
                                                            as *mut Object
                                                            as *mut ::core::ffi::c_void,
                                                        (*edata).stack.items
                                                            as *mut ::core::ffi::c_void,
                                                        (*edata).stack.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            } else {
                                                if (*edata).stack.items
                                                    == &raw mut (*edata).stack.init_array
                                                        as *mut Object
                                                {
                                                    memcpy(
                                                        xmalloc(
                                                            (*edata).stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        ),
                                                        (*edata).stack.items
                                                            as *const ::core::ffi::c_void,
                                                        (*edata).stack.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                } else {
                                                    xrealloc(
                                                        (*edata).stack.items
                                                            as *mut ::core::ffi::c_void,
                                                        (*edata).stack.capacity.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            })
                                                as *mut Object;
                                        } else {
                                        };
                                        let c2rust_fresh19 = (*edata).stack.size;
                                        (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                                        *(*edata).stack.items.offset(c2rust_fresh19 as isize) =
                                            object {
                                                type_0: kObjectTypeNil,
                                                data: C2Rust_Unnamed { boolean: false },
                                            };
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if (*edata).stack.size == (*edata).stack.capacity {
                                                (*edata).stack.capacity = if (*edata)
                                                    .stack
                                                    .capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    (*edata).stack.capacity
                                                        << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                (*edata).stack.items = (if (*edata).stack.capacity
                                                    == ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if (*edata).stack.items
                                                        == &raw mut (*edata).stack.init_array
                                                            as *mut Object
                                                    {
                                                        (*edata).stack.items
                                                            as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut (*edata).stack.init_array
                                                                as *mut Object
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if (*edata).stack.items
                                                        == &raw mut (*edata).stack.init_array
                                                            as *mut Object
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                (*edata)
                                                                    .stack
                                                                    .capacity
                                                                    .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ),
                                                            ),
                                                            (*edata).stack.items
                                                                as *const ::core::ffi::c_void,
                                                            (*edata).stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            (*edata).stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut Object;
                                            } else {
                                            };
                                            let c2rust_fresh20 = (*edata).stack.size;
                                            (*edata).stack.size =
                                                (*edata).stack.size.wrapping_add(1);
                                            *(*edata).stack.items.offset(c2rust_fresh20 as isize) =
                                                object {
                                                    type_0: kObjectTypeBoolean,
                                                    data: C2Rust_Unnamed {
                                                        boolean: (*val_di).di_tv.vval.v_number != 0,
                                                    },
                                                };
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    2 => {
                                        let mut val_list: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut sign: varnumber_T = 0;
                                        let mut highest_bits: varnumber_T = 0;
                                        let mut high_bits: varnumber_T = 0;
                                        let mut low_bits: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list) != 4 as ::core::ffi::c_int
                                            })
                                        {
                                            let sign_li: *const listitem_T =
                                                tv_list_first(val_list);
                                            if !((*sign_li).li_tv.v_type as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || {
                                                    sign = (*sign_li).li_tv.vval.v_number;
                                                    sign == 0 as varnumber_T
                                                })
                                            {
                                                let highest_bits_li: *const listitem_T =
                                                    (*sign_li).li_next;
                                                if !((*highest_bits_li).li_tv.v_type
                                                    as ::core::ffi::c_uint
                                                    != VAR_NUMBER as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    || {
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
                                                        highest_bits < 0 as varnumber_T
                                                    })
                                                {
                                                    let high_bits_li: *const listitem_T =
                                                        (*highest_bits_li).li_next;
                                                    if !((*high_bits_li).li_tv.v_type
                                                        as ::core::ffi::c_uint
                                                        != VAR_NUMBER as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        || {
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
                                                            high_bits < 0 as varnumber_T
                                                        })
                                                    {
                                                        let low_bits_li: *const listitem_T =
                                                            tv_list_last(val_list);
                                                        if !((*low_bits_li).li_tv.v_type
                                                            as ::core::ffi::c_uint
                                                            != VAR_NUMBER as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || {
                                                                low_bits = (*low_bits_li)
                                                                    .li_tv
                                                                    .vval
                                                                    .v_number;
                                                                low_bits < 0 as varnumber_T
                                                            })
                                                        {
                                                            let number: uint64_t = (highest_bits
                                                                as uint64_t)
                                                                << 62 as ::core::ffi::c_int
                                                                | (high_bits as uint64_t)
                                                                    << 31 as ::core::ffi::c_int
                                                                | low_bits as uint64_t;
                                                            if sign > 0 as varnumber_T {
                                                                if (*edata).stack.size
                                                                    == (*edata).stack.capacity
                                                                {
                                                                    (*edata).stack.capacity = if (*edata).stack.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                        > ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as usize,
                                                                            )
                                                                    {
                                                                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                                                                    } else {
                                                                        ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as size_t,
                                                                            )
                                                                    };
                                                                    (*edata).stack.items = (if (*edata).stack.capacity
                                                                        == ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as usize,
                                                                            )
                                                                    {
                                                                        if (*edata).stack.items
                                                                            == &raw mut (*edata).stack.init_array as *mut Object
                                                                        {
                                                                            (*edata).stack.items as *mut ::core::ffi::c_void
                                                                        } else {
                                                                            _memcpy_free(
                                                                                &raw mut (*edata).stack.init_array as *mut Object
                                                                                    as *mut ::core::ffi::c_void,
                                                                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .size
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        }
                                                                    } else {
                                                                        if (*edata).stack.items
                                                                            == &raw mut (*edata).stack.init_array as *mut Object
                                                                        {
                                                                            memcpy(
                                                                                xmalloc(
                                                                                    (*edata)
                                                                                        .stack
                                                                                        .capacity
                                                                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                                ),
                                                                                (*edata).stack.items as *const ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .size
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        } else {
                                                                            xrealloc(
                                                                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .capacity
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        }
                                                                    }) as *mut Object;
                                                                } else {
                                                                };
                                                                let c2rust_fresh21 =
                                                                    (*edata).stack.size;
                                                                (*edata).stack.size = (*edata)
                                                                    .stack
                                                                    .size
                                                                    .wrapping_add(1);
                                                                *(*edata).stack.items.offset(
                                                                    c2rust_fresh21 as isize,
                                                                ) = object {
                                                                    type_0: kObjectTypeInteger,
                                                                    data: C2Rust_Unnamed {
                                                                        integer: number as Integer,
                                                                    },
                                                                };
                                                            } else {
                                                                if (*edata).stack.size
                                                                    == (*edata).stack.capacity
                                                                {
                                                                    (*edata).stack.capacity = if (*edata).stack.capacity
                                                                        << 1 as ::core::ffi::c_int
                                                                        > ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as usize,
                                                                            )
                                                                    {
                                                                        (*edata).stack.capacity << 1 as ::core::ffi::c_int
                                                                    } else {
                                                                        ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as size_t,
                                                                            )
                                                                    };
                                                                    (*edata).stack.items = (if (*edata).stack.capacity
                                                                        == ::core::mem::size_of::<[Object; 2]>()
                                                                            .wrapping_div(::core::mem::size_of::<Object>())
                                                                            .wrapping_div(
                                                                                (::core::mem::size_of::<[Object; 2]>()
                                                                                    .wrapping_rem(::core::mem::size_of::<Object>()) == 0)
                                                                                    as ::core::ffi::c_int as usize,
                                                                            )
                                                                    {
                                                                        if (*edata).stack.items
                                                                            == &raw mut (*edata).stack.init_array as *mut Object
                                                                        {
                                                                            (*edata).stack.items as *mut ::core::ffi::c_void
                                                                        } else {
                                                                            _memcpy_free(
                                                                                &raw mut (*edata).stack.init_array as *mut Object
                                                                                    as *mut ::core::ffi::c_void,
                                                                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .size
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        }
                                                                    } else {
                                                                        if (*edata).stack.items
                                                                            == &raw mut (*edata).stack.init_array as *mut Object
                                                                        {
                                                                            memcpy(
                                                                                xmalloc(
                                                                                    (*edata)
                                                                                        .stack
                                                                                        .capacity
                                                                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                                ),
                                                                                (*edata).stack.items as *const ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .size
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        } else {
                                                                            xrealloc(
                                                                                (*edata).stack.items as *mut ::core::ffi::c_void,
                                                                                (*edata)
                                                                                    .stack
                                                                                    .capacity
                                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                            )
                                                                        }
                                                                    }) as *mut Object;
                                                                } else {
                                                                };
                                                                let c2rust_fresh22 =
                                                                    (*edata).stack.size;
                                                                (*edata).stack.size = (*edata)
                                                                    .stack
                                                                    .size
                                                                    .wrapping_add(1);
                                                                *(*edata).stack.items.offset(
                                                                    c2rust_fresh22 as isize,
                                                                ) = object {
                                                                    type_0: kObjectTypeInteger,
                                                                    data: C2Rust_Unnamed {
                                                                        integer: number
                                                                            .wrapping_neg()
                                                                            as Integer,
                                                                    },
                                                                };
                                                            }
                                                            break '_typval_encode_stop_converting_one_item;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    3 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_FLOAT as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            if (*edata).stack.size == (*edata).stack.capacity {
                                                (*edata).stack.capacity = if (*edata)
                                                    .stack
                                                    .capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    (*edata).stack.capacity
                                                        << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                (*edata).stack.items = (if (*edata).stack.capacity
                                                    == ::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_div(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if (*edata).stack.items
                                                        == &raw mut (*edata).stack.init_array
                                                            as *mut Object
                                                    {
                                                        (*edata).stack.items
                                                            as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut (*edata).stack.init_array
                                                                as *mut Object
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if (*edata).stack.items
                                                        == &raw mut (*edata).stack.init_array
                                                            as *mut Object
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                (*edata)
                                                                    .stack
                                                                    .capacity
                                                                    .wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ),
                                                            ),
                                                            (*edata).stack.items
                                                                as *const ::core::ffi::c_void,
                                                            (*edata).stack.size.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            (*edata).stack.items
                                                                as *mut ::core::ffi::c_void,
                                                            (*edata).stack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut Object;
                                            } else {
                                            };
                                            let c2rust_fresh23 = (*edata).stack.size;
                                            (*edata).stack.size =
                                                (*edata).stack.size.wrapping_add(1);
                                            *(*edata).stack.items.offset(c2rust_fresh23 as isize) =
                                                object {
                                                    type_0: kObjectTypeFloat,
                                                    data: C2Rust_Unnamed {
                                                        floating: (*val_di).di_tv.vval.v_float,
                                                    },
                                                };
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                let len__1: size_t = len;
                                                let str__0: *const ::core::ffi::c_char = buf;
                                                '_c2rust_label_1: {
                                                    if len__1 == 0 as size_t || !str__0.is_null() {
                                                    } else {
                                                        __assert_fail(
                                                            b"len_ == 0 || str_ != NULL\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            519 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                if (*edata).stack.size == (*edata).stack.capacity {
                                                    (*edata).stack.capacity =
                                                        if (*edata).stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            (*edata).stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as size_t,
                                                                )
                                                        };
                                                    (*edata).stack.items =
                                                        (if (*edata).stack.capacity
                                                            == ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                (*edata).stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*edata)
                                                                        .stack
                                                                        .init_array
                                                                        as *mut Object
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                ),
                                                                (*edata).stack.items as *const ::core::ffi::c_void,
                                                                (*edata)
                                                                    .stack
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut Object;
                                                } else {
                                                };
                                                let c2rust_fresh24 = (*edata).stack.size;
                                                (*edata).stack.size =
                                                    (*edata).stack.size.wrapping_add(1);
                                                *(*edata)
                                                    .stack
                                                    .items
                                                    .offset(c2rust_fresh24 as isize) =
                                                    typval_cbuf_to_obj(edata, str__0, len__1);
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_object_check_self_reference(
                                                    edata,
                                                    (*val_di).di_tv.vval.v_list
                                                        as *mut ::core::ffi::c_void,
                                                    &raw mut (*(*val_di).di_tv.vval.v_list)
                                                        .lv_copyID,
                                                    mpstack,
                                                    copyID,
                                                    kMPConvList,
                                                    objname,
                                                );
                                            if te_csr_ret_0 != NOTDONE {
                                                return te_csr_ret_0;
                                            }
                                            typval_encode_list_start(
                                                edata,
                                                tv_list_len((*val_di).di_tv.vval.v_list) as size_t,
                                            );
                                            '_c2rust_label_2: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                (*mpstack).items = (if (*mpstack).capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut (*mpstack).init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                (*mpstack).capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            (*mpstack).items
                                                                as *const ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                            } else {
                                            };
                                            let c2rust_fresh25 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh25 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_2 {
                                                        l: C2Rust_Unnamed_5 {
                                                            list: (*val_di).di_tv.vval.v_list,
                                                            li: tv_list_first(
                                                                (*val_di).di_tv.vval.v_list,
                                                            ),
                                                        },
                                                    },
                                                };
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    6 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                if (*edata).stack.size == (*edata).stack.capacity {
                                                    (*edata).stack.capacity =
                                                        if (*edata).stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            (*edata).stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as size_t,
                                                                )
                                                        };
                                                    (*edata).stack.items =
                                                        (if (*edata).stack.capacity
                                                            == ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                (*edata).stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*edata)
                                                                        .stack
                                                                        .init_array
                                                                        as *mut Object
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                ),
                                                                (*edata).stack.items as *const ::core::ffi::c_void,
                                                                (*edata)
                                                                    .stack
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut Object;
                                                } else {
                                                };
                                                let c2rust_fresh26 = (*edata).stack.size;
                                                (*edata).stack.size =
                                                    (*edata).stack.size.wrapping_add(1);
                                                *(*edata)
                                                    .stack
                                                    .items
                                                    .offset(c2rust_fresh26 as isize) = object {
                                                    type_0: kObjectTypeDict,
                                                    data: C2Rust_Unnamed {
                                                        dict: Dict {
                                                            size: 0 as size_t,
                                                            capacity: 0 as size_t,
                                                            items: ::core::ptr::null_mut::<
                                                                KeyValuePair,
                                                            >(
                                                            ),
                                                        },
                                                    },
                                                };
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_565: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_565;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_647;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_object_check_self_reference(
                                                        edata,
                                                        val_list_0 as *mut ::core::ffi::c_void,
                                                        &raw mut (*val_list_0).lv_copyID,
                                                        mpstack,
                                                        copyID,
                                                        kMPConvPairs,
                                                        objname,
                                                    );
                                                if te_csr_ret_1 != NOTDONE {
                                                    return te_csr_ret_1;
                                                }
                                                typval_encode_dict_start(
                                                    edata,
                                                    tv_list_len(val_list_0) as size_t,
                                                );
                                                '_c2rust_label_3: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                if (*mpstack).size == (*mpstack).capacity {
                                                    (*mpstack).capacity =
                                                        if (*mpstack).capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            (*mpstack).capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                    (*mpstack).items =
                                                        (if (*mpstack).capacity
                                                            == ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                (*mpstack).items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*mpstack).init_array
                                                                        as *mut MPConvStackVal
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).size.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            MPConvStackVal,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                                ),
                                                                (*mpstack).items as *const ::core::ffi::c_void,
                                                                (*mpstack)
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                MPConvStackVal,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut MPConvStackVal;
                                                } else {
                                                };
                                                let c2rust_fresh27 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh27 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_2 {
                                                            l: C2Rust_Unnamed_5 {
                                                                list: val_list_0,
                                                                li: tv_list_first(val_list_0),
                                                            },
                                                        },
                                                    };
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    7 => {
                                        let mut val_list_1: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut type_0: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list_1 = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
                                            }
                                            || (*tv_list_first(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || {
                                                type_0 = (*tv_list_first(val_list_1))
                                                    .li_tv
                                                    .vval
                                                    .v_number;
                                                type_0 > INT8_MAX as varnumber_T
                                            }
                                            || type_0 < INT8_MIN as varnumber_T
                                            || (*tv_list_last(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        {
                                            let mut len_0: size_t = 0;
                                            let mut buf_0: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*tv_list_last(val_list_1)).li_tv.vval.v_list,
                                                &raw mut len_0,
                                                &raw mut buf_0,
                                            ) {
                                                if (*edata).stack.size == (*edata).stack.capacity {
                                                    (*edata).stack.capacity =
                                                        if (*edata).stack.capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            (*edata).stack.capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as size_t,
                                                                )
                                                        };
                                                    (*edata).stack.items =
                                                        (if (*edata).stack.capacity
                                                            == ::core::mem::size_of::<[Object; 2]>()
                                                                .wrapping_div(
                                                                    ::core::mem::size_of::<Object>(
                                                                    ),
                                                                )
                                                                .wrapping_div(
                                                                    (::core::mem::size_of::<
                                                                        [Object; 2],
                                                                    >(
                                                                    )
                                                                    .wrapping_rem(
                                                                        ::core::mem::size_of::<
                                                                            Object,
                                                                        >(
                                                                        ),
                                                                    ) == 0)
                                                                        as ::core::ffi::c_int
                                                                        as usize,
                                                                )
                                                        {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                (*edata).stack.items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*edata)
                                                                        .stack
                                                                        .init_array
                                                                        as *mut Object
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .size
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*edata).stack.items
                                                                == &raw mut (*edata)
                                                                    .stack
                                                                    .init_array
                                                                    as *mut Object
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                                                ),
                                                                (*edata).stack.items as *const ::core::ffi::c_void,
                                                                (*edata)
                                                                    .stack
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*edata).stack.items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*edata)
                                                                        .stack
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                Object,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut Object;
                                                } else {
                                                };
                                                let c2rust_fresh28 = (*edata).stack.size;
                                                (*edata).stack.size =
                                                    (*edata).stack.size.wrapping_add(1);
                                                *(*edata)
                                                    .stack
                                                    .items
                                                    .offset(c2rust_fresh28 as isize) = object {
                                                    type_0: kObjectTypeNil,
                                                    data: C2Rust_Unnamed { boolean: false },
                                                };
                                                xfree(buf_0 as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    _ => {
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                }
                            }
                        }
                    }
                    let saved_copyID_2: ::core::ffi::c_int = (*(*tv).vval.v_dict).dv_copyID;
                    let te_csr_ret_2: ::core::ffi::c_int =
                        _typval_encode_object_check_self_reference(
                            edata,
                            (*tv).vval.v_dict as *mut ::core::ffi::c_void,
                            &raw mut (*(*tv).vval.v_dict).dv_copyID,
                            mpstack,
                            copyID,
                            kMPConvDict,
                            objname,
                        );
                    if te_csr_ret_2 != NOTDONE {
                        return te_csr_ret_2;
                    }
                    typval_encode_dict_start(edata, (*(*tv).vval.v_dict).dv_hashtab.ht_used);
                    '_c2rust_label_4: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_object_convert_one_value(EncodedData *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh29 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh29 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_2 {
                            d: C2Rust_Unnamed_6 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                }
            }
            0 => {
                internal_error(b"_typval_encode_object_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
unsafe extern "C" fn encode_vim_to_object(
    edata: *mut EncodedData,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_2 {
                d: C2Rust_Unnamed_6 {
                    dict: ::core::ptr::null_mut::<dict_T>(),
                    dictp: ::core::ptr::null_mut::<*mut dict_T>(),
                    hi: ::core::ptr::null_mut::<hashitem_T>(),
                    todo: 0,
                },
            },
        }; 8],
    };
    mpstack.capacity = ::core::mem::size_of::<[MPConvStackVal; 8]>()
        .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
        .wrapping_div(
            (::core::mem::size_of::<[MPConvStackVal; 8]>()
                .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mpstack.size = 0 as size_t;
    mpstack.items = &raw mut mpstack.init_array as *mut MPConvStackVal;
    '_encode_vim_to__error_ret: {
        if _typval_encode_object_convert_one_value(
            edata,
            &raw mut mpstack,
            ::core::ptr::null_mut::<MPConvStackVal>(),
            top_tv,
            copyID,
            objname,
        ) != FAIL
        {
            while mpstack.size != 0 {
                let mut cur_mpsv: *mut MPConvStackVal = mpstack.items.offset(
                    mpstack
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t) as isize,
                );
                let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
                match (*cur_mpsv).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if (*cur_mpsv).data.d.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            (*(*cur_mpsv).data.d.dict).dv_copyID = (*cur_mpsv).saved_copyID;
                            typval_encode_dict_end(edata);
                            continue;
                        } else {
                            if (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used
                            {
                                typval_encode_between_dict_items(edata);
                            }
                            while (*(*cur_mpsv).data.d.hi).hi_key.is_null()
                                || (*(*cur_mpsv).data.d.hi).hi_key == &raw mut hash_removed
                            {
                                (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            }
                            let di: *mut dictitem_T = (*(*cur_mpsv).data.d.hi)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T;
                            (*cur_mpsv).data.d.todo = (*cur_mpsv).data.d.todo.wrapping_sub(1);
                            (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            let len_: size_t = strlen(
                                (&raw mut (*di).di_key as *mut ::core::ffi::c_char)
                                    .offset(0 as ::core::ffi::c_int as isize),
                            );
                            let str_: *const ::core::ffi::c_char = (&raw mut (*di).di_key
                                as *mut ::core::ffi::c_char)
                                .offset(0 as ::core::ffi::c_int as isize);
                            '_c2rust_label: {
                                if len_ == 0 as size_t || !str_.is_null() {
                                } else {
                                    __assert_fail(
                                        b"len_ == 0 || str_ != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        694 as ::core::ffi::c_uint,
                                        b"int encode_vim_to_object(EncodedData *const, typval_T *const, const char *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            if (*edata).stack.size == (*edata).stack.capacity {
                                (*edata).stack.capacity = if (*edata).stack.capacity
                                    << 1 as ::core::ffi::c_int
                                    > ::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_div(::core::mem::size_of::<Object>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_rem(::core::mem::size_of::<Object>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    (*edata).stack.capacity << 1 as ::core::ffi::c_int
                                } else {
                                    ::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_div(::core::mem::size_of::<Object>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_rem(::core::mem::size_of::<Object>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as size_t,
                                        )
                                };
                                (*edata).stack.items = (if (*edata).stack.capacity
                                    == ::core::mem::size_of::<[Object; 2]>()
                                        .wrapping_div(::core::mem::size_of::<Object>())
                                        .wrapping_div(
                                            (::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_rem(::core::mem::size_of::<Object>())
                                                == 0)
                                                as ::core::ffi::c_int
                                                as usize,
                                        ) {
                                    if (*edata).stack.items
                                        == &raw mut (*edata).stack.init_array as *mut Object
                                    {
                                        (*edata).stack.items as *mut ::core::ffi::c_void
                                    } else {
                                        _memcpy_free(
                                            &raw mut (*edata).stack.init_array as *mut Object
                                                as *mut ::core::ffi::c_void,
                                            (*edata).stack.items as *mut ::core::ffi::c_void,
                                            (*edata)
                                                .stack
                                                .size
                                                .wrapping_mul(::core::mem::size_of::<Object>()),
                                        )
                                    }
                                } else {
                                    if (*edata).stack.items
                                        == &raw mut (*edata).stack.init_array as *mut Object
                                    {
                                        memcpy(
                                            xmalloc(
                                                (*edata)
                                                    .stack
                                                    .capacity
                                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                                            ),
                                            (*edata).stack.items as *const ::core::ffi::c_void,
                                            (*edata)
                                                .stack
                                                .size
                                                .wrapping_mul(::core::mem::size_of::<Object>()),
                                        )
                                    } else {
                                        xrealloc(
                                            (*edata).stack.items as *mut ::core::ffi::c_void,
                                            (*edata)
                                                .stack
                                                .capacity
                                                .wrapping_mul(::core::mem::size_of::<Object>()),
                                        )
                                    }
                                })
                                    as *mut Object;
                            } else {
                            };
                            let c2rust_fresh0 = (*edata).stack.size;
                            (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                            *(*edata).stack.items.offset(c2rust_fresh0 as isize) =
                                typval_cbuf_to_obj(edata, str_, len_);
                            typval_encode_after_key(edata);
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            typval_encode_list_end(edata);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                typval_encode_between_list_items(edata);
                            }
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            typval_encode_dict_end(edata);
                            continue;
                        } else {
                            if (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list) {
                                typval_encode_between_dict_items(edata);
                            }
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_object_convert_one_value(
                                edata,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            typval_encode_after_key(edata);
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    typval_encode_list_start(edata, (*pt).pt_argc as size_t);
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                mpstack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut mpstack.init_array
                                                        as *mut MPConvStackVal
                                                        as *mut ::core::ffi::c_void,
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                memcpy(
                                                    xmalloc(mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh1 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh1 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_2 {
                                                a: C2Rust_Unnamed_3 {
                                                    arg: (*pt).pt_argv,
                                                    argv: (*pt).pt_argv,
                                                    todo: (*pt).pt_argc as size_t,
                                                },
                                            },
                                        };
                                }
                                continue;
                            }
                            1 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialEnd;
                                let dict: *mut dict_T = if pt.is_null() {
                                    ::core::ptr::null_mut::<dict_T>()
                                } else {
                                    (*pt).pt_dict
                                };
                                if dict.is_null() {
                                    continue;
                                }
                                if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                    if (*edata).stack.size == (*edata).stack.capacity {
                                        (*edata).stack.capacity = if (*edata).stack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_div(::core::mem::size_of::<Object>())
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_rem(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            (*edata).stack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_div(::core::mem::size_of::<Object>())
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_rem(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        (*edata).stack.items = (if (*edata).stack.capacity
                                            == ::core::mem::size_of::<[Object; 2]>()
                                                .wrapping_div(::core::mem::size_of::<Object>())
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[Object; 2]>()
                                                        .wrapping_rem(
                                                            ::core::mem::size_of::<Object>(),
                                                        )
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if (*edata).stack.items
                                                == &raw mut (*edata).stack.init_array as *mut Object
                                            {
                                                (*edata).stack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut (*edata).stack.init_array
                                                        as *mut Object
                                                        as *mut ::core::ffi::c_void,
                                                    (*edata).stack.items
                                                        as *mut ::core::ffi::c_void,
                                                    (*edata).stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<Object>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if (*edata).stack.items
                                                == &raw mut (*edata).stack.init_array as *mut Object
                                            {
                                                memcpy(
                                                    xmalloc((*edata).stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<Object>(),
                                                    )),
                                                    (*edata).stack.items
                                                        as *const ::core::ffi::c_void,
                                                    (*edata).stack.size.wrapping_mul(
                                                        ::core::mem::size_of::<Object>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    (*edata).stack.items
                                                        as *mut ::core::ffi::c_void,
                                                    (*edata).stack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<Object>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut Object;
                                    } else {
                                    };
                                    let c2rust_fresh2 = (*edata).stack.size;
                                    (*edata).stack.size = (*edata).stack.size.wrapping_add(1);
                                    *(*edata).stack.items.offset(c2rust_fresh2 as isize) = object {
                                        type_0: kObjectTypeDict,
                                        data: C2Rust_Unnamed {
                                            dict: Dict {
                                                size: 0 as size_t,
                                                capacity: 0 as size_t,
                                                items: ::core::ptr::null_mut::<KeyValuePair>(),
                                            },
                                        },
                                    };
                                    continue;
                                } else {
                                    let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                    let te_csr_ret: ::core::ffi::c_int =
                                        _typval_encode_object_check_self_reference(
                                            edata,
                                            dict as *mut ::core::ffi::c_void,
                                            &raw mut (*dict).dv_copyID,
                                            &raw mut mpstack,
                                            copyID,
                                            kMPConvDict,
                                            objname,
                                        );
                                    if te_csr_ret != NOTDONE {
                                        if te_csr_ret == FAIL {
                                            break '_encode_vim_to__error_ret;
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        typval_encode_dict_start(edata, (*dict).dv_hashtab.ht_used);
                                        '_c2rust_label_0: {
                                            if saved_copyID != copyID
                                                && saved_copyID != copyID - 1 as ::core::ffi::c_int
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    789 as ::core::ffi::c_uint,
                                                    b"int encode_vim_to_object(EncodedData *const, typval_T *const, const char *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if mpstack.size == mpstack.capacity {
                                            mpstack.capacity =
                                                if mpstack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    mpstack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                            mpstack.items =
                                                (if mpstack.capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        mpstack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut mpstack.init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            )),
                                                            mpstack.items
                                                                as *const ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                        } else {
                                        };
                                        let c2rust_fresh3 = mpstack.size;
                                        mpstack.size = mpstack.size.wrapping_add(1);
                                        *mpstack.items.offset(c2rust_fresh3 as isize) =
                                            MPConvStackVal {
                                                type_0: kMPConvDict,
                                                tv: ::core::ptr::null_mut::<typval_T>(),
                                                saved_copyID: saved_copyID,
                                                data: C2Rust_Unnamed_2 {
                                                    d: C2Rust_Unnamed_6 {
                                                        dict: dict,
                                                        dictp: &raw mut (*pt).pt_dict,
                                                        hi: (*dict).dv_hashtab.ht_array,
                                                        todo: (*dict).dv_hashtab.ht_used,
                                                    },
                                                },
                                            };
                                        continue;
                                    }
                                }
                            }
                            2 => {
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                continue;
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    4 => {
                        if (*cur_mpsv).data.a.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            typval_encode_list_end(edata);
                            continue;
                        } else {
                            if (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg {
                                typval_encode_between_list_items(edata);
                            }
                            let c2rust_fresh4 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh4;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_1: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/eval/typval_encode.c.h\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_object(EncodedData *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_object_convert_one_value(
                    edata,
                    &raw mut mpstack,
                    cur_mpsv,
                    tv,
                    copyID,
                    objname,
                ) == FAIL
                {
                    break '_encode_vim_to__error_ret;
                }
            }
            if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                *ptr_;
            }
            return OK;
        }
    }
    if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        *ptr__0;
    }
    return FAIL;
}
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn vim_to_object(
    mut obj: *mut typval_T,
    mut arena: *mut Arena,
    mut reuse_strdata: bool,
) -> Object {
    let mut edata: EncodedData = EncodedData {
        stack: C2Rust_Unnamed_1 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
            init_array: [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 2],
        },
        arena: ::core::ptr::null_mut::<Arena>(),
        reuse_strdata: false,
    };
    edata.stack.capacity = ::core::mem::size_of::<[Object; 2]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 2]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    edata.stack.size = 0 as size_t;
    edata.stack.items = &raw mut edata.stack.init_array as *mut Object;
    edata.arena = arena;
    edata.reuse_strdata = reuse_strdata;
    let evo_ret: ::core::ffi::c_int = encode_vim_to_object(
        &raw mut edata,
        obj,
        b"vim_to_object argument\0".as_ptr() as *const ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if evo_ret == 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"evo_ret == OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                254 as ::core::ffi::c_uint,
                b"Object vim_to_object(typval_T *, Arena *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut ret: Object = *edata.stack.items.offset(0 as ::core::ffi::c_int as isize);
    '_c2rust_label_0: {
        if edata.stack.size == 1 as size_t {
        } else {
            __assert_fail(
                b"kv_size(edata.stack) == 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/api/private/converter.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                256 as ::core::ffi::c_uint,
                b"Object vim_to_object(typval_T *, Arena *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if edata.stack.items != &raw mut edata.stack.init_array as *mut Object {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut edata.stack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn object_to_vim(
    mut obj: Object,
    mut tv: *mut typval_T,
    mut err: *mut Error,
) {
    object_to_vim_take_luaref(&raw mut obj, tv, false_0 != 0, err);
}
#[no_mangle]
pub unsafe extern "C" fn object_to_vim_take_luaref(
    mut obj: *mut Object,
    mut tv: *mut typval_T,
    mut take_luaref: bool,
    mut err: *mut Error,
) {
    (*tv).v_type = VAR_UNKNOWN;
    (*tv).v_lock = VAR_UNLOCKED;
    match (*obj).type_0 as ::core::ffi::c_uint {
        0 => {
            (*tv).v_type = VAR_SPECIAL;
            (*tv).vval.v_special = kSpecialVarNull;
        }
        1 => {
            (*tv).v_type = VAR_BOOL;
            (*tv).vval.v_bool = (if (*obj).data.boolean as ::core::ffi::c_int != 0 {
                kBoolVarTrue as ::core::ffi::c_int
            } else {
                kBoolVarFalse as ::core::ffi::c_int
            }) as BoolVarValue;
        }
        8 | 9 | 10 | 2 => {
            (*tv).v_type = VAR_NUMBER;
            (*tv).vval.v_number = (*obj).data.integer;
        }
        3 => {
            (*tv).v_type = VAR_FLOAT;
            (*tv).vval.v_float = (*obj).data.floating as float_T;
        }
        4 => {
            let mut s: String_0 = (*obj).data.string;
            *tv = decode_string(s.data, s.size, false_0 != 0, false_0 != 0);
        }
        5 => {
            let list: *mut list_T = tv_list_alloc((*obj).data.array.size as ptrdiff_t);
            let mut i: uint32_t = 0 as uint32_t;
            while (i as size_t) < (*obj).data.array.size {
                let mut li_tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                object_to_vim_take_luaref(
                    (*obj).data.array.items.offset(i as isize),
                    &raw mut li_tv,
                    take_luaref,
                    err,
                );
                tv_list_append_owned_tv(list, li_tv);
                i = i.wrapping_add(1);
            }
            tv_list_ref(list);
            (*tv).v_type = VAR_LIST;
            (*tv).vval.v_list = list;
        }
        6 => {
            let dict: *mut dict_T = tv_dict_alloc();
            let mut i_0: uint32_t = 0 as uint32_t;
            while (i_0 as size_t) < (*obj).data.dict.size {
                let mut item: *mut KeyValuePair = (*obj).data.dict.items.offset(i_0 as isize);
                let mut key: String_0 = (*item).key;
                let di: *mut dictitem_T = tv_dict_item_alloc(key.data);
                object_to_vim_take_luaref(
                    &raw mut (*item).value,
                    &raw mut (*di).di_tv,
                    take_luaref,
                    err,
                );
                tv_dict_add(dict, di);
                i_0 = i_0.wrapping_add(1);
            }
            (*dict).dv_refcount += 1;
            (*tv).v_type = VAR_DICT;
            (*tv).vval.v_dict = dict;
        }
        7 => {
            let mut ref_0: LuaRef = (*obj).data.luaref;
            if take_luaref {
                (*obj).data.luaref = LUA_NOREF as LuaRef;
            } else {
                ref_0 = api_new_luaref(ref_0);
            }
            let mut name: *mut ::core::ffi::c_char = register_luafunc(ref_0);
            (*tv).v_type = VAR_FUNC;
            (*tv).vval.v_string = xstrdup(name);
        }
        _ => {}
    };
}
