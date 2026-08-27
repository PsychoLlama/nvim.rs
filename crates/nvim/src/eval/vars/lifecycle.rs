//! Creating, clearing and collecting the variable dictionaries.
//!
//! [`evalvars_init`] builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::MessagePackType;
use crate::types::{FAIL, NUL, Refcount};

/// Build the `g:` and `v:` scopes and fill the `v:` table.  Called once, at
/// startup.
///
/// # Safety
/// Called once, before anything reads a variable.
pub unsafe fn evalvars_init() {
    unsafe {
        init_var_dict(get_globvar_dict(), globvar_scope_item(), VAR_DEF_SCOPE);
        init_var_dict(get_vimvar_dict(), vimvar_scope_item(), VAR_SCOPE);
        (*get_vimvar_dict()).dv_lock = VarLock::Fixed;
        hash_init(get_compat_ht());

        for i in 0..VIMVAR_COUNT {
            let p = vimvar_table().add(i);
            // The key member is `VIMVAR_KEY_LEN + 1` bytes, which every name
            // in the table fits in.
            debug_assert!(strlen((*p).vv_name) <= 16);
            strcpy((&raw mut (*p).vv_di.di_key).cast(), (*p).vv_name);

            let flags = VimVarFlags::from_bits((*p).vv_flags as c_int);
            (*p).vv_di.di_flags = if flags.has(VimVarFlags::RO) {
                DI_FLAGS_RO | DI_FLAGS_FIX
            } else if flags.has(VimVarFlags::RO_SBX) {
                DI_FLAGS_RO_SBX | DI_FLAGS_FIX
            } else {
                DI_FLAGS_FIX
            };

            // Into the `v:` scope dictionary -- unless the value is not
            // always available, which is what a `VAR_UNKNOWN` row means.
            if (*p).vv_di.di_tv.v_type != VAR_UNKNOWN {
                hash_add(get_vimvar_ht(), (&raw mut (*p).vv_di.di_key).cast());
            }
            if flags.has(VimVarFlags::COMPAT) {
                // ... and into the scope that has no prefix at all.
                hash_add(get_compat_ht(), (&raw mut (*p).vv_di.di_key).cast());
            }
        }

        let vim_version = min_vim_version();
        set_vim_var_nr(Vv::Version, vim_version as varnumber_T);
        set_vim_var_nr(
            Vv::Versionlong,
            (vim_version * 10000 + highest_patch()) as varnumber_T,
        );

        // `v:msgpack_types`: eight empty, locked lists, compared by identity
        // by the msgpack encoder and decoder rather than by name.
        let msgpack_types_dict = tv_dict_alloc();
        let mut type_lists = eval_msgpack_type_lists.get();
        for (i, name) in msgpack_type_names.iter().enumerate() {
            let type_list = tv_list_alloc(0);
            tv_list_set_lock(type_list, VarLock::Fixed);
            tv_list_ref(type_list);
            let di = tv_dict_item_alloc(name.as_ptr());
            (*di).di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX;
            (*di).di_tv = typval_T {
                v_type: VAR_LIST,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union { v_list: type_list },
            };
            type_lists[i] = type_list;
            if tv_dict_add(msgpack_types_dict, di) == FAIL {
                // The names are distinct by construction.
                abort();
            }
        }
        eval_msgpack_type_lists.set(type_lists);
        (*msgpack_types_dict).dv_lock = VarLock::Fixed;
        set_vim_var_dict(Vv::MsgpackTypes, msgpack_types_dict);

        set_vim_var_dict(Vv::CompletedItem, tv_dict_alloc_lock(VarLock::Fixed));
        set_vim_var_dict(Vv::Event, tv_dict_alloc_lock(VarLock::Fixed));
        set_vim_var_list(Vv::Errors, tv_list_alloc(kListLenUnknown as ptrdiff_t));
        set_vim_var_nr(Vv::Stderr, CHAN_STDERR as varnumber_T);
        set_vim_var_nr(Vv::Searchforward, 1);
        set_vim_var_nr(Vv::Hlsearch, 1);
        set_vim_var_nr(Vv::Count1, 1);
        set_vim_var_special(Vv::Exiting, kSpecialVarNull);

        // The `v:t_*` type codes, which `type()` answers with.
        for (idx, code) in [
            (Vv::TNumber, VAR_TYPE_NUMBER),
            (Vv::TString, VAR_TYPE_STRING),
            (Vv::TFunc, VAR_TYPE_FUNC),
            (Vv::TList, VAR_TYPE_LIST),
            (Vv::TDict, VAR_TYPE_DICT),
            (Vv::TFloat, VAR_TYPE_FLOAT),
            (Vv::TBool, VAR_TYPE_BOOL),
            (Vv::TBlob, VAR_TYPE_BLOB),
        ] {
            set_vim_var_nr(idx, code as varnumber_T);
        }

        set_vim_var_bool(Vv::False, kBoolVarFalse);
        set_vim_var_bool(Vv::True, kBoolVarTrue);
        set_vim_var_special(Vv::Null, kSpecialVarNull);
        set_vim_var_nr(Vv::Numbermax, VARNUMBER_MAX as varnumber_T);
        set_vim_var_nr(Vv::Numbermin, VARNUMBER_MIN as varnumber_T);
        set_vim_var_nr(
            Vv::Numbersize,
            (::core::mem::size_of::<varnumber_T>() * 8) as varnumber_T,
        );
        set_vim_var_nr(Vv::Maxcol, MAXCOL as varnumber_T);
        set_vim_var_nr(Vv::Echospace, (sc_col.get() - 1) as varnumber_T);

        let vvlua_partial = xcalloc(1, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
        // The name should never be printed, but do not crash if it is.
        (*vvlua_partial).pt_name = xmallocz(0) as *mut c_char;
        (*vvlua_partial).pt_refcount.retain();
        set_vim_var_partial(Vv::Lua, vvlua_partial);

        // The default for v:register is not 0 but '"'.
        set_reg_var(0);
    }
}

/// Mark everything `g:` reaches as live, for the garbage collector.
///
/// # Safety
/// Called from the collector, with `copyID` its current mark.
pub unsafe fn garbage_collect_globvars(copyID: c_int) -> c_int {
    unsafe { set_ref_in_ht(get_globvar_ht(), copyID, ptr::null_mut()) as c_int }
}

/// [`garbage_collect_globvars`] for `v:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_vimvars(copyID: c_int) -> bool {
    unsafe { set_ref_in_ht(get_vimvar_ht(), copyID, ptr::null_mut()) }
}

/// [`garbage_collect_globvars`] for every script's `s:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_scriptvars(copyID: c_int) -> bool {
    unsafe {
        let mut abort = false;
        for i in 1..=script_count() {
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
            v_lock: VarLock::Unlocked,
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
        let ht = get_globvar_ht();
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
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_globvar_dict()).dv_hashtab }
}

/// The `v:` scope, as a dictionary.
pub fn get_vimvar_dict() -> *mut dict_T {
    vimvardict.ptr()
}

/// The `v:` scope, as a hashtab.
pub(crate) fn get_vimvar_ht() -> *mut hashtab_T {
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_vimvar_dict()).dv_hashtab }
}

/// The `v:` variable table, whose rows are the `Vv` discriminants in order.
pub(crate) fn vimvar_table() -> *mut VimVar {
    vimvars.ptr().cast()
}

/// The scope that has no prefix at all: the names that mean `v:version`
/// wherever they are written. Upstream's `compat_hashtab`.
pub(crate) fn get_compat_ht() -> *mut hashtab_T {
    compat_hashtab.ptr()
}

/// The `dictitem_T` a bare `g:` resolves to.
pub(crate) fn globvar_scope_item() -> *mut ScopeDictDictItem {
    globvars_var.ptr()
}

/// The `dictitem_T` a bare `v:` resolves to.
pub(crate) fn vimvar_scope_item() -> *mut ScopeDictDictItem {
    vimvars_var.ptr()
}

/// The `v:msgpack_types` list for `type_`, compared by identity by the
/// msgpack encoder and decoder.
pub(crate) fn msgpack_type_list(type_: MessagePackType) -> *mut list_T {
    eval_msgpack_type_lists.get()[type_ as usize].cast_mut()
}

/// Give script `id` its own `s:` scope.
///
/// # Safety
/// `id` is a live script id whose `sn_vars` has not been set.
pub unsafe fn new_script_vars(id: scid_T) {
    unsafe {
        let sv = xcalloc(1, ::core::mem::size_of::<scriptvar_T>()) as *mut scriptvar_T;
        init_var_dict(&raw mut (*sv).sv_dict, &raw mut (*sv).sv_var, VAR_SCOPE);
        (*script_item(id)).sn_vars = sv;
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
        (*dict).dv_lock = VarLock::Unlocked;
        (*dict).dv_scope = scope;
        (*dict).dv_refcount = Refcount::new(DO_NOT_FREE_CNT);
        (*dict).dv_copyID = 0;
        (*dict_var).di_tv.vval.v_dict = dict;
        (*dict_var).di_tv.v_type = VAR_DICT;
        (*dict_var).di_tv.v_lock = VarLock::Fixed;
        (*dict_var).di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
        (*dict_var).di_key[0] = NUL as c_char;
        queue_init(&raw mut (*dict).watchers);
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
        (*dict).dv_refcount.release_many(DO_NOT_FREE_CNT - 1);
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
