//! Creating, clearing and collecting the variable dictionaries.
//!
//! [`evalvars_init`] builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{c_char, c_int};
use core::mem::{ManuallyDrop, offset_of};
use core::ptr;

use super::*;
use crate::eval::typval::{DictEntry, DictRef, DictTab, ListRef, tv_dict_item_free};
use crate::types::MessagePackType;
use crate::types::{DictKey, Refcount};

/// Build the `g:` and `v:` scopes and fill the `v:` table.  Called once, at
/// startup.
///
/// # Safety
/// Called once, before anything reads a variable.
pub unsafe fn evalvars_init() {
    unsafe { init_var_dict(get_globvar_dict(), globvar_scope_item(), VAR_DEF_SCOPE) };
    unsafe { init_var_dict(get_vimvar_dict(), vimvar_scope_item(), VAR_SCOPE) };
    unsafe { (*get_vimvar_dict()).dv_lock = VarLock::Fixed };
    unsafe { hash_init(get_compat_ht()) };

    for i in 0..VIMVAR_COUNT {
        let mut row = vimvar_row(i);
        let flags = VimVarFlags::from_bits(row.vv_flags as c_int);
        row.vv_di.di_flags = if flags.has(VimVarFlags::RO) {
            DI_FLAGS_RO | DI_FLAGS_FIX
        } else if flags.has(VimVarFlags::RO_SBX) {
            DI_FLAGS_RO_SBX | DI_FLAGS_FIX
        } else {
            DI_FLAGS_FIX
        };
        let (name, declared) = (row.vv_name, row.vv_di.di_tv.v_type());

        // The item's address is taken *after* the last field access, and
        // nothing touches the row again: `vv_di` is a member of `VimVar`,
        // so a borrow of the whole row -- which `Live`'s `Deref` hands out
        // -- would invalidate the pointer the hashtab is about to keep.
        let item = vimvar_row_item(row);
        // Every `v:` name is short enough to live in the item.
        // SAFETY: the row's name is a NUL-terminated literal, and the item
        // is the row's own.
        let name_bytes = unsafe { cstr::bytes_at(name) };
        debug_assert!(name_bytes.len() <= DictKey::INLINE_MAX);
        unsafe { (*item).di_key = DictKey::new(name_bytes) };

        // Into the `v:` scope dictionary -- unless the value is not
        // always available, which is what a `VAR_UNKNOWN` row means.
        // SAFETY: the two scope hashtabs, and the row's own item.
        if declared != VAR_UNKNOWN {
            let _ = unsafe { hash_add(get_vimvar_ht(), DictEntry::new(item)) };
        }
        if flags.has(VimVarFlags::COMPAT) {
            // ... and into the scope that has no prefix at all.
            let _ = unsafe { hash_add(get_compat_ht(), DictEntry::new(item)) };
        }
    }

    let vim_version = min_vim_version();
    let versionlong = (vim_version * 10000 + highest_patch()) as VarNumber;
    set_vim_var_nr(Vv::Version, vim_version as VarNumber);
    set_vim_var_nr(Vv::Versionlong, versionlong);

    // `v:msgpack_types`: eight empty, locked lists, compared by identity
    // by the msgpack encoder and decoder rather than by name.
    let msgpack_types_dict_held = tv_dict_alloc();
    let msgpack_types_dict = msgpack_types_dict_held.as_ptr();
    let mut type_lists = eval_msgpack_type_lists.get();
    for (i, name) in msgpack_type_names.iter().enumerate() {
        let type_list = tv_list_alloc(0);
        let at = type_list.as_ptr();
        unsafe { tv_list_set_lock(at, VarLock::Fixed) };
        let di = unsafe { tv_dict_item_alloc(name.as_ptr()) };
        // SAFETY: the item just allocated.
        let mut item = unsafe { Di::new(di) };
        item.di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX;
        item.di_tv.write_list(Some(type_list));
        // The encoder and decoder compare these by *identity*, so the table
        // keeps a pointer of its own -- and a reference that is never given
        // back, since `v:msgpack_types` lives as long as the process.
        let kept = unsafe { ListRef::retained(at) };
        type_lists[i] = kept.expect("the list just allocated").into_raw();
        if unsafe { tv_dict_add(msgpack_types_dict, di) }.is_err() {
            // The names are distinct by construction.
            unsafe { abort() };
        }
    }
    eval_msgpack_type_lists.set(type_lists);
    unsafe { (*msgpack_types_dict).dv_lock = VarLock::Fixed };
    unsafe { set_vim_var_dict(Vv::MsgpackTypes, Some(msgpack_types_dict_held)) };

    // SAFETY: `Vv` names a row of the table, and each value below is a live
    // container this hands its reference to.
    unsafe { set_vim_var_dict(Vv::CompletedItem, Some(tv_dict_alloc_lock(VarLock::Fixed))) };
    unsafe { set_vim_var_dict(Vv::Event, Some(tv_dict_alloc_lock(VarLock::Fixed))) };
    let errors = Some(tv_list_alloc(kListLenUnknown as ptrdiff_t));
    unsafe { set_vim_var_list(Vv::Errors, errors) };

    // The `v:` variables that start out at a constant Number, the `v:t_*`
    // type codes `type()` answers with among them. Nothing here reads
    // another row, so the table's order is the source's only.
    let numbersize = (::core::mem::size_of::<VarNumber>() * 8) as VarNumber;
    for (idx, n) in [
        (Vv::Stderr, CHAN_STDERR as VarNumber),
        (Vv::Searchforward, 1),
        (Vv::Hlsearch, 1),
        (Vv::Count1, 1),
        (Vv::TNumber, VAR_TYPE_NUMBER as VarNumber),
        (Vv::TString, VAR_TYPE_STRING as VarNumber),
        (Vv::TFunc, VAR_TYPE_FUNC as VarNumber),
        (Vv::TList, VAR_TYPE_LIST as VarNumber),
        (Vv::TDict, VAR_TYPE_DICT as VarNumber),
        (Vv::TFloat, VAR_TYPE_FLOAT as VarNumber),
        (Vv::TBool, VAR_TYPE_BOOL as VarNumber),
        (Vv::TBlob, VAR_TYPE_BLOB as VarNumber),
        (Vv::Numbermax, VARNUMBER_MAX as VarNumber),
        (Vv::Numbermin, VARNUMBER_MIN as VarNumber),
        (Vv::Numbersize, numbersize),
        (Vv::Maxcol, MAXCOL as VarNumber),
        (Vv::Echospace, (sc_col.get() - 1) as VarNumber),
    ] {
        set_vim_var_nr(idx, n);
    }

    set_vim_var_special(Vv::Exiting, kSpecialVarNull);
    set_vim_var_bool(Vv::False, kBoolVarFalse);
    set_vim_var_bool(Vv::True, kBoolVarTrue);
    set_vim_var_special(Vv::Null, kSpecialVarNull);

    let vvlua_partial = unsafe { xcalloc(1, ::core::mem::size_of::<Partial>()) } as *mut Partial;
    // The name should never be printed, but do not crash if it is.
    unsafe { (*vvlua_partial).pt_name = xmallocz(0) as *mut c_char };
    // SAFETY: the partial just allocated. The region covers the *call*:
    // `unsafe { … }.retain()` would bump a copy of the count.
    unsafe { (*vvlua_partial).pt_refcount.retain() };
    unsafe { set_vim_var_partial(Vv::Lua, vvlua_partial) };

    // The default for v:register is not 0 but '"'.
    unsafe { set_reg_var(0) };
}

/// Mark everything `g:` reaches as live, for the garbage collector.
///
/// # Safety
/// Called from the collector, with `copy_id` its current mark.
pub unsafe fn garbage_collect_globvars(copy_id: c_int) -> c_int {
    unsafe { set_ref_in_ht(get_globvar_ht(), copy_id, ptr::null_mut()) as c_int }
}

/// [`garbage_collect_globvars`] for `v:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_vimvars(copy_id: c_int) -> bool {
    unsafe { set_ref_in_ht(get_vimvar_ht(), copy_id, ptr::null_mut()) }
}

/// [`garbage_collect_globvars`] for every script's `s:`.
///
/// # Safety
/// As [`garbage_collect_globvars`].
pub unsafe fn garbage_collect_scriptvars(copy_id: c_int) -> bool {
    let mut abort = false;
    for i in 1..=script_count() {
        // SAFETY: a live script id, whose own scope dictionary this marks.
        let ht = unsafe { &raw mut (*script_sv(i)).sv_dict.dv_hashtab };
        abort = abort || unsafe { set_ref_in_ht(ht, copy_id, ptr::null_mut()) };
    }
    abort
}

/// Set the variable `name` to the string `value`, taking ownership of it.
///
/// # Safety
/// `name` and `value` are NUL-terminated strings.  `value` stays the
/// caller's: the store copies it.
pub unsafe fn set_internal_string_var(name: *const c_char, value: *mut c_char) {
    let mut tv = ManuallyDrop::new(TypVal::String(value));
    unsafe { set_var(name, cstr::bytes_at(name).len(), &mut tv, true) };
}

/// Delete every `g:menutrans_*` variable, which `:menutranslate clear` does.
///
/// # Safety
/// Nothing.
pub unsafe fn del_menutrans_vars() {
    let ht = get_globvar_ht();
    // The walk removes entries as it goes, so the table has to be locked
    // against the rehash that would otherwise move the slot array.
    unsafe { hash_lock(ht) };
    for hi in unsafe { tv_ht_iter(ht) } {
        if unsafe { cstr::starts_with((*hi.hi_key.item()).di_key.as_ptr(), b"menutrans_") } {
            unsafe { delete_var(ht, hi) };
        }
    }
    unsafe { hash_unlock(ht) };
}

/// The `g:` scope, as a dictionary.
pub fn get_globvar_dict() -> *mut Dict {
    globvardict.ptr()
}

/// The `g:` scope, as a hashtab.
pub fn get_globvar_ht() -> *mut DictTab {
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_globvar_dict()).dv_hashtab }
}

/// The `v:` scope, as a dictionary.
pub fn get_vimvar_dict() -> *mut Dict {
    vimvardict.ptr()
}

/// The `v:` scope, as a hashtab.
pub(crate) fn get_vimvar_ht() -> *mut DictTab {
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_vimvar_dict()).dv_hashtab }
}

/// The `v:` variable table, whose rows are the `Vv` discriminants in order.
pub(crate) fn vimvar_table() -> *mut VimVar {
    vimvars.ptr().cast()
}

/// The scope that has no prefix at all: the names that mean `v:version`
/// wherever they are written. Upstream's `compat_hashtab`.
pub(crate) fn get_compat_ht() -> *mut DictTab {
    compat_hashtab.ptr()
}

/// The `DictItem` a bare `g:` resolves to.
pub(crate) fn globvar_scope_item() -> *mut ScopeDictItem {
    globvars_var.ptr()
}

/// The `DictItem` a bare `v:` resolves to.
pub(crate) fn vimvar_scope_item() -> *mut ScopeDictItem {
    vimvars_var.ptr()
}

/// The `v:msgpack_types` list for `type_`, compared by identity by the
/// msgpack encoder and decoder.
pub(crate) fn msgpack_type_list(type_: MessagePackType) -> *mut List {
    eval_msgpack_type_lists.get()[type_ as usize].cast_mut()
}

/// Give script `id` its own `s:` scope.
///
/// # Safety
/// `id` is a live script id whose `sn_vars` has not been set.
pub unsafe fn new_script_vars(id: ScriptId) {
    let sv = unsafe { xcalloc(1, ::core::mem::size_of::<ScriptVar>()) } as *mut ScriptVar;
    unsafe { init_var_dict(&raw mut (*sv).sv_dict, &raw mut (*sv).sv_var, VAR_SCOPE) };
    unsafe { (*script_item(id)).sn_vars = sv };
}

/// Make `dict` a scope dictionary and point `dict_var` at it.
///
/// A scope dictionary is never freed -- its reference count starts at
/// `DO_NOT_FREE_CNT` -- and the item that names it is read-only and fixed,
/// which is what makes `let g: = …` and `unlet g:` refuse.
///
/// # Safety
/// `dict` and `dict_var` are writable and not yet initialised.
pub unsafe fn init_var_dict(dict: *mut Dict, dict_var: *mut ScopeDictItem, scope: ScopeType) {
    // SAFETY: the caller's obligation -- both are writable and outlive the
    // call; the hashtab and the watcher queue are fields of the dictionary
    // itself, so initialising them in place is what the C does.
    let (mut d, mut var) = unsafe { (Live::new(dict), Live::new(dict_var)) };
    d.dv_lock = VarLock::Unlocked;
    d.dv_scope = scope;
    d.dv_refcount = Refcount::new(DO_NOT_FREE_CNT);
    d.dv_copy_id = 0;
    // The scope variable **names** the dictionary its own storage owns:
    // `DO_NOT_FREE_CNT` above is what keeps anything from freeing it, and
    // `unref_var_dict` gives the whole block back.
    // SAFETY: the caller's dictionary, live for as long as the variable is.
    var.di_tv.write_dict(unsafe { DictRef::owning(dict) });
    var.di_lock = VarLock::Fixed;
    var.di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    var.di_key = DictKey::EMPTY;
    // The watcher queue's head points at its own node, so `queue_init` goes
    // last: a borrow of the whole `Dict` afterwards would invalidate the
    // pointer it has just stored. The hash table has no such constraint any
    // more -- it owns its slots -- but `hash_init` writes over storage that
    // must not already hold a table, which is what a fresh `Dict` is.
    unsafe { hash_init(&raw mut (*dict).dv_hashtab) };
    unsafe { queue_init(&raw mut (*dict).watchers) };
}

/// Undo [`init_var_dict`]'s reference count, so that `dict` can be freed.
///
/// # Safety
/// `dict` came from [`init_var_dict`].
pub unsafe fn unref_var_dict(dict: *mut Dict) {
    // The reference count is what kept the scope alive; take it back to
    // the one reference the caller holds.
    // SAFETY: the caller's obligation -- a dictionary `init_var_dict`
    // built. The region covers the call, not just the dereference.
    unsafe { (*dict).dv_refcount.release_many(DO_NOT_FREE_CNT - 1) };
    unsafe { tv_dict_unref(dict) };
}

/// Free every variable in `ht`, and its values.
///
/// # Safety
/// `ht` is a live variable hashtab.
pub unsafe fn vars_clear(ht: *mut DictTab) {
    unsafe { vars_clear_ext(ht, true) }
}

/// [`vars_clear`], optionally leaving the values alone -- which is what a
/// function's local scope wants when its values have moved elsewhere.
///
/// # Safety
/// As [`vars_clear`].
pub unsafe fn vars_clear_ext(ht: *mut DictTab, free_val: bool) {
    // SAFETY: the caller's obligation -- a live variable hashtab, whose items
    // are the `DictItem`s the walk frees.
    unsafe { hash_lock(ht) };
    for hi in unsafe { tv_ht_iter(ht) } {
        // Free the variable, unless it is one of the fixed ones embedded
        // in a `FuncCall` or a scope dictionary.
        let v = unsafe { Di::new(tv_dict_hi2di(hi)) };
        let tv = v.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
        if free_val {
            unsafe { tv_clear(&mut *tv) };
        } else {
            // The values have moved elsewhere -- an `a:` item names the
            // caller's argument -- so the item must not take them with it.
            unsafe { (*tv).disown() };
        }
        if v.di_flags & DI_FLAGS_ALLOC != 0 {
            // The item owns its key, so the whole item goes at once.
            drop(unsafe { Box::from_raw(v.raw()) });
        }
    }
    // SAFETY: the caller's table, whose items have all been freed.
    hash_reset(unsafe { &mut *ht });
}

/// Remove the variable `hi` names from `ht` and free it.
///
/// # Safety
/// `hi` is a live item of `ht`.
pub(crate) unsafe fn delete_var(ht: *mut DictTab, hi: Slot<DictEntry>) {
    // SAFETY: the caller's obligation -- a live item of `ht`, which this
    // takes out of the table and then frees.
    let di = tv_dict_hi2di(hi);
    unsafe { hash_remove(ht, hi) };
    unsafe { tv_dict_item_free(di) };
}
