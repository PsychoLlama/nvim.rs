//! Reading and writing `v:` from C.
//!
//! Two families: the `get_vim_var_*` readers, which are how the rest of the
//! editor asks what a `v:` variable holds, and the `set_vim_var_*` writers,
//! which are how it publishes one.  [`before_set_vvar`] is the Vimscript
//! side of the same thing: the type enforcement `:let v:x = …` goes through.
//!
//! Every one of them names its row by [`Vv`], so none of them can fail. The
//! rows are items the `v:` dictionary owns, reached through the dictionary's
//! handle and found again by name on every access (a slot hint in
//! in [`scope_vim`] makes that one probe); `v:val` and `v:key`, while they
//! are not in the dictionary, sit in [`outside_vimvars`].
//!
//! A write takes the old value out in one borrow of the item and releases it
//! after the borrow has ended: a value can name the `v:` dictionary itself.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::eval::typval::DictRef;
use crate::eval::typval::ListRef;
use crate::eval::typval::PartialRef;
use crate::memory::ThinCString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};
use core::mem;

use super::*;
use crate::eval::typval::NumBuf;
use crate::eval::typval::tv_dict_free_contents;
use crate::types::{HashTab, SaveVEvent};

/// The `v:` dictionary, built on first use.
pub(crate) fn vimvar_dict() -> DictRef {
    match scope_vim.with(|vim| vim.entry.di_tv.dict_handle()) {
        Some(dict) => dict,
        None => build_vim_scope(),
    }
}

/// Build the `v:` scope: the dictionary and its rows, every one empty and of
/// its declared type. [`evalvars_init`] gives them their first values.
fn build_vim_scope() -> DictRef {
    let entry = new_unrooted_var_scope(VAR_SCOPE);
    let dict = entry
        .di_tv
        .dict_handle()
        .expect("a scope entry names its dictionary");
    dict.edit().dv_lock = VarLock::Fixed;
    for (i, row) in VIMVAR_ROWS.iter().enumerate() {
        let mut item = DictItem::boxed(row.name.to_bytes());
        item.di_flags |= if row.flags.has(VimVarFlags::RO) {
            DI_FLAGS_RO | DI_FLAGS_FIX
        } else if row.flags.has(VimVarFlags::RO_SBX) {
            DI_FLAGS_RO_SBX | DI_FLAGS_FIX
        } else {
            DI_FLAGS_FIX
        };
        item.di_tv = TypVal::empty(row.declared);
        // Into the `v:` scope dictionary -- unless the value is not always
        // available, which is what a `VAR_UNKNOWN` row means.
        if row.declared == VAR_UNKNOWN {
            let at = outside_index(Vv::try_from(i).expect("a row of the table"));
            outside_vimvars.with_mut(|outside| outside[at] = Some(item));
        } else if dict.edit().insert(item).is_err() {
            // The names are distinct by construction.
            unreachable!("v: has two rows named {:?}", row.name);
        }
    }
    scope_vim.with_mut(|vim| vim.entry = entry);
    dict
}

/// Where in [`outside_vimvars`] `v:val` and `v:key` wait.
fn outside_index(idx: Vv) -> usize {
    match idx {
        Vv::Val => 0,
        Vv::Key => 1,
        _ => unreachable!("only v:val and v:key leave the v: dictionary"),
    }
}

/// The slot row `idx` is in, in `dict` (the `v:` dictionary), or `None` when
/// it is not in the dictionary; learnt as the hint for the next access.
fn vimvar_slot(dict: &Dict, idx: Vv) -> Option<usize> {
    let slot = dict.slot_of(VIMVAR_ROWS[idx as usize].name.to_bytes())?;
    let addr = dict
        .item_at(slot)
        .map_or(0, |item| ::core::ptr::from_ref(item).addr());
    scope_vim.with_mut(|vim| vim.slots[idx as usize] = (slot, addr));
    Some(slot)
}

/// Run `f` over row `idx`'s item where the hint says it is, in one access to
/// the record; `None` (with `f` left in place) when the hint is stale or the
/// scope not built yet.
///
/// `f` runs inside that access, so it must be a leaf: it must not reach a
/// `v:` variable (nor anything else of the record) or release a value.
#[inline(always)]
fn vimvar_at_hint<R, F: FnOnce(&mut DictItem) -> R>(idx: Vv, f: &mut Option<F>) -> Option<R> {
    scope_vim.with(|vim| {
        let (hint, addr) = vim.slots[idx as usize];
        let dict = vim.entry.di_tv.dict_shared()?;
        let item = dict.edit().item_at_mut(hint)?;
        if ::core::ptr::from_ref(item).addr() != addr {
            return None;
        }
        f.take().map(|f| f(item))
    })
}

/// Lend row `idx`'s item to `f` to read. `f` must be a leaf, as
/// [`vimvar_at_hint`]'s.
#[inline]
fn with_vimvar_ref<R>(idx: Vv, f: impl FnOnce(&DictItem) -> R) -> R {
    with_vimvar_item(idx, |item| f(item))
}

/// Lend row `idx`'s item to `f` to write. `f` must be a leaf, as
/// [`vimvar_at_hint`]'s: what it replaces it answers, for the caller to
/// release after the borrow.
#[inline]
fn with_vimvar_item<R>(idx: Vv, f: impl FnOnce(&mut DictItem) -> R) -> R {
    let mut f = Some(f);
    if let Some(answer) = vimvar_at_hint(idx, &mut f) {
        return answer;
    }
    let f = f.expect("the hint did not run it");
    with_vimvar_found(idx, f)
}

/// [`with_vimvar_item`] past a stale hint: the dictionary held by a handle
/// of this frame's and the row found by name, so `f` runs with no access to
/// the record open.
#[cold]
fn with_vimvar_found<R>(idx: Vv, f: impl FnOnce(&mut DictItem) -> R) -> R {
    let dict = vimvar_dict();
    match vimvar_slot(&dict, idx) {
        Some(slot) => f(dict.edit().item_at_mut(slot).expect("a kept slot")),
        None => outside_vimvars.with_mut(|outside| {
            f(outside[outside_index(idx)]
                .as_mut()
                .expect("a row out of v: waits outside it"))
        }),
    }
}

/// Replace `v:` variable `idx`'s value with `value`, releasing the old one.
fn replace_vimvar(idx: Vv, value: TypVal) {
    let old = with_vimvar_item(idx, |item| mem::replace(&mut item.di_tv, value));
    drop(old);
}

/// Clear `v:` variable `idx`, freeing whatever it holds.
pub(crate) fn clear_vimvar(idx: Vv) {
    let old = with_vimvar_item(idx, |item| item.di_tv.take_value());
    drop(old);
}

/// Save `v:` variable `idx` into `save_tv` and blank it, adding it to the
/// `v:` dictionary if it is one of the two that are not normally there.
///
/// Pairs with [`restore_vimvar`].
pub fn prepare_vimvar(idx: Vv, save_tv: &mut TypVal) {
    // A take, not a write: the value moves to `save_tv` and the tag stays
    // behind, which is what the test below reads and what
    // [`restore_vimvar`] puts back.
    let (saved, untyped) = with_vimvar_item(idx, |item| {
        let saved = item.di_tv.take_value();
        (saved, item.di_tv.v_type() == VAR_UNKNOWN)
    });
    *save_tv = saved;
    if untyped {
        // `v:val` and `v:key` have no type until something sets one, and
        // are absent from the dictionary until then.
        let at = outside_index(idx);
        if let Some(item) = outside_vimvars.with_mut(|outside| outside[at].take())
            && let Err(item) = vimvar_dict().edit().insert(item)
        {
            outside_vimvars.with_mut(|outside| outside[at] = Some(item));
        }
    }
}

/// Put back what [`prepare_vimvar`] saved.
pub fn restore_vimvar(idx: Vv, save_tv: &mut TypVal) {
    let saved = save_tv.take();
    let (old, untyped) = with_vimvar_item(idx, |item| {
        let old = mem::replace(&mut item.di_tv, saved);
        (old, item.di_tv.v_type() == VAR_UNKNOWN)
    });
    drop(old);
    if !untyped {
        return;
    }
    let name = VIMVAR_ROWS[idx as usize].name.to_bytes();
    let dict = vimvar_dict();
    let removed = dict.edit().remove_key(name);
    match removed {
        Some(RemovedItem::Allocated(item)) => {
            outside_vimvars.with_mut(|outside| outside[outside_index(idx)] = Some(item));
        }
        Some(RemovedItem::Embedded(_)) => unreachable!("v: owns its rows"),
        None => internal_error(c"restore_vimvar()"),
    }
}

/// Copy `tv` into `v:` variable `idx`.
pub fn set_vim_var_tv(idx: Vv, tv: &mut TypVal) {
    let mut copy = TV_INITIAL_VALUE;
    tv_copy(tv, &mut copy);
    replace_vimvar(idx, copy);
}

/// The name of `v:` variable `idx`, without the `v:`.
pub(crate) fn get_vim_var_name(idx: Vv) -> &'static CStr {
    VIMVAR_ROWS[idx as usize].name
}

/// Lend `v:` variable `idx`'s value to `f` to write through. `f` must be a
/// leaf: it must not reach a `v:` variable, nor release a value.
pub(crate) fn with_vim_var_mut<R>(idx: Vv, f: impl FnOnce(&mut TypVal) -> R) -> R {
    with_vimvar_item(idx, |item| f(&mut item.di_tv))
}

/// `v:` variable `idx` as a Number.  The caller knows its declared type.
pub(crate) fn get_vim_var_nr(idx: Vv) -> VarNumber {
    with_vimvar_ref(idx, |item| item.di_tv.number_or_zero())
}

/// Another reference to the List `v:` variable `idx` holds, `None` when it
/// holds none.
pub(crate) fn get_vim_var_list_handle(idx: Vv) -> Option<ListRef> {
    with_vimvar_ref(idx, |item| item.di_tv.list_handle())
}

/// Another reference to the Dict `v:` variable `idx` holds, `None` when it
/// holds none.
pub(crate) fn get_vim_var_dict_handle(idx: Vv) -> Option<DictRef> {
    with_vimvar_ref(idx, |item| item.di_tv.dict_handle())
}

/// Lend `v:` variable `idx`'s string to `f`, with an unset one reading as
/// empty.
///
/// Every variable asked for here is declared `VAR_STRING` and `E963` refuses
/// an assignment of another type, so there is nothing to convert. `f` sees
/// a copy, so it may do anything; a caller that holds the string across
/// user code takes [`vim_var_string`] or [`vim_var_bytes`] all the same.
pub(crate) fn with_vim_var_str<R>(idx: Vv, f: impl FnOnce(&CStr) -> R) -> R {
    let value = vim_var_string(idx);
    f(value.as_ref().map_or(c"", ThinCString::as_cstr))
}

/// A copy of `v:` variable `idx`'s string, `None` for the null string.
pub(crate) fn vim_var_string(idx: Vv) -> Option<ThinCString> {
    with_vimvar_ref(idx, |item| {
        let tv = &item.di_tv;
        debug_assert_eq!(
            tv.v_type(),
            VAR_STRING,
            "v: variable {idx:?} is not a String"
        );
        tv.string_ref().cloned()
    })
}

/// A reference of the caller's own to `v:lua`, the partial a `v:lua.name`
/// callee stands for.
pub(crate) fn lua_partial() -> Option<PartialRef> {
    with_vimvar_ref(Vv::Lua, |item| item.di_tv.partial_shared().cloned())
}

/// Whether `v:testing` is set: [`get_vim_var_nr`]`(Vv::Testing) != 0`
/// without the dictionary.
#[inline]
pub(crate) fn testing_enabled() -> bool {
    let enabled = vim_testing.get();
    debug_assert_eq!(enabled, get_vim_var_nr(Vv::Testing) != 0);
    enabled
}

/// Whether the partial at address `partial` is `v:lua`'s. A comparison of
/// addresses; nothing is read.
#[inline]
pub(crate) fn is_lua_partial(partial: usize) -> bool {
    let lua = lua_partial_addr.get();
    lua != 0 && partial == lua
}

/// Declare `v:` variable `idx` to be of type `type_0`, without touching its
/// value.
pub fn set_vim_var_type(idx: Vv, type_0: VarType) {
    with_vimvar_item(idx, |item| item.di_tv.write_empty(type_0));
}

/// Set `v:` variable `idx` to the Number `val`.
pub fn set_vim_var_nr(idx: Vv, val: VarNumber) {
    if idx == Vv::Testing {
        vim_testing.set(val != 0);
    }
    // A Number over a Number has nothing to release: written in place.
    let old = with_vimvar_item(idx, |item| match &mut item.di_tv {
        TypVal::Number(n) => {
            *n = val;
            None
        }
        tv => Some(mem::replace(tv, TypVal::Number(val))),
    });
    drop(old);
}

/// Set `v:` variable `idx` to `v:true` or `v:false`.
pub fn set_vim_var_bool(idx: Vv, val: BoolVarValue) {
    replace_vimvar(idx, TypVal::Bool(val));
}

/// Set `v:` variable `idx` to `v:null`.
pub fn set_vim_var_special(idx: Vv, val: SpecialVarValue) {
    replace_vimvar(idx, TypVal::Special(val));
}

/// Set `v:char` to the character `c`.
pub fn set_vim_var_char(c: c_int) {
    let mut buf = [0u8; 7];
    let buflen = crate::mbyte::encode_char(c, &mut buf);
    set_vim_var_string(Vv::Char, Some(&buf[..buflen]));
}

/// Set `v:` variable `idx` to a copy of `val`; `None` is the null string.
///
/// A NUL among the bytes ends the string for every reader, as `xstrndup`
/// did.
pub fn set_vim_var_string(idx: Vv, val: Option<&[u8]>) {
    set_vim_var_owned(idx, val.map(ThinCString::from_bytes));
}

/// Set `v:` variable `idx` to `val`, which it takes over; `None` is the null
/// string.
pub(crate) fn set_vim_var_owned(idx: Vv, val: Option<ThinCString>) {
    replace_vimvar(idx, TypVal::string(val));
}

/// Lend `v:` variable `idx`'s value to `f`, which must be a leaf: it must
/// not reach a `v:` variable. A caller with more to do copies the value out.
pub(crate) fn with_vim_var<R>(idx: Vv, f: impl FnOnce(&TypVal) -> R) -> R {
    with_vimvar_ref(idx, |item| f(&item.di_tv))
}

/// Set `v:` variable `idx` to `val`, which takes the handle over.
pub fn set_vim_var_list(idx: Vv, val: Option<ListRef>) {
    replace_vimvar(idx, TypVal::list(val));
}

/// Set `v:` variable `idx` to `val`, which takes the handle over, and make
/// its keys read-only.
pub fn set_vim_var_dict(idx: Vv, val: Option<DictRef>) {
    if let Some(dict) = &val {
        dict.edit().set_keys_readonly();
    }
    replace_vimvar(idx, TypVal::dict(val));
}

/// Set `v:lua`'s partial.
///
/// Upstream writes the union member without setting `v_type`, because the
/// table already declares `v:lua` a `VAR_PARTIAL` and nothing ever replaces
/// it; this runs once, from `evalvars_init`.
///
/// The slot takes `val` over.
pub(crate) fn set_vim_var_partial(idx: Vv, val: PartialRef) {
    if idx == Vv::Lua {
        lua_partial_addr.set(val.as_ptr().addr());
    }
    replace_vimvar(idx, TypVal::partial(Some(val)));
}

/// Set `v:register` to `c`, or to `"` for the unnamed register.
pub fn set_reg_var(c: c_int) {
    let regname = if c == 0 || c == b' ' as c_int {
        b'"' as c_char
    } else {
        c as c_char
    };
    // Only write when it changed, to avoid the reallocation. The test
    // is against `c`, not against the name that would be stored, so
    // `set_reg_var(0)` always rewrites -- upstream's.
    let unchanged = with_vimvar_ref(Vv::Register, |item| {
        item.di_tv
            .string_ref()
            .is_some_and(|cur| cur.first() == c as u8)
    });
    if !unchanged {
        set_vim_var_string(Vv::Register, Some(&[regname as u8]));
    }
}

/// Set `v:cmdarg` to the `++opt` arguments of `excmd`, answering the old value
/// for the caller to restore.
///
/// A `None` `excmd` is the restore half: `oldarg` goes back and the value
/// that was there is freed, answering `None` -- there is nothing left for the
/// caller to put back.
pub fn set_cmdarg(excmd: Option<&ExArg>, oldarg: Option<ThinCString>) -> Option<ThinCString> {
    let Some(command) = excmd else {
        let old = with_vimvar_item(Vv::Cmdarg, |item| {
            let old = item.di_tv.take_string();
            item.di_tv.write_string(oldarg);
            old
        });
        drop(old);
        return None;
    };
    let mut newval: Vec<u8> = Vec::new();
    if command.force_bin == FORCE_BIN {
        newval.extend_from_slice(b" ++bin");
    } else if command.force_bin == FORCE_NOBIN {
        newval.extend_from_slice(b" ++nobin");
    }
    if command.read_edit {
        newval.extend_from_slice(b" ++edit");
    }
    if command.force_ff != 0 {
        let ff: &[u8] = match command.force_ff as u8 {
            b'u' => b"unix",
            b'd' => b"dos",
            _ => b"mac",
        };
        newval.extend_from_slice(b" ++ff=");
        newval.extend_from_slice(ff);
    }
    if command.force_enc != 0 {
        // The encoding name lives inside the command line the `++enc=` was
        // parsed out of, at the offset `force_enc` records from the command.
        newval.extend_from_slice(b" ++enc=");
        newval.extend_from_slice(
            command
                .line
                .rest_of(command.line.cmd + command.force_enc as usize),
        );
    }
    if command.bad_char == BAD_KEEP {
        newval.extend_from_slice(b" ++bad=keep");
    } else if command.bad_char == BAD_DROP {
        newval.extend_from_slice(b" ++bad=drop");
    } else if command.bad_char != 0 {
        newval.extend_from_slice(b" ++bad=");
        newval.push(command.bad_char as u8);
    }
    if command.mkdir_p {
        newval.extend_from_slice(b" ++p");
    }

    // The old value goes to the caller, to put back later.
    with_vimvar_item(Vv::Cmdarg, |item| {
        let oldval = item.di_tv.take_string();
        item.di_tv
            .write_string(Some(ThinCString::from_bytes(&newval)));
        oldval
    })
}

/// Set `v:count` and `v:count1`, and `v:prevcount` from the old `v:count`
/// first when asked.
pub(crate) fn set_vcount(count: int64_t, count1: int64_t, set_prevcount: bool) {
    if set_prevcount {
        let old = get_vim_var_nr(Vv::Count);
        with_vimvar_item(Vv::Prevcount, |item| item.di_tv.write_number(old));
    }
    with_vimvar_item(Vv::Count, |item| {
        item.di_tv.write_number(count as VarNumber)
    });
    with_vimvar_item(Vv::Count1, |item| {
        item.di_tv.write_number(count1 as VarNumber);
    });
}

/// Notify the `v:` dictionary's watchers that `key` changed from `old` to
/// what it holds now.
fn notify_vvar_watchers(key: &[u8], old: &TypVal) {
    let dict = vimvar_dict();
    let new = dict.find(key).map(|item| item.di_tv.clone());
    cstr::with_terminated(key, |key| {
        dict_watcher_notify(&dict, key, new.as_ref(), Some(old));
    });
}

/// Why [`before_set_vvar`] did not leave the store to its caller.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum VvarStore {
    /// The type checked out: store the value the ordinary way.
    Store,
    /// Done: the variable converted the value and stored it itself.
    Done,
    /// The value's type is not the declared one (E963, the caller's to
    /// report).
    TypeError,
}

/// The type enforcement a write to the `v:` variable `key` passes.
///
/// A `v:` variable keeps the type the table declares for it, so a String or
/// a Number one converts what it is given rather than replacing it -- and
/// two of them, `v:searchforward` and `v:hlsearch`, have a side effect on
/// the editor when they change.  Both of those cases do the store
/// themselves, notify the watchers and answer [`VvarStore::Done`]. Any other
/// declared type accepts only a value of the same type.
///
/// `key` must name a variable of the `v:` dictionary.
pub(crate) fn before_set_vvar(key: &[u8], tv: &mut TypVal, copy: bool, watched: bool) -> VvarStore {
    let dict = vimvar_dict();
    let current = |dict: &DictRef| {
        dict.find(key)
            .map_or(VAR_UNKNOWN, |item| item.di_tv.v_type())
    };
    let stored_type = current(&dict);
    if stored_type == VAR_STRING {
        // The old value, for the watchers, and the string taken out.
        let (old, taken) = match dict.edit().find_mut(key) {
            Some(item) => (
                watched.then(|| item.di_tv.clone()),
                item.di_tv.take_string(),
            ),
            None => (None, None),
        };
        drop(taken);

        if copy || tv.v_type() != VAR_STRING {
            let mut numbuf = NumBuf::new();
            // Careful: assigning to v:errmsg, `tv_get_string()` may
            // itself raise an error, which sets the variable -- so only
            // store when it is still empty.
            let val = numbuf.string(tv);
            let val = ThinCString::from_cstr(val);
            if let Some(item) = dict.edit().find_mut(key)
                && item.di_tv.string_ref().is_none()
            {
                item.di_tv.write_string(Some(val));
            }
        } else {
            // Take the string over, rather than copy and free: the value
            // leaves `tv`, so the item now owns the only copy.
            let mut taken = tv.take_value();
            let string = taken.take_string();
            if let Some(item) = dict.edit().find_mut(key) {
                item.di_tv.write_string(string);
            }
        }
        if let Some(old) = old {
            notify_vvar_watchers(key, &old);
        }
        return VvarStore::Done;
    } else if stored_type == VAR_NUMBER {
        let old = watched
            .then(|| dict.find(key).map(|item| item.di_tv.clone()))
            .flatten();
        let n = tv_get_number(tv);
        if let Some(item) = dict.edit().find_mut(key) {
            item.di_tv.write_number(n);
        }
        if key == b"searchforward" {
            set_search_direction(if n != 0 { b'/' as c_int } else { b'?' as c_int });
        } else if key == b"hlsearch" {
            no_hlsearch.set(n == 0);
            redraw_all_later(UPD_SOME_VALID);
        } else if key == b"testing" {
            vim_testing.set(n != 0);
        }
        if let Some(old) = old {
            notify_vvar_watchers(key, &old);
        }
        return VvarStore::Done;
    } else if stored_type != tv.v_type() {
        return VvarStore::TypeError;
    }
    VvarStore::Store
}

/// A write to the existing `v:` variable `key` that reached the scope
/// dictionary directly: `let v:['name'] = value`, with `op` the compound
/// operator's byte. Nothing happens for a key `v:` does not have.
///
/// The subscripted spelling makes `get_lval` resolve `v:` to a plain
/// `Dict` and `set_var_lval` store straight into the `DictItem`, so
/// upstream never runs [`before_set_vvar`] for it and the declared type of
/// the variable is simply replaced.  That is a crash and not only a
/// surprise: `get_vim_var_list(Vv::Oldfiles)` reads `vval.v_list` with no
/// type test, so `let v:['oldfiles'] = 1` followed by `:oldfiles`
/// dereferences the address 1.  See docket O-B14-10.
///
/// This does the same work `set_var_const` does for the unsubscripted
/// spelling, including the compound operators -- which `set_whole_var`
/// applies to a *copy* of the current value before handing the result to
/// `set_var_const`, so that `let v:searchforward .= 'x'` converts back to a
/// Number rather than replacing one.
pub(crate) fn set_vvar_key(key: &[u8], tv: &mut TypVal, copy: bool, op: Option<u8>) {
    let dict = vimvar_dict();
    if dict.find(key).is_none() {
        return;
    }
    let watched = dict_is_watched(Some(&dict));

    // `+=` and friends act on the current value, so evaluate them into a
    // temporary first and enforce the type on the *result*.
    let compound = op.is_some_and(|op| op != b'=');
    let mut tmp = TV_INITIAL_VALUE;
    let val: &mut TypVal = match op.filter(|_| compound) {
        Some(op) => {
            if let Some(item) = dict.find(key) {
                tmp = item.di_tv.clone();
            }
            if eexe_mod_op(&mut tmp, tv, op).is_err() {
                clear_local(&mut tmp);
                return;
            }
            &mut tmp
        }
        None => tv,
    };

    // The temporary is ours to free, so the store must copy out of it
    // rather than take its string.
    let copy_out = copy || compound;
    match before_set_vvar(key, val, copy_out, watched) {
        VvarStore::Store => {}
        VvarStore::Done => return,
        VvarStore::TypeError => {
            let varname = msg_bytes(key);
            semsg!("E963: Setting v:{varname} to value with wrong type");
            return;
        }
    }

    // The declared type matched: the ordinary store, as `set_var_const`
    // performs it.
    let val_type = val.v_type();
    let new = if !compound && (copy || val_type == VAR_NUMBER || val_type == VAR_FLOAT) {
        val.clone()
    } else {
        val.take()
    };
    let Some(old) = dict.edit().find_mut(key).map(|item| {
        // As `set_var_const`: the value stored is unlocked, which with the
        // lock on the slot means this item.
        item.di_lock = VarLock::Unlocked;
        mem::replace(&mut item.di_tv, new)
    }) else {
        return;
    };
    if watched {
        notify_vvar_watchers(key, &old);
    }
    drop(old);
}

/// Blank the six `v:option_*` variables the `OptionSet` autocommand reads.
pub fn reset_v_option_vars() {
    for idx in [
        Vv::OptionNew,
        Vv::OptionOld,
        Vv::OptionOldlocal,
        Vv::OptionOldglobal,
        Vv::OptionCommand,
        Vv::OptionType,
    ] {
        set_vim_var_string(idx, None);
    }
}

/// [`with_vim_var_str`] as owned bytes, an unset string reading as empty.
///
/// The bytes are copied because the variable can be assigned to while the
/// caller holds them.
pub fn vim_var_bytes(idx: Vv) -> Vec<u8> {
    with_vim_var_str(idx, |s| s.to_bytes().to_vec())
}

/// Reserve `v:event` for the duration of one autocommand, saving whatever
/// a surrounding one had put there into `sve`.
///
/// Answers a reference of the caller's own to `v:event`, which goes back to
/// [`restore_v_event`] with the same `sve`.
pub(crate) fn get_v_event(sve: &mut SaveVEvent) -> DictRef {
    let v_event = get_vim_var_dict_handle(Vv::Event).expect("v:event is always a Dict");
    let live = &mut v_event.edit().dv_hashtab;
    let did_save = live.ht_used > 0;
    sve.sve_did_save = did_save;
    if did_save {
        // A plain move: the table owns its slots, so what the surrounding
        // autocommand put in `v:event` travels to `sve` intact and
        // `v:event` starts the inner one empty. `restore_v_event` moves it
        // back.
        sve.sve_hashtab = mem::replace(live, HashTab::init());
    }
    v_event
}

/// Put back what `get_v_event` saved.
///
/// `v_event` and `sve` are the pair [`get_v_event`] produced.
pub(crate) fn restore_v_event(v_event: DictRef, sve: &mut SaveVEvent) {
    tv_dict_free_contents(&v_event);
    // `tv_dict_free_contents` already left `v:event` with a fresh empty
    // table, so the not-saved case has nothing left to do.
    if sve.sve_did_save {
        // The move back. `sve` is left with a table that owns nothing,
        // which is what its `Default` is.
        v_event.edit().dv_hashtab = mem::take(&mut sve.sve_hashtab);
    }
}
