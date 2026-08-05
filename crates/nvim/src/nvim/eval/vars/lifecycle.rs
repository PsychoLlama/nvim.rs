//! Creating, clearing and collecting the variable dictionaries.
//!
//! `evalvars_init` builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn evalvars_init() {
    unsafe {
        init_var_dict(get_globvar_dict(), globvars_var.ptr(), VAR_DEF_SCOPE);
        init_var_dict(vimvardict.ptr(), vimvars_var.ptr(), VAR_SCOPE);
        (*vimvardict.ptr()).dv_lock = VAR_FIXED;
        hash_init(compat_hashtab.ptr());
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[VimVar; 106]>()
            .wrapping_div(::core::mem::size_of::<VimVar>())
            .wrapping_div(
                (::core::mem::size_of::<[VimVar; 106]>()
                    .wrapping_rem(::core::mem::size_of::<VimVar>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            let mut p: *mut VimVar = (vimvars.ptr() as *mut VimVar).offset(i as isize);
            '_c2rust_label: {
                if strlen((*p).vv_name) <= 16 as size_t {
                } else {
                    __assert_fail(
                        b"strlen(p->vv_name) <= VIMVAR_KEY_LEN\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        268 as ::core::ffi::c_uint,
                        b"void evalvars_init(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            strcpy(
                &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
                (*p).vv_name,
            );
            if (*p).vv_flags as ::core::ffi::c_int & VV_RO != 0 {
                (*p).vv_di.di_flags = (DI_FLAGS_RO as ::core::ffi::c_int
                    | DI_FLAGS_FIX as ::core::ffi::c_int)
                    as uint8_t;
            } else if (*p).vv_flags as ::core::ffi::c_int & VV_RO_SBX != 0 {
                (*p).vv_di.di_flags = (DI_FLAGS_RO_SBX as ::core::ffi::c_int
                    | DI_FLAGS_FIX as ::core::ffi::c_int)
                    as uint8_t;
            } else {
                (*p).vv_di.di_flags = DI_FLAGS_FIX as ::core::ffi::c_int as uint8_t;
            }
            if (*p).vv_di.di_tv.v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                hash_add(
                    &raw mut (*vimvardict.ptr()).dv_hashtab,
                    &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
                );
            }
            if (*p).vv_flags as ::core::ffi::c_int & VV_COMPAT != 0 {
                hash_add(
                    compat_hashtab.ptr(),
                    &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
                );
            }
            i = i.wrapping_add(1);
        }
        let vim_version: ::core::ffi::c_int = min_vim_version();
        set_vim_var_nr(VV_VERSION, vim_version as varnumber_T);
        set_vim_var_nr(
            VV_VERSIONLONG,
            (vim_version * 10000 as ::core::ffi::c_int + highest_patch()) as varnumber_T,
        );
        let msgpack_types_dict: *mut dict_T = tv_dict_alloc();
        let mut i_0: size_t = 0 as size_t;
        while i_0 < msgpack_type_names.len() {
            let type_list: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
            tv_list_set_lock(type_list, VAR_FIXED);
            tv_list_ref(type_list);
            let di: *mut dictitem_T = tv_dict_item_alloc(msgpack_type_names[i_0 as usize].as_ptr());
            (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                as uint8_t;
            (*di).di_tv = typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: type_list },
            };
            (*eval_msgpack_type_lists.ptr())[i_0 as usize] = type_list;
            if tv_dict_add(msgpack_types_dict, di) == FAIL {
                abort();
            }
            i_0 = i_0.wrapping_add(1);
        }
        (*msgpack_types_dict).dv_lock = VAR_FIXED;
        set_vim_var_dict(VV_MSGPACK_TYPES, msgpack_types_dict);
        set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
        set_vim_var_dict(VV_EVENT, tv_dict_alloc_lock(VAR_FIXED));
        set_vim_var_list(
            VV_ERRORS,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
        set_vim_var_nr(VV_STDERR, CHAN_STDERR as varnumber_T);
        set_vim_var_nr(VV_SEARCHFORWARD, 1 as varnumber_T);
        set_vim_var_nr(VV_HLSEARCH, 1 as varnumber_T);
        set_vim_var_nr(VV_COUNT1, 1 as varnumber_T);
        set_vim_var_special(VV_EXITING, kSpecialVarNull);
        set_vim_var_nr(
            VV_TYPE_NUMBER,
            VAR_TYPE_NUMBER as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_STRING,
            VAR_TYPE_STRING as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_FUNC,
            VAR_TYPE_FUNC as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_LIST,
            VAR_TYPE_LIST as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_DICT,
            VAR_TYPE_DICT as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_FLOAT,
            VAR_TYPE_FLOAT as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_BOOL,
            VAR_TYPE_BOOL as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_nr(
            VV_TYPE_BLOB,
            VAR_TYPE_BLOB as ::core::ffi::c_int as varnumber_T,
        );
        set_vim_var_bool(VV_FALSE, kBoolVarFalse);
        set_vim_var_bool(VV_TRUE, kBoolVarTrue);
        set_vim_var_special(VV_NULL, kSpecialVarNull);
        set_vim_var_nr(VV_NUMBERMAX, VARNUMBER_MAX as varnumber_T);
        set_vim_var_nr(VV_NUMBERMIN, VARNUMBER_MIN as varnumber_T);
        set_vim_var_nr(
            VV_NUMBERSIZE,
            ::core::mem::size_of::<varnumber_T>().wrapping_mul(8 as usize) as varnumber_T,
        );
        set_vim_var_nr(VV_MAXCOL, MAXCOL as ::core::ffi::c_int as varnumber_T);
        set_vim_var_nr(
            VV_ECHOSPACE,
            (sc_col.get() - 1 as ::core::ffi::c_int) as varnumber_T,
        );
        let mut vvlua_partial: *mut partial_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
        (*vvlua_partial).pt_name = xmallocz(0 as size_t) as *mut ::core::ffi::c_char;
        (*vvlua_partial).pt_refcount += 1;
        set_vim_var_partial(VV_LUA, vvlua_partial);
        set_reg_var(0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn garbage_collect_globvars(
    mut copyID: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        return set_ref_in_ht(
            &raw mut (*globvardict.ptr()).dv_hashtab,
            copyID,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn garbage_collect_vimvars(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        return set_ref_in_ht(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            copyID,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
    }
}

pub unsafe extern "C" fn garbage_collect_scriptvars(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut abort_0: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i <= (*script_items.ptr()).ga_len {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_ht(
                    &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                        .offset((i - 1 as ::core::ffi::c_int) as isize))
                    .sn_vars)
                        .sv_dict
                        .dv_hashtab,
                    copyID,
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0;
            i += 1;
        }
        return abort_0;
    }
}

pub unsafe extern "C" fn set_internal_string_var(
    mut name: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut tv: typval_T = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: value },
        };
        set_var(name, strlen(name), &raw mut tv, true_0 != 0);
    }
}

pub unsafe extern "C" fn del_menutrans_vars() {
    unsafe {
        hash_lock(&raw mut (*globvardict.ptr()).dv_hashtab);
        let hiht_: *mut hashtab_T = &raw mut (*globvardict.ptr()).dv_hashtab;
        let mut hitodo_: size_t = (*hiht_).ht_used;
        let mut hi: *mut hashitem_T = (*hiht_).ht_array;
        while hitodo_ != 0 {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                hitodo_ = hitodo_.wrapping_sub(1);
                if strncmp(
                    (*hi).hi_key,
                    b"menutrans_\0".as_ptr() as *const ::core::ffi::c_char,
                    10 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    delete_var(&raw mut (*globvardict.ptr()).dv_hashtab, hi);
                }
            }
            hi = hi.offset(1);
        }
        hash_unlock(&raw mut (*globvardict.ptr()).dv_hashtab);
    }
}

pub unsafe extern "C" fn get_globvar_dict() -> *mut dict_T {
    return globvardict.ptr();
}

pub unsafe extern "C" fn get_globvar_ht() -> *mut hashtab_T {
    unsafe {
        return &raw mut (*globvardict.ptr()).dv_hashtab;
    }
}

pub unsafe extern "C" fn get_vimvar_dict() -> *mut dict_T {
    return vimvardict.ptr();
}

pub unsafe extern "C" fn new_script_vars(mut id: scid_T) {
    unsafe {
        let mut sv: *mut scriptvar_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<scriptvar_T>()) as *mut scriptvar_T;
        init_var_dict(&raw mut (*sv).sv_dict, &raw mut (*sv).sv_var, VAR_SCOPE);
        (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((id as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
        .sn_vars = sv;
    }
}

pub unsafe extern "C" fn init_var_dict(
    mut dict: *mut dict_T,
    mut dict_var: *mut ScopeDictDictItem,
    mut scope: ScopeType,
) {
    unsafe {
        hash_init(&raw mut (*dict).dv_hashtab);
        (*dict).dv_lock = VAR_UNLOCKED;
        (*dict).dv_scope = scope;
        (*dict).dv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
        (*dict).dv_copyID = 0 as ::core::ffi::c_int;
        (*dict_var).di_tv.vval.v_dict = dict;
        (*dict_var).di_tv.v_type = VAR_DICT;
        (*dict_var).di_tv.v_lock = VAR_FIXED;
        (*dict_var).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        *(&raw mut (*dict_var).di_key as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) = NUL;
        QUEUE_INIT(&raw mut (*dict).watchers);
    }
}

pub unsafe extern "C" fn unref_var_dict(mut dict: *mut dict_T) {
    unsafe {
        (*dict).dv_refcount -= DO_NOT_FREE_CNT as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        tv_dict_unref(dict);
    }
}

pub unsafe extern "C" fn vars_clear(mut ht: *mut hashtab_T) {
    unsafe {
        vars_clear_ext(ht, true_0 != 0);
    }
}

pub unsafe extern "C" fn vars_clear_ext(mut ht: *mut hashtab_T, mut free_val: bool) {
    unsafe {
        let mut todo: ::core::ffi::c_int = 0;
        let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
        let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        hash_lock(ht);
        todo = (*ht).ht_used as ::core::ffi::c_int;
        hi = (*ht).ht_array;
        while todo > 0 as ::core::ffi::c_int {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                v = (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
                if free_val {
                    tv_clear(&raw mut (*v).di_tv);
                }
                if (*v).di_flags as ::core::ffi::c_int & DI_FLAGS_ALLOC as ::core::ffi::c_int != 0 {
                    xfree(v as *mut ::core::ffi::c_void);
                }
            }
            hi = hi.offset(1);
        }
        hash_clear(ht);
        hash_init(ht);
    }
}

pub(crate) unsafe extern "C" fn delete_var(mut ht: *mut hashtab_T, mut hi: *mut hashitem_T) {
    unsafe {
        let mut di: *mut dictitem_T =
            (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
        hash_remove(ht, hi);
        tv_clear(&raw mut (*di).di_tv);
        xfree(di as *mut ::core::ffi::c_void);
    }
}
