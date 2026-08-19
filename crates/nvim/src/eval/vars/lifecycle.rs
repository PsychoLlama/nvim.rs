//! Creating, clearing and collecting the variable dictionaries.
//!
//! [`evalvars_init`] builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::FAIL;

/// Build the `g:` and `v:` scopes and fill the `v:` table.  Called once, at
/// startup.
///
/// # Safety
/// Called once, before anything reads a variable.
pub unsafe fn evalvars_init() {
    unsafe {
        init_var_dict(get_globvar_dict(), globvars_var.ptr(), VAR_DEF_SCOPE);
        init_var_dict(vimvardict.ptr(), vimvars_var.ptr(), VAR_SCOPE);
        (*vimvardict.ptr()).dv_lock = VAR_FIXED;
        hash_init(compat_hashtab.ptr());

        for i in 0..(*vimvars.ptr()).len() {
            let p = (vimvars.ptr() as *mut VimVar).add(i);
            // The key member is `VIMVAR_KEY_LEN + 1` bytes, which every name
            // in the table fits in.
            debug_assert!(strlen((*p).vv_name) <= 16);
            strcpy((&raw mut (*p).vv_di.di_key).cast(), (*p).vv_name);

            (*p).vv_di.di_flags = if (*p).vv_flags as c_int & VV_RO != 0 {
                DI_FLAGS_RO | DI_FLAGS_FIX
            } else if (*p).vv_flags as c_int & VV_RO_SBX != 0 {
                DI_FLAGS_RO_SBX | DI_FLAGS_FIX
            } else {
                DI_FLAGS_FIX
            };

            // Into the `v:` scope dictionary -- unless the value is not
            // always available, which is what a `VAR_UNKNOWN` row means.
            if (*p).vv_di.di_tv.v_type != VAR_UNKNOWN {
                hash_add(
                    &raw mut (*vimvardict.ptr()).dv_hashtab,
                    (&raw mut (*p).vv_di.di_key).cast(),
                );
            }
            if (*p).vv_flags as c_int & VV_COMPAT != 0 {
                // ... and into the scope that has no prefix at all.
                hash_add(compat_hashtab.ptr(), (&raw mut (*p).vv_di.di_key).cast());
            }
        }

        let vim_version = min_vim_version();
        set_vim_var_nr(VV_VERSION, vim_version as varnumber_T);
        set_vim_var_nr(
            VV_VERSIONLONG,
            (vim_version * 10000 + highest_patch()) as varnumber_T,
        );

        // `v:msgpack_types`: eight empty, locked lists, compared by identity
        // by the msgpack encoder and decoder rather than by name.
        let msgpack_types_dict = tv_dict_alloc();
        for (i, name) in msgpack_type_names.iter().enumerate() {
            let type_list = tv_list_alloc(0);
            tv_list_set_lock(type_list, VAR_FIXED);
            tv_list_ref(type_list);
            let di = tv_dict_item_alloc(name.as_ptr());
            (*di).di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX;
            (*di).di_tv = typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: type_list },
            };
            (*eval_msgpack_type_lists.ptr())[i] = type_list;
            if tv_dict_add(msgpack_types_dict, di) == FAIL {
                // The names are distinct by construction.
                abort();
            }
        }
        (*msgpack_types_dict).dv_lock = VAR_FIXED;
        set_vim_var_dict(VV_MSGPACK_TYPES, msgpack_types_dict);

        set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
        set_vim_var_dict(VV_EVENT, tv_dict_alloc_lock(VAR_FIXED));
        set_vim_var_list(VV_ERRORS, tv_list_alloc(kListLenUnknown as ptrdiff_t));
        set_vim_var_nr(VV_STDERR, CHAN_STDERR as varnumber_T);
        set_vim_var_nr(VV_SEARCHFORWARD, 1);
        set_vim_var_nr(VV_HLSEARCH, 1);
        set_vim_var_nr(VV_COUNT1, 1);
        set_vim_var_special(VV_EXITING, kSpecialVarNull);

        // The `v:t_*` type codes, which `type()` answers with.
        for (idx, code) in [
            (VV_TYPE_NUMBER, VAR_TYPE_NUMBER),
            (VV_TYPE_STRING, VAR_TYPE_STRING),
            (VV_TYPE_FUNC, VAR_TYPE_FUNC),
            (VV_TYPE_LIST, VAR_TYPE_LIST),
            (VV_TYPE_DICT, VAR_TYPE_DICT),
            (VV_TYPE_FLOAT, VAR_TYPE_FLOAT),
            (VV_TYPE_BOOL, VAR_TYPE_BOOL),
            (VV_TYPE_BLOB, VAR_TYPE_BLOB),
        ] {
            set_vim_var_nr(idx, code as varnumber_T);
        }

        set_vim_var_bool(VV_FALSE, kBoolVarFalse);
        set_vim_var_bool(VV_TRUE, kBoolVarTrue);
        set_vim_var_special(VV_NULL, kSpecialVarNull);
        set_vim_var_nr(VV_NUMBERMAX, VARNUMBER_MAX as varnumber_T);
        set_vim_var_nr(VV_NUMBERMIN, VARNUMBER_MIN as varnumber_T);
        set_vim_var_nr(
            VV_NUMBERSIZE,
            (::core::mem::size_of::<varnumber_T>() * 8) as varnumber_T,
        );
        set_vim_var_nr(VV_MAXCOL, MAXCOL as varnumber_T);
        set_vim_var_nr(VV_ECHOSPACE, (sc_col.get() - 1) as varnumber_T);

        let vvlua_partial = xcalloc(1, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
        // The name should never be printed, but do not crash if it is.
        (*vvlua_partial).pt_name = xmallocz(0) as *mut c_char;
        (*vvlua_partial).pt_refcount += 1;
        set_vim_var_partial(VV_LUA, vvlua_partial);

        // The default for v:register is not 0 but '"'.
        set_reg_var(0);
    }
}

/// Mark everything `g:` reaches as live, for the garbage collector.
///
/// # Safety
/// Called from the collector, with `copyID` its current mark.
pub unsafe fn garbage_collect_globvars(copyID: c_int) -> c_int {
    unsafe {
        set_ref_in_ht(
            &raw mut (*globvardict.ptr()).dv_hashtab,
            copyID,
            ptr::null_mut(),
        ) as c_int
    }
}

/// [`garbage_collect_globvars`] for `v:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_vimvars(copyID: c_int) -> bool {
    unsafe {
        set_ref_in_ht(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            copyID,
            ptr::null_mut(),
        )
    }
}

/// [`garbage_collect_globvars`] for every script's `s:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_scriptvars(copyID: c_int) -> bool {
    unsafe {
        let mut abort = false;
        for i in 1..=(*script_items.ptr()).ga_len {
            abort = abort
                || set_ref_in_ht(
                    &raw mut (*script_sv(i)).sv_dict.dv_hashtab,
                    copyID,
                    ptr::null_mut(),
                );
        }
        abort
    }
}

/// Set the variable `name` to the string `value`, taking ownership of it.
///
/// # Safety
/// `name` is a NUL-terminated string and `value` an owned one.
pub unsafe fn set_internal_string_var(name: *const c_char, value: *mut c_char) {
    unsafe {
        let mut tv = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: value },
        };
        set_var(name, strlen(name), &raw mut tv, true);
    }
}

/// Delete every `g:menutrans_*` variable, which `:menutranslate clear` does.
///
/// # Safety
/// Nothing.
pub unsafe fn del_menutrans_vars() {
    unsafe {
        let ht = &raw mut (*globvardict.ptr()).dv_hashtab;
        // The walk removes entries as it goes, so the table has to be locked
        // against the rehash that would otherwise move `ht_array`.
        hash_lock(ht);
        for hi in tv_ht_iter(&*ht) {
            if strncmp((*hi).hi_key, c"menutrans_".as_ptr(), 10) == 0 {
                delete_var(ht, hi);
            }
        }
        hash_unlock(ht);
    }
}

/// The `g:` scope, as a dictionary.
pub fn get_globvar_dict() -> *mut dict_T {
    globvardict.ptr()
}

/// The `g:` scope, as a hashtab.
pub fn get_globvar_ht() -> *mut hashtab_T {
    // SAFETY: a field of a `static`, never dereferenced here.
    unsafe { &raw mut (*globvardict.ptr()).dv_hashtab }
}

/// The `v:` scope, as a dictionary.
pub fn get_vimvar_dict() -> *mut dict_T {
    vimvardict.ptr()
}

/// Give script `id` its own `s:` scope.
///
/// # Safety
/// `id` is a live script id whose `sn_vars` has not been set.
pub unsafe fn new_script_vars(id: scid_T) {
    unsafe {
        let sv = xcalloc(1, ::core::mem::size_of::<scriptvar_T>()) as *mut scriptvar_T;
        init_var_dict(&raw mut (*sv).sv_dict, &raw mut (*sv).sv_var, VAR_SCOPE);
        (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
            .offset((id as c_int - 1) as isize))
        .sn_vars = sv;
    }
}

/// Make `dict` a scope dictionary and point `dict_var` at it.
///
/// A scope dictionary is never freed -- its reference count starts at
/// `DO_NOT_FREE_CNT` -- and the item that names it is read-only and fixed,
/// which is what makes `let g: = …` and `unlet g:` refuse.
///
/// # Safety
/// `dict` and `dict_var` are writable and not yet initialised.
pub unsafe fn init_var_dict(dict: *mut dict_T, dict_var: *mut ScopeDictDictItem, scope: ScopeType) {
    unsafe {
        hash_init(&raw mut (*dict).dv_hashtab);
        (*dict).dv_lock = VAR_UNLOCKED;
        (*dict).dv_scope = scope;
        (*dict).dv_refcount = DO_NOT_FREE_CNT;
        (*dict).dv_copyID = 0;
        (*dict_var).di_tv.vval.v_dict = dict;
        (*dict_var).di_tv.v_type = VAR_DICT;
        (*dict_var).di_tv.v_lock = VAR_FIXED;
        (*dict_var).di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
        (*dict_var).di_key[0] = NUL;
        QUEUE_INIT(&raw mut (*dict).watchers);
    }
}

/// Undo [`init_var_dict`]'s reference count, so that `dict` can be freed.
///
/// # Safety
/// `dict` came from [`init_var_dict`].
pub unsafe fn unref_var_dict(dict: *mut dict_T) {
    unsafe {
        // The reference count is what kept the scope alive; take it back to
        // the one reference the caller holds.
        (*dict).dv_refcount -= DO_NOT_FREE_CNT - 1;
        tv_dict_unref(dict);
    }
}

/// Free every variable in `ht`, and its values.
///
/// # Safety
/// `ht` is a live variable hashtab.
pub unsafe fn vars_clear(ht: *mut hashtab_T) {
    unsafe { vars_clear_ext(ht, true) }
}

/// [`vars_clear`], optionally leaving the values alone -- which is what a
/// function's local scope wants when its values have moved elsewhere.
///
/// # Safety
/// As [`vars_clear`].
pub unsafe fn vars_clear_ext(ht: *mut hashtab_T, free_val: bool) {
    unsafe {
        hash_lock(ht);
        for hi in tv_ht_iter(&*ht) {
            // Free the variable, unless it is one of the fixed ones embedded
            // in a `funccall_S` or a scope dictionary.
            let v = tv_dict_hi2di(hi);
            if free_val {
                tv_clear(&raw mut (*v).di_tv);
            }
            if (*v).di_flags & DI_FLAGS_ALLOC != 0 {
                xfree(v.cast());
            }
        }
        hash_clear(ht);
        hash_init(ht);
    }
}

/// Remove the variable `hi` names from `ht` and free it.
///
/// # Safety
/// `hi` is a live item of `ht`.
pub(crate) unsafe fn delete_var(ht: *mut hashtab_T, hi: *mut hashitem_T) {
    unsafe {
        let di = tv_dict_hi2di(hi);
        hash_remove(ht, hi);
        tv_clear(&raw mut (*di).di_tv);
        xfree(di.cast());
    }
}
