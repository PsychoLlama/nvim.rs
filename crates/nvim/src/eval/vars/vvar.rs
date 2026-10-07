//! Reading and writing `v:` from C.
//!
//! Two families: the `get_vim_var_*` readers, which are how the rest of the
//! editor asks what a `v:` variable holds, and the `set_vim_var_*` writers,
//! which are how it publishes one.  [`before_set_vvar`] is the Vimscript
//! side of the same thing: the type enforcement `:let v:x = …` goes through.
//!
//! Every one of them indexes the `vimvars` table by [`Vv`], so none
//! of them can fail; the table's entries are `DictItem`-shaped and are the
//! same items `v:` the dictionary holds.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::eval::typval::DictRef;
use crate::eval::typval::ListRef;
use crate::eval::typval::PartialRef;
use crate::memory::ThinCString;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::eval::typval::DictEntry;
use crate::eval::typval::NumBuf;
use crate::eval::typval::tv_dict_free_contents;
use crate::types::{HashTab, SaveVEvent};

/// Row `i` of the `v:` table, for the walks that visit every one.
///
/// Total, and so a safe `fn`: the table is a `static` of `VIMVAR_COUNT`
/// rows, which outlives every caller.
pub(crate) fn vimvar_row(i: usize) -> Vvr {
    debug_assert!(i < VIMVAR_COUNT, "v: table has no row {i}");
    // SAFETY: a row of a live `static` table.
    unsafe { Vvr::new(vimvar_table().add(i)) }
}

/// The item a `v:` table row *is*: what the `v:` dictionary holds, rather
/// than a copy of it.
///
/// A raw pointer taken from the row's address, never through a borrow of
/// the row: the `v:` hashtab keeps this pointer, and a `&mut VimVar` taken
/// afterwards would invalidate it.
pub(crate) fn vimvar_row_item(row: Vvr) -> *mut DictItem {
    row.field_ptr(offset_of!(VimVar, vv_di))
}

/// The `v:` table row `idx` names.
///
/// Total for [`vimvar_row`]'s reason, with [`Vv`]'s discriminants being
/// exactly the rows of the table.
fn vimvar(idx: Vv) -> Vvr {
    vimvar_row(idx as usize)
}

/// The value of `v:` variable `idx`, without reading the row.
fn vimvar_val(idx: Vv) -> Tv {
    // SAFETY: a field of a live row is live, and `field_ptr` reads nothing.
    unsafe { Tv::new(vimvar(idx).field_ptr(offset_of!(VimVar, vv_di.di_tv))) }
}

/// Clear `v:` variable `idx`, freeing whatever it holds.
///
/// Safe: `tv_clear`'s only precondition is a live, writable value, and a row
/// of the `v:` table is one for the whole program.
pub(crate) fn clear_vimvar(idx: Vv) {
    // SAFETY: a row of a live `static` table.
    unsafe { tv_clear(&mut *vimvar_val(idx).raw()) };
}

/// The item of `v:` variable `idx`, as the hashtab holds it.
fn vimvar_item(idx: Vv) -> *mut DictItem {
    vimvar_row_item(vimvar(idx))
}

/// Save `v:` variable `idx` into `save_tv` and blank it, adding it to the
/// `v:` dictionary if it is one of the two that are not normally there.
///
/// Pairs with [`restore_vimvar`].
pub fn prepare_vimvar(idx: Vv, save_tv: &mut TypVal) {
    // Written through the row's *value* rather than through the row: the
    // `v:` hashtab keeps a pointer to `di_key`, which is a member of
    // `VimVar`, and a write through a borrow of the whole row would
    // invalidate it (see [`Live`]'s module docs). A `Live<TypVal>` borrows
    // only the value.
    let mut tv = vimvar_val(idx);
    // A take, not a write: the value moves to `save_tv` and the tag stays
    // behind, which is what the test below reads and what
    // [`restore_vimvar`] puts back.  Nothing is freed from under the copy.
    // SAFETY: the caller's obligation -- `save_tv` is writable.
    *save_tv = tv.take_value();
    if tv.v_type() == VAR_UNKNOWN {
        // `v:val` and `v:key` have no type until something sets one, and
        // are absent from the dictionary until then.
        // SAFETY: the `v:` hashtab, and a key that is the row's own.
        let _ = unsafe { hash_add(get_vimvar_ht(), DictEntry::new(vimvar_item(idx))) };
    }
}

/// Put back what [`prepare_vimvar`] saved.
pub fn restore_vimvar(idx: Vv, save_tv: &mut TypVal) {
    // Through the value, for [`prepare_vimvar`]'s reason.
    let mut tv = vimvar_val(idx);
    // SAFETY: the caller's obligation -- `save_tv` is the value the paired
    // `prepare_vimvar` filled.
    *tv = (*save_tv).take();
    if tv.v_type() != VAR_UNKNOWN {
        return;
    }
    // SAFETY: the `v:` hashtab and the row's own key; `hash_find` answers an
    // item of the table it was given.
    let hi = unsafe { hash_find(get_vimvar_ht(), (*vimvar_item(idx)).di_key.as_ptr()) };
    if hi.is_kept() {
        unsafe { hash_remove(get_vimvar_ht(), hi) };
    } else {
        internal_error(c"restore_vimvar()");
    }
}

/// Copy `tv` into `v:` variable `idx`.
pub fn set_vim_var_tv(idx: Vv, tv: &mut TypVal) {
    let out = vimvar_val(idx).raw();
    // SAFETY: a live `v:` value, and the caller's obligation for `tv`.
    unsafe { tv_clear(&mut *out) };
    unsafe { tv_copy(tv, &mut *out) };
}

/// The name of `v:` variable `idx`, without the `v:`.
///
/// The table's names are static literals, so the answer outlives everything.
pub(crate) fn get_vim_var_name(idx: Vv) -> &'static CStr {
    // SAFETY: every row's name is a NUL-terminated literal of the static
    // table, never written.
    unsafe { CStr::from_ptr(vimvar(idx).vv_name) }
}

/// Lend `v:` variable `idx`'s value to `f` to write through.
pub(crate) fn with_vim_var_mut<R>(idx: Vv, f: impl FnOnce(&mut TypVal) -> R) -> R {
    f(&mut vimvar_val(idx))
}

/// `v:` variable `idx` as a Number.  The caller knows its declared type.
pub(crate) fn get_vim_var_nr(idx: Vv) -> VarNumber {
    vimvar_val(idx).number_or_zero()
}

/// Another reference to the List `v:` variable `idx` holds, `None` when it
/// holds none.
pub(crate) fn get_vim_var_list_handle(idx: Vv) -> Option<ListRef> {
    vimvar_val(idx).list_handle()
}

/// Another reference to the Dict `v:` variable `idx` holds, `None` when it
/// holds none.
pub(crate) fn get_vim_var_dict_handle(idx: Vv) -> Option<DictRef> {
    vimvar_val(idx).dict_handle()
}

/// Lend `v:` variable `idx`'s string to `f`, with an unset one reading as
/// empty.
///
/// Every variable asked for here is declared `VAR_STRING` and `E963` refuses
/// an assignment of another type, so there is nothing to convert. The
/// borrow is the variable's own, so `f` must not run anything that can
/// assign a `v:` variable -- no autocommands, no `eval`, no `do_cmdline`;
/// a caller that holds the string across such a thing takes
/// [`vim_var_string`] or [`vim_var_bytes`] instead.
pub(crate) fn with_vim_var_str<R>(idx: Vv, f: impl FnOnce(&CStr) -> R) -> R {
    let tv = vimvar_val(idx);
    debug_assert_eq!(
        tv.v_type(),
        VAR_STRING,
        "v: variable {idx:?} is not a String"
    );
    f(tv.string_ref().map_or(c"", ThinCString::as_cstr))
}

/// A copy of `v:` variable `idx`'s string, `None` for the null string.
pub(crate) fn vim_var_string(idx: Vv) -> Option<ThinCString> {
    let tv = vimvar_val(idx);
    debug_assert_eq!(
        tv.v_type(),
        VAR_STRING,
        "v: variable {idx:?} is not a String"
    );
    tv.string_ref().cloned()
}

/// A reference of the caller's own to `v:lua`, the partial a `v:lua.name`
/// callee stands for.
pub(crate) fn lua_partial() -> Option<PartialRef> {
    vimvar_val(Vv::Lua).partial_shared().cloned()
}

/// Declare `v:` variable `idx` to be of type `type_0`, without touching its
/// value.
pub fn set_vim_var_type(idx: Vv, type_0: VarType) {
    let mut tv = vimvar_val(idx);
    tv.write_empty(type_0);
}

/// Set `v:` variable `idx` to the Number `val`.
pub fn set_vim_var_nr(idx: Vv, val: VarNumber) {
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    tv.write_number(val);
}

/// Set `v:` variable `idx` to `v:true` or `v:false`.
pub fn set_vim_var_bool(idx: Vv, val: BoolVarValue) {
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    tv.write_boolean(val);
}

/// Set `v:` variable `idx` to `v:null`.
pub fn set_vim_var_special(idx: Vv, val: SpecialVarValue) {
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    tv.write_special(val);
}

/// Set `v:char` to the character `c`.
pub fn set_vim_var_char(c: c_int) {
    let mut buf = [0u8; 7];
    // SAFETY: `utf_char2bytes` writes at most six bytes into the local.
    let buflen = unsafe { utf_char2bytes(c, buf.as_mut_ptr().cast()) };
    set_vim_var_string(Vv::Char, Some(&buf[..buflen as usize]));
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
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    tv.write_string(val);
}

/// Lend `v:` variable `idx`'s value to `f`.
pub(crate) fn with_vim_var<R>(idx: Vv, f: impl FnOnce(&TypVal) -> R) -> R {
    f(&vimvar_val(idx))
}

/// Set `v:` variable `idx` to `val`, which takes the handle over.
pub fn set_vim_var_list(idx: Vv, val: Option<ListRef>) {
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    tv.write_list(val);
}

/// Set `v:` variable `idx` to `val`, which takes the handle over, and make
/// its keys read-only.
pub fn set_vim_var_dict(idx: Vv, val: Option<DictRef>) {
    let mut tv = vimvar_val(idx);
    clear_vimvar(idx);
    let at = val
        .as_ref()
        .map_or(::core::ptr::null_mut(), DictRef::as_ptr);
    tv.write_dict(val);
    if at.is_null() {
        return;
    }
    // SAFETY: the caller's obligation -- a live dictionary.
    unsafe { (*at).set_keys_readonly() };
}

/// Set `v:lua`'s partial.
///
/// Upstream writes the union member without setting `v_type`, because the
/// table already declares `v:lua` a `VAR_PARTIAL` and nothing ever replaces
/// it; this runs once, from `evalvars_init`.
///
/// The slot takes `val` over.
pub(crate) fn set_vim_var_partial(idx: Vv, val: PartialRef) {
    vimvar_val(idx).write_partial(Some(val));
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
    let unchanged = vimvar_val(Vv::Register)
        .string_ref()
        .is_some_and(|cur| cur.first() == c as u8);
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
    let mut tv = vimvar_val(Vv::Cmdarg);

    let Some(command) = excmd else {
        drop(tv.take_string());
        tv.write_string(oldarg);
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
    let oldval = tv.take_string();
    tv.write_string(Some(ThinCString::from_bytes(&newval)));
    oldval
}

/// Set `v:count` and `v:count1`, and `v:prevcount` from the old `v:count`
/// first when asked.
pub(crate) fn set_vcount(count: int64_t, count1: int64_t, set_prevcount: bool) {
    if set_prevcount {
        let old = vimvar_val(Vv::Count).number_or_zero();
        let mut prev = vimvar_val(Vv::Prevcount);
        prev.write_number(old);
    }
    let (mut count_tv, mut count1_tv) = (vimvar_val(Vv::Count), vimvar_val(Vv::Count1));
    count_tv.write_number(count as VarNumber);
    count1_tv.write_number(count1 as VarNumber);
}

/// The type enforcement a write to a `v:` variable passes.
///
/// A `v:` variable keeps the type the table declares for it, so a String or
/// a Number one converts what it is given rather than replacing it -- and
/// two of them, `v:searchforward` and `v:hlsearch`, have a side effect on
/// the editor when they change.  Both of those cases do the store
/// themselves, notify the watchers and answer **false**: there is nothing
/// left for the caller to do.  Any other declared type accepts only a value
/// of the same type; a mismatch sets `type_error` (E963) and also answers
/// false.  True means "type checked out, store it the ordinary way".
///
/// # Safety
/// `varname` is the name without the `v:`, `di` its item in the `v:` table,
/// `tv` the value being stored and `type_error` writable.
pub unsafe fn before_set_vvar(
    varname: *const c_char,
    di: *mut DictItem,
    tv: &mut TypVal,
    copy: bool,
    watched: bool,
    type_error: *mut bool,
) -> bool {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's obligation -- `di` is an item of the `v:` scope
    // dictionary and `tv` the value being stored, both live for this call.
    // The item is reached through its *value*: `cur` points into the item,
    // so a write through a borrow of the whole item -- which `Live`'s
    // `DerefMut` hands out -- would invalidate the pointer the watcher
    // notification below is handed. See [`Live`]'s module docs.
    let cur: *mut TypVal = unsafe { Di::new(di) }.field_ptr(offset_of!(DictItem, di_tv));
    let (mut stored, mut tv) = unsafe { (Tv::new(cur), Tv::new(tv)) };
    if stored.v_type() == VAR_STRING {
        let mut oldtv = TV_INITIAL_VALUE;
        if watched {
            // SAFETY: a live value and a live local.
            unsafe { tv_copy(&*cur, &mut oldtv) };
        }
        drop(stored.take_string());

        if copy || tv.v_type() != VAR_STRING {
            // SAFETY: a live value; the answer lives in `numbuf` or in it.
            let val = numbuf.string(unsafe { &*tv.raw() });
            // Careful: assigning to v:errmsg, `tv_get_string()` may
            // itself raise an error, which sets the variable -- so only
            // store when it is still empty.
            if stored.string_ref().is_none() {
                stored.write_string(Some(ThinCString::from_cstr(val)));
            }
        } else {
            // Take the string over, rather than copy and free: the value
            // leaves `tv`, so the item now owns the only copy.
            let mut taken = tv.take_value();
            stored.write_string(taken.take_string());
        }
        if watched {
            // SAFETY: the `v:` dictionary, this item's value and a live local.
            let vv_dict = get_vimvar_dict();
            unsafe {
                dict_watcher_notify(
                    &::core::mem::ManuallyDrop::new(
                        DictRef::owning(vv_dict).expect("a watched dictionary"),
                    ),
                    ::core::ffi::CStr::from_ptr(varname),
                    Some(&*cur),
                    Some(&oldtv),
                )
            };
            clear_local(&mut oldtv);
        }
        return false;
    } else if stored.v_type() == VAR_NUMBER {
        let mut oldtv = TV_INITIAL_VALUE;
        if watched {
            // SAFETY: a live value and a live local.
            unsafe { tv_copy(&*cur, &mut oldtv) };
        }
        // SAFETY: a live value; the Number arm is what the tag declares.
        let n = unsafe { tv_get_number(&*tv.raw()) };
        stored.write_number(n);
        // SAFETY: the caller's obligation -- `varname` is NUL-terminated.
        if unsafe { cstr::eq_bytes(varname, b"searchforward") } {
            set_search_direction(if n != 0 { b'/' as c_int } else { b'?' as c_int });
        } else if unsafe { cstr::eq_bytes(varname, b"hlsearch") } {
            no_hlsearch.set(n == 0);
            redraw_all_later(UPD_SOME_VALID);
        }
        if watched {
            // SAFETY: the `v:` dictionary, this item's value and a live local.
            let vv_dict = get_vimvar_dict();
            unsafe {
                dict_watcher_notify(
                    &::core::mem::ManuallyDrop::new(
                        DictRef::owning(vv_dict).expect("a watched dictionary"),
                    ),
                    ::core::ffi::CStr::from_ptr(varname),
                    Some(&*cur),
                    Some(&oldtv),
                )
            };
            clear_local(&mut oldtv);
        }
        return false;
    } else if stored.v_type() != tv.v_type() {
        // SAFETY: the caller's obligation -- `type_error` is writable.
        unsafe { *type_error = true };
        return false;
    }
    true
}

/// [`set_vvar_item`] for the existing `v:` variable `key`, with `op` the
/// compound operator's byte. Nothing happens for a key `v:` does not have.
pub(crate) fn set_vvar_key(key: &[u8], tv: &mut TypVal, copy: bool, op: Option<u8>) {
    // SAFETY: the `v:` scope dictionary is a static.
    let item = unsafe { (*get_vimvar_dict()).find_ptr(key) };
    if item.is_null() {
        return;
    }
    let op = op.map(|op| [op.cast_signed(), 0]);
    let op = op.as_ref().map_or(ptr::null(), |op| op.as_ptr());
    // SAFETY: an item of the `v:` dictionary, and a terminated operator.
    unsafe { set_vvar_item(item, tv, copy, op) };
}

/// A write to a `v:` variable that reached the scope dictionary directly:
/// `let v:['name'] = value`.
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
///
/// # Safety
/// `di` is an item of the `v:` scope dictionary, `tv` the value being
/// assigned, and `op` NULL or the assignment's one-character operator.
pub(crate) unsafe fn set_vvar_item(
    di: *mut DictItem,
    tv: &mut TypVal,
    copy: bool,
    op: *const c_char,
) {
    // SAFETY: the caller's obligation -- `di` is an item of the `v:` scope
    // dictionary, live for this call.
    // As [`before_set_vvar`], the item is written through its value, so that
    // `cur` survives the store.
    let cur: *mut TypVal = unsafe { Di::new(di) }.field_ptr(offset_of!(DictItem, di_tv));
    // SAFETY: the caller's obligation, and the `v:` dictionary is a static.
    let varname = unsafe { (*di).di_key.as_ptr() };
    let watched = dict_is_watched(unsafe { (get_vimvar_dict()).as_ref() });

    // `+=` and friends act on the current value, so evaluate them into a
    // temporary first and enforce the type on the *result*.
    let mut tmp = TV_INITIAL_VALUE;
    // SAFETY: the caller's obligation -- `op` is NUL-terminated or NULL.
    let compound = !op.is_null() && unsafe { *op } != b'=' as c_char;
    let val = if compound {
        // SAFETY: this item's value, a live local, and the caller's `tv`.
        unsafe { tv_copy(&*cur, &mut tmp) };
        // SAFETY: as above -- a one-byte operator.
        if eexe_mod_op(&mut tmp, tv, unsafe { *op }.cast_unsigned()).is_err() {
            clear_local(&mut tmp);
            return;
        }
        &raw mut tmp
    } else {
        tv
    };

    let mut type_error = false;
    // The temporary is ours to free, so the store must copy out of it
    // rather than take its string.
    let copy_out = copy || compound;
    let err = &raw mut type_error;
    // SAFETY: the item and the value are live, and `type_error` is a local.
    let typed = unsafe { before_set_vvar(varname, di, &mut *val, copy_out, watched, err) };
    if !typed {
        if type_error {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let varname = unsafe { c_str(varname) };
            semsg!("E963: Setting v:{varname} to value with wrong type");
        }
        // SAFETY: a live local.
        clear_local(&mut tmp);
        return;
    }

    // The declared type matched: the ordinary store, as `set_var_const`
    // performs it.
    let mut oldtv = TV_INITIAL_VALUE;
    if watched {
        // SAFETY: this item's value and a live local.
        unsafe { tv_copy(&*cur, &mut oldtv) };
    }
    // SAFETY: this item's value, which the store below replaces.
    unsafe { tv_clear(&mut *cur) };
    // SAFETY: `val` is the caller's value or the local temporary.
    let val_type = unsafe { (*val).v_type() };
    if !compound && (copy || val_type == VAR_NUMBER || val_type == VAR_FLOAT) {
        // SAFETY: a live value and this item's own.
        unsafe { tv_copy(&*val, &mut *cur) };
    } else {
        // SAFETY: as above; the value is moved out and blanked.
        let mut cur = unsafe { Tv::new(cur) };
        *cur = unsafe { (*val).take() };
    }
    // As `set_var_const`: the value stored is unlocked, which with the lock
    // on the slot means this item.
    unsafe { *di_lock(di) = VarLock::Unlocked };
    if watched {
        // SAFETY: the `v:` dictionary, this item's value and a live local.
        let vv_dict = get_vimvar_dict();
        unsafe {
            dict_watcher_notify(
                &::core::mem::ManuallyDrop::new(
                    DictRef::owning(vv_dict).expect("a watched dictionary"),
                ),
                ::core::ffi::CStr::from_ptr(varname),
                Some(&*cur),
                Some(&oldtv),
            )
        };
        clear_local(&mut oldtv);
    }
    // SAFETY: a live local.
    clear_local(&mut tmp);
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
        sve.sve_hashtab = core::mem::replace(live, HashTab::init());
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
        v_event.edit().dv_hashtab = core::mem::take(&mut sve.sve_hashtab);
    }
}
