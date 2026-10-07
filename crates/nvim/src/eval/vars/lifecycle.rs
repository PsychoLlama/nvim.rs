//! Creating, clearing and collecting the variable dictionaries.
//!
//! [`evalvars_init`] builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::memory::ThinCString;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::eval::typval::{DictEntry, DictRef, DictTab, ListRef, tv_dict_item_free};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::MessagePackType;
use crate::types::{DictKey, PartialRef, Refcount};
use crate::types::{Failed, NUL};

/// Build the `g:` and `v:` scopes and give the `v:` variables their first
/// values.  Called once, at startup.
pub fn evalvars_init() {
    drop(globvar_dict());
    drop(vimvar_dict());
    debug_assert!(
        VIMVAR_ROWS
            .iter()
            .filter(|row| row.flags.has(VimVarFlags::COMPAT))
            .all(|row| is_compat_name(row.name.to_bytes())),
        "a COMPAT row is spelled in is_compat_name"
    );

    let vim_version = min_vim_version();
    let versionlong = (vim_version * 10000 + highest_patch()) as VarNumber;
    set_vim_var_nr(Vv::Version, vim_version as VarNumber);
    set_vim_var_nr(Vv::Versionlong, versionlong);

    // `v:msgpack_types`: eight empty, locked lists, compared by identity
    // by the msgpack encoder and decoder rather than by name.
    let msgpack_types_dict = tv_dict_alloc();
    for (i, name) in msgpack_type_names.iter().enumerate() {
        let type_list = tv_list_alloc(0);
        list_set_lock(Some(type_list.edit()), VarLock::Fixed);
        // The encoder and decoder compare these by *identity*, so the
        // record keeps a reference of its own -- never given back, since
        // `v:msgpack_types` lives as long as the process.
        let kept = type_list.clone();
        msgpack_type_lists.with_mut(|lists| lists[i] = Some(kept));
        let mut item = DictItem::boxed(name.to_bytes());
        item.di_flags |= DI_FLAGS_RO | DI_FLAGS_FIX;
        item.di_tv.write_list(Some(type_list));
        if msgpack_types_dict.edit().add_item(item).is_err() {
            unreachable!("the msgpack type names are distinct");
        }
    }
    msgpack_types_dict.edit().dv_lock = VarLock::Fixed;
    set_vim_var_dict(Vv::MsgpackTypes, Some(msgpack_types_dict));

    set_vim_var_dict(Vv::CompletedItem, Some(tv_dict_alloc_lock(VarLock::Fixed)));
    set_vim_var_dict(Vv::Event, Some(tv_dict_alloc_lock(VarLock::Fixed)));
    let errors = Some(tv_list_alloc(kListLenUnknown as ptrdiff_t));
    set_vim_var_list(Vv::Errors, errors);

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

    // The name should never be printed, but do not crash if it is.
    let lua = Partial {
        pt_name: Some(ThinCString::empty()),
        ..Partial::EMPTY
    };
    set_vim_var_partial(Vv::Lua, PartialRef::new(lua));

    // The default for v:register is not 0 but '"'.
    set_reg_var(0);
}

/// Whether the unprefixed `name` is a `v:` variable readable without its
/// prefix -- a `VimVarFlags::COMPAT` row. Upstream's `compat_hashtab`, which
/// only ever held `version`.
#[inline]
pub(crate) fn is_compat_name(name: &[u8]) -> bool {
    name == b"version"
}

/// Mark everything `g:` reaches as live, for the garbage collector.
pub fn garbage_collect_globvars(copy_id: c_int) -> c_int {
    c_int::from(set_ref_in_dict_items(&globvar_dict(), copy_id, None))
}

/// [`garbage_collect_globvars`] for `v:`.
pub fn garbage_collect_vimvars(copy_id: c_int) -> bool {
    set_ref_in_dict_items(&vimvar_dict(), copy_id, None)
}

/// [`garbage_collect_globvars`] for every script's `s:`.
pub fn garbage_collect_scriptvars(copy_id: c_int) -> bool {
    let mut abort = false;
    for i in 1..=script_count() {
        if let Some(dict) = script_scope_dict(i) {
            abort = abort || set_ref_in_dict_items(&dict, copy_id, None);
        }
    }
    abort
}

/// [`set_internal_string_var`] for a name and value the caller holds.
pub(crate) fn set_internal_string_var_to(name: &CStr, value: &CStr) {
    // SAFETY: two NUL-terminated strings; the store copies the value.
    unsafe { set_internal_string_var(name.as_ptr(), value.as_ptr().cast_mut()) };
}

/// Set the variable `name` to the string `value`, taking ownership of it.
///
/// # Safety
/// `name` and `value` are NUL-terminated strings.  `value` stays the
/// caller's: the store copies it.
pub unsafe fn set_internal_string_var(name: *const c_char, value: *mut c_char) {
    let value = (!value.is_null()).then(|| {
        // SAFETY: the caller's promise: a NUL-terminated `value`.
        ThinCString::from_cstr(unsafe { CStr::from_ptr(value) })
    });
    // The value holds a copy, which the store copies again and this frame
    // releases.
    let mut tv = TypVal::string(value);
    unsafe { set_var(name, cstr::bytes_at(name).len(), &mut tv, true) };
}

/// Delete every `g:menutrans_*` variable, which `:menutranslate clear` does.
pub fn del_menutrans_vars() {
    let dict = globvar_dict();
    // The walk removes entries as it goes, so the table has to be locked
    // against the rehash that would otherwise move the slot array.
    dict.edit().lock_table();
    let mut cursor = DictCursor::new(&dict);
    while let Some(slot) = cursor.next(&dict) {
        let doomed = dict
            .item_at(slot)
            .is_some_and(|item| item.key().starts_with(b"menutrans_"));
        if doomed {
            let removed = dict.edit().remove_at(slot);
            drop(removed);
        }
    }
    dict.edit().unlock_table();
}

/// The `g:` dictionary, built on first use.
pub(crate) fn globvar_dict() -> DictRef {
    match scope_globals.with(|entry| entry.di_tv.dict_handle()) {
        Some(dict) => dict,
        None => {
            let entry = new_unrooted_var_scope(VAR_DEF_SCOPE);
            let dict = entry
                .di_tv
                .dict_handle()
                .expect("a scope entry names its dictionary");
            scope_globals.set(entry);
            dict
        }
    }
}

/// The `g:` scope, as a dictionary.
pub(crate) fn get_globvar_dict() -> *mut Dict {
    // `g:` is never freed, so its address outlives the handle.
    globvar_dict().as_ptr()
}

/// The `g:` scope, as a hashtab.
pub(crate) fn get_globvar_ht() -> *mut DictTab {
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_globvar_dict()).dv_hashtab }
}

/// The `v:` scope, as a dictionary.
pub(crate) fn get_vimvar_dict() -> *mut Dict {
    // `v:` is never freed, so its address outlives the handle.
    vimvar_dict().as_ptr()
}

/// The `v:` scope, as a hashtab.
pub(crate) fn get_vimvar_ht() -> *mut DictTab {
    // SAFETY: a field of the dictionary above, never dereferenced here.
    unsafe { &raw mut (*get_vimvar_dict()).dv_hashtab }
}

/// The `DictItem` a bare `g:` resolves to.
pub(crate) fn globvar_scope_item() -> *mut DictItem {
    drop(globvar_dict());
    scope_globals.with_mut(ScopeDictItem::item)
}

/// The `DictItem` a bare `v:` resolves to.
pub(crate) fn vimvar_scope_item() -> *mut DictItem {
    drop(vimvar_dict());
    scope_vim.with_mut(ScopeDictItem::item)
}

/// Whether `list` is the `v:msgpack_types` list for `type_`.
pub(crate) fn msgpack_type_list_is(type_: MessagePackType, list: &List) -> bool {
    msgpack_type_lists.with(|lists| {
        lists[type_ as usize]
            .as_ref()
            .is_some_and(|l| ::core::ptr::eq(l.as_ptr().cast_const(), list))
    })
}

/// Which `v:msgpack_types` list `list` is, as an index in `MessagePackType`
/// order.
pub(crate) fn msgpack_type_of(list: &List) -> Option<usize> {
    msgpack_type_lists.with(|lists| {
        lists.iter().position(|l| {
            l.as_ref()
                .is_some_and(|l| ::core::ptr::eq(l.as_ptr().cast_const(), list))
        })
    })
}

/// The `v:msgpack_types` list for `type_`, as a reference of the caller's
/// own, for a special dictionary's `_TYPE`.
pub(crate) fn msgpack_type_list_ref(type_: MessagePackType) -> Option<ListRef> {
    msgpack_type_lists.with(|lists| lists[type_ as usize].clone())
}

/// Give script `id` its own `s:` scope.
pub fn new_script_vars(id: ScriptId) {
    let vars = Box::new(ScriptVar {
        sv_var: new_unrooted_var_scope(VAR_SCOPE),
    });
    with_script_item(id, |si| si.sn_vars = Some(vars));
}

/// The `s:` dictionary of script `sid`, if it has one.
pub(crate) fn script_scope_dict(sid: ScriptId) -> Option<DictRef> {
    if !script_id_valid(sid) {
        return None;
    }
    with_script_item(sid, |si| {
        si.sn_vars
            .as_ref()
            .and_then(|vars| vars.sv_var.di_tv.dict_handle())
    })
}

/// The entry a scope dictionary is reached through: read-only and fixed,
/// which is what makes `let g: = …` and `unlet g:` refuse, and holding the
/// dictionary, whose reference count is seeded with `DO_NOT_FREE_CNT` so
/// that nothing frees it until [`release_var_scope`].
fn scope_entry(dict: DictRef, scope: ScopeType) -> ScopeDictItem {
    {
        let d = dict.edit();
        d.dv_scope = scope;
        d.dv_refcount = Refcount::new(DO_NOT_FREE_CNT);
    }
    ScopeDictItem(ManuallyDrop::new(DictItem {
        di_tv: TypVal::dict(Some(dict)),
        di_lock: VarLock::Fixed,
        di_flags: DI_FLAGS_RO | DI_FLAGS_FIX,
        di_key: DictKey::EMPTY,
    }))
}

/// A fresh `b:`, `w:` or `t:` scope: its dictionary stays in the
/// collector's registry, and is marked through the entry its buffer, window
/// or tab page holds.
pub fn new_var_scope(scope: ScopeType) -> ScopeDictItem {
    scope_entry(tv_dict_alloc(), scope)
}

/// A fresh `g:`, `v:` or `s:` scope: its dictionary is out of the
/// collector's registry, which only marks its items.
pub(crate) fn new_unrooted_var_scope(scope: ScopeType) -> ScopeDictItem {
    let dict = tv_dict_alloc();
    {
        let d = dict.edit();
        unroot_dict(d.dv_root);
        d.dv_root = RootId::NONE;
    }
    scope_entry(dict, scope)
}

/// Undo [`new_var_scope`]'s reference count and give the entry's reference
/// back, so that the dictionary is freed when nothing else holds it.
pub fn release_var_scope(entry: &mut ScopeDictItem) {
    if let Some(dict) = entry.di_tv.dict_shared() {
        dict.edit().dv_refcount.release_many(DO_NOT_FREE_CNT - 1);
    }
    drop(entry.0.di_tv.take());
}

/// Free every variable in `dict`, and its values, leaving it with a fresh
/// empty table.
pub fn vars_clear(dict: &DictRef) {
    dict.edit().lock_table();
    let mut cursor = DictCursor::new(dict);
    while let Some(slot) = cursor.next(dict) {
        // Released with the borrow of the dictionary over: a value can name
        // the dictionary it was in.
        let removed = dict.edit().remove_at(slot);
        drop(removed);
    }
    hash_reset(&mut dict.edit().dv_hashtab);
}

/// Delete the variable `name`, reporting E108 if it does not exist and
/// `forceit` is not set.
pub(crate) fn do_unlet(name: &[u8], forceit: bool) -> Result<(), Failed> {
    // SAFETY: `name` is NUL-terminated at its own length.
    cstr::with_terminated(name, |name| unsafe {
        unlet_terminated(name.as_ptr(), name.count_bytes(), forceit)
    })
}

/// [`do_unlet`] of a terminated name.
///
/// # Safety
/// `name` points at `name_len` readable bytes and is NUL-terminated there.
unsafe fn unlet_terminated(
    name: *const c_char,
    name_len: size_t,
    forceit: bool,
) -> Result<(), Failed> {
    let mut dict: *mut Dict = ptr::null_mut();
    let (mut ht, varname) = unsafe { find_var_ht_dict(name, name_len, &raw mut dict) };
    let varname = name.wrapping_add(varname);

    if !ht.is_null() && unsafe { *varname } != NUL as c_char {
        // The dictionary whose lock decides whether the item may go.
        let mut d = get_current_funccal_dict(ht);
        if d.is_null() {
            if ht == get_globvar_ht() {
                d = get_globvar_dict();
            } else {
                // The scope's own dictionary item holds it.
                let di = unsafe { find_var_in_ht(ht, *name as c_int, c"".as_ptr(), 0, false) };
                d = unsafe { (*di).di_tv.dict_or_null() };
            }
            if d.is_null() {
                internal_error(c"do_unlet()");
                return Err(Failed);
            }
        }

        let found = unsafe { hash_find(ht, varname) };
        let hi = if found.is_kept() {
            Some(found)
        } else {
            unsafe { find_hi_in_scoped_ht(name, &raw mut ht) }
        };
        if let Some(hi) = hi.filter(|hi| hi.is_kept()) {
            // SAFETY: a kept item of a live variable hashtab.
            let di = unsafe { Di::new(tv_dict_hi2di(hi)) };
            let flags = di.di_flags as c_int;
            let (len, lock) = (TV_CSTRING as size_t, unsafe { (*d).dv_lock });
            if unsafe { var_check_fixed(flags, name, len) }
                || unsafe { var_check_ro(flags, name, len) }
                || value_check_lock(lock, LockName::Bytes(unsafe { cstr::bytes_at(name) }))
            {
                return Err(Failed);
            }
            // Upstream asks the same question a second time here. It can
            // only answer the same way -- nothing above it changes
            // `dv_lock` -- so the repetition is dead; kept because
            // deleting it is a change no gate could confirm.
            let name = LockName::Bytes(unsafe { cstr::bytes_at(name) });
            if value_check_lock(unsafe { (*d).dv_lock }, name) {
                return Err(Failed);
            }

            let mut oldtv = TV_INITIAL_VALUE;
            let watched = dict_is_watched(unsafe { (dict).as_ref() });
            if watched {
                let tv = di.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
                unsafe { tv_copy(&*tv, &mut oldtv) };
            }

            unsafe { delete_var(ht, hi) };

            if watched {
                unsafe {
                    dict_watcher_notify(
                        &::core::mem::ManuallyDrop::new(
                            DictRef::owning(dict).expect("a watched dictionary"),
                        ),
                        ::core::ffi::CStr::from_ptr(varname),
                        None,
                        Some(&oldtv),
                    )
                };
                clear_local(&mut oldtv);
            }
            return Ok(());
        }
    }

    if forceit {
        return Ok(());
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let name = unsafe { c_str(name) };
    semsg!("E108: No such variable: \"{name}\"");
    Err(Failed)
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
