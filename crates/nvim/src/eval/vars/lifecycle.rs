//! Creating, clearing and collecting the variable dictionaries.
//!
//! [`evalvars_init`] builds `g:` and `v:` and every entry of the `v:` table;
//! the rest tear one down again -- a script's `s:` scope, a window's `w:`,
//! the whole of `g:` at exit -- and hand the garbage collector its roots.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::memory::ThinCString;
use core::ffi::c_int;

use super::*;
use crate::eval::typval::{DictRef, ListRef};
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::Failed;
use crate::types::MessagePackType;
use crate::types::{DictKey, PartialRef, Refcount};

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

/// Set the variable `name` to a copy of the string `value`; `None` is the
/// null string.
pub(crate) fn set_internal_string_var(name: &[u8], value: Option<&[u8]>) {
    let mut tv = TypVal::string(value.map(ThinCString::from_bytes));
    set_var(name, &mut tv, false);
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

/// Whether `dict` is the `g:` dictionary.
pub(crate) fn is_globvar_dict(dict: &Dict) -> bool {
    scope_globals.with(|entry| {
        entry
            .di_tv
            .dict_shared()
            .is_some_and(|globals| ::core::ptr::eq(globals.as_ptr().cast_const(), dict))
    })
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
    if let Some(home) = find_var_home(name).filter(|home| home.name_at < name.len())
        && let Some(Located::Item { dict, slot }) = locate_in(&home, name, true)
    {
        let varname = &name[home.name_at..];
        // The dictionary whose lock decides whether the item may go is the
        // scope's -- `v:` for a compat name -- even when the item was found
        // in a scope a lambda closed over; and only a scope of its own is
        // watched.
        let lock_dict = &home.dict;
        let watched = home.kind != ScopeKind::Compat && dict_is_watched(Some(lock_dict));

        let flags = dict
            .item_at(slot)
            .map_or(0, |item| c_int::from(item.di_flags));
        if var_check_fixed_named(flags, name)
            || var_check_ro_named(flags, name)
            || value_check_lock(lock_dict.dv_lock, LockName::Bytes(name))
        {
            return Err(Failed);
        }
        // Upstream asks the same question a second time here. It can only
        // answer the same way -- nothing above it changes `dv_lock` -- so
        // the repetition is dead; kept because deleting it is a change no
        // gate could confirm.
        if value_check_lock(lock_dict.dv_lock, LockName::Bytes(name)) {
            return Err(Failed);
        }

        let old = watched
            .then(|| dict.item_at(slot).map(|item| item.di_tv.clone()))
            .flatten();
        // Released with the borrow of the dictionary over: a value can name
        // the dictionary it was in.
        let removed = dict.edit().remove_at(slot);
        drop(removed);

        if let Some(old) = old {
            cstr::with_terminated(varname, |key| {
                dict_watcher_notify(lock_dict, key, None, Some(&old));
            });
        }
        return Ok(());
    }

    if forceit {
        return Ok(());
    }
    let name = msg_bytes(name);
    semsg!("E108: No such variable: \"{name}\"");
    Err(Failed)
}
