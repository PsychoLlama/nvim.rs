use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn abort() -> !;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn eval_call_provider(
        provider: *mut ::core::ffi::c_char,
        method: *mut ::core::ffi::c_char,
        arguments: *mut list_T,
        discard: bool,
    ) -> typval_T;
    fn eval_has_provider(feat: *const ::core::ffi::c_char, throw_if_fast: bool) -> bool;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn redirecting() -> ::core::ffi::c_int;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    static mut cb_flags: ::core::ffi::c_uint;
    fn get_y_register(reg: ::core::ffi::c_int) -> *mut yankreg_T;
    fn get_y_previous() -> *mut yankreg_T;
    fn update_yankreg_width(reg: *mut yankreg_T);
    fn free_register(reg: *mut yankreg_T);
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type ssize_t = isize;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type proftime_T = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
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
pub type colnr_T = ::core::ffi::c_int;
pub type Timestamp = uint64_t;
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
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const NUM_REGISTERS: C2Rust_Unnamed_0 = 39;
pub const PLUS_REGISTER: C2Rust_Unnamed_0 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_0 = 37;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_0 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_0 = 36;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
pub const kOptCbFlagUnnamed: C2Rust_Unnamed_1 = 1;
pub const kOptCbFlagUnnamedplus: C2Rust_Unnamed_1 = 2;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22;
static batch_change_count: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static clipboard_delay_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static clipboard_needs_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static clipboard_didwarn: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
#[no_mangle]
pub unsafe extern "C" fn adjust_clipboard_name(
    mut name: *mut ::core::ffi::c_int,
    mut quiet: bool,
    mut writing: bool,
) -> *mut yankreg_T {
    let mut target: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut explicit_cb_reg: bool =
        *name == '*' as ::core::ffi::c_int || *name == '+' as ::core::ffi::c_int;
    let mut implicit_cb_reg: bool = *name == NUL
        && cb_flags
            & (kOptCbFlagUnnamed as ::core::ffi::c_int
                | kOptCbFlagUnnamedplus as ::core::ffi::c_int) as ::core::ffi::c_uint
            != 0;
    if !(!explicit_cb_reg && !implicit_cb_reg) {
        if !eval_has_provider(
            b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        ) {
            if batch_change_count.get() <= 1 as ::core::ffi::c_int
                && !quiet
                && (!clipboard_didwarn.get()
                    || explicit_cb_reg as ::core::ffi::c_int != 0 && redirecting() == 0)
            {
                clipboard_didwarn.set(true_0 != 0);
                msg(MSG_NO_CLIP.as_ptr(), 0 as ::core::ffi::c_int);
            }
        } else if explicit_cb_reg {
            target = get_y_register(if *name == '*' as ::core::ffi::c_int {
                STAR_REGISTER as ::core::ffi::c_int
            } else {
                PLUS_REGISTER as ::core::ffi::c_int
            });
            if writing as ::core::ffi::c_int != 0
                && cb_flags
                    & (if *name == '*' as ::core::ffi::c_int {
                        kOptCbFlagUnnamed as ::core::ffi::c_int
                    } else {
                        kOptCbFlagUnnamedplus as ::core::ffi::c_int
                    }) as ::core::ffi::c_uint
                    != 0
            {
                clipboard_needs_update.set(false_0 != 0);
            }
        } else if writing as ::core::ffi::c_int != 0
            && clipboard_delay_update.get() as ::core::ffi::c_int != 0
        {
            clipboard_needs_update.set(true_0 != 0);
        } else if !(!writing && clipboard_needs_update.get() as ::core::ffi::c_int != 0) {
            if cb_flags & kOptCbFlagUnnamedplus as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
                *name = if cb_flags & kOptCbFlagUnnamed as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                    && writing as ::core::ffi::c_int != 0
                {
                    '"' as ::core::ffi::c_int
                } else {
                    '+' as ::core::ffi::c_int
                };
                target = get_y_register(PLUS_REGISTER as ::core::ffi::c_int);
            } else {
                *name = '*' as ::core::ffi::c_int;
                target = get_y_register(STAR_REGISTER as ::core::ffi::c_int);
            }
        }
    }
    return target;
}
pub const MSG_NO_CLIP: [::core::ffi::c_char; 62] = unsafe {
    ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
        *b"clipboard: No provider. Try \":checkhealth\" or \":h clipboard\".\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn get_clipboard(
    mut name: ::core::ffi::c_int,
    mut target: *mut *mut yankreg_T,
    mut quiet: bool,
) -> bool {
    let mut res: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut lines: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut tv_idx: size_t = 0;
    let mut errmsg: bool = true_0 != 0;
    let mut reg: *mut yankreg_T = adjust_clipboard_name(&raw mut name, quiet, false_0 != 0);
    if reg.is_null() {
        return false_0 != 0;
    }
    free_register(reg);
    let args: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    let regname: ::core::ffi::c_char = name as ::core::ffi::c_char;
    tv_list_append_string(args, &raw const regname, 1 as ssize_t);
    let mut result: typval_T = eval_call_provider(
        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"get\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
        false_0 != 0,
    );
    '_err: {
        if result.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if result.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && result.vval.v_number == 0 as varnumber_T
            {
                errmsg = false_0 != 0;
            }
        } else {
            res = result.vval.v_list;
            lines = ::core::ptr::null_mut::<list_T>();
            if tv_list_len(res) == 2 as ::core::ffi::c_int
                && (*tv_list_first(res)).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lines = (*tv_list_first(res)).li_tv.vval.v_list;
                if (*tv_list_last(res)).li_tv.v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    break '_err;
                } else {
                    let mut regtype: *mut ::core::ffi::c_char =
                        (*tv_list_last(res)).li_tv.vval.v_string;
                    if regtype.is_null() || strlen(regtype) > 1 as size_t {
                        break '_err;
                    } else {
                        match *regtype.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                        {
                            0 => {
                                (*reg).y_type = kMTUnknown;
                            }
                            118 | 99 => {
                                (*reg).y_type = kMTCharWise;
                            }
                            86 | 108 => {
                                (*reg).y_type = kMTLineWise;
                            }
                            98 | Ctrl_V => {
                                (*reg).y_type = kMTBlockWise;
                            }
                            _ => {
                                break '_err;
                            }
                        }
                    }
                }
            } else {
                lines = res;
                (*reg).y_type = kMTUnknown;
            }
            (*reg).y_array = xcalloc(
                tv_list_len(lines) as size_t,
                ::core::mem::size_of::<String_0>(),
            ) as *mut String_0;
            (*reg).y_size = tv_list_len(lines) as size_t;
            (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
            (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
            (*reg).timestamp = 0 as Timestamp;
            tv_idx = 0 as size_t;
            let l_: *const list_T = lines;
            's_189: {
                if !l_.is_null() {
                    let mut li: *const listitem_T = (*l_).lv_first;
                    loop {
                        if li.is_null() {
                            break 's_189;
                        }
                        if (*li).li_tv.v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break '_err;
                        }
                        let mut s: *const ::core::ffi::c_char = (*li).li_tv.vval.v_string;
                        let c2rust_fresh0 = tv_idx;
                        tv_idx = tv_idx.wrapping_add(1);
                        *(*reg).y_array.offset(c2rust_fresh0 as isize) =
                            cstr_to_string(if !s.is_null() {
                                s
                            } else {
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                            });
                        li = (*li).li_next;
                    }
                }
            }
            if (*reg).y_size > 0 as size_t
                && (*(*reg)
                    .y_array
                    .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize))
                .size
                    == 0 as size_t
            {
                if (*reg).y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int {
                    xfree(
                        (*(*reg)
                            .y_array
                            .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize))
                        .data as *mut ::core::ffi::c_void,
                    );
                    (*reg).y_size = (*reg).y_size.wrapping_sub(1);
                    if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
                        (*reg).y_type = kMTLineWise;
                    }
                }
            } else if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
                (*reg).y_type = kMTCharWise;
            }
            update_yankreg_width(reg);
            *target = reg;
            return true_0 != 0;
        }
    }
    if !(*reg).y_array.is_null() {
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            xfree((*(*reg).y_array.offset(i as isize)).data as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        xfree((*reg).y_array as *mut ::core::ffi::c_void);
    }
    (*reg).y_array = ::core::ptr::null_mut::<String_0>();
    (*reg).y_size = 0 as size_t;
    (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*reg).timestamp = 0 as Timestamp;
    if errmsg {
        emsg(b"clipboard: provider returned invalid data\0".as_ptr() as *const ::core::ffi::c_char);
    }
    *target = reg;
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn set_clipboard(mut name: ::core::ffi::c_int, mut reg: *mut yankreg_T) {
    if adjust_clipboard_name(&raw mut name, false_0 != 0, true_0 != 0).is_null() {
        return;
    }
    let lines: *mut list_T = tv_list_alloc(
        (*reg).y_size as ptrdiff_t
            + ((*reg).y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int)
                as ::core::ffi::c_int as ptrdiff_t,
    );
    let mut i: size_t = 0 as size_t;
    while i < (*reg).y_size {
        tv_list_append_string(
            lines,
            (*(*reg).y_array.offset(i as isize)).data,
            (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
        );
        i = i.wrapping_add(1);
    }
    let mut regtype: ::core::ffi::c_char = 0;
    match (*reg).y_type as ::core::ffi::c_int {
        1 => {
            regtype = 'V' as ::core::ffi::c_char;
            tv_list_append_string(
                lines,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
        }
        0 => {
            regtype = 'v' as ::core::ffi::c_char;
        }
        2 => {
            regtype = 'b' as ::core::ffi::c_char;
            tv_list_append_string(
                lines,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    let mut args: *mut list_T = tv_list_alloc(3 as ptrdiff_t);
    tv_list_append_list(args, lines);
    tv_list_append_string(args, &raw mut regtype, 1 as ssize_t);
    let mut c2rust_lvalue: [::core::ffi::c_char; 1] = [name as ::core::ffi::c_char];
    tv_list_append_string(
        args,
        &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
        1 as ssize_t,
    );
    eval_call_provider(
        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"set\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        args,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn start_batch_changes() {
    (*batch_change_count.ptr()) += 1;
    if batch_change_count.get() > 1 as ::core::ffi::c_int {
        return;
    }
    clipboard_delay_update.set(true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn end_batch_changes() {
    (*batch_change_count.ptr()) -= 1;
    if batch_change_count.get() > 0 as ::core::ffi::c_int {
        return;
    }
    clipboard_delay_update.set(false_0 != 0);
    if clipboard_needs_update.get() {
        clipboard_needs_update.set(false_0 != 0);
        set_clipboard(NUL, get_y_previous());
    }
}
#[no_mangle]
pub unsafe extern "C" fn save_batch_count() -> ::core::ffi::c_int {
    let mut save_count: ::core::ffi::c_int = batch_change_count.get();
    batch_change_count.set(0 as ::core::ffi::c_int);
    clipboard_delay_update.set(false_0 != 0);
    if clipboard_needs_update.get() {
        clipboard_needs_update.set(false_0 != 0);
        set_clipboard(NUL, get_y_previous());
    }
    return save_count;
}
#[no_mangle]
pub unsafe extern "C" fn restore_batch_count(mut save_count: ::core::ffi::c_int) {
    '_c2rust_label: {
        if batch_change_count.get() == 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"batch_change_count == 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/clipboard.rs\0".as_ptr() as *const ::core::ffi::c_char,
                281 as ::core::ffi::c_uint,
                b"void restore_batch_count(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    batch_change_count.set(save_count);
    if batch_change_count.get() > 0 as ::core::ffi::c_int {
        clipboard_delay_update.set(true_0 != 0);
    }
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
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
