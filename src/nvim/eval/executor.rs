extern "C" {
    fn abort() -> !;
    static e_letwrong: [::core::ffi::c_char; 0];
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn num_divide(n1: varnumber_T, n2: varnumber_T) -> varnumber_T;
    fn num_modulus(n1: varnumber_T, n2: varnumber_T) -> varnumber_T;
    fn grow_string_tv(tv1: *mut typval_T, s2: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_extend(l1: *mut list_T, l2: *mut list_T, bef: *mut listitem_T);
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string_buf(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
}
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint64_t = u64;
pub type size_t = usize;
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
pub type linenr_T = int32_t;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
pub type proftime_T = uint64_t;
pub type varnumber_T = int64_t;
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
    pub fc_fixvar: [C2Rust_Unnamed; 12],
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
pub struct C2Rust_Unnamed {
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn tv_op_blob(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int != '+' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if (*tv2).vval.v_blob.is_null() {
        return OK;
    }
    if (*tv1).vval.v_blob.is_null() {
        (*tv1).vval.v_blob = (*tv2).vval.v_blob;
        (*(*tv1).vval.v_blob).bv_refcount += 1;
        return OK;
    }
    let b1: *mut blob_T = (*tv1).vval.v_blob;
    let b2: *mut blob_T = (*tv2).vval.v_blob;
    let len: ::core::ffi::c_int = tv_blob_len(b2);
    if len > 0 as ::core::ffi::c_int {
        ga_grow(&raw mut (*b1).bv_ga, len);
        memmove(
            ((*b1).bv_ga.ga_data as *mut uint8_t).offset((*b1).bv_ga.ga_len as isize)
                as *mut ::core::ffi::c_void,
            (*b2).bv_ga.ga_data as *mut uint8_t as *const ::core::ffi::c_void,
            len as size_t,
        );
        (*b1).bv_ga.ga_len += len;
    }
    return OK;
}
unsafe extern "C" fn tv_op_list(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int != '+' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if (*tv2).vval.v_list.is_null() {
        return OK;
    }
    if (*tv1).vval.v_list.is_null() {
        (*tv1).vval.v_list = (*tv2).vval.v_list;
        tv_list_ref((*tv1).vval.v_list);
    } else {
        tv_list_extend(
            (*tv1).vval.v_list,
            (*tv2).vval.v_list,
            ::core::ptr::null_mut::<listitem_T>(),
        );
    }
    return OK;
}
unsafe extern "C" fn tv_op_number(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut n: varnumber_T = tv_get_number(tv1);
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut f: float_T = n as float_T;
        if *op as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
            return FAIL;
        }
        match *op as ::core::ffi::c_int {
            43 => {
                f += (*tv2).vval.v_float;
            }
            45 => {
                f -= (*tv2).vval.v_float;
            }
            42 => {
                f *= (*tv2).vval.v_float;
            }
            47 => {
                f /= (*tv2).vval.v_float;
            }
            _ => {}
        }
        tv_clear(tv1);
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f;
    } else {
        match *op as ::core::ffi::c_int {
            43 => {
                n += tv_get_number(tv2);
            }
            45 => {
                n -= tv_get_number(tv2);
            }
            42 => {
                n *= tv_get_number(tv2);
            }
            47 => {
                n = num_divide(n, tv_get_number(tv2));
            }
            37 => {
                n = num_modulus(n, tv_get_number(tv2));
            }
            _ => {}
        }
        tv_clear(tv1);
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n;
    }
    return OK;
}
unsafe extern "C" fn tv_op_string(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut _op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut s2: *const ::core::ffi::c_char =
        tv_get_string_buf(tv2, &raw mut numbuf as *mut ::core::ffi::c_char);
    if grow_string_tv(tv1, s2) == OK {
        return OK;
    }
    let mut tvs: *const ::core::ffi::c_char = tv_get_string(tv1);
    let s: *mut ::core::ffi::c_char = concat_str(tvs, s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = s;
    return OK;
}
unsafe extern "C" fn tv_op_nr_or_string(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    if !vim_strchr(
        b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
        *op as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        return tv_op_number(tv1, tv2, op);
    }
    return tv_op_string(tv1, tv2, op);
}
unsafe extern "C" fn tv_op_float(
    mut tv1: *mut typval_T,
    mut tv2: *const typval_T,
    mut op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *op as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        || (*tv2).v_type as ::core::ffi::c_uint
            != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*tv2).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*tv2).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return FAIL;
    }
    let f: float_T = if (*tv2).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*tv2).vval.v_float
    } else {
        tv_get_number(tv2) as float_T
    };
    match *op as ::core::ffi::c_int {
        43 => {
            (*tv1).vval.v_float += f;
        }
        45 => {
            (*tv1).vval.v_float -= f;
        }
        42 => {
            (*tv1).vval.v_float *= f;
        }
        47 => {
            (*tv1).vval.v_float /= f;
        }
        _ => {}
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn eexe_mod_op(
    tv1: *mut typval_T,
    tv2: *const typval_T,
    op: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*tv2).v_type as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv2).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        || ((*tv2).v_type as ::core::ffi::c_uint
            == VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tv2).v_type as ::core::ffi::c_uint
                == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint)
            && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        semsg(
            &raw const e_letwrong as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            op,
        );
        return FAIL;
    }
    let mut retval: ::core::ffi::c_int = FAIL;
    match (*tv1).v_type as ::core::ffi::c_uint {
        10 => {
            retval = tv_op_blob(tv1, tv2, op);
        }
        4 => {
            retval = tv_op_list(tv1, tv2, op);
        }
        1 | 2 => {
            retval = tv_op_nr_or_string(tv1, tv2, op);
        }
        6 => {
            retval = tv_op_float(tv1, tv2, op);
        }
        0 => {
            abort();
        }
        5 | 3 | 9 | 7 | 8 | _ => {}
    }
    if retval != OK {
        semsg(
            &raw const e_letwrong as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            op,
        );
    }
    return retval;
}
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
